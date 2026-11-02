//! Lane C wiring: reasoning-engine signals -> HoldingController -> store.
//!
//! The controller stays pure (measurements in, decision out). This module is
//! the only bridge: it classifies trigger signals, drives the controller one
//! iteration at a time, persists every iteration with zero external
//! emissions, and persists exactly one terminal resolution. A collapsed
//! branch never writes memory here; its synthesized bridge is returned as a
//! [`BridgeRoute`] so the caller routes it through the normal packet, judge,
//! replay, and crystallization path. There is deliberately no direct
//! atom/trajectory write in this module.

use crate::{
    BranchSource, BranchUpdate, Digest, GlobalMindStore, HoldingController, HoldingDecision,
    HoldingPolicy, RecordId, ResolutionOutcome, SuperpositionBranch, SuperpositionHold,
    SuperpositionIteration, SuperpositionResolution, SuperpositionTrigger,
};
use sha2::{Digest as ShaDigest, Sha256};

pub const CONTROLLER_VERSION: &str = "globalmind-holding-v1";

/// Reasoning-engine signals observed at one recall/routing step. Every field
/// is a measurement; classification into a hold trigger is deterministic and
/// ordered by [`classify_trigger`].
#[derive(Clone, Copy, Debug)]
pub struct RecallSignals {
    pub direct_trajectory_found: bool,
    pub cycle_detected: bool,
    pub ambiguous_branches: bool,
    pub precision_weighted_error: f64,
    pub weighted_error_threshold: f64,
    pub readiness: f64,
    pub readiness_floor: f64,
    pub compliance_boundary: bool,
}

/// Gate 1: map real recall/routing signals to a hold trigger. Returns `None`
/// when no signal fires, in which case no hold may be opened. Compliance and
/// cycle signals outrank quality signals because they are safety-relevant.
pub fn classify_trigger(signals: &RecallSignals) -> Result<Option<SuperpositionTrigger>, String> {
    if !signals.precision_weighted_error.is_finite()
        || !signals.weighted_error_threshold.is_finite()
        || !signals.readiness.is_finite()
        || !signals.readiness_floor.is_finite()
    {
        return Err("recall signals must be finite".into());
    }
    if signals.weighted_error_threshold < 0.0
        || !(0.0..=1.0).contains(&signals.readiness)
        || !(0.0..=1.0).contains(&signals.readiness_floor)
    {
        return Err("recall signal thresholds are invalid".into());
    }
    Ok(if signals.compliance_boundary {
        Some(SuperpositionTrigger::ComplianceBoundary)
    } else if signals.cycle_detected {
        Some(SuperpositionTrigger::FixedPointOrCycle)
    } else if !signals.direct_trajectory_found {
        Some(SuperpositionTrigger::NoDirectTrajectory)
    } else if signals.ambiguous_branches {
        Some(SuperpositionTrigger::BranchAmbiguity)
    } else if signals.precision_weighted_error > signals.weighted_error_threshold {
        Some(SuperpositionTrigger::HighPredictionError)
    } else if signals.readiness < signals.readiness_floor {
        Some(SuperpositionTrigger::LowReadiness)
    } else {
        None
    })
}

/// Gate 7 routing instruction for a collapsed branch. The bridge candidate
/// must enter the normal packet admission path; the session never writes it
/// to memory directly.
#[derive(Clone, Debug)]
pub struct BridgeRoute {
    pub branch: RecordId,
    pub from_atom: RecordId,
    pub candidate_atom: RecordId,
    pub source: BranchSource,
    pub bridge_digest: Digest,
    pub required_path: &'static str,
}

#[derive(Clone, Debug)]
pub enum SessionOutcome {
    Collapsed {
        branch: RecordId,
        bridge: BridgeRoute,
    },
    Iterate {
        surviving: Vec<RecordId>,
        next_iteration: u32,
    },
    FailClosed {
        reason: &'static str,
    },
}

/// One persisted holding episode: controller state plus its store receipts.
pub struct HoldingSession<'a> {
    store: &'a GlobalMindStore,
    controller: HoldingController,
    hold: SuperpositionHold,
    next_ordinal: u32,
    terminal: bool,
}

impl<'a> HoldingSession<'a> {
    /// Opens the controller and persists the hold with all branches (gate 6).
    pub fn open(
        store: &'a GlobalMindStore,
        hold: SuperpositionHold,
        policy: HoldingPolicy,
    ) -> Result<Self, String> {
        let controller = HoldingController::open(&hold, policy)?;
        store.open_superposition(&hold)?;
        Ok(Self {
            store,
            controller,
            hold,
            next_ordinal: 0,
            terminal: false,
        })
    }

    pub fn hold_id(&self) -> &RecordId {
        &self.hold.id
    }

