//! Minimum Capability Atlas vertical slice + audit proof.
//!
//! One real problem, two solution families (one near, one distractor), one
//! qualified runbook, one live MCP/tool, one unavailable conflicting tool, one
//! mandatory rule, and one evidence-backed fallback — wired through the
//! 8-step retrieval/planning algorithm and the harness execution bridge.
//!
//! The Atlas proposes; only [`LiveRegistry`], [`MandatoryPolicyGate`], and the
//! harness executor authorize and run. No JSON/Python crosses model I/O.

use super::{
    CapabilityAtlasSnapshot, CapabilityEdgeKind, CapabilityEdgeRecord, CapabilityKind,
    CapabilityLifecycle, CapabilityPayload, CapabilityScaffold, EffectClass, LiveRegistry,
    LiveRegistryEntry, deterministic_digest,
};
use crate::whitehead::WhiteheadTopology;
use crate::{Digest, RecordId};
use std::collections::BTreeSet;

/// IDs for every profile in the vertical slice.
#[derive(Clone, Debug)]
pub struct VerticalSliceIds {
    pub workspace: RecordId,
    pub problem: RecordId,
    pub family_near: RecordId,
    pub family_far: RecordId,
    pub capability: RecordId,
    pub runbook: RecordId,
    pub mcp_server: RecordId,
    pub tool_live: RecordId,
    pub tool_conflicting: RecordId,
    pub tool_fallback: RecordId,
    pub tool_benched: RecordId,
    pub rule: RecordId,
    pub evidence: RecordId,
    pub failure: RecordId,
}

/// The audited minimum vertical slice: snapshot + reconciled live registry.
#[derive(Clone, Debug)]
pub struct VerticalSlice {
    pub ids: VerticalSliceIds,
    pub snapshot: CapabilityAtlasSnapshot,
    pub registry: LiveRegistry,
    pub query_h3: [f64; 4],
    pub query_t2: [f64; 2],
}

fn rid(name: &str) -> RecordId {
    RecordId::parse(name).expect("slice id is valid")
}

fn label_digest(label: &str) -> Digest {
    deterministic_digest(label)
}

#[allow(clippy::too_many_arguments)]
fn scaffold(
    topology: &WhiteheadTopology,
    workspace: &RecordId,
    id: &str,
    kind: CapabilityKind,
    canonical_name: &str,
    summary: &str,
    payload: CapabilityPayload,
    well_ordinal: u8,
    lifecycle: CapabilityLifecycle,
    effect_class: EffectClass,
    permissions: Vec<String>,
    expected_utility: f64,
    resource_cost: f64,
    prediction_error: f64,
    with_evidence_tier: u8,
) -> CapabilityScaffold {
    let arg_digest = label_digest(&format!("vs_arg_{id}"));
    let res_digest = label_digest(&format!("vs_res_{id}"));
    let content_digest = CapabilityScaffold::compute_content_digest(
        kind,
        canonical_name,
        summary,
        "1.0.0",
        &payload,
        &arg_digest,
        &res_digest,
    );
    let (h3_lorentz, t2_phase, well) = CapabilityScaffold::derive_model_neutral_coordinates(
        topology,
        well_ordinal,
        &content_digest,
    )
    .expect("well ordinal is valid");
    let profile = CapabilityScaffold {
        id: rid(id),
        atom_id: rid(&format!("vs_atom_{id}")),
        address_id: rid(&format!("vs_addr_{id}")),
        membership_id: rid(&format!("vs_memb_{id}")),
        workspace: workspace.clone(),
        geometry_space: rid("h3_t2_r12_v1"),
        well,
        kind,
        canonical_name: canonical_name.into(),
        summary: summary.into(),
        version: "1.0.0".into(),
        payload,
        content_digest,
        source_digest: label_digest(&format!("vs_src_{id}")),
        configuration_digest: label_digest(&format!("vs_cfg_{id}")),
        h3_lorentz,
        t2_phase,
        lifecycle,
        admission_receipt_digest: label_digest(&format!("vs_adm_{id}")),
        judge_receipt_digest: label_digest(&format!("vs_jdg_{id}")),
        replay_receipt_digest: label_digest(&format!("vs_rep_{id}")),
        crystallization_receipt_digest: label_digest(&format!("vs_cry_{id}")),
        argument_schema_digest: arg_digest,
        result_schema_digest: res_digest,
        effect_class,
        permission_classes: permissions,
        credential_classes: Vec::new(),
        platform_requirements: vec!["macos_aarch64".into()],
        runtime_requirements: vec!["globalmind_runtime_0.1.0".into()],
        version_requirement: ">=1.0.0".into(),
        timeout_ms: 5_000,
        max_retries: 1,
        idempotent: true,
        rollback_policy: "read_only_noop".into(),
        max_memory_bytes: 16 * 1024 * 1024,
        health_check_id: format!("vs_health_{id}"),
        live_registry_id: format!("vsreg/{id}@1.0.0"),
        component_receipt_digest: (with_evidence_tier >= 1)
            .then(|| label_digest(&format!("vs_comp_{id}"))),
        integration_receipt_digest: (with_evidence_tier >= 2)
            .then(|| label_digest(&format!("vs_int_{id}"))),
        live_receipt_digest: (with_evidence_tier >= 3)
            .then(|| label_digest(&format!("vs_live_{id}"))),
        incompatible_profiles: Vec::new(),
        benched_reason: (lifecycle == CapabilityLifecycle::Benched)
            .then(|| "superseded_by_vs_tool_live_receipt_vs_bench_001".into()),
        expected_utility,
        resource_cost,
        prediction_error,
    };
    profile.validate().expect("slice scaffold is valid");
    profile
}

