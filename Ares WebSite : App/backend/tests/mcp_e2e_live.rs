//! Live end-to-end MCP test: proposal -> automaton -> ares-mcp-mind
//! child -> verified outcome -> Surreal persistence.
//!
//! Ignored by default. Run explicitly with:
//!   cargo test --test mcp_e2e_live -- --ignored
//!
//! Requires: SurrealDB on 127.0.0.1:8000 (governance, ares/ares),
//! the ares-mcp-mind binary built, and a backend SurrealClient
//! configuration. The child is spawned transiently and killed on
//! drop; no server is left running.
//!
//! NOTE — live hypermind-mcp mind is BLOCKED, not used here:
//! spawning hypermind-mcp against the live mind (hypermind/core)
//! fails at startup with "surreal: contract migration
//! 0004_tppe_contract_correction missing or drifted". That is a
//! live mind schema drift, not a test bug, and fixing it means
//! applying a migration to the persistent mind — a decision for
//! whoever owns mind schema, not a silent step in this test.
//! This test proves the automaton → MCP → persistence loop
//! end-to-end on the backend-owned MCP server; the hypermind
//! live-mind path stays blocked until that migration is resolved.

use ares_backend::fixed::Q32_31;
use ares_backend::harness::automaton::{
    confidence_create_sql, confidence_upsert_sql, transition_insert_sql, AuditSignals,
    ProposerEnvelope, ProposerFamily, StrategyState, Verdict,
};
use ares_backend::harness::mcp::{
    execute_via_automaton, McpAuthority, McpTransport, StdioMcpTransport,
};
use ares_backend::surreal::SurrealClient;

#[tokio::test]
#[ignore]
async fn mcp_end_to_end_live() {
    let client = SurrealClient::from_env().expect("surreal client");
    let (gov_base, gov_ns, gov_db) = client.authority();
    assert_eq!(gov_base, "http://127.0.0.1:8000");
    assert_eq!(gov_ns, "ares");
    assert_eq!(gov_db, "ares");

    let authority = McpAuthority {
        base: gov_base.clone(),
        ns: gov_ns.clone(),
        db: gov_db.clone(),
    };
    let bin = std::env::var("ARES_MCP_MIND_BIN").unwrap_or_else(|_| {
        "/tmp/ares-backend-target/debug/ares-mcp-mind".to_string()
    });
    assert!(std::path::Path::new(&bin).is_file(), "ares-mcp-mind binary missing: build it first");
    let mut transport =
        StdioMcpTransport::spawn(&bin, ".", &[]).expect("spawn ares-mcp-mind");

    // 1. Live child attestation: db_status must name the exact authority.
    let status = transport
        .call("mind.db_status", "{}")
        .expect("mind.db_status");
    let inner: serde_json::Value =
        serde_json::from_str(&status.inner_text).expect("db_status inner json");
    assert_eq!(inner.get("mode").and_then(|v| v.as_str()), Some("On"));
    assert_eq!(inner.get("endpoint").and_then(|v| v.as_str()), Some("http://127.0.0.1:8000"));
    assert_eq!(inner.get("namespace").and_then(|v| v.as_str()), Some("ares"));
    assert_eq!(inner.get("database").and_then(|v| v.as_str()), Some("ares"));
    assert_eq!(status.receipt_sha256.len(), 64);

    // 2. Automaton-gated MCP call through the same child.
    let mut state = StrategyState {
        strategy_id: "mcp-e2e".to_string(),
        policy_version: "policy-v1".to_string(),
        prefix_identity: "prefix-1".to_string(),
        map_identity: "globalmind-ares".to_string(),
        confidence: Q32_31::from_f64(0.6),
        confidence_floor: Q32_31::from_f64(0.5),
        calibration_count: 0,
        outcome_history_digest: "h0".to_string(),
    };
    let envelope = ProposerEnvelope {
        family: ProposerFamily::Retrieval,
        strategy_id: "mcp-e2e".to_string(),
        policy_version: "policy-v1".to_string(),
        prefix_identity: "prefix-1".to_string(),
        map_identity: "globalmind-ares".to_string(),
        payload: "mind.db_status".to_string(),
        expiry_ms: u64::MAX / 2,
    };
    let signals = AuditSignals {
        measured_confidence: Q32_31::from_f64(0.8),
        unresolved_dependencies: 0,
        entity_ambiguity: false,
        expected_information_gain: Q32_31::from_f64(0.7),
    };
    let ex = execute_via_automaton(
        &mut state,
        &envelope,
        &signals,
        100,
        &authority,
        &authority,
        "mind.db_status",
        "{}",
        &mut transport,
        true,
        true,
    )
    .expect("automaton-gated mind.db_status");
    assert_eq!(ex.transition.verdict, Verdict::Admit);
    let outcome = ex.outcome.expect("admit must have an outcome");
    assert_eq!(outcome.receipt_sha256.len(), 64);
    assert!(ex.confidence_transition.is_some());

    // 3. Persist confidence + transition receipt to governance.
    let _ = client
        .sql(&confidence_create_sql(&state))
        .await
        .expect("create automaton_confidence");
    client
        .sql(&confidence_upsert_sql(&state))
        .await
        .expect("upsert automaton_confidence");
    client
        .sql(&transition_insert_sql(&ex.transition))
        .await
        .expect("insert automaton_receipt");

    // 4. Read back: persistence proven by a row, not by a write call.
    let back = client
        .sql("SELECT strategy_id, confidence_raw FROM automaton_confidence WHERE strategy_id = 'mcp-e2e';")
        .await
        .expect("read back automaton_confidence");
    let back_text = serde_json::to_string(&back).unwrap();
    assert!(
        back_text.contains("mcp-e2e"),
        "no automaton_confidence row for mcp-e2e: {}",
        back_text
    );
}
