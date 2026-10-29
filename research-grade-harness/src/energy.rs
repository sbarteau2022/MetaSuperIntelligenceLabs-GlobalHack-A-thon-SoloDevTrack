pub const LANDAUER_J_PER_BIT: f64 = 2.87e-21; // approx kBT ln2 at 300K (REPORTED; surrogate)

#[derive(Clone, Debug)]
pub struct EnergyBudget {
    pub task_budget_j: f64,
    pub probe_budget_j: f64,
    pub spent_j: f64,
    pub landauer_bits_erased: u64,
}

impl EnergyBudget {
    pub fn new(task: f64) -> Self {
        Self {
            task_budget_j: task,
            probe_budget_j: task * 0.3,
            spent_j: 0.0,
            landauer_bits_erased: 0,
        }
    }
    pub fn remaining(&self) -> f64 {
        self.task_budget_j - self.spent_j
    }
    pub fn can_afford(&self, cost: f64) -> bool {
        self.spent_j + cost <= self.task_budget_j
    }
    pub fn charge(&mut self, cost: f64, erased_bits: u64) {
        self.spent_j += cost;
        self.landauer_bits_erased += erased_bits;
    }
}

#[derive(Clone, Debug)]
pub struct EnergyLedger {
    pub entries: Vec<(String, f64, u64)>, // (label, joules, erased_bits)
}

impl EnergyLedger {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }
    pub fn charge(&mut self, label: &str, joules: f64, erased_bits: u64) {
        self.entries.push((label.to_string(), joules, erased_bits));
    }
    pub fn total_j(&self) -> f64 {
        self.entries.iter().map(|e| e.1).sum()
    }
    pub fn total_erased_bits(&self) -> u64 {
        self.entries.iter().map(|e| e.2).sum()
    }
    pub fn landauer_min_j(&self) -> f64 {
        self.total_erased_bits() as f64 * LANDAUER_J_PER_BIT
    }
}

impl Default for EnergyLedger {
    fn default() -> Self {
        Self::new()
    }
}
