//! Mirror-parity tests: the toy must agree with the backend byte-for-byte
//! on every shared formula, or the rehearsal is teaching wrong lessons.

use hypermind_toy::mission::{
    chain_hash, dry_run, geometry_mission, research_gate, InformationNeed, Judge, JudgeKind,
    JudgeRef, Packet, PacketStatus, Plan, Receipt, ReceiptLog, ResearchRecord, RetryPolicy,
    SessionEventKind, SessionLog, ToolCall, WorkingSetCache,
};
use hypermind_toy::runtime::{
    run_mission_local, run_mission_with_prior_local, sha256_hex, verify_external_chain,
    CapabilityGate, CapabilityNode, Engine, ScriptExecutor,
};

fn sample_plan() -> Plan {
    Plan {
        id: "mission-geometry-smoke".into(),
        objective: "geometry training readiness smoke".into(),
        packets: vec![Packet {
            id: "packet-geometry-01".into(),
            task: "validate geometry packet".into(),
            depends_on: vec![],
            declared_context_tokens: 100,
            tool_calls: vec![ToolCall {
                tool: "judge".into(),
                args_json: "{}".into(),
                declared: true,
            }],
            judges: vec![JudgeRef {
                id: "judge-pass".into(),
                kind: "contains".into(),
            }],
            branch: None,
            retry_policy: RetryPolicy {
                max_rounds: 2,
                material_change_required: true,
            },
            status: PacketStatus::Pending,
            attempts: 0,
            judge_joined: false,
        }],
    }
}

// --- chain_hash canonical form: prev|packet|attempt|outcome|judges|n3|energy|erased|ts ---

#[test]
fn chain_hash_matches_backend_canonical_form() {
    // Independent recomputation of the backend's exact format string,
    // including {:?} energy formatting.
    let got = chain_hash("GENESIS", "p", 1, "PASS", &["j".into()], &[], 0.5, 4, 7);
    let canon = format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{}|{}",
        "GENESIS", "p", 1, "PASS", "j", "", 0.5, 4, 7
    );
    assert_eq!(got, sha256_hex(&canon));
}

#[test]
fn receipt_log_mints_and_verifies_like_backend() {
    let mut log = ReceiptLog::default();
    log.append(Receipt {
        packet_id: "p".into(),
        attempt: 1,
        outcome: "PASS".into(),
        judge_ids: vec!["j".into()],
        evidence_hash: "forged".into(), // overwritten by the mint
        energy_joules: 0.5,
        erased_bits: 4,
        timestamp_ms: 0,            // filled by the mint
        prev_hash: "forged".into(), // overwritten by the mint
        n3_receipts: vec![],
    });
    assert_eq!(log.verify_chain(), Ok(1));
    assert_ne!(log.entries[0].evidence_hash, "forged");
    assert_ne!(log.entries[0].prev_hash, "forged");
    assert!(log.entries[0].timestamp_ms > 0);
    log.entries[0].outcome = "FAIL".into();
    assert_eq!(log.verify_chain(), Err(0));
}

#[test]
fn verify_external_chain_is_independent_recomputation() {
    // An externally produced chain (simulating backend rows) must verify
    // here iff the canonical form matches — this is the port gate.
    let ts = 1_000_001u64;
    let ev = chain_hash(
        "GENESIS",
        "p1",
        1,
        "PASS",
        &["j".into()],
        &["a".repeat(64)],
        0.05,
        400,
        ts,
    );
    let rows = vec![(
        "p1".to_string(),
        "GENESIS".to_string(),
        1u32,
        "PASS".to_string(),
        vec!["j".to_string()],
        vec!["a".repeat(64)],
        0.05f64,
        400u64,
        ts,
        ev,
    )];
    assert_eq!(verify_external_chain(&rows), Ok(1));
    let mut tampered = rows.clone();
    tampered[0].3 = "FAIL".to_string();
    assert_eq!(verify_external_chain(&tampered), Err(0));
}

// --- mission honesty floor ---

#[test]
fn mission_fails_closed_without_live_executor() {
    let input = geometry_mission();
    let receipt = run_mission_local(input, 100.0);
    assert!(!receipt.passed_all());
    assert!(receipt.passed.is_empty());
    assert!(!receipt.receipts.entries.is_empty());
    assert!(receipt.receipts.entries.iter().all(|e| e.outcome != "PASS"));
    assert!(receipt.receipts.verify_chain().is_ok());
}

