use crate::{
    CrystallizedAtom, Digest, DistillationPacket, DistillationPhase, DistillationRunEvent,
    ForwardMode, LocalJacobianChart, PredictionErrorObservation, RecordId, SuperpositionHold,
    SuperpositionIteration, SuperpositionResolution, TensorPage, TensorPayload, TensorSession,
    Trajectory, VerifiedManifest, WHITEHEAD_COORDINATE_MODEL, WHITEHEAD_MANIFOLD,
    WHITEHEAD_QUOTIENT_SYMBOL, WHITEHEAD_SPACE_NAME, WellClassification, WhiteheadTopology,
};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct StoreConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub allow_quarantined_core: bool,
}

impl StoreConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("GLOBALMIND_SURREAL_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000".into()),
            namespace: std::env::var("GLOBALMIND_SURREAL_NS")
                .unwrap_or_else(|_| "hypermind".into()),
            database: std::env::var("GLOBALMIND_SURREAL_DB")
                .unwrap_or_else(|_| "globalmind_dev".into()),
            username: std::env::var("GLOBALMIND_SURREAL_USER").ok(),
            password: std::env::var("GLOBALMIND_SURREAL_PASS").ok(),
            allow_quarantined_core: std::env::var("GLOBALMIND_ALLOW_QUARANTINED_CORE")
                .is_ok_and(|value| value == "1"),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.namespace.is_empty() || self.database.is_empty() {
            return Err("SurrealDB namespace and database must be non-empty".into());
        }
        if self.database == "core" && !self.allow_quarantined_core {
            return Err(
                "hypermind/core is quarantined; use an isolated database or set the explicit override"
                    .into(),
            );
        }
        match (&self.username, &self.password) {
            (Some(_), Some(_)) | (None, None) => Ok(()),
            _ => Err("SurrealDB username and password must be supplied together".into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlobalMindStore {
    config: StoreConfig,
    host: String,
    port: u16,
}

impl GlobalMindStore {
    pub fn connect(config: StoreConfig) -> Result<Self, String> {
        config.validate()?;
        let (host, port) = parse_host_port(&config.endpoint)?;
        Ok(Self { config, host, port })
    }

    /// The MCP read surface needs the atom-address and exact tensor K/V
    /// contracts. Distillation ingress has additional tables and is checked
    /// separately by `verify_contract` before any distillation write.
    pub fn verify_recall_kv_contract(&self) -> Result<(), String> {
        for (table, required) in [
            (
                "atom",
                &["content_digest", "content_kind", "provenance"][..],
            ),
            (
                "atom_address",
                &["atom", "h3_lorentz", "t2_phase", "revision"][..],
            ),
            (
                "transformer_kv_session",
                &[
                    "model_digest",
                    "prefix_digest",
                    "tensor_contract_digest",
                    "tppe_configuration_digest",
                ][..],
            ),
            (
                "transformer_kv_page",
                &[
                    "key_tensor",
                    "value_tensor",
                    "key_digest",
                    "value_digest",
                    "phase_state_digest",
                ][..],
            ),
        ] {
            let result = self.one(&format!("INFO FOR TABLE {table};"))?;
            let fields = result
                .get("fields")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("GlobalMind table {table} is missing"))?;
            for field in required {
                let definition = fields
                    .get(*field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("GlobalMind field {table}.{field} is missing"))?;
                if !definition.contains("READONLY") {
                    return Err(format!("GlobalMind field {table}.{field} is not immutable"));
                }
            }
        }
        Ok(())
    }

    pub fn verify_contract(&self) -> Result<(), String> {
        for (table, required) in [
            (
                "distillation_packet",
                &["teacher_neutral_input_digest", "cell_kind", "verdict"][..],
            ),
            (
                "distilled_trajectory",
                &[
                    "state",
                    "replay_receipt_digest",
                    "address_contract_digest",
                    "tppe_configuration_digest",
                    "tppe_radial_law_digest",
                    "tppe_boundary_receipt_digest",
                ][..],
            ),
            (
                "trajectory_step",
                &["ordinal", "from_atom", "to_atom", "replay_receipt_digest"][..],
            ),
            (
                "surprise_observation",
                &[
                    "prediction_error",
                    "precision",
                    "precision_weighted_error",
                    "surprisal",
                ][..],
            ),
            (
                "superposition_hold",
                &["trigger", "branch_limit", "iteration_limit"][..],
            ),
            (
                "superposition_iteration",
                &["surviving_branches", "external_emissions"][..],
            ),
            (
                "superposition_resolution",
                &["outcome", "resolution_receipt_digest"][..],
            ),
            (
                "transformer_kv_session",
                &[
                    "model_digest",
                    "prefix_digest",
                    "tensor_contract_digest",
                    "tppe_configuration_digest",
                ][..],
            ),
            (
                "transformer_kv_page",
                &[
                    "key_tensor",
                    "value_tensor",
                    "key_digest",
                    "value_digest",
                    "phase_state_digest",
                ][..],
            ),
            (
                "model_artifact_receipt",
                &["model_source", "absolute_path", "sha256", "byte_count"][..],
            ),
            (
                "distillation_run_event",
                &[
                    "run",
                    "ordinal",
                    "phase",
                    "input_digest",
                    "tppe_version",
                    "tppe_configuration_digest",
                    "receipt_digest",
                ][..],
            ),
            (
                "edge_crystallization",
                &[
                    "edge",
                    "packet",
                    "admission",
                    "transition_digest",
                    "replay_receipt_digest",
                ][..],
            ),
            (
                "trajectory_step",
                &[
                    "packet",
                    "admission",
                    "edge_crystallization",
                    "tppe_radius_before",
                    "tppe_radius_after",
                    "tppe_revolutions_before",
                    "tppe_revolutions_after",
                    "tppe_boundary_margin_before",
                    "tppe_boundary_margin_after",
                ][..],
            ),
            (
                "local_jacobian_chart",
                &[
                    "run",
                    "admission",
                    "tensor_digest",
                    "numerical_receipt_digest",
                ][..],
            ),
        ] {
            let result = self.one(&format!("INFO FOR TABLE {table};"))?;
            let fields = result
                .get("fields")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("GlobalMind table {table} is missing"))?;
            for field in required {
                let definition = fields
                    .get(*field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("GlobalMind field {table}.{field} is missing"))?;
                if !definition.contains("READONLY") {
                    return Err(format!("GlobalMind field {table}.{field} is not immutable"));
                }
            }
        }
        Ok(())
    }

    /// Opens an immutable distillation ingress after the caller has rehashed
    /// every artifact. This transaction writes provenance only; it does not
    /// claim that candidate extraction or distillation has occurred.
    pub fn open_distillation_ingress(&self, ingress: &VerifiedManifest) -> Result<(), String> {
        self.verify_contract()?;
        let workspace = record("workspace", &ingress.workspace);
        let workspace_rows = self.one(&format!("SELECT id FROM {workspace} LIMIT 1;"))?;
        if workspace_rows.as_array().is_none_or(Vec::is_empty) {
            return Err("manifest workspace does not exist in the isolated database".into());
        }

        let mut query = String::from("BEGIN TRANSACTION; ");
        let mut source_records = Vec::with_capacity(ingress.models.len());
        for model in &ingress.models {
            let eligibility_receipt = model
                .eligibility_receipt_digest
                .as_ref()
                .ok_or_else(|| format!("model {} has no eligibility receipt", model.name))?;
            let judge_receipt = model
                .judge_receipt_digest
                .as_ref()
                .ok_or_else(|| format!("model {} has no independent judge receipt", model.name))?;
            let source = record("model_source", &model.source_id);
            source_records.push(source.clone());
            query.push_str(&format!(
                "CREATE {source} CONTENT {{ workspace: {workspace}, provider: {provider}, model: {model_name}, revision: {revision}, modality: {modality}, artifact_digest: {artifact_digest}, admitted_at: time::now() }} RETURN NONE; ",
                provider = sq(&model.provider),
                model_name = sq(&model.name),
                revision = sq(&model.revision),
                modality = sq(&model.modality),
                artifact_digest = sq(model.roster_digest.as_str()),
            ));
            for (ordinal, artifact) in model.artifacts.iter().enumerate() {
                let artifact_id = RecordId::parse(format!(
                    "artifact_{}_{}_{}",
                    &model.roster_digest.as_str()[..24],
                    &artifact.sha256.as_str()[..24],
                    ordinal
                ))?;
                query.push_str(&format!(
                    "CREATE {id} CONTENT {{ workspace: {workspace}, model_source: {source}, role: {role}, absolute_path: {path}, sha256: {sha256}, byte_count: {bytes}, verified_at: time::now() }} RETURN NONE; ",
                    id = record("model_artifact_receipt", &artifact_id),
                    role = sq(&artifact.role),
                    path = sq(&artifact.path),
                    sha256 = sq(artifact.sha256.as_str()),
                    bytes = artifact.bytes,
                ));
            }
            query.push_str(&format!(
                "CREATE {admission} CONTENT {{ workspace: {workspace}, model_source: {source}, source_artifact_digest: {artifact_digest}, source_artifact_uri: {artifact_uri}, roster_receipt_digest: {roster_receipt}, judge_receipt_digest: {judge_receipt}, eligibility_receipt_digest: {eligibility_receipt}, admitted_at: time::now() }} RETURN NONE; ",
                admission = record("distillation_admission", &model.admission_id),
                artifact_digest = sq(model.roster_digest.as_str()),
                artifact_uri = sq(&format!("roster:sha256:{}", model.roster_digest.as_str())),
                roster_receipt = sq(model.roster_digest.as_str()),
                judge_receipt = sq(judge_receipt.as_str()),
                eligibility_receipt = sq(eligibility_receipt.as_str()),
            ));
        }
        query.push_str(&format!(
            "CREATE {run} CONTENT {{ workspace: {workspace}, sources: [{sources}], method: {method}, configuration_digest: {configuration_digest}, input_digest: {input_digest}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, output_digest: NONE, receipt_digest: {receipt_digest}, status: 'opened', started_at: time::now(), completed_at: NONE }} RETURN NONE; \
             CREATE {event} CONTENT {{ workspace: {workspace}, run: {run}, ordinal: 0, phase: 'opened', input_digest: {input_digest}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, output_digest: NONE, receipt_digest: {receipt_digest}, recorded_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            run = record("distillation_run", &ingress.run_id),
            sources = source_records.join(", "),
            method = sq(&ingress.method),
            configuration_digest = sq(ingress.configuration_digest.as_str()),
            input_digest = sq(ingress.input_digest.as_str()),
            tppe_version = sq(&ingress.tppe_version),
            tppe_configuration_digest = sq(ingress.tppe_configuration_digest.as_str()),
            receipt_digest = sq(ingress.run_receipt_digest.as_str()),
            event = record("distillation_run_event", &ingress.opened_event_id),
        ));
        self.transaction(&query)
    }

    /// Opens a follow-on run over an already-ingested roster. Source and
    /// admission records are roster-scoped and immutable, so a second run
    /// (for example a holding bridge) must reuse them; only the run and its
    /// opened event are created. Fails if the roster was never ingested.
    pub fn open_distillation_run(&self, ingress: &VerifiedManifest) -> Result<(), String> {
        self.verify_contract()?;
        for model in &ingress.models {
            self.require_record_exists("model_source", &model.source_id)?;
            self.require_record_exists("distillation_admission", &model.admission_id)?;
        }
        let workspace = record("workspace", &ingress.workspace);
        let source_records = ingress
            .models
            .iter()
            .map(|model| record("model_source", &model.source_id))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {run} CONTENT {{ workspace: {workspace}, sources: [{sources}], method: {method}, configuration_digest: {configuration_digest}, input_digest: {input_digest}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, output_digest: NONE, receipt_digest: {receipt_digest}, status: 'opened', started_at: time::now(), completed_at: NONE }} RETURN NONE; \
             CREATE {event} CONTENT {{ workspace: {workspace}, run: {run}, ordinal: 0, phase: 'opened', input_digest: {input_digest}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, output_digest: NONE, receipt_digest: {receipt_digest}, recorded_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            run = record("distillation_run", &ingress.run_id),
            sources = source_records,
            method = sq(&ingress.method),
            configuration_digest = sq(ingress.configuration_digest.as_str()),
            input_digest = sq(ingress.input_digest.as_str()),
            tppe_version = sq(&ingress.tppe_version),
            tppe_configuration_digest = sq(ingress.tppe_configuration_digest.as_str()),
            receipt_digest = sq(ingress.run_receipt_digest.as_str()),
            event = record("distillation_run_event", &ingress.opened_event_id),
        );
        self.transaction(&query)
    }

    pub fn admit_packet(&self, packet: &DistillationPacket) -> Result<(), String> {
        require_nonempty("projector_version", &packet.projector_version)?;
        self.require_packet_context(packet)?;
        self.require_run_phase(&packet.run, DistillationPhase::Judging)?;
        let query = format!(
            "BEGIN TRANSACTION; {body} COMMIT TRANSACTION;",
            body = Self::packet_sql(packet)
        );
        self.transaction(&query)
    }

    fn packet_sql(packet: &DistillationPacket) -> String {
        format!(
            "CREATE {id} CONTENT {{ workspace: {workspace}, run: {run}, admission: {admission}, source_kind: {source_kind}, cell_kind: {cell_kind}, source_fragment_digest: {source_fragment}, teacher_neutral_input_digest: {neutral_input}, projector_version: {projector_version}, projector_digest: {projector_digest}, configuration_digest: {configuration_digest}, candidate_digest: {candidate_digest}, novelty_receipt_digest: {novelty_receipt}, judge_receipt_digest: {judge_receipt}, verdict: {verdict}, created_at: time::now() }} RETURN NONE; ",
            id = record("distillation_packet", &packet.id),
            workspace = record("workspace", &packet.workspace),
            run = record("distillation_run", &packet.run),
            admission = record("distillation_admission", &packet.admission),
            source_kind = sq(packet.source_kind.as_str()),
            cell_kind = sq(packet.cell_kind.as_str()),
            source_fragment = sq(packet.source_fragment_digest.as_str()),
            neutral_input = sq(packet.teacher_neutral_input_digest.as_str()),
            projector_version = sq(&packet.projector_version),
            projector_digest = sq(packet.projector_digest.as_str()),
            configuration_digest = sq(packet.configuration_digest.as_str()),
            candidate_digest = sq(packet.candidate_digest.as_str()),
            novelty_receipt = sq(packet.novelty_receipt_digest.as_str()),
            judge_receipt = sq(packet.judge_receipt_digest.as_str()),
            verdict = sq(packet.verdict.as_str()),
        )
    }

    pub fn crystallize_atom(&self, atom: &CrystallizedAtom) -> Result<(), String> {
        atom.validate()?;
        self.require_admitted_packet_binding(
            &atom.packet,
            "rank0_atom",
            &atom.workspace,
            &atom.run,
            &atom.admission,
        )?;
        self.require_run_phase(&atom.run, DistillationPhase::Replaying)?;
        let query = format!(
            "BEGIN TRANSACTION; {body} COMMIT TRANSACTION;",
            body = Self::atom_sql(atom)
        );
        self.transaction(&query)
    }

    fn atom_sql(atom: &CrystallizedAtom) -> String {
        let content_text = optional_string(atom.content_text.as_deref());
        let content_uri = optional_string(atom.content_uri.as_deref());
        let h3 = atom
            .h3_lorentz
            .iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let t2 = atom
            .t2_phase
            .iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "CREATE {atom_id} CONTENT {{ workspace: {workspace}, external_id: {external_id}, content_kind: {content_kind}, content_text: {content_text}, content_uri: {content_uri}, content_digest: {content_digest}, domain: {domain}, rank: 0, salience: {salience:?}, provenance: {provenance}, taint: {taint}, pinned: {pinned}, current_address: {address_id}, created_at: time::now(), updated_at: time::now(), distillation_run: {run} }} RETURN NONE; \
             CREATE {address_id} CONTENT {{ workspace: {workspace}, atom: {atom_id}, h3_lorentz: [{h3}], t2_phase: [{t2}], geometry_model: {geometry_model}, geometry_version: {geometry_version}, revision: {revision}, active: true, created_at: time::now(), space: {geometry_space} }} RETURN NONE; \
             CREATE {origin_id} CONTENT {{ workspace: {workspace}, atom: {atom_id}, admission: {admission}, source_step_digest: {source_step}, mint_receipt_digest: {mint_receipt}, recorded_at: time::now() }} RETURN NONE; \
             CREATE {crystallization_id} CONTENT {{ workspace: {workspace}, atom: {atom_id}, packet: {packet}, payload_digest: {payload_digest}, reconstruction_contract_digest: {reconstruction_contract}, reconstruction_receipt_digest: {reconstruction_receipt}, crystallization_receipt_digest: {crystallization_receipt}, created_at: time::now() }} RETURN NONE; \
             CREATE {membership_id} CONTENT {{ workspace: {workspace}, address: {address_id}, well: {well}, classifier_version: {classifier_version}, classifier_digest: {classifier_digest}, boundary_receipt_digest: {boundary_receipt}, classified_at: time::now() }} RETURN NONE; ",
            atom_id = record("atom", &atom.id),
            address_id = record("atom_address", &atom.address_id),
            origin_id = record("atom_origin", &atom.origin_id),
            crystallization_id = record("atom_crystallization", &atom.crystallization_id),
            membership_id = record("address_well_membership", &atom.membership_id),
            workspace = record("workspace", &atom.workspace),
            run = record("distillation_run", &atom.run),
            packet = record("distillation_packet", &atom.packet),
            admission = record("distillation_admission", &atom.admission),
            geometry_space = record("geometry_space", &atom.geometry_space),
            well = record("cognitive_well", &atom.well),
            external_id = sq(&atom.external_id),
            content_kind = sq(atom.content_kind.as_str()),
            content_text = content_text,
            content_uri = content_uri,
            content_digest = sq(atom.content_digest.as_str()),
            domain = sq(&atom.domain),
            salience = atom.salience,
            provenance = sq(&atom.provenance),
            taint = sq(&atom.taint),
            pinned = atom.pinned,
            h3 = h3,
            t2 = t2,
            geometry_model = sq(&atom.geometry_model),
            geometry_version = sq(&atom.geometry_version),
            revision = atom.address_revision,
            source_step = sq(atom.source_step_digest.as_str()),
            mint_receipt = sq(atom.mint_receipt_digest.as_str()),
            payload_digest = sq(atom.payload_digest.as_str()),
            reconstruction_contract = sq(atom.reconstruction_contract_digest.as_str()),
            reconstruction_receipt = sq(atom.reconstruction_receipt_digest.as_str()),
            crystallization_receipt = sq(atom.crystallization_receipt_digest.as_str()),
            classifier_version = sq(&atom.classifier_version),
            classifier_digest = sq(atom.classifier_digest.as_str()),
            boundary_receipt = sq(atom.boundary_receipt_digest.as_str()),
        )
    }

    pub fn admit_trajectory(&self, trajectory: &Trajectory) -> Result<(), String> {
        trajectory.validate()?;
        require_nonempty("tppe_version", &trajectory.tppe_version)?;
        self.require_run_phase(&trajectory.run, DistillationPhase::Replaying)?;
        self.require_atoms_exist(&trajectory.atoms)?;
        for step in &trajectory.steps {
            self.require_admitted_packet_binding(
                &step.packet,
                "rank1_transition",
                &trajectory.workspace,
                &trajectory.run,
                &step.admission,
            )?;
            self.require_atoms_exist(&step.evidence_atoms)?;
        }
        for loop_record in &trajectory.loops {
            self.require_admitted_packet_binding(
                &loop_record.packet,
                "rank2_loop",
                &trajectory.workspace,
                &trajectory.run,
                &loop_record.admission,
            )?;
        }
        for origin in &trajectory.origins {
            self.require_record_exists("distillation_admission", &origin.admission)?;
        }
        let query = format!(
            "BEGIN TRANSACTION; {body} COMMIT TRANSACTION;",
            body = Self::trajectory_sql(trajectory)?
        );
        self.transaction(&query)
    }

    fn trajectory_sql(trajectory: &Trajectory) -> Result<String, String> {
        let atom_records = trajectory
            .atoms
            .iter()
            .map(|id| record("atom", id))
            .collect::<Vec<_>>()
            .join(", ");
        let crystal = trajectory
            .crystallization_receipt_digest
            .as_ref()
            .ok_or_else(|| "crystallized trajectory receipt is missing".to_string())?;
        let mut query = format!(
            "
             CREATE {id} CONTENT {{ workspace: {workspace}, atoms: [{atoms}], trajectory_digest: {trajectory_digest}, judge_receipt_digest: {judge_receipt}, minted_at: time::now(), run: {run}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, tppe_radial_law_digest: {tppe_radial_law_digest}, tppe_boundary_receipt_digest: {tppe_boundary_receipt_digest}, address_contract_digest: {address_contract}, step_count: {step_count}, replay_receipt_digest: {replay_receipt}, crystallization_receipt_digest: {crystal}, state: {state} }} RETURN NONE; ",
            id = record("distilled_trajectory", &trajectory.id),
            workspace = record("workspace", &trajectory.workspace),
            atoms = atom_records,
            trajectory_digest = sq(trajectory.trajectory_digest.as_str()),
            judge_receipt = sq(trajectory.judge_receipt_digest.as_str()),
            run = record("distillation_run", &trajectory.run),
            tppe_version = sq(&trajectory.tppe_version),
            tppe_configuration_digest = sq(trajectory.tppe_configuration_digest.as_str()),
            tppe_radial_law_digest = sq(trajectory.tppe_radial_law_digest.as_str()),
            tppe_boundary_receipt_digest = sq(trajectory.tppe_boundary_receipt_digest.as_str(),),
            address_contract = sq(trajectory.address_contract_digest.as_str()),
            step_count = trajectory.steps.len(),
            replay_receipt = sq(trajectory.replay_receipt_digest.as_str()),
            crystal = sq(crystal.as_str()),
            state = sq(trajectory.state.as_str()),
        );
        for step in &trajectory.steps {
            let evidence_atoms = step
                .evidence_atoms
                .iter()
                .map(|atom| record("atom", atom))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(
                "RELATE {from_atom}->{edge}->{to_atom} CONTENT {{ workspace: {workspace}, kind: {kind}, weight: {weight:?}, confidence: {confidence:?}, evidence_atoms: [{evidence_atoms}], evidence_digest: {evidence_digest}, producer: {producer}, revision: {revision}, active: true, valid_from: time::now(), valid_to: NONE, created_at: time::now() }} RETURN NONE; \
                 CREATE {edge_crystallization} CONTENT {{ workspace: {workspace}, edge: {edge}, packet: {packet}, admission: {admission}, transition_digest: {transition}, replay_receipt_digest: {replay}, crystallization_receipt_digest: {crystallization_receipt}, created_at: time::now() }} RETURN NONE; ",
                from_atom = record("atom", &step.from_atom),
                edge = record("atom_edge", &step.edge),
                to_atom = record("atom", &step.to_atom),
                workspace = record("workspace", &trajectory.workspace),
                kind = sq(step.transition_kind.as_str()),
                weight = step.weight,
                confidence = step.confidence,
                evidence_atoms = evidence_atoms,
                evidence_digest = sq(step.evidence_digest.as_str()),
                producer = sq(&step.producer),
                revision = step.revision,
                edge_crystallization = record(
                    "edge_crystallization",
                    &step.edge_crystallization_id,
                ),
                packet = record("distillation_packet", &step.packet),
                admission = record("distillation_admission", &step.admission),
                transition = sq(step.transition_digest.as_str()),
                replay = sq(step.replay_receipt_digest.as_str()),
                crystallization_receipt = sq(step.crystallization_receipt_digest.as_str()),
            ));
            query.push_str(&format!(
                "CREATE {id} CONTENT {{ workspace: {workspace}, trajectory: {trajectory}, ordinal: {ordinal}, from_atom: {from_atom}, to_atom: {to_atom}, edge: {edge}, packet: {packet}, admission: {admission}, edge_crystallization: {edge_crystallization}, transition_kind: {kind}, t2_phase_before: [{before0:?}, {before1:?}], t2_phase_after: [{after0:?}, {after1:?}], tppe_radius_before: {radius_before:?}, tppe_radius_after: {radius_after:?}, tppe_revolutions_before: {revolutions_before}, tppe_revolutions_after: {revolutions_after}, tppe_boundary_margin_before: {margin_before:?}, tppe_boundary_margin_after: {margin_after:?}, action_digest: {action}, observation_digest: {observation}, transition_digest: {transition}, replay_receipt_digest: {replay}, created_at: time::now() }} RETURN NONE; ",
                id = record("trajectory_step", &step.id),
                workspace = record("workspace", &trajectory.workspace),
                trajectory = record("distilled_trajectory", &trajectory.id),
                ordinal = step.ordinal,
                from_atom = record("atom", &step.from_atom),
                to_atom = record("atom", &step.to_atom),
                edge = record("atom_edge", &step.edge),
                packet = record("distillation_packet", &step.packet),
                admission = record("distillation_admission", &step.admission),
                edge_crystallization = record(
                    "edge_crystallization",
                    &step.edge_crystallization_id,
                ),
                kind = sq(step.transition_kind.as_str()),
                before0 = step.t2_phase_before[0],
                before1 = step.t2_phase_before[1],
                after0 = step.t2_phase_after[0],
                after1 = step.t2_phase_after[1],
                radius_before = step.tppe_radius_before,
                radius_after = step.tppe_radius_after,
                revolutions_before = step.tppe_revolutions_before,
                revolutions_after = step.tppe_revolutions_after,
                margin_before = step.tppe_boundary_margin_before,
                margin_after = step.tppe_boundary_margin_after,
                action = sq(step.action_digest.as_str()),
                observation = sq(step.observation_digest.as_str()),
                transition = sq(step.transition_digest.as_str()),
                replay = sq(step.replay_receipt_digest.as_str()),
            ));
        }
        for loop_record in &trajectory.loops {
            query.push_str(&format!(
                "CREATE {id} CONTENT {{ workspace: {workspace}, trajectory: {trajectory}, packet: {packet}, admission: {admission}, start_atom: {start_atom}, end_atom: {end_atom}, first_step: {first_step}, last_step: {last_step}, loop_invariant_kind: {kind}, loop_invariant_digest: {invariant}, closure_receipt_digest: {closure}, created_at: time::now() }} RETURN NONE; ",
                id = record("trajectory_loop", &loop_record.id),
                workspace = record("workspace", &trajectory.workspace),
                trajectory = record("distilled_trajectory", &trajectory.id),
                packet = record("distillation_packet", &loop_record.packet),
                admission = record("distillation_admission", &loop_record.admission),
                start_atom = record("atom", &trajectory.atoms[loop_record.first_step as usize]),
                end_atom = record("atom", &trajectory.atoms[loop_record.last_step as usize + 1]),
                first_step = loop_record.first_step,
                last_step = loop_record.last_step,
                kind = sq(&loop_record.loop_invariant_kind),
                invariant = sq(loop_record.loop_invariant_digest.as_str()),
                closure = sq(loop_record.closure_receipt_digest.as_str()),
            ));
        }
        for origin in &trajectory.origins {
            query.push_str(&format!(
                "CREATE {id} CONTENT {{ workspace: {workspace}, trajectory: {trajectory}, admission: {admission}, source_walk_digest: {source_walk}, receipt_digest: {receipt}, recorded_at: time::now() }} RETURN NONE; ",
                id = record("trajectory_origin", &origin.id),
                workspace = record("workspace", &trajectory.workspace),
                trajectory = record("distilled_trajectory", &trajectory.id),
                admission = record("distillation_admission", &origin.admission),
                source_walk = sq(origin.source_walk_digest.as_str()),
                receipt = sq(origin.receipt_digest.as_str()),
            ));
        }
        Ok(query)
    }

    pub fn admit_local_jacobian_chart(&self, chart: &LocalJacobianChart) -> Result<(), String> {
        chart.validate()?;
        self.require_run_phase(&chart.run, DistillationPhase::Replaying)?;
        self.require_record_exists("atom", &chart.anchor)?;
        self.require_admitted_packet_binding(
            &chart.packet,
            "rank8_jacobian",
            &chart.workspace,
            &chart.run,
            &chart.admission,
        )?;
        let query = format!(
            "BEGIN TRANSACTION; {body} COMMIT TRANSACTION;",
            body = Self::chart_sql(chart)
        );
        self.transaction(&query)
    }

    fn chart_sql(chart: &LocalJacobianChart) -> String {
        let shape = chart
            .tensor_shape
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "CREATE {id} CONTENT {{ workspace: {workspace}, run: {run}, anchor: {anchor}, packet: {packet}, admission: {admission}, chart_rank: {chart_rank}, dtype: {dtype}, tensor_shape: [{shape}], tangent_basis: {tangent_basis}, tensor_digest: {tensor_digest}, chart_digest: {chart_digest}, marginal_gain_receipt_digest: {marginal_gain_receipt}, numerical_receipt_digest: {numerical_receipt}, created_at: time::now() }} RETURN NONE; ",
            id = record("local_jacobian_chart", &chart.id),
            workspace = record("workspace", &chart.workspace),
            run = record("distillation_run", &chart.run),
            anchor = record("atom", &chart.anchor),
            packet = record("distillation_packet", &chart.packet),
            admission = record("distillation_admission", &chart.admission),
            chart_rank = chart.chart_rank,
            dtype = sq(chart.dtype.as_str()),
            shape = shape,
            tangent_basis = bytes_literal(&chart.tangent_basis),
            tensor_digest = sq(chart.tensor_digest.as_str()),
            chart_digest = sq(chart.chart_digest.as_str()),
            marginal_gain_receipt = sq(chart.marginal_gain_receipt_digest.as_str()),
            numerical_receipt = sq(chart.numerical_receipt_digest.as_str()),
        )
    }

    pub fn append_distillation_run_event(
        &self,
        event: &DistillationRunEvent,
    ) -> Result<(), String> {
        if event.phase == DistillationPhase::Opened || event.ordinal == 0 {
            return Err("opened is created only by the ingress transaction".into());
        }
        if event.tppe_version.trim().is_empty() {
            return Err("distillation event TPPE version must be non-empty".into());
        }
        if event.phase == DistillationPhase::Completed && event.output_digest.is_none() {
            return Err("completed distillation event requires an output digest".into());
        }
        let rows = self.one(&format!(
            "SELECT workspace, ordinal, phase, input_digest, tppe_version, tppe_configuration_digest FROM distillation_run_event WHERE run = {} ORDER BY ordinal DESC LIMIT 1;",
            record("distillation_run", &event.run)
        ))?;
        let prior = rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "distillation run has no opened event".to_string())?;
        let prior_ordinal = prior
            .get("ordinal")
            .and_then(Value::as_u64)
            .ok_or_else(|| "prior distillation event ordinal is missing".to_string())?;
        let expected_ordinal = prior_ordinal
            .checked_add(1)
            .ok_or_else(|| "distillation event ordinal overflow".to_string())?;
        if u64::from(event.ordinal) != expected_ordinal {
            return Err("distillation event ordinal does not extend the prior event".into());
        }
        let prior_phase = DistillationPhase::parse(
            prior
                .get("phase")
                .and_then(Value::as_str)
                .ok_or_else(|| "prior distillation event phase is missing".to_string())?,
        )?;
        if !event.phase.can_follow(prior_phase) {
            return Err("illegal distillation phase transition".into());
        }
        if prior.get("workspace").and_then(Value::as_str)
            != Some(record_ref("workspace", &event.workspace).as_str())
            || prior.get("input_digest").and_then(Value::as_str)
                != Some(event.input_digest.as_str())
            || prior.get("tppe_version").and_then(Value::as_str)
                != Some(event.tppe_version.as_str())
            || prior
                .get("tppe_configuration_digest")
                .and_then(Value::as_str)
                != Some(event.tppe_configuration_digest.as_str())
        {
            return Err("distillation event identity drift".into());
        }
        let query = format!(
            "BEGIN TRANSACTION; {body} COMMIT TRANSACTION;",
            body = Self::run_event_sql(event)
        );
        self.transaction(&query)
    }

    fn run_event_sql(event: &DistillationRunEvent) -> String {
        let output_digest = event
            .output_digest
            .as_ref()
            .map_or_else(|| "NONE".into(), |digest| sq(digest.as_str()));
        format!(
            "CREATE {id} CONTENT {{ workspace: {workspace}, run: {run}, ordinal: {ordinal}, phase: {phase}, input_digest: {input_digest}, output_digest: {output_digest}, tppe_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration_digest}, receipt_digest: {receipt_digest}, recorded_at: time::now() }} RETURN NONE; ",
            id = record("distillation_run_event", &event.id),
            workspace = record("workspace", &event.workspace),
            run = record("distillation_run", &event.run),
            ordinal = event.ordinal,
            phase = sq(event.phase.as_str()),
            input_digest = sq(event.input_digest.as_str()),
            output_digest = output_digest,
            tppe_version = sq(&event.tppe_version),
            tppe_configuration_digest = sq(event.tppe_configuration_digest.as_str()),
            receipt_digest = sq(event.receipt_digest.as_str()),
        )
    }

    /// One atomic crystallization: phase events, packets, atoms, the
    /// trajectory (header, edges, steps, loops, origins), and charts in
    /// a single Surreal transaction. A crash or any statement failure
    /// persists nothing — there is no partial run to resume. Callers
    /// pre-validate identities and run state; this method validates
    /// every entity in Rust before the transaction is sent.
    pub fn admit_crystallization(
        &self,
        events: &[DistillationRunEvent],
        packets: &[DistillationPacket],
        atoms: &[CrystallizedAtom],
        trajectory: &Trajectory,
        charts: &[LocalJacobianChart],
    ) -> Result<(), String> {
        trajectory.validate()?;
        require_nonempty("tppe_version", &trajectory.tppe_version)?;
        for event in events {
            if event.phase == DistillationPhase::Opened || event.ordinal == 0 {
                return Err("crystallization events must extend the opened event".into());
            }
            if event.tppe_version.trim().is_empty() {
                return Err("crystallization event TPPE version must be non-empty".into());
            }
        }
        for packet in packets {
            require_nonempty("projector_version", &packet.projector_version)?;
        }
        for atom in atoms {
            atom.validate()?;
        }
        for chart in charts {
            chart.validate()?;
        }
        let mut query = String::from("BEGIN TRANSACTION; ");
        for event in events {
            query.push_str(&Self::run_event_sql(event));
        }
        for packet in packets {
            query.push_str(&Self::packet_sql(packet));
        }
        for atom in atoms {
            query.push_str(&Self::atom_sql(atom));
        }
        query.push_str(&Self::trajectory_sql(trajectory)?);
        for chart in charts {
            query.push_str(&Self::chart_sql(chart));
        }
        query.push_str("COMMIT TRANSACTION;");
        self.transaction(&query)
    }

    /// Persists the active-inference decision a hold is opened under. The
    /// decision is the audit anchor for trigger observations; it grants no
    /// execution permission by itself.
    #[allow(clippy::too_many_arguments)]
    pub fn create_active_inference_decision(
        &self,
        id: &RecordId,
        workspace: &RecordId,
        request_digest: &Digest,
        objective_digest: &Digest,
        selected_wells: &[RecordId],
        retrieval_depth: u32,
        forward_mode: ForwardMode,
        controller_version: &str,
        controller_digest: &Digest,
        quality_receipt_digest: &Digest,
        resource_receipt_digest: &Digest,
    ) -> Result<(), String> {
        require_nonempty("controller_version", controller_version)?;
        let wells = selected_wells
            .iter()
            .map(|well| record("cognitive_well", well))
            .collect::<Vec<_>>()
            .join(", ");
        let mode = match forward_mode {
            ForwardMode::RecallOnly => "recall_only",
            ForwardMode::AdaptiveForward => "adaptive_forward",
            ForwardMode::FullForward => "full_forward",
        };
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, request_digest: {request_digest}, objective_digest: {objective_digest}, selected_wells: [{wells}], selected_trajectories: [], retrieval_depth: {retrieval_depth}, forward_mode: {mode}, controller_version: {controller_version}, controller_digest: {controller_digest}, quality_receipt_digest: {quality_receipt}, resource_receipt_digest: {resource_receipt}, created_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("active_inference_decision", id),
            workspace = record("workspace", workspace),
            request_digest = sq(request_digest.as_str()),
            objective_digest = sq(objective_digest.as_str()),
            wells = wells,
            retrieval_depth = retrieval_depth,
            mode = sq(mode),
            controller_version = sq(controller_version),
            controller_digest = sq(controller_digest.as_str()),
            quality_receipt = sq(quality_receipt_digest.as_str()),
            resource_receipt = sq(resource_receipt_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub fn record_prediction_error(
        &self,
        observation: &PredictionErrorObservation,
    ) -> Result<(), String> {
        observation.validate()?;
        require_nonempty("estimator_version", &observation.estimator_version)?;
        let well = optional_record("cognitive_well", observation.well.as_ref());
        let trajectory = optional_record("distilled_trajectory", observation.trajectory.as_ref());
        let bayesian_surprise = observation
            .bayesian_surprise
            .map_or_else(|| "NONE".into(), |value| format!("{value:?}"));
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, decision: {decision}, hub: {hub}, well: {well}, trajectory: {trajectory}, prior_digest: {prior}, observation_digest: {observation}, estimator_version: {estimator_version}, estimator_digest: {estimator_digest}, prediction_error: {prediction_error:?}, precision: {precision:?}, precision_weighted_error: {weighted:?}, surprisal: {surprisal:?}, unit: {unit}, bayesian_surprise: {bayesian_surprise}, resolution: {resolution}, receipt_digest: {receipt}, observed_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("surprise_observation", &observation.id),
            workspace = record("workspace", &observation.workspace),
            decision = record("active_inference_decision", &observation.decision),
            hub = record("cognitive_hub", &observation.hub),
            well = well,
            trajectory = trajectory,
            prior = sq(observation.prior_digest.as_str()),
            observation = sq(observation.observation_digest.as_str()),
            estimator_version = sq(&observation.estimator_version),
            estimator_digest = sq(observation.estimator_digest.as_str()),
            prediction_error = observation.prediction_error,
            precision = observation.precision,
            weighted = observation.precision_weighted_error,
            surprisal = observation.surprisal,
            unit = sq(observation.unit.as_str()),
            bayesian_surprise = bayesian_surprise,
            resolution = sq(observation.resolution.as_str()),
            receipt = sq(observation.receipt_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub fn open_superposition(&self, hold: &SuperpositionHold) -> Result<(), String> {
        hold.validate()?;
        let trigger_observation =
            optional_record("surprise_observation", hold.trigger_observation.as_ref());
        let well = optional_record("cognitive_well", hold.well.as_ref());
        let mut query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, decision: {decision}, trigger_observation: {trigger_observation}, hub: {hub}, well: {well}, request_digest: {request_digest}, prefix_digest: {prefix_digest}, trigger: {trigger}, controller_version: {controller_version}, controller_digest: {controller_digest}, branch_limit: {branch_limit}, iteration_limit: {iteration_limit}, opened_at: time::now() }} RETURN NONE; ",
            id = record("superposition_hold", &hold.id),
            workspace = record("workspace", &hold.workspace),
            decision = record("active_inference_decision", &hold.decision),
            trigger_observation = trigger_observation,
            hub = record("cognitive_hub", &hold.hub),
            well = well,
            request_digest = sq(hold.request_digest.as_str()),
            prefix_digest = sq(hold.prefix_digest.as_str()),
            trigger = sq(hold.trigger.as_str()),
            controller_version = sq(&hold.controller_version),
            controller_digest = sq(hold.controller_digest.as_str()),
            branch_limit = hold.branches.len(),
            iteration_limit = hold.iteration_limit,
        );
        for branch in &hold.branches {
            query.push_str(&format!(
                "CREATE {id} CONTENT {{ workspace: {workspace}, hold: {hold}, ordinal: {ordinal}, source: {source}, source_trajectory: {source_trajectory}, from_atom: {from_atom}, candidate_atom: {candidate_atom}, prediction_digest: {prediction_digest}, scorer_version: {scorer_version}, scorer_digest: {scorer_digest}, free_energy_value: {free_energy:?}, precision_weighted_error: {weighted_error:?}, readiness: {readiness:?}, compliance_permitted: {compliance}, score_receipt_digest: {score_receipt}, compliance_receipt_digest: {compliance_receipt}, created_at: time::now() }} RETURN NONE; ",
                id = record("superposition_branch", &branch.id),
                workspace = record("workspace", &hold.workspace),
                hold = record("superposition_hold", &hold.id),
                ordinal = branch.ordinal,
                source = sq(branch.source.as_str()),
                source_trajectory = optional_record(
                    "distilled_trajectory",
                    branch.source_trajectory.as_ref(),
                ),
                from_atom = record("atom", &branch.from_atom),
                candidate_atom = record("atom", &branch.candidate_atom),
                prediction_digest = sq(branch.prediction_digest.as_str()),
                scorer_version = sq(&branch.scorer_version),
                scorer_digest = sq(branch.scorer_digest.as_str()),
                free_energy = branch.free_energy_value,
                weighted_error = branch.precision_weighted_error,
                readiness = branch.readiness,
                compliance = branch.compliance_permitted,
                score_receipt = sq(branch.score_receipt_digest.as_str()),
                compliance_receipt = sq(branch.compliance_receipt_digest.as_str()),
            ));
        }
        query.push_str("COMMIT TRANSACTION;");
        self.transaction(&query)
    }

    pub fn record_superposition_iteration(
        &self,
        iteration: &SuperpositionIteration,
    ) -> Result<(), String> {
        iteration.validate()?;
        let surviving = iteration
            .surviving_branches
            .iter()
            .map(|id| record("superposition_branch", id))
            .collect::<Vec<_>>()
            .join(", ");
        let loop_receipt = iteration
            .loop_receipt_digest
            .as_ref()
            .map_or_else(|| "NONE".into(), |digest| sq(digest.as_str()));
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, hold: {hold}, ordinal: {ordinal}, input_branch_digest: {input_digest}, surviving_branches: [{surviving}], rollout_receipt_digest: {rollout_receipt}, loop_receipt_digest: {loop_receipt}, suppression_receipt_digest: {suppression_receipt}, output_branch_digest: {output_digest}, external_emissions: 0, created_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("superposition_iteration", &iteration.id),
            workspace = record("workspace", &iteration.workspace),
            hold = record("superposition_hold", &iteration.hold),
            ordinal = iteration.ordinal,
            input_digest = sq(iteration.input_branch_digest.as_str()),
            surviving = surviving,
            rollout_receipt = sq(iteration.rollout_receipt_digest.as_str()),
            loop_receipt = loop_receipt,
            suppression_receipt = sq(iteration.suppression_receipt_digest.as_str()),
            output_digest = sq(iteration.output_branch_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub fn resolve_superposition(
        &self,
        resolution: &SuperpositionResolution,
    ) -> Result<(), String> {
        resolution.validate()?;
        let replay = resolution
            .replay_receipt_digest
            .as_ref()
            .map_or_else(|| "NONE".into(), |digest| sq(digest.as_str()));
        let crystallization = resolution
            .crystallization_receipt_digest
            .as_ref()
            .map_or_else(|| "NONE".into(), |digest| sq(digest.as_str()));
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, hold: {hold}, outcome: {outcome}, selected_branch: {selected_branch}, resulting_trajectory: {resulting_trajectory}, resulting_atom: {resulting_atom}, judge_receipt_digest: {judge_receipt}, replay_receipt_digest: {replay_receipt}, crystallization_receipt_digest: {crystallization_receipt}, resolution_receipt_digest: {resolution_receipt}, resolved_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("superposition_resolution", &resolution.id),
            workspace = record("workspace", &resolution.workspace),
            hold = record("superposition_hold", &resolution.hold),
            outcome = sq(resolution.outcome.as_str()),
            selected_branch =
                optional_record("superposition_branch", resolution.selected_branch.as_ref(),),
            resulting_trajectory = optional_record(
                "distilled_trajectory",
                resolution.resulting_trajectory.as_ref(),
            ),
            resulting_atom = optional_record("atom", resolution.resulting_atom.as_ref()),
            judge_receipt = sq(resolution.judge_receipt_digest.as_str()),
            replay_receipt = replay,
            crystallization_receipt = crystallization,
            resolution_receipt = sq(resolution.resolution_receipt_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub fn create_tensor_session(&self, session: &TensorSession) -> Result<(), String> {
        session.validate()?;
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, model_id: {model_id}, model_revision: {model_revision}, model_digest: {model_digest}, request_digest: {request_digest}, prefix_digest: {prefix_digest}, dtype: {dtype}, layer_count: {layer_count}, kv_head_count: {kv_head_count}, head_dim: {head_dim}, geometry_space: {geometry_space}, toroidal_phase_encoder_version: {tppe_version}, tppe_configuration_digest: {tppe_configuration}, address_contract_digest: {address_contract}, tensor_contract_digest: {tensor_contract}, active: true, created_at: time::now(), closed_at: NONE }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("transformer_kv_session", &session.id),
            workspace = record("workspace", &session.workspace),
            model_id = sq(&session.model_id),
            model_revision = sq(&session.model_revision),
            model_digest = sq(session.model_digest.as_str()),
            request_digest = sq(session.request_digest.as_str()),
            prefix_digest = sq(session.prefix_digest.as_str()),
            dtype = sq(session.dtype.as_str()),
            layer_count = session.layer_count,
            kv_head_count = session.kv_head_count,
            head_dim = session.head_dim,
            geometry_space = record("geometry_space", &session.geometry_space),
            tppe_version = sq(&session.tppe_version),
            tppe_configuration = sq(session.tppe_configuration_digest.as_str()),
            address_contract = sq(session.address_contract_digest.as_str()),
            tensor_contract = sq(session.tensor_contract_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub fn put_tensor_page(&self, page: &TensorPage) -> Result<(), String> {
        page.validate()?;
        self.require_compatible_tensor_session(page)?;
        let atoms = page
            .trajectory_atoms
            .iter()
            .map(|id| record("atom", id))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, session: {session}, layer: {layer}, kv_head: {kv_head}, token_start: {token_start}, token_end: {token_end}, tensor_shape: [{shape0}, {shape1}], dtype: {dtype}, layout: {layout}, key_tensor: {key_tensor}, value_tensor: {value_tensor}, key_digest: {key_digest}, value_digest: {value_digest}, key_bytes: {key_bytes}, value_bytes: {value_bytes}, address: {address}, trajectory_atoms: [{atoms}], well: {well}, trajectory: {trajectory}, phase_state_digest: {phase_state}, receipt_digest: {receipt}, revision: {revision}, hits: 0, active: true, created_at: time::now(), last_accessed_at: NONE }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("transformer_kv_page", &page.id),
            workspace = record("workspace", &page.workspace),
            session = record("transformer_kv_session", &page.session),
            layer = page.layer,
            kv_head = page.kv_head,
            token_start = page.token_start,
            token_end = page.token_end,
            shape0 = page.tensor_shape[0],
            shape1 = page.tensor_shape[1],
            dtype = sq(page.dtype.as_str()),
            layout = sq(page.layout.as_str()),
            key_tensor = bytes_literal(&page.key_tensor),
            value_tensor = bytes_literal(&page.value_tensor),
            key_digest = sq(page.key_digest.as_str()),
            value_digest = sq(page.value_digest.as_str()),
            key_bytes = page.key_tensor.len(),
            value_bytes = page.value_tensor.len(),
            address = record("atom_address", &page.address),
            atoms = atoms,
            well = optional_record("cognitive_well", page.well.as_ref()),
            trajectory = optional_record("distilled_trajectory", page.trajectory.as_ref()),
            phase_state = sq(page.phase_state_digest.as_str()),
            receipt = sq(page.receipt_digest.as_str()),
            revision = page.revision,
        );
        self.transaction(&query)
    }

    pub fn fetch_tensor_payload(&self, page: &RecordId) -> Result<TensorPayload, String> {
        let rows = self.one(&format!(
            "SELECT key_tensor, value_tensor, key_digest, value_digest FROM {} WHERE active = true LIMIT 1;",
            record("transformer_kv_page", page)
        ))?;
        let row = rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "active tensor page does not exist".to_string())?;
        let key = json_bytes(row.get("key_tensor"), "key_tensor")?;
        let value = json_bytes(row.get("value_tensor"), "value_tensor")?;
        let key_digest = row
            .get("key_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| "tensor page key digest is missing".to_string())?;
        let value_digest = row
            .get("value_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| "tensor page value digest is missing".to_string())?;
        if Digest::sha256(&key).as_str() != key_digest
            || Digest::sha256(&value).as_str() != value_digest
        {
            return Err("stored tensor page digest mismatch".into());
        }
        Ok(TensorPayload {
            key: key.into_boxed_slice(),
            value: value.into_boxed_slice(),
        })
    }

    /// Hydrate one ranked inline atom only when workspace and content digest
    /// match the retrieval witness. URI-only atoms need a separate source gate.
    pub fn fetch_atom_content_exact(
        &self,
        workspace: &RecordId,
        atom: &RecordId,
        expected_digest: &Digest,
    ) -> Result<Option<String>, String> {
        let rows = self.one(&format!(
            "SELECT content_text, content_digest FROM {} WHERE workspace = {} LIMIT 1;",
            record("atom", atom),
            record("workspace", workspace),
        ))?;
        let row = rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "ranked atom does not exist in expected workspace".to_string())?;
        let stored_digest = row
            .get("content_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| "ranked atom has no content digest".to_string())?;
        if stored_digest != expected_digest.as_str() {
            return Err("ranked atom content digest changed".into());
        }
        let Some(text) = row.get("content_text").and_then(Value::as_str) else {
            return Ok(None);
        };
        if text.is_empty() || text.len() > 16 * 1024 {
            return Err("ranked atom inline content is empty or over the 16 KiB bound".into());
        }
        if Digest::sha256(text.as_bytes()) != *expected_digest {
            return Err("ranked atom inline content fails SHA-256 verification".into());
        }
        Ok(Some(text.to_string()))
    }

    /// Persists the executable Whitehead 12-around-1 topology (`geometry_space`,
    /// `cognitive_hub`, 12 `cognitive_well` records, and 12 `hub_has_well`
    /// relation edges) in one atomic SurrealDB transaction.
    pub fn admit_whitehead_topology(&self, topology: &WhiteheadTopology) -> Result<(), String> {
        let workspace = record("workspace", &topology.workspace);
        let space = record("geometry_space", &topology.geometry_space_id);
        let hub = record("cognitive_hub", &topology.hub.id);
        let hub_h3 = topology
            .hub
            .h3_center_lorentz
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let hub_t2 = topology
            .hub
            .t2_phase
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(", ");

        let mut query = format!(
            "BEGIN TRANSACTION; \
             UPSERT {space} CONTENT {{ name: {space_name}, manifold: {manifold}, quotient_symbol: {quotient_symbol}, quotient_status: {status}, coordinate_model: {coord_model}, contract_version: {contract_version}, contract_digest: {contract_digest}, created_at: time::now() }} RETURN NONE; \
             CREATE {hub} CONTENT {{ workspace: {workspace}, geometry_space: {space}, label: {hub_label}, h3_center_lorentz: [{hub_h3}], t2_phase: [{hub_t2}], reference_contract_digest: {ref_digest}, status: {hub_status}, created_at: time::now() }} RETURN NONE; ",
            space_name = sq(WHITEHEAD_SPACE_NAME),
            manifold = sq(WHITEHEAD_MANIFOLD),
            quotient_symbol = sq(WHITEHEAD_QUOTIENT_SYMBOL),
            status = sq(topology.hub.status.as_str()),
            coord_model = sq(WHITEHEAD_COORDINATE_MODEL),
            contract_version = sq(&topology.contract_version),
            contract_digest = sq(topology.contract_digest.as_str()),
            hub_label = sq(&topology.hub.label),
            ref_digest = sq(topology.hub.reference_contract_digest.as_str()),
            hub_status = sq(topology.hub.status.as_str()),
        );

        for well in &topology.wells {
            let well_rec = record("cognitive_well", &well.id);
            let edge_id = RecordId::parse(format!("hub_well_{:02}", well.ordinal))?;
            let edge_rec = record("hub_has_well", &edge_id);
            let well_h3 = well
                .h3_center_lorentz
                .iter()
                .map(|v| format!("{v:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(
                "CREATE {well_rec} CONTENT {{ workspace: {workspace}, geometry_space: {space}, ordinal: {ordinal}, label: {label}, h3_center_lorentz: [{well_h3}], t2_sector: {t2_sector}, boundary_contract_digest: {boundary_digest}, status: {well_status}, created_at: time::now() }} RETURN NONE; \
                 RELATE {hub}->{edge_rec}->{well_rec} CONTENT {{ workspace: {workspace}, topology_contract_digest: {topo_digest}, created_at: time::now() }} RETURN NONE; ",
                ordinal = well.ordinal,
                label = sq(&well.label),
                t2_sector = well.t2_sector,
                boundary_digest = sq(well.boundary_contract_digest.as_str()),
                well_status = sq(well.status.as_str()),
                topo_digest = sq(topology.topology_contract_digest.as_str()),
            ));
        }

        query.push_str("COMMIT TRANSACTION;");
        self.transaction(&query)
    }

    /// Verifies that the stored `cognitive_hub`, 12 `cognitive_well` records,
    /// and 12 `hub_has_well` relation edges in SurrealDB match the executable
    /// `WhiteheadTopology` contract.
    pub fn verify_whitehead_topology(&self, topology: &WhiteheadTopology) -> Result<(), String> {
        let workspace_ref = record_ref("workspace", &topology.workspace);
        let space_ref = record_ref("geometry_space", &topology.geometry_space_id);
        let hub_rows = self.one(&format!(
            "SELECT workspace, geometry_space, label, reference_contract_digest, status FROM {} LIMIT 1;",
            record("cognitive_hub", &topology.hub.id)
        ))?;
        let hub = hub_rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "cognitive_hub record is missing".to_string())?;
        if hub.get("workspace").and_then(Value::as_str) != Some(workspace_ref.as_str())
            || hub.get("geometry_space").and_then(Value::as_str) != Some(space_ref.as_str())
            || hub.get("label").and_then(Value::as_str) != Some(topology.hub.label.as_str())
            || hub.get("reference_contract_digest").and_then(Value::as_str)
                != Some(topology.hub.reference_contract_digest.as_str())
            || hub.get("status").and_then(Value::as_str) != Some(topology.hub.status.as_str())
        {
            return Err("stored cognitive_hub does not match WhiteheadTopology contract".into());
        }

        let well_rows = self.one(&format!(
            "SELECT ordinal, label, t2_sector, boundary_contract_digest, status FROM cognitive_well WHERE workspace = {} AND geometry_space = {} ORDER BY ordinal ASC;",
            record("workspace", &topology.workspace),
            record("geometry_space", &topology.geometry_space_id),
        ))?;
        let wells = well_rows
            .as_array()
            .ok_or_else(|| "cognitive_well query did not return an array".to_string())?;
        if wells.len() != 12 {
            return Err(format!(
                "expected 12 cognitive_well records, found {}",
                wells.len()
            ));
        }
        for (expected, actual) in topology.wells.iter().zip(wells.iter()) {
            if actual.get("ordinal").and_then(Value::as_u64) != Some(u64::from(expected.ordinal))
                || actual.get("label").and_then(Value::as_str) != Some(expected.label.as_str())
                || actual.get("t2_sector").and_then(Value::as_u64)
                    != Some(u64::from(expected.t2_sector))
                || actual
                    .get("boundary_contract_digest")
                    .and_then(Value::as_str)
                    != Some(expected.boundary_contract_digest.as_str())
                || actual.get("status").and_then(Value::as_str) != Some(expected.status.as_str())
            {
                return Err(format!(
                    "stored cognitive_well ordinal {} does not match WhiteheadTopology contract",
                    expected.ordinal
                ));
            }
        }

        let edge_rows = self.one(&format!(
            "SELECT in, out, topology_contract_digest FROM hub_has_well WHERE workspace = {} AND in = {};",
            record("workspace", &topology.workspace),
            record("cognitive_hub", &topology.hub.id),
        ))?;
        let edges = edge_rows
            .as_array()
            .ok_or_else(|| "hub_has_well query did not return an array".to_string())?;
        if edges.len() != 12 {
            return Err(format!(
                "expected 12 hub_has_well edges from hub, found {}",
                edges.len()
            ));
        }
        for edge in edges {
            if edge.get("topology_contract_digest").and_then(Value::as_str)
                != Some(topology.topology_contract_digest.as_str())
            {
                return Err("hub_has_well topology_contract_digest mismatch".into());
            }
        }
        Ok(())
    }

    /// Persists an `address_well_membership` receipt produced by `WhiteheadTopology::classify_well`.
    pub fn record_address_well_membership(
        &self,
        membership_id: &RecordId,
        workspace: &RecordId,
        address_id: &RecordId,
        classification: &WellClassification,
    ) -> Result<(), String> {
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE {id} CONTENT {{ workspace: {workspace}, address: {address}, well: {well}, classifier_version: {version}, classifier_digest: {classifier_digest}, boundary_receipt_digest: {boundary_digest}, classified_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            id = record("address_well_membership", membership_id),
            workspace = record("workspace", workspace),
            address = record("atom_address", address_id),
            well = record("cognitive_well", &classification.primary_well_id),
            version = sq(&classification.classifier_version),
            classifier_digest = sq(classification.classifier_digest.as_str()),
            boundary_digest = sq(classification.boundary_receipt_digest.as_str()),
        );
        self.transaction(&query)
    }

    pub(crate) fn transaction(&self, query: &str) -> Result<(), String> {
        self.sql(query)?;
        Ok(())
    }

    fn require_packet_context(&self, packet: &DistillationPacket) -> Result<(), String> {
        let run_rows = self.one(&format!(
            "SELECT workspace, sources, configuration_digest FROM {} LIMIT 1;",
            record("distillation_run", &packet.run)
        ))?;
        let run = run_rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "distillation run does not exist".to_string())?;
        let admission_rows = self.one(&format!(
            "SELECT workspace, model_source FROM {} LIMIT 1;",
            record("distillation_admission", &packet.admission)
        ))?;
        let admission = admission_rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "distillation admission does not exist".to_string())?;
        let expected_workspace = record_ref("workspace", &packet.workspace);
        let model_source = admission
            .get("model_source")
            .and_then(Value::as_str)
            .ok_or_else(|| "distillation admission model source is missing".to_string())?;
        let source_is_in_run =
            run.get("sources")
                .and_then(Value::as_array)
                .is_some_and(|sources| {
                    sources
                        .iter()
                        .any(|source| source.as_str() == Some(model_source))
                });
        if run.get("workspace").and_then(Value::as_str) != Some(expected_workspace.as_str())
            || admission.get("workspace").and_then(Value::as_str)
                != Some(expected_workspace.as_str())
            || run.get("configuration_digest").and_then(Value::as_str)
                != Some(packet.configuration_digest.as_str())
            || !source_is_in_run
        {
            return Err("distillation packet context does not match its run/admission".into());
        }
        Ok(())
    }

    fn require_run_phase(&self, run: &RecordId, required: DistillationPhase) -> Result<(), String> {
        let rows = self.one(&format!(
            "SELECT phase FROM distillation_run_event WHERE run = {} ORDER BY ordinal DESC LIMIT 1;",
            record("distillation_run", run)
        ))?;
        let phase = rows
            .as_array()
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("phase"))
            .and_then(Value::as_str)
            .ok_or_else(|| "distillation run has no lifecycle phase".to_string())?;
        if DistillationPhase::parse(phase)? != required {
            return Err(format!(
                "distillation run must be in {} phase",
                required.as_str()
            ));
        }
        Ok(())
    }

    fn require_admitted_packet_binding(
        &self,
        packet: &RecordId,
        expected_kind: &str,
        workspace: &RecordId,
        run: &RecordId,
        admission: &RecordId,
    ) -> Result<(), String> {
        let rows = self.one(&format!(
            "SELECT cell_kind, verdict, workspace, run, admission FROM {} LIMIT 1;",
            record("distillation_packet", packet)
        ))?;
        let row = rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "distillation packet does not exist".to_string())?;
        if row.get("cell_kind").and_then(Value::as_str) != Some(expected_kind)
            || row.get("verdict").and_then(Value::as_str) != Some("admitted")
            || row.get("workspace").and_then(Value::as_str)
                != Some(record_ref("workspace", workspace).as_str())
            || row.get("run").and_then(Value::as_str)
                != Some(record_ref("distillation_run", run).as_str())
            || row.get("admission").and_then(Value::as_str)
                != Some(record_ref("distillation_admission", admission).as_str())
        {
            return Err(format!(
                "distillation packet is not an admitted {expected_kind} bound to this workspace/run/admission"
            ));
        }
        Ok(())
    }

    pub(crate) fn require_record_exists(&self, table: &str, id: &RecordId) -> Result<(), String> {
        let rows = self.one(&format!("SELECT id FROM {} LIMIT 1;", record(table, id)))?;
        if rows.as_array().is_none_or(Vec::is_empty) {
            return Err(format!("required record does not exist: {table}:{id}"));
        }
        Ok(())
    }

    fn require_atoms_exist(&self, atoms: &[RecordId]) -> Result<(), String> {
        for atom in atoms {
            self.require_record_exists("atom", atom)?;
        }
        Ok(())
    }

    fn require_compatible_tensor_session(&self, page: &TensorPage) -> Result<(), String> {
        let rows = self.one(&format!(
            "SELECT workspace, dtype, layer_count, kv_head_count, head_dim FROM {} WHERE active = true LIMIT 1;",
            record("transformer_kv_session", &page.session)
        ))?;
        let session = rows
            .as_array()
            .and_then(|rows| rows.first())
            .ok_or_else(|| "active tensor session does not exist".to_string())?;
        if session.get("dtype").and_then(Value::as_str) != Some(page.dtype.as_str()) {
            return Err("tensor page dtype does not match its session".into());
        }
        let layer_count = session
            .get("layer_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| "tensor session layer_count is missing".to_string())?;
        let kv_head_count = session
            .get("kv_head_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| "tensor session kv_head_count is missing".to_string())?;
        let head_dim = session
            .get("head_dim")
            .and_then(Value::as_u64)
            .ok_or_else(|| "tensor session head_dim is missing".to_string())?;
        if u64::from(page.layer) >= layer_count || u64::from(page.kv_head) >= kv_head_count {
            return Err("tensor page layer or KV head is outside its session".into());
        }
        if u64::from(page.tensor_shape[1]) != head_dim {
            return Err("tensor page head dimension does not match its session".into());
        }
        let expected_workspace = record_ref("workspace", &page.workspace);
        if session.get("workspace").and_then(Value::as_str) != Some(&expected_workspace) {
            return Err("tensor page workspace does not match its session".into());
        }
        Ok(())
    }

    pub(crate) fn one(&self, query: &str) -> Result<Value, String> {
        self.sql(query)?
            .into_iter()
            .next()
            .ok_or_else(|| "SurrealDB returned no result".into())
    }

    pub(crate) fn sql(&self, query: &str) -> Result<Vec<Value>, String> {
        let token = self.token()?;
        let body = format!(
            "USE NS {} DB {}; {query}",
            ident(&self.config.namespace),
            ident(&self.config.database)
        );
        let response = http_post(
            &self.host,
            self.port,
            "/sql",
            "text/plain",
            &body,
            token.as_deref(),
        )?;
        let parsed: Value = serde_json::from_str(&response)
            .map_err(|error| format!("invalid SurrealDB response: {error}"))?;
        let statements = parsed
            .as_array()
            .ok_or_else(|| "SurrealDB response was not a statement array".to_string())?;
        let mut results = Vec::new();
        for (index, statement) in statements.iter().enumerate() {
            if index == 0 {
                continue;
            }
            if statement.get("status").and_then(Value::as_str) == Some("ERR") {
                let result = statement.get("result").cloned().unwrap_or(Value::Null);
                return Err(format!("SurrealDB statement {index} failed: {result}"));
            }
            results.push(statement.get("result").cloned().unwrap_or(Value::Null));
        }
        Ok(results)
    }

    fn token(&self) -> Result<Option<String>, String> {
        let (username, password) = match (&self.config.username, &self.config.password) {
            (Some(username), Some(password)) => (username, password),
            _ => return Ok(None),
        };
        let body = if username == "root" {
            format!("{{\"user\":{},\"pass\":{}}}", sq(username), sq(password))
        } else {
            format!(
                "{{\"user\":{},\"pass\":{},\"ns\":{},\"db\":{}}}",
                sq(username),
                sq(password),
                sq(&self.config.namespace),
                sq(&self.config.database)
            )
        };
        let response = http_post(
            &self.host,
            self.port,
            "/signin",
            "application/json",
            &body,
            None,
        )?;
        let value: Value = serde_json::from_str(&response)
            .map_err(|error| format!("invalid signin response: {error}"))?;
        value
            .get("token")
            .and_then(Value::as_str)
            .map(|token| Some(token.to_string()))
            .ok_or_else(|| "SurrealDB signin did not return a token".into())
    }
}

pub(crate) fn require_nonempty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must be non-empty"))
    } else {
        Ok(())
    }
}

