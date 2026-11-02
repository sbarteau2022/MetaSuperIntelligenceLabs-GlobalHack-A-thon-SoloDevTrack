//! Adversarial receipt-chain tests: forgery, reorder, and replay must all
//! fail closed (whole prior file rejected, `accepted == 0`).
//!
//! Every fixture is minted through the real `ReceiptLog::append` (the sole
//! chain mint) and serialized in the exact CLI emit format:
//! `receipt packet=<id> attempt=<n> outcome=PASS judges=<j> energy_j=<f>
//! erased_bits=<n> ts=<ms> prev=<h> chain=<h> n3=<digests|-`
//! No hand-forged hashes anywhere: each attack mutates, reorders, or replays
//! genuinely minted lines.

use rgah::receipt::{Receipt, ReceiptLog, parse_prior_receipt_text};

const P1: &str = "packet-geometry-01";
const P2: &str = "packet-geometry-02";
const JUDGE: &str = "judge-pass";
const E1: f64 = 0.06;
const E2: f64 = 0.09;

fn plan_ids() -> Vec<String> {
    vec![P1.to_string(), P2.to_string()]
}

/// Mint a genuine two-receipt chain through the real mint and return its
/// serialized lines in the CLI emit format.
fn minted_lines() -> Vec<String> {
    let mut log = ReceiptLog::default();
    for (id, energy) in [(P1, E1), (P2, E2)] {
        log.append(Receipt {
            packet_id: id.to_string(),
            attempt: 1,
            outcome: "PASS".to_string(),
            judge_ids: vec![JUDGE.to_string()],
            evidence_hash: String::new(), // minted by append
            energy_joules: energy,
            erased_bits: 400,
            timestamp_ms: 0,          // stamped by append
            prev_hash: String::new(), // linked by append
            n3_receipts: vec!["a".repeat(64)],
        });
    }
    assert_eq!(log.verify_chain(), Ok(2));
    log.entries
        .iter()
        .map(|r| {
            format!(
                "receipt packet={} attempt={} outcome={} judges={} energy_j={:?} erased_bits={} ts={} prev={} chain={} n3={}",
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
            )
        })
        .collect()
}

fn to_text(lines: &[String]) -> String {
    let mut text = String::from("mission=adversarial\n");
    for l in lines {
        text.push_str(l);
        text.push('\n');
    }
    text
}

/// Positive control: the unmodified mint must verify (guards against vacuous
/// rejection if the mint/emit format ever drifts).
fn assert_control_accepts(lines: &[String], plan: &[String]) {
    let (log, stats) = parse_prior_receipt_text(&to_text(lines), plan);
    assert!(stats.chain_ok, "control fixture must verify: {stats:?}");
    assert_eq!(stats.accepted, 2);
    assert!(log.resume_skips(P1));
    assert!(log.resume_skips(P2));
}

fn assert_fail_closed(text: &str, plan: &[String]) {
    let (log, stats) = parse_prior_receipt_text(text, plan);
    assert!(!stats.chain_ok, "attack must break the chain: {stats:?}");
    assert_eq!(stats.accepted, 0, "fail closed: whole file rejected");
    assert!(stats.chain_broken > 0, "break must be located: {stats:?}");
    assert!(log.entries.is_empty());
    assert!(!log.resume_skips(P1));
    assert!(!log.resume_skips(P2));
}

#[test]
fn energy_bit_flip_rejects_whole_file() {
    let lines = minted_lines();
    let plan = plan_ids();
    assert_control_accepts(&lines, &plan);

    // Flip the energy value on the first chained line. Shape stays valid
    // (a parseable f64), so stage 1 accepts it and stage 2 must reject it.
    let from = format!("energy_j={:?}", E1);
    let to = format!("energy_j={:?}", E1 + 0.5);
    let bad0 = lines[0].replacen(from.as_str(), to.as_str(), 1);
    assert_ne!(bad0, lines[0], "tamper must change the line");
    assert_fail_closed(&to_text(&[bad0, lines[1].clone()]), &plan);
}

#[test]
fn reordered_lines_reject_whole_file() {
    let lines = minted_lines();
    let plan = plan_ids();
    assert_control_accepts(&lines, &plan);

    // Swap the two chained receipts: the second line's `prev` no longer
    // follows GENESIS, so the chain must break at seq 0.
    assert_fail_closed(&to_text(&[lines[1].clone(), lines[0].clone()]), &plan);
}

#[test]
fn replayed_first_line_rejects_whole_file() {
    let lines = minted_lines();
    let plan = plan_ids();
    assert_control_accepts(&lines, &plan);

    // Duplicate the first line: the replay's `prev=GENESIS` no longer links
    // from the first copy's chain hash, so the chain must break at seq 1.
    assert_fail_closed(&to_text(&[lines[0].clone(), lines[0].clone()]), &plan);
}
