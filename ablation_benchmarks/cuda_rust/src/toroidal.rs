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
        assert!(results[0]); // Close phase → coherent
        assert!(!results[1]); // Far phase → not coherent
    }
}
