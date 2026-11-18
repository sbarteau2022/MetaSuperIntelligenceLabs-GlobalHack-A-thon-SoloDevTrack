use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Chain-authenticated execution receipt (P0-1/P0-2 closure).
///
/// Every receipt binds: the previous receipt's chain hash (`prev_hash`),
/// the exact N3 `receipt_sha256` digests observed during execution
/// (`n3_receipts`), and the wall-clock issue time (`timestamp_ms`, millis
/// since UNIX epoch — `0` is FORBIDDEN on new receipts; only deserialized
/// legacy fixtures may carry it, and they never verify).
///
/// `evidence_hash` is the canonical chain hash over
/// `prev_hash|packet_id|attempt|outcome|judges|n3_receipts|energy|erased|timestamp`.
/// `verify_chain` recomputes every link; any forgery (edited outcome,
/// invented PASS, truncated file) breaks the chain deterministically.
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
    /// Previous receipt's `evidence_hash` (`GENESIS` for the first entry).
    #[serde(default)]
    pub prev_hash: String,
    /// Exact N3 `receipt_sha256` digests bound to this packet execution.
    /// Empty for executor-less (fail-closed) receipts.
    #[serde(default)]
    pub n3_receipts: Vec<String>,
}

/// Millis since the UNIX epoch. The single wall-clock source for receipts.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Canonical chain hash for one receipt link.
///
/// `energy` is formatted with `{:?}` (Rust Debug for f64: shortest
/// round-trip representation) so hashing is deterministic across runs.
pub fn chain_hash(
    prev_hash: &str,
    packet_id: &str,
    attempt: u32,
    outcome: &str,
    judge_ids: &[String],
    n3_receipts: &[String],
    energy_joules: f64,
    erased_bits: u64,
    timestamp_ms: u64,
) -> String {
    let canon = format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{}|{}",
        prev_hash,
        packet_id,
        attempt,
        outcome,
        judge_ids.join(","),
        n3_receipts.join(","),
        energy_joules,
        erased_bits,
        timestamp_ms,
    );
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    hex::encode(h.finalize())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiptLog {
    pub entries: Vec<Receipt>,
}

