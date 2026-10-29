//! Superposition holding: deliberation AFTER task-decomp, BEFORE orchestration.
//!
//! Task-decomp emits leaves with information needs. Orchestration executes
//! them. THIS module governs the deliberation phase in between: hold
//! parallel potentials, give each honest consideration, pressure-test each
//! against the others, and let the prediction operation fire only when it
//! has enough data — so dynamic iteration minimizes prediction error.
//!
//! Three invariants govern (executable, not aspirational):
//! 1. HOMOGENEITY: every potential scored by the SAME free-energy functional.
//!    Same scorer, same arithmetic, same compliance gate. No branch gets
//!    different physics. Enforced structurally (one `score()` fn).
//! 2. ISOTROPIC SUPPRESSION: pruning acts on scalar total-F only, never on
//!    branch source, direction, or identity. A test proves no source-bias.
//! 3. DYNAMIC ITERATION: when no branch verifies AND no cached answer
//!    exists, the system MUST iterate (plan/research/execute) — never
//!    retry, never escalate blindly, never stall silently.
//!
//! I-RECALL lives here as the pre-hold check: before harvesting branches,
//! query the KV cache for covering conclusions. Hit → depth-0 recall path,
//! receipt says `recalled`. The hold only happens on recall miss. Checking
//! what is known before deliberating is what makes deliberation affordable.

use crate::fep::{FreeEnergyTerms, SessionBound};
use crate::gate::dynamic_iteration_directive;

/// Where a deliberation branch was harvested from. Mirrors the vault's
/// four geometric sources, adapted to our Atom/Basin substrate.
#[derive(Clone, Debug, PartialEq)]
pub enum BranchSource {
    /// Direct: existing KV conclusion / carved arc covering the need.
    DirectRecall,
    /// Neighbor: nearby atoms by shell distance (phi-winding analogue).
    NeighborAtom,
    /// Bridge: cross-basin geodesic (basin centroid hop).
    CrossBasinBridge,
    /// Constructed: fresh derivation (the expensive path, last resort).
    FreshDerivation,
}

/// One held potential: close enough to compare honestly, distinct enough
/// to distinguish. Separation is enforced at harvest (see below).
#[derive(Clone, Debug)]
pub struct DeliberationBranch {
    pub branch_id: usize,
    pub source: BranchSource,
    /// Free-energy terms for THIS branch (same functional for all — H1).
    pub terms: FreeEnergyTerms,
    /// Prediction: what this branch claims the answer is (hash for tests).
    pub prediction_hash: String,
    /// Whether a prediction operation has enough data to fire reliably.
    pub prediction_ready: bool,
}

impl DeliberationBranch {
    pub fn bound(&self) -> f64 {
        self.terms.bound()
    }
}

/// The holding state: parallel potentials under consideration.
#[derive(Clone, Debug, Default)]
pub struct HoldingState {
    pub branches: Vec<DeliberationBranch>,
    pub iteration_traces: Vec<String>,
}

/// Separation bounds: potentials must be within `max_spread` (close enough
/// to compare — same information need) but at least `min_separation` apart
/// (distinguishable — not duplicates of each other).
pub struct SeparationBounds {
    pub max_spread: f64,
    pub min_separation: f64,
}

impl Default for SeparationBounds {
    fn default() -> Self {
        Self {
            max_spread: 2.0,
            min_separation: 1e-3,
        }
    }
}

/// Harvest check: reject candidates outside the separation bounds.
/// Too far = different question (not a potential for THIS need).
/// Too close = duplicate (coalesce, don't double-count).
pub fn check_separation(
    existing: &[DeliberationBranch],
    candidate_bound: f64,
    bounds: &SeparationBounds,
) -> bool {
    if existing.is_empty() {
        return true;
    }
    let closest = existing
        .iter()
        .map(|b| (b.bound() - candidate_bound).abs())
        .fold(f64::INFINITY, f64::min);
    let farthest = existing
        .iter()
        .map(|b| (b.bound() - candidate_bound).abs())
        .fold(0.0f64, f64::max);
    closest >= bounds.min_separation && farthest <= bounds.max_spread
}

