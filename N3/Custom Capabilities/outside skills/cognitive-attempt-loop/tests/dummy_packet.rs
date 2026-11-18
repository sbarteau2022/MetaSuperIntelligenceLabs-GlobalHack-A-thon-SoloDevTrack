//! Dummy packet: clean -> assessed_clean, defective -> caught, flaky -> unresolved.
//! Every packet TERMINATES (Terminated, or Escalated after two strikes) with
//! iteration proof (iter <= MAX_ITERS, receipt count, strike count).

use cognitive_attempt_loop::{AttemptMachine, MachineState, TerminalOutcome, MAX_ITERS, STRIKE_LIMIT};
use cogmap_slice2::attempt_loop::{
    CriterionKind, CriterionSpec, FlakySecondRun, LocalProcessRunner, Session, TaskTemplate,
};

fn fix(rel: &str) -> String {
    let p = format!(
        "{}/../../cognitive-map/slice2/{}",
        env!("CARGO_MANIFEST_DIR"),
        rel
    );
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("load {p}: {e}"))
}

fn clean_template() -> TaskTemplate {
    TaskTemplate {
        template_id: "T-CLEAN-1".into(),
        concept_id: "search-return".into(),
        dimension: "SPEC".into(),
        difficulty: 1,
        subtlety: 1,
        variant: "clean-a".into(),
        independent: true,
        defective_task: false,
        required_defects: Vec::new(),
        decoys: vec!["X1".into()],
        ground_truth: "verdict=clean;decoy=X1(return -1 line)".into(),
        task_body: fix("fixtures/clean/snippet.py"),
        test_bundle_name: "clean/test_bundle.py".into(),
        test_bundle: fix("fixtures/clean/test_bundle.py"),
        criteria: vec![
            CriterionSpec { test_id: "t_verdict".into(), kind: CriterionKind::Verdict },
            CriterionSpec { test_id: "t_decoy_X1".into(), kind: CriterionKind::Decoy("X1".into()) },
            CriterionSpec { test_id: "t_evidence".into(), kind: CriterionKind::Evidence },
        ],
    }
}

fn defective_template() -> TaskTemplate {
    TaskTemplate {
        template_id: "T-DEFECT-1".into(),
        concept_id: "search-return".into(),
        dimension: "SPEC".into(),
        difficulty: 1,
        subtlety: 2,
        variant: "defect-a".into(),
        independent: true,
        defective_task: true,
        required_defects: vec!["D1".into()],
        decoys: Vec::new(),
        ground_truth: "verdict=defect;defect=D1(range one past end)".into(),
        task_body: fix("fixtures/defective/snippet.py"),
        test_bundle_name: "defective/test_bundle.py".into(),
        test_bundle: fix("fixtures/defective/test_bundle.py"),
        criteria: vec![
            CriterionSpec { test_id: "t_verdict".into(), kind: CriterionKind::Verdict },
            CriterionSpec {
                test_id: "t_loc_D1".into(),
                kind: CriterionKind::Localization("D1".into()),
            },
            CriterionSpec { test_id: "t_evidence".into(), kind: CriterionKind::Evidence },
        ],
    }
}

fn post_run_receipt(m: &AttemptMachine) -> &cognitive_attempt_loop::TransitionReceipt {
    m.receipts()
        .iter()
        .find(|rc| rc.job_id.is_some())
        .expect("machine must mint a post-run receipt with runner evidence")
}

#[test]
fn packet_clean_terminates_assessed_clean() {
    let t = clean_template();
    let mut s = Session::new();
    let mut m = AttemptMachine::new();
    m.plan(&mut s, &t, "pkt-clean", b"VERDICT: clean (sealed)").unwrap();
    assert_eq!(*m.state(), MachineState::Run);
    m.run(&mut s, "pkt-clean", "pkt-clean-sub", fix("fixtures/clean/submission_good.txt").as_bytes())
        .unwrap();
    assert_eq!(*m.state(), MachineState::Judge);
    let mut r = LocalProcessRunner::new();
    let out = m.judge(&mut s, &t, &mut r, "pkt-clean", "pkt-clean-g1").unwrap();
    assert_eq!(out, TerminalOutcome::AssessedClean);
    assert_eq!(*m.state(), MachineState::Terminated);
    assert_eq!(m.iter(), 1);
    assert!(m.iter() <= MAX_ITERS);
    let rc = post_run_receipt(&m);
    assert!(rc.env_digest.is_some());
    assert!(rc.input_hash.is_some());
    assert!(rc.test_bundle_hash.is_some());
    assert!(rc.exit_status.is_some());
    let g = rc.grade.as_ref().expect("grade on assessed receipt");
    assert!((g.v, g.j) == (1, 3) && g.fully_clean && !g.false_negative && !g.false_positive);
    eprintln!(
        "PACKET clean: TERMINATED outcome={:?} iter={} receipts={} strikes={}",
        out, m.iter(), m.receipts().len(), m.strikes()
    );
}

