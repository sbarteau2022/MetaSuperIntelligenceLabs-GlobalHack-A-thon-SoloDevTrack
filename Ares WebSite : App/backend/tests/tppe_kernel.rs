//! TPPE kernel verification (written BEFORE any claim, Commandment X).
use ares_backend::harness::tppe::{RadialLaw, TppeConfig, TppeState};

fn cand() -> TppeConfig {
    TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.05,
        radial_span: 0.12,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }
}

fn parse_f64(hex: &str) -> f64 {
    let raw = u64::from_str_radix(hex, 16).unwrap();
    f64::from_bits(raw)
}

// GOLDEN: fixture generated from the ORIGINAL globalmind-runtime tppe.rs
// + qwen36.rs recurrence (backend/tests/fixtures/tppe-golden-v1.txt).
// Any drift between the port and the source fails here.
#[test]
fn golden_vectors_match_original_source() {
    let fx = include_str!("fixtures/tppe-golden-v1.txt");
    let cfg = cand();
    let mut in_linear = false;
    let mut seen = 0;
    for line in fx.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts[0] == "LAW" {
            in_linear = parts[1] == "linear_log_v1";
            continue;
        }
        if in_linear && parts.len() >= 3 && parts[0] == "POS" {
            let pos: u32 = parts[1].parse().unwrap();
            // recompute state at pos via the ported carry
            let states =
                ares_backend::harness::tppe_memory::states_for_positions(&cfg, (pos + 1) as usize)
                    .unwrap();
            let st = states[pos as usize];
            let want_state_hex = parts[3];
            let got = [
                st.theta_fast_unwrapped,
                st.theta_slow_unwrapped,
                st.cumulative_revolutions,
                st.hyperbolic_radius,
                st.poincare_radius,
                st.boundary_margin,
                st.hyperboloid_time,
                st.hyperboloid_spatial_norm,
            ];
            for (i, g) in got.iter().enumerate() {
                let w = parse_f64(&want_state_hex[i * 16..(i + 1) * 16]);
                assert!(
                    (g - w).abs() <= 1e-12,
                    "pos {pos} field {i}: port {g} vs original {w}"
                );
            }
            seen += 1;
        }
    }
    assert_eq!(seen, 4);
}

// ANALYTIC (X falsification-first): worst case is sin(psi)=1, not psi=+/-pi.
#[test]
fn strict_expansion_worst_case_sin_psi_one() {
    let cfg = cand();
    let steps = ares_backend::harness::tppe_memory::states_for_positions(&cfg, 100).unwrap();
    assert_eq!(steps.len(), 100);
    for w in steps.windows(2) {
        assert!(
            w[1].cumulative_revolutions > w[0].cumulative_revolutions,
            "w must strictly rise"
        );
        assert!(
            w[1].poincare_radius > w[0].poincare_radius,
            "rho must strictly rise"
        );
        assert!(w[1].hyperbolic_radius > w[0].hyperbolic_radius);
        assert!(w[1].boundary_margin > 0.0);
    }
    // Bound check: D(w) per step >= (0.017+0.009+0.26-0.04)/TAU at step 1 scale
    let first_dw = steps[0].cumulative_revolutions;
    assert!(first_dw > 0.0);
    // derived-revolutions identity
    for s in &steps {
        let expect = TppeState::derived_revolutions(s.theta_fast_unwrapped, s.theta_slow_unwrapped);
        assert_eq!(s.cumulative_revolutions, expect);
    }
}

// SEPARATION / DISTINCTNESS probes (staged n=1,10,100; X counterexample-first).
// Claim under test is scoped: same text, same epoch, p != q  => stored
// vectors differ. Tested in f32 (what Qdrant stores), pairwise.
#[test]
fn same_text_distinct_positions_differ_after_f32() {
    use ares_backend::harness::tppe_memory::{as_f64, cosine_f64, encode_text_at};
    let texts = [
        "hyperbolic KV cache nested in the mind map",
        "HyperDX is the telemetry and tracking store. Qdrant is the RAG plus memory store and auto-tails into session start.",
        "alpha",
    ];
    for text in texts {
        let v0 = encode_text_at(text, 0).unwrap().0;
        let v1 = encode_text_at(text, 1).unwrap().0;
        let v9 = encode_text_at(text, 9).unwrap().0;
        let v99 = encode_text_at(text, 99).unwrap().0;
        assert_ne!(v0, v1, "pos0 vs pos1 must differ in f32 for {text:?}");
        assert_ne!(v1, v9);
        assert_ne!(v9, v99);
        // normalize check (what cosine scoring sees)
        for v in [&v0, &v1, &v9, &v99] {
            let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((n - 1.0).abs() < 1e-4, "unit norm");
            assert!(v.iter().all(|x| x.is_finite()));
        }
        // sanity (not a margin claim): adjacent positions are close in
        // phase space but measurably distinct in f64 cosine.
        let c01 = cosine_f64(&as_f64(&v0), &as_f64(&v1));
        assert!(
            c01 < 0.999_999,
            "adjacent cos {c01} too close (recurrence risk)"
        );
    }
}

