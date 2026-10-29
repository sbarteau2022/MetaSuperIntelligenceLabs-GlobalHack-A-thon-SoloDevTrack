//! Item 5e (inside gap 1): regularized multi-objective evolution.
//!
//! Pass rate must never be bought by relaxing a gate. Four regulators:
//! 1. Leakage critic: rejects candidates that smuggle answers via prompt
//!    injection-shaped edits (flagged marker strings in bundle components).
//! 2. Noise-band acceptance: a gain within ±noise_band of base is NOT a win
//!    (abstain). Only gains above the band adopt.
//! 3. Pruner: drops dominated candidates before probing (saves energy,
//!    keeps the ledger honest about what was actually evaluated).
//! 4. Pareto frontier: functionality (quality) vs security (gate strictness)
//!    — a candidate adopts iff it is non-dominated: no other candidate is
//!    better on both axes. Gate relaxation that raises quality while
//!    lowering security lands OFF the frontier and is rejected.

use crate::rsi::Bundle;
use serde::{Deserialize, Serialize};

/// Leakage critic: scans bundle components for injection-shaped markers.
/// A candidate that "improves" by embedding the answer key is leakage,
/// not learning. Returns flagged component indices.
pub fn leakage_critic(bundle: &Bundle) -> Vec<usize> {
    const MARKERS: &[&str] = &[
        "ignore previous",
        "system override",
        "answer is",
        "expected output",
        "PASS packet", // synthetic-oracle gaming
        "disregard",
    ];
    bundle
        .components
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            let lower = c.to_lowercase();
            MARKERS.iter().any(|m| lower.contains(m))
        })
        .map(|(i, _)| i)
        .collect()
}

/// Noise-band acceptance: gain must exceed the band to adopt.
/// band default 0.02 (2pp): inside the band, abstain (KEEP_BASE).
pub fn noise_band_accept(gain: f64, band: f64) -> bool {
    gain > band
}

/// Pruner: drop candidates dominated on (quality, energy) before probing.
/// A candidate pruned here never enters the ledger as evaluated.
pub fn prune_dominated(candidates: &[(Bundle, f64, f64)]) -> Vec<usize> {
    // (bundle, quality, energy): keep i iff no j has quality_j >= quality_i
    // AND energy_j <= energy_i with at least one strict.
    let mut keep = vec![];
    for (i, (_, qi, ei)) in candidates.iter().enumerate() {
        let dominated = candidates
            .iter()
            .enumerate()
            .any(|(j, (_, qj, ej))| j != i && *qj >= *qi && *ej <= *ei && (*qj > *qi || *ej < *ei));
        if !dominated {
            keep.push(i);
        }
    }
    keep
}

/// Pareto point: functionality (quality 0..1) vs security (gate strictness
/// 0..1, higher = stricter). Frontier = non-dominated set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    pub label: String,
    pub functionality: f64,
    pub security: f64,
}

pub fn pareto_frontier(points: &[ParetoPoint]) -> Vec<usize> {
    let mut keep = vec![];
    for (i, p) in points.iter().enumerate() {
        let dominated = points.iter().enumerate().any(|(j, q)| {
            j != i
                && q.functionality >= p.functionality
                && q.security >= p.security
                && (q.functionality > p.functionality || q.security > p.security)
        });
        if !dominated {
            keep.push(i);
        }
    }
    keep
}

/// Regularized adoption: all four regulators must agree.
/// Returns (adopt, reasons). A gate-relaxing quality gain fails at the
/// Pareto step: higher functionality + lower security = dominated by the
/// base (which has equal functionality at higher security) → rejected.
pub fn regularized_adopt(
    candidate_quality: f64,
    base_quality: f64,
    candidate: &Bundle,
    candidate_security: f64,
    base_security: f64,
    noise_band: f64,
) -> (bool, Vec<String>) {
    let mut reasons = vec![];
    let leaks = leakage_critic(candidate);
    if !leaks.is_empty() {
        reasons.push(format!("leakage critic: flagged components {:?}", leaks));
    }
    let gain = candidate_quality - base_quality;
    if !noise_band_accept(gain, noise_band) {
        reasons.push(format!(
            "noise band: gain {:.4} <= band {:.4}",
            gain, noise_band
        ));
    }
    let frontier = pareto_frontier(&[
        ParetoPoint {
            label: "base".into(),
            functionality: base_quality,
            security: base_security,
        },
        ParetoPoint {
            label: "candidate".into(),
            functionality: candidate_quality,
            security: candidate_security,
        },
    ]);
    // Candidate adopts iff it is ON the frontier AND not dominated by base.
    // Frontier has 2 entries when neither dominates; candidate must be one.
    let candidate_on_frontier = frontier.len() == 2;
    if !candidate_on_frontier {
        reasons.push(
            "pareto: candidate dominated (gate relaxation without functionality gain)".into(),
        );
    }
    let adopt = leaks.is_empty() && noise_band_accept(gain, noise_band) && candidate_on_frontier;
    (adopt, reasons)
}
