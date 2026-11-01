//! Rank-5 trigonometric kernel recurrent linear attention (`f64`, CPU, no Metal).
//!
//! CPU transcription of `ToroidalHarmonicFieldState::step_and_readout` in
//! `/Users/stewartbarteau/.Trash/skills-desktop-orchestrator-20261030/src/toroidal_attention.rs`
//! (lines 640-800, read 2026-11-01). The vault version operates on `f32`
//! I/O; this fiber is `f64` end-to-end (state, I/O, readout).
//!
//! Kernel: `k(d) = (1 + cos d)^2 = 3/2 + 2 cos d + (1/2) cos 2d`
//! with 5-dim feature map
//! `phi(t) = [sqrt(3/2), sqrt(2) cos t, sqrt(2) sin t, (1/sqrt2) cos 2t, (1/sqrt2) sin 2t]`
//! so that `k(t_q - t_k) = <phi(t_q), phi(t_k)>`.
//!
//! Frost constants (copied exactly from the vault, must not drift):
//! weights `1.5 / 2.0 / 0.5`, gamma clamp `[0.90, 1.0]`, `r_iso` clamp
//! `[0, 4]`, floors `1e-12` (radius) / `1e-9` (denominator, RMS).

/// Isotropic (order-0) feature weight.
pub const W0: f64 = 1.5;
/// First-harmonic (order-1) feature weight.
pub const W1: f64 = 2.0;
/// Second-harmonic (order-2) feature weight.
pub const W2: f64 = 0.5;
/// Lower bound of the per-step decay clamp.
pub const GAMMA_LO: f64 = 0.90;
/// Upper bound of the per-step decay clamp.
pub const GAMMA_HI: f64 = 1.0;
/// Upper bound of the isotropic radius clamp (`r_iso` in `[0, 4]`).
pub const R_ISO_MAX: f64 = 4.0;
/// Radius floor: below this the phase defaults to `(c1, s1) = (1, 0)`.
pub const R_FLOOR: f64 = 1e-12;
/// Denominator floor for the readout division.
pub const DENOM_FLOOR: f64 = 1e-9;
/// RMS radius floor.
pub const RMS_FLOOR: f64 = 1e-9;

/// Clamp a raw decay value to the frosted `[0.90, 1.0]` horizon band.
#[inline]
pub fn clamp_gamma(decay: f64) -> f64 {
    decay.clamp(GAMMA_LO, GAMMA_HI)
}

/// 5-dim trigonometric feature map for angle `theta` (radians).
pub fn phi(theta: f64) -> [f64; 5] {
    const SQRT_3_2: f64 = 1.224_744_871_391_589_2; // sqrt(3/2)
    const SQRT_2: f64 = 1.414_213_562_373_095_1; // sqrt(2)
    const INV_SQRT_2: f64 = 0.707_106_781_186_547_6; // 1/sqrt(2)
    let (s1, c1) = theta.sin_cos();
    let t2 = 2.0 * theta;
    let (s2, c2) = t2.sin_cos();
    [
        SQRT_3_2,
        SQRT_2 * c1,
        SQRT_2 * s1,
        INV_SQRT_2 * c2,
        INV_SQRT_2 * s2,
    ]
}

/// Closed-form kernel `k(d) = (1 + cos d)^2`.
pub fn kernel_closed(d: f64) -> f64 {
    let k = 1.0 + d.cos();
    k * k
}

/// Expanded kernel `k(d) = 3/2 + 2 cos d + (1/2) cos 2d`.
pub fn kernel_expanded(d: f64) -> f64 {
    1.5 + 2.0 * d.cos() + 0.5 * (2.0 * d).cos()
}

/// `O(1)` recurrent linear-attention state with the 2nd-order trigonometric
/// kernel feature map (Katharopoulos et al., ICML 2020 formulation).
#[derive(Debug, Clone)]
pub struct Rank5State {
    pub pairs: usize,
    pub hidden_dim: usize,
    pub tokens: u64,
    /// Normalizer accumulators `Z_t` for the 5 trigonometric basis terms.
    pub z0: f64,
    pub zc1: Vec<f64>,
    pub zs1: Vec<f64>,
    pub zc2: Vec<f64>,
    pub zs2: Vec<f64>,
    /// Numerator accumulators `S_t` (feature-weighted value sums).
    pub m0: Vec<f64>,
    pub mc1: Vec<f64>,
    pub ms1: Vec<f64>,
    pub mc2: Vec<f64>,
    pub ms2: Vec<f64>,
}

