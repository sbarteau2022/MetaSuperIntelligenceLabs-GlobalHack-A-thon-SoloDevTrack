//! Item 5d: contributor lane with pinned headless contract.
//!
//! A contributor is any lane
//! that accepts a TaskPacket and returns a TaskReceipt with no human in
//! the loop. Native lane and contributor lane implement the same trait, so
//! a side-by-side dual-lane run compares identical interfaces — the last
//! step to a real head-to-head.

use serde::{Deserialize, Serialize};

/// Headless contract: TaskPacket in, TaskReceipt out. No stdin, no prompt,
/// no human. Both lanes implement ContributorLane; the harness cannot tell
/// which lane produced a receipt except by the lane_id field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPacket {
    pub packet_id: String,
    pub lane_id: String,
    pub task: String,
    pub context_json: String,
    pub budget_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LaneVerdict {
    Done { output_hash: String },
    Refused { reason: String },
    Escalated { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReceipt {
    pub packet_id: String,
    pub lane_id: String,
    pub verdict: LaneVerdict,
    pub tokens_spent: usize,
    pub energy_joules: f64,
}

/// The pinned contract: every lane implements run() headlessly.
pub trait ContributorLane {
    fn lane_id(&self) -> &str;
    fn run(&self, packet: &TaskPacket) -> TaskReceipt;
}

/// Native lane placeholder. It cannot claim execution without an executor.
pub struct NativeLane;

impl ContributorLane for NativeLane {
    fn lane_id(&self) -> &str {
        "native"
    }
    fn run(&self, packet: &TaskPacket) -> TaskReceipt {
        TaskReceipt {
            packet_id: packet.packet_id.clone(),
            lane_id: self.lane_id().into(),
            verdict: LaneVerdict::Escalated {
                reason: "native model executor is not configured".into(),
            },
            tokens_spent: 0,
            energy_joules: 0.0,
        }
    }
}

// Muse contributor lane removed — the real executor is `ModelLane` below,
// which binds Qwen3.6Cyber into the harness and actually claims execution.

/// Side-by-side dual-lane run: same packets, both lanes, paired receipts.
/// The comparison is structural (both Done? same cost class?) — quality
/// comparison waits on real-model execution (item 5b calibration).
pub struct DualLaneRun {
    pub packets: Vec<TaskPacket>,
    pub native_receipts: Vec<TaskReceipt>,
    pub contributor_receipts: Vec<TaskReceipt>,
}

impl DualLaneRun {
    pub fn run(
        native: &dyn ContributorLane,
        contrib: &dyn ContributorLane,
        packets: Vec<TaskPacket>,
    ) -> Self {
        let mut nr = vec![];
        let mut cr = vec![];
        for p in &packets {
            let mut pn = p.clone();
            pn.lane_id = native.lane_id().into();
            nr.push(native.run(&pn));
            let mut pc = p.clone();
            pc.lane_id = contrib.lane_id().into();
            cr.push(contrib.run(&pc));
        }
        Self {
            packets,
            native_receipts: nr,
            contributor_receipts: cr,
        }
    }

    /// Structural agreement: both lanes Done on the same packets.
    pub fn agreement_rate(&self) -> f64 {
        let n = self.packets.len().max(1) as f64;
        let agree = self
            .native_receipts
            .iter()
            .zip(self.contributor_receipts.iter())
            .filter(|(a, b)| {
                matches!(a.verdict, LaneVerdict::Done { .. })
                    && matches!(b.verdict, LaneVerdict::Done { .. })
            })
            .count();
        agree as f64 / n
    }
}

/// Math-kernel lane: executes a typed `KernelOp` on the deterministic CPU
/// oracle and reports the result's canonical sha256 as the receipt's
/// output_hash. No model, no human, no JSON — the op is a Rust value the
/// meta-automaton constructs, and the hash is the verification anchor the
/// superposition layer compares across independent computational routes.
pub struct MathKernelLane {
    pub op: crate::mathkernel::KernelOp,
}

impl ContributorLane for MathKernelLane {
    fn lane_id(&self) -> &str { "math-kernel" }
    fn run(&self, packet: &TaskPacket) -> TaskReceipt {
        match crate::mathkernel::execute(&self.op) {
            Ok(res) => TaskReceipt {
                packet_id: packet.packet_id.clone(),
                lane_id: self.lane_id().into(),
                verdict: LaneVerdict::Done { output_hash: res.hash },
                tokens_spent: 0,
                // Landauer floor for the bits the computation necessarily erased.
                energy_joules: crate::fep::LANDAUER_J_PER_BIT * (res.canonical.len() as f64) * 8.0,
            },
            Err(e) => TaskReceipt {
                packet_id: packet.packet_id.clone(),
                lane_id: self.lane_id().into(),
                verdict: LaneVerdict::Refused { reason: format!("{:?}", e) },
                tokens_spent: 0,
                energy_joules: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod kernel_lane_tests {
    use super::*;
    use crate::mathkernel::KernelOp;
    #[test]
    fn kernel_lane_emits_hash_receipt() {
        let lane = MathKernelLane { op: KernelOp::ModPow { base: 7, exp: 100, modulus: 13 } };
        let pkt = TaskPacket { packet_id: "p1".into(), lane_id: "math-kernel".into(),
            task: "modpow".into(), context_json: String::new(), budget_tokens: 0 };
        match lane.run(&pkt).verdict {
            LaneVerdict::Done { output_hash } => assert_eq!(output_hash.len(), 64),
            v => panic!("expected Done, got {:?}", v),
        }
    }
}

/// The model bound into the harness as a real executor. Fulfills the
/// ContributorLane contract (TaskPacket in, TaskReceipt out, headless) by
/// calling Qwen3.6Cyber on the DSpark server and hashing the completion into
/// the receipt's output_hash. This supersedes the NativeLane stub that
/// escalate with "no executor configured".
///
/// Endpoint from env (DSPARK_URL / DSPARK_MODEL / DSPARK_API_KEY), same as the
/// rest of the harness. `enable_thinking` lets callers pick reasoning (solving)
/// vs. terse formatting (route proposal).
pub struct ModelLane {
    pub system: String,
    pub enable_thinking: bool,
}

impl ModelLane {
    pub fn new(system: impl Into<String>, enable_thinking: bool) -> Self {
        Self { system: system.into(), enable_thinking }
    }

    /// Direct call returning the model's text (for callers that need content,
    /// e.g. route proposal) plus token count. Endpoint resolved from env.
    pub fn complete(&self, user: &str) -> Result<(String, u64), String> {
        let url = std::env::var("DSPARK_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8001/v1/chat/completions".into());
        let model = std::env::var("DSPARK_MODEL").unwrap_or_else(|_| "Qwen3.6Cyber".into());
        let key = std::env::var("DSPARK_API_KEY").unwrap_or_else(|_| "dummy".into());
        let (host, port, path) = crate::ppc_probe::split_url(&url)?;
        crate::ppc_probe::chat_once_ex(&host, port, &path, &model, &key, &self.system, user, self.enable_thinking)
    }
}

impl ContributorLane for ModelLane {
    fn lane_id(&self) -> &str { "model:qwen3.6cyber" }
    fn run(&self, packet: &TaskPacket) -> TaskReceipt {
        match self.complete(&packet.task) {
            Ok((content, tokens)) => {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(content.as_bytes());
                TaskReceipt {
                    packet_id: packet.packet_id.clone(),
                    lane_id: self.lane_id().into(),
                    verdict: LaneVerdict::Done { output_hash: hex::encode(h.finalize()) },
                    tokens_spent: tokens as usize,
                    energy_joules: crate::fep::LANDAUER_J_PER_BIT * (content.len() as f64) * 8.0,
                }
            }
            Err(e) => TaskReceipt {
                packet_id: packet.packet_id.clone(),
                lane_id: self.lane_id().into(),
                verdict: LaneVerdict::Escalated { reason: format!("model call failed: {e}") },
                tokens_spent: 0,
                energy_joules: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod model_lane_tests {
    use super::*;
    // Live: requires the DSpark server on :8001. Run with:
    //   cargo test --release --lib model_lane -- --ignored --nocapture
    #[test]
    #[ignore]
    fn model_lane_live_receipt() {
        let lane = ModelLane::new("Reply with exactly: OK", false);
        let pkt = TaskPacket { packet_id: "m1".into(), lane_id: lane.lane_id().into(),
            task: "say ok".into(), context_json: String::new(), budget_tokens: 0 };
        match lane.run(&pkt).verdict {
            LaneVerdict::Done { output_hash } => assert_eq!(output_hash.len(), 64),
            v => panic!("expected Done, got {:?}", v),
        }
    }
}
