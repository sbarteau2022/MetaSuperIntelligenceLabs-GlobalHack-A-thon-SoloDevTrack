//! Item 5d: contributor lane with pinned headless contract.
//!
//! The Muse contributor run() was a 501 refusal by design until the
//! headless contract was pinned. This pins it: a contributor is any lane
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

/// Muse lane placeholder. It cannot claim execution without a model call.
pub struct ContributorLaneMuse;

impl ContributorLane for ContributorLaneMuse {
    fn lane_id(&self) -> &str {
        "contributor-muse"
    }
    fn run(&self, packet: &TaskPacket) -> TaskReceipt {
        TaskReceipt {
            packet_id: packet.packet_id.clone(),
            lane_id: self.lane_id().into(),
            verdict: LaneVerdict::Escalated {
                reason: "Muse model executor is not configured".into(),
            },
            tokens_spent: 0,
            energy_joules: 0.0,
        }
    }
}

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
