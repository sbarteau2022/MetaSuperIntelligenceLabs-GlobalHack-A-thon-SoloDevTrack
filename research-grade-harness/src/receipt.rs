use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub packet_id: String,
    pub attempt: u32,
    pub outcome: String, // "PASS" / "FAIL" / "SKIP" / "ESCALATE"
    pub judge_ids: Vec<String>,
    pub evidence_hash: String,
    pub energy_joules: f64, // surrogate
    pub erased_bits: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiptLog {
    pub entries: Vec<Receipt>,
}

impl ReceiptLog {
    pub fn append(&mut self, r: Receipt) {
        self.entries.push(r);
    }

    pub fn passed_packets(&self) -> std::collections::HashSet<String> {
        self.entries
            .iter()
            .filter(|r| r.outcome == "PASS")
            .map(|r| r.packet_id.clone())
            .collect()
    }

    pub fn total_energy(&self) -> f64 {
        self.entries.iter().map(|r| r.energy_joules).sum()
    }

    pub fn total_erased_bits(&self) -> u64 {
        self.entries.iter().map(|r| r.erased_bits).sum()
    }

    pub fn resume_skips(&self, packet_id: &str) -> bool {
        self.entries
            .iter()
            .any(|r| r.packet_id == packet_id && r.outcome == "PASS")
    }
}
