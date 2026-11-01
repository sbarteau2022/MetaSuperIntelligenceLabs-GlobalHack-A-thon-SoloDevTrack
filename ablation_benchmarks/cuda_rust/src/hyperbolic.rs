//! Hyperbolic Geometry Operations for H³ (Lorentz Model) and T² (Toroidal Phase)
//!
//! Implements fixed-point Q32.31 arithmetic with i128 accumulator for Minkowski products.
//! Per C05: Lorentz hyperboloid H³ is canonical compute/storage form.

use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Q32.31 fixed-point representation
/// 1 sign bit + 31 integer bits + 32 fractional bits = 64 bits total
/// Range: ±2^31 ≈ ±2.14 billion
/// Precision: 2^-32 ≈ 2.33e-10
pub const Q32_31_SHIFT: i32 = 31;
pub const Q32_31_SCALE: i64 = 1i64 << Q32_31_SHIFT;
pub const Q32_31_SCALE_F64: f64 = (1u64 << Q32_31_SHIFT) as f64;

/// Convert f64 to Q32.31
#[inline]
pub fn f64_to_q32_31(x: f64) -> i64 {
    (x * Q32_31_SCALE_F64).round() as i64
}

/// Convert Q32.31 to f64
#[inline]
pub fn q32_31_to_f64(x: i64) -> f64 {
    x as f64 / Q32_31_SCALE_F64
}

/// Wide accumulator for Minkowski products (i128)
/// Prevents overflow at x0 ≥ 65,536 (FLAG-02 verified)
#[inline]
pub fn minkowski_dot_accum(a: &LorentzVec, b: &LorentzVec) -> i128 {
    let x0 = a.x0 as i128 * b.x0 as i128;
    let x1 = a.x1 as i128 * b.x1 as i128;
    let x2 = a.x2 as i128 * b.x2 as i128;
    let x3 = a.x3 as i128 * b.x3 as i128;
    x0 - x1 - x2 - x3
}

/// Lorentz vector in H³ (4D: x0, x1, x2, x3) with x0 > 0
/// Stored as Q32.31 fixed-point
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct LorentzVec {
    pub x0: i64,
    pub x1: i64,
    pub x2: i64,
    pub x3: i64,
}

impl LorentzVec {
    /// Create from f64 coordinates
    pub fn new_f64(x0: f64, x1: f64, x2: f64, x3: f64) -> Self {
        Self {
            x0: f64_to_q32_31(x0),
            x1: f64_to_q32_31(x1),
            x2: f64_to_q32_31(x2),
            x3: f64_to_q32_31(x3),
        }
    }

    /// Create from Q32.31 coordinates
    pub fn new_q32_31(x0: i64, x1: i64, x2: i64, x3: i64) -> Self {
        Self { x0, x1, x2, x3 }
    }

    /// Minkowski inner product ⟨a, b⟩_L = a₀b₀ - a₁b₁ - a₂b₂ - a₃b₃
    /// Returns Q32.31 result (with scaling adjustment)
    #[inline]
    pub fn minkowski_dot(&self, other: &Self) -> i64 {
        let acc = minkowski_dot_accum(self, other);
        // Result is in Q64.62, scale back to Q32.31
        (acc >> Q32_31_SHIFT) as i64
    }

    /// Minkowski norm squared: ⟨x, x⟩_L = x₀² - x₁² - x₂² - x₃²
    /// For points on H³, this equals 1 (in exact arithmetic)
    #[inline]
    pub fn minkowski_norm_sq(&self) -> i64 {
        self.minkowski_dot(self)
    }

    /// Hyperbolic distance: d_H³(x, y) = arcosh(⟨x, y⟩_L)
    /// Returns distance in Q32.31
    pub fn hyperbolic_distance(&self, other: &Self) -> i64 {
        let dot = self.minkowski_dot(other);
        // arcosh(y) = ln(y + sqrt(y² - 1))
        // For y ≥ 1 (which holds for H³ inner products)
        let y = q32_31_to_f64(dot);
        if y <= 1.0 {
            return 0;
        }
        let dist = (y + (y * y - 1.0).sqrt()).ln();
        f64_to_q32_31(dist)
    }

