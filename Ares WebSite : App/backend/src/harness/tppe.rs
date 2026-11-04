//! TPPE geometry primitives — verified kernel core (candidate identity).
//!
//! PROVENANCE (Commandments II/III): logic byte-copied from
//! `globalmind-runtime/src/tppe.rs`, sha256
//! `2af606fe7eeaae3d9664db755a2189ba85d3495628a5e62b0181c624c00af298`,
//! named by `contracts/tppe-layer3-candidate-v2.contract`
//! (`status=component_candidate_only`). This module does not select a
//! winner or qualify TPPE for model use. Candidate selection remains a
//! real-trace bakeoff decision; the 2026-11-02 held-out bakeoff returned
//! OPEN (promoted=none, radial tie). The geometry-law code below is the
//! source's code verbatim; only the header and the test gating differ
//! (source tests run behind `#[cfg(test)]` here too). Layer A (math)
//! per the kernel doctrine: identities proved, never benchmarked as
//! quality evidence.
//!
//! DERIVATIONS (M1, before code — the code below IS the frozen law):
//! 1. Derived revolutions: w = (theta_f + theta_s)/TAU. Never an input.
//!    Differentiating: dw/dstep = (Dtheta_f + Dtheta_s)/TAU.
//! 2. Strict radial expansion, worst case sin(psi)=1 (NOT psi=+/-pi,
//!    where sin=0): Dtheta_f >= drive_f + omega_f - K_f > 0 and
//!    Dtheta_s >= drive_s + omega_s - K_s > 0 under the pinned
//!    constants, so w strictly increases each step; with
//!    growth'(w) = 0.35 + 0.65/(1+w) > 0 (linear_log_v1), u and
//!    rho = tanh(u/2) strictly increase. `step` enforces this and
//!    fails closed. Zero-advance steps (Dw=0) are accepted by `step`;
//!    the pinned drives never produce them.
//! 3. Lorentz sheet: x0 = cosh(u), ||x_spatial|| = sinh(u), so
//!    x0^2 - ||x||^2 = cosh^2(u) - sinh^2(u) = 1. `validate` checks
//!    this identity numerically plus tanh/boundary consistency; a
//!    forged coordinate fails closed.
//! 4. Boundary: rho = tanh(u/2) in [0,1), margin = 1 - rho > 0.
//!    Saturation to rho = 1 is rejected, never clamped (source test
//!    `represented_boundary_saturation_fails_instead_of_clamping`).

use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadialLaw {
    LinearLogV1,
    LogV1,
}

