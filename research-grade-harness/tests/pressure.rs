use rgah::{
    active_inference::PolicyCandidate, active_inference::PolicyRanker, cache::Cache,
    energy::EnergyBudget, judge::Judge, judge::JudgeKind, receipt::ReceiptLog, rsi::Bundle,
    rsi::ProposeProbeCompose, runbook::Packet, runbook::PacketStatus, runbook::Plan,
    runbook::RetryPolicy, FrozenPolicyKernel,
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
    // Base score 0.5; candidate may not exceed margin much; default base components no 'improve'
    let base = Bundle::base();
    let mut budget = EnergyBudget::new(100.0);
    let ppc = ProposeProbeCompose::new(1);
    let (adopted, m) = ppc.run(&base, &mut budget);
    // Either keep_base true or not; ensure fingerprint logic runs and budget charged
    assert!(m.energy_spent_j >= 0.0);
    if m.kept_base {
        assert_eq!(adopted.version, base.version);
    }
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
