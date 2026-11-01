//! Toroidal Phase Operations (T²)
//!
//! Implements toroidal phase coordinates and distance metrics for H³×T² joint retrieval.

use super::hyperbolic::{f64_to_q32_31, q32_31_to_f64, ToroidalPhase};

/// Generate random toroidal phase
pub fn random_toroidal_phase(rng: &mut impl rand::Rng) -> ToroidalPhase {
    use rand_distr::{Distribution, Uniform};
    let dist =
        Uniform::new(-std::f64::consts::PI, std::f64::consts::PI).expect("valid uniform range");
    ToroidalPhase {
        theta_f: f64_to_q32_31(dist.sample(rng)),
        theta_s: f64_to_q32_31(dist.sample(rng)),
    }
}

/// Phase coherence check for retrieval validation
pub fn phase_coherence(
    query: &ToroidalPhase,
    candidates: &[ToroidalPhase],
    threshold: f64,
) -> Vec<bool> {
    candidates
        .iter()
        .map(|c| {
            let df = (q32_31_to_f64(query.theta_f - c.theta_f)).abs();
            let ds = (q32_31_to_f64(query.theta_s - c.theta_s)).abs();
            let coherence = (df.cos() + ds.cos()) / 2.0;
            coherence > threshold
        })
        .collect()
}

/// Phase-wrapped distance for T² (handles 2π periodicity)
#[inline]
pub fn phase_distance_wrapped(a: i64, b: i64) -> i64 {
    let diff = a - b;
    let pi_q32 = f64_to_q32_31(std::f64::consts::PI);
    let two_pi_q32 = pi_q32 * 2;

    // Wrap to [-π, π]
    let wrapped = if diff > pi_q32 {
        diff - two_pi_q32
    } else if diff < -pi_q32 {
        diff + two_pi_q32
    } else {
        diff
    };
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_random_toroidal_phase() {
        let mut rng = StdRng::seed_from_u64(42);
        let phase = random_toroidal_phase(&mut rng);
        assert!(q32_31_to_f64(phase.theta_f).abs() <= std::f64::consts::PI);
        assert!(q32_31_to_f64(phase.theta_s).abs() <= std::f64::consts::PI);
    }

    #[test]
    fn test_phase_distance_wrapped() {
        let pi = f64_to_q32_31(std::f64::consts::PI);
        let half_pi = f64_to_q32_31(std::f64::consts::FRAC_PI_2);

        // Distance between π and -π should be 0 (same point)
        assert_eq!(phase_distance_wrapped(pi, -pi), 0);

        // Distance between pi/2 and -pi/2 is pi in magnitude (boundary antipode)
        let dist = phase_distance_wrapped(half_pi, -half_pi);
        assert!(dist.abs() - pi < 1000);
    }

    #[test]
    fn test_wrapped_inputs_agree_with_raw_on_cos_path() {
        // Iron-out 2026-10-31: phase_coherence + distance_sq consume raw
        // deltas (no wrap call). cos() is even + 2PI-periodic so coherence
        // scores are unaffected; this pins that unwrapped vs wrapped deltas
        // agree through cos to 1e-9 over the full [-2PI, 2PI] delta range.
        let cases = [
            -6.0f64,
            -3.5,
            -3.14159265,
            -1.0,
            0.0,
            1.0,
            3.14159265,
            3.5,
            6.0,
        ];
        for d in cases {
            let wrapped =
                q32_31_to_f64(phase_distance_wrapped(f64_to_q32_31(d), f64_to_q32_31(0.0)));
            assert!((d.cos() - wrapped.cos()).abs() < 1e-9, "delta {}", d);
        }
    }

    #[test]
    fn test_phase_coherence() {
        let query = ToroidalPhase {
            theta_f: f64_to_q32_31(0.0),
            theta_s: f64_to_q32_31(0.0),
        };
        let candidates = vec![
            ToroidalPhase {
                theta_f: f64_to_q32_31(0.1),
                theta_s: f64_to_q32_31(0.1),
            },
            ToroidalPhase {
                theta_f: f64_to_q32_31(3.0),
                theta_s: f64_to_q32_31(3.0),
            },
        ];
        let results = phase_coherence(&query, &candidates, 0.9);
        assert!(results[0]); // Close phase -> coherent
        assert!(!results[1]); // Far phase -> not coherent
    }
}

