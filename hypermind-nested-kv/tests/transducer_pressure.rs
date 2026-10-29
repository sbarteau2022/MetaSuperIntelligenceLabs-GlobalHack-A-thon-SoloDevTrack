//! Transducer pressure test: five reading levels, both directions.
//! Measures loss at every conversion stage (TokenlessInference §4 meters).

use hypermind_nested_kv::boundary_lexical_transducer::BoundaryLexicalTransducer;

fn corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "elementary",
            "The cat sat on the mat. The dog ran. See the red ball.",
        ),
        (
            "highschool",
            "Photosynthesis converts light energy into chemical energy. Mitochondria produce ATP through cellular respiration.",
        ),
        (
            "collegiate",
            "The treaty established a framework for multilateral cooperation, though enforcement mechanisms remained conspicuously underdetermined across jurisdictions.",
        ),
        (
            "phd",
            "Hyperbolic isometry invariance renders recentering moot: Möbius maps preserve the Lorentz inner product, so the variational free energy is coordinate-free up to the chosen gauge.",
        ),
        (
            "postdoc-multilingual",
            "La résonance capturée — 捕获共振 — يحدث عندما يستقر الجاذب: κ(T,t) → 0 across 细胞, 神经, سلوك scales. Π = 3.14159… ✓",
        ),
    ]
}

#[test]
fn roundtrip_all_levels() {
    let t = BoundaryLexicalTransducer::new();
    for (level, text) in corpus() {
        let ops = t.transduce_ingress_text(text, true);
        assert!(!ops.is_empty(), "[{}] ingress produced nothing", level);
        let expanded = t
            .expand_checked(
                &ops.iter()
                    .filter(|&&o| o != 0x10000)
                    .copied()
                    .collect::<Vec<_>>(),
            )
            .expect(&format!("[{}] expand failed", level));
        match t.transduce_egress_ops(&expanded) {
            Ok((_, back)) => {
                println!("[{}] roundtrip exact: {}", level, back == text);
                println!(
                    "[{}] in_bytes={} ops={} out_bytes={}",
                    level,
                    text.len(),
                    ops.len(),
                    back.len()
                );
                if back != text {
                    let a = text.as_bytes();
                    let b = back.as_bytes();
                    let diffs = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count()
                        + a.len().abs_diff(b.len());
                    println!(
                        "[{}] DRIFT bytes: {}/{}",
                        level,
                        diffs,
                        a.len().max(b.len())
                    );
                }
            }
            Err(r) => {
                println!(
                    "[{}] EGRESS FAIL: unhandled={} utf8={:?}",
                    level, r.egress_unhandled_ids, r.invalid_utf8
                );
                panic!("[{}] egress failed on valid ingress output", level);
            }
        }
    }
}

#[test]
fn fold_regurgitation() {
    let mut t = BoundaryLexicalTransducer::new();
    t.register_span(1000, vec![65, 66, 67]);
    let ops = t.transduce_ingress_text("xABCy", false);
    println!("folded ops: {:?}", ops);
    assert!(ops.contains(&1000), "fold should fire on ABC");
    let expanded = t.expand_checked(&ops).unwrap();
    assert_eq!(
        expanded,
        vec![120, 65, 66, 67, 121],
        "expand must restore carriers"
    );
    let re_expanded = t.expand_checked(&expanded).unwrap();
    assert_eq!(re_expanded, expanded, "carrier expand is stable");
    println!(
        "F-4 confirmed: fold is lossy on re-ingest by design (macro-op identity not preserved in carriers)"
    );
}

#[test]
fn f1_silent_drop_probe() {
    // F-1: unknown carrier IDs must ERROR with the ID named, never vanish.
    // expand_checked already does this; this test pins the contract.
    let t = BoundaryLexicalTransducer::new();
    // 99999 is above base_vocab (256) with no registered span -> must Err.
    let r = t.expand_checked(&[65, 99999, 66]);
    assert!(
        r.is_err(),
        "F-1 REGRESSION: unknown carrier silently dropped"
    );
    let e = r.unwrap_err();
    assert!(e.contains("99999"), "error must name the ID, got: {}", e);
    // Egress side: unhandled IDs (>=256, no vocab entry) must Err with count.
    let r2 = t.transduce_egress_ops(&[65, 99999, 66]);
    assert!(r2.is_err(), "F-1 REGRESSION: egress silently dropped");
    let receipt = r2.unwrap_err();
    assert_eq!(receipt.egress_unhandled_ids, 1, "drop count must be exact");
    assert!(!receipt.invalid_utf8, "no UTF-8 involved here");
    println!("F-1 pinned: drops are errors with receipts, never silent");
}

