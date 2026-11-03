use ares_backend::harness::tppe_memory::{as_f64, cosine_f64, encode_dual, encode_text_at};

/// Falsifier suite for the memory kernel (X, counterexample-first).
/// v1 (single mixed cosine) FAILED with exact_later=0.112 < unrelated=0.930
/// and is preserved below as a regression pin. v2 (dual vectors) must pass:
/// recall ranks in CONTENT space only; position is address evidence.
#[test]
fn v2_content_recall_ignores_position_and_position_space_separates() {
    let same = "HyperDX stores telemetry observations for the research harness.";
    let other = "Qdrant stores research papers and preprints for library retrieval.";
    let cfg = ares_backend::harness::tppe_memory::candidate_config();
    let states = ares_backend::harness::tppe_memory::states_for_positions(&cfg, 100).unwrap();

    // recall path: content vectors (position-free by construction)
    let (q_content, _qp, _) = encode_dual(same, 0, &states).unwrap();
    let (exact_content, _ep, _) = encode_dual(same, 9, &states).unwrap();
    let (other_content, _op, _) = encode_dual(other, 0, &states).unwrap();
    let q = as_f64(&q_content);
    let exact_score = cosine_f64(&q, &as_f64(&exact_content));
    let other_score = cosine_f64(&q, &as_f64(&other_content));
    eprintln!("v2 content recall: exact={exact_score:.6} unrelated={other_score:.6}");
    assert!(
        exact_score > other_score,
        "content recall must rank exact above unrelated"
    );
    assert!(
        (exact_score - 1.0).abs() < 1e-4,
        "same text, any position: content vector identical"
    );

    // address path: position vectors separate same text across positions
    let (_, pos0, _) = encode_dual(same, 0, &states).unwrap();
    let (_, pos9, _) = encode_dual(same, 9, &states).unwrap();
    assert_ne!(
        pos0, pos9,
        "same text at pos 0 vs 9 must differ in position space"
    );
}

/// v1 regression pin: the mixed single-vector cosine space is KNOWN-bad
/// for recall. Kept so the failure stays documented, not rediscovered.
#[test]
#[ignore = "regression pin: v1 mixed cosine fails recall by design (see STORE-SUBSTRATE-DESIGN.md)"]
fn v1_mixed_cosine_falsifier_regression_pin() {
    let same = "HyperDX stores telemetry observations for the research harness.";
    let other = "Qdrant stores research papers and preprints for library retrieval.";
    let (query, _) = encode_text_at(same, 0).unwrap();
    let (exact_later, _) = encode_text_at(same, 9).unwrap();
    let (unrelated, _) = encode_text_at(other, 0).unwrap();
    let exact_score = cosine_f64(&as_f64(&query), &as_f64(&exact_later));
    let unrelated_score = cosine_f64(&as_f64(&query), &as_f64(&unrelated));
    assert!(
        exact_score < unrelated_score,
        "v1 pin: position dominates recall"
    );
}
