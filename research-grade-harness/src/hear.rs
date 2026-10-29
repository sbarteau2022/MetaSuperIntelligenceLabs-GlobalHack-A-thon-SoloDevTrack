#[derive(Debug, Clone)]
pub struct WorkflowIntent {
    pub declared_tokens: usize,
    pub needs_prefill: bool,
}

#[derive(Debug, Clone)]
pub struct EngineRuntimeState {
    pub queue_depth: usize,
    pub pressure: bool,
}

#[derive(Debug, Clone)]
pub struct CoordinationDecision {
    pub prefill: bool,
    pub mode: &'static str,
}

pub struct HearCoordinator;

impl HearCoordinator {
    pub fn coordinate(intent: &WorkflowIntent, state: &EngineRuntimeState) -> CoordinationDecision {
        let prefill = intent.needs_prefill && !state.pressure;
        let mode = if state.pressure {
            "workload-aware"
        } else {
            "cache-aware"
        };
        CoordinationDecision { prefill, mode }
    }
}