#[test]
fn forged_prior_skips_nothing() {
    let forged = ReceiptLog {
        entries: vec![Receipt {
            packet_id: "packet-geometry-01".into(),
            attempt: 1,
            outcome: "PASS".into(),
            judge_ids: vec!["judge-pass".into()],
            evidence_hash: "forged".into(),
            energy_joules: 0.0,
            erased_bits: 0,
            timestamp_ms: 0,
            prev_hash: "forged".into(),
            n3_receipts: vec![],
        }],
    };
    let input = hypermind_toy::mission::MissionInput {
        plan: sample_plan(),
        judges: vec![Judge {
            id: "judge-pass".into(),
            kind: JudgeKind::Contains {
                needle: "PASS".into(),
            },
        }],
    };
    let second = run_mission_with_prior_local(input, 100.0, Some(forged));
    assert!(second.passed.is_empty());
    assert_eq!(second.escalated, vec!["packet-geometry-01".to_string()]);
}

#[test]
fn judges_are_deterministic() {
    let j = Judge {
        id: "j".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    };
    assert!(matches!(
        j.run("all PASS here", 0).outcome,
        hypermind_toy::mission::JudgeOutcome::Pass
    ));
    assert!(matches!(
        j.run("nothing", 0).outcome,
        hypermind_toy::mission::JudgeOutcome::Fail(_)
    ));
    let g = Judge {
        id: "g".into(),
        kind: JudgeKind::GoldenHash {
            expected: "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".into(),
        },
    };
    assert!(matches!(
        g.run("hello", 0).outcome,
        hypermind_toy::mission::JudgeOutcome::Pass
    ));
}

#[test]
fn capability_gate_requires_judged_atom_and_receipt() {
    let mut gate = CapabilityGate::default();
    assert!(gate.admit("map.query").is_err());
    gate.register(CapabilityNode {
        name: "map.query".into(),
        family: "map".into(),
        judged_atom: true,
        receipt_present: false,
    });
    assert!(gate.admit("map.query").is_err());
    gate.register(CapabilityNode {
        name: "map.query".into(),
        family: "map".into(),
        judged_atom: true,
        receipt_present: true,
    });
    assert!(gate.admit("map.query").is_ok());
    assert!(gate.admit("nuclear.launch").is_err());
}

#[test]
fn engine_judge_output_needs_registered_judge() {
    let engine = Engine::new();
    let plan = sample_plan();
    let (pass, _ids) = engine.judge_output(&plan.packets[0], "PASS", 0);
    assert!(!pass, "unregistered judge must fail closed");
}

#[test]
fn research_gate_and_session_chain() {
    let mut log = SessionLog::default();
    let need = InformationNeed {
        packet_id: "p1".into(),
        required_terms: vec!["hyperbolic".into()],
        min_depth: 2,
    };
    assert!(research_gate(&log, &need, &[], 0.5).is_err());
    log.append("s", "w", SessionEventKind::ContextInject, "{}".into());
    let rec = ResearchRecord {
        packet_id: "p1".into(),
        covered_terms: vec!["hyperbolic".into()],
        depth_reached: 2,
        rag_coverage: 0.9,
    };
    assert!(research_gate(&log, &need, &[rec], 0.5).is_ok());
    assert!(log.verify_chain().is_ok());
}

#[test]
fn working_set_cache_mass_and_eviction() {
    let mut c = WorkingSetCache::new("m", "t", "q", 2);
    let k1 = c.insert(None, "alpha".into(), 10, true);
    let _k2 = c.insert(None, "beta".into(), 10, false);
    assert_eq!(c.blocks.len(), 2);
    c.decay_epoch();
    assert!(c.blocks[&k1].mass > 0);
    for b in c.blocks.values_mut() {
        b.refcount = 0;
    }
    assert!(c.evict_one());
}

#[test]
fn plan_validation_rejects_judgeless_and_cyclic() {
    assert!(dry_run(&sample_plan()).is_ok());
    let mut judgeless = sample_plan();
    judgeless.packets[0].judges.clear();
    assert!(dry_run(&judgeless).is_err());
    let mut cyclic = sample_plan();
    cyclic.packets[0].depends_on = vec!["packet-geometry-01".into()];
    assert!(dry_run(&cyclic).is_err());
}

