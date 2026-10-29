use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct TelemetryFrame {
    pub coverage_exact: f64,
    pub coverage_estimate: f64,
    pub tokens_per_passed_judge: f64,
    pub landauer_cost_per_passed_judge: f64,
    pub vfe: f64,
    pub eior: f64,
    pub sampling_mode: String,
}
