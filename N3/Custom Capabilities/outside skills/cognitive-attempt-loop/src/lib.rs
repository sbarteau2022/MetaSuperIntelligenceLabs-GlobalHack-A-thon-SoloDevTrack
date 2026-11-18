//! cognitive-attempt-loop — compiled state machine over Slice 2 (`attempt_loop`).
//!
//! REFERENCE ONLY: this crate path-depends on `cogmap-slice2` and never copies,
//! deletes, or modifies `/Users/stewartbarteau/Desktop/N3/cognitive-map/slice2/`.
//! Defer-deletion dependents (Agent 5 call, re-verified 2026-10-31): see DEFER-DELETION.md.
//!
//! Machine: Plan -> Run -> Judge -> Receipt, bounded, two-strike escalation.
//! Every transition mints a [`TransitionReceipt`] carrying the runner-evidence
//! fields (job_id / env_digest / input_hash / test_bundle_hash / exit_status / grade)
//! so a grade is reproducible per Slice 2 §D.2 / §H.

use cogmap_slice2::attempt_loop::{
    Grade, GradeOutcome, LoopError, Runner, RunnerJob, RunnerRecord, Session, TaskTemplate,
};

/// Hard iteration bound. Task GATE: `MAX_ITERS <= 8`.
pub const MAX_ITERS: u8 = 8;

/// Consecutive unresolved (flaky) strikes that force escalation.
pub const STRIKE_LIMIT: u8 = 2;

/// Compiled machine states. Linear order Plan -> Run -> Judge -> Receipt,
/// plus terminal states. No transition may skip Judge (grading is mandatory).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineState {
    /// Attempt assigned, prediction sealed, task body revealed. Ready to run.
    Plan,
    /// Submission recorded, runner job dispatched (runs twice internally).
    Run,
    /// Runner records in hand, deterministic grade computed.
    Judge,
    /// Grade admitted (or dispute/unresolved recorded). Transition closed.
    Receipt,
    /// Flaky/unavailable twice in a row — human review required.
    Escalated,
    /// Terminal: assessed (head + observation written) or disputed/closed.
    Terminated,
}

/// Terminal outcome of one machine pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalOutcome {
    AssessedClean,
    Caught,
    Missed,
    FalsePositive,
    Unresolved,
    Disputed,
    Escalated,
}

/// One receipt per transition. Runner-evidence fields are `Option` because
/// pre-run transitions (assign/commit/reveal/submit) have no runner record yet;
/// every post-run transition MUST populate all of them (enforced by
/// [`TransitionReceipt::post_run`]).
#[derive(Clone, Debug)]
pub struct TransitionReceipt {
    pub from: MachineState,
    pub to: MachineState,
    pub iter: u8,
    pub job_id: Option<String>,
    pub env_digest: Option<String>,
    pub input_hash: Option<String>,
    pub test_bundle_hash: Option<String>,
    pub exit_status: Option<i32>,
    pub grade: Option<GradeSummary>,
}

/// Integer-only grade summary (mirrors Slice 2 `Grade`; no floats anywhere).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradeSummary {
    pub v: u8,
    pub j: u8,
    pub q: u8,
    pub false_negative: bool,
    pub false_positive: bool,
    pub fully_clean: bool,
}

impl From<&Grade> for GradeSummary {
    fn from(g: &Grade) -> Self {
        GradeSummary {
            v: g.v,
            j: g.j,
            q: g.q,
            false_negative: g.false_negative,
            false_positive: g.false_positive,
            fully_clean: g.fully_clean,
        }
    }
}

impl TransitionReceipt {
    /// Pre-run transition: no runner evidence yet.
    pub fn pre_run(from: MachineState, to: MachineState, iter: u8) -> Self {
        TransitionReceipt {
            from,
            to,
            iter,
            job_id: None,
            env_digest: None,
            input_hash: None,
            test_bundle_hash: None,
            exit_status: None,
            grade: None,
        }
    }

    /// Post-run transition: all runner-evidence fields REQUIRED.
    pub fn post_run(
        from: MachineState,
        to: MachineState,
        iter: u8,
        record: &RunnerRecord,
        grade: Option<&Grade>,
    ) -> Self {
        TransitionReceipt {
            from,
            to,
            iter,
            job_id: Some(record.job_id.clone()),
            env_digest: Some(record.env_digest.clone()),
            input_hash: Some(record.input_hash.clone()),
            test_bundle_hash: Some(record.test_bundle_hash.clone()),
            exit_status: Some(record.exit_status),
            grade: grade.map(GradeSummary::from),
        }
    }
}

