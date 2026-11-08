//! Loop controller: picks a loop shape from task properties, then reselects
//! mid-task from per-cycle verifier signals. Every switch leaves a receipt.

#[derive(Debug, Clone, PartialEq)]
pub enum LoopShape {
    OneShot,
    BoundedRetry { max_rounds: u32 },
    JudgeGated { max_rounds: u32 },
    Scheduled { interval_ms: u64, max_cycles: u32 },
    FanOut { workers: u32 },
}

/// Properties of a task that determine its starting loop shape.
#[derive(Debug, Clone)]
pub struct TaskProfile {
    /// A deterministic verifier exists for the output.
    pub verifiable: bool,
    /// Sub-work can run concurrently without shared state.
    pub parallel_independent: bool,
    /// The task runs over hours or days and must recur.
    pub long_running: bool,
    /// Upper bound on cycles the task may consume.
    pub cycle_budget: u32,
}

/// What one cycle reported back to the controller.
#[derive(Debug, Clone)]
pub struct CycleSignal {
    pub verifier_pass: bool,
    pub stalled: bool,
    pub budget_used_frac: f64,
    pub error_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Continue,
    Reselect(LoopShape),
    Stop(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchReceipt {
    pub from: LoopShape,
    pub to: LoopShape,
    pub reason: String,
}

/// Budget fraction at which the controller downgrades to one-shot.
const BUDGET_DOWNGRADE_FRAC: f64 = 0.8;

pub fn select_shape(p: &TaskProfile) -> LoopShape {
    if !p.verifiable {
        LoopShape::OneShot
    } else if p.long_running {
        LoopShape::Scheduled { interval_ms: 60_000, max_cycles: p.cycle_budget }
    } else if p.parallel_independent {
        LoopShape::FanOut { workers: 4 }
    } else {
        LoopShape::BoundedRetry { max_rounds: p.cycle_budget }
    }
}

pub struct Controller {
    shape: LoopShape,
    streak: u32,
    max_switches: u32,
    receipts: Vec<SwitchReceipt>,
}

impl Controller {
    pub fn new(shape: LoopShape, max_switches: u32) -> Self {
        Self { shape, streak: 0, max_switches, receipts: Vec::new() }
    }

    pub fn receipts(&self) -> &[SwitchReceipt] {
        &self.receipts
    }

    pub fn observe(&mut self, sig: &CycleSignal) -> Decision {
        if sig.verifier_pass {
            self.streak = 0;
            return Decision::Continue;
        }
        self.streak += 1;

        let over_budget = sig.budget_used_frac >= BUDGET_DOWNGRADE_FRAC;
        if over_budget && self.shape != LoopShape::OneShot {
            return self.switch(LoopShape::OneShot, "budget burn at or above 80%");
        }
        if self.streak < 2 {
            return Decision::Continue;
        }
        match escalate(&self.shape) {
            Some(next) => self.switch(next, "two consecutive verifier failures"),
            None => Decision::Stop("no stronger shape available".into()),
        }
    }

    fn switch(&mut self, to: LoopShape, reason: &str) -> Decision {
        if self.receipts.len() as u32 >= self.max_switches {
            return Decision::Stop(format!("switch cap of {} reached", self.max_switches));
        }
        self.receipts.push(SwitchReceipt {
            from: self.shape.clone(),
            to: to.clone(),
            reason: reason.to_string(),
        });
        self.shape = to.clone();
        self.streak = 0;
        Decision::Reselect(to)
    }
}

/// The next shape up the ladder when a loop keeps failing.
fn escalate(shape: &LoopShape) -> Option<LoopShape> {
    match shape {
        LoopShape::OneShot => Some(LoopShape::BoundedRetry { max_rounds: 3 }),
        LoopShape::BoundedRetry { max_rounds } => Some(LoopShape::JudgeGated { max_rounds: *max_rounds }),
        LoopShape::JudgeGated { .. } | LoopShape::Scheduled { .. } | LoopShape::FanOut { .. } => None,
    }
}