impl RadialLaw {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LinearLogV1 => "linear_log_v1",
            Self::LogV1 => "log_v1",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TppeConfig {
    pub law: RadialLaw,
    pub radial_start: f64,
    pub radial_span: f64,
    pub omega_fast: f64,
    pub omega_slow: f64,
    pub coupling_fast: f64,
    pub coupling_slow: f64,
}

impl TppeConfig {
    pub fn validate(self) -> Result<Self, String> {
        let values = [
            self.radial_start,
            self.radial_span,
            self.omega_fast,
            self.omega_slow,
            self.coupling_fast,
            self.coupling_slow,
        ];
        if !values.iter().all(|value| value.is_finite()) {
            return Err("TPPE configuration contains a non-finite value".into());
        }
        if self.radial_start < 0.0 || self.radial_span <= 0.0 {
            return Err("TPPE radial_start must be nonnegative and radial_span positive".into());
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TppeState {
    pub theta_fast_unwrapped: f64,
    pub theta_slow_unwrapped: f64,
    pub cumulative_revolutions: f64,
    pub hyperbolic_radius: f64,
    pub poincare_radius: f64,
    pub boundary_margin: f64,
    pub hyperboloid_time: f64,
    pub hyperboloid_spatial_norm: f64,
}

impl TppeState {
    pub fn origin(config: TppeConfig) -> Result<Self, String> {
        let config = config.validate()?;
        Self::from_components(config, 0.0, 0.0)
    }

    pub fn phase_t2(self) -> [f64; 2] {
        [
            self.theta_fast_unwrapped.rem_euclid(TAU),
            self.theta_slow_unwrapped.rem_euclid(TAU),
        ]
    }

    /// Cumulative revolutions are *derived* from the unwrapped phases:
    /// `w = (theta_fast + theta_slow) / TAU`, the total unwrapped phase
    /// advance in turns since the origin. Revolution count is never an
    /// independent input; supplying it separately would decouple the
    /// radial law from the phase turns it is defined over.
    pub fn derived_revolutions(theta_fast_unwrapped: f64, theta_slow_unwrapped: f64) -> f64 {
        (theta_fast_unwrapped + theta_slow_unwrapped) / TAU
    }

    pub fn step(
        self,
        config: TppeConfig,
        drive_fast: f64,
        drive_slow: f64,
    ) -> Result<Self, String> {
        let config = config.validate()?;
        if !drive_fast.is_finite() || !drive_slow.is_finite() {
            return Err("TPPE step drives must be finite".into());
        }
        self.validate()?;
        let psi = self.theta_fast_unwrapped - 2.0 * self.theta_slow_unwrapped;
        let next_fast = self.theta_fast_unwrapped + drive_fast + config.omega_fast
            - config.coupling_fast * psi.sin();
        let next_slow = self.theta_slow_unwrapped
            + drive_slow
            + config.omega_slow
            + config.coupling_slow * psi.sin();
        let next = Self::from_components(config, next_fast, next_slow)?;
        if next.cumulative_revolutions < self.cumulative_revolutions {
            return Err(
                "TPPE step retreated in unwrapped phase; cumulative revolutions must not decrease"
                    .into(),
            );
        }
        if next.cumulative_revolutions > self.cumulative_revolutions
            && (next.hyperbolic_radius <= self.hyperbolic_radius
                || next.poincare_radius <= self.poincare_radius)
        {
            return Err("TPPE radial law failed strict spiral expansion".into());
        }
        Ok(next)
    }

    pub fn validate(self) -> Result<(), String> {
        let values = [
            self.theta_fast_unwrapped,
            self.theta_slow_unwrapped,
            self.cumulative_revolutions,
            self.hyperbolic_radius,
            self.poincare_radius,
            self.boundary_margin,
            self.hyperboloid_time,
            self.hyperboloid_spatial_norm,
        ];
        if !values.iter().all(|value| value.is_finite()) {
            return Err("TPPE state contains a non-finite value".into());
        }
        if self.cumulative_revolutions < 0.0
            || self.hyperbolic_radius < 0.0
            || !(0.0..1.0).contains(&self.poincare_radius)
            || self.boundary_margin <= 0.0
        {
            return Err("TPPE state violates radial or ideal-boundary invariants".into());
        }
        let consistent = |actual: f64, expected: f64| {
            (actual - expected).abs() <= 64.0 * f64::EPSILON * expected.abs().max(1.0)
        };
        if !consistent(
            self.cumulative_revolutions,
            Self::derived_revolutions(self.theta_fast_unwrapped, self.theta_slow_unwrapped),
        ) || !consistent(self.poincare_radius, (self.hyperbolic_radius / 2.0).tanh())
            || !consistent(self.boundary_margin, 1.0 - self.poincare_radius)
            || !consistent(self.hyperboloid_time, self.hyperbolic_radius.cosh())
            || !consistent(self.hyperboloid_spatial_norm, self.hyperbolic_radius.sinh())
        {
            return Err("TPPE state coordinates are inconsistent".into());
        }
        let lorentz = self.hyperboloid_time * self.hyperboloid_time
            - self.hyperboloid_spatial_norm * self.hyperboloid_spatial_norm;
        let tolerance = 64.0 * f64::EPSILON * self.hyperboloid_time.powi(2).max(1.0);
        if (lorentz - 1.0).abs() > tolerance {
            return Err("TPPE state violates the unit Lorentz-sheet invariant".into());
        }
        Ok(())
    }

    fn from_components(
        config: TppeConfig,
        theta_fast_unwrapped: f64,
        theta_slow_unwrapped: f64,
    ) -> Result<Self, String> {
        let cumulative_revolutions =
            Self::derived_revolutions(theta_fast_unwrapped, theta_slow_unwrapped);
        if !cumulative_revolutions.is_finite() || cumulative_revolutions < 0.0 {
            return Err("TPPE cumulative revolutions must be finite and nonnegative".into());
        }
        let growth = match config.law {
            RadialLaw::LinearLogV1 => {
                0.35 * cumulative_revolutions + 0.65 * cumulative_revolutions.ln_1p()
            }
            RadialLaw::LogV1 => cumulative_revolutions.ln_1p(),
        };
        let hyperbolic_radius = config.radial_start + config.radial_span * growth;
        let poincare_radius = (hyperbolic_radius / 2.0).tanh();
        let boundary_margin = 1.0 - poincare_radius;
        let hyperboloid_time = hyperbolic_radius.cosh();
        let hyperboloid_spatial_norm = hyperbolic_radius.sinh();
        let state = Self {
            theta_fast_unwrapped,
            theta_slow_unwrapped,
            cumulative_revolutions,
            hyperbolic_radius,
            poincare_radius,
            boundary_margin,
            hyperboloid_time,
            hyperboloid_spatial_norm,
        };
        state.validate()?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(law: RadialLaw) -> TppeConfig {
        TppeConfig {
            law,
            radial_start: 0.0,
            radial_span: 0.01,
            omega_fast: 0.1,
            omega_slow: 0.05,
            coupling_fast: 0.3,
            coupling_slow: 0.15,
        }
    }

    #[test]
    fn both_candidates_spiral_and_preserve_geometry() {
        for law in [RadialLaw::LinearLogV1, RadialLaw::LogV1] {
            let cfg = config(law);
            let mut state = TppeState::origin(cfg).unwrap();
            for _ in 0..1_000 {
                let next = state.step(cfg, 0.02, -0.01).unwrap();
                assert!(next.hyperbolic_radius > state.hyperbolic_radius);
                assert!(next.poincare_radius > state.poincare_radius);
                assert!(next.boundary_margin > 0.0);
                state = next;
            }
        }
    }

    #[test]
    fn revolutions_are_derived_from_unwrapped_phase() {
        let cfg = config(RadialLaw::LinearLogV1);
        let mut state = TppeState::origin(cfg).unwrap();
        assert_eq!(state.cumulative_revolutions, 0.0);
        for _ in 0..100 {
            state = state.step(cfg, 0.02, -0.01).unwrap();
            let expected = TppeState::derived_revolutions(
                state.theta_fast_unwrapped,
                state.theta_slow_unwrapped,
            );
            assert_eq!(state.cumulative_revolutions, expected);
            assert!(state.cumulative_revolutions > 0.0);
        }
    }

    #[test]
    fn phase_retreat_fails_closed() {
        let cfg = config(RadialLaw::LogV1);
        let state = TppeState::origin(cfg).unwrap();
        // Net unwrapped advance is negative: -(omega sum) dominates.
        assert!(state.step(cfg, -10.0, -10.0).is_err());
    }

    #[test]
    fn represented_boundary_saturation_fails_instead_of_clamping() {
        for law in [RadialLaw::LinearLogV1, RadialLaw::LogV1] {
            let cfg = TppeConfig {
                radial_span: match law {
                    RadialLaw::LinearLogV1 => 2.0,
                    RadialLaw::LogV1 => 5.0,
                },
                ..config(law)
            };
            let mut state = TppeState::origin(cfg).unwrap();
            // Drive derived revolutions outward. The last admitted state must
            // be finite and interior; the first boundary failure is rejected.
            let mut rejected = None;
            for step in 0..2_000 {
                match state.step(cfg, 100.0, 100.0) {
                    Ok(next) => {
                        assert!(next.hyperbolic_radius > state.hyperbolic_radius);
                        assert!(next.poincare_radius > state.poincare_radius);
                        assert!(next.boundary_margin > 0.0);
                        state = next;
                    }
                    Err(error) => {
                        rejected = Some((step, error));
                        break;
                    }
                }
            }
            let (step, error) = rejected.expect("boundary saturation must fail closed");
            eprintln!(
                "law={} first_rejected_step={} last_u={:.17} last_rho={:.17} last_margin={:.17} error={}",
                law.as_str(),
                step,
                state.hyperbolic_radius,
                state.poincare_radius,
                state.boundary_margin,
                error
            );
        }
    }

    #[test]
    fn invalid_inputs_fail_closed() {
        let cfg = config(RadialLaw::LogV1);
        let state = TppeState::origin(cfg).unwrap();
        assert!(state.step(cfg, f64::NAN, 0.0).is_err());
        assert!(state.step(cfg, 0.0, f64::INFINITY).is_err());
    }

    #[test]
    fn forged_coordinates_fail_closed() {
        let cfg = config(RadialLaw::LogV1);
        let state = TppeState::origin(cfg)
            .unwrap()
            .step(cfg, 0.02, -0.01)
            .unwrap();
        let mut forged = state;
        forged.poincare_radius = 0.9;
        assert!(forged.validate().is_err());
        forged = state;
        forged.cumulative_revolutions += 1.0;
        assert!(forged.validate().is_err());
        forged = state;
        forged.hyperboloid_time += 1.0;
        assert!(forged.validate().is_err());
    }
}