/// Errors unique to the machine wrapper (Slice 2 errors pass through as `Loop`).
#[derive(Debug, PartialEq)]
pub enum MachineError {
    Loop(LoopError),
    IterBudgetExhausted,
    BadTransition { from: MachineState, what: &'static str },
}

impl From<LoopError> for MachineError {
    fn from(e: LoopError) -> Self {
        MachineError::Loop(e)
    }
}

/// A single-channel runner probe: runs the job once through the inner runner
/// to capture the evidence record for the receipt, then delegates the real
/// double-run verdict to `Session::grade_and_admit` (which runs twice more).
/// The double-run flakiness gate inside Slice 2 is authoritative; this probe
/// only supplies receipt fields and never overrides the verdict.
struct Probe<'a, R: Runner> {
    inner: &'a mut R,
    last: Option<RunnerRecord>,
}

impl<R: Runner> Runner for Probe<'_, R> {
    fn run(&mut self, job: &RunnerJob) -> Result<RunnerRecord, LoopError> {
        let rec = self.inner.run(job)?;
        self.last = Some(rec.clone());
        Ok(rec)
    }
}

/// The compiled machine. Owns one Slice 2 `Session` (single state holder),
/// an iteration counter bounded by `MAX_ITERS`, and a consecutive-flaky
/// strike counter that escalates at `STRIKE_LIMIT`.
pub struct AttemptMachine {
    state: MachineState,
    iter: u8,
    strikes: u8,
    receipts: Vec<TransitionReceipt>,
}

impl AttemptMachine {
    pub fn new() -> Self {
        AttemptMachine {
            state: MachineState::Plan,
            iter: 0,
            strikes: 0,
            receipts: Vec::new(),
        }
    }

    pub fn state(&self) -> &MachineState {
        &self.state
    }

    pub fn iter(&self) -> u8 {
        self.iter
    }

    pub fn strikes(&self) -> u8 {
        self.strikes
    }

    pub fn receipts(&self) -> &[TransitionReceipt] {
        &self.receipts
    }

    fn bump_iter(&mut self) -> Result<(), MachineError> {
        if self.iter >= MAX_ITERS {
            return Err(MachineError::IterBudgetExhausted);
        }
        self.iter += 1;
        Ok(())
    }

    /// Plan: assign -> commit_prediction -> reveal. Preserves Slice 2
    /// commit-before-reveal ordering (reveal without prediction is rejected
    /// by the reducer; content withheld).
    pub fn plan(
        &mut self,
        session: &mut Session,
        template: &TaskTemplate,
        attempt_id: &str,
        prediction: &[u8],
    ) -> Result<(), MachineError> {
        if self.state != MachineState::Plan {
            return Err(MachineError::BadTransition {
                from: self.state.clone(),
                what: "plan requires Plan state",
            });
        }
        self.bump_iter()?;
        let n = self.iter;
        session.assign(template, &format!("{attempt_id}-assign-i{n}"), attempt_id)?;
        session.commit_prediction(attempt_id, prediction, &format!("{attempt_id}-pred-i{n}"))?;
        session.reveal(attempt_id, &format!("{attempt_id}-reveal-i{n}"), template)?;
        self.receipts.push(TransitionReceipt::pre_run(
            MachineState::Plan,
            MachineState::Run,
            self.iter,
        ));
        self.state = MachineState::Run;
        Ok(())
    }

    /// Run: submit immutable finalized artifact. Idempotent retry (same id +
    /// same bytes) is a no-op receipt, never a state change (Slice 2 I03).
    pub fn run(
        &mut self,
        session: &mut Session,
        attempt_id: &str,
        submission_id: &str,
        artifact: &[u8],
    ) -> Result<(), MachineError> {
        if self.state != MachineState::Run {
            return Err(MachineError::BadTransition {
                from: self.state.clone(),
                what: "run requires Run state",
            });
        }
        session.submit(
            attempt_id,
            submission_id,
            artifact,
            &format!("{attempt_id}-submit-i{}", self.iter),
        )?;
        self.receipts.push(TransitionReceipt::pre_run(
            MachineState::Run,
            MachineState::Judge,
            self.iter,
        ));
        self.state = MachineState::Judge;
        Ok(())
    }

