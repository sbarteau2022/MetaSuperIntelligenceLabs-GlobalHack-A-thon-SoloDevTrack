//! Model forward-pass smoke run ON the audited minimum vertical slice.
//!
//! Queue 1 item 7: prove the harness forward path (plan -> reconcile ->
//! mandatory gate -> execute) fails closed when GlobalMind is unavailable or
//! identity contracts mismatch, and completes a receipted read-only call when
//! healthy. The slice itself is never modified or expanded here.
//!
//! No model inference is claimed: the native `trace` CLI does not exist yet,
//! so no forward activations are produced. The smoke binds to the frozen
//! artifact manifest by presence/size only (rehash belongs to `verify`).

use super::vertical_slice::vertical_slice;
use super::{
    CapabilityKind, EffectClass, LiveRegistry, MandatoryPolicyGate, UtilityOutcome,
    execute_planned_readonly_tool,
};
use crate::{Digest, GlobalMindStore, RecordId, StoreConfig};

pub const FORWARD_SMOKE_MANIFEST: &str = "manifests/qwen36-layer3-candidate-v2.manifest";
pub const FORWARD_SMOKE_SCHEMA_PROBE: &str =
    "../hypermind-nested-kv/schema/0012_capability_atlas.surql";

#[derive(Clone, Debug)]
pub struct SmokeArtifactPresence {
    pub role: String,
    pub path: String,
    pub bytes: u64,
    pub present: bool,
}

/// Complete receipt bundle for one forward-pass smoke run.
#[derive(Clone, Debug)]
pub struct ForwardSmokeReport {
    pub manifest_path: String,
    pub manifest_models: usize,
    pub artifacts: Vec<SmokeArtifactPresence>,
    pub slice_profiles: usize,
    pub slice_edges: usize,
    pub nearest_family: RecordId,
    pub selected_tool: RecordId,
    pub ranked_candidate_ids: Vec<RecordId>,
    pub neighborhood_size: usize,
    pub excluded_count: usize,
    pub dag_digest: Digest,
    pub used_fallback: bool,
    pub dag_mandatory_rules: Vec<RecordId>,
    pub unavailable_probe_error: String,
    pub gate_receipt_without_globalmind: Digest,
    pub mismatch_reasons: Vec<(String, String)>,
    pub execution_without_registry_error: String,
    pub executor_invoked_on_failure: bool,
    pub readonly_tool: RecordId,
    pub policy_receipt: Digest,
    pub response_digest: Digest,
    pub execution_receipt: Digest,
    pub utility_outcome: UtilityOutcome,
    pub response_bytes: usize,
}

impl ForwardSmokeReport {
    /// Deterministic plain-text rendering for receipt transcription.
    pub fn render_plaintext(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        let _ = writeln!(out, "forward_smoke_receipt_v1");
        let _ = writeln!(out, "manifest_path={}", self.manifest_path);
        let _ = writeln!(out, "manifest_models={}", self.manifest_models);
        for artifact in &self.artifacts {
            let _ = writeln!(
                out,
                "artifact role={} bytes={} present={} path={}",
                artifact.role, artifact.bytes, artifact.present, artifact.path
            );
        }
        let _ = writeln!(out, "slice_profiles={}", self.slice_profiles);
        let _ = writeln!(out, "slice_edges={}", self.slice_edges);
        let _ = writeln!(out, "nearest_family={}", self.nearest_family.as_str());
        let _ = writeln!(out, "selected_tool={}", self.selected_tool.as_str());
        for id in &self.ranked_candidate_ids {
            let _ = writeln!(out, "ranked_candidate={}", id.as_str());
        }
        let _ = writeln!(out, "neighborhood_size={}", self.neighborhood_size);
        let _ = writeln!(out, "excluded_count={}", self.excluded_count);
        let _ = writeln!(out, "dag_digest={}", self.dag_digest.as_str());
        let _ = writeln!(out, "used_fallback={}", self.used_fallback);
        for rule in &self.dag_mandatory_rules {
            let _ = writeln!(out, "dag_mandatory_rule={}", rule.as_str());
        }
        let _ = writeln!(
            out,
            "unavailable_probe_error={}",
            self.unavailable_probe_error
        );
        let _ = writeln!(
            out,
            "gate_receipt_without_globalmind={}",
            self.gate_receipt_without_globalmind.as_str()
        );
        for (case, reason) in &self.mismatch_reasons {
            let _ = writeln!(out, "mismatch case={case} reason={reason}");
        }
        let _ = writeln!(
            out,
            "execution_without_registry_error={}",
            self.execution_without_registry_error
        );
        let _ = writeln!(
            out,
            "executor_invoked_on_failure={}",
            self.executor_invoked_on_failure
        );
        let _ = writeln!(out, "readonly_tool={}", self.readonly_tool.as_str());
        let _ = writeln!(out, "policy_receipt={}", self.policy_receipt.as_str());
        let _ = writeln!(out, "response_digest={}", self.response_digest.as_str());
        let _ = writeln!(out, "execution_receipt={}", self.execution_receipt.as_str());
        let _ = writeln!(out, "utility_outcome={}", self.utility_outcome.as_str());
        let _ = writeln!(out, "response_bytes={}", self.response_bytes);
        out
    }
}