#[test]
fn packet_defective_terminates_caught() {
    let t = defective_template();
    let mut s = Session::new();
    let mut m = AttemptMachine::new();
    m.plan(&mut s, &t, "pkt-defect", b"VERDICT: defect D1 (sealed)").unwrap();
    m.run(
        &mut s,
        "pkt-defect",
        "pkt-defect-sub",
        fix("fixtures/defective/submission_good.txt").as_bytes(),
    )
    .unwrap();
    let mut r = LocalProcessRunner::new();
    let out = m.judge(&mut s, &t, &mut r, "pkt-defect", "pkt-defect-g1").unwrap();
    assert_eq!(out, TerminalOutcome::Caught);
    assert_eq!(*m.state(), MachineState::Terminated);
    assert_eq!(m.iter(), 1);
    assert!(m.iter() <= MAX_ITERS);
    let rc = post_run_receipt(&m);
    let g = rc.grade.as_ref().expect("grade on caught receipt");
    assert!(!g.false_negative && !g.false_positive && !g.fully_clean);
    eprintln!(
        "PACKET defective: TERMINATED outcome={:?} iter={} receipts={} strikes={}",
        out, m.iter(), m.receipts().len(), m.strikes()
    );
}

#[test]
fn packet_flaky_two_strikes_escalate_terminated() {
    let t = clean_template();
    let good = fix("fixtures/clean/submission_good.txt");
    let mut s = Session::new();
    let mut m = AttemptMachine::new();
    let mut r = FlakySecondRun::new(LocalProcessRunner::new());
    // Strike 1: double-run diverges -> Unresolved, back to Plan.
    m.plan(&mut s, &t, "pkt-flaky-1", b"VERDICT: clean (sealed)").unwrap();
    m.run(&mut s, "pkt-flaky-1", "pkt-flaky-sub-1", good.as_bytes()).unwrap();
    let out1 = m.judge(&mut s, &t, &mut r, "pkt-flaky-1", "pkt-flaky-g1").unwrap();
    assert_eq!(out1, TerminalOutcome::Unresolved);
    assert_eq!(m.strikes(), 1);
    assert_eq!(*m.state(), MachineState::Plan);
    // Strike 2 (fresh attempt id, bounded retry): STRIKE_LIMIT -> Escalated in code.
    m.plan(&mut s, &t, "pkt-flaky-2", b"VERDICT: clean (sealed)").unwrap();
    m.run(&mut s, "pkt-flaky-2", "pkt-flaky-sub-2", good.as_bytes()).unwrap();
    let out2 = m.judge(&mut s, &t, &mut r, "pkt-flaky-2", "pkt-flaky-g2").unwrap();
    assert_eq!(out2, TerminalOutcome::Escalated);
    assert_eq!(*m.state(), MachineState::Escalated);
    assert_eq!(m.strikes(), STRIKE_LIMIT);
    assert!(m.iter() <= MAX_ITERS);
    assert_eq!(m.iter(), 2);
    eprintln!(
        "PACKET flaky: TERMINATED-ESCALATED first={:?} second={:?} iter={} receipts={} strikes={}",
        out1, out2, m.iter(), m.receipts().len(), m.strikes()
    );
}

#[test]
fn packet_dispute_clears_evidence_terminated() {
    let t = clean_template();
    let mut s = Session::new();
    let mut m = AttemptMachine::new();
    m.plan(&mut s, &t, "pkt-disp", b"VERDICT: clean (sealed)").unwrap();
    m.run(&mut s, "pkt-disp", "pkt-disp-sub", fix("fixtures/clean/submission_good.txt").as_bytes())
        .unwrap();
    let mut r = LocalProcessRunner::new();
    let out = m.judge(&mut s, &t, &mut r, "pkt-disp", "pkt-disp-g1").unwrap();
    assert_eq!(out, TerminalOutcome::AssessedClean);
    assert_eq!(s.state.observations.len(), 1);
    let dout = m.dispute(&mut s, "pkt-disp").unwrap();
    assert_eq!(dout, TerminalOutcome::Disputed);
    assert!(s.state.observations.is_empty());
    assert_eq!(*m.state(), MachineState::Terminated);
    eprintln!(
        "PACKET dispute: TERMINATED outcome={:?} iter={} receipts={}",
        dout, m.iter(), m.receipts().len()
    );
}
