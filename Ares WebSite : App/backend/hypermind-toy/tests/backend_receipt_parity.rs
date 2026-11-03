use ares_backend::harness::mission::{Receipt, ReceiptLog};
use hypermind_toy::runtime::{verify_external_chain, ExternalRow};

#[test]
fn backend_minted_chain_verifies_independently_and_rejects_tampering() {
    let mut log = ReceiptLog::default();
    for (packet, outcome) in [("first", "PASS"), ("second", "ESCALATE")] {
        log.append(Receipt {
            packet_id: packet.into(),
            attempt: 1,
            outcome: outcome.into(),
            judge_ids: vec!["judge-a".into()],
            evidence_hash: String::new(),
            energy_joules: 0.125,
            erased_bits: 7,
            timestamp_ms: 1_000_001,
            prev_hash: String::new(),
            n3_receipts: vec!["a".repeat(64)],
        });
    }
    assert_eq!(log.verify_chain(), Ok(2));
    let rows: Vec<ExternalRow> = log
        .entries
        .iter()
        .map(|e| {
            (
                e.packet_id.clone(),
                e.prev_hash.clone(),
                e.attempt,
                e.outcome.clone(),
                e.judge_ids.clone(),
                e.n3_receipts.clone(),
                e.energy_joules,
                e.erased_bits,
                e.timestamp_ms,
                e.evidence_hash.clone(),
            )
        })
        .collect();
    assert_eq!(verify_external_chain(&rows), Ok(2));
    let mut tampered = rows;
    tampered[1].3 = "PASS".into();
    assert_eq!(verify_external_chain(&tampered), Err(1));
}
