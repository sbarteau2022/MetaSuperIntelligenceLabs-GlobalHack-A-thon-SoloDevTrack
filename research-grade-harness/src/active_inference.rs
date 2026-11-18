#[derive(Clone, Debug)]
pub struct BeliefState {
    pub task_confidence: f64, // 0..1
    pub precision: f64,       // higher = more deterministic
    pub observations: u32,
}

impl BeliefState {
    pub fn new() -> Self {
        Self {
            task_confidence: 0.5,
            precision: 1.0,
            observations: 0,
        }
    }
    pub fn update_success(&mut self, weight: f64) {
        self.task_confidence = (self.task_confidence + weight).min(1.0);
        self.precision = (self.precision + 0.1).min(10.0);
        self.observations += 1;
    }
    pub fn update_failure(&mut self) {
        self.task_confidence = (self.task_confidence * 0.85).max(0.01);
        self.precision = (self.precision * 0.95).max(0.1);
        self.observations += 1;
    }
    /// variational free energy surrogate
    pub fn vfe(&self) -> f64 {
        let err = 1.0 - self.task_confidence;
        err * err / self.precision.max(0.01)
    }
}

impl Default for BeliefState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct PolicyCandidate {
    pub id: String,
    pub description: String,
    pub pragmatics: f64, // higher = more judge pass expected
    pub epistemic: f64,  // info gain
    pub landauer_cost: f64,
    pub reversible: bool,
}

#[derive(Clone, Debug)]
pub struct RankedPolicy {
    pub candidate: PolicyCandidate,
    pub efe: f64,  // lower better
    pub eior: f64, // expected info on investment ratio
}

pub struct PolicyRanker {
    pub tau_probe: f64,
}

impl Default for PolicyRanker {
    fn default() -> Self {
        Self { tau_probe: 1.5 }
    }
}

impl PolicyRanker {
    /// EFE = -pragmatic - epistemic (lower better); cost-filtered.
    pub fn rank(&self, cands: &[PolicyCandidate]) -> Vec<RankedPolicy> {
        let mut out: Vec<RankedPolicy> = cands
            .iter()
            .map(|c| {
                let efe = -c.pragmatics - 0.5 * c.epistemic;
                let denom = c.landauer_cost.max(1e-9);
                let eior = c.epistemic / denom;
                RankedPolicy {
                    candidate: c.clone(),
                    efe,
                    eior,
                }
            })
            .collect();
        out.sort_by(|a, b| {
            a.efe
                .partial_cmp(&b.efe)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    pub fn should_probe(&self, p: &RankedPolicy) -> bool {
        p.eior >= self.tau_probe
    }
}

/// A declared resource denomination. Values from different lanes are never
/// compared or added without an explicit conversion contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceUnit {
    Token,
    ToolCall,
    CpuMicrosecond,
}

/// Q32.31 policy scalar: the represented value is `raw / 2^31`.
/// Decisions compare raw values exactly; this path performs no float math.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Q32_31(i64);

impl Q32_31 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_i64 << 31);

    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> i64 {
        self.0
    }

    fn is_fraction(self) -> bool {
        self >= Self::ZERO && self <= Self::ONE
    }
}

#[derive(Clone, Debug)]
pub struct BudgetedCandidate {
    pub id: String,
    pub quality_lower_bound: Q32_31,
    pub information_gain: Q32_31,
    pub cost_upper_bound: Q32_31,
    pub unit: ResourceUnit,
    pub reversible: bool,
}

/// A deterministic, one-unit resource gate. Estimates are supplied by the
/// caller and must be calibrated before this can govern live execution.
pub struct BoundedResourcePolicy {
    pub unit: ResourceUnit,
    pub remaining: Q32_31,
    pub minimum_quality: Q32_31,
    pub require_reversible: bool,
}

impl BoundedResourcePolicy {
    pub fn select<'a>(
        &self,
        candidates: &'a [BudgetedCandidate],
    ) -> Result<&'a BudgetedCandidate, String> {
        if !self.minimum_quality.is_fraction() || self.remaining < Q32_31::ZERO {
            return Err("invalid Q32.31 quality floor or resource budget".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for candidate in candidates {
            if candidate.id.trim().is_empty()
                || !ids.insert(candidate.id.as_str())
                || !candidate.quality_lower_bound.is_fraction()
                || !candidate.information_gain.is_fraction()
                || candidate.cost_upper_bound < Q32_31::ZERO
                || candidate.unit != self.unit
            {
                return Err("invalid or duplicate resource-policy candidate".into());
            }
        }
        candidates
            .iter()
            .filter(|candidate| {
                candidate.quality_lower_bound >= self.minimum_quality
                    && candidate.cost_upper_bound <= self.remaining
                    && (!self.require_reversible || candidate.reversible)
            })
            .min_by(|a, b| {
                a.cost_upper_bound
                    .cmp(&b.cost_upper_bound)
                    .then_with(|| b.information_gain.cmp(&a.information_gain))
                    .then_with(|| b.quality_lower_bound.cmp(&a.quality_lower_bound))
                    .then_with(|| a.id.cmp(&b.id))
            })
            .ok_or_else(|| "no admissible action within quality floor and resource budget".into())
    }
}
