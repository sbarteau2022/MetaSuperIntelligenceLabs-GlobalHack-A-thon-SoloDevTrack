use rgah::loop_ctl::{Controller, CycleSignal, Decision, LoopShape, TaskProfile, select_shape};

fn profile(verifiable: bool, parallel: bool, long_running: bool) -> TaskProfile {
    TaskProfile {
        verifiable,
        parallel_independent: parallel,
        long_running,
        cycle_budget: 10,
    }
}

fn fail(budget_used_frac: f64) -> CycleSignal {
    CycleSignal {
        verifier_pass: false,
        stalled: false,
        budget_used_frac,
        error_class: Some("verifier".into()),
    }
}

fn pass(budget_used_frac: f64) -> CycleSignal {
    CycleSignal {
        verifier_pass: true,
        stalled: false,
        budget_used_frac,
        error_class: None,
    }
}

#[test]
fn selects_shape_from_task_properties() {
    assert_eq!(select_shape(&profile(false, false, false)), LoopShape::OneShot);
    assert!(matches!(
        select_shape(&profile(true, false, false)),
        LoopShape::BoundedRetry { .. }
    ));
    assert!(matches!(
        select_shape(&profile(true, true, false)),
        LoopShape::FanOut { .. }
    ));
    assert!(matches!(
        select_shape(&profile(true, false, true)),
        LoopShape::Scheduled { .. }
    ));
}

#[test]
fn two_consecutive_failures_escalate_to_judge_gated() {
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 5 }, 3);
    assert_eq!(c.observe(&fail(0.1)), Decision::Continue);
    let d = c.observe(&fail(0.2));
    assert!(matches!(d, Decision::Reselect(LoopShape::JudgeGated { .. })), "got {d:?}");
    assert_eq!(c.receipts().len(), 1);
    assert!(!c.receipts()[0].reason.is_empty());
}

#[test]
fn budget_burn_downgrades_to_one_shot() {
    let mut c = Controller::new(LoopShape::JudgeGated { max_rounds: 8 }, 3);
    let d = c.observe(&fail(0.9));
    assert_eq!(d, Decision::Reselect(LoopShape::OneShot));
}

#[test]
fn pass_resets_failure_streak() {
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 5 }, 3);
    c.observe(&fail(0.1));
    assert_eq!(c.observe(&pass(0.2)), Decision::Continue);
    assert_eq!(c.observe(&fail(0.3)), Decision::Continue);
}

#[test]
fn switch_cap_stops_the_task() {
    // One allowed switch: OneShot -> BoundedRetry. The next escalation must be refused.
    let mut c = Controller::new(LoopShape::OneShot, 1);
    c.observe(&fail(0.1));
    assert!(matches!(c.observe(&fail(0.1)), Decision::Reselect(_)));
    c.observe(&fail(0.1));
    let d = c.observe(&fail(0.1));
    assert!(matches!(d, Decision::Stop(_)), "got {d:?}");
    assert_eq!(c.receipts().len(), 1);
}
