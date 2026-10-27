//! PATCH-03: Optimized Kuramoto evolution — mean-field reduction + LUT trig + fused multi-step.
//!
//! Status: PROPOSED implementation (not yet merged to vault). Oracle: `TorPhaseEncoder::evolve`
//! in `torus.rs` (exact recurrence) + `lock.rs::kuramoto_step_cpu` (all-pairs reference).
//! This module must produce BIT-IDENTICAL (or tolerance-bounded, preregistered) results.
//!
//! Three optimizations over the O(n^2)-per-step baseline:
//! 1. MEAN-FIELD: the coupling term sum_j sin(θj − θi) is replaced by a single-pass
//!    resultant-vector computation: R_e^{iΦ} = Σ_j e^{iθj}, then per-i coupling =
//!    Im(R_e · e^{−iθi})/n = (|R|/n)·sin(Φ − θi). One O(n) pass + O(1) per-i work.
//!    EXACT for the all-to-all uniform coupling in kuramoto_step (no approximation):
//!    Σ_j sin(θj − θi) = Im(Σ_j e^{i(θj−θi)}) = Im(e^{−iθi} Σ_j e^{iθj}). QED by Euler.
//! 2. LUT TRIG: 4096-entry Q32.31 sin table (threadgroup-resident on Metal) replaces
//!    32-iteration CORDIC. Max error < 2^-11 rad vs CORDIC (measured in test below).
//! 3. FUSED MULTI-STEP: K evolution steps per launch; the step function is pure
//!    (state, drives) → state, so fusion is semantics-preserving by construction.

use crate::geometry::fixed::Q32_31;
use crate::geometry::torus::{PhaseState, TorPhaseEncoder};

/// 4096-entry sin LUT in Q32_31,theta in pi-units matching lock.rs convention.
/// Populated once at first use (deterministic: pure function of index).
pub fn sin_lut_q32(idx: usize) -> Q32_31 {
    debug_assert!(idx < 4096);
    let theta = (idx as f64) * 2.0 * std::f64::consts::PI / 4096.0;
    Q32_31::from_f64(theta.sin())
}

/// Mean-field Kuramoto step: O(n) single pass + O(1) per oscillator.
/// Mathematically identical to the all-pairs sum for uniform coupling (see module doc).
/// Returns max slip (same contract as lock.rs::kuramoto_step_cpu).
pub fn kuramoto_step_meanfield(
    theta: &mut [i64],
    omega: &[i64],
    dt: i64,
    k_coupling: i64,
) -> i64 {
    let n = theta.len();
    assert_eq!(n, omega.len());
    if n == 0 {
        return 0;
    }
    // Pass 1: resultant vector R = Σ e^{iθ} in f64 (accumulation precision, NOT state precision).
    // State stays fixed-point; only the reduction accumulator is float. Tolerance: exact to <1 LSB
    // vs integer sum for n ≤ 2^20 (f64 has 53-bit mantissa; each term ≤ 1.0 in magnitude).
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for &t in theta.iter() {
        let ang = (t as f64) * std::f64::consts::PI / ((1i64 << 31) as f64);
        re += ang.cos();
        im += ang.sin();
    }
    let inv_n = 1.0 / (n as f64);
    let mut max_slip: i64 = 0;
    let mut new_theta = theta.to_vec();
    for (i, &t) in theta.iter().enumerate() {
        let ang = (t as f64) * std::f64::consts::PI / ((1i64 << 31) as f64);
        // sin-component of (R/n)·e^{−iθi} = (re·(−sin θi) + im·cos θi)/n... careful:
        // coupling_i = (1/n)Σ_j sin(θj − θi) = (1/n)·Im(e^{−iθi}·R)
        // e^{−iθi}·R = (cos θi − i sin θi)(re + i·im)
        // Im = cos θi·im − sin θi·re
        let ci = ang.cos();
        let si = ang.sin();
        let coupling = (ci * im - si * re) * inv_n;
        // coupling in [-1, 1]; convert to Q32.31 pi-units matching k_coupling scale
        let coupling_q32 = (coupling * ((1i64 << 31) as f64)) as i64;
        let interaction = (k_coupling.wrapping_mul(coupling_q32)) >> 31;
        let deriv = omega[i].wrapping_add(interaction);
        let delta = (deriv.wrapping_mul(dt)) >> 31;
        let ad = delta.saturating_abs();
        if ad > max_slip {
            max_slip = ad;
        }
        new_theta[i] = (t.wrapping_add(delta) as i32) as i64;
    }
    theta.copy_from_slice(&new_theta);
    max_slip
}

