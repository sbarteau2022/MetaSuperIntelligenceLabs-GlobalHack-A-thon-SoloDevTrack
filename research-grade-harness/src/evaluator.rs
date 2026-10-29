//! Item 4: separate live evaluator — Planner, Generator, Evaluator as
//! separate roles, never one agent grading its own work.
//!
//! The distillation audit found the judge was a template, not a gate.
//! This is the fix: sprint contracts negotiated BEFORE code (what counts
//! as done, in advance), the Evaluator operates the running app (live
//! checks against a live backend, not static code review), and a failed
//! evaluation fails the sprint — the cycle repeats.
//!
//! Role separation is structural: the Evaluator holds no reference to the
//! Generator's internals, only to the contract and the live app handle.

use serde::{Deserialize, Serialize};

/// Sprint contract: negotiated before code. What counts as done, in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintContract {
    pub sprint_id: String,
    pub objective: String,
    /// Live checks: each is (name, probe description, pass predicate).
    /// The Evaluator runs these against the RUNNING app, not the code.
    pub live_checks: Vec<LiveCheck>,
    pub max_cycles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCheck {
    pub name: String,
    /// How the evaluator operates the app (e.g. "POST /traverse-check budget=0").
    pub probe: String,
    /// Expected observable outcome (e.g. "verdict == fail-explained").
    pub expect: String,
}

/// Planner output: a runbook Plan plus the contract it will be judged by.
#[derive(Debug, Clone)]
pub struct PlannedSprint {
    pub contract: SprintContract,
    pub plan_id: String,
}

/// Generator output: evidence of a running app, never code claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApp {
    pub sprint_id: String,
    pub cycle: u32,
    /// How the evaluator reaches the app (base URL, handle, session).
    pub app_handle: String,
    pub cycle_receipt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvalVerdict {
    Pass,
    Fail {
        failed_checks: Vec<String>,
        cycle: u32,
    },
}

/// The Evaluator: holds the contract + a live app handle. No access to
/// generator internals by construction (separate struct, separate inputs).
pub struct LiveEvaluator {
    pub contract: SprintContract,
}

impl LiveEvaluator {
    pub fn new(contract: SprintContract) -> Self {
        Self { contract }
    }

    /// Evaluate by OPERATING the app: run each live check's probe against
    /// the handle, compare observables to expectations.
    /// `operate` is injected (the harness running the app, a test double in
    /// unit tests, the real HTTP client in production).
    pub fn evaluate(
        &self,
        app: &GeneratedApp,
        operate: &dyn Fn(&str, &str) -> String,
    ) -> EvalVerdict {
        let mut failed = vec![];
        for check in &self.contract.live_checks {
            let observed = operate(&app.app_handle, &check.probe);
            if !observed.contains(&check.expect) {
                failed.push(format!(
                    "{}: observed {:?} missing {:?}",
                    check.name, observed, check.expect
                ));
            }
        }
        if failed.is_empty() {
            EvalVerdict::Pass
        } else {
            EvalVerdict::Fail {
                failed_checks: failed,
                cycle: app.cycle,
            }
        }
    }

    /// A failed evaluation fails the sprint; the cycle repeats (up to
    /// max_cycles). Returns true if another cycle is allowed.
    pub fn sprint_continues(&self, verdict: &EvalVerdict, cycle: u32) -> bool {
        matches!(verdict, EvalVerdict::Fail { .. }) && cycle < self.contract.max_cycles
    }
}
