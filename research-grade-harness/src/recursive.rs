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

pub fn mandate_iteration(
    packet_id: &str,
    bound: &SessionBound,
    has_answer: bool,
) -> Option<IterationMandate> {
    let spike = bound.requires_iteration(2.0);
    match crate::gate::dynamic_iteration_directive(packet_id, spike, has_answer) {
        Some(reason) => Some(IterationMandate {
            packet_id: packet_id.into(),
            reason,
            bound_at_trigger: bound.total(),
            plan_required: true,
        }),
        None => None,
    }
}