// -- TPE HARD-PART fiber (append-only, 2026-10-31) -------------------------
// Frost law: byte-identical constants from
// MasterDesignDocument/C06-Frontier-Push/TorodialPhasePositionalEncoder-Spec.md
// sections 1-4 and tpe_fep_budget.py (R_MIN=0.22, R_SCALE=0.90). No deviation.

/// TPE carrier defaults (spec section 2; vault torus.rs conformance).
pub const TPE_OMEGA_F: f64 = 0.1;
pub const TPE_OMEGA_S: f64 = 0.05;
pub const TPE_K_F: f64 = 0.3;
pub const TPE_K_S: f64 = 0.15;
/// Stability constant K = k_f + 2*k_s = 0.6 (freeze Prop 2).
pub const TPE_STABILITY_K: f64 = 0.6;

/// Radius law constants (tpe_fep_budget.py: R_MIN=0.22, R_SCALE=0.90).
pub const TPE_R_MIN: f64 = 0.22;
pub const TPE_R_SCALE: f64 = 0.90;
pub const TPE_R_LIN: f64 = 0.35;
pub const TPE_R_LOG: f64 = 0.65;

/// Rapidity law: eta = 0.22 + 0.38*w + 0.28*amp + 0.20*ln(1+rms) (spec 4.2).
pub const TPE_ETA_0: f64 = 0.22;
pub const TPE_ETA_W: f64 = 0.38;
pub const TPE_ETA_AMP: f64 = 0.28;
pub const TPE_ETA_RMS: f64 = 0.20;

/// TPE dynamical state. Phases + amplitude stored Q32.31 (i64);
/// rotations `w` is cumulative arc (f64, advanced by arc_step each evolve).
pub struct TpeState {
    pub theta_f: i64,
    pub theta_s: i64,
    pub amplitude: i64,
    pub rotations: f64,
}

impl TpeState {
    /// Zeros, amp 0.
    pub fn new() -> Self {
        Self {
            theta_f: 0,
            theta_s: 0,
            amplitude: 0,
            rotations: 0.0,
        }
    }

