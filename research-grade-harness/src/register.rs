#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub id: String,
    pub shape: String,
    pub confidence: f64,
    pub falsified: bool,
}

#[derive(Debug, Clone, Default)]
pub struct HypothesisRegister {
    pub hypotheses: Vec<Hypothesis>,
}

impl HypothesisRegister {
    pub fn add(&mut self, id: &str, shape: &str, conf: f64) {
        self.hypotheses.push(Hypothesis {
            id: id.into(),
            shape: shape.into(),
            confidence: conf,
            falsified: false,
        });
    }
    pub fn strike(&mut self, id: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == id) {
            h.falsified = true;
        }
    }
    /// commit when best probe cost > expected plan change
    pub fn should_commit(&self, best_probe_cost: f64, expected_change: f64) -> bool {
        best_probe_cost > expected_change
    }
    pub fn best_confidence(&self) -> f64 {
        self.hypotheses
            .iter()
            .filter(|h| !h.falsified)
            .map(|h| h.confidence)
            .fold(0.0_f64, f64::max)
    }
}

#[derive(Debug, Clone)]
pub struct ProbePlan {
    pub hypothesis_id: String,
    pub expected_info: f64,
    pub cost: f64,
}

pub fn rank_probes(probes: &mut [ProbePlan]) {
    probes.sort_by(|a, b| {
        let ia = a.expected_info / a.cost.max(1e-9);
        let ib = b.expected_info / b.cost.max(1e-9);
        ib.partial_cmp(&ia).unwrap_or(std::cmp::Ordering::Equal)
    })
}
