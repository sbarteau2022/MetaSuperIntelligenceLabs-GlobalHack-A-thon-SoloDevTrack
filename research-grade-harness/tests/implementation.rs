use rgah::{
    cordis::HmrGate,
    energy::EnergyBudget,
    engine::Engine,
    hear::{EngineRuntimeState, HearCoordinator, WorkflowIntent},
    judge::Judge,
    judge::JudgeKind,
    mode::{Command, TacticalLevel, allowed_commands, authorize, raise_level},
    receipt::ReceiptLog,
    register::{HypothesisRegister, ProbePlan, rank_probes},
    rsi::GovernedLibrary,
    runbook::{Packet, PacketStatus, Plan, RetryPolicy},
    temple::TempleRecord,
};

#[test]
fn engine_sequential_passes() {
    let mut plan = Plan {
        id: "p".into(),
        objective: "o".into(),
        packets: vec![Packet {
            id: "a".into(),
            task: "t".into(),
            depends_on: vec![],
            declared_context_tokens: 10,
            tool_calls: vec![],
            judges: vec![rgah::runbook::JudgeRef {
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
        }],
    };
    let mut eng = Engine::new();
    eng.register_judge(Judge {
        id: "j".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    });
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(10.0);
    let report = eng.run_sequential(&mut plan, &mut receipts, &mut budget);
    assert_eq!(report.passed, vec!["a".to_string()]);
    assert!(receipts.resume_skips("a"));
}

#[test]
fn engine_parallel_runs() {
    let mut plan = Plan {
        id: "p".into(),
        objective: "o".into(),
        packets: vec![
            Packet {
                id: "a".into(),
                task: "t".into(),
                depends_on: vec![],
                declared_context_tokens: 5,
                tool_calls: vec![],
                judges: vec![rgah::runbook::JudgeRef {
                    id: "j".into(),
                    kind: "c".into(),
                }],
                branch: None,
                retry_policy: RetryPolicy {
                    max_rounds: 1,
                    material_change_required: true,
                },
                status: PacketStatus::Pending,
                attempts: 0,

                judge_joined: false,
            },
            Packet {
                id: "b".into(),
                task: "t".into(),
                depends_on: vec!["a".into()],
                declared_context_tokens: 5,
                tool_calls: vec![],
                judges: vec![rgah::runbook::JudgeRef {
                    id: "j".into(),
                    kind: "c".into(),
                }],
                branch: None,
                retry_policy: RetryPolicy {
                    max_rounds: 1,
                    material_change_required: true,
                },
                status: PacketStatus::Pending,
                attempts: 0,

                judge_joined: false,
            },
        ],
    };
    let mut eng = Engine::new();
    eng.register_judge(Judge {
        id: "j".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    });
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(10.0);
    let report = eng.run_parallel(&mut plan, &mut receipts, &mut budget);
    assert_eq!(report.passed.len(), 2);
}

#[test]
fn mode_authorization_monotone() {
    let l = TacticalLevel::T0March;
    assert!(authorize(l, Command::Run).is_ok());
    assert!(authorize(l, Command::Probe).is_err());
    assert!(raise_level(l, TacticalLevel::T4GrandStrategy) == TacticalLevel::T4GrandStrategy);
    assert!(
        allowed_commands(TacticalLevel::T4GrandStrategy).len()
            > allowed_commands(TacticalLevel::T0March).len()
    );
}

#[test]
fn temple_completeness_gate() {
    let mut t = TempleRecord::default();
    assert!(!t.complete());
    t.ground = Some("g".into());
    t.forces = Some("f".into());
    t.command = Some("c".into());
    t.method = Some("m".into());
    t.worth = Some("w".into());
    t.sizing = Some("s".into());
    t.waste_budget = Some(1.0);
    t.gates.push("gate".into());
    assert!(t.complete());
}

#[test]
fn register_probe_ranking_and_strike() {
    let mut reg = HypothesisRegister::default();
    reg.add("h1", "shape1", 0.5);
    reg.add("h2", "shape2", 0.9);
    reg.strike("h2");
    assert_eq!(reg.best_confidence(), 0.5);
    let mut probes = vec![
        ProbePlan {
            hypothesis_id: "h1".into(),
            expected_info: 1.0,
            cost: 2.0,
        },
        ProbePlan {
            hypothesis_id: "h2".into(),
            expected_info: 5.0,
            cost: 1.0,
        },
    ];
    rank_probes(&mut probes);
    assert_eq!(probes[0].hypothesis_id, "h2");
    assert!(reg.should_commit(3.0, 2.0));
}

#[test]
fn hear_coordination() {
    let intent = WorkflowIntent {
        declared_tokens: 10,
        needs_prefill: true,
    };
    let state = EngineRuntimeState {
        queue_depth: 0,
        pressure: false,
    };
    let d = HearCoordinator::coordinate(&intent, &state);
    assert!(d.prefill);
    let state2 = EngineRuntimeState {
        queue_depth: 10,
        pressure: true,
    };
    let d2 = HearCoordinator::coordinate(&intent, &state2);
    assert!(!d2.prefill);
}

#[test]
fn hmr_gate() {
    let g = HmrGate {
        signature_ok: true,
        shadow_receipts_ok: true,
        judge_suite_ok: true,
        energy_ok: true,
        rollback_retained: true,
    };
    assert!(g.allows());
    let g2 = HmrGate {
        signature_ok: false,
        ..g
    };
    assert!(!g2.allows());
}

#[test]
fn governed_library_sid() {
    let mut lib = GovernedLibrary::default();
    let sid = lib.nominate(rgah::rsi::Bundle::base(), "lesson");
    assert!(lib.promote(&sid, true, true, true));
    assert!(lib.entries[0].promoted);
}
