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

/// Parse result of a prior receipt file for `resume`.
#[derive(Debug, Clone, Default)]
pub struct PriorParseStats {
    pub accepted: usize,
    pub ignored: usize,
}

/// Parse a prior receipt file for `resume`.
///
/// Guardrail (2026-10-29): only lines in the exact form this CLI emits —
/// `receipt packet=<id> ... outcome=PASS ...` — are trusted. Bare packet ids,
/// summary lines (`passed=...`), receipt lines with a non-PASS outcome, and
/// ids outside this mission are ignored and counted, never marked Passed.
/// A prior file is operator convenience, not acceptance proof: the file
/// itself carries no authentication, only the outcome token is checked.
pub fn parse_prior_receipt_text(text: &str, plan_ids: &[String]) -> (ReceiptLog, PriorParseStats) {
    let mut log = ReceiptLog::default();
    let mut stats = PriorParseStats::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(rest) = line.strip_prefix("receipt packet=") else {
            // Bare id, summary line, or foreign format: not evidence. Ignore.
            stats.ignored += 1;
            continue;
        };
        let packet_id = rest.split_whitespace().next().unwrap_or(rest);
        if !plan_ids.iter().any(|p| p == packet_id) {
            stats.ignored += 1;
            continue;
        }
        let outcome_pass = line.split_whitespace().any(|tok| tok == "outcome=PASS");
        if !outcome_pass {
            stats.ignored += 1;
            continue;
        }
        log.append(Receipt {
            packet_id: packet_id.to_string(),
            attempt: 1,
            outcome: "PASS".into(),
            judge_ids: vec![],
            evidence_hash: String::new(),
            energy_joules: 0.0,
            erased_bits: 0,
            timestamp_ms: 0,
        });
        stats.accepted += 1;
    }
    (log, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_ids() -> Vec<String> {
        vec!["packet-geometry-01".into(), "packet-geometry-02".into()]
    }

    #[test]
    fn prior_accepts_pass_receipt_lines() {
        let text = "mission=mission-geometry-2026-10-29\n\
            passed=packet-geometry-01,packet-geometry-02\n\
            receipt packet=packet-geometry-01 attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n\
            receipt packet=packet-geometry-02 attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n";
        let (log, stats) = parse_prior_receipt_text(text, &plan_ids());
        assert_eq!(stats.accepted, 2);
        // mission= AND passed= summary lines are both ignored (not evidence).
        assert_eq!(stats.ignored, 2);
        assert!(log.resume_skips("packet-geometry-01"));
        assert!(log.resume_skips("packet-geometry-02"));
    }

    #[test]
    fn prior_ignores_bare_ids_and_non_pass() {
        let text = "packet-geometry-01\n\
            receipt packet=packet-geometry-01 attempt=1 outcome=FAIL judges=judge-pass energy_j=0.0600 erased_bits=400\n\
            receipt packet=packet-geometry-02 attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n\
            receipt packet=some-other-packet attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n";
        let (log, stats) = parse_prior_receipt_text(text, &plan_ids());
        assert_eq!(stats.accepted, 1);
        assert_eq!(stats.ignored, 3); // bare id + FAIL outcome + foreign packet
        assert!(!log.resume_skips("packet-geometry-01"));
        assert!(log.resume_skips("packet-geometry-02"));
    }
}