    /// Runs one zero-emission iteration (gate 2): the controller requires
    /// exactly one homogeneous update per active branch (gate 3), suppression
    /// reads only free energy and compliance so source identity cannot
    /// influence it (gate 4), and collapse requires the full gate-5
    /// conjunction inside the controller. The iteration and, on a terminal
    /// decision, the resolution are persisted before returning (gate 6).
    pub fn advance(&mut self, updates: Vec<BranchUpdate>) -> Result<SessionOutcome, String> {
        if self.terminal {
            return Err("holding session is already terminal".into());
        }
        let active_before = self.controller.snapshot().active_branches;
        let input_branch_digest = digest_ids("globalmind-hold-input-v1", &active_before);
        let decision = self.controller.advance(updates)?;
        let ordinal = self.next_ordinal;
        let (surviving, outcome) = match &decision {
            HoldingDecision::Collapse { branch, surviving } => (
                surviving.clone(),
                SessionOutcome::Collapsed {
                    branch: branch.clone(),
                    bridge: self.bridge_route(branch)?,
                },
            ),
            HoldingDecision::Iterate {
                surviving,
                next_iteration,
            } => (
                surviving.clone(),
                SessionOutcome::Iterate {
                    surviving: surviving.clone(),
                    next_iteration: *next_iteration,
                },
            ),
            HoldingDecision::FailClosed { reason } => (
                self.controller.snapshot().active_branches,
                SessionOutcome::FailClosed { reason },
            ),
        };
        let output_branch_digest = digest_ids("globalmind-hold-output-v1", &surviving);
        let iteration = SuperpositionIteration {
            id: RecordId::parse(format!(
                "holditer_{}_{ordinal}",
                &self.hold.id.as_str()[..self.hold.id.as_str().len().min(24)]
            ))?,
            workspace: self.hold.workspace.clone(),
            hold: self.hold.id.clone(),
            ordinal,
            input_branch_digest: input_branch_digest.clone(),
            surviving_branches: surviving,
            rollout_receipt_digest: digest_fields(
                b"globalmind-hold-rollout-v1",
                &[
                    self.hold.id.as_str(),
                    &ordinal.to_string(),
                    input_branch_digest.as_str(),
                ],
            ),
            loop_receipt_digest: None,
            suppression_receipt_digest: digest_fields(
                b"globalmind-hold-suppression-v1",
                &[
                    self.hold.id.as_str(),
                    &ordinal.to_string(),
                    output_branch_digest.as_str(),
                ],
            ),
            output_branch_digest,
        };
        // The store writes external_emissions: 0 for every iteration; the
        // schema asserts that value, so a nonzero emission cannot persist.
        self.store.record_superposition_iteration(&iteration)?;
        self.next_ordinal += 1;

        match &outcome {
            SessionOutcome::Collapsed { branch, bridge } => {
                self.persist_resolution(ResolutionOutcome::Collapsed, Some(branch.clone()))?;
                self.terminal = true;
                Ok(SessionOutcome::Collapsed {
                    branch: branch.clone(),
                    bridge: bridge.clone(),
                })
            }
            SessionOutcome::FailClosed { reason } => {
                self.persist_resolution(ResolutionOutcome::FailClosed, None)?;
                self.terminal = true;
                Ok(SessionOutcome::FailClosed { reason })
            }
            SessionOutcome::Iterate { .. } => Ok(outcome),
        }
    }

    fn bridge_route(&self, branch_id: &RecordId) -> Result<BridgeRoute, String> {
        let branch = self
            .hold
            .branches
            .iter()
            .find(|branch| &branch.id == branch_id)
            .ok_or_else(|| "collapsed branch is not part of the hold".to_string())?;
        Ok(BridgeRoute {
            branch: branch.id.clone(),
            from_atom: branch.from_atom.clone(),
            candidate_atom: branch.candidate_atom.clone(),
            source: branch.source,
            bridge_digest: digest_fields(
                b"globalmind-bridge-v1",
                &[
                    branch.id.as_str(),
                    branch.from_atom.as_str(),
                    branch.candidate_atom.as_str(),
                    branch.prediction_digest.as_str(),
                ],
            ),
            required_path: "packet_admission>judge>replay>crystallization",
        })
    }

    fn persist_resolution(
        &self,
        outcome: ResolutionOutcome,
        selected_branch: Option<RecordId>,
    ) -> Result<(), String> {
        let resolution_receipt = digest_fields(
            b"globalmind-hold-resolution-v1",
            &[
                self.hold.id.as_str(),
                outcome_name(outcome),
                selected_branch.as_ref().map_or("", RecordId::as_str),
                &self.next_ordinal.to_string(),
            ],
        );
        self.store.resolve_superposition(&SuperpositionResolution {
            id: RecordId::parse(format!("holdres_{}", &resolution_receipt.as_str()[..32]))?,
            workspace: self.hold.workspace.clone(),
            hold: self.hold.id.clone(),
            outcome,
            selected_branch,
            // Gate 7: no resulting trajectory/atom is claimed here. Those
            // records may only exist after the bridge passes the normal
            // packet/judge/replay/crystallization path elsewhere.
            resulting_trajectory: None,
            resulting_atom: None,
            judge_receipt_digest: digest_fields(
                b"globalmind-hold-judge-v1",
                &[self.hold.id.as_str(), resolution_receipt.as_str()],
            ),
            replay_receipt_digest: None,
            crystallization_receipt_digest: None,
            resolution_receipt_digest: resolution_receipt,
        })
    }
}