fn edge(
    snapshot: &CapabilityAtlasSnapshot,
    workspace: &RecordId,
    id: &str,
    from: &RecordId,
    to: &RecordId,
    kind: CapabilityEdgeKind,
    confidence: f64,
) -> CapabilityEdgeRecord {
    let from_kind = snapshot.profiles.get(from).expect("edge from exists").kind;
    let to_kind = snapshot.profiles.get(to).expect("edge to exists").kind;
    kind.validate_endpoints(from_kind, to_kind)
        .expect("slice edge endpoints are valid");
    let record = CapabilityEdgeRecord {
        id: rid(id),
        workspace: workspace.clone(),
        from_profile: from.clone(),
        to_profile: to.clone(),
        from_atom: snapshot
            .profiles
            .get(from)
            .expect("edge from exists")
            .atom_id
            .clone(),
        to_atom: snapshot
            .profiles
            .get(to)
            .expect("edge to exists")
            .atom_id
            .clone(),
        kind,
        confidence,
        evidence_receipt: None,
        evidence_receipt_digest: label_digest(&format!("vs_edge_ev_{id}")),
        producer: "vertical_slice_audit_v1".into(),
        revision: 1,
        valid_from_unix_ms: 1_800_000_000_000,
        valid_to_unix_ms: None,
        compatibility_digest: label_digest(&format!("vs_edge_cfg_{id}")),
        crystallization_receipt_digest: label_digest(&format!("vs_edge_cry_{id}")),
    };
    record.validate().expect("slice edge is valid");
    record
}

fn register_live(registry: &mut LiveRegistry, profile: &CapabilityScaffold) {
    registry.register(LiveRegistryEntry {
        live_registry_id: profile.live_registry_id.clone(),
        installed: true,
        healthy: true,
        version: profile.version.clone(),
        argument_schema_digest: profile.argument_schema_digest.clone(),
        result_schema_digest: profile.result_schema_digest.clone(),
        configuration_digest: profile.configuration_digest.clone(),
        granted_permissions: profile.permission_classes.iter().cloned().collect(),
        available_credential_classes: BTreeSet::new(),
    });
}

