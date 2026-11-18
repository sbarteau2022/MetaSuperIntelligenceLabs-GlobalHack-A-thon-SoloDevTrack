use globalmind_runtime::capability::store::CrystallizedAtlasReceipt;
use globalmind_runtime::capability::{
    CapabilityEdgeKind, CapabilityEdgeRecord, CapabilityKind, CapabilityLifecycle,
    CapabilityPayload, CapabilityScaffold, EffectClass, LiveRegistry, LiveRegistryEntry,
    MandatoryPolicyGate, ReconciliationDisqualification, deterministic_digest, digest_bytes,
    execute_planned_readonly_tool,
};
use globalmind_runtime::whitehead::WhiteheadTopology;
use globalmind_runtime::{Digest, GlobalMindStore, RecordId, StoreConfig};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct IsolatedSurreal {
    data_dir: PathBuf,
    endpoint: String,
    port: u16,
    child: Option<Child>,
    namespace: String,
    database: String,
}

impl IsolatedSurreal {
    fn start(tag: &str) -> Self {
        let port = reserve_free_port();
        let data_dir = std::env::temp_dir().join(format!(
            "globalmind-capability-atlas-{tag}-{}-{port}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data_dir);
        fs::create_dir_all(&data_dir).expect("create temp surreal dir");
        let endpoint = format!("http://127.0.0.1:{port}");
        let mut instance = Self {
            data_dir,
            endpoint,
            port,
            child: None,
            namespace: "hypermind_capability_test".into(),
            database: "capability_atlas_isolated".into(),
        };
        instance.spawn_process();
        instance
    }

    fn spawn_process(&mut self) {
        assert_ne!(
            self.port, 8000,
            "isolated SurrealDB must never touch port 8000"
        );
        let db_arg = format!("surrealkv://{}", self.data_dir.display());
        let bind_arg = format!("127.0.0.1:{}", self.port);
        let child = Command::new("surreal")
            .args([
                "start", "--bind", &bind_arg, "--user", "root", "--pass", "root", "--log", "warn",
                &db_arg,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated surreal server");
        self.child = Some(child);
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(40));
        }
        panic!(
            "isolated SurrealDB did not become ready at {}",
            self.endpoint
        );
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn restart(&mut self) -> u32 {
        self.stop();
        self.spawn_process();
        self.child.as_ref().expect("restarted child").id()
    }

    fn config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: self.endpoint.clone(),
            namespace: self.namespace.clone(),
            database: self.database.clone(),
            username: Some("root".into()),
            password: Some("root".into()),
            allow_quarantined_core: false,
        }
    }

    fn apply_sql(&self, sql: &str) {
        let address = format!("127.0.0.1:{}", self.port);
        let body = format!("USE NS {} DB {}; {sql}", self.namespace, self.database);
        let req = format!(
            "POST /sql HTTP/1.1\r\nHost: {address}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic cm9vdDpyb290\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let mut stream = TcpStream::connect(&address).expect("connect surreal");
        stream.write_all(req.as_bytes()).expect("write sql");
        let mut resp = String::new();
        stream.read_to_string(&mut resp).expect("read response");
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "Surreal HTTP error: {resp}"
        );
        assert!(
            !resp.contains("\"status\":\"ERR\""),
            "Surreal statement error: {resp}"
        );
    }

    fn apply_sql_expect_err(&self, sql: &str) -> String {
        let address = format!("127.0.0.1:{}", self.port);
        let body = format!("USE NS {} DB {}; {sql}", self.namespace, self.database);
        let req = format!(
            "POST /sql HTTP/1.1\r\nHost: {address}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic cm9vdDpyb290\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let mut stream = TcpStream::connect(&address).expect("connect surreal");
        stream.write_all(req.as_bytes()).expect("write sql");
        let mut resp = String::new();
        stream.read_to_string(&mut resp).expect("read response");
        assert!(
            resp.contains("\"status\":\"ERR\""),
            "expected SurrealDB statement error, got: {resp}"
        );
        resp
    }
}

impl Drop for IsolatedSurreal {
    fn drop(&mut self) {
        self.stop();
        let _ = fs::remove_dir_all(&self.data_dir);
    }
}

fn reserve_free_port() -> u16 {
    loop {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().expect("local addr").port();
        if port != 8000 {
            return port;
        }
    }
}

fn apply_all_migrations_through_0012(surreal: &IsolatedSurreal) {
    let schema_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../hypermind-nested-kv/schema");
    for file in [
        "0001_hypermind_graph.surql",
        "0003_nested_hyperbolic_kv.surql",
        "0004_transformer_kv_tensor_pages.surql",
        "0005_universal_atom_lineage.surql",
        "0006_immutable_atom_address.surql",
        "0007_globalmind_distillation_trajectories.surql",
        "0008_superposition_holding.surql",
        "0009_immutable_tensor_kv.surql",
        "0010_native_distillation_ingress.surql",
        "0011_atomic_trajectory_cells.surql",
        "0012_capability_atlas.surql",
    ] {
        let sql = fs::read_to_string(schema_dir.join(file)).expect("read migration file");
        surreal.apply_sql(&sql);
    }
    surreal.apply_sql(
        "UPSERT workspace:globalmind_cap_test CONTENT { slug: 'globalmind_cap_test', created_at: time::now() };",
    );
}

#[allow(clippy::too_many_arguments)]
fn make_scaffold(
    topology: &WhiteheadTopology,
    workspace: &RecordId,
    id_slug: &str,
    well_ordinal: u8,
    kind: CapabilityKind,
    canonical_name: &str,
    summary: &str,
    version: &str,
    payload: CapabilityPayload,
    lifecycle: CapabilityLifecycle,
    effect_class: EffectClass,
    live_registry_id: &str,
    expected_utility: f64,
    resource_cost: f64,
    prediction_error: f64,
    benched_reason: Option<String>,
) -> CapabilityScaffold {
    let arg_digest = deterministic_digest(&format!("arg:{canonical_name}:{version}"));
    let res_digest = deterministic_digest(&format!("res:{canonical_name}:{version}"));
    let content_digest = CapabilityScaffold::compute_content_digest(
        kind,
        canonical_name,
        summary,
        version,
        &payload,
        &arg_digest,
        &res_digest,
    );
    let (h3_lorentz, t2_phase, well_id) = CapabilityScaffold::derive_model_neutral_coordinates(
        topology,
        well_ordinal,
        &content_digest,
    )
    .expect("derive coordinates");

    CapabilityScaffold {
        id: RecordId::parse(format!("prof_{id_slug}")).unwrap(),
        atom_id: RecordId::parse(format!("atom_{id_slug}")).unwrap(),
        address_id: RecordId::parse(format!("addr_{id_slug}")).unwrap(),
        membership_id: RecordId::parse(format!("memb_{id_slug}")).unwrap(),
        workspace: workspace.clone(),
        geometry_space: topology.geometry_space_id.clone(),
        well: well_id,
        kind,
        canonical_name: canonical_name.into(),
        summary: summary.into(),
        version: version.into(),
        payload,
        content_digest,
        source_digest: deterministic_digest(&format!("audit_source:{id_slug}")),
        configuration_digest: deterministic_digest(&format!("cfg:{canonical_name}:{version}")),
        h3_lorentz,
        t2_phase,
        lifecycle,
        admission_receipt_digest: deterministic_digest(&format!("adm:{id_slug}")),
        judge_receipt_digest: deterministic_digest(&format!("judge:{id_slug}")),
        replay_receipt_digest: deterministic_digest(&format!("replay:{id_slug}")),
        crystallization_receipt_digest: deterministic_digest(&format!("crys:{id_slug}")),
        argument_schema_digest: arg_digest,
        result_schema_digest: res_digest,
        effect_class,
        permission_classes: vec!["surreal.schema.read".into()],
        credential_classes: vec!["loopback_session_token".into()],
        platform_requirements: vec!["macos-aarch64".into()],
        runtime_requirements: vec!["globalmind-runtime>=0.1.0".into()],
        version_requirement: ">=1.0.0".into(),
        timeout_ms: 5_000,
        max_retries: 1,
        idempotent: true,
        rollback_policy: "read_only_noop".into(),
        max_memory_bytes: 32 * 1024 * 1024,
        health_check_id: format!("health://{canonical_name}"),
        live_registry_id: live_registry_id.into(),
        component_receipt_digest: Some(deterministic_digest(&format!("comp:{id_slug}"))),
        integration_receipt_digest: Some(deterministic_digest(&format!("int:{id_slug}"))),
        live_receipt_digest: Some(deterministic_digest(&format!("live:{id_slug}"))),
        incompatible_profiles: Vec::new(),
        benched_reason,
        expected_utility,
        resource_cost,
        prediction_error,
    }
}

fn make_edge(
    workspace: &RecordId,
    id_slug: &str,
    from: &CapabilityScaffold,
    to: &CapabilityScaffold,
    kind: CapabilityEdgeKind,
    evidence_receipt: Option<&CapabilityScaffold>,
) -> CapabilityEdgeRecord {
    CapabilityEdgeRecord {
        id: RecordId::parse(format!("cedge_{id_slug}")).unwrap(),
        workspace: workspace.clone(),
        from_profile: from.id.clone(),
        to_profile: to.id.clone(),
        from_atom: from.atom_id.clone(),
        to_atom: to.atom_id.clone(),
        kind,
        confidence: 0.98,
        evidence_receipt: evidence_receipt.map(|e| e.id.clone()),
        evidence_receipt_digest: evidence_receipt
            .map(|e| e.content_digest.clone())
            .unwrap_or_else(|| deterministic_digest(&format!("ev:{id_slug}"))),
        producer: "capability_atlas_scaffolder_v1".into(),
        revision: 1,
        valid_from_unix_ms: 1_762_000_000_000,
        valid_to_unix_ms: Some(1_893_456_000_000),
        compatibility_digest: deterministic_digest(&format!(
            "compat:{}:{}:{}",
            from.id.as_str(),
            to.id.as_str(),
            kind.as_str()
        )),
        crystallization_receipt_digest: deterministic_digest(&format!("edge_crys:{id_slug}")),
    }
}

/// Builds the complete vertical slice containing all 12 atom kinds, all 13
/// edge kinds, two solution families, one qualified runbook, one live MCP/tool,
/// one unavailable/conflicting tool, one mandatory rule, one evidence-backed
/// fallback, one superseded tool version, and one intentional cycle.
fn build_vertical_slice(
    topology: &WhiteheadTopology,
    workspace: &RecordId,
) -> (
    Vec<CapabilityScaffold>,
    Vec<CapabilityEdgeRecord>,
    LiveRegistry,
) {
    // 1. problem_pattern (Well 6: L7_ConvergentLogic)
    let problem = make_scaffold(
        topology,
        workspace,
        "problem_schema_audit",
        6,
        CapabilityKind::ProblemPattern,
        "verify_surreal_capability_contract",
        "Verify isolated SurrealDB 3.3.5 READONLY capability contract before forward-pass smoke",
        "1.0.0",
        CapabilityPayload::ProblemPattern {
            symptoms: "Unverified schema tables or mutable fields before forward-pass smoke".into(),
            constraints:
                "Read-only inspection on isolated SurrealDB; zero JSON/Python in model I/O".into(),
            success_condition: "All capability tables verified READONLY with signed binary receipt"
                .into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "problem://verify_surreal_capability_contract@1.0.0",
        0.95,
        0.05,
        0.01,
        None,
    );

    // 2a. solution_family #1 (Well 6: nearest to problem_pattern)
    let family_primary = make_scaffold(
        topology,
        workspace,
        "family_native_surreal_verifier",
        6,
        CapabilityKind::SolutionFamily,
        "native_surreal_contract_verification",
        "Inspect live SurrealDB table definitions through native binary-enveloped read-only probes",
        "1.0.0",
        CapabilityPayload::SolutionFamily {
            scope: "SurrealDB 3.3.5 INFO FOR TABLE and immutable field verification".into(),
            exclusions: "Never mutates rows or touches quarantined hypermind/core".into(),
            selection_criteria:
                "Select when task requires read-only schema and contract verification".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "family://native_surreal_contract_verification@1.0.0",
        0.96,
        0.06,
        0.02,
        None,
    );

    // 2b. solution_family #2 (Well 3: alternative distant family)
    let family_secondary = make_scaffold(
        topology,
        workspace,
        "family_offline_migration_hash",
        3,
        CapabilityKind::SolutionFamily,
        "offline_migration_file_hashing",
        "Rehash static .surql migration files on disk without querying a live database",
        "1.0.0",
        CapabilityPayload::SolutionFamily {
            scope: "Static filesystem SHA-256 verification of .surql files".into(),
            exclusions: "Cannot verify runtime table state inside SurrealDB".into(),
            selection_criteria: "Select only when no SurrealDB instance is running".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "family://offline_migration_file_hashing@1.0.0",
        0.72,
        0.02,
        0.08,
        None,
    );

    // 3. capability (Well 6)
    let capability = make_scaffold(
        topology,
        workspace,
        "cap_inspect_schema_contract",
        6,
        CapabilityKind::Capability,
        "inspect_readonly_schema_contract",
        "Query SurrealDB table metadata and verify READONLY field invariants",
        "1.0.0",
        CapabilityPayload::Capability {
            input_output_contract:
                "Binary GMBI table-list request -> Binary GMBI verified digest receipt".into(),
            effects: "Read-only metadata inspection".into(),
            failure_modes: "Missing table, non-READONLY field, or endpoint unreachable".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "capability://inspect_readonly_schema_contract@1.0.0",
        0.97,
        0.05,
        0.01,
        None,
    );

    // 4. runbook (Well 6)
    let runbook = make_scaffold(
        topology,
        workspace,
        "runbook_isolated_schema_verify",
        6,
        CapabilityKind::Runbook,
        "isolated_surreal_contract_runbook",
        "Ordered read-only procedure to verify 0001..0012 schema tables and record utility receipt",
        "1.0.0",
        CapabilityPayload::Runbook {
            steps: "1. Reconcile live tool; 2. Enforce policy gate; 3. Query INFO FOR TABLE; 4. Record receipt".into(),
            stop_rollback_rules: "Abort immediately on any non-READONLY field or quarantined DB target".into(),
            evidence_required: "SHA-256 digest of verified table definitions".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "runbook://isolated_surreal_contract_runbook@1.0.0",
        0.95,
        0.05,
        0.02,
        None,
    );

    // 5. mcp_server (Well 6)
    let mcp_server = make_scaffold(
        topology,
        workspace,
        "mcp_globalmind_contract",
        6,
        CapabilityKind::McpServer,
        "globalmind_contract_mcp_provider",
        "Local loopback provider exposing read-only SurrealDB contract inspection tools",
        "1.0.0",
        CapabilityPayload::McpServer {
            server_identity: "mcp://globalmind-contract-provider".into(),
            transport: "native_loopback_binary".into(),
            advertised_contracts: "surreal_contract_verify@1.0.0, surreal_readonly_snapshot@1.0.0"
                .into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract-provider@1.0.0",
        0.96,
        0.04,
        0.01,
        None,
    );

    // 6. plugin (Well 6)
    let plugin = make_scaffold(
        topology,
        workspace,
        "plugin_schema_inspector",
        6,
        CapabilityKind::Plugin,
        "schema_inspector_extension",
        "Independently loadable read-only schema assertion plugin",
        "1.0.0",
        CapabilityPayload::Plugin {
            loader_contract: "native_rust_cdylib_v1".into(),
            isolation_behavior: "sandboxed_read_only_memory".into(),
            unload_behavior: "deterministic_drop_zero_handles".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "plugin://schema_inspector_extension@1.0.0",
        0.90,
        0.06,
        0.03,
        None,
    );

    // 7a. Primary live read-only tool v1.0.0 (Well 6)
    let tool_primary = make_scaffold(
        topology,
        workspace,
        "tool_surreal_contract_verify",
        6,
        CapabilityKind::Tool,
        "surreal_contract_verify",
        "Callable read-only operation verifying SurrealDB 3.3.5 READONLY table invariants",
        "1.0.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI SchemaVerifyArgs -> GMBI SchemaVerifyResult".into(),
            side_effects: "none (read_only)".into(),
            timeout_spec: "5000ms hard deadline, 1 retry".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract/surreal_contract_verify@1.0.0",
        0.98,
        0.05,
        0.01,
        None,
    );

    // 7b. Older superseded version of the primary tool v0.9.0 (Well 6)
    let tool_superseded = make_scaffold(
        topology,
        workspace,
        "tool_surreal_contract_verify_v090",
        6,
        CapabilityKind::Tool,
        "surreal_contract_verify",
        "Legacy 0.9.0 read-only contract verifier superseded by 1.0.0",
        "0.9.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI SchemaVerifyArgsV0 -> GMBI SchemaVerifyResultV0".into(),
            side_effects: "none (read_only)".into(),
            timeout_spec: "5000ms hard deadline".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract/surreal_contract_verify@0.9.0",
        0.99,
        0.01,
        0.01,
        None,
    );

    // 7c. Unavailable & conflicting tool (Well 6)
    let tool_conflicting = make_scaffold(
        topology,
        workspace,
        "tool_conflicting_direct_mutate",
        6,
        CapabilityKind::Tool,
        "surreal_direct_schema_mutate",
        "Conflicting and unregistered tool that attempts direct schema mutation",
        "1.0.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI RawSqlMutate -> GMBI MutateAck".into(),
            side_effects: "external_write (mutates schema)".into(),
            timeout_spec: "10000ms".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ExternalWrite,
        "mcp://unregistered/surreal_direct_schema_mutate@1.0.0",
        0.99,
        0.02,
        0.01,
        None,
    );

    // 7d. Benched tool (Well 6)
    let tool_benched = make_scaffold(
        topology,
        workspace,
        "tool_benched_experimental_probe",
        6,
        CapabilityKind::Tool,
        "experimental_unverified_probe",
        "Benched tool excluded before planning",
        "0.1.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI ProbeArgs -> GMBI ProbeResult".into(),
            side_effects: "none".into(),
            timeout_spec: "2000ms".into(),
        },
        CapabilityLifecycle::Benched,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract/experimental_unverified_probe@0.1.0",
        0.99,
        0.01,
        0.01,
        Some("Benched pending live integration qualification".into()),
    );

    // 7e. Evidence-backed fallback tool (Well 6)
    let tool_fallback = make_scaffold(
        topology,
        workspace,
        "tool_fallback_readonly_snapshot",
        6,
        CapabilityKind::Tool,
        "surreal_readonly_snapshot",
        "Evidence-backed read-only fallback tool when primary verifier is unavailable",
        "1.0.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI SnapshotInspectArgs -> GMBI SnapshotInspectResult".into(),
            side_effects: "none (read_only)".into(),
            timeout_spec: "5000ms hard deadline".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract/surreal_readonly_snapshot@1.0.0",
        0.91,
        0.07,
        0.02,
        None,
    );

    // 8. skill (Well 6)
    let skill = make_scaffold(
        topology,
        workspace,
        "skill_schema_triage",
        6,
        CapabilityKind::Skill,
        "schema_invariance_triage_skill",
        "Reusable reasoning procedure for auditing SurrealDB schema invariants",
        "1.0.0",
        CapabilityPayload::Skill {
            trigger: "Before any model forward-pass smoke or distillation ingress".into(),
            required_references: "0012_capability_atlas.surql, GLOBALMIND-HANDOFF-2026-11-01.md"
                .into(),
            completion_gates: "All 12 atom kinds and 13 edge kinds verified READONLY".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "skill://schema_invariance_triage_skill@1.0.0",
        0.93,
        0.04,
        0.02,
        None,
    );

    // 9. hook (Well 6)
    let hook = make_scaffold(
        topology,
        workspace,
        "hook_pre_smoke_gate",
        6,
        CapabilityKind::Hook,
        "pre_forward_smoke_verification_hook",
        "Event-triggered pre-smoke gate ensuring capability contract is verified",
        "1.0.0",
        CapabilityPayload::Hook {
            event: "on_pre_forward_pass_smoke".into(),
            ordering: "before_all_model_execution".into(),
            idempotency_and_failure_policy: "idempotent; fail_closed_on_error".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "hook://pre_forward_smoke_verification_hook@1.0.0",
        0.92,
        0.03,
        0.01,
        None,
    );

    // 10. policy_rule (Well 6)
    let policy_rule = make_scaffold(
        topology,
        workspace,
        "rule_mandatory_binary_io_and_readonly",
        6,
        CapabilityKind::PolicyRule,
        "mandatory_binary_io_and_readonly_gate",
        "Harness-enforced rule forbidding JSON/Python in model I/O and mutations during smoke",
        "1.0.0",
        CapabilityPayload::PolicyRule {
            scope: "all_planning_and_execution_boundaries".into(),
            severity: "critical_fail_closed".into(),
            enforcement_point: "MandatoryPolicyGate::enforce_call".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "policy://mandatory_binary_io_and_readonly_gate@1.0.0",
        1.0,
        0.01,
        0.0,
        None,
    );

    // 11. evidence_receipt (Well 6)
    let evidence_receipt = make_scaffold(
        topology,
        workspace,
        "evidence_schema_contract_qualified",
        6,
        CapabilityKind::EvidenceReceipt,
        "surreal_contract_verify_qualification_receipt",
        "Immutable live qualification receipt for surreal_contract_verify@1.0.0",
        "1.0.0",
        CapabilityPayload::EvidenceReceipt {
            subject_config_runtime_digests:
                "subject=surreal_contract_verify@1.0.0;runtime=surreal-3.3.5".into(),
            tier: "live_isolated_surrealdb".into(),
            result: "passed_all_readonly_and_replay_gates".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "evidence://surreal_contract_verify_qualification_receipt@1.0.0",
        0.99,
        0.01,
        0.0,
        None,
    );

    // 12. failure_pattern (Well 6)
    let failure_pattern = make_scaffold(
        topology,
        workspace,
        "failure_untyped_edge_and_mutable_field",
        6,
        CapabilityKind::FailurePattern,
        "untyped_edge_or_mutable_schema_drift",
        "Known failure pattern when untyped 'related' edges or non-READONLY fields are admitted",
        "1.0.0",
        CapabilityPayload::FailurePattern {
            signature: "SurrealDB ERR on untyped edge or missing READONLY modifier".into(),
            cause: "Direct unvalidated schema write or legacy migration drift".into(),
            containment: "Abort transaction and bench offending tool".into(),
            recovery: "Route through surreal_contract_verify@1.0.0 or fallback snapshot".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "failure://untyped_edge_or_mutable_schema_drift@1.0.0",
        0.90,
        0.01,
        0.0,
        None,
    );

    // All 13 required edge kinds:
    let edges = vec![
        // 1. solves: family_primary -> problem
        make_edge(
            workspace,
            "solves_primary",
            &family_primary,
            &problem,
            CapabilityEdgeKind::Solves,
            Some(&evidence_receipt),
        ),
        // solves: family_secondary -> problem
        make_edge(
            workspace,
            "solves_secondary",
            &family_secondary,
            &problem,
            CapabilityEdgeKind::Solves,
            Some(&evidence_receipt),
        ),
        // 2. specializes: capability -> family_primary
        make_edge(
            workspace,
            "specializes_cap_family",
            &capability,
            &family_primary,
            CapabilityEdgeKind::Specializes,
            Some(&evidence_receipt),
        ),
        // 3. implements: tool_primary -> capability
        make_edge(
            workspace,
            "implements_tool_primary",
            &tool_primary,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: tool_superseded -> capability
        make_edge(
            workspace,
            "implements_tool_superseded",
            &tool_superseded,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: tool_conflicting -> capability
        make_edge(
            workspace,
            "implements_tool_conflicting",
            &tool_conflicting,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: tool_benched -> capability
        make_edge(
            workspace,
            "implements_tool_benched",
            &tool_benched,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: runbook -> capability
        make_edge(
            workspace,
            "implements_runbook",
            &runbook,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: skill -> capability
        make_edge(
            workspace,
            "implements_skill",
            &skill,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // implements: plugin -> capability
        make_edge(
            workspace,
            "implements_plugin",
            &plugin,
            &capability,
            CapabilityEdgeKind::Implements,
            Some(&evidence_receipt),
        ),
        // 4. exposes: mcp_server -> tool_primary
        make_edge(
            workspace,
            "exposes_mcp_primary",
            &mcp_server,
            &tool_primary,
            CapabilityEdgeKind::Exposes,
            Some(&evidence_receipt),
        ),
        // exposes: plugin -> tool_fallback
        make_edge(
            workspace,
            "exposes_plugin_fallback",
            &plugin,
            &tool_fallback,
            CapabilityEdgeKind::Exposes,
            Some(&evidence_receipt),
        ),
        // 5. requires: skill -> hook AND hook -> skill (intentional Atlas cycle!)
        make_edge(
            workspace,
            "requires_skill_hook",
            &skill,
            &hook,
            CapabilityEdgeKind::Requires,
            Some(&evidence_receipt),
        ),
        make_edge(
            workspace,
            "requires_hook_skill_cycle",
            &hook,
            &skill,
            CapabilityEdgeKind::Requires,
            Some(&evidence_receipt),
        ),
        // 6. composes_with: tool_primary -> runbook
        make_edge(
            workspace,
            "composes_tool_runbook",
            &tool_primary,
            &runbook,
            CapabilityEdgeKind::ComposesWith,
            Some(&evidence_receipt),
        ),
        // 7. conflicts_with: tool_conflicting -> policy_rule
        make_edge(
            workspace,
            "conflicts_mutator_policy",
            &tool_conflicting,
            &policy_rule,
            CapabilityEdgeKind::ConflictsWith,
            Some(&evidence_receipt),
        ),
        // 8. fallback_for: tool_fallback -> tool_primary
        make_edge(
            workspace,
            "fallback_snapshot_for_primary",
            &tool_fallback,
            &tool_primary,
            CapabilityEdgeKind::FallbackFor,
            Some(&evidence_receipt),
        ),
        // 9. enforced_by: capability -> policy_rule
        make_edge(
            workspace,
            "enforced_by_mandatory_rule",
            &capability,
            &policy_rule,
            CapabilityEdgeKind::EnforcedBy,
            Some(&evidence_receipt),
        ),
        // 10. triggered_by: hook -> capability
        make_edge(
            workspace,
            "triggered_by_pre_smoke",
            &hook,
            &capability,
            CapabilityEdgeKind::TriggeredBy,
            Some(&evidence_receipt),
        ),
        // 11. verified_by: tool_primary -> evidence_receipt
        make_edge(
            workspace,
            "verified_by_live_receipt",
            &tool_primary,
            &evidence_receipt,
            CapabilityEdgeKind::VerifiedBy,
            Some(&evidence_receipt),
        ),
        // 12. failed_on: tool_conflicting -> failure_pattern
        make_edge(
            workspace,
            "failed_on_mutation_pattern",
            &tool_conflicting,
            &failure_pattern,
            CapabilityEdgeKind::FailedOn,
            Some(&evidence_receipt),
        ),
        // 13. supersedes: tool_primary (1.0.0) -> tool_superseded (0.9.0)
        make_edge(
            workspace,
            "supersedes_v100_over_v090",
            &tool_primary,
            &tool_superseded,
            CapabilityEdgeKind::Supersedes,
            Some(&evidence_receipt),
        ),
    ];

    let mut live_registry = LiveRegistry::default();
    for live_prof in [
        &runbook,
        &mcp_server,
        &plugin,
        &tool_primary,
        &tool_superseded,
        &tool_fallback,
        &skill,
        &hook,
    ] {
        live_registry.register(LiveRegistryEntry {
            live_registry_id: live_prof.live_registry_id.clone(),
            installed: true,
            healthy: true,
            version: live_prof.version.clone(),
            argument_schema_digest: live_prof.argument_schema_digest.clone(),
            result_schema_digest: live_prof.result_schema_digest.clone(),
            configuration_digest: live_prof.configuration_digest.clone(),
            granted_permissions: ["surreal.schema.read".to_string()].into_iter().collect(),
            available_credential_classes: ["loopback_session_token".to_string()]
                .into_iter()
                .collect(),
        });
    }

    let profiles = vec![
        problem,
        family_primary,
        family_secondary,
        capability,
        runbook,
        mcp_server,
        plugin,
        tool_primary,
        tool_superseded,
        tool_conflicting,
        tool_benched,
        tool_fallback,
        skill,
        hook,
        policy_rule,
        evidence_receipt,
        failure_pattern,
    ];

    (profiles, edges, live_registry)
}

#[test]
fn gate1_schema_0012_twelve_kinds_thirteen_edges_and_fail_closed_invariants() {
    let surreal = IsolatedSurreal::start("gate1");
    apply_all_migrations_through_0012(&surreal);

    let store = GlobalMindStore::connect(surreal.config()).expect("connect store");
    store.verify_contract().expect("verify 0007..0011 contract");
    store
        .verify_capability_atlas_contract()
        .expect("verify 0012 capability atlas contract");

    let workspace = RecordId::parse("globalmind_cap_test").unwrap();
    let topology = WhiteheadTopology::canonical(workspace.clone()).expect("topology");
    store
        .admit_whitehead_topology(&topology)
        .expect("admit whitehead topology");

    let (profiles, edges, _registry) = build_vertical_slice(&topology, &workspace);

    // Prove all 12 CapabilityKind variants and all 13 CapabilityEdgeKind variants are present.
    let present_kinds: BTreeSet<CapabilityKind> = profiles.iter().map(|p| p.kind).collect();
    assert_eq!(present_kinds.len(), 12, "all 12 atom kinds must be present");
    for required_kind in CapabilityKind::ALL {
        assert!(present_kinds.contains(&required_kind));
    }

    let present_edge_kinds: BTreeSet<CapabilityEdgeKind> = edges.iter().map(|e| e.kind).collect();
    assert_eq!(
        present_edge_kinds.len(),
        13,
        "all 13 edge kinds must be present"
    );
    for required_edge_kind in CapabilityEdgeKind::ALL {
        assert!(present_edge_kinds.contains(&required_edge_kind));
    }

    let receipt: CrystallizedAtlasReceipt = store
        .scaffold_capability_bundle(&workspace, &topology, &profiles, &edges)
        .expect("scaffold complete vertical slice");
    assert_eq!(receipt.profile_count, profiles.len());
    assert_eq!(receipt.edge_count, edges.len());

    // 1. Prove untyped "related" edge fails closed in Rust and SurrealDB.
    assert!(CapabilityEdgeKind::parse("related").is_err());
    let untyped_err = surreal.apply_sql_expect_err(
        "RELATE capability_profile:prof_family_native_surreal_verifier->capability_edge:bad_related->capability_profile:prof_problem_schema_audit CONTENT { \
           workspace: workspace:globalmind_cap_test, \
           from_atom: atom:atom_family_native_surreal_verifier, \
           to_atom: atom:atom_problem_schema_audit, \
           kind: 'related', \
           confidence: 0.5, \
           evidence_receipt: NONE, \
           evidence_receipt_digest: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', \
           producer: 'test', \
           revision: 1, \
           valid_from: time::now(), \
           valid_to: NONE, \
           compatibility_digest: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', \
           edge_envelope_bytes: b\"474D4345\", \
           crystallization_receipt_digest: 'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc', \
           created_at: time::now() \
         };",
    );
    assert!(!untyped_err.is_empty());

    // 2. Prove overwrite attempt on READONLY capability_profile fails closed.
    let overwrite_err = surreal.apply_sql_expect_err(
        "UPDATE capability_profile:prof_tool_surreal_contract_verify SET version = '9.9.9';",
    );
    assert!(!overwrite_err.is_empty());

    // 3. Prove duplicate (workspace, atom) profile attachment fails closed.
    let mut dup_profile = profiles[0].clone();
    dup_profile.id = RecordId::parse("prof_duplicate_on_same_atom").unwrap();
    assert!(
        store
            .scaffold_capability_bundle(&workspace, &topology, &[dup_profile], &[])
            .is_err(),
        "attaching a second capability_profile to the same atom must fail closed"
    );

    // 4. Prove secrets, live tokens, and K/V tensor references fail closed.
    let mut secret_profile = profiles[0].clone();
    secret_profile.summary = "contains live bearer token sk-1234567890abcdef".into();
    assert!(secret_profile.validate().is_err());

    let mut cred_value_profile = profiles[0].clone();
    cred_value_profile.credential_classes = vec!["api_key=real_secret_value".into()];
    assert!(cred_value_profile.validate().is_err());

    let mut kv_tensor_profile = profiles[0].clone();
    kv_tensor_profile.summary = "embeds transformer_kv_page bytes into capability".into();
    assert!(kv_tensor_profile.validate().is_err());

    println!(
        "CAPABILITY_GATE1_RECEIPT profiles={} edges={} bundle_digest={} replay_digest={}",
        receipt.profile_count,
        receipt.edge_count,
        receipt.bundle_digest.as_str(),
        receipt.replay_receipt_digest.as_str()
    );
}

#[test]
fn gate2_two_hop_retrieval_live_reconciliation_source_invariance_and_acyclic_dag() {
    let surreal = IsolatedSurreal::start("gate2");
    apply_all_migrations_through_0012(&surreal);

    let store = GlobalMindStore::connect(surreal.config()).expect("connect store");
    let workspace = RecordId::parse("globalmind_cap_test").unwrap();
    let topology = WhiteheadTopology::canonical(workspace.clone()).expect("topology");
    store
        .admit_whitehead_topology(&topology)
        .expect("admit whitehead topology");

    let (profiles, edges, mut live_registry) = build_vertical_slice(&topology, &workspace);
    store
        .scaffold_capability_bundle(&workspace, &topology, &profiles, &edges)
        .expect("scaffold vertical slice");

    let snapshot = store
        .load_capability_atlas_snapshot(&workspace)
        .expect("load snapshot");

    // Fixed real task set (3 query perturbations around Well 6 where the solution family lives):
    let family_primary = snapshot
        .profiles
        .get(&RecordId::parse("prof_family_native_surreal_verifier").unwrap())
        .expect("family_primary");

    for (task_idx, delta) in [(0usize, 0.0), (1, 0.01), (2, -0.01)] {
        let query_t2 = [
            (family_primary.t2_phase[0] + delta).rem_euclid(std::f64::consts::TAU),
            (family_primary.t2_phase[1] + delta).rem_euclid(std::f64::consts::TAU),
        ];
        let plan = snapshot
            .plan_for_query(family_primary.h3_lorentz, query_t2, &live_registry)
            .expect("plan_for_query must succeed within two hops");

        assert_eq!(
            plan.nearest_solution_family,
            RecordId::parse("prof_family_native_surreal_verifier").unwrap(),
            "task {task_idx} must land on native_surreal_contract_verification family"
        );
        assert!(
            plan.dag
                .topological_order
                .contains(&RecordId::parse("prof_tool_surreal_contract_verify").unwrap()),
            "selected DAG must include live primary tool surreal_contract_verify@1.0.0"
        );
        assert!(!plan.dag.used_fallback);

        // Verify benched, superseded, and conflicting/unregistered tools were excluded before planning:
        assert!(
            plan.excluded_profiles
                .contains_key(&RecordId::parse("prof_tool_benched_experimental_probe").unwrap()),
            "benched tool must be excluded"
        );
        assert!(
            plan.excluded_profiles
                .contains_key(&RecordId::parse("prof_tool_surreal_contract_verify_v090").unwrap()),
            "superseded v0.9.0 tool must be excluded"
        );
        assert!(
            plan.excluded_profiles
                .contains_key(&RecordId::parse("prof_tool_conflicting_direct_mutate").unwrap()),
            "conflicting/unregistered tool must be excluded"
        );

        // Verify the materialized ExecutionDag is strictly acyclic even though
        // the Atlas neighborhood contains the `skill <-> hook` cycle:
        plan.dag
            .verify_acyclic()
            .expect("ExecutionDag must be strictly acyclic");

        // Verify ONLY selected callable contracts enter the binary context frame
        // (3 projections: mcp_server, tool, runbook out of 17 total profiles):
        assert_eq!(plan.context_frame.projections.len(), 3);
        assert!(
            plan.context_frame_bytes
                .starts_with(b"GMCT\x01\x00\x00\x00")
        );
    }

    // Prove source_digest invariance: mutating `source_digest` on a clone of
    // the snapshot produces identical coordinates, content_digest, scores, and DAG digest.
    let mut snapshot_alt_source = snapshot.clone();
    for prof in snapshot_alt_source.profiles.values_mut() {
        prof.source_digest = deterministic_digest(&format!(
            "completely_different_teacher_source:{}",
            prof.id.as_str()
        ));
    }
    let plan_orig = snapshot
        .plan_for_query(
            family_primary.h3_lorentz,
            family_primary.t2_phase,
            &live_registry,
        )
        .unwrap();
    let plan_alt = snapshot_alt_source
        .plan_for_query(
            family_primary.h3_lorentz,
            family_primary.t2_phase,
            &live_registry,
        )
        .unwrap();
    assert_eq!(plan_orig.dag.dag_digest, plan_alt.dag.dag_digest);
    assert_eq!(plan_orig.ranked_compositions, plan_alt.ranked_compositions);

    // Prove LiveRegistry authority & fallback promotion:
    // If `surreal_contract_verify@1.0.0` has a schema digest mismatch in LiveRegistry,
    // `reconcile` rejects it and `plan_for_query` promotes the evidence-backed
    // fallback `surreal_readonly_snapshot@1.0.0` via `fallback_for`.
    live_registry
        .entries
        .get_mut("mcp://globalmind-contract/surreal_contract_verify@1.0.0")
        .unwrap()
        .argument_schema_digest = deterministic_digest("mismatched_live_arg_schema");

    let primary_prof = snapshot
        .profiles
        .get(&RecordId::parse("prof_tool_surreal_contract_verify").unwrap())
        .unwrap();
    assert_eq!(
        live_registry.reconcile(primary_prof),
        Err(ReconciliationDisqualification::ArgumentSchemaDigestMismatch)
    );

    let fallback_plan = snapshot
        .plan_for_query(
            family_primary.h3_lorentz,
            family_primary.t2_phase,
            &live_registry,
        )
        .expect("fallback plan must succeed when primary tool is digest-mismatched");
    assert!(fallback_plan.dag.used_fallback);
    assert!(
        fallback_plan
            .dag
            .topological_order
            .contains(&RecordId::parse("prof_tool_fallback_readonly_snapshot").unwrap()),
        "planner must select evidence-backed fallback tool"
    );
    fallback_plan.dag.verify_acyclic().unwrap();

    println!(
        "CAPABILITY_GATE2_RECEIPT primary_dag_digest={} fallback_dag_digest={} projections={} excluded={}",
        plan_orig.dag.dag_digest.as_str(),
        fallback_plan.dag.dag_digest.as_str(),
        plan_orig.context_frame.projections.len(),
        plan_orig.excluded_profiles.len()
    );
}

#[test]
fn gate3_offline_mandatory_rules_restart_replay_supersede_and_live_execution() {
    let mut surreal = IsolatedSurreal::start("gate3");
    apply_all_migrations_through_0012(&surreal);

    let store = GlobalMindStore::connect(surreal.config()).expect("connect store");
    let workspace = RecordId::parse("globalmind_cap_test").unwrap();
    let topology = WhiteheadTopology::canonical(workspace.clone()).expect("topology");
    store
        .admit_whitehead_topology(&topology)
        .expect("admit whitehead topology");

    let (profiles, edges, mut live_registry) = build_vertical_slice(&topology, &workspace);
    store
        .scaffold_capability_bundle(&workspace, &topology, &profiles, &edges)
        .expect("scaffold vertical slice");

    let snapshot_before = store
        .load_capability_atlas_snapshot(&workspace)
        .expect("snapshot before restart");
    let family_primary = snapshot_before
        .profiles
        .get(&RecordId::parse("prof_family_native_surreal_verifier").unwrap())
        .unwrap();
    let plan_before = snapshot_before
        .plan_for_query(
            family_primary.h3_lorentz,
            family_primary.t2_phase,
            &live_registry,
        )
        .expect("plan before restart");

    // 1. Stop SurrealDB completely so Atlas recall is offline/unavailable, and
    //    prove MandatoryPolicyGate remains enforced in Rust.
    surreal.stop();
    let policy_gate = MandatoryPolicyGate::read_only_smoke();
    let selected_tool_proj = plan_before
        .context_frame
        .projections
        .iter()
        .find(|p| p.kind == CapabilityKind::Tool)
        .unwrap()
        .clone();

    // JSON payload on model I/O boundary is blocked while SurrealDB is offline:
    assert!(
        policy_gate
            .enforce_call(
                &selected_tool_proj,
                b"GMBI\x01\x00\x00\x00{\"cmd\":\" sql\"}"
            )
            .is_err()
    );
    // ExternalWrite effect is blocked while SurrealDB is offline:
    let mut write_proj = selected_tool_proj.clone();
    write_proj.effect_class = EffectClass::ExternalWrite;
    assert!(
        policy_gate
            .enforce_call(&write_proj, b"GMBI\x01\x00\x00\x00\x01\x02\x03\x04")
            .is_err()
    );

    // 2. Restart SurrealDB 3.3.5 and prove restart replay returns identical
    //    candidate IDs, 2-hop neighborhood, DAG digest, and context frame digest.
    let new_pid = surreal.restart();
    let snapshot_after = store
        .load_capability_atlas_snapshot(&workspace)
        .expect("snapshot after restart");
    assert_eq!(
        snapshot_before.profiles.keys().collect::<Vec<_>>(),
        snapshot_after.profiles.keys().collect::<Vec<_>>(),
        "restart replay must return identical profile IDs"
    );
    let plan_after = snapshot_after
        .plan_for_query(
            family_primary.h3_lorentz,
            family_primary.t2_phase,
            &live_registry,
        )
        .expect("plan after restart");
    assert_eq!(
        plan_before.expanded_two_hop_neighborhood, plan_after.expanded_two_hop_neighborhood,
        "restart replay must return identical 2-hop neighborhood"
    );
    assert_eq!(
        plan_before.dag.dag_digest, plan_after.dag.dag_digest,
        "restart replay must return identical DAG digest"
    );
    assert_eq!(
        plan_before.context_frame_bytes, plan_after.context_frame_bytes,
        "restart replay must return identical binary context frame"
    );

    // 3. Execute a real read-only tool call through:
    //    Atlas -> LiveRegistry -> AtlasPlanner -> MandatoryPolicyGate -> HarnessExecutionBridge
    //    that queries the live isolated SurrealDB contract and records a utility receipt.
    let request_envelope = b"GMBI\x01\x00\x00\x00VERIFY_CAPABILITY_ATLAS_TABLES";
    let util_id = RecordId::parse("util_receipt_smoke_001").unwrap();
    let exec_receipt = execute_planned_readonly_tool(
        &plan_after,
        &live_registry,
        &policy_gate,
        &workspace,
        &util_id,
        request_envelope,
        |proj, req_bytes| {
            assert_eq!(proj.canonical_name, "surreal_contract_verify");
            assert_eq!(req_bytes, request_envelope);
            // Perform a real read-only verification against the isolated SurrealDB:
            store.verify_capability_atlas_contract()?;
            let mut out = b"GMBI\x01\x00\x00\x00OK:CAPABILITY_ATLAS_VERIFIED:".to_vec();
            out.extend_from_slice(proj.argument_schema_digest.as_str().as_bytes());
            Ok(out)
        },
    )
    .expect("execute_planned_readonly_tool must succeed");

    store
        .append_capability_utility_receipt(&exec_receipt.utility_receipt)
        .expect("append utility receipt");

    // Prove recording a utility receipt does NOT mutate semantic coordinates:
    let snapshot_post_util = store
        .load_capability_atlas_snapshot(&workspace)
        .expect("snapshot after utility receipt");
    for (id, prof_before) in &snapshot_after.profiles {
        let prof_after = snapshot_post_util.profiles.get(id).unwrap();
        assert_eq!(prof_before.h3_lorentz, prof_after.h3_lorentz);
        assert_eq!(prof_before.t2_phase, prof_after.t2_phase);
        assert_eq!(prof_before.content_digest, prof_after.content_digest);
    }

    // 4. Prove updating a tool (`supersede_capability_profile`) creates a new
    //    immutable version `1.1.0` plus `supersedes` edge without overwriting `1.0.0`.
    let tool_v110 = make_scaffold(
        &topology,
        &workspace,
        "tool_surreal_contract_verify_v110",
        6,
        CapabilityKind::Tool,
        "surreal_contract_verify",
        "Upgraded v1.1.0 read-only SurrealDB contract verifier",
        "1.1.0",
        CapabilityPayload::Tool {
            typed_arguments_result: "GMBI SchemaVerifyArgsV1_1 -> GMBI SchemaVerifyResultV1_1"
                .into(),
            side_effects: "none (read_only)".into(),
            timeout_spec: "4000ms hard deadline, 1 retry".into(),
        },
        CapabilityLifecycle::Qualified,
        EffectClass::ReadOnly,
        "mcp://globalmind-contract/surreal_contract_verify@1.1.0",
        0.99,
        0.04,
        0.01,
        None,
    );
    let old_tool = snapshot_after
        .profiles
        .get(&RecordId::parse("prof_tool_surreal_contract_verify").unwrap())
        .unwrap();
    let supersedes_edge = make_edge(
        &workspace,
        "supersedes_v110_over_v100",
        &tool_v110,
        old_tool,
        CapabilityEdgeKind::Supersedes,
        None,
    );
    let new_digest: Digest = store
        .supersede_capability_profile(
            &workspace,
            &topology,
            &old_tool.id,
            &tool_v110,
            &supersedes_edge,
        )
        .expect("supersede v1.0.0 with v1.1.0");
    assert_eq!(new_digest, tool_v110.content_digest);

    // Register v1.1.0 in LiveRegistry and verify v1.0.0 is still intact in the snapshot:
    live_registry.register(LiveRegistryEntry {
        live_registry_id: tool_v110.live_registry_id.clone(),
        installed: true,
        healthy: true,
        version: tool_v110.version.clone(),
        argument_schema_digest: tool_v110.argument_schema_digest.clone(),
        result_schema_digest: tool_v110.result_schema_digest.clone(),
        configuration_digest: tool_v110.configuration_digest.clone(),
        granted_permissions: ["surreal.schema.read".to_string()].into_iter().collect(),
        available_credential_classes: ["loopback_session_token".to_string()].into_iter().collect(),
    });
    let snapshot_v110 = store
        .load_capability_atlas_snapshot(&workspace)
        .expect("load snapshot after v1.1.0 supersede");
    assert!(
        snapshot_v110
            .profiles
            .contains_key(&RecordId::parse("prof_tool_surreal_contract_verify").unwrap()),
        "v1.0.0 profile must remain preserved in qualified history"
    );
    assert!(
        snapshot_v110
            .superseded_profiles()
            .contains(&RecordId::parse("prof_tool_surreal_contract_verify").unwrap()),
        "v1.0.0 must be marked superseded by v1.1.0"
    );
    let context_digest = digest_bytes(&plan_after.context_frame_bytes).unwrap();
    println!(
        "CAPABILITY_GATE3_RECEIPT restarted_pid={} dag_digest={} context_frame_digest={} policy_receipt={} response_digest={} v110_digest={}",
        new_pid,
        plan_after.dag.dag_digest.as_str(),
        context_digest.as_str(),
        exec_receipt.policy_gate_receipt_digest.as_str(),
        exec_receipt.response_digest.as_str(),
        new_digest.as_str()
    );
}