pub(crate) fn record(table: &str, id: &RecordId) -> String {
    format!("{table}:`{}`", id.as_str())
}

/// Plain record reference as SurrealDB serializes stored record links in
/// query results. Comparisons against stored values must use this form;
/// the backticked form is only valid inside SurrealQL text.
pub(crate) fn record_ref(table: &str, id: &RecordId) -> String {
    format!("{table}:{}", id.as_str())
}

pub(crate) fn optional_record(table: &str, id: Option<&RecordId>) -> String {
    id.map_or_else(|| "NONE".into(), |id| record(table, id))
}

pub(crate) fn optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "NONE".into(), sq)
}

pub(crate) fn bytes_literal(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(3 + bytes.len() * 2);
    output.push_str("b\"");
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
    }
    output.push('"');
    output
}

pub(crate) fn json_bytes(value: Option<&Value>, field: &str) -> Result<Vec<u8>, String> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| format!("tensor page {field} is not a byte array"))?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| format!("tensor page {field} contains a non-byte value"))
        })
        .collect()
}

pub(crate) fn sq(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}

fn ident(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        value.to_string()
    } else {
        format!("`{}`", value.replace('`', ""))
    }
}

fn parse_host_port(endpoint: &str) -> Result<(String, u16), String> {
    let endpoint = endpoint
        .strip_prefix("http://")
        .ok_or_else(|| "only loopback HTTP SurrealDB endpoints are supported".to_string())?;
    let host_port = endpoint.split('/').next().unwrap_or(endpoint);
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| "SurrealDB endpoint must include a port".to_string())?;
    let port = port
        .parse::<u16>()
        .map_err(|error| format!("invalid SurrealDB port: {error}"))?;
    if host != "127.0.0.1" && host != "localhost" {
        return Err("v1 admits loopback SurrealDB endpoints only".into());
    }
    Ok((host.to_string(), port))
}

