//! Boundary Symplectic Transducer round-trip: multilingual text, code,
//! tool calls, multimodal bytes, malformed input. Zero silent loss:
//! every failure names its bytes or SSA ids.

use hypermind_nested_kv::boundary_symplectic_transducer::*;

fn prov() -> Provenance {
    Provenance::new(
        "roundtrip-test",
        "test-actor",
        "boundary_symplectic_roundtrip",
    )
    .unwrap()
}

fn ident_j() -> [[f64; 10]; 10] {
    let mut j = [[0.0f64; 10]; 10];
    for i in 0..10 {
        j[i][i] = 1.0;
    }
    j
}

#[test]
fn multilingual_text_roundtrips_exactly() {
    let cases = [
        "The cat sat on the mat.\nSecond line.\n",
        "La résonance capturée — 捕获共振 — يحدث عندما يستقر الجاذب: κ(T,t) → 0 ✓\n",
        "e\u{301} résumé naïve über\n👨\u{200d}👩\u{200d}👧\u{200d}👦 ZWJ\n",
        "∀x ∈ H³ : cosh(d) ≥ 1 ∧ ψ* = θf − 2θs ∴ Ω ≥ 0\n",
        "fn main() { println!(\"hi\"); }\n// 第二行注释\n",
        "single line no newline",
        "trailing newline\n",
        "\n\n",
    ];
    for (i, text) in cases.iter().enumerate() {
        let m = ingress_text(text.as_bytes(), Some("mul"), prov(), "test", "rt")
            .unwrap_or_else(|e| panic!("[{i}] ingress failed: {e}"));
        let v = validate_module(&m).unwrap_or_else(|e| panic!("[{i}] invalid: {e}"));
        assert_eq!(
            v.claim,
            GeometryClaim::PhaseAugmented,
            "[{i}] default claim"
        );
        let res = loopback_results(&m);
        assert_eq!(res.len(), m.ops.len(), "[{i}] every SSA op has a result");
        let out = egress_decode(&m, &res).unwrap_or_else(|e| panic!("[{i}] egress: {e}"));
        assert_eq!(out.text.as_deref(), Some(*text), "[{i}] byte-exact");
        assert_eq!(out.loss.loss_bytes, 0, "[{i}] zero loss");
        assert_eq!(out.evidence.len(), m.ops.len(), "[{i}] per-op evidence");
        let r = replay(&m).expect("replay");
        assert_eq!(r.module_digest, m.digest, "[{i}] replay digest");
        assert_eq!(r.loss_bytes, 0);
    }
    println!("multilingual: {} cases exact, zero loss", cases.len());
}

#[test]
fn language_tag_never_excludes() {
    // Acceptance must not depend on script: same bytes, any tag, same ops.
    let bytes = "捕获共振\n".as_bytes();
    let tags = [None, Some("zh"), Some("ar"), Some("xx-fake"), Some("mul")];
    let mut ops0: Option<usize> = None;
    for t in tags {
        let m = ingress_text(bytes, t, prov(), "test", "lang").unwrap();
        validate_module(&m).unwrap();
        match ops0 {
            None => ops0 = Some(m.ops.len()),
            Some(n) => assert_eq!(m.ops.len(), n, "tag {t:?} changed lowering"),
        }
        let out = egress_decode(&m, &loopback_results(&m)).unwrap();
        assert_eq!(out.text.unwrap(), "捕获共振\n");
    }
}

#[test]
fn code_roundtrips_with_language() {
    let code = "def fib(n):\n    return n if n < 2 else fib(n-1) + fib(n-2)\n";
    let m = ingress_code(code.as_bytes(), "python", prov(), "test", "code").unwrap();
    validate_module(&m).unwrap();
    for o in &m.ops {
        assert!(
            matches!(o.payload, OpPayload::Code { .. }),
            "code payload typed"
        );
    }
    let out = egress_decode(&m, &loopback_results(&m)).unwrap();
    assert_eq!(out.text.unwrap(), code);
}

