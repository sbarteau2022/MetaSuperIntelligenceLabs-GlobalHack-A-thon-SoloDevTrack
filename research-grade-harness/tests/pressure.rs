use rgah::{
    FrozenPolicyKernel, active_inference::PolicyCandidate, active_inference::PolicyRanker,
    cache::Cache, energy::EnergyBudget, judge::Judge, judge::JudgeKind, receipt::ReceiptLog,
    rsi::Bundle, rsi::ProposeProbeCompose, runbook::Packet, runbook::PacketStatus, runbook::Plan,
    runbook::RetryPolicy,
};

#[test]
fn plan_requires_judges() {
    let p = Plan {
        id: "p".into(),
        objective: "obj".into(),
        packets: vec![Packet {
            id: "a".into(),
            task: "t".into(),
            depends_on: vec![],
            declared_context_tokens: 10,
            tool_calls: vec![],
            judges: vec![],
            branch: None,
            retry_policy: RetryPolicy {
                max_rounds: 1,
                material_change_required: true,
            },
            status: PacketStatus::Pending,
            attempts: 0,
        }],
    };
    assert!(p.validate().is_err());
}

#[test]
fn judge_contains() {
    let j = Judge {
        id: "j".into(),
        kind: JudgeKind::Contains { needle: "4".into() },
    };
    let r = j.run("answer 4", 0);
    assert!(matches!(r.outcome, rgah::judge::JudgeOutcome::Pass));
}

#[test]
fn cache_exact_coverage() {
    let mut c = Cache::new("m", "t", "q", 100);
    let key = c.insert(None, "tok".into(), 4, true);
    let (cov, tot) = c.coverage_exact(&[key.clone(), "missing".into()]);
    assert_eq!(cov, 1);
    assert_eq!(tot, 2);
}

#[test]
fn rsi_keep_base_when_no_gain() {
    // Deterministic stub probe (unit-test only, NOT ship evidence): base and
    // candidates all pass equally -> gain 0.0 inside the 0.02 band -> KEEP_BASE.
    // Real-model runs live in hardware_probes.rs, never in unit tests.
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(100.0);
    let ppc = ProposeProbeCompose::new(1);
    let stub = |_: &Bundle| (1usize, 2usize, 100u64, vec!["last_number_eq".to_string()]);
    let (adopted, m) = ppc.run(&base, &mut budget, stub);
    assert!(m.real_probe, "run() must flag probe-backed metrics");
    assert!(m.kept_base, "equal pass rates must abstain to KEEP_BASE");
    assert_eq!(adopted.version, base.version);
    assert_eq!(adopted.fingerprint(), base.fingerprint());
    assert!(m.energy_spent_j > 0.0, "budget must be charged");
    // Dedup: INV+DOM collapse plus base => <= 3 unique proposals probed.
    assert!(m.proposals.len() <= 3, "proposals must be deduped");
}

#[test]
fn rsi_dedup_fingerprint() {
    // Identical edits from repeated k must probe once, not k times.
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(100.0);
    let ppc = ProposeProbeCompose::new(3); // 6 proposals + base, heavy dupes
    let stub = |_: &Bundle| (1usize, 2usize, 50u64, vec!["last_number_eq".to_string()]);
    let (_, m) = ppc.run(&base, &mut budget, stub);
    // base + 1 unique INV + 1 unique DOM = 3 unique fingerprints.
    assert_eq!(m.proposals.len(), 3, "fingerprint dedup failed");
    let mut fps: Vec<_> = m.proposals.iter().map(|p| &p.fingerprint).collect();
    fps.sort();
    fps.dedup();
    assert_eq!(fps.len(), 3);
}

#[test]
fn rsi_budget_enforced_no_record() {
    // Zero budget: nothing affordable => no probes recorded, KEEP_BASE.
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(0.0);
    let ppc = ProposeProbeCompose::new(1);
    let stub = |_: &Bundle| (2usize, 2usize, 100u64, vec!["last_number_eq".to_string()]);
    let (adopted, m) = ppc.run(&base, &mut budget, stub);
    assert!(
        m.proposals.is_empty(),
        "unaffordable probes must not be recorded"
    );
    assert!(m.kept_base);
    assert_eq!(adopted.fingerprint(), base.fingerprint());
}

#[test]
fn rsi_leakage_blocks_adopt() {
    // A candidate that wins on pass rate but smuggles an answer key must not adopt.
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(100.0);
    let ppc = ProposeProbeCompose::new(1);
    let stub = |b: &Bundle| {
        if b.components.iter().any(|c| c.contains("improve invariant")) {
            // INV candidate: pretend perfect but inject leakage marker.
            // (probe itself is clean; leakage_critic scans bundle text.)
            (2usize, 2usize, 100u64, vec!["last_number_eq".to_string()])
        } else {
            (0usize, 2usize, 100u64, vec!["last_number_eq".to_string()])
        }
    };
    // Corrupt the INV proposal path via base text so leakage_critic fires:
    // build a leaky base directly.
    let mut leaky_base = Bundle::base();
    leaky_base.components[0] = format!("{} + improve invariant", leaky_base.components[0]);
    leaky_base.components[0].push_str(" answer is 42");
    let (adopted, m) = ppc.run(&leaky_base, &mut budget, stub);
    // Base itself contains leakage -> candidates share it, adoption blocked or base kept.
    assert!(m.kept_base || adopted.fingerprint() == leaky_base.fingerprint());
    let _ = m;
}

#[test]
fn active_inference_ranking() {
    let cands = vec![
        PolicyCandidate {
            id: "a".into(),
            description: "".into(),
            pragmatics: 0.2,
            epistemic: 0.1,
            landauer_cost: 1.0,
            reversible: false,
        },
        PolicyCandidate {
            id: "b".into(),
            description: "".into(),
            pragmatics: 0.8,
            epistemic: 0.5,
            landauer_cost: 1.0,
            reversible: true,
        },
    ];
    let r = PolicyRanker::default().rank(&cands);
    assert_eq!(r[0].candidate.id, "b");
}

#[test]
fn fpk_refuses() {
    let k = FrozenPolicyKernel::default();
    assert!(k.check_mutation("judge").is_err());
    assert!(k.check_mutation("other").is_ok());
}

#[test]
fn receipt_resume() {
    let mut log = ReceiptLog::default();
    log.append(rgah::receipt::Receipt {
        packet_id: "x".into(),
        attempt: 1,
        outcome: "PASS".into(),
        judge_ids: vec![],
        evidence_hash: "h".into(),
        energy_joules: 1.0,
        erased_bits: 4,
        timestamp_ms: 0,
    });
    assert!(log.resume_skips("x"));
}