/// Runs the full forward-pass smoke against the audited slice and returns
/// the receipt bundle. Any unexpected success or failure is an error.
pub fn run_forward_pass_smoke() -> Result<ForwardSmokeReport, String> {
    // 1. Manifest presence/size binding (no rehash; that is `verify`).
    let manifest = crate::Manifest::load(FORWARD_SMOKE_MANIFEST)?;
    let mut artifacts = Vec::new();
    for model in &manifest.models {
        for artifact in &model.artifacts {
            let (bytes, present) = match std::fs::metadata(&artifact.path) {
                Ok(meta) => (meta.len(), true),
                Err(_) => (0, false),
            };
            artifacts.push(SmokeArtifactPresence {
                role: format!("{}:{}", model.name, artifact.role),
                path: artifact.path.clone(),
                bytes,
                present,
            });
        }
    }

    // 2. Replay the audited slice; bind by counts, selected tool, and DAG digest.
    let slice = vertical_slice();
    if slice.snapshot.profiles.len() != 13 || slice.snapshot.edges.len() != 17 {
        return Err("forward smoke requires the audited 13-profile / 17-edge slice".into());
    }
    let planned = slice
        .snapshot
        .plan_for_query(slice.query_h3, slice.query_t2, &slice.registry)?;
    let (selected_tool, _) = planned
        .ranked_compositions
        .first()
        .cloned()
        .ok_or_else(|| "smoke plan produced no ranked composition".to_string())?;
    if selected_tool != slice.ids.tool_live {
        return Err("smoke plan did not select the audited live tool".into());
    }
    let tool_projection = planned
        .context_frame
        .projections
        .iter()
        .find(|p| p.kind == CapabilityKind::Tool)
        .cloned()
        .ok_or_else(|| "smoke plan has no tool projection".to_string())?;

    // 3. GlobalMind-unavailable probe: verifier against a dead endpoint.
    let dead_port = {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind: {e}"))?;
        listener
            .local_addr()
            .map_err(|e| format!("port: {e}"))?
            .port()
    };
    let dead_store = GlobalMindStore::connect(StoreConfig {
        endpoint: format!("http://127.0.0.1:{dead_port}"),
        namespace: "vs_smoke_dead".into(),
        database: "vs_smoke_dead".into(),
        username: Some("root".into()),
        password: Some("vs_smoke_dead".into()),
        allow_quarantined_core: false,
    })?;
    let unavailable_probe_error = match dead_store.verify_capability_atlas_contract() {
        Ok(()) => return Err("verifier unexpectedly succeeded against a dead endpoint".into()),
        Err(reason) => reason,
    };

    // 4. Mandatory gate enforces with zero GlobalMind contact.
    let gate = MandatoryPolicyGate::read_only_smoke();
    let valid_request = b"GMBI\x01\x00\x00\x00vs_smoke_probe";
    let gate_receipt_without_globalmind = gate.enforce_call(&tool_projection, valid_request)?;
    let mut writer = tool_projection.clone();
    writer.effect_class = EffectClass::LocalWrite;
    if gate.enforce_call(&writer, valid_request).is_ok() {
        return Err("mandatory gate admitted a write effect during smoke".into());
    }

    // 5. Identity-mismatch matrix: every drift class must refuse.
    let live = slice
        .snapshot
        .profiles
        .get(&slice.ids.tool_live)
        .cloned()
        .ok_or_else(|| "audited live tool missing".to_string())?;
    let entry_id = live.live_registry_id.clone();
    let mut mismatch_reasons = Vec::new();
    for case in [
        "missing_entry",
        "not_installed",
        "unhealthy",
        "version_mismatch",
        "arg_digest_mismatch",
        "result_digest_mismatch",
        "config_digest_mismatch",
        "missing_permission",
    ] {
        let mut registry = slice.registry.clone();
        if case == "missing_entry" {
            registry.entries.remove(&entry_id);
        } else if let Some(entry) = registry.entries.get_mut(&entry_id) {
            match case {
                "not_installed" => entry.installed = false,
                "unhealthy" => entry.healthy = false,
                "version_mismatch" => entry.version = "9.9.9-smoke".into(),
                "arg_digest_mismatch" => {
                    entry.argument_schema_digest = Digest::sha256(b"smoke-wrong-arg");
                }
                "result_digest_mismatch" => {
                    entry.result_schema_digest = Digest::sha256(b"smoke-wrong-res");
                }
                "config_digest_mismatch" => {
                    entry.configuration_digest = Digest::sha256(b"smoke-wrong-cfg");
                }
                "missing_permission" => entry.granted_permissions.clear(),
                _ => {}
            }
        }
        match registry.reconcile(&live) {
            Ok(()) => return Err(format!("mismatch case {case} unexpectedly reconciled")),
            Err(reason) => mismatch_reasons.push((case.into(), format!("{reason:?}"))),
        }
    }

    // 6. Execution without a registry fails closed and never invokes the tool.
    let invoked = std::cell::Cell::new(false);
    let execution_without_registry_error = match execute_planned_readonly_tool(
        &planned,
        &LiveRegistry::default(),
        &gate,
        &slice.ids.workspace,
        &RecordId::parse("vs_smoke_util_fail")?,
        valid_request,
        |_, _| {
            invoked.set(true);
            Ok(vec![])
        },
    ) {
        Ok(_) => {
            return Err("execution unexpectedly succeeded without a registry".into());
        }
        Err(reason) => reason,
    };
    let executor_invoked_on_failure = invoked.get();
    if executor_invoked_on_failure {
        return Err("executor was invoked despite registry failure".into());
    }

    // 7. Healthy read-only execution with a real disk-read executor.
    let receipt = execute_planned_readonly_tool(
        &planned,
        &slice.registry,
        &gate,
        &slice.ids.workspace,
        &RecordId::parse("vs_smoke_util_001")?,
        valid_request,
        |projection, binary_request| {
            if projection.profile_id != slice.ids.tool_live {
                return Err("smoke executed an unselected tool".into());
            }
            if binary_request != valid_request {
                return Err("smoke request bytes drifted".into());
            }
            let schema_bytes = std::fs::read(FORWARD_SMOKE_SCHEMA_PROBE)
                .map_err(|e| format!("smoke schema read failed: {e}"))?;
            let text = String::from_utf8(schema_bytes.clone())
                .map_err(|_| "smoke schema file is not UTF-8".to_string())?;
            if !text.contains("DEFINE TABLE IF NOT EXISTS capability_profile") {
                return Err("smoke schema file lacks capability_profile table".into());
            }
            let file_digest = Digest::sha256(&schema_bytes);
            let mut response = b"GMBI\x01\x00\x00\x00".to_vec();
            response.extend_from_slice(file_digest.as_str().as_bytes());
            Ok(response)
        },
    )?;

    Ok(ForwardSmokeReport {
        manifest_path: FORWARD_SMOKE_MANIFEST.into(),
        manifest_models: manifest.models.len(),
        artifacts,
        slice_profiles: slice.snapshot.profiles.len(),
        slice_edges: slice.snapshot.edges.len(),
        nearest_family: planned.nearest_solution_family.clone(),
        selected_tool,
        ranked_candidate_ids: planned
            .ranked_compositions
            .iter()
            .map(|(id, _)| id.clone())
            .collect(),
        neighborhood_size: planned.expanded_two_hop_neighborhood.len(),
        excluded_count: planned.excluded_profiles.len(),
        dag_digest: planned.dag.dag_digest.clone(),
        used_fallback: planned.dag.used_fallback,
        dag_mandatory_rules: planned.dag.mandatory_rules.clone(),
        unavailable_probe_error,
        gate_receipt_without_globalmind,
        mismatch_reasons,
        execution_without_registry_error,
        executor_invoked_on_failure,
        readonly_tool: receipt.tool_profile_id.clone(),
        policy_receipt: receipt.policy_gate_receipt_digest.clone(),
        response_digest: receipt.response_digest.clone(),
        execution_receipt: receipt.utility_receipt.execution_receipt_digest.clone(),
        utility_outcome: receipt.utility_receipt.outcome,
        response_bytes: receipt.response_envelope_bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex64(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }

    #[test]
    fn smoke_binds_to_the_audited_slice() {
        let report = run_forward_pass_smoke().expect("smoke runs");
        assert_eq!(report.slice_profiles, 13);
        assert_eq!(report.slice_edges, 17);
        assert_eq!(report.selected_tool.as_str(), "vs_tool_live");
        assert_eq!(report.nearest_family.as_str(), "vs_family_audit");
        assert!(!report.used_fallback);
        assert_eq!(report.ranked_candidate_ids.len(), 1);
        assert!(
            report
                .dag_mandatory_rules
                .iter()
                .any(|r| r.as_str() == "vs_rule"),
            "mandatory rule rides the DAG"
        );
        assert!(is_hex64(report.dag_digest.as_str()));
        assert_eq!(report.manifest_models, 1);
        assert!(
            report.artifacts.iter().all(|a| a.present),
            "all manifest artifacts present"
        );
        assert!(report.artifacts.iter().all(|a| a.bytes > 0));
    }

    #[test]
    fn smoke_proves_unavailable_fail_closed() {
        let report = run_forward_pass_smoke().expect("smoke runs");
        assert!(!report.unavailable_probe_error.is_empty());
        assert!(is_hex64(report.gate_receipt_without_globalmind.as_str()));
        assert!(!report.executor_invoked_on_failure);
        assert!(!report.execution_without_registry_error.is_empty());
    }

    #[test]
    fn smoke_records_every_mismatch_class() {
        let report = run_forward_pass_smoke().expect("smoke runs");
        let cases: Vec<_> = report
            .mismatch_reasons
            .iter()
            .map(|(c, _)| c.as_str())
            .collect();
        assert_eq!(
            cases,
            [
                "missing_entry",
                "not_installed",
                "unhealthy",
                "version_mismatch",
                "arg_digest_mismatch",
                "result_digest_mismatch",
                "config_digest_mismatch",
                "missing_permission",
            ]
        );
        for (case, reason) in &report.mismatch_reasons {
            assert!(!reason.is_empty(), "{case} records a reason");
        }
    }

    #[test]
    fn smoke_completes_readonly_execution_with_receipts() {
        let report = run_forward_pass_smoke().expect("smoke runs");
        assert_eq!(report.readonly_tool.as_str(), "vs_tool_live");
        assert_eq!(report.utility_outcome, UtilityOutcome::Succeeded);
        assert!(is_hex64(report.policy_receipt.as_str()));
        assert!(is_hex64(report.response_digest.as_str()));
        assert!(is_hex64(report.execution_receipt.as_str()));
        assert_eq!(report.response_bytes, 8 + 64);
    }

    #[test]
    fn smoke_receipt_renders_transcribable_plaintext() {
        let report = run_forward_pass_smoke().expect("smoke runs");
        let text = report.render_plaintext();
        println!("{text}");
        for marker in [
            "forward_smoke_receipt_v1",
            "manifest_path=",
            "slice_profiles=13",
            "selected_tool=vs_tool_live",
            "dag_digest=",
            "unavailable_probe_error=",
            "mismatch case=",
            "executor_invoked_on_failure=false",
            "utility_outcome=succeeded",
        ] {
            assert!(text.contains(marker), "receipt renders {marker}");
        }
    }
}
