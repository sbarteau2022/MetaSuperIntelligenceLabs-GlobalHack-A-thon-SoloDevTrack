use super::{
    CapabilityAtlasSnapshot, CapabilityEdgeKind, CapabilityEdgeRecord, CapabilityScaffold,
    CapabilityUtilityReceipt, CompactCallableProjection, deterministic_digest, digest_bytes,
};
use crate::store::{
    bytes_literal, json_bytes, optional_record, optional_string, record, record_ref, sq,
};
use crate::whitehead::{WHITEHEAD_COORDINATE_MODEL, WHITEHEAD_SPACE_NAME, WhiteheadTopology};
use crate::{Digest, GlobalMindStore, RecordId};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Receipt returned when a Capability Atlas bundle passes verification,
/// independent judgment, atomic SurrealDB admission, neighborhood replay,
/// and crystallization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrystallizedAtlasReceipt {
    pub workspace: RecordId,
    pub profile_count: usize,
    pub edge_count: usize,
    pub bundle_digest: Digest,
    pub replay_receipt_digest: Digest,
}

impl GlobalMindStore {
    /// Verifies that migration `0012_capability_atlas.surql` is applied and
    /// that all required fields on `capability_profile`, `capability_edge`,
    /// and `capability_utility_receipt` are `READONLY`.
    pub fn verify_capability_atlas_contract(&self) -> Result<(), String> {
        for (table, required) in [
            (
                "capability_profile",
                &[
                    "workspace",
                    "atom",
                    "address",
                    "well",
                    "kind",
                    "canonical_name",
                    "summary",
                    "version",
                    "content_digest",
                    "source_digest",
                    "configuration_digest",
                    "lifecycle",
                    "admission_receipt_digest",
                    "judge_receipt_digest",
                    "replay_receipt_digest",
                    "crystallization_receipt_digest",
                    "argument_schema_digest",
                    "result_schema_digest",
                    "effect_class",
                    "permission_classes",
                    "credential_classes",
                    "platform_requirements",
                    "runtime_requirements",
                    "version_requirement",
                    "timeout_ms",
                    "max_retries",
                    "idempotent",
                    "rollback_policy",
                    "max_memory_bytes",
                    "health_check_id",
                    "live_registry_id",
                    "component_receipt_digest",
                    "integration_receipt_digest",
                    "live_receipt_digest",
                    "incompatible_profiles",
                    "benched_reason",
                    "expected_utility",
                    "resource_cost",
                    "prediction_error",
                    "payload_schema_tag",
                    "payload_slot_1",
                    "payload_slot_2",
                    "payload_slot_3",
                    "payload_slot_4",
                    "context_projection_bytes",
                    "context_projection_digest",
                    "scaffold_envelope_bytes",
                    "scaffold_envelope_digest",
                ][..],
            ),
            (
                "capability_edge",
                &[
                    "workspace",
                    "from_atom",
                    "to_atom",
                    "kind",
                    "confidence",
                    "evidence_receipt",
                    "evidence_receipt_digest",
                    "producer",
                    "revision",
                    "valid_from",
                    "valid_to",
                    "compatibility_digest",
                    "edge_envelope_bytes",
                    "crystallization_receipt_digest",
                ][..],
            ),
            (
                "capability_utility_receipt",
                &[
                    "workspace",
                    "problem_profile",
                    "selected_profiles",
                    "dag_digest",
                    "execution_receipt_digest",
                    "observed_utility",
                    "prediction_error",
                    "resource_cost",
                    "outcome",
                ][..],
            ),
        ] {
            let result = self.one(&format!("INFO FOR TABLE {table};"))?;
            let fields = result
                .get("fields")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("Capability Atlas table {table} is missing"))?;
            for field in required {
                let definition = fields
                    .get(*field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("Capability Atlas field {table}.{field} is missing"))?;
                if !definition.contains("READONLY") {
                    return Err(format!(
                        "Capability Atlas field {table}.{field} is not READONLY"
                    ));
                }
            }
        }
        Ok(())
    }

    /// Executes the full Capability Atlas scaffolding transaction:
    /// 1. Verify source and contracts;
    /// 2. Independently judge every profile and edge;
    /// 3. Create semantic `atom`, `atom_address`, `address_well_membership`,
    ///    and `capability_profile` records;
    /// 4. Create typed `capability_edge` records;
    /// 5. Replay neighborhood queries from SurrealDB and verify binary/digest
    ///    parity;
    /// 6. Crystallize and return `CrystallizedAtlasReceipt`.
    pub fn scaffold_capability_bundle(
        &self,
        workspace: &RecordId,
        topology: &WhiteheadTopology,
        profiles: &[CapabilityScaffold],
        edges: &[CapabilityEdgeRecord],
    ) -> Result<CrystallizedAtlasReceipt, String> {
        self.verify_capability_atlas_contract()?;
        if profiles.is_empty() {
            return Err("scaffold_capability_bundle requires at least one profile".into());
        }

        // Step 1 & 2: Verify contracts and independent judge bindings.
        let mut profile_by_id = BTreeMap::new();
        let mut seen_atoms = BTreeSet::new();
        for profile in profiles {
            if &profile.workspace != workspace {
                return Err(format!(
                    "capability profile {} workspace mismatch",
                    profile.id.as_str()
                ));
            }
            profile.validate()?;
            if profile_by_id.insert(profile.id.clone(), profile).is_some() {
                return Err(format!(
                    "duplicate capability profile id {}",
                    profile.id.as_str()
                ));
            }
            if !seen_atoms.insert(profile.atom_id.clone()) {
                return Err(format!(
                    "multiple capability profiles cannot share atom {}",
                    profile.atom_id.as_str()
                ));
            }
        }

        for edge in edges {
            if &edge.workspace != workspace {
                return Err(format!(
                    "capability edge {} workspace mismatch",
                    edge.id.as_str()
                ));
            }
            edge.validate()?;
            let from_prof = profile_by_id.get(&edge.from_profile).ok_or_else(|| {
                format!(
                    "capability edge {} references unknown from_profile {}",
                    edge.id.as_str(),
                    edge.from_profile.as_str()
                )
            })?;
            let to_prof = profile_by_id.get(&edge.to_profile).ok_or_else(|| {
                format!(
                    "capability edge {} references unknown to_profile {}",
                    edge.id.as_str(),
                    edge.to_profile.as_str()
                )
            })?;
            if from_prof.atom_id != edge.from_atom || to_prof.atom_id != edge.to_atom {
                return Err(format!(
                    "capability edge {} atom endpoints do not match profile atoms",
                    edge.id.as_str()
                ));
            }
            edge.kind.validate_endpoints(from_prof.kind, to_prof.kind)?;
            if let Some(ev_id) = &edge.evidence_receipt {
                let ev_prof = profile_by_id.get(ev_id).ok_or_else(|| {
                    format!(
                        "capability edge {} references unknown evidence_receipt {}",
                        edge.id.as_str(),
                        ev_id.as_str()
                    )
                })?;
                if ev_prof.kind != super::CapabilityKind::EvidenceReceipt {
                    return Err(format!(
                        "capability edge {} evidence_receipt {} is not an evidence_receipt profile",
                        edge.id.as_str(),
                        ev_id.as_str()
                    ));
                }
            }
        }

        // Step 3 & 4: Build atomic SurrealDB transaction.
        let ws = record("workspace", workspace);
        let mut query = String::from("BEGIN TRANSACTION; ");

        for profile in profiles {
            Self::append_profile_create_sql(&mut query, &ws, topology, profile)?;
        }
        for edge in edges {
            Self::append_edge_relate_sql(&mut query, &ws, edge)?;
        }
        query.push_str("COMMIT TRANSACTION;");
        self.transaction(&query)?;

        // Step 5: Replay neighborhood queries from SurrealDB and verify exact parity.
        let snapshot = self.load_capability_atlas_snapshot(workspace)?;
        for profile in profiles {
            let loaded = snapshot.profiles.get(&profile.id).ok_or_else(|| {
                format!(
                    "replay failed: capability profile {} missing after transaction",
                    profile.id.as_str()
                )
            })?;
            if loaded.content_digest != profile.content_digest
                || loaded.argument_schema_digest != profile.argument_schema_digest
                || loaded.result_schema_digest != profile.result_schema_digest
            {
                return Err(format!(
                    "replay failed: capability profile {} digest mismatch",
                    profile.id.as_str()
                ));
            }
        }
        for edge in edges {
            if !snapshot.edges.iter().any(|e| {
                e.id == edge.id
                    && e.kind == edge.kind
                    && e.from_profile == edge.from_profile
                    && e.to_profile == edge.to_profile
                    && e.compatibility_digest == edge.compatibility_digest
            }) {
                return Err(format!(
                    "replay failed: capability edge {} missing or mismatched after transaction",
                    edge.id.as_str()
                ));
            }
        }

        // Step 6: Compute bundle and replay receipts.
        let mut bundle_hasher = Sha256::new();
        bundle_hasher.update(b"globalmind-capability-bundle-v1:");
        for profile in profiles {
            bundle_hasher.update(profile.id.as_str().as_bytes());
            bundle_hasher.update(b":");
            bundle_hasher.update(profile.content_digest.as_str().as_bytes());
            bundle_hasher.update(b";");
        }
        for edge in edges {
            bundle_hasher.update(edge.id.as_str().as_bytes());
            bundle_hasher.update(b":");
            bundle_hasher.update(edge.compatibility_digest.as_str().as_bytes());
            bundle_hasher.update(b";");
        }
        let bundle_hex = hex_lower(&bundle_hasher.finalize());
        let bundle_digest = Digest::parse(bundle_hex.clone())?;
        let replay_receipt_digest = deterministic_digest(&format!("replay:{bundle_hex}"));

        Ok(CrystallizedAtlasReceipt {
            workspace: workspace.clone(),
            profile_count: profiles.len(),
            edge_count: edges.len(),
            bundle_digest,
            replay_receipt_digest,
        })
    }

    /// Creates a new immutable version of a capability profile and links it to
    /// the prior version with a `supersedes` edge, without overwriting or
    /// mutating the qualified history of the prior profile.
    pub fn supersede_capability_profile(
        &self,
        workspace: &RecordId,
        topology: &WhiteheadTopology,
        older_profile_id: &RecordId,
        newer_profile: &CapabilityScaffold,
        supersedes_edge: &CapabilityEdgeRecord,
    ) -> Result<Digest, String> {
        self.verify_capability_atlas_contract()?;
        newer_profile.validate()?;
        supersedes_edge.validate()?;
        if supersedes_edge.kind != CapabilityEdgeKind::Supersedes {
            return Err("supersede_capability_profile requires a 'supersedes' edge".into());
        }
        if supersedes_edge.from_profile != newer_profile.id
            || &supersedes_edge.to_profile != older_profile_id
        {
            return Err("supersedes edge must point from newer_profile to older_profile_id".into());
        }

        let older_rows = self.one(&format!(
            "SELECT kind, canonical_name, version, content_digest FROM {} LIMIT 1;",
            record("capability_profile", older_profile_id)
        ))?;
        let older = older_rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| {
                format!(
                    "older capability_profile {} does not exist",
                    older_profile_id.as_str()
                )
            })?;
        let older_kind = older
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| "older capability_profile kind missing".to_string())?;
        let older_name = older
            .get("canonical_name")
            .and_then(Value::as_str)
            .ok_or_else(|| "older capability_profile canonical_name missing".to_string())?;
        let older_version = older
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| "older capability_profile version missing".to_string())?;
        let older_digest = older
            .get("content_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| "older capability_profile content_digest missing".to_string())?
            .to_string();

        if older_kind != newer_profile.kind.as_str() || older_name != newer_profile.canonical_name {
            return Err(
                "superseding profile must have the same kind and canonical_name as the older profile"
                    .into(),
            );
        }
        if older_version == newer_profile.version {
            return Err(
                "superseding profile must have a new version string rather than overwriting".into(),
            );
        }

        let ws = record("workspace", workspace);
        let mut query = String::from("BEGIN TRANSACTION; ");
        Self::append_profile_create_sql(&mut query, &ws, topology, newer_profile)?;
        Self::append_edge_relate_sql(&mut query, &ws, supersedes_edge)?;
        query.push_str("COMMIT TRANSACTION;");
        self.transaction(&query)?;

        // Prove the older profile's content_digest and version remain untouched.
        let verify_rows = self.one(&format!(
            "SELECT version, content_digest FROM {} LIMIT 1;",
            record("capability_profile", older_profile_id)
        ))?;
        let verify_older = verify_rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "older profile disappeared after supersede".to_string())?;
        if verify_older.get("version").and_then(Value::as_str) != Some(older_version)
            || verify_older.get("content_digest").and_then(Value::as_str)
                != Some(older_digest.as_str())
        {
            return Err("qualified history of older profile was mutated".into());
        }

        Ok(newer_profile.content_digest.clone())
    }

    /// Loads all `capability_profile` and `capability_edge` records for
    /// `workspace`, verifying every binary envelope and SHA-256 digest on
    /// readback.
    pub fn load_capability_atlas_snapshot(
        &self,
        workspace: &RecordId,
    ) -> Result<CapabilityAtlasSnapshot, String> {
        let ws_ref = record_ref("workspace", workspace);
        let profile_rows = self.one(&format!(
            "SELECT meta::id(id) AS raw_id, workspace, scaffold_envelope_bytes, scaffold_envelope_digest, context_projection_bytes, context_projection_digest FROM capability_profile WHERE workspace = {} ORDER BY raw_id ASC;",
            record("workspace", workspace)
        ))?;
        let profile_arr = profile_rows
            .as_array()
            .ok_or_else(|| "capability_profile query did not return an array".to_string())?;

        let mut profiles = BTreeMap::new();
        for row in profile_arr {
            if row.get("workspace").and_then(Value::as_str) != Some(ws_ref.as_str()) {
                return Err("capability_profile workspace mismatch on readback".into());
            }
            let env_bytes = json_bytes(
                row.get("scaffold_envelope_bytes"),
                "scaffold_envelope_bytes",
            )?;
            let env_digest_str = row
                .get("scaffold_envelope_digest")
                .and_then(Value::as_str)
                .ok_or_else(|| "scaffold_envelope_digest missing".to_string())?;
            let computed_env_digest = digest_bytes(&env_bytes)?;
            if computed_env_digest.as_str() != env_digest_str {
                return Err(format!(
                    "capability_profile scaffold_envelope_digest mismatch: expected {}, got {}",
                    env_digest_str,
                    computed_env_digest.as_str()
                ));
            }
            let proj_bytes = json_bytes(
                row.get("context_projection_bytes"),
                "context_projection_bytes",
            )?;
            let proj_digest_str = row
                .get("context_projection_digest")
                .and_then(Value::as_str)
                .ok_or_else(|| "context_projection_digest missing".to_string())?;
            let computed_proj_digest = digest_bytes(&proj_bytes)?;
            if computed_proj_digest.as_str() != proj_digest_str {
                return Err("capability_profile context_projection_digest mismatch".into());
            }
            let scaffold = CapabilityScaffold::from_binary_envelope(&env_bytes)?;
            let decoded_proj = CompactCallableProjection::from_binary_envelope(&proj_bytes)?;
            if decoded_proj != scaffold.compact_projection() {
                return Err("capability_profile decoded projection does not match scaffold".into());
            }
            profiles.insert(scaffold.id.clone(), scaffold);
        }

        let edge_rows = self.one(&format!(
            "SELECT meta::id(id) AS raw_id, workspace, edge_envelope_bytes FROM capability_edge WHERE workspace = {} ORDER BY raw_id ASC;",
            record("workspace", workspace)
        ))?;
        let edge_arr = edge_rows
            .as_array()
            .ok_or_else(|| "capability_edge query did not return an array".to_string())?;

        let mut edges = Vec::with_capacity(edge_arr.len());
        for row in edge_arr {
            if row.get("workspace").and_then(Value::as_str) != Some(ws_ref.as_str()) {
                return Err("capability_edge workspace mismatch on readback".into());
            }
            let edge_bytes = json_bytes(row.get("edge_envelope_bytes"), "edge_envelope_bytes")?;
            let edge = CapabilityEdgeRecord::from_binary_envelope(&edge_bytes)?;
            edges.push(edge);
        }

        Ok(CapabilityAtlasSnapshot { profiles, edges })
    }

    /// Appends an immutable `capability_utility_receipt` after harness
    /// execution without mutating any semantic `atom_address` coordinate.
    pub fn append_capability_utility_receipt(
        &self,
        receipt: &CapabilityUtilityReceipt,
    ) -> Result<(), String> {
        let selected = receipt
            .selected_profiles
            .iter()
            .map(|id| record("capability_profile", id))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ \
               workspace: {workspace}, \
               problem_profile: {problem_profile}, \
               selected_profiles: [{selected}], \
               dag_digest: {dag_digest}, \
               execution_receipt_digest: {exec_digest}, \
               observed_utility: {utility:?}, \
               prediction_error: {pred_err:?}, \
               resource_cost: {cost:?}, \
               outcome: {outcome}, \
               recorded_at: time::now() \
             }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("capability_utility_receipt", &receipt.id),
            workspace = record("workspace", &receipt.workspace),
            problem_profile = record("capability_profile", &receipt.problem_profile),
            selected = selected,
            dag_digest = sq(receipt.dag_digest.as_str()),
            exec_digest = sq(receipt.execution_receipt_digest.as_str()),
            utility = receipt.observed_utility,
            pred_err = receipt.prediction_error,
            cost = receipt.resource_cost,
            outcome = sq(receipt.outcome.as_str()),
        );
        self.transaction(&query)
    }

    fn append_profile_create_sql(
        query: &mut String,
        ws: &str,
        topology: &WhiteheadTopology,
        profile: &CapabilityScaffold,
    ) -> Result<(), String> {
        let classification = topology.classify_well(profile.h3_lorentz, profile.t2_phase)?;
        let h3_list = profile
            .h3_lorentz
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let t2_list = profile
            .t2_phase
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");

        let proj = profile.compact_projection();
        let proj_bytes = proj.to_binary_envelope()?;
        let proj_digest = digest_bytes(&proj_bytes)?;
        let env_bytes = profile.to_binary_envelope()?;
        let env_digest = digest_bytes(&env_bytes)?;

        let (slot1, slot2, slot3, slot4) = profile.payload.slots();
        let perms = profile
            .permission_classes
            .iter()
            .map(|s| sq(s))
            .collect::<Vec<_>>()
            .join(", ");
        let creds = profile
            .credential_classes
            .iter()
            .map(|s| sq(s))
            .collect::<Vec<_>>()
            .join(", ");
        let platforms = profile
            .platform_requirements
            .iter()
            .map(|s| sq(s))
            .collect::<Vec<_>>()
            .join(", ");
        let runtimes = profile
            .runtime_requirements
            .iter()
            .map(|s| sq(s))
            .collect::<Vec<_>>()
            .join(", ");
        let incompat = profile
            .incompatible_profiles
            .iter()
            .map(|id| record("capability_profile", id))
            .collect::<Vec<_>>()
            .join(", ");

        query.push_str(&format!(
            "CREATE {atom_id} CONTENT {{ \
               workspace: {ws}, \
               external_id: {ext_id}, \
               content_kind: 'code', \
               content_text: {summary}, \
               content_uri: NONE, \
               content_digest: {content_digest}, \
               domain: 'capability_atlas', \
               rank: 0, \
               salience: {salience:?}, \
               provenance: 'capability_atlas_v1', \
               taint: 'clean', \
               pinned: true, \
               current_address: {address_id}, \
               created_at: time::now(), \
               updated_at: time::now(), \
               distillation_run: NONE \
             }} RETURN NONE; \
             CREATE {address_id} CONTENT {{ \
               workspace: {ws}, \
               atom: {atom_id}, \
               h3_lorentz: [{h3_list}], \
               t2_phase: [{t2_list}], \
               geometry_model: {geom_model}, \
               geometry_version: {geom_version}, \
               revision: 1, \
               active: true, \
               created_at: time::now(), \
               space: {geom_space} \
             }} RETURN NONE; \
             CREATE {membership_id} CONTENT {{ \
               workspace: {ws}, \
               address: {address_id}, \
               well: {well_id}, \
               classifier_version: {cls_version}, \
               classifier_digest: {cls_digest}, \
               boundary_receipt_digest: {bnd_digest}, \
               classified_at: time::now() \
             }} RETURN NONE; \
             CREATE {prof_id} CONTENT {{ \
               workspace: {ws}, \
               atom: {atom_id}, \
               address: {address_id}, \
               well: {well_id}, \
               kind: {kind}, \
               canonical_name: {canonical_name}, \
               summary: {summary}, \
               version: {version}, \
               content_digest: {content_digest}, \
               source_digest: {source_digest}, \
               configuration_digest: {config_digest}, \
               lifecycle: {lifecycle}, \
               admission_receipt_digest: {adm_digest}, \
               judge_receipt_digest: {judge_digest}, \
               replay_receipt_digest: {replay_digest}, \
               crystallization_receipt_digest: {crys_digest}, \
               argument_schema_digest: {arg_digest}, \
               result_schema_digest: {res_digest}, \
               effect_class: {effect_class}, \
               permission_classes: [{perms}], \
               credential_classes: [{creds}], \
               platform_requirements: [{platforms}], \
               runtime_requirements: [{runtimes}], \
               version_requirement: {ver_req}, \
               timeout_ms: {timeout_ms}, \
               max_retries: {max_retries}, \
               idempotent: {idempotent}, \
               rollback_policy: {rollback}, \
               max_memory_bytes: {max_mem}, \
               health_check_id: {health_id}, \
               live_registry_id: {live_id}, \
               component_receipt_digest: {comp_digest}, \
               integration_receipt_digest: {int_digest}, \
               live_receipt_digest: {live_digest}, \
               incompatible_profiles: [{incompat}], \
               benched_reason: {benched_reason}, \
               expected_utility: {utility:?}, \
               resource_cost: {cost:?}, \
               prediction_error: {pred_err:?}, \
               payload_schema_tag: {kind}, \
               payload_slot_1: {slot1}, \
               payload_slot_2: {slot2}, \
               payload_slot_3: {slot3}, \
               payload_slot_4: {slot4}, \
               context_projection_bytes: {proj_bytes}, \
               context_projection_digest: {proj_digest}, \
               scaffold_envelope_bytes: {env_bytes}, \
               scaffold_envelope_digest: {env_digest}, \
               created_at: time::now() \
             }} RETURN NONE; ",
            atom_id = record("atom", &profile.atom_id),
            address_id = record("atom_address", &profile.address_id),
            membership_id = record("address_well_membership", &profile.membership_id),
            prof_id = record("capability_profile", &profile.id),
            ws = ws,
            ext_id = sq(&format!(
                "cap:{}:{}:{}",
                profile.kind.as_str(),
                profile.canonical_name,
                profile.version
            )),
            summary = sq(&profile.summary),
            content_digest = sq(profile.content_digest.as_str()),
            salience = profile.expected_utility,
            h3_list = h3_list,
            t2_list = t2_list,
            geom_model = sq(WHITEHEAD_COORDINATE_MODEL),
            geom_version = sq(WHITEHEAD_SPACE_NAME),
            geom_space = record("geometry_space", &profile.geometry_space),
            well_id = record("cognitive_well", &profile.well),
            cls_version = sq(&classification.classifier_version),
            cls_digest = sq(classification.classifier_digest.as_str()),
            bnd_digest = sq(classification.boundary_receipt_digest.as_str()),
            kind = sq(profile.kind.as_str()),
            canonical_name = sq(&profile.canonical_name),
            version = sq(&profile.version),
            source_digest = sq(profile.source_digest.as_str()),
            config_digest = sq(profile.configuration_digest.as_str()),
            lifecycle = sq(profile.lifecycle.as_str()),
            adm_digest = sq(profile.admission_receipt_digest.as_str()),
            judge_digest = sq(profile.judge_receipt_digest.as_str()),
            replay_digest = sq(profile.replay_receipt_digest.as_str()),
            crys_digest = sq(profile.crystallization_receipt_digest.as_str()),
            arg_digest = sq(profile.argument_schema_digest.as_str()),
            res_digest = sq(profile.result_schema_digest.as_str()),
            effect_class = sq(profile.effect_class.as_str()),
            perms = perms,
            creds = creds,
            platforms = platforms,
            runtimes = runtimes,
            ver_req = sq(&profile.version_requirement),
            timeout_ms = profile.timeout_ms,
            max_retries = profile.max_retries,
            idempotent = profile.idempotent,
            rollback = sq(&profile.rollback_policy),
            max_mem = profile.max_memory_bytes,
            health_id = sq(&profile.health_check_id),
            live_id = sq(&profile.live_registry_id),
            comp_digest = optional_string(
                profile
                    .component_receipt_digest
                    .as_ref()
                    .map(Digest::as_str)
            ),
            int_digest = optional_string(
                profile
                    .integration_receipt_digest
                    .as_ref()
                    .map(Digest::as_str)
            ),
            live_digest = optional_string(profile.live_receipt_digest.as_ref().map(Digest::as_str)),
            incompat = incompat,
            benched_reason = optional_string(profile.benched_reason.as_deref()),
            utility = profile.expected_utility,
            cost = profile.resource_cost,
            pred_err = profile.prediction_error,
            slot1 = sq(slot1),
            slot2 = sq(slot2),
            slot3 = sq(slot3),
            slot4 = optional_string(slot4),
            proj_bytes = bytes_literal(&proj_bytes),
            proj_digest = sq(proj_digest.as_str()),
            env_bytes = bytes_literal(&env_bytes),
            env_digest = sq(env_digest.as_str()),
        ));
        Ok(())
    }

    fn append_edge_relate_sql(
        query: &mut String,
        ws: &str,
        edge: &CapabilityEdgeRecord,
    ) -> Result<(), String> {
        let edge_bytes = edge.to_binary_envelope()?;
        let valid_from_iso = format!("d'1970-01-01T00:00:00Z' + {}ms", edge.valid_from_unix_ms);
        let valid_to_expr = match edge.valid_to_unix_ms {
            Some(ms) => format!("d'1970-01-01T00:00:00Z' + {ms}ms"),
            None => "NONE".into(),
        };
        query.push_str(&format!(
            "RELATE {from_prof}->{edge_id}->{to_prof} CONTENT {{ \
               workspace: {ws}, \
               from_atom: {from_atom}, \
               to_atom: {to_atom}, \
               kind: {kind}, \
               confidence: {confidence:?}, \
               evidence_receipt: {ev_rec}, \
               evidence_receipt_digest: {ev_digest}, \
               producer: {producer}, \
               revision: {revision}, \
               valid_from: {valid_from}, \
               valid_to: {valid_to}, \
               compatibility_digest: {compat_digest}, \
               edge_envelope_bytes: {edge_bytes}, \
               crystallization_receipt_digest: {crys_digest}, \
               created_at: time::now() \
             }} RETURN NONE; ",
            from_prof = record("capability_profile", &edge.from_profile),
            edge_id = record("capability_edge", &edge.id),
            to_prof = record("capability_profile", &edge.to_profile),
            ws = ws,
            from_atom = record("atom", &edge.from_atom),
            to_atom = record("atom", &edge.to_atom),
            kind = sq(edge.kind.as_str()),
            confidence = edge.confidence,
            ev_rec = optional_record("capability_profile", edge.evidence_receipt.as_ref()),
            ev_digest = sq(edge.evidence_receipt_digest.as_str()),
            producer = sq(&edge.producer),
            revision = edge.revision,
            valid_from = valid_from_iso,
            valid_to = valid_to_expr,
            compat_digest = sq(edge.compatibility_digest.as_str()),
            edge_bytes = bytes_literal(&edge_bytes),
            crys_digest = sq(edge.crystallization_receipt_digest.as_str()),
        ));
        Ok(())
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}
