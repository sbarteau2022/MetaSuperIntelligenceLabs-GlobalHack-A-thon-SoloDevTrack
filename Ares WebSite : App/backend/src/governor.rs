//! The backend run governor + application state.
//!
//! The Governor cannot dispatch without a permit: every dispatch method on
//! this type takes the `ImmuneSystem` lock first, runs the gate, and only
//! then performs the work. There is no code path that skips the gate, no
//! "compliance off" flag, no debug bypass.
//!
//! Startup is fail-closed: the audit chain is loaded from SurrealDB and
//! verified from genesis to head before the server binds. A broken chain
//! is a startup failure with a why-receipt, not a warning.

use crate::compliance::{CompliancePermit, ComplianceViolation, ImmuneSystem, OverlapPolicyMatrix};
use crate::digest::Sha256Digest;
use crate::fixed::Q32_31;
use crate::roles::RoleId;
use crate::store::{
    AtomId, BridgeLossReport, CarvedSwarmMap, EphemeralMapDeposit, FrozenBaseWeights,
};
use crate::surreal::{sql_escape, SurrealClient};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub immune: Mutex<ImmuneSystem>,
    pub map: Mutex<CarvedSwarmMap>,
    /// RG-AH harness state (runtime framework, doctrine-bounded).
    /// Telemetry is exported append-only into HyperDX; memory is
    /// append-only in Qdrant (research RAG + session memories).
    pub telemetry: Option<crate::harness::telemetry::HyperdxTelemetry>,
    pub memory: Option<crate::harness::qdrant_memory::QdrantMemory>,
    pub capability_gate: Mutex<crate::harness::runtime::CapabilityGate>,
    /// Ephemeral, role-bound worker read capabilities. Restart revokes all.
    pub carved_scopes: Mutex<crate::harness::scope::ScopeRegistry>,
    /// O(1) sequence cache: one TppeSequence per session stream per
    /// process. Append advances one step; rehydrate happens once per
    /// (session, process lifetime), never per append (no O(N^2)).
    pub sequences:
        Mutex<std::collections::HashMap<String, crate::harness::tppe_memory::TppeSequence>>,
    /// Bounded-concurrency doctrine: at most this many governed runs at once.
    pub run_semaphore: Arc<tokio::sync::Semaphore>,
    pub surreal: Option<SurrealClient>,
    /// Model seat that served the last model call (recorded per call).
    pub last_seat: Mutex<Option<String>>,
    /// Restart-cap config for Gate 7.
    pub max_restarts: u32,
}

impl AppState {
    pub fn new(immune: ImmuneSystem, map: CarvedSwarmMap, surreal: Option<SurrealClient>) -> Self {
        Self {
            immune: Mutex::new(immune),
            map: Mutex::new(map),
            telemetry: None,
            memory: None,
            capability_gate: Mutex::new(crate::harness::runtime::CapabilityGate::default()),
            carved_scopes: Mutex::new(crate::harness::scope::ScopeRegistry::default()),
            sequences: Mutex::new(std::collections::HashMap::new()),
            run_semaphore: Arc::new(tokio::sync::Semaphore::new(4)),
            surreal,
            last_seat: Mutex::new(None),
            max_restarts: 3,
        }
    }
}

/// Why-receipt: every failure returns what ran, the route/command, the
/// status, the last output, the expected result, the evidenced cause, and
/// the unblocker.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WhyReceipt {
    pub what_ran: String,
    pub route: String,
    pub status: String,
    pub last_output: String,
    pub expected: String,
    pub evidenced_cause: String,
    pub unblocker: String,
}

impl WhyReceipt {
    pub fn from_violation(route: &str, v: &ComplianceViolation) -> Self {
        Self {
            what_ran: format!("compliance gate for {}", route),
            route: route.to_string(),
            status: "rejected".to_string(),
            last_output: v.to_string(),
            expected: "a compliance permit minted by ImmuneSystem".to_string(),
            evidenced_cause: v.to_string(),
            unblocker: "satisfy the named gate; the violation names the law, roles, and atom"
                .to_string(),
        }
    }

