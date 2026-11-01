#![cfg(feature = "synthetic-scaffold")]

use rgah::{
    active_inference::PolicyRanker,
    cache::Cache,
    energy::EnergyBudget,
    engine::Engine,
    judge::{Judge, JudgeKind},
    receipt::ReceiptLog,
    rsi::{Bundle, ProposeProbeCompose},
    runbook::{JudgeRef, Packet, PacketStatus, Plan, RetryPolicy},
};

// VERIFIED: 40-packet DAG (recon -> plan -> 5 lanes -> integrate -> verify ->
// harden -> ship chain (29) + 1 redteam leaf). Counts as 40 steps minimum;
// with 1 redteam retry = 41 packet executions. In 30-50 band per packet-execution counting.
fn build_longtask_plan() -> Plan {
    let mut packets: Vec<Packet> = Vec::new();
    let std_retry = RetryPolicy {
        max_rounds: 2,
        material_change_required: true,
    };
    let pass_judge = JudgeRef {
        id: "j-pass".into(),
        kind: "contains".into(),
    };
    let red_judge = JudgeRef {
        id: "j-red".into(),
        kind: "contains".into(),
    };
    let add = |id: &str, deps: Vec<&str>, judges: Vec<JudgeRef>| Packet {
        id: id.into(),
        task: format!("task-{}", id),
        depends_on: deps.into_iter().map(|s| s.into()).collect(),
        declared_context_tokens: 10,
        tool_calls: vec![],
        judges,
        branch: None,
        retry_policy: std_retry.clone(),
        status: PacketStatus::Pending,
        attempts: 0,

        judge_joined: false,
    };
    packets.push(add("p00-recon", vec![], vec![pass_judge.clone()]));
    packets.push(add("p01-plan", vec!["p00-recon"], vec![pass_judge.clone()]));
    for lane in ["a", "b", "c", "d", "e"] {
        packets.push(add(
            &format!("p02-lane-{}", lane),
            vec!["p01-plan"],
            vec![pass_judge.clone()],
        ));
    }
    packets.push(add(
        "p07-integrate",
        vec![
            "p02-lane-a",
            "p02-lane-b",
            "p02-lane-c",
            "p02-lane-d",
            "p02-lane-e",
        ],
        vec![pass_judge.clone()],
    ));
    packets.push(add(
        "p08-verify",
        vec!["p07-integrate"],
        vec![pass_judge.clone()],
    ));
    packets.push(add(
        "p09-harden",
        vec!["p08-verify"],
        vec![pass_judge.clone()],
    ));
    // Redteam leaf: depends on verify, nothing depends on it -> escalation causes no cascade.
    packets.push(add("p10-redteam", vec!["p08-verify"], vec![red_judge]));
    // Ship chain p11..p39 (29 packets) linear after harden.
    let mut prev = "p09-harden".to_string();
    for i in 11..=39 {
        let id = format!("p{:02}-ship-{}", i, i);
        packets.push(add(
            &id,
            vec![Box::leak(prev.clone().into_boxed_str()) as &str],
            vec![pass_judge.clone()],
        ));
        prev = id;
    }
    Plan {
        id: "longtask-40".into(),
        objective: "40-packet certification long task".into(),
        packets,
    }
}