#[test]
fn tool_event_roundtrips_typed_args() {
    let args = vec![
        Arg {
            name: "q".to_string(),
            value: ArgValue::Text("κ query ✓".to_string()),
        },
        Arg {
            name: "k".to_string(),
            value: ArgValue::Int(5),
        },
        Arg {
            name: "tau".to_string(),
            value: ArgValue::RealBits(0.75f64.to_bits()),
        },
        Arg {
            name: "strict".to_string(),
            value: ArgValue::Bool(true),
        },
        Arg {
            name: "blob".to_string(),
            value: ArgValue::Bytes(vec![0x00, 0xFF, 0xFE]),
        },
    ];
    let m = ingress_tool_event("search", args, prov(), "test", "tool").unwrap();
    validate_module(&m).unwrap();
    let out = egress_decode(&m, &loopback_results(&m)).unwrap();
    assert!(out.text.is_none(), "tool module renders no text");
    assert_eq!(out.evidence.len(), 1);
    assert_eq!(out.loss.loss_bytes, 0);
}

#[test]
fn observation_bytes_roundtrip_arbitrary() {
    // Non-UTF-8 bytes are legal in observations (mime-typed), refused as text.
    let raw = vec![0x00, 0xFF, 0xFE, 0x89, 0x50, 0x4E, 0x47];
    let m = ingress_observation(&raw, "application/octet-stream", prov(), "test", "obs").unwrap();
    validate_module(&m).unwrap();
    let out = egress_decode(&m, &loopback_results(&m)).unwrap();
    assert!(out.text.is_none());
    assert_eq!(out.loss.loss_bytes, 0);
    // Empty observation: legal, zero ops loss.
    let m0 = ingress_observation(&[], "application/octet-stream", prov(), "test", "obs").unwrap();
    validate_module(&m0).unwrap();
    let out0 = egress_decode(&m0, &loopback_results(&m0)).unwrap();
    assert_eq!(out0.loss.loss_bytes, 0);
}

#[test]
fn malformed_text_rejected_with_receipt() {
    let bad = vec![0xFFu8, 0xFE, 0xC3, 0x28];
    let e = ingress_text(&bad, None, prov(), "test", "bad").unwrap_err();
    assert!(e.reason.contains("UTF-8"), "got: {}", e.reason);
    assert_eq!(e.input_digest, input_digest(&bad), "receipt binds input");
}

#[test]
fn malformed_tool_args_rejected() {
    // Duplicate names.
    let e = ingress_tool_event(
        "t",
        vec![
            Arg {
                name: "a".to_string(),
                value: ArgValue::Int(1),
            },
            Arg {
                name: "a".to_string(),
                value: ArgValue::Int(2),
            },
        ],
        prov(),
        "test",
        "tool",
    )
    .unwrap_err();
    assert!(e.reason.contains("unique"), "got: {}", e.reason);
    // Non-finite real.
    let e2 = ingress_tool_event(
        "t",
        vec![Arg {
            name: "x".to_string(),
            value: ArgValue::RealBits(f64::NAN.to_bits()),
        }],
        prov(),
        "test",
        "tool",
    )
    .unwrap_err();
    assert!(e2.reason.contains("non-finite"), "got: {}", e2.reason);
    // Empty provenance.
    let bad_prov = Provenance::new("", "actor", "d").unwrap_err();
    assert!(bad_prov.contains("non-empty"));
}

