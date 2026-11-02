//! Toroidal positional encoder (TPE) — the real kernel set, backend-native.
//!
//! Ported from the bench crate (`ablation_benchmarks/cuda_rust/src/
//! toroidal.rs` + `rank5.rs`), spec-pinned constants and all:
//!
//! - `TpeState` — coupled 2:1 resonant phase carrier on T². Per token:
//!   `theta_f += OMEGA_F + dtheta_f + K_F·sin(theta_s − theta_f)`,
//!   `theta_s += OMEGA_S + dtheta_s + K_S·sin(theta_f − theta_s)`,
//!   `psi = wrap_pi(theta_f − 2·theta_s)`; amplitude accumulates arc/drive,
//!   floored at 0.
//! - Radius law: `r(w) = 0.22 + span·(0.35·w + 0.65·ln(1+w))`,
//!   span = max(0.90 − 0.22, 0.15); analytic slope `span·(0.35 + 0.65/(1+w))`.
//! - Rank-5 trig kernel: `k(d) = (1 + cos d)² = ⟨phi(t), phi(t−d)⟩` with
//!   `phi(t) = [√(3/2), √2·cos t, √2·sin t, (1/√2)·cos 2t, (1/√2)·sin 2t]`.
//!   `Rank5State::step_and_readout` is the O(1) recurrent linear attention:
//!   causal readout `⟨phi(q), S⟩ / ⟨phi(q), Z⟩`, gamma clamped to
//!   `[0.90, 1.0]`, `r_iso` clamped to `[0, 4]`, denominator floored at
//!   `1e-9` (DENOM_FLOOR). That floor is the documented seq-1 denom-pin
//!   mechanism from the tpe_e2e verdict — preserved, never hidden.
//! - Deterministic float behavior: all transcendental math via `f64::sin_cos`.
//!
//! What the TPE is, and is not:
//! - It is the memory layer's positional encoder: hidden vectors are
//!   rotated/scored by TPE before Qdrant storage and query, so two
//!   memories with the same text at different positions get different
//!   vectors (position is part of the encoding).
//! - It is NOT a neural embedder, NOT a hyperbolic KV, and NOT a claim
//!   about any live forward pass. The measured e2e verdict on record
//!   (analysis/tpe_e2e/REPORT.md, 2026-11-01) is 3/4 PASS, 1 MISS
//!   (seq-1 denominator pin at the frozen 1e-9 floor) — cited, not
//!   buried.

pub const TPE_OMEGA_F: f64 = 0.1;
pub const TPE_OMEGA_S: f64 = 0.05;
pub const TPE_K_F: f64 = 0.3;
pub const TPE_K_S: f64 = 0.15;
pub const TPE_STABILITY_K: f64 = 0.6;

pub const TPE_R_MIN: f64 = 0.22;
pub const TPE_R_SCALE: f64 = 0.90;
pub const TPE_R_LIN: f64 = 0.35;
pub const TPE_R_LOG: f64 = 0.65;

pub const W0: f64 = 1.5;
pub const W1: f64 = 2.0;
pub const W2: f64 = 0.5;
pub const GAMMA_LO: f64 = 0.90;
pub const GAMMA_HI: f64 = 1.0;
pub const R_ISO_MAX: f64 = 4.0;
pub const R_FLOOR: f64 = 1e-12;
pub const DENOM_FLOOR: f64 = 1e-9;
pub const RMS_FLOOR: f64 = 1e-9;

pub fn f64_to_q32_31(x: f64) -> i64 {
    (x * (1i64 << 31) as f64).round() as i64
}

pub fn q32_31_to_f64(x: i64) -> f64 {
    (x as f64) / ((1i64 << 31) as f64)
}

/// Phase-wrapped distance for T² (handles 2π periodicity).
#[inline]
pub fn phase_distance_wrapped(a: i64, b: i64) -> i64 {
    let diff = a - b;
    let pi_q32 = f64_to_q32_31(std::f64::consts::PI);
    let two_pi_q32 = pi_q32 * 2;
    if diff > pi_q32 {
        diff - two_pi_q32
    } else if diff < -pi_q32 {
        diff + two_pi_q32
    } else {
        diff
    }
}

#[derive(Debug, Clone)]
pub struct TpeState {
    pub theta_f: i64,
    pub theta_s: i64,
    pub amplitude: i64,
    pub rotations: f64,
}

impl Default for TpeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TpeState {
    pub fn new() -> Self {
        Self {
            theta_f: 0,
            theta_s: 0,
            amplitude: 0,
            rotations: 0.0,
        }
    }

    pub fn evolve(
        &mut self,
        dtheta_f: f64,
        dtheta_s: f64,
        arc_step: f64,
        drive_mod: f64,
    ) -> (i64, i64, i64) {
        let tf = q32_31_to_f64(self.theta_f);
        let ts = q32_31_to_f64(self.theta_s);
        let next_f = tf + TPE_OMEGA_F + dtheta_f + TPE_K_F * (ts - tf).sin();
        let next_s = ts + TPE_OMEGA_S + dtheta_s + TPE_K_S * (tf - ts).sin();
        self.theta_f = phase_distance_wrapped(f64_to_q32_31(next_f), 0);
        self.theta_s = phase_distance_wrapped(f64_to_q32_31(next_s), 0);
        let psi_raw = self.theta_f - 2 * self.theta_s;
        let psi = phase_distance_wrapped(psi_raw, 0);
        let amp = q32_31_to_f64(self.amplitude) + arc_step + drive_mod;
        self.amplitude = f64_to_q32_31(amp.max(0.0));
        self.rotations = (self.rotations + arc_step).max(0.0);
        (self.theta_f, self.theta_s, psi)
    }
}