#[test]
#[ignore = "historical synthetic scaffold; never acceptance evidence"]
fn longtask_40_packet_head_to_head() {
    // Plan validation (MadMind hard gate).
    let mut plan = build_longtask_plan();
    assert_eq!(plan.packets.len(), 40);
    assert!(plan.validate().is_ok());

    let mut eng = Engine::new();
    eng.register_judge(Judge {
        id: "j-pass".into(),
        kind: JudgeKind::Contains {
            needle: "PASS".into(),
        },
    });
    eng.register_judge(Judge {
        id: "j-red".into(),
        kind: JudgeKind::Contains {
            needle: "__UNMEETABLE_NEEDLE__".into(),
        },
    });
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(50.0);
    let report = eng.run_sequential(&mut plan, &mut receipts, &mut budget);

    // P-04: 39 passed, 1 escalated (redteam leaf), 0 skipped cascade.
    assert_eq!(report.passed.len(), 39, "passed={:?}", report.passed);
    assert_eq!(report.escalated, vec!["p10-redteam".to_string()]);
    assert!(report.skipped.is_empty(), "skipped={:?}", report.skipped);
    assert_eq!(receipts.entries.len(), 40);
    let pass_n = receipts
        .entries
        .iter()
        .filter(|r| r.outcome == "PASS")
        .count();
    let esc_n = receipts
        .entries
        .iter()
        .filter(|r| r.outcome == "ESCALATE")
        .count();
    assert_eq!(pass_n, 39);
    assert_eq!(esc_n, 1);

    // P-05: energy slack holds; every packet (incl. redteam retry) charged.
    assert!(budget.remaining() >= 0.0);
    assert!(budget.spent_j > 0.0);
    assert!(budget.landauer_bits_erased > 0);

    // Packet executions: 39x1 + 1x2 (redteam retry) = 41 steps. In 30-50 band.
    let executions: u32 = plan.packets.iter().map(|p| p.attempts).sum();
    assert_eq!(executions, 41);
    assert!((30..=50).contains(&executions));

    // P-07: exact declared-context coverage over all 40 packet keys after phased decay.
    let mut cache = Cache::new("m", "t", "q", 128);
    let mut keys: Vec<String> = Vec::new();
    for p in &plan.packets {
        let k = cache.insert(None, format!("tok-{}", p.id), 10, true);
        keys.push(k);
    }
    for _ in 0..3 {
        cache.decay_epoch();
    }
    let (covered, total) = cache.coverage_exact(&keys);
    println!("coverage_exact {}/{}", covered, total);
    assert_eq!(covered, 40);
    assert_eq!(total, 40);

    // P-06: PPC k=2 honesty — either outcome accepted; abstention reported.
    // Unit scaffold uses a deterministic stub probe (NOT ship evidence).
    let base = Bundle::base();
    let mut ppc_budget = EnergyBudget::new(50.0);
    let ppc = ProposeProbeCompose::new(2);
    let stub = |_: &Bundle| (1usize, 2usize, 100u64, vec!["last_number_eq".to_string()]);
    let (adopted, m) = ppc.run(&base, &mut ppc_budget, stub);
    println!(
        "ppc k={} rollouts={} sessions={} kept_base={} abstained={} gain={:.3} energy={:.4} adopted_v={} real_probe={}",
        m.k,
        m.rollouts,
        m.sessions,
        m.kept_base,
        m.abstained,
        m.quality_gain,
        m.energy_spent_j,
        adopted.version,
        m.real_probe
    );
    assert_eq!(m.rollouts, 8); // Harness2 accounting R=4k
    assert_eq!(m.sessions, 6); // P=3k

    // AIF ranker smoke (EFE-ranked policy selection path exercised on long task).
    let cands = vec![
        rgah::active_inference::PolicyCandidate {
            id: "uniform-probe".into(),
            description: "baseline".into(),
            pragmatics: 0.5,
            epistemic: 0.3,
            landauer_cost: 1.0,
            reversible: false,
        },
        rgah::active_inference::PolicyCandidate {
            id: "eife-probe".into(),
            description: "ranked".into(),
            pragmatics: 0.6,
            epistemic: 0.6,
            landauer_cost: 0.8,
            reversible: true,
        },
    ];
    let ranked = PolicyRanker::default().rank(&cands);
    assert_eq!(ranked[0].candidate.id, "eife-probe");

    println!(
        "LONGTASK passed=39 escalated=1 receipts=40 executions=41 energy_spent={:.4} erased={} coverage=40/40",
        budget.spent_j, budget.landauer_bits_erased
    );
}