    /// One coupled 2:1 resonant step (spec section 2):
    /// theta_f += omega_f + dtheta_f + k_f*sin(theta_s - theta_f);
    /// theta_s += omega_s + dtheta_s + k_s*sin(theta_f - theta_s);
    /// psi = wrap_pi(theta_f - 2*theta_s);
    /// amp += arc_step + drive_mod, floored at 0.
    ///
    /// Phases convert to f64, advance, then re-wrap to [-pi,pi] via the
    /// file's existing single-wrap `phase_distance_wrapped`. Valid while one
    /// step advances well under 2pi (prior state is wrapped; omega+k<=0.4).
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

/// Radius law (spec 4.1): r(w) = r_min + span*(0.35*w + 0.65*ln(1+w)),
/// span = max(R_SCALE - R_MIN, 0.15), w clamped >= 0.
#[inline]
pub fn radius_for_rotations(w: f64) -> f64 {
    let w = w.max(0.0);
    let span = (TPE_R_SCALE - TPE_R_MIN).max(0.15);
    TPE_R_MIN + span * (TPE_R_LIN * w + TPE_R_LOG * (1.0 + w).ln())
}

/// Analytic dr/dw (spec 4.1 proof): span*(0.35 + 0.65/(1+w)).
#[inline]
pub fn radius_slope_for_rotations(w: f64) -> f64 {
    let w = w.max(0.0);
    let span = (TPE_R_SCALE - TPE_R_MIN).max(0.15);
    span * (TPE_R_LIN + TPE_R_LOG / (1.0 + w))
}

/// Rapidity law (spec 4.2): eta = 0.22 + 0.38*w + 0.28*amp + 0.20*ln(1+rms).
#[inline]
pub fn rapidity_for_state(w: f64, amp: f64, rms: f64) -> f64 {
    TPE_ETA_0 + TPE_ETA_W * w + TPE_ETA_AMP * amp + TPE_ETA_RMS * (1.0 + rms).ln()
}

/// Rank-3 feature map (freeze Thm 1): phi = [1,cos,sin]/sqrt(2).
#[inline]
pub fn tpe_phi(theta: f64) -> [f64; 3] {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    [s, theta.cos() * s, theta.sin() * s]
}

/// Coherence C = phi(a) . phi(b) = 0.5 + 0.5*cos(a-b).
#[inline]
pub fn tpe_coherence(a: f64, b: f64) -> f64 {
    let pa = tpe_phi(a);
    let pb = tpe_phi(b);
    pa[0] * pb[0] + pa[1] * pb[1] + pa[2] * pb[2]
}

/// Linear scan step, d_v = 1 (freeze section 6): S is kept 3x1 in column 0,
/// S += outer(phi, v); z += phi; y = S^T phi / max(z^T phi, 1).
/// Exact normalization variant: denominator >= 1, no softmax ever.
#[inline]
pub fn tpe_scan_step(s: &mut [[f64; 3]; 3], z: &mut [f64; 3], phi: [f64; 3], v: f64) -> f64 {
    for r in 0..3 {
        s[r][0] += phi[r] * v;
        z[r] += phi[r];
    }
    let num: f64 = (0..3).map(|r| s[r][0] * phi[r]).sum();
    let den: f64 = (0..3).map(|r| z[r] * phi[r]).sum();
    num / den.max(1.0)
}

#[cfg(test)]
mod tpe_tests {
    use super::*;

    #[test]
    fn tpe_spiral_monotonicity() {
        // Gate 11: r strictly increasing over 1e3 rotations; numeric dr/dw
        // matches the analytic slope to 1e-6.
        let mut prev = radius_for_rotations(0.0);
        for w in 1..=1000 {
            let r = radius_for_rotations(w as f64);
            assert!(r > prev, "not spiral-out at w={}", w);
            prev = r;
        }
        for w in [0.0, 1.0, 4.0, 16.0, 100.0, 999.0] {
            let h = 1e-5;
            let num = if w - h < 0.0 {
                (-3.0 * radius_for_rotations(w) + 4.0 * radius_for_rotations(w + h)
                    - radius_for_rotations(w + 2.0 * h))
                    / (2.0 * h)
            } else {
                (radius_for_rotations(w + h) - radius_for_rotations(w - h)) / (2.0 * h)
            };
            let ana = radius_slope_for_rotations(w);
            assert!(
                (num - ana).abs() < 1e-6,
                "slope mismatch at w={}: {} vs {}",
                w,
                num,
                ana
            );
            assert!(ana > 0.0, "slope must stay positive");
        }
    }

    #[test]
    fn tpe_no_corkscrew() {
        // Gate 12: radius variance across a trajectory exceeds the Q32.31
        // quantization noise floor (variance of one LSB ~ 2.2e-19) by >> 1e3x.
        // A constant-radius (corkscrew) implementation FAILS this.
        let n = 100u32;
        let mut mean = 0.0;
        let mut m2 = 0.0;
        for i in 0..=n {
            let r = radius_for_rotations(i as f64);
            let d = r - mean;
            mean += d / (i as f64 + 1.0);
            m2 += d * (r - mean);
        }
        let var = m2 / (n as f64 + 1.0);
        let q_lsb = 2f64.powi(-31);
        assert!(
            var > 1e3 * q_lsb * q_lsb,
            "radius looks constant, var={}",
            var
        );
        assert!(var > 1e-3, "var={} far below honest margin", var);
    }