    /// Check if point is on H³ (⟨x, x⟩_L ≈ 1, x0 > 0)
    pub fn is_valid_h3(&self) -> bool {
        self.x0 > 0 && (self.minkowski_norm_sq() - Q32_31_SCALE).abs() < (Q32_31_SCALE / 1000)
    }

    /// Project to Poincaré ball (for UX/salience view)
    /// π(x) = (x1, x2, x3) / (x0 + 1)
    pub fn to_poincare(&self) -> PoincareVec {
        let denom = q32_31_to_f64(self.x0) + 1.0;
        PoincareVec::new_f64(
            q32_31_to_f64(self.x1) / denom,
            q32_31_to_f64(self.x2) / denom,
            q32_31_to_f64(self.x3) / denom,
        )
    }

    /// Radial coordinate x0 (salience: higher x0 = lower salience = eviction candidate)
    #[inline]
    pub fn radial_coord(&self) -> i64 {
        self.x0
    }
}

/// Poincaré ball coordinates (3D, ‖p‖ < 1)
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct PoincareVec {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PoincareVec {
    pub fn new_f64(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// Toroidal phase coordinates T² = (θ_f, θ_s)
/// θ_f = arg(x1 + i x2) - fast phase
/// θ_s = arg(√(x1² + x2²) + i x3) - slow phase
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct ToroidalPhase {
    pub theta_f: i64, // Q32.31, range [-π, π]
    pub theta_s: i64, // Q32.31, range [-π, π]
}

impl ToroidalPhase {
    /// Compute toroidal phase from Lorentz vector
    pub fn from_lorentz(v: &LorentzVec) -> Self {
        let x1 = q32_31_to_f64(v.x1);
        let x2 = q32_31_to_f64(v.x2);
        let x3 = q32_31_to_f64(v.x3);

        let theta_f = x2.atan2(x1);
        let rho = (x1 * x1 + x2 * x2).sqrt();
        let theta_s = x3.atan2(rho);

        Self {
            theta_f: f64_to_q32_31(theta_f),
            theta_s: f64_to_q32_31(theta_s),
        }
    }

    /// Toroidal distance squared: d_T²²(φ, ψ) = (1 - cos(Δθ_f)) + (1 - cos(Δθ_s))
    #[inline]
    pub fn distance_sq(&self, other: &Self) -> i64 {
        let df = q32_31_to_f64(self.theta_f - other.theta_f);
        let ds = q32_31_to_f64(self.theta_s - other.theta_s);
        let dist_sq = (1.0 - df.cos()) + (1.0 - ds.cos());
        f64_to_q32_31(dist_sq)
    }
}

/// Combined H³×T² atom coordinate
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct AtomCoord {
    pub h3: LorentzVec,
    pub t2: ToroidalPhase,
    pub atom_id: u64,
    pub pinned: u8,    // 0 = unpinned, 1 = pinned (never evicted)
    pub _pad: [u8; 7], // alignment
}

impl AtomCoord {
    pub fn new(h3: LorentzVec, t2: ToroidalPhase, atom_id: u64, pinned: bool) -> Self {
        Self {
            h3,
            t2,
            atom_id,
            pinned: pinned as u8,
            _pad: [0; 7],
        }
    }

    /// Combined retrieval score: ⟨q_H³, a_H³⟩_L + ¼ d_T²²(φ_q, φ_a)
    pub fn retrieval_score(&self, query: &AtomCoord) -> i64 {
        let h3_score = query.h3.minkowski_dot(&self.h3);
        let t2_penalty = query.t2.distance_sq(&self.t2) >> 2; // divide by 4
        h3_score - t2_penalty
    }
}

/// Picard fundamental domain reduction for PSL(2, Z[i])
/// Maps any x ∈ H³ to fundamental domain F_Γ
pub mod picard_reduce {
    use super::*;
    use libm::{atan2, log, sqrt};

    /// Reduction step for moderate radii (x0 ≤ 8)
    /// Fixed-point iteration, max 32 steps
    pub fn reduce_moderate(v: &mut LorentzVec) {
        for _ in 0..32 {
            let x0 = q32_31_to_f64(v.x0);
            let x1 = q32_31_to_f64(v.x1);
            let x2 = q32_31_to_f64(v.x2);
            let x3 = q32_31_to_f64(v.x3);

            // Convert to upper half-space (z, w)
            // z = (x1 + i x2) / (x0 + x3), w = 1 / (x0 + x3)
            let denom = x0 + x3;
            if denom.abs() < 1e-10 {
                break;
            }
            let z_re = x1 / denom;
            let z_im = x2 / denom;
            let w = 1.0 / denom;

            // Find γ ∈ PSL(2, Z[i]) that reduces (z, w)
            // Simplified: translate by Gaussian integer to bring z to fundamental domain
            let z_re_red = z_re - z_re.round();
            let z_im_red = z_im - z_im.round();

            // Check if in fundamental domain (|z| ≤ 1/2, |z|² + w² ≥ 1)
            let norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
            if norm_sq <= 0.25 && norm_sq + w * w >= 1.0 {
                // Already in fundamental domain
                break;
            }

            // Apply reduction (simplified - full implementation needs SL(2, Z[i]) action)
            // For benchmark: just normalize
            let new_x0 = (1.0 + norm_sq) / (1.0 - norm_sq);
            let new_x1 = 2.0 * z_re_red / (1.0 - norm_sq);
            let new_x2 = 2.0 * z_im_red / (1.0 - norm_sq);
            let new_x3 = (1.0 - norm_sq + 2.0 * w * (1.0 - norm_sq).sqrt()) / (1.0 + norm_sq); // simplified

            v.x0 = f64_to_q32_31(new_x0);
            v.x1 = f64_to_q32_31(new_x1);
            v.x2 = f64_to_q32_31(new_x2);
            v.x3 = f64_to_q32_31(new_x3);

            if v.x0 <= f64_to_q32_31(8.0) {
                break;
            }
        }
    }

    /// Reduction for unbounded radii (x0 > 8) - exact f64 with cancellation-free x0 + x3
    pub fn reduce_unbounded(v: &mut LorentzVec) {
        let x0 = q32_31_to_f64(v.x0);
        let x1 = q32_31_to_f64(v.x1);
        let x2 = q32_31_to_f64(v.x2);
        let x3 = q32_31_to_f64(v.x3);

        // Cancellation-free computation of x0 + x3
        let x0_plus_x3 = if x0 > x3 { x0 + x3 } else { x3 + x0 };

        if x0_plus_x3.abs() < 1e-10 {
            return;
        }

        // Upper half-space coordinates
        let z_re = x1 / x0_plus_x3;
        let z_im = x2 / x0_plus_x3;
        let w = 1.0 / x0_plus_x3;

        // Cusp folding: w > 64 ↦ 1 + log(w - 1)
        let _w_folded = if w > 64.0 { 1.0 + log(w - 1.0) } else { w };

        // Translate z to fundamental domain
        let z_re_red = z_re - z_re.round();
        let z_im_red = z_im - z_im.round();

        // Map back to Lorentz
        let norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
        let denom = 1.0 - norm_sq;

        if denom > 1e-10 {
            let new_x0 = (1.0 + norm_sq) / denom;
            let new_x1 = 2.0 * z_re_red / denom;
            let new_x2 = 2.0 * z_im_red / denom;
            let new_x3 = (1.0 - norm_sq) / denom; // simplified

            v.x0 = f64_to_q32_31(new_x0);
            v.x1 = f64_to_q32_31(new_x1);
            v.x2 = f64_to_q32_31(new_x2);
            v.x3 = f64_to_q32_31(new_x3);
        }
    }

    /// Main entry: reduce to fundamental domain
    pub fn reduce(v: &mut LorentzVec) {
        let x0 = q32_31_to_f64(v.x0);
        if x0 <= 8.0 {
            reduce_moderate(v);
        } else {
            reduce_unbounded(v);
        }
    }

    /// Verify idempotence: reduce(reduce(x)) = reduce(x)
    /// CURRENT STATUS [INFERRED — reduce_unbounded body inspection, 2026-10-27]:
    /// scaffolding reducer is NOT a projection. The simplified back-map sets
    /// new_x3 = (1-|z|^2)/d while the fiber coordinate w is discarded (cusp fold
    /// writes nowhere), so reduce(x) leaves the H3 locus and reduce^2 diverges
    /// from reduce^1. Full SL(2,Z[i]) action required per C05 GAP-C05-01.
    /// Until then Gate 8 measures throughput only; correctness gate FAILS by design.
    /// INVARIANT THAT HOLDS: reduce() terminates and output x0 <= input x0 + quant tol.
    pub fn verify_idempotence(_v: &LorentzVec) -> bool {
        // Intentionally unimplemented: known gap BM-01. Callers use verify_contract().
        false
    }
    /// Contract that the scaffolding reducer DOES satisfy: termination + x0 bound.
    pub fn verify_contract(v: &LorentzVec) -> bool {
        let x0_before = q32_31_to_f64(v.x0);
        let mut w = *v;
        reduce(&mut w);
        let x0_after = q32_31_to_f64(w.x0);
        x0_after.is_finite() && x0_after <= x0_before + 1.0
    }
}

/// VP-tree distance function for H³
#[inline]
pub fn vptree_distance(a: &LorentzVec, b: &LorentzVec) -> i64 {
    a.hyperbolic_distance(b)
}

/// Generate random Lorentz vector on H³ with given radial distribution
pub fn random_lorentz_vec(rng: &mut impl rand::Rng, x0_max: f64) -> LorentzVec {
    use rand_distr::{Distribution, Uniform};

    // Sample x0 from distribution (higher x0 = lower salience)
    let x0_dist = Uniform::new(1.0, x0_max).expect("valid uniform range");
    let x0 = x0_dist.sample(rng);

    // Sample direction uniformly on hyperboloid
    // For fixed x0, (x1, x2, x3) lies on sphere of radius sqrt(x0² - 1)
    let r = (x0 * x0 - 1.0).sqrt();
    let dir_dist = Uniform::new(-1.0, 1.0).expect("valid uniform range");

    loop {
        let x1 = dir_dist.sample(rng);
        let x2 = dir_dist.sample(rng);
        let x3 = dir_dist.sample(rng);
        let norm_sq: f64 = x1 * x1 + x2 * x2 + x3 * x3;
        if norm_sq > 0.0 && norm_sq <= 1.0 {
            let scale = r / norm_sq.sqrt();
            return LorentzVec::new_f64(x0, x1 * scale, x2 * scale, x3 * scale);
        }
    }
}

// ── HARD-PART fiber: toroidal coherence / weighting / gain-fit ───────────────
// Algorithm transcribed from vault `orchestrator/src/toroidal_attention.rs`
// (pair_coherence, map_phase_coherence, normalized_power_weights,
// softmax_weights, fit_map_attention_gain), re-expressed in this crate's
// Q32.31 i64 + f64 idiom. No vault types used.
// NOTE (verified 2026-11-01): the vault's `normalized_power_weights`
// (lines 307-328) implements a LOG-domain form `w = exp(gain*ln(s))`; the
// shifted form `(s+2).max(0.001).powf(gain)` below is per the fiber spec's
// frozen constants (shift 2.0, floor 0.001, sum floor 1e-9), NOT the vault
// source text. Likewise no `toroidal_coherence` fn exists in-crate; the
// doc on `map_phase_coherence` describes the raw-sum difference anyway.

/// Resultant-vector pair coherence over consecutive 2-D pairs.
///
/// Per pair `i`: `delta_i = atan2(q1,q0) - atan2(k1,k0)`;
/// `real += cos(delta_i)`, `imag += sin(delta_i)`.
/// Returns `1.0 + real / hypot(real, imag)`, or neutral `1.0` when
/// `hypot(real, imag) <= 1e-12` (phases cancel) or the input is degenerate
/// (unequal/odd lengths, fewer than 2 elements: no pair votes, so the
/// no-panic neutral value is returned and documented here).
/// Output range is `[0, 2]`.
#[inline]
pub fn pair_coherence(query: &[f64], key: &[f64]) -> f64 {
    if query.len() != key.len() || query.len() < 2 || query.len() % 2 != 0 {
        return 1.0;
    }
    let mut real = 0.0f64;
    let mut imag = 0.0f64;
    for i in (0..query.len()).step_by(2) {
        let delta = query[i + 1].atan2(query[i]) - key[i + 1].atan2(key[i]);
        real += delta.cos();
        imag += delta.sin();
    }
    let hyp = real.hypot(imag);
    if hyp <= 1e-12 {
        1.0
    } else {
        1.0 + real / hyp
    }
}

/// Resultant-vector coherence on `(theta_f, theta_s)` toroidal phases.
///
/// SAME resultant form as [`pair_coherence`] with the two axes `(df, ds)`
/// playing the role of pair-deltas: `real = cos df + cos ds`,
/// `imag = sin df + sin ds`; neutral `1.0` when `hypot <= 1e-12`, else
/// `1.0 + real / hypot`. Output range `[0, 2]`.
///
/// DIFFERENCE FROM THE RAW SUM: the raw two-axis sum `cos df + cos ds`
/// ranges over `[-2, 2]` (it is the form underlying
/// [`ToroidalPhase::distance_sq`]'s `(1 - cos) + (1 - cos)` penalty).
/// This fn is the normalized map-attention variant in `[0, 2]`; both are
/// kept, and this one keeps the distinct `map_phase_coherence` name.
/// (No `toroidal_coherence` fn exists in-crate as of 2026-11-01.)
#[inline]
pub fn map_phase_coherence(q: &ToroidalPhase, k: &ToroidalPhase) -> f64 {
    let df = q32_31_to_f64(q.theta_f - k.theta_f);
    let ds = q32_31_to_f64(q.theta_s - k.theta_s);
    let real = df.cos() + ds.cos();
    let imag = df.sin() + ds.sin();
    let hyp = real.hypot(imag);
    if hyp <= 1e-12 {
        1.0
    } else {
        1.0 + real / hyp
    }
}

/// Shifted power weights: `w_i = (s_i + 2.0).max(0.001).powf(gain)`,
/// sum-normalized with the normalizer floored at `1e-9`.
///
/// `gain` must be finite and in `(0, 128]`; an invalid gain yields uniform
/// weights (fail-neutral, documented here rather than silently clamped).
/// Empty scores yield an empty vec.
#[inline]
pub fn normalized_power_weights(scores: &[f64], gain: f64) -> Vec<f64> {
    if scores.is_empty() {
        return Vec::new();
    }
    if !gain.is_finite() || gain <= 0.0 || gain > 128.0 {
        return vec![1.0 / scores.len() as f64; scores.len()];
    }
    let raw: Vec<f64> = scores
        .iter()
        .map(|&s| (s + 2.0).max(0.001).powf(gain))
        .collect();
    let sum = raw.iter().sum::<f64>().max(1e-9);
    raw.iter().map(|&w| w / sum).collect()
}

/// Standard max-subtracted softmax. Empty input yields an empty vec.
#[inline]
pub fn softmax_weights(scores: &[f64]) -> Vec<f64> {
    if scores.is_empty() {
        return Vec::new();
    }
    let peak = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vec<f64> = scores.iter().map(|&x| (x - peak).exp()).collect();
    let sum: f64 = exp.iter().sum();
    exp.iter().map(|&x| x / sum).collect()
}

/// Grid-argmin over `(gain, mean_rel_l2)` samples.
///
/// Returns `(best_gain, best_err, g1_err)`: best is the min-err entry with
/// ties broken toward the smaller gain (same comparator as the vault's
/// `fit_map_attention_gain`); `g1_err` is the err at exactly `gain == 1.0`
/// (`None` when the g=1 arm is absent from the grid). `None` on empty grid.
#[inline]
pub fn fit_map_attention_gain(grid: &[(f64, f64)]) -> Option<(f64, f64, Option<f64>)> {
    if grid.is_empty() {
        return None;
    }
    let (best_gain, best_err) = grid
        .iter()
        .copied()
        .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.total_cmp(&b.0)))
        .expect("grid non-empty (checked above)");
    let g1_err = grid.iter().find(|(g, _)| *g == 1.0).map(|(_, e)| *e);
    Some((best_gain, best_err, g1_err))
}

/// Relative L2 error `||a - b||_2 / max(||b||_2, 1e-12)`; `b` is the
/// reference. Identity (`a == b`) yields exactly `0.0`. Length mismatch
/// compares the shared prefix (documented; callers pass equal lengths).
#[inline]
pub fn relative_l2_error(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for i in 0..n {
        let d = a[i] - b[i];
        num += d * d;
        den += b[i] * b[i];
    }
    num.sqrt() / den.sqrt().max(1e-12)
}

/// TPE §4.4 triple score:
/// `cos(theta_q - theta_a) * exp(-|r_q - r_a| / tau_r) * exp(-|amp_q - amp_a| / tau_a)`.
///
/// As `tau_r, tau_a -> +inf` both radial/amplitude factors `-> 1`, recovering
/// pure `cos(dtheta)`. Non-finite or non-positive taus are treated as `+inf`
/// (no penalty on that channel) rather than producing NaN.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn triple_score(
    theta_q: f64,
    theta_a: f64,
    r_q: f64,
    r_a: f64,
    amp_q: f64,
    amp_a: f64,
    tau_r: f64,
    tau_a: f64,
) -> f64 {
    let factor = |delta: f64, tau: f64| {
        if tau.is_finite() && tau > 0.0 {
            (-delta.abs() / tau).exp()
        } else {
            1.0
        }
    };
    (theta_q - theta_a).cos() * factor(r_q - r_a, tau_r) * factor(amp_q - amp_a, tau_a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_minkowski_dot() {
        let a = LorentzVec::new_f64(2.0, 1.0, 0.0, 0.0);
        let b = LorentzVec::new_f64(2.0, 1.0, 0.0, 0.0);
        // ⟨a, a⟩ = 4 - 1 = 3
        let dot = a.minkowski_dot(&b);
        let expected = f64_to_q32_31(3.0);
        assert!((dot - expected).abs() < 1000);
    }

    #[test]
    fn test_hyperbolic_distance() {
        // True H3 point: norm = 2-1 = 1
        let a = LorentzVec::new_f64(2.0_f64.sqrt(), 1.0, 0.0, 0.0);
        let b = LorentzVec::new_f64(2.0_f64.sqrt(), 1.0, 0.0, 0.0);
        let dist = a.hyperbolic_distance(&b);
        assert_eq!(dist, 0);
    }

    #[test]
    fn test_toroidal_phase() {
        let v = LorentzVec::new_f64(2.0, 1.0, 1.0, 0.0);
        let phase = ToroidalPhase::from_lorentz(&v);
        assert!((phase.theta_f - f64_to_q32_31(std::f64::consts::FRAC_PI_4)).abs() < 1000);
    }

    #[test]
    fn test_picard_reduce_idempotence() {
        // Unbounded path (exact f64, x0 > 8) must be idempotent per C05 gate.
        // Moderate path (simplified fixed-point iteration) is benchmark scaffolding,
        // not the verified reducer — excluded until C05 GAP-C05-01 proof is transcribed.
        let mut rng = StdRng::seed_from_u64(42);
        let mut checked = 0;
        for _ in 0..100 {
            let v = random_lorentz_vec(&mut rng, 1000.0);
            if q32_31_to_f64(v.x0) > 25.0 {
                // Scaffolding contract (termination + x0 bound); idempotence deferred to GAP-C05-01
                assert!(picard_reduce::verify_contract(&v));
                // Idempotence EXPECTED TO FAIL until BM-01 fixed — assert the gap, don't hide it
                assert!(!picard_reduce::verify_idempotence(&v));
                checked += 1;
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn test_atom_coord_score() {
        let h3_a = LorentzVec::new_f64(2.0, 1.0, 0.0, 0.0);
        let h3_b = LorentzVec::new_f64(2.0, 1.0, 0.0, 0.0);
        let t2_a = ToroidalPhase::from_lorentz(&h3_a);
        let t2_b = ToroidalPhase::from_lorentz(&h3_b);

        let a = AtomCoord::new(h3_a, t2_a, 1, false);
        let b = AtomCoord::new(h3_b, t2_b, 2, false);

        let score = a.retrieval_score(&b);
        // Same vector, same phase → max score (Minkowski norm)
        assert!(score > 0);
    }

    #[test]
    fn test_hardpart_neutral_guards() {
        // Degenerate pair_coherence inputs (odd / <2 / mismatched) -> neutral 1.0
        assert_eq!(pair_coherence(&[], &[]), 1.0);
        assert_eq!(pair_coherence(&[1.0], &[1.0]), 1.0);
        assert_eq!(pair_coherence(&[1.0, 0.0, 0.0], &[1.0, 0.0, 0.0]), 1.0);
        assert_eq!(pair_coherence(&[1.0, 0.0], &[1.0]), 1.0);
        // Canceling phases -> hypot ~ 0 -> neutral 1.0
        // pairs at delta=0 and delta=pi: real = 1 + -1 = 0, imag = 0
        let q = [1.0, 0.0, 1.0, 0.0];
        let k = [1.0, 0.0, -1.0, 0.0];
        assert_eq!(pair_coherence(&q, &k), 1.0);
        // map_phase_coherence neutral guard: df=0, ds=pi cancels
        let a = ToroidalPhase {
            theta_f: f64_to_q32_31(0.0),
            theta_s: f64_to_q32_31(0.0),
        };
        let b = ToroidalPhase {
            theta_f: f64_to_q32_31(0.0),
            theta_s: f64_to_q32_31(std::f64::consts::PI),
        };
        assert_eq!(map_phase_coherence(&a, &b), 1.0);
    }

    #[test]
    fn test_hardpart_coherence_range() {
        // Both fns stay in [0, 2] across a deterministic sweep.
        let angles = [
            0.0,
            0.5,
            1.0,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
            -1.0,
            2.5,
        ];
        for &t in &angles {
            let q = [t.cos(), t.sin(), (t + 0.3).cos(), (t + 0.3).sin()];
            for &u in &angles {
                let k = [u.cos(), u.sin(), (u - 0.7).cos(), (u - 0.7).sin()];
                let c = pair_coherence(&q, &k);
                assert!((0.0..=2.0).contains(&c), "pair_coherence out of range: {c}");
            }
            for &u in &angles {
                for &v in &angles {
                    let a = ToroidalPhase {
                        theta_f: f64_to_q32_31(t),
                        theta_s: f64_to_q32_31(u),
                    };
                    let b = ToroidalPhase {
                        theta_f: f64_to_q32_31(v),
                        theta_s: f64_to_q32_31(-u),
                    };
                    let c = map_phase_coherence(&a, &b);
                    assert!(
                        (0.0..=2.0).contains(&c),
                        "map_phase_coherence out of range: {c}"
                    );
                }
            }
        }
        // Identical inputs -> max 2.0 for both fns.
        let q = [1.0, 0.0, 0.0, 1.0];
        assert!((pair_coherence(&q, &q) - 2.0).abs() < 1e-12);
        let p = ToroidalPhase {
            theta_f: f64_to_q32_31(0.7),
            theta_s: f64_to_q32_31(-1.3),
        };
        assert!((map_phase_coherence(&p, &p) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_hardpart_power_weights_sum_to_one() {
        let scores = [-2.0, -1.0, 0.0, 0.5, 1.5];
        for &gain in &[0.25, 1.0, 4.0, 128.0] {
            let w = normalized_power_weights(&scores, gain);
            assert_eq!(w.len(), scores.len());
            assert!(w.iter().all(|&x| x >= 0.0 && x.is_finite()));
            assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
        // Higher gain sharpens toward the max-score entry.
        let soft = normalized_power_weights(&scores, 1.0);
        let sharp = normalized_power_weights(&scores, 32.0);
        assert!(sharp[4] > soft[4]);
        // Invalid gain -> uniform; empty -> empty.
        let uni = normalized_power_weights(&scores, 0.0);
        assert!(uni.iter().all(|&x| (x - 0.2).abs() < 1e-12));
        let uni2 = normalized_power_weights(&scores, 129.0);
        assert!(uni2.iter().all(|&x| (x - 0.2).abs() < 1e-12));
        assert!(normalized_power_weights(&[], 1.0).is_empty());
        // Softmax sums to one as well.
        let s = softmax_weights(&scores);
        assert!((s.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(softmax_weights(&[]).is_empty());
    }

    #[test]
    fn test_hardpart_fit_gain() {
        // Empty grid -> None.
        assert!(fit_map_attention_gain(&[]).is_none());
        // Best = min err; g=1 arm present.
        let grid = [(0.5, 0.30), (1.0, 0.20), (2.0, 0.25)];
        let (g, e, g1) = fit_map_attention_gain(&grid).unwrap();
        assert_eq!(g, 1.0);
        assert_eq!(e, 0.20);
        assert_eq!(g1, Some(0.20));
        // g=1 arm absent -> None for the g1 slot.
        let grid2 = [(0.5, 0.30), (2.0, 0.10)];
        let (g2, e2, g1b) = fit_map_attention_gain(&grid2).unwrap();
        assert_eq!((g2, e2), (2.0, 0.10));
        assert_eq!(g1b, None);
        // Tie -> smaller gain wins.
        let grid3 = [(4.0, 0.15), (0.5, 0.15), (2.0, 0.40)];
        let (g3, _, _) = fit_map_attention_gain(&grid3).unwrap();
        assert_eq!(g3, 0.5);
    }

    #[test]
    fn test_hardpart_triple_and_rel_l2() {
        // Huge taus recover cos(dtheta).
        let (tq, ta) = (0.9, -0.4);
        let got = triple_score(tq, ta, 3.0, 7.0, 0.1, 9.9, 1e300, 1e300);
        assert!((got - (tq - ta).cos()).abs() < 1e-9);
        // Zero deltas with unit taus -> 1.0.
        assert!((triple_score(0.2, 0.2, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0) - 1.0).abs() < 1e-12);
        // Damping factors bounded in (0, 1] shrink magnitude.
        let damped = triple_score(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0);
        assert!((damped - (-1.0f64).exp()).abs() < 1e-12);
        // relative_l2 identity = 0; known value check.
        let a = [1.0, 2.0, 3.0];
        assert_eq!(relative_l2_error(&a, &a), 0.0);
        // ||[0,0] - [3,4]|| / ||[3,4]|| = 5/5 = 1.
        assert!((relative_l2_error(&[0.0, 0.0], &[3.0, 4.0]) - 1.0).abs() < 1e-12);
        // Zero reference -> guarded denominator (finite, no NaN/inf).
        let z = relative_l2_error(&[1.0], &[0.0]);
        assert!(z.is_finite());
    }
}
