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
