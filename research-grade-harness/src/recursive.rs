//! Dynamic self-recursive optimization: the system improves ITSELF by
//! descending the FreeEnergy bound across EVERY subsystem — never one
//! cherry-picked lane. Each cycle: measure every subsystem's bound
//! contribution, pick the worst, improve it, re-measure. When a problem
//! has no cached answer, the system MUST iterate (plan/research/execute)
//! rather than retry or escalate blindly.
//!
//! "Self-recursive" because the optimizer's own decisions are session-logged
//! and hunger-tapped: the optimizer watches itself, and its failures become
//! hunger gaps like any other.

use crate::fep::SessionBound;

/// One subsystem's measured contribution to the session bound.
#[derive(Clone, Debug)]
pub struct SubsystemScore {
    pub name: &'static str,
    pub bound: f64,
    pub improvable: bool,
}

/// Whole-system scan: every subsystem reports, worst-first. No lane is
/// exempt, none is favored — the bound decides.
/// Scheduler that closes the mandate→execution gap (G5).
/// Counts mandates issued vs optimize_cycle calls; `gap()` reports
/// mandates that died unexecuted. A run with a pending mandate and no
/// `run_pending` call leaves gap() > 0 — caught by tests/probe.
#[derive(Clone, Debug, Default)]
pub struct OptimizerScheduler {
    mandates_issued: u64,
    cycles_executed: u64,
    pending: Option<String>,
}

impl OptimizerScheduler {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_mandate(&mut self, m: &IterationMandate) {
        self.mandates_issued += 1;
        self.pending = Some(m.packet_id.clone());
    }
    pub fn run_pending(
        &mut self,
        bound: &mut SessionBound,
        scores: &[SubsystemScore],
        improve: &dyn Fn(&str) -> crate::fep::FreeEnergyTerms,
        log: &mut crate::session::SessionLog,
        workflow_id: &str,
        timestamp_ms: u64,
    ) -> Option<(String, f64)> {
        self.pending.as_ref()?;
        let out = optimize_cycle(bound, scores, improve, log, workflow_id, timestamp_ms);
        if out.is_some() {
            self.cycles_executed += 1;
            self.pending = None;
        }
        out
    }
    pub fn gap(&self) -> u64 {
        self.mandates_issued.saturating_sub(self.cycles_executed)
    }
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
}

pub fn rank_subsystems(scores: &[SubsystemScore]) -> Vec<&SubsystemScore> {
    let mut out: Vec<&SubsystemScore> = scores.iter().filter(|s| s.improvable).collect();
    out.sort_by(|a, b| {
        b.bound
            .partial_cmp(&a.bound)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Optimization cycle: measure all, improve the worst, log the decision.
/// Returns the improved subsystem name + the new session total.
///
/// TECH DEBT (MATH-INVARIANTS.md Invariant 3): no scheduler forces the
/// cadence — the mandate exists, the loop-closer doesn't. The live map
/// closes its loop (fail → hunger → consolidate → sated, proven); this
/// optimizer loop is specified but unscheduled. Named, not hand-waved.
/// Improvement is a caller-supplied step (the harness implements per
/// subsystem: reweight, prune, re-plan, consolidate...); the optimizer
/// only decides WHERE, never HOW — separation of concerns.
pub fn optimize_cycle(
    bound: &mut SessionBound,
    scores: &[SubsystemScore],
    improve: &dyn Fn(&str) -> crate::fep::FreeEnergyTerms,
    log: &mut crate::session::SessionLog,
    workflow_id: &str,
    timestamp_ms: u64,
) -> Option<(String, f64)> {
    let ranked = rank_subsystems(scores);
    let worst = ranked.first()?;
    let terms = improve(worst.name);
    let total = bound.descend(terms);
    log.append(
        workflow_id,
        crate::session::SessionEventKind::Checkpoint,
        &format!("recursive-opt: improved {} total={:.4}", worst.name, total),
        timestamp_ms,
    );
    Some((worst.name.to_string(), total))
}

/// The iteration mandate: called when the engine encounters a problem with
/// no cached answer. It MUST return a plan (never a retry, never a blind
/// escalate). The plan goes through research_gate before execution.
#[derive(Clone, Debug)]
pub struct IterationMandate {
    pub packet_id: String,
    pub reason: String,
    pub bound_at_trigger: f64,
    pub plan_required: bool,
}

/// Calibrated spike sensitivity (G4).
/// Source: probe over 5000 normal-only sessions (false-positive rate)
/// and 5000 sessions with a +1.0 bound drift spike (detection rate).
///   sigma=1.5 -> FPR 7.44%  / DET 100%
///   sigma=2.0 -> FPR 0.92%  / DET 100%   <- chosen (best FPR at 100% detection)
///   sigma=2.5 -> FPR 0.00%  / DET 0%     (kills sensitivity: spike inflates std)
/// Values beyond 2.0 collapse detection because the spike itself inflates
/// the running stddev; 2.0 is the knee. Re-run the probe if the bound
/// distribution changes.
pub const SPIKE_SIGMA: f64 = 2.0;

pub fn mandate_iteration(
    packet_id: &str,
    bound: &SessionBound,
    has_answer: bool,
) -> Option<IterationMandate> {
    let spike = bound.requires_iteration(SPIKE_SIGMA);
    crate::gate::dynamic_iteration_directive(packet_id, spike, has_answer).map(|reason| {
        IterationMandate {
            packet_id: packet_id.into(),
            reason,
            bound_at_trigger: bound.total(),
            plan_required: true,
        }
    })
}