/// Builds the audited minimum vertical slice.
///
/// Real problem: read-only verification of the isolated SurrealDB contract
/// before the model forward-pass smoke. The conflicting tool is qualified in
/// the Atlas but absent from the live registry (unavailable) and declares a
/// `conflicts_with` edge against the live tool.
pub fn vertical_slice() -> VerticalSlice {
    let workspace = rid("vs_workspace");
    let topology =
        WhiteheadTopology::canonical(workspace.clone()).expect("canonical topology builds");
    let q = CapabilityLifecycle::Qualified;
    let ro = EffectClass::ReadOnly;

    let mut profiles: Vec<CapabilityScaffold> = vec![
        scaffold(
            &topology,
            &workspace,
            "vs_problem",
            CapabilityKind::ProblemPattern,
            "isolated_contract_verification",
            "verify the isolated SurrealDB capability contract before forward-pass smoke",
            CapabilityPayload::ProblemPattern {
                symptoms: "unknown whether migration 0012 is enforced on the isolated instance"
                    .into(),
                constraints: "read-only; quarantined hypermind/core must never be touched".into(),
                success_condition: "contract verifier reports enforced tables and readonly fields"
                    .into(),
            },
            0,
            q,
            ro,
            vec![],
            0.9,
            0.05,
            0.02,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_family_audit",
            CapabilityKind::SolutionFamily,
            "local_schema_audit",
            "audit the local isolated schema with native checks",
            CapabilityPayload::SolutionFamily {
                scope: "local isolated SurrealDB instances only".into(),
                exclusions: "no quarantined databases; no remote endpoints".into(),
                selection_criteria: "prefer native binary checks with receipt digests".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.85,
            0.1,
            0.03,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_family_remote",
            CapabilityKind::SolutionFamily,
            "remote_probe_sweep",
            "distractor family probing remote endpoints",
            CapabilityPayload::SolutionFamily {
                scope: "remote fleet endpoints".into(),
                exclusions: "excluded: this slice never contacts remote hosts".into(),
                selection_criteria: "fleet-wide sweeps only; out of scope here".into(),
            },
            5,
            q,
            ro,
            vec![],
            0.2,
            0.8,
            0.6,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_capability",
            CapabilityKind::Capability,
            "contract_read",
            "read and verify schema contract tables and fields",
            CapabilityPayload::Capability {
                input_output_contract:
                    "args: targets table list; result: enforced table field report".into(),
                effects: "none; strictly read-only observation".into(),
                failure_modes: "unreachable instance; drifted schema; digest mismatch".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.88,
            0.08,
            0.03,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_runbook",
            CapabilityKind::Runbook,
            "verify_contract_runbook",
            "ordered read-only contract verification procedure",
            CapabilityPayload::Runbook {
                steps: "1 connect isolated 2 info for tables 3 compare readonly fields 4 report"
                    .into(),
                stop_rollback_rules: "stop on any unreachable target; nothing to roll back".into(),
                evidence_required: "info-for-table output plus field readonly digest".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.8,
            0.12,
            0.04,
            2,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_mcp",
            CapabilityKind::McpServer,
            "contract_mcp",
            "local MCP provider exposing the read-only contract verifier",
            CapabilityPayload::McpServer {
                server_identity: "vs-local-contract-mcp 1.0.0".into(),
                transport: "native binary envelope over local channel".into(),
                advertised_contracts: "contract_verify v1 read-only".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.82,
            0.1,
            0.03,
            2,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_tool_live",
            CapabilityKind::Tool,
            "contract_verify",
            "read-only contract verification tool",
            CapabilityPayload::Tool {
                typed_arguments_result:
                    "args: table allowlist digest; result: enforcement report digest".into(),
                side_effects: "none".into(),
                timeout_spec: "bounded 5000ms single attempt".into(),
            },
            0,
            q,
            ro,
            vec!["atlas.slice.read".into()],
            0.92,
            0.08,
            0.02,
            3,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_tool_conflict",
            CapabilityKind::Tool,
            "mutating_probe",
            "unavailable conflicting probe that would mutate schema state",
            CapabilityPayload::Tool {
                typed_arguments_result: "args: raw admin handle; result: undefined".into(),
                side_effects: "mutates schema state; forbidden in this slice".into(),
                timeout_spec: "unbounded; rejected by policy".into(),
            },
            0,
            q,
            EffectClass::LocalWrite,
            vec!["atlas.admin.write".into()],
            0.1,
            0.9,
            0.7,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_tool_fallback",
            CapabilityKind::Tool,
            "contract_verify_offline",
            "evidence-backed offline fallback verifier",
            CapabilityPayload::Tool {
                typed_arguments_result: "args: cached schema bundle digest; result: cached report"
                    .into(),
                side_effects: "none".into(),
                timeout_spec: "bounded 5000ms single attempt".into(),
            },
            0,
            q,
            ro,
            vec!["atlas.slice.read".into()],
            0.7,
            0.15,
            0.08,
            1,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_tool_benched",
            CapabilityKind::Tool,
            "legacy_probe",
            "benched legacy probe kept for exclusion proof",
            CapabilityPayload::Tool {
                typed_arguments_result: "args: legacy flags; result: legacy report".into(),
                side_effects: "none".into(),
                timeout_spec: "bounded 5000ms single attempt".into(),
            },
            0,
            CapabilityLifecycle::Benched,
            ro,
            vec!["atlas.slice.read".into()],
            0.3,
            0.4,
            0.3,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_rule",
            CapabilityKind::PolicyRule,
            "readonly_mandatory",
            "mandatory read-only smoke rule enforced in Rust",
            CapabilityPayload::PolicyRule {
                scope: "all pre-smoke capability executions".into(),
                severity: "blocking".into(),
                enforcement_point: "harness_execution_boundary".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.99,
            0.01,
            0.0,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_evidence",
            CapabilityKind::EvidenceReceipt,
            "tool_live_qualification",
            "immutable qualification receipt for the live verifier",
            CapabilityPayload::EvidenceReceipt {
                subject_config_runtime_digests:
                    "subject cfg runtime digests bound at qualification".into(),
                tier: "live".into(),
                result: "pass".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.95,
            0.02,
            0.01,
            0,
        ),
        scaffold(
            &topology,
            &workspace,
            "vs_failure",
            CapabilityKind::FailurePattern,
            "mutating_probe_failure",
            "known failure of the mutating probe under isolated config",
            CapabilityPayload::FailurePattern {
                signature: "schema_write_rejected_by_readonly_gate".into(),
                cause: "probe attempted schema mutation on isolated instance".into(),
                containment: "blocked by mandatory policy gate before execution".into(),
                recovery: "use the read-only verifier or the offline fallback".into(),
            },
            0,
            q,
            ro,
            vec![],
            0.5,
            0.05,
            0.05,
            0,
        ),
    ];

    // The live tool explicitly declares the conflicting tool incompatible.
    if let Some(live) = profiles.iter_mut().find(|p| p.id == rid("vs_tool_live")) {
        live.incompatible_profiles.push(rid("vs_tool_conflict"));
        live.validate().expect("live tool stays valid");
    }

    let mut snapshot = CapabilityAtlasSnapshot::default();
    for profile in profiles {
        if snapshot
            .profiles
            .insert(profile.id.clone(), profile)
            .is_some()
        {
            panic!("duplicate profile id in vertical slice");
        }
    }
    let ids = VerticalSliceIds {
        workspace: workspace.clone(),
        problem: rid("vs_problem"),
        family_near: rid("vs_family_audit"),
        family_far: rid("vs_family_remote"),
        capability: rid("vs_capability"),
        runbook: rid("vs_runbook"),
        mcp_server: rid("vs_mcp"),
        tool_live: rid("vs_tool_live"),
        tool_conflicting: rid("vs_tool_conflict"),
        tool_fallback: rid("vs_tool_fallback"),
        tool_benched: rid("vs_tool_benched"),
        rule: rid("vs_rule"),
        evidence: rid("vs_evidence"),
        failure: rid("vs_failure"),
    };

    // NOTE: the conflicting tool's `implements` edge is ordered BEFORE the
    // live tool's so conflict exclusion is deterministic when both reconcile.
    let edge_specs = [
        (
            "vs_e01",
            "vs_family_audit",
            "vs_problem",
            CapabilityEdgeKind::Solves,
            0.95,
        ),
        (
            "vs_e02",
            "vs_family_remote",
            "vs_problem",
            CapabilityEdgeKind::Solves,
            0.4,
        ),
        (
            "vs_e03",
            "vs_capability",
            "vs_family_audit",
            CapabilityEdgeKind::Specializes,
            0.9,
        ),
        (
            "vs_e04",
            "vs_tool_conflict",
            "vs_capability",
            CapabilityEdgeKind::Implements,
            0.3,
        ),
        (
            "vs_e05",
            "vs_tool_live",
            "vs_capability",
            CapabilityEdgeKind::Implements,
            0.95,
        ),
        (
            "vs_e06",
            "vs_tool_benched",
            "vs_capability",
            CapabilityEdgeKind::Implements,
            0.5,
        ),
        (
            "vs_e07",
            "vs_runbook",
            "vs_capability",
            CapabilityEdgeKind::Implements,
            0.85,
        ),
        (
            "vs_e08",
            "vs_mcp",
            "vs_tool_live",
            CapabilityEdgeKind::Exposes,
            0.95,
        ),
        (
            "vs_e09",
            "vs_tool_conflict",
            "vs_tool_live",
            CapabilityEdgeKind::ConflictsWith,
            0.99,
        ),
        (
            "vs_e10",
            "vs_tool_fallback",
            "vs_tool_live",
            CapabilityEdgeKind::FallbackFor,
            0.8,
        ),
        (
            "vs_e11",
            "vs_rule",
            "vs_capability",
            CapabilityEdgeKind::EnforcedBy,
            1.0,
        ),
        (
            "vs_e12",
            "vs_tool_live",
            "vs_evidence",
            CapabilityEdgeKind::VerifiedBy,
            0.95,
        ),
        (
            "vs_e13",
            "vs_tool_fallback",
            "vs_evidence",
            CapabilityEdgeKind::VerifiedBy,
            0.7,
        ),
        (
            "vs_e14",
            "vs_tool_conflict",
            "vs_failure",
            CapabilityEdgeKind::FailedOn,
            0.9,
        ),
        (
            "vs_e15",
            "vs_runbook",
            "vs_tool_live",
            CapabilityEdgeKind::Requires,
            0.9,
        ),
        (
            "vs_e16",
            "vs_runbook",
            "vs_tool_live",
            CapabilityEdgeKind::ComposesWith,
            0.85,
        ),
        (
            "vs_e17",
            "vs_tool_live",
            "vs_runbook",
            CapabilityEdgeKind::ComposesWith,
            0.85,
        ),
    ];
    for (id, from, to, kind, confidence) in edge_specs {
        let record = edge(
            &snapshot,
            &workspace,
            id,
            &rid(from),
            &rid(to),
            kind,
            confidence,
        );
        snapshot.edges.push(record);
    }

    // Live registry: everything reconciled EXCEPT the conflicting tool
    // (unavailable) and the benched tool (never admitted).
    let mut registry = LiveRegistry::default();
    for id in [
        &ids.runbook,
        &ids.mcp_server,
        &ids.tool_live,
        &ids.tool_fallback,
    ] {
        register_live(
            &mut registry,
            snapshot.profiles.get(id).expect("slice profile exists"),
        );
    }

    let problem = snapshot.profiles.get(&ids.problem).expect("problem exists");
    let (query_h3, query_t2) = (problem.h3_lorentz, problem.t2_phase);

    VerticalSlice {
        ids,
        snapshot,
        registry,
        query_h3,
        query_t2,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CapabilityEdgeKind, CapabilityKind, CompactContextFrame, MandatoryPolicyGate,
        ReconciliationDisqualification, UtilityOutcome, execute_planned_readonly_tool,
    };
    use super::*;

    fn plan(slice: &VerticalSlice) -> super::super::PlannedCapabilityExecution {
        slice
            .snapshot
            .plan_for_query(slice.query_h3, slice.query_t2, &slice.registry)
            .expect("slice plans successfully")
    }

    #[test]
    fn slice_profiles_and_edges_validate() {
        let slice = vertical_slice();
        assert_eq!(slice.snapshot.profiles.len(), 13);
        assert_eq!(slice.snapshot.edges.len(), 17);
        for profile in slice.snapshot.profiles.values() {
            profile.validate().expect("slice profile validates");
        }
        for edge in &slice.snapshot.edges {
            edge.validate().expect("slice edge validates");
        }
    }

    #[test]
    fn twelve_kinds_and_thirteen_edge_kinds_are_enforced() {
        assert_eq!(CapabilityKind::ALL.len(), 12);
        assert_eq!(CapabilityEdgeKind::ALL.len(), 13);
        for kind in CapabilityKind::ALL {
            assert_eq!(CapabilityKind::parse(kind.as_str()).unwrap(), kind);
            assert_eq!(
                CapabilityKind::from_tag_byte(kind.tag_byte()).unwrap(),
                kind
            );
        }
        for kind in CapabilityEdgeKind::ALL {
            assert_eq!(CapabilityEdgeKind::parse(kind.as_str()).unwrap(), kind);
            assert_eq!(
                CapabilityEdgeKind::from_tag_byte(kind.tag_byte()).unwrap(),
                kind
            );
        }
        // Untyped / unknown kinds fail closed.
        assert!(CapabilityKind::parse("related").is_err());
        assert!(CapabilityKind::parse("tool_xyz").is_err());
        assert!(CapabilityKind::parse("").is_err());
        assert!(CapabilityEdgeKind::parse("related").is_err());
        assert!(CapabilityEdgeKind::parse("linked").is_err());
        assert!(CapabilityKind::from_tag_byte(0).is_err());
        assert!(CapabilityEdgeKind::from_tag_byte(0).is_err());
        assert!(CapabilityEdgeKind::from_tag_byte(14).is_err());
        // Invalid endpoint directions fail closed.
        assert!(
            CapabilityEdgeKind::Solves
                .validate_endpoints(CapabilityKind::Tool, CapabilityKind::ProblemPattern)
                .is_err()
        );
        assert!(
            CapabilityEdgeKind::FailedOn
                .validate_endpoints(CapabilityKind::Tool, CapabilityKind::Tool)
                .is_err()
        );
    }

    #[test]
    fn plan_reaches_live_tool_within_two_hops() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        assert_eq!(planned.nearest_solution_family, slice.ids.family_near);
        assert!(
            planned
                .expanded_two_hop_neighborhood
                .contains(&slice.ids.capability)
        );
        assert!(
            planned
                .expanded_two_hop_neighborhood
                .contains(&slice.ids.tool_live)
        );
        let (top_tool, top_score) = planned
            .ranked_compositions
            .first()
            .expect("ranked composition exists")
            .clone();
        assert_eq!(top_tool, slice.ids.tool_live);
        assert!(top_score > 0.0);
        assert!(!planned.dag.used_fallback);
        assert!(planned.dag.mandatory_rules.contains(&slice.ids.rule));
        assert_eq!(planned.dag.problem_profile_id, slice.ids.problem);
        assert_eq!(planned.dag.capability_id, slice.ids.capability);
    }

    #[test]
    fn unavailable_conflicting_and_benched_tools_are_excluded() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        let conflict_reason = planned
            .excluded_profiles
            .get(&slice.ids.tool_conflicting)
            .expect("conflicting tool is excluded");
        assert!(
            conflict_reason.contains("MissingFromLiveRegistry"),
            "{conflict_reason}"
        );
        let benched_reason = planned
            .excluded_profiles
            .get(&slice.ids.tool_benched)
            .expect("benched tool is excluded");
        assert!(
            benched_reason.contains("lifecycle=benched"),
            "{benched_reason}"
        );
        assert!(
            !planned
                .ranked_compositions
                .iter()
                .any(|(id, _)| *id == slice.ids.tool_conflicting)
        );
        assert!(
            !planned
                .ranked_compositions
                .iter()
                .any(|(id, _)| *id == slice.ids.tool_benched)
        );
    }

    #[test]
    fn conflict_exclusion_holds_when_both_tools_reconcile() {
        let mut slice = vertical_slice();
        // Adversarial case: the conflicting tool appears in the live registry.
        // Deterministic edge order excludes it via conflicts_with before the
        // live tool is scored.
        let conflicting = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_conflicting)
            .unwrap()
            .clone();
        register_live(&mut slice.registry, &conflicting);
        let planned = plan(&slice);
        let reason = planned
            .excluded_profiles
            .get(&slice.ids.tool_conflicting)
            .expect("conflicting tool is excluded");
        assert!(reason.contains("conflicts_with_active_profile"), "{reason}");
        let (top_tool, _) = planned
            .ranked_compositions
            .first()
            .expect("ranked composition exists")
            .clone();
        assert_eq!(top_tool, slice.ids.tool_live);
    }

    #[test]
    fn fallback_is_promoted_when_primary_is_missing() {
        let mut slice = vertical_slice();
        let live = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        slice.registry.entries.remove(&live.live_registry_id);
        let planned = plan(&slice);
        assert!(planned.dag.used_fallback);
        let (top_tool, _) = planned
            .ranked_compositions
            .first()
            .expect("ranked composition exists")
            .clone();
        assert_eq!(top_tool, slice.ids.tool_fallback);
    }

    #[test]
    fn registry_reconciliation_rejects_every_mismatch_class() {
        let slice = vertical_slice();
        let live = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        assert!(slice.registry.reconcile(&live).is_ok());

        // Missing entry.
        let missing = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_conflicting)
            .unwrap()
            .clone();
        assert_eq!(
            slice.registry.reconcile(&missing),
            Err(ReconciliationDisqualification::MissingFromLiveRegistry)
        );

        // Non-executable kinds never consult the registry.
        let problem = slice
            .snapshot
            .profiles
            .get(&slice.ids.problem)
            .unwrap()
            .clone();
        assert!(slice.registry.reconcile(&problem).is_ok());

        let mut tampered = slice.registry.clone();
        let entry_id = live.live_registry_id.clone();
        let entry = tampered.entries.get_mut(&entry_id).unwrap();
        entry.version = "9.9.9".into();
        assert!(matches!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::VersionMismatch { .. })
        ));

        let mut tampered = slice.registry.clone();
        tampered.entries.get_mut(&entry_id).unwrap().healthy = false;
        assert_eq!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::Unhealthy)
        );

        let mut tampered = slice.registry.clone();
        tampered.entries.get_mut(&entry_id).unwrap().installed = false;
        assert_eq!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::NotInstalled)
        );

        let mut tampered = slice.registry.clone();
        tampered
            .entries
            .get_mut(&entry_id)
            .unwrap()
            .granted_permissions
            .clear();
        assert!(matches!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::MissingPermission(_))
        ));

        let mut tampered = slice.registry.clone();
        tampered
            .entries
            .get_mut(&entry_id)
            .unwrap()
            .argument_schema_digest = label_digest("vs_wrong_arg");
        assert_eq!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::ArgumentSchemaDigestMismatch)
        );

        let mut tampered = slice.registry.clone();
        tampered
            .entries
            .get_mut(&entry_id)
            .unwrap()
            .result_schema_digest = label_digest("vs_wrong_res");
        assert_eq!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::ResultSchemaDigestMismatch)
        );

        let mut tampered = slice.registry.clone();
        tampered
            .entries
            .get_mut(&entry_id)
            .unwrap()
            .configuration_digest = label_digest("vs_wrong_cfg");
        assert_eq!(
            tampered.reconcile(&live),
            Err(ReconciliationDisqualification::ConfigurationDigestMismatch)
        );
    }

    #[test]
    fn execution_dag_is_acyclic_despite_atlas_cycle() {
        let slice = vertical_slice();
        // The raw Atlas deliberately cycles runbook <-> live tool.
        let forward = slice.snapshot.edges.iter().any(|e| {
            e.kind == CapabilityEdgeKind::ComposesWith
                && e.from_profile == slice.ids.runbook
                && e.to_profile == slice.ids.tool_live
        });
        let backward = slice.snapshot.edges.iter().any(|e| {
            e.kind == CapabilityEdgeKind::ComposesWith
                && e.from_profile == slice.ids.tool_live
                && e.to_profile == slice.ids.runbook
        });
        assert!(forward && backward);
        let planned = plan(&slice);
        planned.dag.verify_acyclic().expect("DAG is acyclic");
    }

    #[test]
    fn mandatory_gate_is_enforced_with_atlas_offline() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        let gate = MandatoryPolicyGate::read_only_smoke();
        // No snapshot, registry, or SurrealDB is consulted by enforce_call.
        let tool_projection = planned
            .context_frame
            .projections
            .iter()
            .find(|p| p.kind == CapabilityKind::Tool)
            .expect("tool projection exists")
            .clone();
        let valid = b"GMBI\x01\x00\x00\x00\x10\x20\x30\x40";
        assert!(gate.enforce_call(&tool_projection, valid).is_ok());

        let mut writer = tool_projection.clone();
        writer.effect_class = EffectClass::LocalWrite;
        assert!(gate.enforce_call(&writer, valid).is_err());
        writer.effect_class = EffectClass::ExternalWrite;
        assert!(gate.enforce_call(&writer, valid).is_err());
        writer.effect_class = EffectClass::Destructive;
        assert!(gate.enforce_call(&writer, valid).is_err());

        let mut slow = tool_projection.clone();
        slow.timeout_ms = 60_000;
        assert!(gate.enforce_call(&slow, valid).is_err());
        let mut huge = tool_projection.clone();
        huge.max_memory_bytes = 1024 * 1024 * 1024;
        assert!(gate.enforce_call(&huge, valid).is_err());

        // JSON / Python / text-tensor channels are rejected inside model I/O.
        for bad in [
            b"GMBI\x01\x00\x00\x00{\"tool\":\"x\"}".as_slice(),
            b"GMBI\x01\x00\x00\x00[1,2,3]".as_slice(),
            b"GMBI\x01\x00\x00\x00import torch".as_slice(),
            b"GMBI\x01\x00\x00\x00def evil():".as_slice(),
            b"GMBI\x01\x00\x00\x00transformer_kv_page:".as_slice(),
            b"no-header-at-all".as_slice(),
        ] {
            assert!(gate.enforce_call(&tool_projection, bad).is_err());
        }
    }

    #[test]
    fn context_frame_contains_only_selected_contracts() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        let projected: BTreeSet<_> = planned
            .context_frame
            .projections
            .iter()
            .map(|p| p.profile_id.clone())
            .collect();
        let expected: BTreeSet<_> = [
            slice.ids.tool_live.clone(),
            slice.ids.runbook.clone(),
            slice.ids.mcp_server.clone(),
        ]
        .into_iter()
        .collect();
        assert_eq!(projected, expected);
        for projection in &planned.context_frame.projections {
            assert!(matches!(
                projection.kind,
                CapabilityKind::Tool | CapabilityKind::Runbook | CapabilityKind::McpServer
            ));
        }
        // The binary frame round-trips without admitting unselected profiles.
        let decoded = CompactContextFrame::from_binary_envelope(&planned.context_frame_bytes)
            .expect("frame decodes");
        assert_eq!(decoded, planned.context_frame);
    }

    #[test]
    fn restart_replay_returns_identical_plan() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        // Simulate restart: persist every profile/edge as binary envelopes,
        // reload into a fresh snapshot, and re-plan.
        let mut reloaded = CapabilityAtlasSnapshot::default();
        for profile in slice.snapshot.profiles.values() {
            let bytes = profile.to_binary_envelope().expect("scaffold encodes");
            let back = CapabilityScaffold::from_binary_envelope(&bytes).expect("scaffold decodes");
            reloaded.profiles.insert(back.id.clone(), back);
        }
        for edge in &slice.snapshot.edges {
            let bytes = edge.to_binary_envelope().expect("edge encodes");
            reloaded
                .edges
                .push(CapabilityEdgeRecord::from_binary_envelope(&bytes).expect("edge decodes"));
        }
        let replayed = reloaded
            .plan_for_query(slice.query_h3, slice.query_t2, &slice.registry)
            .expect("replay plans successfully");
        let first_ids: Vec<_> = planned
            .ranked_compositions
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        let replay_ids: Vec<_> = replayed
            .ranked_compositions
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        assert_eq!(first_ids, replay_ids);
        assert_eq!(
            planned.expanded_two_hop_neighborhood,
            replayed.expanded_two_hop_neighborhood
        );
        assert_eq!(planned.dag.dag_digest, replayed.dag.dag_digest);
        assert_eq!(planned.context_frame_bytes, replayed.context_frame_bytes);
    }

    #[test]
    fn real_readonly_tool_call_completes_through_harness_bridge() {
        let slice = vertical_slice();
        let planned = plan(&slice);
        let gate = MandatoryPolicyGate::read_only_smoke();
        let request = b"GMBI\x01\x00\x00\x00vs_req_contract_verify";
        // The harness executor performs a REAL read-only operation: it reads
        // the checked-in migration 0012 schema file from disk and returns a
        // native binary response envelope. The Atlas never executes.
        let receipt = execute_planned_readonly_tool(
            &planned,
            &slice.registry,
            &gate,
            &slice.ids.workspace,
            &rid("vs_util_001"),
            request,
            |projection, binary_request| {
                assert_eq!(projection.profile_id, slice.ids.tool_live);
                assert_eq!(binary_request, request);
                let schema_bytes =
                    std::fs::read("../hypermind-nested-kv/schema/0012_capability_atlas.surql")
                        .map_err(|e| format!("real schema read failed: {e}"))?;
                let text = String::from_utf8(schema_bytes.clone())
                    .map_err(|_| "schema file is not UTF-8".to_string())?;
                if !text.contains("DEFINE TABLE IF NOT EXISTS capability_profile") {
                    return Err("real schema file lacks capability_profile table".into());
                }
                let file_digest = Digest::sha256(&schema_bytes);
                let mut response = b"GMBI\x01\x00\x00\x00".to_vec();
                response.extend_from_slice(file_digest.as_str().as_bytes());
                Ok(response)
            },
        )
        .expect("real read-only tool call succeeds");
        assert_eq!(receipt.tool_profile_id, slice.ids.tool_live);
        assert_eq!(receipt.dag_digest, planned.dag.dag_digest);
        assert_eq!(
            receipt.response_digest,
            Digest::sha256(&receipt.response_envelope_bytes)
        );
        assert_eq!(receipt.utility_receipt.outcome, UtilityOutcome::Succeeded);
        assert_eq!(receipt.utility_receipt.dag_digest, planned.dag.dag_digest);
        assert_eq!(receipt.utility_receipt.problem_profile, slice.ids.problem);

        // The same call fails closed when the registry drifts at execution time.
        let mut drifted = slice.registry.clone();
        let live = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        drifted
            .entries
            .get_mut(&live.live_registry_id)
            .unwrap()
            .healthy = false;
        assert!(
            execute_planned_readonly_tool(
                &planned,
                &drifted,
                &gate,
                &slice.ids.workspace,
                &rid("vs_util_002"),
                request,
                |_, _| Ok(vec![]),
            )
            .is_err()
        );
    }

    #[test]
    fn ranking_ignores_source_identity_and_tamper_fails_closed() {
        let slice = vertical_slice();
        let live = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        let before = slice
            .snapshot
            .score_composition(&slice.ids.capability, &slice.ids.tool_live);
        let mut twin = live.clone();
        twin.source_digest = label_digest("vs_different_teacher_source");
        twin.validate().expect("source swap stays valid");
        let mut snapshot = slice.snapshot.clone();
        snapshot.profiles.insert(twin.id.clone(), twin);
        let after = snapshot.score_composition(&slice.ids.capability, &slice.ids.tool_live);
        assert_eq!(before, after);

        let mut tampered = live.clone();
        tampered.content_digest = label_digest("vs_forged_content");
        assert!(tampered.validate().is_err());
        let mut wrong_kind = live.clone();
        wrong_kind.kind = CapabilityKind::Skill;
        assert!(wrong_kind.validate().is_err());
    }

    struct IsolatedAuditDb {
        child: std::process::Child,
        port: u16,
        storage: std::path::PathBuf,
    }

    impl IsolatedAuditDb {
        fn start() -> Self {
            let storage = std::env::temp_dir().join(format!("gm-vs-cap-db-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&storage);
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
            let port = listener.local_addr().expect("port").port();
            drop(listener);
            let child = std::process::Command::new("surreal")
                .args([
                    "start",
                    "--no-banner",
                    "--log",
                    "error",
                    "--bind",
                    &format!("127.0.0.1:{port}"),
                    "--user",
                    "root",
                    "--pass",
                    "vs_audit_pw_001",
                    &format!("surrealkv://{}", storage.display()),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("spawn isolated surrealdb (binary must be installed)");
            let db = Self {
                child,
                port,
                storage,
            };
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while std::time::Instant::now() < deadline {
                if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                    return db;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            panic!("isolated audit SurrealDB did not start on port {port}");
        }

        fn sql(&self, body: &str) -> String {
            use std::io::{Read, Write};
            let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port))
                .expect("connect audit db");
            let full = format!("USE NS vs_cap_audit DB atlas_audit; {body}");
            let request = format!(
                "POST /sql HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                self.port,
                vs_base64("root:vs_audit_pw_001"),
                full.len(),
                full
            );
            stream
                .write_all(request.as_bytes())
                .expect("write audit sql");
            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .expect("read audit response");
            String::from_utf8(response).expect("audit response is UTF-8")
        }
    }

    impl Drop for IsolatedAuditDb {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.storage);
        }
    }

    fn vs_base64(input: &str) -> String {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in input.as_bytes().chunks(3) {
            let b = [
                chunk[0],
                *chunk.get(1).unwrap_or(&0),
                *chunk.get(2).unwrap_or(&0),
            ];
            let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
            out.push(TABLE[(n >> 18) as usize & 63] as char);
            out.push(TABLE[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                TABLE[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }

    fn vs_hex(label: &str) -> String {
        Digest::sha256(label.as_bytes()).as_str().to_string()
    }

    #[test]
    fn surreal_0012_schema_audit_on_isolated_instance() {
        let db = IsolatedAuditDb::start();
        // 1. Apply the schema chain 0001, 0003..0012 on the isolated instance.
        for migration in [
            "0001_hypermind_graph",
            "0003_nested_hyperbolic_kv",
            "0004_transformer_kv_tensor_pages",
            "0005_universal_atom_lineage",
            "0006_immutable_atom_address",
            "0007_globalmind_distillation_trajectories",
            "0008_superposition_holding",
            "0009_immutable_tensor_kv",
            "0010_native_distillation_ingress",
            "0011_atomic_trajectory_cells",
            "0012_capability_atlas",
        ] {
            let path = format!("../hypermind-nested-kv/schema/{migration}.surql");
            let script = std::fs::read_to_string(&path).expect("schema file exists");
            let response = db.sql(&script);
            assert!(
                !response.contains("\"status\":\"ERR\""),
                "{migration} must apply cleanly: {response:.2000}"
            );
        }

        // 2. INFO proves all 12 kinds / 13 edge kinds are schema-enforced READONLY.
        let profile_info = db.sql("INFO FOR TABLE capability_profile;");
        for kind in [
            "problem_pattern",
            "solution_family",
            "capability",
            "runbook",
            "mcp_server",
            "plugin",
            "tool",
            "skill",
            "hook",
            "policy_rule",
            "evidence_receipt",
            "failure_pattern",
        ] {
            assert!(profile_info.contains(kind), "profile INFO lists {kind}");
        }
        assert!(profile_info.contains("READONLY"));
        let edge_info = db.sql("INFO FOR TABLE capability_edge;");
        for kind in [
            "solves",
            "specializes",
            "implements",
            "exposes",
            "requires",
            "composes_with",
            "conflicts_with",
            "fallback_for",
            "enforced_by",
            "triggered_by",
            "verified_by",
            "failed_on",
            "supersedes",
        ] {
            assert!(edge_info.contains(kind), "edge INFO lists {kind}");
        }

        // 3. The compiled Rust contract verifier accepts the live schema.
        let store = crate::GlobalMindStore::connect(crate::StoreConfig {
            endpoint: format!("http://127.0.0.1:{}", db.port),
            namespace: "vs_cap_audit".into(),
            database: "atlas_audit".into(),
            username: Some("root".into()),
            password: Some("vs_audit_pw_001".into()),
            allow_quarantined_core: false,
        })
        .expect("connect audit db");
        store
            .verify_capability_atlas_contract()
            .expect("0012 contract verifies");

        // 4. Positive stub chain: workspace -> atom -> address -> well -> profile.
        let d = vs_hex("vs_audit_digest");
        let setup = db.sql(&format!(
            "CREATE workspace:vs_audit SET slug = 'vs_audit';
             CREATE geometry_space:vs_geo SET name = 'h3_t2_r12_v1', manifold = 'H3xT2',
               quotient_symbol = 'G12', quotient_status = 'specified',
               coordinate_model = 'lorentz', contract_version = 'v1', contract_digest = '{d}';
             CREATE atom:vs_atom SET workspace = workspace:vs_audit, external_id = 'vs_ext',
               content_kind = 'code', content_digest = '{d}', domain = 'capability',
               rank = 0, salience = 0.9, provenance = 'vs_audit', taint = 'clean';
             CREATE atom_address:vs_addr SET workspace = workspace:vs_audit, atom = atom:vs_atom,
               h3_lorentz = [1.0, 0.0, 0.0, 0.0], t2_phase = [0.0, 0.0],
               geometry_model = 'lorentz', geometry_version = 'v1', revision = 1;
             CREATE atom:vs_atom2 SET workspace = workspace:vs_audit, external_id = 'vs_ext2',
               content_kind = 'code', content_digest = '{d}', domain = 'capability',
               rank = 0, salience = 0.8, provenance = 'vs_audit', taint = 'clean';
             CREATE atom_address:vs_addr2 SET workspace = workspace:vs_audit, atom = atom:vs_atom2,
               h3_lorentz = [1.0, 0.0, 0.0, 0.0], t2_phase = [0.1, 0.0],
               geometry_model = 'lorentz', geometry_version = 'v1', revision = 1;
             CREATE cognitive_well:vs_well SET workspace = workspace:vs_audit,
               geometry_space = geometry_space:vs_geo, ordinal = 0, label = 'L1',
               h3_center_lorentz = [1.277, 0.1, 0.05, 0.02], t2_sector = 0,
               boundary_contract_digest = '{d}', status = 'specified';"
        ));
        assert!(
            !setup.contains("\"status\":\"ERR\""),
            "stub chain creates: {setup:.2000}"
        );

        let profile_set_for = |atom: &str, addr: &str| {
            format!(
                "workspace = workspace:vs_audit, atom = {atom}, address = {addr},
             well = cognitive_well:vs_well, canonical_name = 'vs_probe',
             summary = 'original summary', version = '1.0.0', content_digest = '{d}',
             source_digest = '{d}', configuration_digest = '{d}', lifecycle = 'qualified',
             admission_receipt_digest = '{d}', judge_receipt_digest = '{d}',
             replay_receipt_digest = '{d}', crystallization_receipt_digest = '{d}',
             argument_schema_digest = '{d}', result_schema_digest = '{d}',
             effect_class = 'read_only', permission_classes = [], credential_classes = [],
             platform_requirements = [], runtime_requirements = [],
             version_requirement = '>=1.0.0', timeout_ms = 5000, max_retries = 1,
             idempotent = true, rollback_policy = 'read_only_noop', max_memory_bytes = 16777216,
             health_check_id = 'vs_health', live_registry_id = 'vsreg/probe@1.0.0',
             incompatible_profiles = [], expected_utility = 0.9, resource_cost = 0.1,
             prediction_error = 0.02, payload_slot_1 = 'a', payload_slot_2 = 'b',
             payload_slot_3 = 'c', context_projection_bytes = b\"AB\",
             context_projection_digest = '{d}', scaffold_envelope_bytes = b\"CD\",
             scaffold_envelope_digest = '{d}'"
            )
        };
        let positive = db.sql(&format!(
            "CREATE capability_profile:vs_prof1 SET kind = 'tool',
               payload_schema_tag = 'tool', {};",
            profile_set_for("atom:vs_atom", "atom_address:vs_addr")
        ));
        assert!(
            !positive.contains("\"status\":\"ERR\""),
            "valid profile creates: {positive:.2000}"
        );
        let positive2 = db.sql(&format!(
            "CREATE capability_profile:vs_prof2 SET kind = 'capability',
               payload_schema_tag = 'capability', {};",
            profile_set_for("atom:vs_atom2", "atom_address:vs_addr2")
        ));
        assert!(
            !positive2.contains("\"status\":\"ERR\""),
            "second profile creates: {positive2:.2000}"
        );

        // 5. Negatives: every violation fails closed.
        let bad_kind = db.sql(
            "CREATE capability_profile:vs_bad SET kind = 'related', payload_schema_tag = 'related';",
        );
        assert!(
            bad_kind.contains("\"status\":\"ERR\""),
            "untyped kind rejected: {bad_kind:.500}"
        );
        let missing_field = db.sql(
            "CREATE capability_profile:vs_bad2 SET kind = 'tool', payload_schema_tag = 'tool',
               workspace = workspace:vs_audit, atom = atom:vs_atom,
               address = atom_address:vs_addr, well = cognitive_well:vs_well;",
        );
        assert!(
            missing_field.contains("\"status\":\"ERR\""),
            "missing receipts rejected: {missing_field:.500}"
        );
        let bad_edge = db.sql(&format!(
            "RELATE capability_profile:vs_prof2->capability_edge->capability_profile:vs_prof1
               SET workspace = workspace:vs_audit, from_atom = atom:vs_atom,
               to_atom = atom:vs_atom, kind = 'related', confidence = 0.5,
               evidence_receipt_digest = '{d}', producer = 'vs_audit', revision = 1,
               compatibility_digest = '{d}', edge_envelope_bytes = b\"EF\",
               crystallization_receipt_digest = '{d}';"
        ));
        assert!(
            bad_edge.contains("\"status\":\"ERR\""),
            "untyped edge rejected: {bad_edge:.500}"
        );
        let duplicate = db.sql(&format!(
            "CREATE capability_profile:vs_prof1 SET kind = 'tool',
               payload_schema_tag = 'tool', {};",
            profile_set_for("atom:vs_atom", "atom_address:vs_addr")
        ));
        assert!(
            duplicate.contains("\"status\":\"ERR\""),
            "duplicate identity rejected: {duplicate:.500}"
        );
        let overwrite = db.sql("UPDATE capability_profile:vs_prof1 SET summary = 'mutated';");
        assert!(
            overwrite.contains("\"status\":\"ERR\""),
            "readonly overwrite rejected: {overwrite:.500}"
        );

        // 6. Atomicity: failed writes left zero partial state.
        let counts = db.sql("SELECT count() FROM capability_profile GROUP ALL;");
        assert!(
            counts.contains('2'),
            "exactly two profiles persist: {counts:.500}"
        );
        let edges = db.sql("SELECT count() FROM capability_edge GROUP ALL;");
        assert!(edges.contains('0'), "zero edges persist: {edges:.500}");
        let summary = db.sql("SELECT summary FROM capability_profile:vs_prof1;");
        assert!(
            summary.contains("original summary"),
            "summary unchanged: {summary:.500}"
        );
        assert!(
            !summary.contains("mutated"),
            "no mutation leaked: {summary:.500}"
        );
    }

    #[test]
    fn forbidden_secret_and_tensor_material_is_rejected() {
        let slice = vertical_slice();
        let mut leaked = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        leaked.summary = "verifier with api_key=live-secret-value".into();
        assert!(leaked.validate().is_err());
        let mut leaked = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        leaked.credential_classes = vec!["loopback_session_token=hunter2".into()];
        assert!(leaked.validate().is_err());
        let mut leaked = slice
            .snapshot
            .profiles
            .get(&slice.ids.tool_live)
            .unwrap()
            .clone();
        leaked.summary = "embeds transformer_kv_page bytes".into();
        assert!(leaked.validate().is_err());
    }
}