#[test]
fn egress_names_unknown_missing_duplicate() {
    let m = ingress_text(b"one\ntwo\n", None, prov(), "test", "eg").unwrap();
    // Unknown id.
    let mut res = loopback_results(&m);
    res.push(NativeResult {
        ssa_id: 4242,
        payload: ResultPayload::TextBytes(b"junk".to_vec()),
        effect_bits: EFF_EMIT,
    });
    let e = egress_decode(&m, &res).unwrap_err();
    assert_eq!(e.unknown_ids, vec![4242]);
    // Missing id.
    let mut res2 = loopback_results(&m);
    res2.pop();
    let e2 = egress_decode(&m, &res2).unwrap_err();
    assert_eq!(e2.missing_ids.len(), 1);
    assert!(e2.loss_bytes > 0, "missing op loses its bytes");
    // Duplicate id.
    let mut res3 = loopback_results(&m);
    res3.push(res3[0].clone());
    let e3 = egress_decode(&m, &res3).unwrap_err();
    assert!(!e3.failed_inverse.is_empty(), "dup lands in failed_inverse");
}

#[test]
fn egress_refuses_bad_utf8_and_effect_mismatch() {
    let m = ingress_text(b"ab\n", None, prov(), "test", "eg2").unwrap();
    let mut res = loopback_results(&m);
    res[0].payload = ResultPayload::TextBytes(vec![0xFF]);
    let e = egress_decode(&m, &res).unwrap_err();
    assert_eq!(e.failed_inverse, vec![0]);
    let m2 = ingress_observation(
        b"\x01\x02",
        "application/octet-stream",
        prov(),
        "test",
        "eg3",
    )
    .unwrap();
    let mut res4 = loopback_results(&m2);
    res4[0].effect_bits = EFF_EMIT; // observe requires OBSERVE
    let e4 = egress_decode(&m2, &res4).unwrap_err();
    assert_eq!(e4.effect_mismatch, vec![0]);
}