impl Rank5State {
    pub fn new(hidden_dim: usize) -> Self {
        let pairs = (hidden_dim / 2).max(1);
        Self {
            pairs,
            hidden_dim,
            tokens: 0,
            z0: 0.0,
            zc1: vec![0.0; pairs],
            zs1: vec![0.0; pairs],
            zc2: vec![0.0; pairs],
            zs2: vec![0.0; pairs],
            m0: vec![0.0; hidden_dim],
            mc1: vec![0.0; hidden_dim],
            ms1: vec![0.0; hidden_dim],
            mc2: vec![0.0; hidden_dim],
            ms2: vec![0.0; hidden_dim],
        }
    }

    /// Ingest one causal token `x_in` and return `<phi(q_t), S_t> / <phi(q_t), Z_t>`
    /// in `O(D)` time. Exact `f64` port of the vault `step_and_readout`.
    pub fn step_and_readout(&mut self, x_in: &[f64], decay: f64) -> Vec<f64> {
        let d_len = x_in.len().min(self.hidden_dim);
        let pairs = d_len / 2;
        if pairs == 0 {
            return vec![0.0; x_in.len()];
        }

        // Gentle exponential horizon decay so recent phase structure stays sharp.
        let gamma = clamp_gamma(decay);
        self.z0 *= gamma;
        for p in 0..pairs {
            self.zc1[p] *= gamma;
            self.zs1[p] *= gamma;
            self.zc2[p] *= gamma;
            self.zs2[p] *= gamma;
        }
        for d in 0..d_len {
            self.m0[d] *= gamma;
            self.mc1[d] *= gamma;
            self.ms1[d] *= gamma;
            self.mc2[d] *= gamma;
            self.ms2[d] *= gamma;
        }

        // Isotropic RMS radius across pairs so no outlier dimension dominates.
        let mut mean_r_sq = 0.0f64;
        for p in 0..pairs {
            let x0 = x_in[2 * p];
            let x1 = x_in[2 * p + 1];
            mean_r_sq += x0 * x0 + x1 * x1;
        }
        let rms_r = (mean_r_sq / (pairs as f64)).sqrt().max(RMS_FLOOR);

        let mut c1_q = vec![0.0f64; pairs];
        let mut s1_q = vec![0.0f64; pairs];
        let mut c2_q = vec![0.0f64; pairs];
        let mut s2_q = vec![0.0f64; pairs];

        self.z0 += W0;
        for p in 0..pairs {
            let x0 = x_in[2 * p];
            let x1 = x_in[2 * p + 1];
            let r = x0.hypot(x1);
            let (c1, s1) = if r > R_FLOOR {
                (x0 / r, x1 / r)
            } else {
                (1.0, 0.0)
            };
            // Double-angle identity: cos(2t) = c1^2 - s1^2, sin(2t) = 2*c1*s1.
            let c2 = c1 * c1 - s1 * s1;
            let s2 = 2.0 * c1 * s1;
            let r_iso = (r / rms_r).clamp(0.0, R_ISO_MAX);

            c1_q[p] = c1;
            s1_q[p] = s1;
            c2_q[p] = c2;
            s2_q[p] = s2;

            self.zc1[p] += W1 * r_iso * c1;
            self.zs1[p] += W1 * r_iso * s1;
            self.zc2[p] += W2 * r_iso * c2;
            self.zs2[p] += W2 * r_iso * s2;

            for lane in 0..2 {
                let idx = 2 * p + lane;
                let v = x_in[idx];
                self.m0[idx] += W0 * v;
                self.mc1[idx] += W1 * r_iso * c1 * v;
                self.ms1[idx] += W1 * r_iso * s1 * v;
                self.mc2[idx] += W2 * r_iso * c2 * v;
                self.ms2[idx] += W2 * r_iso * s2 * v;
            }
        }
        self.tokens += 1;

        // O(D) recurrent readout: <phi(q_t), S_t> / <phi(q_t), Z_t>.
        let mut out = vec![0.0f64; x_in.len()];
        for p in 0..pairs {
            let denom = (self.z0
                + c1_q[p] * self.zc1[p]
                + s1_q[p] * self.zs1[p]
                + c2_q[p] * self.zc2[p]
                + s2_q[p] * self.zs2[p])
                .max(DENOM_FLOOR);
            for lane in 0..2 {
                let idx = 2 * p + lane;
                let numer = self.m0[idx]
                    + c1_q[p] * self.mc1[idx]
                    + s1_q[p] * self.ms1[idx]
                    + c2_q[p] * self.mc2[idx]
                    + s2_q[p] * self.ms2[idx];
                out[idx] = numer / denom;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot5(a: [f64; 5], b: [f64; 5]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    #[test]
    fn kernel_identity() {
        // k(d) = (1+cos d)^2 = 3/2 + 2cos d + (1/2)cos 2d = <phi(t), phi(t-d)>.
        let deltas = [0.0, 0.1, 0.5, 1.0, 2.0, std::f64::consts::PI, 3.7, -1.3];
        for d in deltas {
            let k = kernel_closed(d);
            let e = kernel_expanded(d);
            assert!((k - e).abs() < 1e-12, "closed vs expanded at d={d}: {k} vs {e}");
            let inner = dot5(phi(d), phi(0.0));
            assert!(
                (k - inner).abs() < 1e-9,
                "kernel identity at d={d}: {k} vs <phi,phi'>={inner}"
            );
        }
    }

    #[test]
    fn o1_state_constant_size() {
        let mut st = Rank5State::new(8);
        let x = vec![0.5, -0.25, 1.0, 0.75, -1.5, 0.125, 2.0, -0.5];
        let (zp, mp) = (st.zc1.len(), st.m0.len());
        for _ in 0..5 {
            let _ = st.step_and_readout(&x, 0.97);
            assert_eq!(st.zc1.len(), zp);
            assert_eq!(st.zs1.len(), zp);
            assert_eq!(st.zc2.len(), zp);
            assert_eq!(st.zs2.len(), zp);
            assert_eq!(st.m0.len(), mp);
            assert_eq!(st.mc1.len(), mp);
            assert_eq!(st.ms1.len(), mp);
            assert_eq!(st.mc2.len(), mp);
            assert_eq!(st.ms2.len(), mp);
        }
        assert_eq!(st.tokens, 5);
    }

    /// Recompute the step-`t` readout from scratch: re-accumulate every past
    /// token with weight `gamma^{t-1-i}` (constant gamma) and read out with
    /// the last token's query features. Must match the recurrent state.
    fn naive_readout(history: &[Vec<f64>], hidden_dim: usize, decay: f64) -> Vec<f64> {
        let gamma = clamp_gamma(decay);
        let t = history.len();
        let d_len = history[t - 1].len().min(hidden_dim);
        let pairs = d_len / 2;
        let mut z0 = 0.0;
        let mut zc1 = vec![0.0; pairs];
        let mut zs1 = vec![0.0; pairs];
        let mut zc2 = vec![0.0; pairs];
        let mut zs2 = vec![0.0; pairs];
        let mut m0 = vec![0.0; d_len];
        let mut mc1 = vec![0.0; d_len];
        let mut ms1 = vec![0.0; d_len];
        let mut mc2 = vec![0.0; d_len];
        let mut ms2 = vec![0.0; d_len];
        for (i, x) in history.iter().enumerate() {
            let w = gamma.powi((t - 1 - i) as i32);
            let mut mean_r_sq = 0.0;
            for p in 0..pairs {
                mean_r_sq += x[2 * p] * x[2 * p] + x[2 * p + 1] * x[2 * p + 1];
            }
            let rms_r = (mean_r_sq / (pairs as f64)).sqrt().max(RMS_FLOOR);
            z0 += w * W0;
            for p in 0..pairs {
                let (x0, x1) = (x[2 * p], x[2 * p + 1]);
                let r = x0.hypot(x1);
                let (c1, s1) = if r > R_FLOOR {
                    (x0 / r, x1 / r)
                } else {
                    (1.0, 0.0)
                };
                let c2 = c1 * c1 - s1 * s1;
                let s2 = 2.0 * c1 * s1;
                let r_iso = (r / rms_r).clamp(0.0, R_ISO_MAX);
                zc1[p] += w * W1 * r_iso * c1;
                zs1[p] += w * W1 * r_iso * s1;
                zc2[p] += w * W2 * r_iso * c2;
                zs2[p] += w * W2 * r_iso * s2;
                for lane in 0..2 {
                    let idx = 2 * p + lane;
                    m0[idx] += w * W0 * x[idx];
                    mc1[idx] += w * W1 * r_iso * c1 * x[idx];
                    ms1[idx] += w * W1 * r_iso * s1 * x[idx];
                    mc2[idx] += w * W2 * r_iso * c2 * x[idx];
                    ms2[idx] += w * W2 * r_iso * s2 * x[idx];
                }
            }
        }
        let q = &history[t - 1];
        let mut out = vec![0.0; q.len()];
        for p in 0..pairs {
            let (x0, x1) = (q[2 * p], q[2 * p + 1]);
            let r = x0.hypot(x1);
            let (c1, s1) = if r > R_FLOOR {
                (x0 / r, x1 / r)
            } else {
                (1.0, 0.0)
            };
            let c2 = c1 * c1 - s1 * s1;
            let s2 = 2.0 * c1 * s1;
            let denom = (z0 + c1 * zc1[p] + s1 * zs1[p] + c2 * zc2[p] + s2 * zs2[p])
                .max(DENOM_FLOOR);
            for lane in 0..2 {
                let idx = 2 * p + lane;
                out[idx] = (m0[idx]
                    + c1 * mc1[idx]
                    + s1 * ms1[idx]
                    + c2 * mc2[idx]
                    + s2 * ms2[idx])
                    / denom;
            }
        }
        out
    }

    #[test]
    fn causal_equivalence_8_steps() {
        let hidden_dim = 6;
        let decay = 0.95;
        // Deterministic non-degenerate inputs.
        let history: Vec<Vec<f64>> = (0..8)
            .map(|i| {
                (0..hidden_dim)
                    .map(|d| ((i * 7 + d * 13) as f64 * 0.31).sin() + 0.5 * ((i + d) as f64).cos())
                    .collect()
            })
            .collect();
        let mut st = Rank5State::new(hidden_dim);
        for (t, x) in history.iter().enumerate() {
            let got = st.step_and_readout(x, decay);
            let want = naive_readout(&history[..=t], hidden_dim, decay);
            assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(want.iter()) {
                assert!(
                    (g - w).abs() < 1e-9,
                    "step {t}: recurrent {g} vs naive {w}"
                );
            }
        }
        assert_eq!(st.tokens, 8);
    }

    #[test]
    fn decay_bounds_clamp() {
        assert_eq!(clamp_gamma(0.0), GAMMA_LO);
        assert_eq!(clamp_gamma(-5.0), GAMMA_LO);
        assert_eq!(clamp_gamma(0.90), 0.90);
        assert_eq!(clamp_gamma(1.0), GAMMA_HI);
        assert_eq!(clamp_gamma(5.0), GAMMA_HI);
        // Behavioral: below-range decay acts as 0.90, above-range as 1.0.
        // Two steps are needed: on a fresh (zero) state gamma is unobservable.
        let x1 = vec![0.5, 0.25, -1.0, 0.75];
        let x2 = vec![-0.3, 0.8, 0.4, -0.6];
        let mut a = Rank5State::new(4);
        let mut b = Rank5State::new(4);
        let _ = a.step_and_readout(&x1, 0.0);
        let _ = b.step_and_readout(&x1, 0.90);
        assert_eq!(a.step_and_readout(&x2, 0.0), b.step_and_readout(&x2, 0.90));
        let mut c = Rank5State::new(4);
        let mut d = Rank5State::new(4);
        let _ = c.step_and_readout(&x1, 99.0);
        let _ = d.step_and_readout(&x1, 1.0);
        assert_eq!(c.step_and_readout(&x2, 99.0), d.step_and_readout(&x2, 1.0));
    }
}