    #[test]
    fn tpe_drift_free_identity() {
        // Zero drives: phases stay wrapped, psi stays bounded.
        let mut st = TpeState::new();
        let pi = std::f64::consts::PI;
        for _ in 0..2000 {
            let (pf, ps, psi) = st.evolve(0.0, 0.0, 0.0, 0.0);
            assert!(q32_31_to_f64(pf).abs() <= pi + 1e-9);
            assert!(q32_31_to_f64(ps).abs() <= pi + 1e-9);
            assert!(q32_31_to_f64(psi).abs() <= pi + 1e-9);
        }
        assert_eq!(q32_31_to_f64(st.amplitude), 0.0);
        assert_eq!(st.rotations, 0.0);
        // Amplitude floor: negative drive energy cannot drive amp below 0.
        st.evolve(0.0, 0.0, 0.0, -5.0);
        assert_eq!(q32_31_to_f64(st.amplitude), 0.0);
        // Positive accumulation is monotone non-decreasing.
        st.evolve(0.0, 0.0, 0.3, 0.2);
        let a1 = q32_31_to_f64(st.amplitude);
        st.evolve(0.0, 0.0, 0.1, 0.0);
        let a2 = q32_31_to_f64(st.amplitude);
        assert!((a1 - 0.5).abs() < 1e-9);
        assert!(a2 >= a1);
    }

    #[test]
    fn tpe_radius_matches_python() {
        // python tpe_fep_budget.py: r(4)=1.88337, r(16)=5.28028 (gate tol 0.05).
        // Asserted here at 1e-9 to pin byte-identical constants.
        assert!((radius_for_rotations(4.0) - 1.8833715572958725).abs() < 1e-9);
        assert!((radius_for_rotations(16.0) - 5.280280298072847).abs() < 1e-9);
    }

    #[test]
    fn tpe_phi_identity() {
        // Freeze Thm 1: dot(phi(a),phi(b)) = 0.5 + 0.5*cos(a-b).
        for i in 0..16 {
            let a = i as f64 * 0.37;
            let b = -(i as f64) * 0.53 + 0.11;
            let got = tpe_coherence(a, b);
            let want = 0.5 + 0.5 * (a - b).cos();
            assert!((got - want).abs() < 1e-12, "a={} b={}", a, b);
        }
        assert!((tpe_coherence(1.23, 1.23) - 1.0).abs() < 1e-12);
        assert!(tpe_coherence(0.0, std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn tpe_scan_equivalence() {
        // Stepwise tpe_scan_step output matches a naive from-scratch
        // recomputation (S = sum phi_i v_i, z = sum phi_i) over 8 steps.
        let mut s = [[0.0f64; 3]; 3];
        let mut z = [0.0f64; 3];
        let mut phis: Vec<[f64; 3]> = Vec::new();
        let mut vs: Vec<f64> = Vec::new();
        for i in 0..8 {
            let phi = tpe_phi(i as f64 * 0.7);
            let v = (i as f64).sin() * 2.0 + 0.5;
            phis.push(phi);
            vs.push(v);
            let y = tpe_scan_step(&mut s, &mut z, phi, v);
            let mut sr = [0.0f64; 3];
            let mut zr = [0.0f64; 3];
            for (p, vv) in phis.iter().zip(vs.iter()) {
                for r in 0..3 {
                    sr[r] += p[r] * vv;
                    zr[r] += p[r];
                }
            }
            let num: f64 = (0..3).map(|r| sr[r] * phi[r]).sum();
            let den: f64 = (0..3).map(|r| zr[r] * phi[r]).sum();
            let y_ref = num / den.max(1.0);
            assert!((y - y_ref).abs() < 1e-12, "step {}: {} vs {}", i, y, y_ref);
        }
    }
}
