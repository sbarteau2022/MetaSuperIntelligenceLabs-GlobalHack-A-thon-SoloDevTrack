use crate::{RecordId, SuperpositionHold};
use std::collections::{BTreeMap, BTreeSet};

/// Frozen, preregistered controls for one holding episode.
#[derive(Clone, Copy, Debug)]
pub struct HoldingPolicy {
    pub readiness_floor: f64,
    pub suppression_delta: f64,
}

impl HoldingPolicy {
    pub fn validate(self) -> Result<Self, String> {
        if !self.readiness_floor.is_finite() || !(0.0..=1.0).contains(&self.readiness_floor) {
            return Err("readiness_floor must be finite and in [0,1]".into());
        }
        if !self.suppression_delta.is_finite() || self.suppression_delta < 0.0 {
            return Err("suppression_delta must be finite and nonnegative".into());
        }
        Ok(self)
    }
}

/// One branch's newly measured state after a counterfactual iteration.
/// The controller accepts measurements only; receipt construction and
/// persistence remain in the caller/GlobalMindStore boundary.
#[derive(Clone, Debug)]
pub struct BranchUpdate {
    pub branch: RecordId,
    pub free_energy_value: f64,
    pub precision_weighted_error: f64,
    pub readiness: f64,
    pub compliance_permitted: bool,
    pub prediction_ready: bool,
    pub positive_progress: bool,
    pub replayable: bool,
}

impl BranchUpdate {
    fn validate(&self) -> Result<(), String> {
        if ![
            self.free_energy_value,
            self.precision_weighted_error,
            self.readiness,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return Err("branch update measurements must be finite".into());
        }
        if !(0.0..=1.0).contains(&self.readiness) {
            return Err("branch update readiness must be in [0,1]".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HoldingDecision {
    Collapse {
        branch: RecordId,
        surviving: Vec<RecordId>,
    },
    Iterate {
        surviving: Vec<RecordId>,
        next_iteration: u32,
    },
    FailClosed {
        reason: &'static str,
    },
}

#[derive(Clone, Debug)]
pub struct HoldingSnapshot {
    pub hold: RecordId,
    pub completed_iterations: u32,
    pub active_branches: Vec<RecordId>,
    pub terminal: bool,
}

/// Fresh GlobalMind superposition controller.
///
/// It contains no token/output API. During a hold the only legal state change
/// is `advance`, which returns a persistence-ready decision. Every candidate
/// is filtered and suppressed using the same fields and the same policy.
pub struct HoldingController {
    hold: RecordId,
    iteration_limit: u32,
    completed_iterations: u32,
    active: BTreeSet<RecordId>,
    policy: HoldingPolicy,
    terminal: bool,
}

impl HoldingController {
    pub fn open(hold: &SuperpositionHold, policy: HoldingPolicy) -> Result<Self, String> {
        hold.validate()?;
        let policy = policy.validate()?;
        let active = hold
            .branches
            .iter()
            .map(|branch| branch.id.clone())
            .collect::<BTreeSet<_>>();
        if active.len() != hold.branches.len() {
            return Err("superposition branch ids must be unique".into());
        }
        Ok(Self {
            hold: hold.id.clone(),
            iteration_limit: hold.iteration_limit,
            completed_iterations: 0,
            active,
            policy,
            terminal: false,
        })
    }

    pub fn snapshot(&self) -> HoldingSnapshot {
        HoldingSnapshot {
            hold: self.hold.clone(),
            completed_iterations: self.completed_iterations,
            active_branches: self.active.iter().cloned().collect(),
            terminal: self.terminal,
        }
    }

    pub fn advance(&mut self, updates: Vec<BranchUpdate>) -> Result<HoldingDecision, String> {
        if self.terminal {
            return Err("superposition hold is already terminal".into());
        }
        if updates.len() != self.active.len() {
            return Err("every active branch must have exactly one update".into());
        }
        let mut measured = BTreeMap::new();
        for update in updates {
            update.validate()?;
            if !self.active.contains(&update.branch) {
                return Err("branch update does not belong to the active hold".into());
            }
            let id = update.branch.clone();
            if measured.insert(id, update).is_some() {
                return Err("duplicate branch update".into());
            }
        }

        self.completed_iterations = self
            .completed_iterations
            .checked_add(1)
            .ok_or_else(|| "superposition iteration counter overflow".to_string())?;

        let permitted = measured
            .values()
            .filter(|update| update.compliance_permitted)
            .collect::<Vec<_>>();
        if permitted.is_empty() {
            self.active.clear();
            self.terminal = true;
            return Ok(HoldingDecision::FailClosed {
                reason: "no compliant branch",
            });
        }

        let minimum = permitted
            .iter()
            .map(|update| update.free_energy_value)
            .min_by(f64::total_cmp)
            .ok_or_else(|| "no measured branch".to_string())?;
        let threshold = minimum + self.policy.suppression_delta;
        if !threshold.is_finite() {
            self.terminal = true;
            return Ok(HoldingDecision::FailClosed {
                reason: "suppression threshold overflow",
            });
        }

        let mut survivors = permitted
            .into_iter()
            .filter(|update| update.free_energy_value <= threshold)
            .collect::<Vec<_>>();
        survivors.sort_by(|left, right| {
            left.free_energy_value
                .total_cmp(&right.free_energy_value)
                .then_with(|| left.branch.cmp(&right.branch))
        });
        self.active = survivors
            .iter()
            .map(|update| update.branch.clone())
            .collect();
        let survivor_ids = self.active.iter().cloned().collect::<Vec<_>>();

        if let Some(winner) = survivors.first()
            && winner.prediction_ready
            && winner.positive_progress
            && winner.replayable
            && winner.readiness >= self.policy.readiness_floor
        {
            self.terminal = true;
            return Ok(HoldingDecision::Collapse {
                branch: winner.branch.clone(),
                surviving: survivor_ids,
            });
        }

        if self.completed_iterations >= self.iteration_limit {
            self.terminal = true;
            return Ok(HoldingDecision::FailClosed {
                reason: "iteration limit reached without a replayable ready branch",
            });
        }

        Ok(HoldingDecision::Iterate {
            surviving: survivor_ids,
            next_iteration: self.completed_iterations,
        })
    }
}