#[test]
fn f3_invalid_utf8_probe() {
    // F-3: lone surrogates / truncated sequences must NOT become U+FFFD
    // silently. Strict from_utf8 -> Err with invalid_utf8=true.
    let t = BoundaryLexicalTransducer::new();
    // 0xFF is never valid UTF-8 alone; 0xC3 0x28 is a truncated 2-byte seq.
    for bad in [vec![0xFFu32], vec![0xC3, 0x28], vec![0xE2, 0x82]] {
        let r = t.transduce_egress_ops(&bad);
        assert!(r.is_err(), "F-3 REGRESSION: invalid UTF-8 passed as text");
        assert!(r.unwrap_err().invalid_utf8, "receipt must flag UTF-8");
    }
    // Valid multibyte MUST pass (no false positives on CJK/Arabic/math).
    let good = "κ(T,t) → 0 across 细胞 Π ✓"
        .as_bytes()
        .iter()
        .map(|&b| b as u32)
        .collect::<Vec<_>>();
    let (_, back) = t
        .transduce_egress_ops(&good)
        .expect("valid UTF-8 must pass");
    assert_eq!(back, "κ(T,t) → 0 across 细胞 Π ✓");
    println!("F-3 pinned: invalid rejected, valid multilingual passes");
}

#[test]
fn fold_collision_probe() {
    // Two spans sharing a prefix: longest-match must win deterministically.
    let mut t = BoundaryLexicalTransducer::new();
    t.register_span(1000, vec![65, 66]); // AB
    t.register_span(1001, vec![65, 66, 67]); // ABC (longer wins)
    let ops = t.transduce_ingress_text("xABCy", false);
    assert!(ops.contains(&1001), "longest match must win, got {:?}", ops);
    assert!(
        !ops.contains(&1000),
        "shorter shadow must not fire, got {:?}",
        ops
    );
    // Tie-break: same length, lowest atom id wins.
    let mut t2 = BoundaryLexicalTransducer::new();
    t2.register_span(2001, vec![68, 69]);
    t2.register_span(2000, vec![68, 69]);
    let ops2 = t2.transduce_ingress_text("xDEy", false);
    assert!(
        ops2.contains(&2000),
        "tie goes to lowest id, got {:?}",
        ops2
    );
    println!("fold precedence pinned: longest-wins, ties-lowest-id");
}

#[test]
fn adversarial_multilingual_stress() {
    // Mixed scripts, RTL, ZWJ sequences, lone combiners, max-length runs.
    // The transducer is byte-total: everything ingresses, everything egresses.
    let long_run = "abcdefghij".repeat(500);
    let cases: Vec<(&str, &str)> = vec![
        ("arabic_rtl", "يحدث عندما يستقر الجاذب في الخلايا العصبية"),
        (
            "cjk_mixed",
            "捕获共振发生在细胞水平而神經退化發生在大腦網絡層面",
        ),
        ("emoji_zwj", "👨‍👩‍👧‍👦 family \u{200d} joiner \u{fe0f} variation"),
        ("combining", "e\u{301} résumé naïve über"),
        (
            "math_symbols",
            "∀x ∈ H³ : cosh(d) ≥ 1 ∧ ψ* = θf − 2θs ∴ Ω ≥ 0",
        ),
        ("empty", ""),
        ("single_byte", "a"),
        ("long_run", &long_run),
    ];
    let t = BoundaryLexicalTransducer::new();
    for (name, text) in cases {
        let ops = t.transduce_ingress_text(text, false);
        // Empty text -> empty ops (no BOS when prepend=false). Non-empty -> non-empty.
        assert_eq!(
            ops.is_empty(),
            text.is_empty(),
            "[{}] ingress emptiness mismatch",
            name
        );
        if text.is_empty() {
            continue;
        }
        let expanded = t
            .expand_checked(&ops)
            .unwrap_or_else(|e| panic!("[{}] expand failed: {}", name, e));
        let (_, back) = t
            .transduce_egress_ops(&expanded)
            .unwrap_or_else(|r| panic!("[{}] egress failed: {:?}", name, r));
        assert_eq!(
            back, text,
            "[{}] byte corruption (transducer-level, gate-independent)",
            name
        );
        println!(
            "[{}] OK: {} bytes -> {} ops -> {} bytes",
            name,
            text.len(),
            ops.len(),
            back.len()
        );
    }
}

#[test]
fn regurgitation_loop_closes() {
    // Stage B miniature: atoms -> ops -> bytes -> ops -> atoms must land
    // within cosh_drift tolerance. Here at transducer level: ingress ->
    // egress -> ingress must reproduce the SAME op stream (loop closure).
    let t = BoundaryLexicalTransducer::new();
    for (level, text) in corpus() {
        let ops1 = t.transduce_ingress_text(text, false);
        let expanded = t.expand_checked(&ops1).unwrap();
        let (_, back) = t.transduce_egress_ops(&expanded).unwrap();
        let ops2 = t.transduce_ingress_text(&back, false);
        assert_eq!(
            ops1, ops2,
            "[{}] loop did not close: {:?} vs {:?}",
            level, ops1, ops2
        );
    }
    println!("regurgitation loop closes on all 5 levels (carrier band, no folds)");
}

#[test]
fn multilingual_bytes_survive() {
    let t = BoundaryLexicalTransducer::new();
    for (level, text) in corpus() {
        let ops = t.transduce_ingress_text(text, false);
        let expanded = t.expand_checked(&ops).unwrap();
        let (_, back) = t.transduce_egress_ops(&expanded).unwrap();
        assert_eq!(
            back, text,
            "[{}] byte loss in transducer (gate-independent)",
            level
        );
        println!(
            "[{}] bytes preserved: {} -> {} ops -> {} bytes",
            level,
            text.len(),
            ops.len(),
            back.len()
        );
    }
}