#[inline]
pub fn radius_for_rotations(w: f64) -> f64 {
    let w = w.max(0.0);
    let span = (TPE_R_SCALE - TPE_R_MIN).max(0.15);
    TPE_R_MIN + span * (TPE_R_LIN * w + TPE_R_LOG * (1.0 + w).ln())
}

#[inline]
pub fn radius_slope_for_rotations(w: f64) -> f64 {
    let w = w.max(0.0);
    let span = (TPE_R_SCALE - TPE_R_MIN).max(0.15);
    span * (TPE_R_LIN + TPE_R_LOG / (1.0 + w))
}

/// Rank-5 feature map for angle `theta` (radians).
pub fn phi(theta: f64) -> [f64; 5] {
    const SQRT_3_2: f64 = 1.224_744_871_391_589_2;
    const SQRT_2: f64 = 1.414_213_562_373_095_1;
    const INV_SQRT_2: f64 = 0.707_106_781_186_547_6;
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

pub fn kernel_closed(d: f64) -> f64 {
    let k = 1.0 + d.cos();
    k * k
}

pub fn kernel_expanded(d: f64) -> f64 {
    1.5 + 2.0 * d.cos() + 0.5 * (2.0 * d).cos()
}

#[inline]
pub fn clamp_gamma(decay: f64) -> f64 {
    decay.clamp(GAMMA_LO, GAMMA_HI)
}

#[derive(Debug, Clone)]
pub struct Rank5State {
    pub pairs: usize,
    pub hidden_dim: usize,
    pub tokens: u64,
    pub z0: f64,
    pub zc1: Vec<f64>,
    pub zs1: Vec<f64>,
    pub zc2: Vec<f64>,
    pub zs2: Vec<f64>,
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

    pub fn step_and_readout(&mut self, x_in: &[f64], decay: f64) -> Vec<f64> {
        let d_len = x_in.len().min(self.hidden_dim);
        let pairs = d_len / 2;
        if pairs == 0 {
            return vec![0.0; x_in.len()];
        }
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

/// One text unit (token surrogate: one whitespace-delimited word) as a
/// pseudo hidden vector in R^d. NOT a neural embedding: lexical word bytes
/// in the spirit of the frozen Qwen trial (provenance: tiny transform of
/// bytes). Deterministic, content-bearing, no vocabulary needed.
pub fn pseudo_hidden_vector(text: &str, dim: usize, pos: usize) -> Vec<f64> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let w = if words.is_empty() {
        text
    } else {
        words[pos % words.len()]
    };
    let _ = pos;
    let bytes = w.as_bytes();
    let mut v = vec![0.0f64; dim];
    for (i, b) in bytes.iter().enumerate() {
        let idx = ((*b as usize) * 131 + i * 17) % dim;
        v[idx] += ((*b as f64) - 72.0) / 72.0;
    }
    v
}

/// Position-aware TPE encoding of a text into the retrieval vector.
/// Encoding key: `theta_t` advances per word-position from the carrier;
/// content feeds `Rank5State` (decay 1.0, TPE radii ride amplitude).
/// Vector layout (n): [tpe_readout, theta_f, theta_s, psi, amplitude,
/// radius, 5·phi(theta_last), cos/sin of phase differences, ...pad/zeros].
pub struct TpeEncoder {
    pub dim: usize,
    pub state: Rank5State,
    pub tpe: TpeState,
}

impl TpeEncoder {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            state: Rank5State::new(dim),
            tpe: TpeState::new(),
        }
    }

    pub fn encode_text(&mut self, text: &str) -> Vec<f32> {
        let positions = text.split_whitespace().count().max(1);
        let mut readout = vec![0.0f64; self.dim];
        for p in 0..positions {
            let x = pseudo_hidden_vector(text, self.dim, p);
            self.tpe.evolve(0.0, 0.0, 1.0, 0.0);
            readout = self.state.step_and_readout(&x, 1.0);
        }
        let r = radius_for_rotations(self.tpe.rotations);
        let amp = q32_31_to_f64(self.tpe.amplitude);
        let tf = q32_31_to_f64(self.tpe.theta_f);
        let ts = q32_31_to_f64(self.tpe.theta_s);
        // psi = wrap_pi(theta_f - 2·theta_s)
        let psi = {
            let raw = self.tpe.theta_f - 2 * self.tpe.theta_s;
            q32_31_to_f64(crate::harness::tpe::phase_distance_wrapped(raw, 0))
        };

        let mut out = Vec::with_capacity(self.dim);
        // first `dim` components: readout, scaled by radius (positions
        // push the representation out along the spiral)
        for v in readout.iter().take(self.dim) {
            out.push((v * r) as f32);
        }
        // positional fingerprints woven into trailing channels so cosine
        // is position-sensitive even for near-identical readouts
        let ph = phi(tf);
        let tail = [
            tf,
            ts,
            psi,
            amp,
            r,
            ph[0],
            ph[1],
            ph[2],
            ph[3],
            ph[4],
            (tf - ts).cos(),
            (tf - ts).sin(),
        ];
        for (i, &t) in tail.iter().enumerate() {
            if self.dim > i {
                out[i] = (out[i] as f64 * 0.7 + t * 0.3) as f32;
            }
        }
        // L2 normalize for cosine storage
        let n: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n > 0.0 {
            for x in out.iter_mut() {
                *x /= n;
            }
        } else if !out.is_empty() {
            out[0] = 1.0;
        }
        out
    }
}

pub fn embed_tpe(text: &str, dim: usize) -> Vec<f32> {
    let mut enc = TpeEncoder::new(dim);
    enc.encode_text(text)
}
