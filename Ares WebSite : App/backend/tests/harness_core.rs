//! RG-AH harness acceptance tests (native backend port).
//! Baseline (source crate `rgah`, 2026-11-02): 45 unit + 36 integration
//! pass; `a14_run01` fails only on a missing external manifest — an
//! environmental baseline failure, recorded, not a harness defect.

use ares_backend::harness::mission::{
    dry_run, geometry_mission, research_gate, FrozenPolicyKernel, InformationNeed, Judge,
    JudgeKind, Packet, PacketStatus, Plan, Receipt, ReceiptLog, ResearchRecord, RetryPolicy,
    SessionEventKind, SessionLog, ToolCall, WorkingSetCache,
};
use ares_backend::harness::qdrant_memory::embed_text;
use ares_backend::harness::runtime::{
    run_mission_local, run_mission_with_prior_local, CapabilityGate, CapabilityNode, Engine,
};
use ares_backend::harness::telemetry::{HyperdxConfig, HyperdxTelemetry};

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
            judges: vec![ares_backend::harness::mission::JudgeRef {
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
fn receipt_chain_detects_tamper_and_forged_prior_skips_nothing() {
    let mut log = ReceiptLog::default();
    log.append(Receipt {
        packet_id: "p".into(),
        attempt: 1,
        outcome: "PASS".into(),
        judge_ids: vec!["j".into()],
        evidence_hash: "forged".into(),
        energy_joules: 0.5,
        erased_bits: 4,
        timestamp_ms: 0,
        prev_hash: "forged".into(),
        n3_receipts: vec![],
    });
    assert_eq!(log.verify_chain(), Ok(1));
    assert_ne!(log.entries[0].evidence_hash, "forged");
    assert!(log.entries[0].timestamp_ms > 0);
    log.entries[0].outcome = "FAIL".into();
    assert_eq!(log.verify_chain(), Err(0));

    // Forged prior (hand-built, no chain) marks nothing passed.
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
    let input = ares_backend::harness::mission::MissionInput {
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
        ares_backend::harness::mission::JudgeOutcome::Pass
    ));
    assert!(matches!(
        j.run("nothing", 0).outcome,
        ares_backend::harness::mission::JudgeOutcome::Fail(_)
    ));
    let g = Judge {
        id: "g".into(),
        kind: JudgeKind::GoldenHash {
            expected: "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".into(),
        },
    };
    assert!(matches!(
        g.run("hello", 0).outcome,
        ares_backend::harness::mission::JudgeOutcome::Pass
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
fn research_gate_and_policy_kernel() {
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

    let fpk = FrozenPolicyKernel::default();
    assert!(fpk.check_mutation("judge").is_err());
    assert!(fpk.check_mutation("anything-else").is_ok());
}

#[test]
fn working_set_cache_mass_and_eviction() {
    let mut c = WorkingSetCache::new("m", "t", "q", 2);
    let k1 = c.insert(None, "alpha".into(), 10, true);
    let _k2 = c.insert(None, "beta".into(), 10, false);
    assert_eq!(c.blocks.len(), 2);
    c.decay_epoch();
    assert!(c.blocks[&k1].mass > 0);
    // Release refcounts so eviction can reclaim the unanchored block.
    for b in c.blocks.values_mut() {
        b.refcount = 0;
    }
    assert!(c.evict_one());
}

#[test]
fn hyperdx_exporter_never_puts_key_in_payload() {
    let cfg = HyperdxConfig::from_env();
    let telemetry = HyperdxTelemetry::new(cfg).unwrap();
    let resource = telemetry.resource_json().to_string();
    assert!(!resource.contains("HYPERDX_API_KEY_STRING_REMOVED"));
    assert!(!resource.contains("api_key"));
}

#[test]
fn memory_embedding_is_deterministic_and_normalized() {
    let a = embed_text("hyperbolic KV cache nested in the mind map");
    let b = embed_text("hyperbolic KV cache nested in the mind map");
    assert_eq!(a.len(), 384);
    assert_eq!(a, b, "same text must get the same hashed embedding");
    let norm: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={norm}");
}