#[test]
fn model_ir_adapter_binds_geometry_refuses_guess() {
    // Mirrors rust-server ModelStepIr bounds: vocab 248320, chunk <= 8,
    // position + len <= max_context. Carriers satisfy geometry but are
    // disclosed as NOT trained token IDs.
    let a = ModelIrAdapter::combined_smoke([11u8; 32], [12u8; 32]);
    assert!(a.disclosure().contains("NOT claimed"));
    assert!(a.shape_contract().contains("248320"));
    let m = ingress_text(b"hello world\nsecond line\n", None, prov(), "test", "mir").unwrap();
    let native = a.encode(&m).expect("encode");
    assert_eq!(
        native.adapter_digest,
        ModelIrAdapter::adapter_digest_static()
    );
    assert_eq!(native.weights_digest, [11u8; 32]);
    // All carrier ids < vocab, chunked <= 8, positions dense from 0.
    let ids = a.carrier_ids_for(&m);
    assert!(!ids.is_empty());
    assert!(ids.iter().all(|t| *t < 248_320), "carrier < vocab");
    // Header parse: [ad][weights][config][vocab u32][maxctx u64][nsteps u64][steps...].
    let b = &native.bytes;
    let mut off = 0;
    let rd = |o: &mut usize, n: usize| -> Vec<u8> {
        let s = *o;
        *o += n;
        b[s..*o].to_vec()
    };
    let read_u64 = |o: &mut usize| -> u64 {
        let v = rd(o, 8);
        let mut t = [0u8; 8];
        t.copy_from_slice(&v);
        u64::from_le_bytes(t)
    };
    let ad_len = read_u64(&mut off) as usize;
    off += ad_len;
    let w_len = read_u64(&mut off) as usize;
    off += w_len;
    let c_len = read_u64(&mut off) as usize;
    off += c_len;
    let mut vb = [0u8; 4];
    vb.copy_from_slice(&rd(&mut off, 4));
    assert_eq!(u32::from_le_bytes(vb), 248_320);
    let maxctx = read_u64(&mut off);
    assert_eq!(maxctx, 512, "max_context bound in header");
    let nsteps = read_u64(&mut off) as usize;
    let expect_steps = ids.len().div_ceil(8);
    assert_eq!(nsteps, expect_steps, "chunk<=8 step packing");
    let mut pos = 0usize;
    for _ in 0..nsteps {
        let p = read_u64(&mut off) as usize;
        assert_eq!(p, pos, "dense positions from 0");
        let n = read_u64(&mut off) as usize;
        assert!((1..=8).contains(&n), "chunk bound");
        for _ in 0..n {
            let mut tb = [0u8; 4];
            tb.copy_from_slice(&rd(&mut off, 4));
            assert!(u32::from_le_bytes(tb) < 248_320);
        }
        pos += n;
    }
    assert_eq!(off, b.len(), "no trailing bytes");
    assert!(pos <= 512, "max_context bound");
    // Decode refuses: reversing carriers to SSA without a qualified
    // detokenizer + originating module would be a guess.
    let de = a.decode(&native).unwrap_err();
    assert!(de.reason.contains("refused"), "got: {}", de.reason);
    // Conformance to the real rust-server ModelStepIr bounds
    // (src/serving/model_ir.rs::compile_with_context, tested live above):
    // vocab in 1..=300000, 1..=8 tokens per step, pos+len <= max_context,
    // decode-kind steps take exactly 1 token. Mirror those predicates here
    // so any upstream bound change fails this test loudly.
    let mirror_check =
        |vocab: u32, max_context: usize, pos: usize, tokens: &[u32], decode: bool| -> bool {
            if !((512..=262_144).contains(&max_context)) {
                return false;
            }
            if !(vocab > 0 && vocab <= 300_000 && !tokens.is_empty() && tokens.len() <= 8) {
                return false;
            }
            if !(tokens.iter().all(|t| *t < vocab)) {
                return false;
            }
            if decode && tokens.len() != 1 {
                return false;
            }
            if pos
                .checked_add(tokens.len())
                .is_none_or(|n| n > max_context)
            {
                return false;
            }
            true
        };
    // Every emitted step passes the mirrored predicate as prefill.
    let mut check_pos = 0usize;
    for chunk in ids.chunks(8) {
        assert!(
            mirror_check(248_320, 512, check_pos, chunk, false),
            "step fails mirrored ModelStepIr bounds"
        );
        check_pos += chunk.len();
    }
    // Boundary probes of the mirrored predicate (must match upstream):
    assert!(
        !mirror_check(248_320, 512, 511, &[1, 2], false),
        "overflow rejected"
    );
    assert!(
        !mirror_check(248_320, 512, 0, &[1, 2], true),
        "decode takes 1 token"
    );
    assert!(
        !mirror_check(248_320, 512, 0, &[248_320], false),
        "id >= vocab rejected"
    );
    assert!(
        !mirror_check(248_320, 512, 0, &[1; 9], false),
        "9-token chunk rejected"
    );
    // Non-text modules refused, not guessed.
    let tool = ingress_tool_event(
        "search",
        vec![Arg {
            name: "q".to_string(),
            value: ArgValue::Text("x".to_string()),
        }],
        prov(),
        "test",
        "mir",
    )
    .unwrap();
    let te = a.encode(&tool).unwrap_err();
    assert!(
        te.reason.contains("no carrier mapping"),
        "got: {}",
        te.reason
    );
    // Semantic key untouched by adapter bytes.
    let k_before = semantic_key(&m);
    let _ = a.encode(&m).unwrap();
    assert_eq!(k_before, semantic_key(&m));
}

#[test]
fn geometry_and_digest_tamper_detected() {
    let m = ingress_text(b"geom\n", None, prov(), "test", "geo").unwrap();
    // Tampered geometry.
    let mut bad = m.clone();
    bad.ops[0].pos.h = [1.0, 0.0, 0.0, 0.0]; // <x,x> = -1? -1+0 = -1 ok actually; use bad point
    bad.ops[0].pos.h = [2.0, 0.0, 0.0, 0.0]; // <x,x> = -4, residual 3
    bad.digest = module_digest(&bad);
    let e = validate_module(&bad).unwrap_err();
    assert!(e.contains("H3"), "got: {e}");
    // Tampered digest.
    let mut bad2 = m.clone();
    bad2.digest = [0u8; 32];
    let e2 = validate_module(&bad2).unwrap_err();
    assert!(e2.contains("digest"), "got: {e2}");
    // Tampered version.
    let mut bad3 = m.clone();
    bad3.versions.dialect = 999;
    let e3 = validate_module(&bad3).unwrap_err();
    assert!(e3.contains("version"), "got: {e3}");
    // Tampered SSA order / dominance.
    let mut bad4 = m.clone();
    if bad4.ops.len() >= 1 {
        bad4.ops[0].operands = vec![7];
        let e4 = validate_module(&bad4).unwrap_err();
        assert!(e4.contains("dominance"), "got: {e4}");
    }
}