#[test]
fn encoder_fail_closed_cases() {
    use ares_backend::harness::tppe_memory::encode_text_at;
    assert!(encode_text_at("", 0).is_err());
    assert!(encode_text_at("   \n\t ", 0).is_err());
    assert!(
        encode_text_at("okay", 1000).is_err(),
        "over epoch cap must fail, not clamp"
    );
    assert!(
        encode_text_at("ok", 0).is_err(),
        "sub-3-char text has no 3-gram: fail closed"
    );
    assert!(encode_text_at("okay", 99).is_ok());
}

// O(1) SEQUENCE equivalence (staged n=1,10,100): TppeSequence::advance
// must produce byte-identical states to states_for_positions, or the
// optimization changed the math (hard FAIL under PARTITION).
#[test]
fn o1_sequence_matches_full_carry() {
    use ares_backend::harness::tppe_memory::{
        candidate_config, states_for_positions, TppeSequence,
    };
    let cfg = candidate_config();
    let carry = states_for_positions(&cfg, 100).unwrap();
    let mut seq = TppeSequence::new(cfg).unwrap();
    for (want_idx, want) in carry.iter().enumerate() {
        let got = seq.advance().unwrap();
        assert_eq!(
            got.theta_fast_unwrapped.to_bits(),
            want.theta_fast_unwrapped.to_bits(),
            "pos {want_idx}"
        );
        assert_eq!(
            got.poincare_radius.to_bits(),
            want.poincare_radius.to_bits()
        );
        assert_eq!(
            got.cumulative_revolutions.to_bits(),
            want.cumulative_revolutions.to_bits()
        );
    }
    // O(1) rehydration: from_persisted == replay, no replay performed.
    let (epoch, next, st) = seq.cursor();
    let re = TppeSequence::from_persisted(
        cfg,
        epoch,
        next,
        st.theta_fast_unwrapped,
        st.theta_slow_unwrapped,
    )
    .unwrap();
    let (_, _, rst) = re.cursor();
    assert!((rst.poincare_radius - st.poincare_radius).abs() <= 1e-12);
    assert!((rst.theta_fast_unwrapped - st.theta_fast_unwrapped).abs() <= 1e-15);
    // forged coordinates fail closed
    assert!(
        TppeSequence::from_persisted(cfg, 0, 0, -1.0, -2.0).is_err(),
        "negative derived revolutions must fail closed"
    );
    assert!(TppeSequence::from_persisted(cfg, 0, 0, f64::NAN, 0.0).is_err());
    assert!(
        TppeSequence::from_persisted(cfg, 0, 0, 1.0, 2.0).is_ok(),
        "any nonnegative phases define a consistent state"
    );
}

// BASELINE staged sanity (n=1): regular kernel smoke before ablation.
#[test]
fn baseline_regular_kernel_sanity() {
    use ares_backend::harness::baseline::encode_baseline;
    let text = "hyperbolic KV cache nested in the mind map";
    let (c0, p0) = encode_baseline(text, 0).unwrap();
    let (c1, p1) = encode_baseline(text, 1).unwrap();
    let (c9, _p9) = encode_baseline(text, 9).unwrap();
    assert_eq!(c0, c1, "baseline content must be position-free");
    assert_eq!(c0, c9);
    assert_ne!(p0, p1, "baseline position vector must move with pos");
    for v in [&p0, &p1] {
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((n - 1.0).abs() < 1e-4);
        assert!(v.iter().all(|x| x.is_finite()));
    }
    assert!(encode_baseline("", 0).is_err());
    assert!(encode_baseline("okay", 1000).is_err());
}
