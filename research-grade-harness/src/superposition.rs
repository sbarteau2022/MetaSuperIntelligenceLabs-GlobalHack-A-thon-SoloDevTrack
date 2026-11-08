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
    let mut state = HoldingState {
        branches: branches.clone(),
        ..HoldingState::default()
    };
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

/// Result of folding math-kernel receipts into the held branches.
#[derive(Clone, Debug, PartialEq)]
pub struct KernelAgreement {
    /// The output_hash confirmed by >=2 independent routes, if any.
    pub verified_hash: Option<String>,
    /// How many routes agreed on that hash.
    pub multiplicity: usize,
}

/// Fold math-kernel receipts into held branches as the EXTERNAL verifier.
///
/// The meta-automaton does not re-derive; it compares `output_hash`es across
/// independent computational routes. A branch whose route is CONFIRMED (its
/// hash is produced by >=2 independent branches) has its inaccuracy driven
/// down by `confirm_bonus` (externally verified ⇒ unsurprising) and becomes
/// `prediction_ready`. A branch with no receipt, a Refused receipt, or a
/// singleton hash gains `unverified_penalty` nats (can't be confirmed ⇒
/// surprising) and stays not-ready.
///
/// Verification acts ONLY through the free-energy accuracy term — so the same
/// functional scores everything (H1), and pruning still sees scalar F alone;
/// the hash's SOURCE is never consulted, only agreement multiplicity (H2).
/// An unverified winner therefore fails the `prediction_ready` gate in
/// `deliberate` and mandates iteration (H3).
pub fn fold_kernel_receipts(
    branches: &mut [DeliberationBranch],
    receipts: &[(usize, Option<String>)],
    confirm_bonus: f64,
    unverified_penalty: f64,
) -> KernelAgreement {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, h) in receipts {
        if let Some(h) = h {
            *counts.entry(h.clone()).or_insert(0) += 1;
        }
    }
    let verified = counts
        .iter()
        .filter(|(_, c)| **c >= 2)
        .max_by_key(|(_, c)| **c)
        .map(|(h, c)| (h.clone(), *c));

    for b in branches.iter_mut() {
        let rh = receipts
            .iter()
            .find(|(id, _)| *id == b.branch_id)
            .and_then(|(_, h)| h.clone());
        let confirmed = rh
            .as_ref()
            .map_or(false, |h| counts.get(h).copied().unwrap_or(0) >= 2);
        b.prediction_hash = rh.unwrap_or_default();
        if confirmed {
            b.terms.inaccuracy_nats = (b.terms.inaccuracy_nats - confirm_bonus).max(0.0);
            b.prediction_ready = true;
        } else {
            b.terms.inaccuracy_nats += unverified_penalty;
            b.prediction_ready = false;
        }
    }

    match verified {
        Some((h, c)) => KernelAgreement { verified_hash: Some(h), multiplicity: c },
        None => KernelAgreement { verified_hash: None, multiplicity: 0 },
    }
}

#[cfg(test)]
mod kernel_verify_tests {
    use super::*;
    use crate::fep::FreeEnergyTerms;

    fn br(id: usize, src: BranchSource, inacc: f64) -> DeliberationBranch {
        DeliberationBranch {
            branch_id: id,
            source: src,
            terms: FreeEnergyTerms::new(0.5, inacc, 0.0),
            prediction_hash: String::new(),
            prediction_ready: false,
        }
    }

    #[test]
    fn two_routes_agree_confirms_and_readies() {
        // Branches 1 & 2 computed the SAME hash via different routes; 3 diverged.
        let mut bs = vec![
            br(1, BranchSource::FreshDerivation, 3.0),
            br(2, BranchSource::NeighborAtom, 3.0),
            br(3, BranchSource::CrossBasinBridge, 3.0),
        ];
        let receipts = vec![
            (1, Some("HASH_A".to_string())),
            (2, Some("HASH_A".to_string())),
            (3, Some("HASH_B".to_string())),
        ];
        let agree = fold_kernel_receipts(&mut bs, &receipts, 3.0, 5.0);
        assert_eq!(agree.verified_hash.as_deref(), Some("HASH_A"));
        assert_eq!(agree.multiplicity, 2);
        // confirmed routes: ready, inaccuracy driven to ~0 (lower F ⇒ they win)
        assert!(bs[0].prediction_ready && bs[1].prediction_ready);
        assert!(bs[0].terms.inaccuracy_nats < 0.001);
        // diverged singleton: not ready, penalized (higher F ⇒ pruned)
        assert!(!bs[2].prediction_ready);
        assert!(bs[2].terms.inaccuracy_nats > 3.0);
        // the verified routes carry the lowest free energy
        assert!(bs[0].bound() < bs[2].bound());
    }

    #[test]
    fn no_agreement_forces_iteration_path() {
        // All distinct hashes ⇒ nothing confirmed ⇒ no branch prediction_ready.
        let mut bs = vec![
            br(1, BranchSource::FreshDerivation, 1.0),
            br(2, BranchSource::NeighborAtom, 1.0),
        ];
        let receipts = vec![(1, Some("X".into())), (2, Some("Y".into()))];
        let agree = fold_kernel_receipts(&mut bs, &receipts, 3.0, 5.0);
        assert!(agree.verified_hash.is_none());
        assert!(bs.iter().all(|b| !b.prediction_ready));
    }
}
