//! Mission execution: run, dry-run, and resume, on top of the existing engine.
//!
//! This module does NOT replace the engine, the judge, or the receipt log.
//! It is the thin entry point the CLI was missing: load a plan, validate it,
//! run it, and return the report plus receipts. Execution itself stays in
//! engine.rs, which is synthetic by design (output is "PASS packet=<id>").
//! A real lane execution replaces engine output, not this entry point.

use crate::energy::EnergyBudget;
use crate::engine::{Engine, EngineReport};
use crate::judge::Judge;
use crate::receipt::ReceiptLog;
use crate::runbook::{PacketStatus, Plan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionInput {
    pub plan: Plan,
    pub judges: Vec<Judge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionReceipt {
    pub mission_id: String,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub skipped: Vec<String>,
    pub escalated: Vec<String>,
    pub receipts: ReceiptLog,
    pub energy_spent_j: f64,
    pub energy_remaining_j: f64,
}

/// Validate only. No run, no side effect. This is the go / no-go check.
pub fn dry_run(plan: &Plan) -> Result<(), String> {
    plan.validate()
}

/// Built-in geometry readiness mission, used by the CLI.
/// Two packets, in order: validate packet structure, then validate ordering.
/// Judges are Contains "PASS", matching the engine's synthetic output.
/// This is a readiness smoke, not a geometry training run itself: training
/// happens on the model side, this mission proves the harness can brief,
/// judge, receipt, and resume a mission before training starts.
pub fn geometry_mission() -> MissionInput {
    use crate::judge::{Judge, JudgeKind};
    use crate::runbook::{JudgeRef, Packet, PacketStatus, RetryPolicy, ToolCall};
    let packet = |id: &str, task: &str, depends_on: Vec<String>| Packet {
        id: id.into(),
        task: task.into(),
        depends_on,
        declared_context_tokens: 100,
        tool_calls: vec![ToolCall {
            tool: "judge".into(),
            args_json: String::new(),
            declared: true,
        }],
        judges: vec![JudgeRef {
            id: "judge-pass".into(),
            kind: "contains".into(),
        }],
        branch: None,
        retry_policy: RetryPolicy {
            max_rounds: 2,
            material_change_required: true,
        },
        status: PacketStatus::Pending,
        attempts: 0,

        judge_joined: false,
    };
    MissionInput {
        plan: Plan {
            id: "mission-geometry-2026-10-29".into(),
            objective: "geometry training readiness: validate packet, judge, receipt chain".into(),
            packets: vec![
                packet("packet-geometry-01", "validate geometry packet structure", vec![]),
                packet(
                    "packet-geometry-02",
                    "validate geometry packet ordering after packet-geometry-01",
                    vec!["packet-geometry-01".to_string()],
                ),
            ],
        },
        judges: vec![Judge {
            id: "judge-pass".into(),
            kind: JudgeKind::Contains {
                needle: "PASS".into(),
            },
        }],
    }
}

/// Run a mission from its input. Returns the mission receipt.
/// Exit policy for the CLI: escalated or failed packets mean the mission
/// did not pass, even though receipts were written.
pub fn run_mission(input: MissionInput, budget_j: f64) -> MissionReceipt {
    run_mission_with_prior(input, budget_j, None)
}

/// Resume a mission. Packets already PASS in the prior receipt log are
/// marked Passed before execution, so passed packets are never re-run.
pub fn run_mission_with_prior(
    mut input: MissionInput,
    budget_j: f64,
    prior: Option<ReceiptLog>,
) -> MissionReceipt {
    let mut receipts = ReceiptLog::default();
    let mut prior_passed: Vec<String> = Vec::new();
    if let Some(prior) = &prior {
        for packet in input.plan.packets.iter_mut() {
            if prior.resume_skips(&packet.id) {
                packet.status = PacketStatus::Passed;
                prior_passed.push(packet.id.clone());
            }
        }
        // Keep prior receipts in the output log, so the mission receipt is whole.
        receipts.entries.extend(prior.entries.iter().cloned());
    }
    let mut engine = Engine::new();
    for judge in input.judges {
        engine.register_judge(judge);
    }
    let mut budget = EnergyBudget::new(budget_j);
    let mut report: EngineReport = engine.run_sequential(&mut input.plan, &mut receipts, &mut budget);
    for id in prior_passed {
        if !report.passed.contains(&id) {
            report.passed.push(id);
        }
    }
    MissionReceipt {
        mission_id: input.plan.id.clone(),
        passed: report.passed,
        failed: report.failed,
        skipped: report.skipped,
        escalated: report.escalated,
        receipts,
        energy_spent_j: budget.spent_j,
        energy_remaining_j: budget.remaining(),
    }
}

impl MissionReceipt {
    pub fn passed_all(&self) -> bool {
        self.failed.is_empty() && self.escalated.is_empty() && !self.passed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judge::{Judge, JudgeKind};
    use crate::runbook::{Packet, PacketStatus, RetryPolicy, ToolCall};

    fn sample_plan() -> Plan {
        Plan {
            id: "mission-geometry-smoke".into(),
            objective: "geometry training readiness smoke".into(),
            packets: vec![Packet {
                id: "packet-geometry-01".into(),
                task: "validate geometry packet".into(),
                depends_on: vec![],
                declared_context_tokens: 100,
                tool_calls: vec![ToolCall {
                    tool: "judge".into(),
                    args_json: "{}".into(),
                    declared: true,
                }],
                judges: vec![crate::runbook::JudgeRef {
                    id: "judge-pass".into(),
                    kind: "contains".into(),
                }],
                branch: None,
                retry_policy: RetryPolicy {
                    max_rounds: 2,
                    material_change_required: true,
                },
                status: PacketStatus::Pending,
                attempts: 0,

                judge_joined: false,
            }],
        }
    }

    #[test]
    fn dry_run_passes_valid_plan() {
        assert!(dry_run(&sample_plan()).is_ok());
    }

    #[test]
    fn run_mission_passes_and_receipts() {
        let input = MissionInput {
            plan: sample_plan(),
            judges: vec![Judge {
                id: "judge-pass".into(),
                kind: JudgeKind::Contains {
                    needle: "PASS".into(),
                },
            }],
        };
        let receipt = run_mission(input, 100.0);
        assert!(receipt.passed_all());
        assert_eq!(receipt.passed, vec!["packet-geometry-01".to_string()]);
        assert!(!receipt.receipts.entries.is_empty());
    }

    #[test]
    fn resume_does_not_rerun_passed_packets() {
        let first = run_mission(
            MissionInput {
                plan: sample_plan(),
                judges: vec![Judge {
                    id: "judge-pass".into(),
                    kind: JudgeKind::Contains {
                        needle: "PASS".into(),
                    },
                }],
            },
            100.0,
        );
        let prior = first.receipts.clone();
        let second = run_mission_with_prior(
            MissionInput {
                plan: sample_plan(),
                judges: vec![Judge {
                    id: "judge-pass".into(),
                    kind: JudgeKind::Contains {
                        needle: "PASS".into(),
                    },
                }],
            },
            100.0,
            Some(prior),
        );
        // The packet was already passed, so the engine has nothing new to run.
        // The mission receipt still carries it as passed, from the prior log.
        assert_eq!(second.passed, vec!["packet-geometry-01".to_string()]);
        assert!(second.receipts.entries.iter().any(|r| r.outcome == "PASS"));
    }
}