// --- the rehearsal the backend cannot do yet: scripted PASS paths ---

#[test]
fn live_mission_passes_with_scripted_executor() {
    let mut gate = CapabilityGate::default();
    gate.register(CapabilityNode {
        name: "map.query".into(),
        family: "map".into(),
        judged_atom: true,
        receipt_present: true,
    });
    let mut plan = Plan {
        id: "toy-live".into(),
        objective: "rehearse PASS path".into(),
        packets: vec![
            Packet {
                id: "p1".into(),
                task: "recall scheduling knowledge".into(),
                depends_on: vec![],
                declared_context_tokens: 100,
                tool_calls: vec![ToolCall {
                    tool: "map.query".into(),
                    args_json: r#"{"qtext":"prefill decode batching"}"#.into(),
                    declared: true,
                }],
                judges: vec![JudgeRef {
                    id: "j".into(),
                    kind: "contains".into(),
                }],
                branch: None,
                retry_policy: RetryPolicy {
                    max_rounds: 2,
                    material_change_required: true,
                },
                status: PacketStatus::Pending,
                attempts: 0,
                judge_joined: false,
            },
            Packet {
                id: "p2".into(),
                task: "use the recall".into(),
                depends_on: vec!["p1".into()],
                declared_context_tokens: 50,
                tool_calls: vec![],
                judges: vec![JudgeRef {
                    id: "j".into(),
                    kind: "contains".into(),
                }],
                branch: None,
                retry_policy: RetryPolicy {
                    max_rounds: 1,
                    material_change_required: true,
                },
                status: PacketStatus::Pending,
                attempts: 0,
                judge_joined: true,
            },
        ],
    };
    let mut engine = Engine::new();
    engine.register_judge(Judge {
        id: "j".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    });
    let mut receipts = ReceiptLog::default();
    let mut budget = hypermind_toy::mission::EnergyBudget::new(100.0);
    let mut ledger = Vec::new();
    // p2 has no map.* calls: judge_joined on empty output escalates it.
    // Give p1 a PASS answer; p2 proves the escalate-and-skip shape instead.
    let mut exec = ScriptExecutor::new().answer("map.query", "recall PASS depth_k=2");
    let report = engine.run_sequential_live(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut exec,
        &gate,
        &mut ledger,
    );
    assert_eq!(report.passed, vec!["p1".to_string()]);
    assert_eq!(report.escalated, vec!["p2".to_string()]);
    assert!(receipts.verify_chain().is_ok());
    // Energy was charged exactly once: 1e6 ns = 0.001 s + 100 tokens.
    let expect = 0.001 + 100.0 * 0.0005;
    assert!(
        (budget.spent_j - expect).abs() < 1e-9,
        "spent={}",
        budget.spent_j
    );
    // The executor digest bound into the receipt is exact 64-hex.
    assert_eq!(receipts.entries[0].n3_receipts.len(), 1);
    assert_eq!(receipts.entries[0].n3_receipts[0].len(), 64);
}

#[test]
fn transport_error_retries_once_then_escalates() {
    let mut gate = CapabilityGate::default();
    gate.register(CapabilityNode {
        name: "map.query".into(),
        family: "map".into(),
        judged_atom: true,
        receipt_present: true,
    });
    let mut plan = sample_plan();
    plan.packets[0].tool_calls = vec![ToolCall {
        tool: "map.query".into(),
        args_json: r#"{"qtext":"x"}"#.into(),
        declared: true,
    }];
    plan.packets[0].judges = vec![JudgeRef {
        id: "j".into(),
        kind: "contains".into(),
    }];
    let mut engine = Engine::new();
    engine.register_judge(Judge {
        id: "j".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    });
    let mut receipts = ReceiptLog::default();
    let mut budget = hypermind_toy::mission::EnergyBudget::new(100.0);
    let mut ledger = Vec::new();
    // fail_once: first attempt transport-errors (no retry — transport),
    // so the packet escalates on attempt 1, not retries.
    let mut exec = ScriptExecutor::new()
        .answer("map.query", "PASS")
        .fail_once("map.query");
    let report = engine.run_sequential_live(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut exec,
        &gate,
        &mut ledger,
    );
    assert_eq!(report.escalated, vec!["packet-geometry-01".to_string()]);
    assert!(ledger.iter().any(|l| l.contains("transport")));
}