fn http_post(
    host: &str,
    port: u16,
    path: &str,
    content_type: &str,
    body: &str,
    token: Option<&str>,
) -> Result<String, String> {
    let address = format!("{host}:{port}");
    let mut stream =
        TcpStream::connect(&address).map_err(|error| format!("connect {address}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|error| error.to_string())?;
    let authorization = token
        .map(|token| format!("\r\nAuthorization: Bearer {token}"))
        .unwrap_or_default();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: {content_type}\r\nAccept: application/json\r\nContent-Length: {}{authorization}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("SurrealDB write failed: {error}"))?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| format!("SurrealDB read failed: {error}"))?;
    let response = String::from_utf8(response)
        .map_err(|error| format!("SurrealDB response was not UTF-8: {error}"))?;
    if response.starts_with("HTTP/1.1 401") || response.starts_with("HTTP/1.1 403") {
        return Err("SurrealDB authentication failed".into());
    }
    if !response.starts_with("HTTP/1.1 200") {
        let status = response.lines().next().unwrap_or("unknown HTTP status");
        return Err(format!("SurrealDB request failed: {status}"));
    }
    let body_start = response
        .find("\r\n\r\n")
        .map(|index| index + 4)
        .ok_or_else(|| "SurrealDB response had no body separator".to_string())?;
    Ok(response[body_start..].to_string())
}