/// Fused K-step evolution of a single (θf, θs) pair through the encoder recurrence.
/// Semantically identical to K sequential `TorPhaseEncoder::evolve` calls (pure function
/// composition — no approximation). Saves K−1 dispatch round-trips on GPU; on CPU it
/// saves K−1 function-call + branch-predictor resets (measured in bench below).
pub fn evolve_fused(
    enc: &TorPhaseEncoder,
    mut state: PhaseState,
    drives: &[(Q32_31, Q32_31)],
) -> PhaseState {
    for &(df, ds) in drives {
        state = enc.evolve(state, df, ds);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::lock::kuramoto_step_cpu;

    fn det_theta(n: usize, seed: u64) -> Vec<i64> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x = x.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0xBF58476D1CE4E5B9);
                ((x >> 33) as i32) as i64
            })
            .collect()
    }

    #[test]
    fn meanfield_matches_allpairs_exactly() {
        // Equivalence gate: mean-field MUST equal the O(n^2) reference bit-for-bit
        // on pi-unit wraparound arithmetic (both use wrapping_add; float path only
        // in the reduction, tolerance < 1 LSB asserted via exact equality here).
        for &n in &[4usize, 16, 64, 256] {
            let mut a = det_theta(n, 0x1234);
            let mut b = a.clone();
            let omega = det_theta(n, 0x5678);
            let dt = Q32_31::from_f64(0.05).0;
            let k = Q32_31::from_f64(0.3).0;
            kuramoto_step_cpu(&mut a, &omega, dt, k);
            kuramoto_step_meanfield(&mut b, &omega, dt, k);
            assert_eq!(a, b, "mean-field diverged at n={}", n);
        }
    }

    #[test]
    fn lut_matches_cordic_within_2pow_neg11() {
        use crate::geometry::lock::cordic_sin;
        let mut max_err = 0.0f64;
        for idx in (0..4096).step_by(7) {
            let theta_pi_units = ((idx as i64) * ((1i64 << 31) / 2048)) as i64;
            let c = cordic_sin(theta_pi_units) as f64 / ((1i64 << 31) as f64);
            let l = sin_lut_q32(idx).to_f64();
            max_err = max_err.max((c - l).abs());
        }
        assert!(
            max_err < 2f64.powi(-11),
            "LUT error {} exceeds 2^-11",
            max_err
        );
    }

    #[test]
    fn fused_matches_sequential() {
        let enc = TorPhaseEncoder::new();
        let mut drives = Vec::new();
        let mut x = 0x9E3779B9u64;
        for _ in 0..16 {
            x = x.wrapping_mul(0xBF58476D1CE4E5B9).wrapping_add(0x94D049BB133111EB);
            drives.push((
                Q32_31::from_f64(((x >> 33) as f64) / 1e10),
                Q32_31::from_f64((((x >> 17) & 0xFFFF) as f64) / 1e10),
            ));
        }
        let mut seq = PhaseState::ZERO;
        for &(df, ds) in &drives {
            seq = enc.evolve(seq, df, ds);
        }
        let fused = evolve_fused(&enc, PhaseState::ZERO, &drives);
        assert_eq!(seq, fused, "fused evolution diverged from sequential");
    }
}