    pub fn internal(route: &str, cause: &str) -> Self {
        Self {
            what_ran: format!("backend operation for {}", route),
            route: route.to_string(),
            status: "error".to_string(),
            last_output: cause.to_string(),
            expected: "successful governed operation".to_string(),
            evidenced_cause: cause.to_string(),
            unblocker: "see evidenced_cause; governance state is recorded in SurrealDB".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Governor dispatch path. Each method: lock immune → gate → work → persist.
// ---------------------------------------------------------------------------

/// Gate 2 dispatch: spawn/restart a lane role.
pub async fn dispatch_spawn(
    state: &Arc<AppState>,
    role: RoleId,
) -> Result<CompliancePermit, ComplianceViolation> {
    let map = state.map.lock().await;
    let mut immune = state.immune.lock().await;
    let permit = immune.authorize_role_spawn(role, &map)?;
    persist_audit_tail(state, &immune).await;
    Ok(permit)
}

/// Live dispatch currently uses an MCP-only executor. It does not
/// generate model tokens or certify a Glimmer forward. The permit
/// gate still runs first. Gate nodes are NOT
/// auto-registered here: a `map.*` call with no judged-atom gate node
/// is denied by the engine, and an unreachable automaton escalates —
/// both by design, never papered over. Returns (permit, receipt, seat).
pub async fn dispatch_mission_live(
    state: &Arc<AppState>,
    role: RoleId,
    mut input: crate::harness::mission::MissionInput,
    budget_j: f64,
) -> Result<
    (
        CompliancePermit,
        crate::harness::mission::MissionReceipt,
        String,
    ),
    String,
> {
    let _permit_slot = state
        .run_semaphore
        .clone()
        .acquire_owned()
        .await
        .expect("run semaphore never closes");
    let permit = dispatch_spawn(state, role)
        .await
        .map_err(|_| "permit denied".to_string())?;
    let telemetry = state
        .telemetry
        .as_ref()
        .ok_or_else(|| "live run requires HyperDX telemetry configured".to_string())?;
    let gate = state.capability_gate.lock().await.clone();
    let surreal = state
        .surreal
        .as_ref()
        .ok_or_else(|| "live run requires Surreal governance".to_string())?;
    let authority = crate::harness::mcp::McpAuthority {
        base: std::env::var("ARES_MCP_MIND_URL")
            .map_err(|_| "ARES_MCP_MIND_URL required".to_string())?,
        ns: std::env::var("ARES_MCP_MIND_NS")
            .map_err(|_| "ARES_MCP_MIND_NS required".to_string())?,
        db: std::env::var("ARES_MCP_MIND_DB")
            .map_err(|_| "ARES_MCP_MIND_DB required".to_string())?,
    };
    let creds = surreal.credentials();
    let mind =
        crate::harness::mind_recall_executor::MindRecallExecutor::from_state(authority, creds)?;
    // ARES_GLIMMER_NATIVE_URL selects the native Glimmer executor (bound
    // receipts); otherwise the MCP-only executor runs and bound calls refuse.
    let (mut executor, seat): (Box<dyn crate::harness::runtime::Executor>, String) =
        if std::env::var("ARES_GLIMMER_NATIVE_URL").is_ok() {
            let native =
                crate::harness::native_glimmer_executor::NativeGlimmerExecutor::from_env(mind)?;
            let seat = native.seat_identity();
            (Box::new(native), seat)
        } else {
            let seat = mind.seat_identity();
            (Box::new(mind), seat)
        };
    let mut engine = crate::harness::runtime::Engine::new();
    for judge in input.judges {
        engine.register_judge(judge);
    }
    let mut budget = crate::harness::mission::EnergyBudget::new(budget_j);
    let mut receipts = crate::harness::mission::ReceiptLog::default();
    let report = engine
        .run_sequential_live(
            &mut input.plan,
            &mut receipts,
            &mut budget,
            &mut *executor,
            telemetry,
            &gate,
        )
        .await;
    let receipt = crate::harness::mission::MissionReceipt {
        mission_id: input.plan.id.clone(),
        passed: report.passed,
        failed: report.failed,
        skipped: report.skipped,
        escalated: report.escalated,
        receipts,
        energy_spent_j: budget.spent_j,
        energy_remaining_j: budget.remaining(),
    };
    if let Some(client) = &state.surreal {
        let stmt = format!(
            "CREATE run SET thread = '{}', status = '{}', model_seat = '{}';",
            sql_escape(&receipt.mission_id),
            if receipt.passed_all() {
                "passed"
            } else {
                "escalated"
            },
            sql_escape(&seat),
        );
        let _ = client.sql(&stmt).await;
    }
    Ok((permit, receipt, seat))
}

/// Gate 3 dispatch: inter-role handoff carrying an atom.
pub async fn dispatch_handoff(
    state: &Arc<AppState>,
    from: RoleId,
    to: RoleId,
    atom_id: AtomId,
) -> Result<CompliancePermit, ComplianceViolation> {
    let map = state.map.lock().await;
    let mut immune = state.immune.lock().await;
    let permit = immune.authorize_handoff(from, to, atom_id, &map)?;
    persist_audit_tail(state, &immune).await;
    Ok(permit)
}

/// Gate 6 dispatch: authorize a forward pass for a role.
pub async fn dispatch_forward_pass(
    state: &Arc<AppState>,
    role: RoleId,
    token_id: u32,
    pos: usize,
    bridge_loss: &BridgeLossReport,
) -> Result<CompliancePermit, ComplianceViolation> {
    let map = state.map.lock().await;
    let mut immune = state.immune.lock().await;
    let permit = immune.authorize_forward_pass(role, token_id, pos, &map, bridge_loss)?;
    persist_audit_tail(state, &immune).await;
    Ok(permit)
}

/// Gate 7 dispatch: governor supervision tick (restart cap enforced).
pub async fn dispatch_supervision(
    state: &Arc<AppState>,
    tick: u64,
    active_restarts_in_window: u32,
) -> Result<CompliancePermit, ComplianceViolation> {
    let map = state.map.lock().await;
    let mut immune = state.immune.lock().await;
    let permit = immune.authorize_governor_supervision(
        tick,
        active_restarts_in_window,
        state.max_restarts,
        &map,
    )?;
    persist_audit_tail(state, &immune).await;
    Ok(permit)
}

/// Gate 8 dispatch: compile an ephemeral deposit into the map. On permit,
/// the deposit's atoms are kept and carved under the current policy.
pub async fn dispatch_deposit(
    state: &Arc<AppState>,
    deposit: &EphemeralMapDeposit,
) -> Result<Vec<AtomId>, ComplianceViolation> {
    let mut map = state.map.lock().await;
    let policy = {
        let immune = state.immune.lock().await;
        immune.policy.clone()
    };
    {
        let mut immune = state.immune.lock().await;
        immune.authorize_deposit_compilation(deposit, &map)?;
        persist_audit_tail(state, &immune).await;
    }
    let mut ids = Vec::new();
    for coord in &deposit.records {
        let id = map.keep_and_carve(*coord, Vec::new(), deposit.from_role, &policy);
        ids.push(id);
    }
    Ok(ids)
}

// ---------------------------------------------------------------------------
// Persistence (SurrealDB). Audit entries are append-only; the head digest
// is stored so startup can verify recomputation from genesis.
// ---------------------------------------------------------------------------

/// Persist every audit entry not yet stored. A gate that internally runs
/// Gate 1 appends TWO entries (the weight check + its own verdict), so
/// persisting only the last entry would silently drop chain links and the
/// stored chain would no longer recompute to the head. We therefore diff
/// against the stored max sequence and persist the full missing tail.
/// Record ids are deterministic (`audit_entry:<sequence>`), so a re-run is
/// an idempotent "already exists" we can safely skip — the row content is
/// identical by construction (same sequence ⇒ same chained digest).
pub async fn persist_audit_tail(state: &Arc<AppState>, immune: &ImmuneSystem) {
    let Some(client) = &state.surreal else {
        return;
    };
    let stored_max: u64 = match client
        .sql("SELECT sequence FROM audit_entry ORDER BY sequence DESC LIMIT 1;")
        .await
    {
        Ok(v) => v
            .as_array()
            .and_then(|a| a.first())
            .and_then(|r| r.get("sequence"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
        Err(_) => 0,
    };
    for e in immune
        .audit_log()
        .iter()
        .filter(|e| e.sequence > stored_max)
    {
        let stmt = format!(
            "CREATE audit_entry:{} SET sequence = {}, action = '{}', approved = {}, mark = '{}', prev_digest = '{}', entry_digest = '{}';",
            e.sequence,
            e.sequence,
            sql_escape(&e.action),
            e.approved,
            e.mark.as_tag(),
            e.prev_digest.to_hex(),
            e.entry_digest.to_hex()
        );
        let _ = client.sql(&stmt).await;
        // The permit table mirrors every approved entry that minted a
        // permit (PASS gates). Rejections have no permit row — by design.
        if e.approved && e.action.starts_with("PASS") {
            let permit_stmt = format!(
                "CREATE permit:{} SET sequence = {}, action_summary = '{}', epistemic_mark = '{}', chain_digest = '{}';",
                e.sequence,
                e.sequence,
                sql_escape(&e.action),
                e.mark.as_tag(),
                e.entry_digest.to_hex()
            );
            let _ = client.sql(&permit_stmt).await;
        }
    }
    let _ = client
        .sql(&format!(
            "CREATE merkle_root:head SET root_hash = '{}', leaf_count = {}, created_at = time::now();",
            immune.head_digest().to_hex(),
            immune.audit_log().len()
        ))
        .await;
}

/// Load the audit chain from SurrealDB and verify it. Fail-closed: any
/// mismatch is an `Err` with the evidenced cause.
pub async fn load_and_verify_chain(
    client: &SurrealClient,
) -> Result<(Vec<crate::compliance::ComplianceAuditEntry>, Sha256Digest), String> {
    let rows = client
        .sql("SELECT * FROM audit_entry ORDER BY sequence ASC;")
        .await?;
    let mut entries = Vec::new();
    if let Some(arr) = rows.as_array() {
        for r in arr {
            let sequence = r.get("sequence").and_then(|v| v.as_u64()).unwrap_or(0);
            let action = r
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let approved = r.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
            let mark_tag = r
                .get("mark")
                .and_then(|v| v.as_str())
                .unwrap_or("[verified]");
            let mark = crate::roles::EpistemicMark::from_tag(mark_tag)
                .ok_or_else(|| format!("audit_entry {}: unknown mark {}", sequence, mark_tag))?;
            let prev_digest =
                Sha256Digest::from_hex(r.get("prev_digest").and_then(|v| v.as_str()).unwrap_or(""))
                    .ok_or_else(|| format!("audit_entry {}: bad prev_digest", sequence))?;
            let entry_digest = Sha256Digest::from_hex(
                r.get("entry_digest").and_then(|v| v.as_str()).unwrap_or(""),
            )
            .ok_or_else(|| format!("audit_entry {}: bad entry_digest", sequence))?;
            entries.push(crate::compliance::ComplianceAuditEntry {
                sequence,
                action,
                approved,
                mark,
                prev_digest,
                entry_digest,
            });
        }
    }
    let head = entries
        .last()
        .map(|e| e.entry_digest)
        .unwrap_or(Sha256Digest([0u8; 32]));
    crate::compliance::verify_audit_chain(&entries, &head)
        .map_err(|e| format!("startup chain verification failed: {}", e))?;
    Ok((entries, head))
}

/// Schema, created from day one (RG-AH vocabulary only).
pub const SCHEMA_STATEMENTS: &[&str] = &[
    "DEFINE NAMESPACE IF NOT EXISTS ares;",
    "DEFINE DATABASE IF NOT EXISTS ares;",
    "DEFINE TABLE IF NOT EXISTS thread SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS title ON thread TYPE string;",
    "DEFINE FIELD IF NOT EXISTS created_at ON thread TYPE datetime DEFAULT time::now();",
    "DEFINE TABLE IF NOT EXISTS message SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS thread ON message TYPE string;",
    "DEFINE FIELD IF NOT EXISTS role ON message TYPE string;",
    "DEFINE FIELD IF NOT EXISTS content ON message TYPE string;",
    "DEFINE FIELD IF NOT EXISTS created_at ON message TYPE datetime DEFAULT time::now();",
    "DEFINE TABLE IF NOT EXISTS run SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS thread ON run TYPE string;",
    "DEFINE FIELD IF NOT EXISTS status ON run TYPE string;",
    "DEFINE FIELD IF NOT EXISTS model_seat ON run TYPE string;",
    "DEFINE FIELD IF NOT EXISTS created_at ON run TYPE datetime DEFAULT time::now();",
    "DEFINE TABLE IF NOT EXISTS run_step SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS run ON run_step TYPE string;",
    "DEFINE FIELD IF NOT EXISTS label ON run_step TYPE string;",
    "DEFINE FIELD IF NOT EXISTS status ON run_step TYPE string;",
    "DEFINE TABLE IF NOT EXISTS tool_call SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS run ON tool_call TYPE string;",
    "DEFINE FIELD IF NOT EXISTS name ON tool_call TYPE string;",
    "DEFINE TABLE IF NOT EXISTS receipt SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS run ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS what_ran ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS route ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS status ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS last_output ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS expected ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS evidenced_cause ON receipt TYPE string;",
    "DEFINE FIELD IF NOT EXISTS unblocker ON receipt TYPE string;",
    "DEFINE TABLE IF NOT EXISTS atom SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS payload ON atom TYPE string;",
    "DEFINE FIELD IF NOT EXISTS home_role ON atom TYPE string;",
    "DEFINE TABLE IF NOT EXISTS packet SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS thread ON packet TYPE string;",
    "DEFINE FIELD IF NOT EXISTS body ON packet TYPE string;",
    "DEFINE TABLE IF NOT EXISTS control SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS framework ON control TYPE string;",
    "DEFINE FIELD IF NOT EXISTS article ON control TYPE string;",
    "DEFINE FIELD IF NOT EXISTS title ON control TYPE string;",
    "DEFINE FIELD IF NOT EXISTS required_invariants ON control TYPE string;",
    "DEFINE FIELD IF NOT EXISTS evidence_ref ON control TYPE string;",
    "DEFINE FIELD IF NOT EXISTS status ON control TYPE string;",
    "DEFINE TABLE IF NOT EXISTS permit SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS sequence ON permit TYPE int;",
    "DEFINE FIELD IF NOT EXISTS action_summary ON permit TYPE string;",
    "DEFINE FIELD IF NOT EXISTS epistemic_mark ON permit TYPE string;",
    "DEFINE FIELD IF NOT EXISTS chain_digest ON permit TYPE string;",
    "DEFINE FIELD IF NOT EXISTS granted_at ON permit TYPE datetime DEFAULT time::now();",
    "DEFINE TABLE IF NOT EXISTS audit_entry SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS sequence ON audit_entry TYPE int;",
    "DEFINE FIELD IF NOT EXISTS action ON audit_entry TYPE string;",
    "DEFINE FIELD IF NOT EXISTS approved ON audit_entry TYPE bool;",
    "DEFINE FIELD IF NOT EXISTS mark ON audit_entry TYPE string;",
    "DEFINE FIELD IF NOT EXISTS prev_digest ON audit_entry TYPE string;",
    "DEFINE FIELD IF NOT EXISTS entry_digest ON audit_entry TYPE string;",
    "DEFINE TABLE IF NOT EXISTS merkle_root SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS root_hash ON merkle_root TYPE string;",
    "DEFINE FIELD IF NOT EXISTS leaf_count ON merkle_root TYPE int;",
    "DEFINE FIELD IF NOT EXISTS created_at ON merkle_root TYPE datetime DEFAULT time::now();",
    "DEFINE TABLE IF NOT EXISTS erasure SCHEMAFULL;",
    "DEFINE FIELD IF NOT EXISTS atom_id ON erasure TYPE int;",
    "DEFINE FIELD IF NOT EXISTS pre_root ON erasure TYPE string;",
    "DEFINE FIELD IF NOT EXISTS post_root ON erasure TYPE string;",
    "DEFINE FIELD IF NOT EXISTS proof ON erasure TYPE string;",
    "DEFINE FIELD IF NOT EXISTS residual_count ON erasure TYPE int;",
    "DEFINE FIELD IF NOT EXISTS verified ON erasure TYPE bool;",
    "DEFINE FIELD IF NOT EXISTS created_at ON erasure TYPE datetime DEFAULT time::now();",
];

pub async fn apply_schema(client: &SurrealClient) -> Result<(), String> {
    for stmt in SCHEMA_STATEMENTS {
        // DEFINE NAMESPACE/DATABASE must run without ns/db headers in some
        // versions; the /sql endpoint with ns/db set accepts DEFINE TABLE.
        // We skip the first two when they error (ns/db already selected).
        if let Err(e) = client.sql(stmt).await {
            if stmt.contains("DEFINE TABLE") || stmt.contains("DEFINE FIELD") {
                return Err(format!("schema failed at {}: {}", stmt, e));
            }
        }
    }
    Ok(())
}

/// Seed the 12 statutory controls into SurrealDB (idempotent by record id).
pub async fn seed_controls(client: &SurrealClient) -> Result<(), String> {
    for rule in crate::first_principles::FIRST_PRINCIPLE_STATUTES {
        let inv = rule
            .required_invariants
            .iter()
            .map(|i| format!("{:?}", i))
            .collect::<Vec<_>>()
            .join(",");
        let stmt = format!(
            "CREATE control:{} SET framework = '{}', article = '{}', title = '{}', required_invariants = '{}', evidence_ref = 'unproven', status = 'unproven';",
            rule.id.replace('-', "_"),
            sql_escape(rule.framework.label()),
            sql_escape(rule.article),
            sql_escape(rule.title),
            sql_escape(&inv)
        );
        // Ignore "already exists" — seeding is idempotent by record id.
        let _ = client.sql(&stmt).await;
    }
    Ok(())
}

/// Default pinned weights for the skeleton: the backend's governed model
/// identity is the local seat's model id + a content fingerprint of the
/// pinned descriptor. Phase 2 replaces the descriptor with the live model
/// fingerprint at startup; the gate semantics do not change.
pub fn default_base_weights() -> FrozenBaseWeights {
    FrozenBaseWeights::freeze("ares-backend-pinned-seat", 1, vec![1.0, 0.0, 0.0, 0.0])
}

pub fn default_policy() -> OverlapPolicyMatrix {
    OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08))
}

/// Governed mission dispatch (Gate 2 + RG-AH runtime). The compliance
/// guard is the permit itself, obtained from `authorize_role_spawn`;
/// the mission runs fail-closed locally unless a live executor is wired
/// by the caller. The mission receipt is persisted to SurrealDB
/// (run/receipt rows) and anchored by the audit chain.
///
/// Governed MCP execution through the automaton, live dispatch.
/// Permit first, telemetry required, automaton transition, stdio
/// MCP child only if Admit, confidence update only after an independent
/// outcome judge,
/// persistence to the governance Surreal substrate, HyperDX
/// observation. The MCP child binary path and root come from
/// server configuration (env), never from the request — a
/// request-controlled binary path would be a code-execution hole.
/// A well-formed response is only a transport result; this path deliberately
/// marks it unverified and does not raise confidence from the child's claim.
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_mcp_execute(
    state: &Arc<AppState>,
    role: RoleId,
    mut strategy: crate::harness::automaton::StrategyState,
    envelope: crate::harness::automaton::ProposerEnvelope,
    signals: crate::harness::automaton::AuditSignals,
    now_ms: u64,
    tool: &str,
    args_json: &str,
    mcp_bin: &str,
    mcp_root: &str,
    mcp_authority: crate::harness::mcp::McpAuthority,
    mind_credentials: (String, String),
) -> Result<
    (
        CompliancePermit,
        crate::harness::mcp::McpExecution,
        crate::harness::automaton::StrategyState,
    ),
    String,
> {
    let _permit_slot = state
        .run_semaphore
        .clone()
        .acquire_owned()
        .await
        .expect("run semaphore never closes");
    let permit = dispatch_spawn(state, role)
        .await
        .map_err(|_| "permit denied".to_string())?;
    let telemetry = state
        .telemetry
        .as_ref()
        .ok_or_else(|| "live run requires HyperDX telemetry configured".to_string())?;
    let child_env = [
        ("GLOBALMIND_SURREAL_URL", mcp_authority.base.as_str()),
        ("GLOBALMIND_SURREAL_NS", mcp_authority.ns.as_str()),
        ("GLOBALMIND_SURREAL_DB", mcp_authority.db.as_str()),
        ("GLOBALMIND_SURREAL_USER", mind_credentials.0.as_str()),
        ("GLOBALMIND_SURREAL_PASS", mind_credentials.1.as_str()),
        ("GLOBALMIND_ALLOW_QUARANTINED_CORE", "0"),
    ];
    let mut transport =
        crate::harness::mcp::StdioMcpTransport::spawn(mcp_bin, mcp_root, &child_env)?;
    crate::harness::mcp::attest_mind_child(&mut transport, &mcp_authority)?;
    let execution = crate::harness::mcp::execute_via_automaton(
        &mut strategy,
        &envelope,
        &signals,
        now_ms,
        &mcp_authority,
        &mcp_authority,
        tool,
        args_json,
        &mut transport,
        false,
        false,
    )?;
    // Verified = outcome present with a valid receipt (adapter
    // enforces receipt hex + transport enforces inner status ok).
    // Confidence was already updated inside execute_via_automaton
    // on verified=true; here we only persist and observe.
    if let Some(client) = &state.surreal {
        let _ = client
            .sql(&crate::harness::automaton::confidence_create_sql(&strategy))
            .await;
        client
            .sql(&crate::harness::automaton::confidence_upsert_sql(&strategy))
            .await
            .map_err(|e| format!("persist automaton_confidence: {}", e))?;
        client
            .sql(&crate::harness::automaton::transition_insert_sql(
                &execution.transition,
            ))
            .await
            .map_err(|e| format!("persist automaton_receipt: {}", e))?;
    }
    let (ok, receipt) = match &execution.outcome {
        Some(o) => (true, Some(o.receipt_sha256.as_str())),
        None => (false, None),
    };
    let _ = telemetry
        .record_call(
            &execution.transition.receipt_digest,
            tool,
            args_json,
            execution.outcome.as_ref().map(|o| o.dt_ns).unwrap_or(0),
            ok,
            receipt,
        )
        .await;
    Ok((permit, execution, strategy))
}

pub async fn dispatch_mission(
    state: &Arc<AppState>,
    role: RoleId,
    input: crate::harness::mission::MissionInput,
    budget_j: f64,
) -> Result<(CompliancePermit, crate::harness::mission::MissionReceipt), ComplianceViolation> {
    // Bounded concurrency (doctrine): one of at most N governed runs.
    let _permit_slot = state
        .run_semaphore
        .clone()
        .acquire_owned()
        .await
        .expect("run semaphore never closes");
    let permit = dispatch_spawn(state, role).await?;
    let receipt = crate::harness::runtime::run_mission_local(input, budget_j);
    if let Some(client) = &state.surreal {
        let stmt = format!(
            "CREATE run SET thread = '{}', status = '{}', model_seat = '{}';",
            crate::surreal::sql_escape(&receipt.mission_id),
            if receipt.passed_all() {
                "passed"
            } else {
                "escalated"
            },
            "none-local-fail-closed"
        );
        let _ = client.sql(&stmt).await;
    }
    Ok((permit, receipt))
}
