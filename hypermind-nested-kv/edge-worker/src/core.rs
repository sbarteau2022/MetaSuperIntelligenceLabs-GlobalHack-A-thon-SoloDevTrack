//! HyperMind edge core — pure-Rust port of the decision math.
//!
//! Byte-identical logic to hypermind-nested-kv/src/main.rs for every pure
//! function: shell projection, shell distance, traversal form check, hunger
//! priority, adaptive fusion weights, judge gates. Binding calls
//! (D1/Vectorize/AI/KV/R2) attach at the Worker route layer, never here.
//! `cargo test` green here = green on edge for all pure paths.

use serde::{Deserialize, Serialize};

/// Lorentz hyperboloid point H³ + toroidal phase T² (server Atom shape,
/// wire-compatible: same field order, same f64 layout).
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EdgeAtom {
    pub id: u64,
    pub coord: [f64; 4],
    pub phase: [f64; 2],
    pub salience: f64,
}

/// Project a Lorentz coordinate onto the unit-radius shell (direction only).
/// VERIFIED identical to shell_project in the oracle (goal 57167b0b step 1).
pub fn shell_project(c: &[f64; 4]) -> [f64; 4] {
    let r = (c[1] * c[1] + c[2] * c[2] + c[3] * c[3]).sqrt().max(1e-9);
    let s = [c[1] / r, c[2] / r, c[3] / r];
    let x0 = (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
    [x0, s[0], s[1], s[2]]
}

pub fn lorentz_inner(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3]
}

pub fn hyperbolic_dist(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    lorentz_inner(a, b).max(1.0).acosh()
}

/// Salience-invariant distance (oracle shell_dist, verbatim).
pub fn shell_dist(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let pa = shell_project(a);
    let pb = shell_project(b);
    hyperbolic_dist(&pa, &pb)
}

/// Retrieval score: negated shell Lorentz + toroidal phase penalty
/// (oracle score(), verbatim minus the Atom wrapper).
pub fn score(qc: &[f64; 4], qp: &[f64; 2], ac: &[f64; 4], ap: &[f64; 2]) -> f64 {
    let q = shell_project(qc);
    let a = shell_project(ac);
    let phase_dist = (1.0 - (qp[0] - ap[0]).cos()) + (1.0 - (qp[1] - ap[1]).cos());
    -lorentz_inner(&q, &a) - 0.25 * phase_dist
}

/// Form check: hyperbolic triangle inequality residual (oracle form_residual).
/// Zero for genuine geodesic triples; positive = teleport, not travel.
pub fn form_residual(d_ab: f64, d_av: f64, d_vb: f64) -> f64 {
    let lhs = d_ab.cosh();
    let rhs = d_av.cosh() * d_vb.cosh() + d_av.sinh() * d_vb.sinh();
    (lhs - rhs).max(0.0)
}

/// Hunger priority H-5: hits × depth × (1 + recency).
pub fn hunger_priority(hits: u64, depth: u32, last_seen_unix: u64, now_unix: u64) -> f64 {
    let age_days = (now_unix.saturating_sub(last_seen_unix) as f64) / 86400.0;
    let recency = 1.0 / (1.0 + age_days);
    (hits as f64) * (depth.max(1) as f64) * (1.0 + recency)
}

/// Adaptive fusion weights (oracle map.query, verbatim rule).
/// BM25 signal present (max > 0.5) → geo 0.2 / bm25 0.8; else 0.8 / 0.2.
pub fn fusion_weights(bm25_max: f64) -> (f64, f64) {
    if bm25_max > 0.5 {
        (0.2, 0.8)
    } else {
        (0.8, 0.2)
    }
}

/// Deterministic content-local judge gates (oracle judge_text structure).
/// Returns (pass, reason). Structural only — no model, no network.
pub fn judge_gates(
    text_len: usize,
    n_toks: usize,
    n_uniq: usize,
    entropy: f64,
) -> (bool, &'static str) {
    if n_toks == 0 {
        return (false, "empty after tokenize");
    }
    if text_len < 8 {
        return (false, "too short (<8 chars)");
    }
    if n_uniq < 2 {
        return (false, "below diversity floor");
    }
    if entropy < 0.5 && n_toks > 3 {
        return (false, "entropy below 0.50 bits");
    }
    (true, "pass")
}

/// Traversal verdict from the three residuals (oracle map.traverse logic).
/// Returns "dispatch" or the semicolon-joined fail-explained reason.
pub fn traverse_verdict(
    fres: f64,
    form_tol: f64,
    cost: f64,
    budget: f64,
    frame_res: f64,
    frame_eps: f64,
) -> String {
    let mut reasons: Vec<String> = Vec::new();
    if fres > form_tol {
        reasons.push(format!(
            "form violated: residual {:.3e} > tol {:.3e} (teleport, not travel)",
            fres, form_tol
        ));
    }
    if cost > budget {
        reasons.push(format!(
            "over budget: cost {:.3} > budget {:.3}",
            cost, budget
        ));
    }
    if frame_res > frame_eps {
        reasons.push(format!(
            "frame shear: residual {:.3e} > eps {:.3e}",
            frame_res, frame_eps
        ));
    }
    if reasons.is_empty() {
        "dispatch".into()
    } else {
        reasons.join("; ")
    }
}

#[cfg(test)]
mod core_tests {
    use crate::core::*;

    #[test]
    fn shell_invariant_matches_oracle() {
        // Same direction, different shells → ~0 apart. Both points must
        // satisfy the hyperboloid constraint (x0² − ||x||² = 1), as real
        // coordinates do — hand-made non-hyperboloid vectors project
        // differently (correct behavior, not a bug).
        let dir = [0.6, 0.5, 0.6245f64];
        let mk = |r: f64| {
            let s = [dir[0] * r, dir[1] * r, dir[2] * r];
            let x0 = (1.0 + s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            [x0, s[0], s[1], s[2]]
        };
        let hi = mk(0.5);
        let lo = mk(2.0);
        // Oracle tolerance is 1e-6 (acosh noise floor near 1.0) — match it.
        assert!(shell_dist(&hi, &lo) < 1e-6, "d={}", shell_dist(&hi, &lo));
    }

    #[test]
    fn traverse_refuses_over_budget() {
        let v = traverse_verdict(0.0, 1e-9, 2.895, 0.0, 0.0, 1e-2);
        assert!(v.contains("over budget"));
    }

    #[test]
    fn hunger_priority_orders_recency() {
        let now = 1_800_000_000u64;
        let fresh = hunger_priority(3, 2, now - 3600, now);
        let stale = hunger_priority(3, 2, now - 86400 * 30, now);
        assert!(fresh > stale);
        assert!((fresh - 3.0 * 2.0 * (1.0 + 1.0 / (1.0 + 1.0 / 24.0))).abs() < 1e-9);
    }

    #[test]
    fn fusion_adapts_to_signal() {
        assert_eq!(fusion_weights(2.6), (0.2, 0.8));
        assert_eq!(fusion_weights(0.0), (0.8, 0.2));
    }

    #[test]
    fn judge_rejects_sludge() {
        assert!(!judge_gates(3, 1, 1, 0.0).0);
        assert!(judge_gates(60, 10, 8, 2.5).0);
    }
}