/// I-RECALL pre-hold check: query cached conclusions for coverage BEFORE
/// harvesting branches. Returns the recalling conclusion if one covers
/// the need at depth 0. The hold only happens on recall miss.
///
/// `cached` = (key, bound_cost_to_recall, covers_need) triples from KV.
/// A conclusion covers the need iff its terms subsume the required terms.
pub fn irecall_check(
    required_terms: &[&str],
    cached: &[(&str, f64, Vec<&str>)],
) -> Option<(String, f64)> {
    cached
        .iter()
        .filter(|(_, _, terms)| required_terms.iter().all(|t| terms.iter().any(|c| c == t)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(key, cost, _)| (key.to_string(), *cost))
}

/// Isotropic suppression: prune to branches within `F_min + delta`.
/// Acts on scalar total-F ONLY — source is never consulted (H2).
/// Returns surviving branch ids.
pub fn suppress_isotropic(branches: &[DeliberationBranch], delta: f64) -> Vec<usize> {
    let fmin = branches
        .iter()
        .map(|b| b.bound())
        .fold(f64::INFINITY, f64::min);
    branches
        .iter()
        .filter(|b| b.bound() <= fmin + delta)
        .map(|b| b.branch_id)
        .collect()
}

/// Boltzmann weights over surviving branches: w_k = exp(-F_k) / Z.
/// Same temperature for all — homogeneity extends to the collapse.
pub fn boltzmann_weights(branches: &[DeliberationBranch]) -> Vec<(usize, f64)> {
    let exps: Vec<f64> = branches.iter().map(|b| (-b.bound()).exp()).collect();
    let z: f64 = exps.iter().sum::<f64>().max(1e-300);
    branches
        .iter()
        .zip(exps)
        .map(|(b, e)| (b.branch_id, e / z))
        .collect()
}

/// Deliberation outcome: collapse onto the verified minimum, or mandate
/// iteration when nothing verifies AND nothing was recalled.
#[derive(Clone, Debug)]
pub enum DeliberationOutcome {
    Recalled { key: String, recall_cost: f64 },
    Collapsed { branch_id: usize, bound: f64 },
    Iterate { reason: String },
}

/// Full deliberation cycle: I-RECALL first, then hold, suppress, collapse.
/// `has_answer` = whether ANY branch (post-suppression) verified.
/// Mirrors the vault's hold_and_iterate resolution shape, adapted.
pub fn deliberate(
    required_terms: &[&str],
    cached: &[(&str, f64, Vec<&str>)],
    mut branches: Vec<DeliberationBranch>,
    delta: f64,
    bound: &mut SessionBound,
    timestamp_note: &str,
) -> (DeliberationOutcome, HoldingState) {
    // Step 0 — I-RECALL: check what's known before deliberating.
    if let Some((key, cost)) = irecall_check(required_terms, cached) {
        bound.descend(FreeEnergyTerms::new(0.0, 0.0, cost));
        return (
            DeliberationOutcome::Recalled {
                key,
                recall_cost: cost,
            },
            HoldingState::default(),
        );
    }

    // Step 1 — HOLD: separation-enforced harvest already done by caller;
    // record the hold in the state.
    let mut state = HoldingState::default();
    state.branches = branches.clone();
    state.iteration_traces.push(format!(
        "hold: {} branches ({})",
        branches.len(),
        timestamp_note
    ));

    // Step 2 — ISOTROPIC SUPPRESSION: prune on scalar F only.
    let survivors = suppress_isotropic(&branches, delta);
    branches.retain(|b| survivors.contains(&b.branch_id));
    state.branches = branches.clone();
    state.iteration_traces.push(format!(
        "suppress: {}/{} survive Δ={}",
        survivors.len(),
        state.branches.len(),
        delta
    ));

    // Step 3 — COLLAPSE or ITERATE.
    if branches.is_empty() {
        let reason = dynamic_iteration_directive("deliberation", true, false)
            .unwrap_or_else(|| "no branch survived suppression; iterate".into());
        return (DeliberationOutcome::Iterate { reason }, state);
    }
    branches.sort_by(|a, b| {
        a.bound()
            .partial_cmp(&b.bound())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let winner = &branches[0];
    // Prediction fires only with enough data: winner must be prediction-ready.
    if !winner.prediction_ready {
        let reason = dynamic_iteration_directive("deliberation", true, false)
            .unwrap_or_else(|| "winner not prediction-ready; iterate for more data".into());
        return (DeliberationOutcome::Iterate { reason }, state);
    }
    bound.descend(FreeEnergyTerms::new(
        winner.terms.complexity_nats,
        winner.terms.inaccuracy_nats,
        winner.terms.dissipation_j,
    ));
    (
        DeliberationOutcome::Collapsed {
            branch_id: winner.branch_id,
            bound: winner.bound(),
        },
        state,
    )
}