fn outcome_name(outcome: ResolutionOutcome) -> &'static str {
    match outcome {
        ResolutionOutcome::Collapsed => "collapsed",
        ResolutionOutcome::Iterate => "iterate",
        ResolutionOutcome::FailClosed => "fail_closed",
    }
}

fn digest_ids(domain: &str, ids: &[RecordId]) -> Digest {
    let fields: Vec<&str> = ids.iter().map(RecordId::as_str).collect();
    digest_fields(domain.as_bytes(), &fields)
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}

/// Controller digest binding the persisted hold to this exact controller.
pub fn controller_digest() -> Digest {
    digest_fields(b"globalmind-holding-controller-v1", &[CONTROLLER_VERSION])
}

/// Builds one branch record. The prediction preimage includes the hold id:
/// branch identity is hold-scoped, so identical measurements in two
/// episodes can never collide on record identity or uniqueness indexes.
#[allow(clippy::too_many_arguments)]
pub fn branch(
    hold: &RecordId,
    ordinal: u8,
    source: BranchSource,
    from_atom: &str,
    candidate_atom: &str,
    free_energy_value: f64,
    precision_weighted_error: f64,
    readiness: f64,
    compliance_permitted: bool,
) -> Result<SuperpositionBranch, String> {
    let prediction_digest = digest_fields(
        b"globalmind-branch-prediction-v1",
        &[
            hold.as_str(),
            &ordinal.to_string(),
            from_atom,
            candidate_atom,
            &format!("{free_energy_value:?}"),
        ],
    );
    Ok(SuperpositionBranch {
        id: RecordId::parse(format!("branch_{}", &prediction_digest.as_str()[..32]))?,
        ordinal,
        source,
        source_trajectory: None,
        from_atom: RecordId::parse(from_atom)?,
        candidate_atom: RecordId::parse(candidate_atom)?,
        prediction_digest: prediction_digest.clone(),
        scorer_version: CONTROLLER_VERSION.to_string(),
        scorer_digest: controller_digest(),
        free_energy_value,
        precision_weighted_error,
        readiness,
        compliance_permitted,
        score_receipt_digest: digest_fields(
            b"globalmind-branch-score-v1",
            &[prediction_digest.as_str()],
        ),
        compliance_receipt_digest: digest_fields(
            b"globalmind-branch-compliance-v1",
            &[
                prediction_digest.as_str(),
                if compliance_permitted { "1" } else { "0" },
            ],
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals() -> RecallSignals {
        RecallSignals {
            direct_trajectory_found: true,
            cycle_detected: false,
            ambiguous_branches: false,
            precision_weighted_error: 0.1,
            weighted_error_threshold: 0.5,
            readiness: 0.9,
            readiness_floor: 0.5,
            compliance_boundary: false,
        }
    }

    #[test]
    fn every_trigger_class_fires_and_clean_signals_do_not() {
        assert_eq!(classify_trigger(&signals()).unwrap(), None);
        let cases = [
            (
                RecallSignals {
                    compliance_boundary: true,
                    ..signals()
                },
                SuperpositionTrigger::ComplianceBoundary,
            ),
            (
                RecallSignals {
                    cycle_detected: true,
                    ..signals()
                },
                SuperpositionTrigger::FixedPointOrCycle,
            ),
            (
                RecallSignals {
                    direct_trajectory_found: false,
                    ..signals()
                },
                SuperpositionTrigger::NoDirectTrajectory,
            ),
            (
                RecallSignals {
                    ambiguous_branches: true,
                    ..signals()
                },
                SuperpositionTrigger::BranchAmbiguity,
            ),
            (
                RecallSignals {
                    precision_weighted_error: 0.9,
                    ..signals()
                },
                SuperpositionTrigger::HighPredictionError,
            ),
            (
                RecallSignals {
                    readiness: 0.2,
                    ..signals()
                },
                SuperpositionTrigger::LowReadiness,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(classify_trigger(&input).unwrap(), Some(expected));
        }
    }

    #[test]
    fn invalid_signals_fail_closed() {
        assert!(
            classify_trigger(&RecallSignals {
                precision_weighted_error: f64::NAN,
                ..signals()
            })
            .is_err()
        );
        assert!(
            classify_trigger(&RecallSignals {
                readiness: 1.5,
                ..signals()
            })
            .is_err()
        );
    }
}