impl ReceiptLog {
    /// Chain-authenticated append: stamps `prev_hash` from the current tip,
    /// fills `timestamp_ms` when it is 0, recomputes `evidence_hash` as the
    /// chain hash, then pushes. Callers MUST NOT pre-set `evidence_hash` —
    /// the log is the sole mint.
    pub fn append(&mut self, mut r: Receipt) {
        let prev = self
            .entries
            .last()
            .map(|e| e.evidence_hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        r.prev_hash = prev;
        if r.timestamp_ms == 0 {
            r.timestamp_ms = now_ms();
        }
        r.evidence_hash = chain_hash(
            &r.prev_hash,
            &r.packet_id,
            r.attempt,
            &r.outcome,
            &r.judge_ids,
            &r.n3_receipts,
            r.energy_joules,
            r.erased_bits,
            r.timestamp_ms,
        );
        self.entries.push(r);
    }

    /// Recompute every link. `Ok(count)` when the chain is intact;
    /// `Err(seq)` names the first broken entry (gap, relink, or recompute
    /// mismatch — covers edited outcomes, invented PASS lines, truncation).
    pub fn verify_chain(&self) -> Result<usize, usize> {
        let mut prev = "GENESIS".to_string();
        for (seq, e) in self.entries.iter().enumerate() {
            if e.prev_hash != prev {
                return Err(seq);
            }
            if e.timestamp_ms == 0 {
                return Err(seq);
            }
            let recomputed = chain_hash(
                &e.prev_hash,
                &e.packet_id,
                e.attempt,
                &e.outcome,
                &e.judge_ids,
                &e.n3_receipts,
                e.energy_joules,
                e.erased_bits,
                e.timestamp_ms,
            );
            if recomputed != e.evidence_hash {
                return Err(seq);
            }
            prev = e.evidence_hash.clone();
        }
        Ok(self.entries.len())
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
    /// Chain links that failed verification (forgery signal).
    pub chain_broken: usize,
    /// Whether the whole parsed file verifies (`accepted > 0` + intact chain).
    pub chain_ok: bool,
}

/// Parse a prior receipt file for `resume` (P0-1 closure: chain-verified).
///
/// Accepted lines carry the FULL chained shape this CLI emits:
/// `receipt packet=<id> attempt=<n> outcome=PASS judges=<ids> energy_j=<f>
/// erased_bits=<n> ts=<ms> prev=<hash> chain=<hash> n3=<d1,d2,...>`.
///
/// Verification is two-stage:
/// 1. per-line shape + packet membership (non-PASS, foreign ids, malformed
///    lines are ignored and counted);
/// 2. whole-log chain verification (`verify_chain`): the accepted entries
///    must link gaplessly from GENESIS with recomputed hashes. ANY break →
///    the entire prior file is REJECTED (fail closed): the returned log is
///    empty, `accepted` resets to 0, `chain_ok` is false, and the caller
///    must refuse to report prior passes.
///
/// Forged plaintext PASS lines (the old unauthenticated shape without
/// `prev=`/`chain=` fields) can never verify and are therefore rejected.
pub fn parse_prior_receipt_text(text: &str, plan_ids: &[String]) -> (ReceiptLog, PriorParseStats) {
    let mut log = ReceiptLog::default();
    let mut stats = PriorParseStats::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(rest) = line.strip_prefix("receipt packet=") else {
            stats.ignored += 1;
            continue;
        };
        let packet_id = rest.split_whitespace().next().unwrap_or(rest);
        if !plan_ids.iter().any(|p| p == packet_id) {
            stats.ignored += 1;
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let get = |prefix: &str| -> Option<&str> {
            tokens.iter().find_map(|tok| tok.strip_prefix(prefix))
        };
        // outcome MUST be PASS (case-sensitive); anything else is ignored.
        if get("outcome=") != Some("PASS") {
            stats.ignored += 1;
            continue;
        }
        let attempt: u32 = match get("attempt=").and_then(|v| v.parse().ok()) {
            Some(n) if n >= 1 => n,
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let judge_ids: Vec<String> = get("judges=")
            .map(|v| {
                if v.is_empty() {
                    vec![]
                } else {
                    v.split(',').map(|s| s.to_string()).collect()
                }
            })
            .unwrap_or_default();
        if judge_ids.is_empty() {
            stats.ignored += 1;
            continue;
        }
        let energy: f64 = match get("energy_j=").and_then(|v| v.parse().ok()) {
            Some(v) => v,
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let erased: u64 = match get("erased_bits=").and_then(|v| v.parse().ok()) {
            Some(v) => v,
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let ts: u64 = match get("ts=").and_then(|v| v.parse().ok()) {
            Some(v) if v > 0 => v,
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let prev = match get("prev=") {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let chain = match get("chain=") {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => {
                stats.ignored += 1;
                continue;
            }
        };
        let n3: Vec<String> = get("n3=")
            .map(|v| {
                if v.is_empty() || v == "-" {
                    vec![]
                } else {
                    v.split(',').map(|s| s.to_string()).collect()
                }
            })
            .unwrap_or_default();
        // All N3 digests must be exact 64-hex when present.
        if n3
            .iter()
            .any(|d| d.len() != 64 || !d.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            stats.ignored += 1;
            continue;
        }
        // Push WITHOUT re-minting: the file's own prev/chain/ts are the
        // claim under test. (append() would re-mint; verification needs the
        // raw claimed link.)
        log.entries.push(Receipt {
            packet_id: packet_id.to_string(),
            attempt,
            outcome: "PASS".into(),
            judge_ids,
            evidence_hash: chain,
            energy_joules: energy,
            erased_bits: erased,
            timestamp_ms: ts,
            prev_hash: prev,
            n3_receipts: n3,
        });
        stats.accepted += 1;
    }
    // Stage 2: the accepted entries must form an intact chain.
    match log.verify_chain() {
        Ok(_) if !log.entries.is_empty() => {
            stats.chain_ok = true;
        }
        Ok(_) => {
            // empty file: nothing to verify, nothing to trust.
            stats.chain_ok = false;
        }
        Err(seq) => {
            stats.chain_broken = log.entries.len().saturating_sub(seq);
            stats.chain_ok = false;
            // FAIL CLOSED: a broken chain poisons the whole prior file.
            // Forged lines must never surface as passes.
            log.entries.clear();
            stats.accepted = 0;
        }
    }
    (log, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_ids() -> Vec<String> {
        vec!["packet-geometry-01".into(), "packet-geometry-02".into()]
    }

    /// Build a genuinely chained two-PASS prior file through the real mint
    /// (the only producer of verifiable chains).
    fn chained_prior() -> String {
        let mut log = ReceiptLog::default();
        for id in ["packet-geometry-01", "packet-geometry-02"] {
            log.append(Receipt {
                packet_id: id.into(),
                attempt: 1,
                outcome: "PASS".into(),
                judge_ids: vec!["judge-pass".into()],
                evidence_hash: String::new(), // minted by append
                energy_joules: 0.06,
                erased_bits: 400,
                timestamp_ms: 0, // stamped by append
                prev_hash: String::new(),
                n3_receipts: vec!["a".repeat(64)],
            });
        }
        let mut text = String::from("mission=m\npassed=a,b\n");
        for r in &log.entries {
            text.push_str(&format!(
                "receipt packet={} attempt={} outcome={} judges={} energy_j={:?} erased_bits={} ts={} prev={} chain={} n3={}\n",
                r.packet_id,
                r.attempt,
                r.outcome,
                r.judge_ids.join(","),
                r.energy_joules,
                r.erased_bits,
                r.timestamp_ms,
                r.prev_hash,
                r.evidence_hash,
                r.n3_receipts.join(","),
            ));
        }
        text
    }

    #[test]
    fn chain_append_verifies_and_mints() {
        let mut log = ReceiptLog::default();
        assert_eq!(log.verify_chain(), Ok(0));
        log.append(Receipt {
            packet_id: "p".into(),
            attempt: 1,
            outcome: "PASS".into(),
            judge_ids: vec!["j".into()],
            evidence_hash: "forged".into(), // overwritten by the mint
            energy_joules: 0.5,
            erased_bits: 4,
            timestamp_ms: 0, // stamped (0 forbidden on output)
            prev_hash: "forged".into(),
            n3_receipts: vec![],
        });
        assert_eq!(log.verify_chain(), Ok(1));
        let e = &log.entries[0];
        assert_eq!(e.prev_hash, "GENESIS");
        assert!(e.timestamp_ms > 0, "mint must stamp time");
        assert_ne!(e.evidence_hash, "forged", "mint must recompute");
    }

    #[test]
    fn chain_detects_edited_outcome() {
        let mut log = ReceiptLog::default();
        log.append(Receipt {
            packet_id: "p".into(),
            attempt: 1,
            outcome: "PASS".into(),
            judge_ids: vec!["j".into()],
            evidence_hash: String::new(),
            energy_joules: 0.5,
            erased_bits: 4,
            timestamp_ms: 0,
            prev_hash: String::new(),
            n3_receipts: vec![],
        });
        assert_eq!(log.verify_chain(), Ok(1));
        log.entries[0].outcome = "FAIL".into(); // tamper
        assert_eq!(log.verify_chain(), Err(0));
    }

    #[test]
    fn prior_accepts_chained_pass_lines() {
        let (log, stats) = parse_prior_receipt_text(&chained_prior(), &plan_ids());
        assert!(stats.chain_ok, "genuine chain must verify: {:?}", stats);
        assert_eq!(stats.accepted, 2);
        // mission= AND passed= summary lines are both ignored (not evidence).
        assert_eq!(stats.ignored, 2);
        assert!(log.resume_skips("packet-geometry-01"));
        assert!(log.resume_skips("packet-geometry-02"));
    }

    #[test]
    fn prior_rejects_forged_plaintext_pass_lines() {
        // P0-1 reproducer: the old unauthenticated shape (no prev/chain/ts).
        let text = "mission=x\n\
            receipt packet=packet-geometry-01 attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n\
            receipt packet=packet-geometry-02 attempt=1 outcome=PASS judges=judge-pass energy_j=0.0600 erased_bits=400\n";
        let (log, stats) = parse_prior_receipt_text(text, &plan_ids());
        assert_eq!(stats.accepted, 0, "forgery must accept nothing");
        assert!(!stats.chain_ok);
        assert!(log.entries.is_empty());
        assert!(!log.resume_skips("packet-geometry-01"));
        assert!(!log.resume_skips("packet-geometry-02"));
    }

    #[test]
    fn prior_rejects_tampered_chain() {
        // Swarm review 2026-11-01: the old version of this test flipped the
        // trailing n3 digest digit (a no-op whitespace edit plus wrong field).
        // This version flips a hex digit inside the claimed `chain=` value —
        // the actual chain-forgery path.
        let good = chained_prior();
        let mut tampered = String::new();
        for line in good.lines() {
            if line.contains("packet-geometry-02") {
                let forged = flip_chain_digit(line);
                assert_ne!(forged, line, "chain digit must actually flip");
                tampered.push_str(&forged);
                tampered.push('\n');
            } else {
                tampered.push_str(line);
                tampered.push('\n');
            }
        }
        let (log, stats) = parse_prior_receipt_text(&tampered, &plan_ids());
        assert!(!stats.chain_ok, "tampered chain must fail");
        assert_eq!(stats.accepted, 0, "fail closed: whole file rejected");
        assert!(log.entries.is_empty());
    }

    /// Flip one hex digit inside the `chain=<hex>` token (fails the test
    /// loudly if the line carries no chain token, so a silent no-op flip
    /// can never pass as coverage).
    fn flip_chain_digit(line: &str) -> String {
        let mut tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        let mut flipped = false;
        for tok in tokens.iter_mut() {
            if let Some(hex) = tok.strip_prefix("chain=")
                && hex.len() == 64
                && hex.bytes().all(|b| b.is_ascii_hexdigit())
            {
                let mut chars: Vec<char> = hex.chars().collect();
                chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
                *tok = format!("chain={}", chars.into_iter().collect::<String>());
                flipped = true;
            }
        }
        assert!(
            flipped,
            "fixture line must carry a 64-hex chain token: {}",
            line
        );
        tokens.join(" ")
    }

    #[test]
    fn prior_ignores_bare_ids_and_non_pass() {
        let text = "packet-geometry-01\n\
            receipt packet=packet-geometry-01 attempt=1 outcome=FAIL judges=judge-pass energy_j=0.0600 erased_bits=400 ts=5 prev=x chain=y n3=-\n\
            receipt packet=some-other-packet attempt=1 outcome=PASS judges=j energy_j=0.06 erased_bits=1 ts=5 prev=GENESIS chain=z n3=-\n";
        let (log, stats) = parse_prior_receipt_text(text, &plan_ids());
        assert_eq!(stats.accepted, 0);
        assert_eq!(stats.ignored, 3); // bare id + FAIL outcome + foreign packet
        assert!(!log.resume_skips("packet-geometry-01"));
    }
}
