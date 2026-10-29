#[derive(Debug, Clone, Default)]
pub struct TempleRecord {
    pub ground: Option<String>,
    pub forces: Option<String>,
    pub command: Option<String>,
    pub method: Option<String>,
    pub worth: Option<String>,
    pub sizing: Option<String>,
    pub waste_budget: Option<f64>,
    pub gates: Vec<String>,
}

impl TempleRecord {
    pub fn complete(&self) -> bool {
        self.ground.is_some()
            && self.forces.is_some()
            && self.command.is_some()
            && self.method.is_some()
            && self.worth.is_some()
            && self.sizing.is_some()
            && self.waste_budget.is_some()
            && !self.gates.is_empty()
    }
}