    /// Judge -> Receipt: grade_and_admit through the double-run gate.
    /// - Assessed: strike counter resets, receipt carries full evidence.
    /// - Unresolved (flaky): strike++; at STRIKE_LIMIT the machine Escalates
    ///   in code (no further auto-retries); below the limit it returns to
    ///   Plan for one more bounded iteration.
    pub fn judge<R: Runner>(
        &mut self,
        session: &mut Session,
        template: &TaskTemplate,
        runner: &mut R,
        attempt_id: &str,
        assessment_id: &str,
    ) -> Result<TerminalOutcome, MachineError> {
        if self.state != MachineState::Judge {
            return Err(MachineError::BadTransition {
                from: self.state.clone(),
                what: "judge requires Judge state",
            });
        }
        let n = self.iter;
        let mut probe = Probe { inner: runner, last: None };
        let outcome = session.grade_and_admit(
            attempt_id,
            template,
            &mut probe,
            &format!("{attempt_id}-job-i{n}"),
            &format!("{attempt_id}-accept-i{n}"),
            assessment_id,
        )?;
        match outcome {
            GradeOutcome::Assessed(grade, _) => {
                self.strikes = 0;
                let probe_rec = probe.last.clone().unwrap_or(RunnerRecord {
                    job_id: format!("{attempt_id}-job-i{n}"),
                    env_digest: String::new(),
                    input_hash: String::new(),
                    test_bundle_hash: template.test_bundle_hash(),
                    exit_status: 0,
                    output_hash: String::new(),
                    results: Vec::new(),
                });
                let terminal = if grade.fully_clean {
                    TerminalOutcome::AssessedClean
                } else if grade.false_negative {
                    TerminalOutcome::Missed
                } else if grade.false_positive {
                    TerminalOutcome::FalsePositive
                } else {
                    TerminalOutcome::Caught
                };
                self.receipts.push(TransitionReceipt::post_run(
                    MachineState::Judge,
                    MachineState::Receipt,
                    self.iter,
                    &probe_rec,
                    Some(&grade),
                ));
                self.receipts.push(TransitionReceipt::pre_run(
                    MachineState::Receipt,
                    MachineState::Terminated,
                    self.iter,
                ));
                self.state = MachineState::Terminated;
                Ok(terminal)
            }
            GradeOutcome::Unresolved(job_id) => {
                self.strikes += 1;
                let rec = probe.last.clone().unwrap_or(RunnerRecord {
                    job_id: job_id.clone(),
                    env_digest: String::new(),
                    input_hash: String::new(),
                    test_bundle_hash: template.test_bundle_hash(),
                    exit_status: -1,
                    output_hash: String::new(),
                    results: Vec::new(),
                });
                if self.strikes >= STRIKE_LIMIT {
                    self.receipts.push(TransitionReceipt::post_run(
                        MachineState::Judge,
                        MachineState::Escalated,
                        self.iter,
                        &rec,
                        None,
                    ));
                    self.state = MachineState::Escalated;
                    Ok(TerminalOutcome::Escalated)
                } else {
                    self.receipts.push(TransitionReceipt::post_run(
                        MachineState::Judge,
                        MachineState::Plan,
                        self.iter,
                        &rec,
                        None,
                    ));
                    self.state = MachineState::Plan;
                    Ok(TerminalOutcome::Unresolved)
                }
            }
        }
    }

    /// Dispute path: removes evidence from the active map (Slice 2 §C),
    /// then terminates. Callable from Judge (pre-grade), Receipt, or Terminated
    /// (post-assessment dispute, Slice 2: Assessed -> Disputed).
    pub fn dispute(
        &mut self,
        session: &mut Session,
        attempt_id: &str,
    ) -> Result<TerminalOutcome, MachineError> {
        if self.state != MachineState::Judge
            && self.state != MachineState::Receipt
            && self.state != MachineState::Terminated
        {
            return Err(MachineError::BadTransition {
                from: self.state.clone(),
                what: "dispute requires Judge or Receipt state",
            });
        }
        let n = self.iter;
        session.dispute(attempt_id, &format!("{attempt_id}-dispute-i{n}"))?;
        self.receipts.push(TransitionReceipt::pre_run(
            self.state.clone(),
            MachineState::Terminated,
            self.iter,
        ));
        self.state = MachineState::Terminated;
        Ok(TerminalOutcome::Disputed)
    }
}

impl Default for AttemptMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn max_iters_bound_holds() {
        assert!(MAX_ITERS <= 8);
    }

    #[test]
    fn strike_limit_is_two() {
        assert_eq!(STRIKE_LIMIT, 2);
    }

    #[test]
    fn bad_transition_rejected() {
        let mut m = AttemptMachine::new();
        let mut s = Session::new();
        let r = m.run(&mut s, "att-x", "sub-x", b"bytes");
        assert!(matches!(
            r,
            Err(MachineError::BadTransition { .. })
        ));
    }

    #[test]
    fn iter_budget_exhausts() {
        let mut m = AttemptMachine::new();
        m.iter = MAX_ITERS;
        let mut s = Session::new();
        let t = TaskTemplate {
            template_id: "T".into(),
            concept_id: "c".into(),
            dimension: "SPEC".into(),
            difficulty: 1,
            subtlety: 1,
            variant: "v".into(),
            independent: true,
            defective_task: false,
            required_defects: Vec::new(),
            decoys: Vec::new(),
            ground_truth: "g".into(),
            task_body: "b".into(),
            test_bundle_name: "tb".into(),
            test_bundle: "bundle".into(),
            criteria: Vec::new(),
        };
        let r = m.plan(&mut s, &t, "att-x", b"pred");
        assert_eq!(r, Err(MachineError::IterBudgetExhausted));
    }
}