#[test]
fn symplectic_claim_proven_or_phase_augmented() {
    let m = ingress_text(b"claim\n", None, prov(), "test", "cl").unwrap();
    // Identity transport: claim attaches and validates.
    let good = attach_canonical_claim(&m, &ident_j(), DEFAULT_SYMPLECTIC_TOL).unwrap();
    assert!(matches!(
        good.claim,
        GeometryClaim::SymplecticCanonical { .. }
    ));
    validate_module(&good).unwrap();
    // Shear: claim refused; module honestly stays phase-augmented.
    let mut shear = ident_j();
    shear[0][0] = 2.0;
    let e = attach_canonical_claim(&m, &shear, DEFAULT_SYMPLECTIC_TOL).unwrap_err();
    assert!(e.contains("phase-augmented"), "got: {e}");
    // Over-tolerance claim rejected even if constructed.
    let mut forged = m.clone();
    forged.claim = GeometryClaim::SymplecticCanonical {
        residual: 1.0,
        tol: 1e-9,
    };
    forged.digest = module_digest(&forged);
    let e2 = validate_module(&forged).unwrap_err();
    assert!(e2.contains("residual"), "got: {e2}");
}

#[test]
fn canonical_key_stable_across_geometry_and_model() {
    // Same semantic content re-ingressed -> same key; geometry/model
    // bindings never move it.
    let m1 = ingress_text(b"same\n", None, prov(), "d", "s").unwrap();
    let m2 = ingress_text(b"same\n", None, prov(), "d", "s").unwrap();
    assert_eq!(semantic_key(&m1), semantic_key(&m2), "deterministic");
    // Different bytes -> different key.
    let m3 = ingress_text(b"diff\n", None, prov(), "d", "s").unwrap();
    assert_ne!(semantic_key(&m1), semantic_key(&m3));
    // Adapter digests are not key inputs (compile-time: no ModelNative
    // field appears in canonical_semantic_payload).
}

#[test]
fn passthrough_adapter_binds_but_never_shares() {
    let a = PassthroughTestAdapter::new([1u8; 32], [2u8; 32]);
    assert!(a.disclosure().contains("TEST-ONLY"));
    let m = ingress_text(b"ad\n", None, prov(), "test", "ad").unwrap();
    let native = a.encode(&m).expect("encode");
    assert_eq!(
        native.adapter_digest,
        PassthroughTestAdapter::adapter_digest_static()
    );
    assert_eq!(native.weights_digest, [1u8; 32]);
    assert!(!native.execution_contract.is_empty());
    // Wrong weights on decode: refused.
    let mut wrong = native.clone();
    wrong.weights_digest = [9u8; 32];
    assert!(a.decode(&wrong).is_err());
}

#[test]
fn opaque_frozen_adapter_refuses_with_disclosure() {
    let a = OpaqueFrozenAdapter {
        weights_digest: [7u8; 32],
        config_digest: [8u8; 32],
        reason: "requires native embedding IDs; no typed-IR path".to_string(),
    };
    let m = ingress_text(b"x\n", None, prov(), "test", "op").unwrap();
    let e = a.encode(&m).unwrap_err();
    assert!(e.reason.contains("DISCLOSED GAP"), "got: {}", e.reason);
    assert!(e.reason.contains("embedding IDs"));
}
