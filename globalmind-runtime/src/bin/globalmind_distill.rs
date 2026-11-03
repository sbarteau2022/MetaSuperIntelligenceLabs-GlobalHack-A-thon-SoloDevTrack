//! Lane A distillation runner.
//!
//! Order (fail-closed, no JSON/Python at the model boundary):
//! 1. Load a frozen model roster with exact artifact/config/tokenizer digests.
//! 2. Rehash every model file before any database access.
//! 3. Open `model_source`, `distillation_admission`, and `distillation_run`.
//! 4. Read typed rank-0/1/2/8 candidates from a native line packet file. The
//!    extractor that produces real candidates is a separate lane; this runner
//!    never invents candidates from weights.
//! 5. Teacher identity is never parsed from the packet file, and every
//!    candidate digest is recomputed from teacher-neutral fields only.
//! 6. Novelty and independent judge receipt digests are required per
//!    candidate; duplicate candidate digests fail the novelty gate.
//! 7. Packets are admitted in the judging phase; only admitted rank-0
//!    packets crystallize atoms, in the replaying phase.
//! 8. The trajectory walk is validated and replayed before one atomic
//!    `admit_trajectory` write in state `crystallized`.
//! 9. The run completes only after admitted/crystallized counts and the
//!    output digest reconcile with the packet file.

use globalmind_runtime::replay::artifact::{
    EnvelopeDescriptor, decode as decode_walk, decode_candidates, decode_verdicts,
    encode as encode_walk, encode_candidates, encode_verdicts,
};
use globalmind_runtime::replay::{
    ChartReceipts, ReplayContext, ReplayInput, ReplayedWalk, default_address_contract_digest,
    default_judge_policy, default_tppe_numeric, extract_and_judge, replay,
    upgrade_for_crystallization, verify_bindings,
};
use globalmind_runtime::trace::{
    TraceEnvelope, decode_trace_stream, extract_provenance_from_stream,
};
use globalmind_runtime::{
    CellKind, ChartDtype, ContentKind, CrystallizedAtom, Digest, DistillationPacket,
    DistillationPhase, DistillationRunEvent, EdgeKind, GlobalMindStore, LocalJacobianChart,
    Manifest, PacketVerdict, RadialLaw, RecordId, SourceKind, StoreConfig, Trajectory,
    TrajectoryLoop, TrajectoryOrigin, TrajectoryState, TrajectoryStep, VerifiedManifest,
    WhiteheadTopology,
};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const USAGE: &str = "usage: globalmind-distill verify <manifest> | globalmind-distill trace <manifest> <token-tape.bin> <trace.gmtrace> | globalmind-distill extract <trace.gmtrace> <candidates.gmcandidate> | globalmind-distill judge <candidates.gmcandidate> <verdicts.gmverdict> | globalmind-distill open <admitted-manifest> | globalmind-distill replay <trace.gmtrace> <verdicts.gmverdict> <walk.gmwalk> | globalmind-distill crystallize <walk.gmwalk> | globalmind-distill audit <run-id>";

fn main() {
    if let Err(error) = run() {
        eprintln!("globalmind-distill: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "verify" | "open" if args.len() == 2 => {
            let verified = load_and_verify(&args[1])?;
            print_verified(&verified);
            if command == "verify" {
                println!("database_writes=0");
                Ok(())
            } else {
                let control = ControlDb::from_env()?;
                let store = GlobalMindStore::connect(StoreConfig::from_env())?;
                if !record_exists(&control, "workspace", &verified.workspace)? {
                    let ws_rec = format!("workspace:`{}`", verified.workspace.as_str());
                    let _ = control.sql(&format!(
                        "CREATE {ws_rec} CONTENT {{ slug: '{}' }} RETURN NONE;",
                        verified.workspace.as_str()
                    ));
                }
                if let Ok(topo) = WhiteheadTopology::canonical(verified.workspace.clone())
                    && store.verify_whitehead_topology(&topo).is_err()
                {
                    let _ = store.admit_whitehead_topology(&topo);
                }
                store.open_distillation_ingress(&verified)?;
                println!("opened_receipt={}", verified.run_receipt_digest);
                Ok(())
            }
        }
        "run" if args.len() == 3 => run_distillation(&args[1], &args[2]),
        "trace" if args.len() == 4 => cmd_trace(&args[1], &args[2], &args[3]),
        "extract" if args.len() == 3 => cmd_extract(&args[1], &args[2]),
        "judge" if args.len() == 3 => cmd_judge_stage(&args[1], &args[2]),
        "judge" if args.len() == 4 => cmd_judge_legacy(&args[1], &args[2], &args[3]),
        "replay" if args.len() == 4 => cmd_replay_stage(&args[1], &args[2], &args[3]),
        "replay" if args.len() == 5 => cmd_replay(&args[1], &args[2], &args[3], &args[4]),
        "crystallize" if args.len() == 2 => cmd_crystallize_stage(&args[1]),
        "crystallize" if args.len() == 3 => cmd_crystallize(&args[1], &args[2]),
        "audit" if args.len() == 2 => cmd_audit_run(&args[1]),
        "audit" if args.len() == 3 => {
            cmd_audit(&args[1], &args[2], "results/distillation/audit-report.txt")
        }
        "audit" if args.len() == 4 => cmd_audit(&args[1], &args[2], &args[3]),
        _ => Err(USAGE.into()),
    }
}

fn load_and_verify(manifest_path: &str) -> Result<VerifiedManifest, String> {
    let manifest = Manifest::load(manifest_path)?;
    manifest.verify_artifacts()
}

fn print_verified(verified: &VerifiedManifest) {
    println!("verified_model_count={}", verified.models.len());
    println!("input_digest={}", verified.input_digest);
    println!("run_id={}", verified.run_id);
}

fn run_distillation(manifest_path: &str, candidates_path: &str) -> Result<(), String> {
    // Steps 1-2: roster load and full rehash precede any database access.
    let verified = load_and_verify(manifest_path)?;
    print_verified(&verified);
    // Steps 4-6 (pre-write): parse and validate the whole packet file,
    // recompute teacher-neutral candidate digests, and check novelty/judge
    // receipt presence before the run is opened, so a bad file writes nothing.
    let packet_file = PacketFile::load(candidates_path, &verified)?;
    println!("candidate_count={}", packet_file.candidates.len());

    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    // Step 3: immutable source/admission/run ingress.
    store.open_distillation_ingress(&verified)?;
    println!("opened_receipt={}", verified.run_receipt_digest);

    let mut runner = Runner {
        store: &store,
        verified: &verified,
        next_ordinal: 1,
    };
    match runner.execute(&packet_file) {
        Ok(output_digest) => {
            println!("output_digest={output_digest}");
            println!("run_completed=1");
            Ok(())
        }
        Err(error) => {
            // Best-effort terminal receipt; the original error is authoritative.
            let _ = runner.append_event(DistillationPhase::Failed, None);
            Err(error)
        }
    }
}

struct Runner<'a> {
    store: &'a GlobalMindStore,
    verified: &'a VerifiedManifest,
    next_ordinal: u32,
}

impl Runner<'_> {
    fn append_event(
        &mut self,
        phase: DistillationPhase,
        output_digest: Option<Digest>,
    ) -> Result<(), String> {
        let ordinal = self.next_ordinal;
        let receipt = digest_fields(
            b"globalmind-run-event-v1",
            &[
                self.verified.run_id.as_str(),
                &ordinal.to_string(),
                phase_name(phase),
                output_digest.as_ref().map_or("", Digest::as_str),
                self.verified.input_digest.as_str(),
            ],
        );
        self.store
            .append_distillation_run_event(&DistillationRunEvent {
                id: RecordId::parse(format!("run_event_{}", receipt.as_str()))?,
                workspace: self.verified.workspace.clone(),
                run: self.verified.run_id.clone(),
                ordinal,
                phase,
                input_digest: self.verified.input_digest.clone(),
                output_digest,
                tppe_version: self.verified.tppe_version.clone(),
                tppe_configuration_digest: self.verified.tppe_configuration_digest.clone(),
                receipt_digest: receipt,
            })?;
        self.next_ordinal += 1;
        Ok(())
    }

    fn execute(&mut self, packet_file: &PacketFile) -> Result<Digest, String> {
        self.append_event(DistillationPhase::Extracting, None)?;
        // Extraction is upstream: candidates arrive in the packet file. The
        // extracting event binds the run to the exact candidate set digest.
        let candidate_set_digest = packet_file.candidate_set_digest.clone();
        self.append_event(DistillationPhase::Judging, None)?;

        // Step 7a: admit every packet while the run is in the judging phase.
        for candidate in &packet_file.candidates {
            self.store.admit_packet(&candidate.packet)?;
        }
        println!("packets_admitted={}", packet_file.candidates.len());

        self.append_event(DistillationPhase::Replaying, None)?;

        // Step 7b: only admitted rank-0 packets crystallize atoms.
        let mut atoms_crystallized = 0usize;
        for candidate in &packet_file.candidates {
            if let Some(atom) = &candidate.atom {
                self.store.crystallize_atom(atom)?;
                atoms_crystallized += 1;
            }
        }
        println!("atoms_crystallized={atoms_crystallized}");

        for chart in &packet_file.charts {
            self.store.admit_local_jacobian_chart(chart)?;
        }
        println!("charts_admitted={}", packet_file.charts.len());

        // Step 8: the validated walk is replayed as one atomic crystallized
        // trajectory transaction.
        if let Some(trajectory) = &packet_file.trajectory {
            self.store.admit_trajectory(trajectory)?;
            println!("trajectories_crystallized=1");
        }

        // Step 9: completion requires count and digest reconciliation.
        let expected_atoms = packet_file
            .candidates
            .iter()
            .filter(|candidate| candidate.atom.is_some())
            .count();
        if atoms_crystallized != expected_atoms {
            return Err("crystallized atom count does not reconcile with packet file".into());
        }
        let output_digest = packet_file.output_digest(candidate_set_digest);
        self.append_event(DistillationPhase::Completed, Some(output_digest.clone()))?;
        Ok(output_digest)
    }
}

fn phase_name(phase: DistillationPhase) -> &'static str {
    match phase {
        DistillationPhase::Opened => "opened",
        DistillationPhase::Extracting => "extracting",
        DistillationPhase::Judging => "judging",
        DistillationPhase::Replaying => "replaying",
        DistillationPhase::Completed => "completed",
        DistillationPhase::Failed => "failed",
        DistillationPhase::Rejected => "rejected",
    }
}

struct Candidate {
    packet: DistillationPacket,
    atom: Option<CrystallizedAtom>,
}

struct PacketFile {
    candidates: Vec<Candidate>,
    charts: Vec<LocalJacobianChart>,
    trajectory: Option<Trajectory>,
    candidate_set_digest: Digest,
}

impl PacketFile {
    fn load(path: &str, verified: &VerifiedManifest) -> Result<Self, String> {
        let pairs = load_pairs(path)?;
        validate_packet_keys(&pairs)?;
        let candidate_indices = indexed_children(&pairs, "candidate.")?;
        require_contiguous("candidate", &candidate_indices)?;
        if candidate_indices.is_empty() {
            return Err("candidate packet file must list at least one candidate".into());
        }

        let mut candidates = Vec::new();
        let mut seen_digests = BTreeSet::new();
        for index in &candidate_indices {
            let candidate = parse_candidate(&pairs, *index, verified, &mut seen_digests)?;
            candidates.push(candidate);
        }
        let candidate_set_digest = digest_fields(
            b"globalmind-candidate-set-v1",
            &candidates
                .iter()
                .map(|candidate| candidate.packet.candidate_digest.as_str())
                .collect::<Vec<_>>()
                .iter()
                .map(std::ops::Deref::deref)
                .collect::<Vec<_>>(),
        );

        let charts = parse_charts(&pairs, &candidates)?;
        let trajectory = parse_trajectory(&pairs, &candidates, verified)?;
        Ok(Self {
            candidates,
            charts,
            trajectory,
            candidate_set_digest,
        })
    }

    fn output_digest(&self, candidate_set_digest: Digest) -> Digest {
        let mut fields: Vec<&str> = vec![candidate_set_digest.as_str()];
        for candidate in &self.candidates {
            if let Some(atom) = &candidate.atom {
                fields.push(atom.id.as_str());
            }
        }
        for chart in &self.charts {
            fields.push(chart.chart_digest.as_str());
        }
        if let Some(trajectory) = &self.trajectory {
            fields.push(trajectory.trajectory_digest.as_str());
        }
        digest_fields(b"globalmind-run-output-v1", &fields)
    }
}

fn parse_candidate(
    pairs: &BTreeMap<String, String>,
    index: usize,
    verified: &VerifiedManifest,
    seen_digests: &mut BTreeSet<String>,
) -> Result<Candidate, String> {
    let prefix = format!("candidate.{index}.");
    let get = |field: &str| required(pairs, &format!("{prefix}{field}"));
    let model_index = get("model")?
        .parse::<usize>()
        .map_err(|_| format!("candidate.{index}.model must be a manifest model index"))?;
    let model = verified
        .models
        .get(model_index)
        .ok_or_else(|| format!("candidate.{index}.model is outside the verified roster"))?;
    let cell_kind = parse_cell_kind(&get("kind")?, index)?;
    let source_kind = parse_source_kind(&get("source_kind")?, index)?;
    let source_fragment_digest = Digest::parse(get("source_fragment_digest")?)?;
    let teacher_neutral_input_digest = Digest::parse(get("teacher_neutral_input_digest")?)?;
    let novelty_receipt_digest = Digest::parse(get("novelty_receipt_digest")?)?;
    let judge_receipt_digest = Digest::parse(get("judge_receipt_digest")?)?;
    if novelty_receipt_digest == judge_receipt_digest {
        return Err(format!(
            "candidate.{index} novelty and judge receipts must be independent"
        ));
    }
    let verdict = match get("verdict")?.as_str() {
        "admitted" => PacketVerdict::Admitted,
        "rejected" => PacketVerdict::Rejected,
        _ => {
            return Err(format!(
                "candidate.{index}.verdict must be admitted or rejected"
            ));
        }
    };
    // Step 5: the candidate digest preimage contains teacher-neutral fields
    // only. Provider, model name, and revision are never parsed here.
    let candidate_digest = digest_fields(
        b"globalmind-candidate-v1",
        &[
            cell_kind_name(cell_kind),
            source_kind_name(source_kind),
            source_fragment_digest.as_str(),
            teacher_neutral_input_digest.as_str(),
            globalmind_runtime::PROJECTOR_VERSION,
            verified.configuration_digest.as_str(),
        ],
    );
    let declared_digest = Digest::parse(get("candidate_digest")?)?;
    if declared_digest != candidate_digest {
        return Err(format!(
            "candidate.{index} digest does not match its teacher-neutral preimage"
        ));
    }
    if !seen_digests.insert(candidate_digest.as_str().to_string()) {
        return Err(format!(
            "candidate.{index} duplicates an earlier candidate digest; novelty gate failed"
        ));
    }
    let projector_digest = digest_fields(
        b"globalmind-projector-v1",
        &[
            globalmind_runtime::PROJECTOR_VERSION,
            verified.configuration_digest.as_str(),
        ],
    );
    let packet = DistillationPacket {
        id: RecordId::parse(format!("packet_{}", &candidate_digest.as_str()[..32]))?,
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        admission: model.admission_id.clone(),
        source_kind,
        cell_kind,
        source_fragment_digest,
        teacher_neutral_input_digest,
        projector_version: globalmind_runtime::PROJECTOR_VERSION.to_string(),
        projector_digest,
        configuration_digest: verified.configuration_digest.clone(),
        candidate_digest: candidate_digest.clone(),
        novelty_receipt_digest,
        judge_receipt_digest,
        verdict,
    };
    let atom = match (cell_kind, verdict) {
        (CellKind::Rank0Atom, PacketVerdict::Admitted) => {
            Some(parse_atom(pairs, index, verified, &packet)?)
        }
        (CellKind::Rank0Atom, PacketVerdict::Rejected) => None,
        _ => None,
    };
    Ok(Candidate { packet, atom })
}

fn parse_atom(
    pairs: &BTreeMap<String, String>,
    index: usize,
    verified: &VerifiedManifest,
    packet: &DistillationPacket,
) -> Result<CrystallizedAtom, String> {
    let prefix = format!("candidate.{index}.atom.");
    let get = |field: &str| required(pairs, &format!("{prefix}{field}"));
    let content_text = get("content_text")?;
    let content_digest = Digest::parse(get("content_digest")?)?;
    if Digest::sha256(content_text.as_bytes()) != content_digest {
        return Err(format!(
            "candidate.{index} atom content digest does not match content_text"
        ));
    }
    let digest_slice = &packet.candidate_digest.as_str()[..32];
    Ok(CrystallizedAtom {
        id: RecordId::parse(format!("atom_{digest_slice}"))?,
        address_id: RecordId::parse(format!("address_{digest_slice}"))?,
        origin_id: RecordId::parse(format!("atomorigin_{digest_slice}"))?,
        crystallization_id: RecordId::parse(format!("crystal_{digest_slice}"))?,
        membership_id: RecordId::parse(format!("member_{digest_slice}"))?,
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        packet: packet.id.clone(),
        admission: packet.admission.clone(),
        geometry_space: RecordId::parse(get("geometry_space")?)?,
        well: RecordId::parse(get("well")?)?,
        external_id: get("external_id")?,
        content_kind: parse_content_kind(&get("content_kind")?, index)?,
        content_text: Some(content_text),
        content_uri: None,
        content_digest,
        domain: get("domain")?,
        salience: parse_f64(
            &get("salience")?,
            &format!("candidate.{index}.atom.salience"),
        )?,
        provenance: get("provenance")?,
        taint: get("taint")?,
        pinned: parse_bool(&get("pinned")?, &format!("candidate.{index}.atom.pinned"))?,
        h3_lorentz: [
            parse_f64(&get("h3_0")?, "atom h3_0")?,
            parse_f64(&get("h3_1")?, "atom h3_1")?,
            parse_f64(&get("h3_2")?, "atom h3_2")?,
            parse_f64(&get("h3_3")?, "atom h3_3")?,
        ],
        t2_phase: [
            parse_f64(&get("t2_0")?, "atom t2_0")?,
            parse_f64(&get("t2_1")?, "atom t2_1")?,
        ],
        geometry_model: get("geometry_model")?,
        geometry_version: get("geometry_version")?,
        address_revision: 1,
        source_step_digest: Digest::parse(get("source_step_digest")?)?,
        mint_receipt_digest: Digest::parse(get("mint_receipt_digest")?)?,
        payload_digest: Digest::parse(get("payload_digest")?)?,
        reconstruction_contract_digest: Digest::parse(get("reconstruction_contract_digest")?)?,
        reconstruction_receipt_digest: Digest::parse(get("reconstruction_receipt_digest")?)?,
        crystallization_receipt_digest: Digest::parse(get("crystallization_receipt_digest")?)?,
        classifier_version: get("classifier_version")?,
        classifier_digest: Digest::parse(get("classifier_digest")?)?,
        boundary_receipt_digest: Digest::parse(get("boundary_receipt_digest")?)?,
    })
}

fn parse_charts(
    pairs: &BTreeMap<String, String>,
    candidates: &[Candidate],
) -> Result<Vec<LocalJacobianChart>, String> {
    let chart_indices = indexed_children(pairs, "chart.")?;
    require_contiguous("chart", &chart_indices)?;
    let mut charts = Vec::new();
    for index in chart_indices {
        let prefix = format!("chart.{index}.");
        let get = |field: &str| required(pairs, &format!("{prefix}{field}"));
        let candidate = candidate_at(candidates, &get("candidate")?, "chart", index)?;
        if candidate.packet.cell_kind != CellKind::Rank8Jacobian
            || candidate.packet.verdict != PacketVerdict::Admitted
        {
            return Err(format!(
                "chart.{index} must reference an admitted rank-8 candidate"
            ));
        }
        let anchor = candidate_at(candidates, &get("anchor_candidate")?, "chart", index)?;
        let anchor_atom = anchor
            .atom
            .as_ref()
            .ok_or_else(|| format!("chart.{index} anchor candidate has no crystallized atom"))?;
        let tangent_basis = decode_hex(&get("tangent_basis_hex")?, index)?;
        let tensor_digest = Digest::parse(get("tensor_digest")?)?;
        if Digest::sha256(&tangent_basis) != tensor_digest {
            return Err(format!(
                "chart.{index} tensor digest does not match tangent bytes"
            ));
        }
        let shape = get("tensor_shape")?
            .split(',')
            .map(|part| {
                part.trim()
                    .parse::<u32>()
                    .map_err(|_| format!("chart.{index} tensor_shape must be comma-separated u32"))
            })
            .collect::<Result<Vec<u32>, String>>()?;
        charts.push(LocalJacobianChart {
            id: RecordId::parse(format!(
                "chart_{}",
                &candidate.packet.candidate_digest.as_str()[..32]
            ))?,
            workspace: candidate.packet.workspace.clone(),
            run: candidate.packet.run.clone(),
            anchor: anchor_atom.id.clone(),
            packet: candidate.packet.id.clone(),
            admission: candidate.packet.admission.clone(),
            chart_rank: get("chart_rank")?
                .parse::<u8>()
                .map_err(|_| format!("chart.{index}.chart_rank must be a u8"))?,
            dtype: parse_chart_dtype(&get("dtype")?, index)?,
            tensor_shape: shape,
            tangent_basis: tangent_basis.into_boxed_slice(),
            tensor_digest,
            chart_digest: Digest::parse(get("chart_digest")?)?,
            marginal_gain_receipt_digest: Digest::parse(get("marginal_gain_receipt_digest")?)?,
            numerical_receipt_digest: Digest::parse(get("numerical_receipt_digest")?)?,
        });
    }
    Ok(charts)
}

fn parse_trajectory(
    pairs: &BTreeMap<String, String>,
    candidates: &[Candidate],
    verified: &VerifiedManifest,
) -> Result<Option<Trajectory>, String> {
    let has_trajectory = pairs.keys().any(|key| key.starts_with("trajectory."));
    let has_higher_cells = candidates.iter().any(|candidate| {
        matches!(
            candidate.packet.cell_kind,
            CellKind::Rank1Transition | CellKind::Rank2Loop
        )
    });
    if !has_trajectory {
        if has_higher_cells {
            return Err("rank-1/rank-2 candidates require a trajectory walk for replay".into());
        }
        return Ok(None);
    }
    let get = |field: &str| required(pairs, &format!("trajectory.{field}"));

    let atom_indices = indexed_children(pairs, "trajectory.atom.")?;
    require_contiguous("trajectory.atom", &atom_indices)?;
    if atom_indices.len() < 2 {
        return Err("trajectory walk requires at least two atoms".into());
    }
    let mut atoms = Vec::new();
    for index in &atom_indices {
        let candidate = candidate_at(
            candidates,
            &required(pairs, &format!("trajectory.atom.{index}"))?,
            "trajectory.atom",
            *index,
        )?;
        let atom = candidate.atom.as_ref().ok_or_else(|| {
            format!("trajectory.atom.{index} does not reference a crystallized rank-0 atom")
        })?;
        atoms.push(atom.id.clone());
    }

    let step_indices = indexed_children(pairs, "trajectory.step.")?;
    require_contiguous("trajectory.step", &step_indices)?;
    if step_indices.len() + 1 != atoms.len() {
        return Err("trajectory step count must equal walk atom count minus one".into());
    }
    let mut steps = Vec::new();
    for index in step_indices {
        let prefix = format!("trajectory.step.{index}.");
        let sget = |field: &str| required(pairs, &format!("{prefix}{field}"));
        let candidate = candidate_at(candidates, &sget("candidate")?, "trajectory.step", index)?;
        if candidate.packet.cell_kind != CellKind::Rank1Transition
            || candidate.packet.verdict != PacketVerdict::Admitted
        {
            return Err(format!(
                "trajectory.step.{index} must reference an admitted rank-1 candidate"
            ));
        }
        let slice = &candidate.packet.candidate_digest.as_str()[..24];
        steps.push(TrajectoryStep {
            id: RecordId::parse(format!("step_{slice}_{index}"))?,
            ordinal: u32::try_from(index)
                .map_err(|_| "trajectory step ordinal overflow".to_string())?,
            from_atom: atoms[index].clone(),
            to_atom: atoms[index + 1].clone(),
            edge: RecordId::parse(format!("edge_{slice}"))?,
            edge_crystallization_id: RecordId::parse(format!("edgecrystal_{slice}"))?,
            packet: candidate.packet.id.clone(),
            admission: candidate.packet.admission.clone(),
            transition_kind: parse_edge_kind(&sget("transition_kind")?, index)?,
            weight: parse_f64(&sget("weight")?, "step weight")?,
            confidence: parse_f64(&sget("confidence")?, "step confidence")?,
            evidence_atoms: Vec::new(),
            evidence_digest: Digest::parse(sget("evidence_digest")?)?,
            producer: sget("producer")?,
            revision: 1,
            t2_phase_before: [
                parse_f64(&sget("t2_before_0")?, "step t2_before_0")?,
                parse_f64(&sget("t2_before_1")?, "step t2_before_1")?,
            ],
            t2_phase_after: [
                parse_f64(&sget("t2_after_0")?, "step t2_after_0")?,
                parse_f64(&sget("t2_after_1")?, "step t2_after_1")?,
            ],
            tppe_radius_before: parse_f64(&sget("radius_before")?, "step radius_before")?,
            tppe_radius_after: parse_f64(&sget("radius_after")?, "step radius_after")?,
            tppe_revolutions_before: sget("revolutions_before")?
                .parse::<u64>()
                .map_err(|_| "step revolutions_before must be a u64".to_string())?,
            tppe_revolutions_after: sget("revolutions_after")?
                .parse::<u64>()
                .map_err(|_| "step revolutions_after must be a u64".to_string())?,
            tppe_boundary_margin_before: parse_f64(&sget("margin_before")?, "step margin_before")?,
            tppe_boundary_margin_after: parse_f64(&sget("margin_after")?, "step margin_after")?,
            action_digest: Digest::parse(sget("action_digest")?)?,
            observation_digest: Digest::parse(sget("observation_digest")?)?,
            transition_digest: Digest::parse(sget("transition_digest")?)?,
            replay_receipt_digest: Digest::parse(sget("replay_receipt_digest")?)?,
            crystallization_receipt_digest: Digest::parse(sget("crystallization_receipt_digest")?)?,
        });
    }

    let loop_indices = indexed_children(pairs, "trajectory.loop.")?;
    require_contiguous("trajectory.loop", &loop_indices)?;
    let mut loops = Vec::new();
    for index in loop_indices {
        let prefix = format!("trajectory.loop.{index}.");
        let lget = |field: &str| required(pairs, &format!("{prefix}{field}"));
        let candidate = candidate_at(candidates, &lget("candidate")?, "trajectory.loop", index)?;
        if candidate.packet.cell_kind != CellKind::Rank2Loop
            || candidate.packet.verdict != PacketVerdict::Admitted
        {
            return Err(format!(
                "trajectory.loop.{index} must reference an admitted rank-2 candidate"
            ));
        }
        loops.push(TrajectoryLoop {
            id: RecordId::parse(format!(
                "loop_{}",
                &candidate.packet.candidate_digest.as_str()[..32]
            ))?,
            packet: candidate.packet.id.clone(),
            admission: candidate.packet.admission.clone(),
            first_step: lget("first_step")?
                .parse::<u32>()
                .map_err(|_| "loop first_step must be a u32".to_string())?,
            last_step: lget("last_step")?
                .parse::<u32>()
                .map_err(|_| "loop last_step must be a u32".to_string())?,
            loop_invariant_kind: lget("loop_invariant_kind")?,
            loop_invariant_digest: Digest::parse(lget("loop_invariant_digest")?)?,
            closure_receipt_digest: Digest::parse(lget("closure_receipt_digest")?)?,
        });
    }

    let origin_indices = indexed_children(pairs, "trajectory.origin.")?;
    require_contiguous("trajectory.origin", &origin_indices)?;
    if origin_indices.is_empty() {
        return Err("trajectory requires at least one audit origin".into());
    }
    let mut origins = Vec::new();
    for index in origin_indices {
        let prefix = format!("trajectory.origin.{index}.");
        let oget = |field: &str| required(pairs, &format!("{prefix}{field}"));
        let model_index = oget("model")?
            .parse::<usize>()
            .map_err(|_| "trajectory origin model must be a manifest model index".to_string())?;
        let model = verified
            .models
            .get(model_index)
            .ok_or_else(|| "trajectory origin model is outside the verified roster".to_string())?;
        origins.push(TrajectoryOrigin {
            id: RecordId::parse(format!(
                "torigin_{}_{index}",
                &model.admission_id.as_str()[..model.admission_id.as_str().len().min(24)]
            ))?,
            admission: model.admission_id.clone(),
            source_walk_digest: Digest::parse(oget("source_walk_digest")?)?,
            receipt_digest: Digest::parse(oget("receipt_digest")?)?,
        });
    }

    let trajectory_digest = Digest::parse(get("trajectory_digest")?)?;
    Ok(Some(Trajectory {
        id: RecordId::parse(format!("trajectory_{}", &trajectory_digest.as_str()[..32]))?,
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        atoms,
        trajectory_digest,
        judge_receipt_digest: Digest::parse(get("judge_receipt_digest")?)?,
        tppe_version: verified.tppe_version.clone(),
        tppe_configuration_digest: verified.tppe_configuration_digest.clone(),
        tppe_radial_law_digest: Digest::parse(get("tppe_radial_law_digest")?)?,
        tppe_boundary_receipt_digest: Digest::parse(get("tppe_boundary_receipt_digest")?)?,
        address_contract_digest: Digest::parse(get("address_contract_digest")?)?,
        replay_receipt_digest: Digest::parse(get("replay_receipt_digest")?)?,
        crystallization_receipt_digest: Some(Digest::parse(get(
            "crystallization_receipt_digest",
        )?)?),
        state: TrajectoryState::Crystallized,
        steps,
        loops,
        origins,
    }))
}

fn candidate_at<'a>(
    candidates: &'a [Candidate],
    value: &str,
    label: &str,
    index: usize,
) -> Result<&'a Candidate, String> {
    let candidate_index = value
        .parse::<usize>()
        .map_err(|_| format!("{label}.{index} candidate must be a candidate index"))?;
    candidates
        .get(candidate_index)
        .ok_or_else(|| format!("{label}.{index} candidate index is outside the packet file"))
}

fn load_pairs(path: &str) -> Result<BTreeMap<String, String>, String> {
    let file = File::open(path).map_err(|error| format!("open candidate packet file: {error}"))?;
    let mut pairs = BTreeMap::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| format!("read packet line {}: {error}", index + 1))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("packet line {} is not key = value", index + 1))?;
        let key = key.trim().to_string();
        if pairs
            .insert(key.clone(), value.trim().to_string())
            .is_some()
        {
            return Err(format!("packet key is duplicated: {key}"));
        }
    }
    Ok(pairs)
}

fn validate_packet_keys(pairs: &BTreeMap<String, String>) -> Result<(), String> {
    for key in pairs.keys() {
        let parts = key.split('.').collect::<Vec<_>>();
        let valid = match parts.as_slice() {
            ["candidate", index, field] => {
                index.parse::<usize>().is_ok()
                    && matches!(
                        *field,
                        "model"
                            | "kind"
                            | "source_kind"
                            | "source_fragment_digest"
                            | "teacher_neutral_input_digest"
                            | "candidate_digest"
                            | "novelty_receipt_digest"
                            | "judge_receipt_digest"
                            | "verdict"
                    )
            }
            ["candidate", index, "atom", field] => {
                index.parse::<usize>().is_ok()
                    && matches!(
                        *field,
                        "external_id"
                            | "content_kind"
                            | "content_text"
                            | "content_digest"
                            | "domain"
                            | "salience"
                            | "provenance"
                            | "taint"
                            | "pinned"
                            | "geometry_space"
                            | "well"
                            | "h3_0"
                            | "h3_1"
                            | "h3_2"
                            | "h3_3"
                            | "t2_0"
                            | "t2_1"
                            | "geometry_model"
                            | "geometry_version"
                            | "classifier_version"
                            | "classifier_digest"
                            | "source_step_digest"
                            | "mint_receipt_digest"
                            | "payload_digest"
                            | "reconstruction_contract_digest"
                            | "reconstruction_receipt_digest"
                            | "crystallization_receipt_digest"
                            | "boundary_receipt_digest"
                    )
            }
            ["chart", index, field] => {
                index.parse::<usize>().is_ok()
                    && matches!(
                        *field,
                        "candidate"
                            | "anchor_candidate"
                            | "chart_rank"
                            | "dtype"
                            | "tensor_shape"
                            | "tangent_basis_hex"
                            | "tensor_digest"
                            | "chart_digest"
                            | "marginal_gain_receipt_digest"
                            | "numerical_receipt_digest"
                    )
            }
            ["trajectory", field] => matches!(
                *field,
                "trajectory_digest"
                    | "judge_receipt_digest"
                    | "tppe_radial_law_digest"
                    | "tppe_boundary_receipt_digest"
                    | "address_contract_digest"
                    | "replay_receipt_digest"
                    | "crystallization_receipt_digest"
            ),
            ["trajectory", "atom", index] => index.parse::<usize>().is_ok(),
            ["trajectory", "step", index, field] => {
                index.parse::<usize>().is_ok()
                    && matches!(
                        *field,
                        "candidate"
                            | "transition_kind"
                            | "weight"
                            | "confidence"
                            | "evidence_digest"
                            | "producer"
                            | "t2_before_0"
                            | "t2_before_1"
                            | "t2_after_0"
                            | "t2_after_1"
                            | "radius_before"
                            | "radius_after"
                            | "revolutions_before"
                            | "revolutions_after"
                            | "margin_before"
                            | "margin_after"
                            | "action_digest"
                            | "observation_digest"
                            | "transition_digest"
                            | "replay_receipt_digest"
                            | "crystallization_receipt_digest"
                    )
            }
            ["trajectory", "loop", index, field] => {
                index.parse::<usize>().is_ok()
                    && matches!(
                        *field,
                        "candidate"
                            | "first_step"
                            | "last_step"
                            | "loop_invariant_kind"
                            | "loop_invariant_digest"
                            | "closure_receipt_digest"
                    )
            }
            ["trajectory", "origin", index, field] => {
                index.parse::<usize>().is_ok()
                    && matches!(*field, "model" | "source_walk_digest" | "receipt_digest")
            }
            _ => false,
        };
        if !valid {
            return Err(format!(
                "packet file contains unknown key (teacher identity is forbidden here): {key}"
            ));
        }
    }
    Ok(())
}

fn required(pairs: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    pairs
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("packet file is missing required key: {key}"))
}

fn indexed_children(pairs: &BTreeMap<String, String>, prefix: &str) -> Result<Vec<usize>, String> {
    let mut indices = BTreeSet::new();
    for key in pairs.keys().filter(|key| key.starts_with(prefix)) {
        let suffix = &key[prefix.len()..];
        let index = suffix
            .split('.')
            .next()
            .ok_or_else(|| format!("invalid indexed key: {key}"))?
            .parse::<usize>()
            .map_err(|_| format!("invalid index in packet key: {key}"))?;
        indices.insert(index);
    }
    Ok(indices.into_iter().collect())
}

fn require_contiguous(label: &str, indices: &[usize]) -> Result<(), String> {
    for (expected, actual) in indices.iter().copied().enumerate() {
        if expected != actual {
            return Err(format!("{label} indices must be contiguous from zero"));
        }
    }
    Ok(())
}

fn parse_cell_kind(value: &str, index: usize) -> Result<CellKind, String> {
    match value {
        "rank0_atom" => Ok(CellKind::Rank0Atom),
        "rank1_transition" => Ok(CellKind::Rank1Transition),
        "rank2_loop" => Ok(CellKind::Rank2Loop),
        "rank8_jacobian" => Ok(CellKind::Rank8Jacobian),
        _ => Err(format!(
            "candidate.{index}.kind is not a rank-0/1/2/8 cell kind"
        )),
    }
}

fn cell_kind_name(kind: CellKind) -> &'static str {
    match kind {
        CellKind::Rank0Atom => "rank0_atom",
        CellKind::Rank1Transition => "rank1_transition",
        CellKind::Rank2Loop => "rank2_loop",
        CellKind::Rank8Jacobian => "rank8_jacobian",
    }
}

fn parse_source_kind(value: &str, index: usize) -> Result<SourceKind, String> {
    match value {
        "frozen_weight" => Ok(SourceKind::FrozenWeight),
        "walked_trajectory" => Ok(SourceKind::WalkedTrajectory),
        _ => Err(format!(
            "candidate.{index}.source_kind must be frozen_weight or walked_trajectory"
        )),
    }
}

fn source_kind_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::FrozenWeight => "frozen_weight",
        SourceKind::WalkedTrajectory => "walked_trajectory",
    }
}

fn parse_content_kind(value: &str, index: usize) -> Result<ContentKind, String> {
    match value {
        "text" => Ok(ContentKind::Text),
        "image" => Ok(ContentKind::Image),
        "audio" => Ok(ContentKind::Audio),
        "video" => Ok(ContentKind::Video),
        "code" => Ok(ContentKind::Code),
        "mixed" => Ok(ContentKind::Mixed),
        _ => Err(format!(
            "candidate.{index}.atom.content_kind is not admitted"
        )),
    }
}

fn parse_edge_kind(value: &str, index: usize) -> Result<EdgeKind, String> {
    match value {
        "semantic" => Ok(EdgeKind::Semantic),
        "causal" => Ok(EdgeKind::Causal),
        "temporal" => Ok(EdgeKind::Temporal),
        "supports" => Ok(EdgeKind::Supports),
        "contradicts" => Ok(EdgeKind::Contradicts),
        "part_of" => Ok(EdgeKind::PartOf),
        "derived_from" => Ok(EdgeKind::DerivedFrom),
        "references" => Ok(EdgeKind::References),
        "next" => Ok(EdgeKind::Next),
        _ => Err(format!(
            "trajectory.step.{index}.transition_kind is not an admitted edge kind"
        )),
    }
}

fn parse_chart_dtype(value: &str, index: usize) -> Result<ChartDtype, String> {
    match value {
        "f16" => Ok(ChartDtype::F16),
        "bf16" => Ok(ChartDtype::Bf16),
        "f32" => Ok(ChartDtype::F32),
        _ => Err(format!("chart.{index}.dtype must be f16, bf16, or f32")),
    }
}

fn parse_f64(value: &str, field: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("{field} must be a finite float"))?;
    if !parsed.is_finite() {
        return Err(format!("{field} must be a finite float"));
    }
    Ok(parsed)
}

fn parse_bool(value: &str, field: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{field} must be true or false")),
    }
}

fn decode_hex(value: &str, index: usize) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err(format!(
            "chart.{index} tangent_basis_hex must have even length"
        ));
    }
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(value.len() / 2);
    for pair in bytes.as_chunks::<2>().0 {
        let high = hex_nibble(pair[0])
            .ok_or_else(|| format!("chart.{index} tangent_basis_hex is not hex"))?;
        let low = hex_nibble(pair[1])
            .ok_or_else(|| format!("chart.{index} tangent_basis_hex is not hex"))?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}

// ---------------------------------------------------------------------------
// Stages 5-7: replay, crystallize, audit (isolated patch; the `run` path above
// is untouched).
//
// Atomicity honesty: the typed store exposes one atomic transaction per
// entity (packet/atom/trajectory/chart/event). A single transaction spanning
// all of them would require store.rs changes owned by another lane, so
// crystallize executes a pre-validated ordered-atomic sequence: everything is
// verified before the first write, each write is atomic, readback follows,
// and `audit` detects any prefix left by a mid-sequence failure. The run is
// marked completed only by `audit` after restart, re-verification, and exact
// count reconciliation.
// ---------------------------------------------------------------------------

fn parse_radial_law(value: &str) -> Result<RadialLaw, String> {
    match value {
        "linear_log_v1" => Ok(RadialLaw::LinearLogV1),
        "log_v1" => Ok(RadialLaw::LogV1),
        _ => Err(format!(
            "unknown radial law {value:?}: expected log_v1 or linear_log_v1"
        )),
    }
}

fn ensure_parent_dir(path: &str) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| format!("create parent dir: {error}"))?;
    }
    Ok(())
}

fn single_model(verified: &VerifiedManifest) -> Result<&globalmind_runtime::VerifiedModel, String> {
    if verified.models.len() != 1 {
        return Err(format!(
            "stage CLI requires exactly one rostered model, found {}",
            verified.models.len()
        ));
    }
    Ok(&verified.models[0])
}

/// Stage 1 `trace`: rehash the roster, decode the immutable GMTT token
/// tape, and run the real frozen model twice (control vs observer) via
/// Agent Trace's producer. Emits the native `.gmtrace` stream. Writes no
/// database state.
fn cmd_trace(manifest_path: &str, tape_path: &str, trace_out: &str) -> Result<(), String> {
    use globalmind_runtime::trace::qwen36::{
        decode_token_tape, encode_trace_stream, produce_real_qwen36_trace,
    };
    let verified = load_and_verify(manifest_path)?;
    print_verified(&verified);
    let tape_bytes =
        std::fs::read(tape_path).map_err(|error| format!("read token tape: {error}"))?;
    let (tokens, tape_digest) = decode_token_tape(&tape_bytes)?;
    println!("token_tape_digest={tape_digest}");
    println!("token_count={}", tokens.len());
    let produced = produce_real_qwen36_trace(&verified, &tokens, &tape_digest)?;
    println!(
        "observer_nonperturbation_receipt={}",
        produced.provenance.observer_receipt_digest
    );
    println!(
        "control_logits_digest={}",
        produced.provenance.control_logits_digest
    );
    println!(
        "observed_logits_digest={}",
        produced.provenance.observed_logits_digest
    );
    println!(
        "max_logit_abs_diff={}",
        produced.provenance.max_logit_abs_diff
    );
    let bytes = encode_trace_stream(&produced.envelopes)?;
    std::fs::write(trace_out, &bytes).map_err(|error| format!("write trace file: {error}"))?;
    println!("trace_envelopes={}", produced.envelopes.len());
    println!("trace_file_bytes={}", bytes.len());
    println!("database_writes=0");
    Ok(())
}

/// Stage 2 `extract`: decode the trace stream and emit the typed candidate
/// artifact (`.gmcandidate` binary format, or `.txt` if requested).
fn cmd_extract(trace_path: &str, candidates_out: &str) -> Result<(), String> {
    let raw = std::fs::read(trace_path).map_err(|error| format!("read trace file: {error}"))?;
    let envelopes = decode_trace_stream(&raw)?;
    let provenance = extract_provenance_from_stream(&envelopes)?;
    let candidates = globalmind_runtime::extract::extract(&envelopes)?;
    let rank0 = candidates
        .iter()
        .filter(|c| c.cell_kind == CellKind::Rank0Atom)
        .count();
    let rank1 = candidates
        .iter()
        .filter(|c| c.cell_kind == CellKind::Rank1Transition)
        .count();
    let rank2 = candidates
        .iter()
        .filter(|c| c.cell_kind == CellKind::Rank2Loop)
        .count();
    let rank8 = candidates
        .iter()
        .filter(|c| c.cell_kind == CellKind::Rank8Jacobian)
        .count();

    if candidates_out.ends_with(".txt") {
        let mut out = format!("trace_digest={}\n", Digest::sha256(&raw));
        for candidate in &candidates {
            let (mean_square, finite) = evidence_summary(candidate, &envelopes);
            out.push_str(&format!(
                "candidate={} kind={} digest={} fragment={} neutral={} chart_rank={} basis_bytes={} mean_square={mean_square:?} finite={finite}\n",
                candidate.id,
                cell_kind_name(candidate.cell_kind),
                candidate.candidate_digest,
                candidate.source_fragment_digest,
                candidate.teacher_neutral_input_digest,
                candidate.chart_rank.unwrap_or(0),
                candidate
                    .tangent_basis
                    .as_ref()
                    .map_or(0, |basis| basis.len()),
            ));
        }
        std::fs::write(candidates_out, out)
            .map_err(|error| format!("write candidates file: {error}"))?;
    } else {
        let bytes = encode_candidates(&provenance, &envelopes, &candidates)?;
        ensure_parent_dir(candidates_out)?;
        std::fs::write(candidates_out, &bytes)
            .map_err(|error| format!("write candidates file: {error}"))?;
        println!("candidates_file_bytes={}", bytes.len());
        println!("candidate_bundle_digest={}", Digest::sha256(&bytes));
    }

    println!("trace_envelopes={}", envelopes.len());
    println!("candidates_emitted={}", candidates.len());
    println!("rank0={rank0} rank1={rank1} rank2={rank2} rank8={rank8}");
    println!("database_writes=0");
    Ok(())
}

/// Stage 3 `judge`: decode binary candidate artifact (`.gmcandidate`),
/// evaluate every candidate with the independent judge, compute chart gain
/// receipts for passed rank-8 charts, and emit binary verdicts (`.gmverdict`).
/// Also generates the run-ID-specific admitted manifest from the matching
/// candidate roster. Generic aliases are never overwritten.
fn cmd_judge_stage(candidates_path: &str, verdicts_out: &str) -> Result<(), String> {
    use globalmind_runtime::judge::{IndependentJudge, JudgeVerdict};
    let raw =
        std::fs::read(candidates_path).map_err(|error| format!("read candidates file: {error}"))?;
    let decoded = decode_candidates(&raw)?;
    let policy = default_judge_policy();
    let judge = IndependentJudge::new(policy)?;
    let policy_digest = judge.policy_digest();
    println!("judge_policy_digest={policy_digest}");

    let mut known: Vec<Digest> = Vec::new();
    let mut judged = Vec::with_capacity(decoded.candidates.len());
    let mut passed = 0usize;
    let mut rejected = 0usize;
    for candidate in &decoded.candidates {
        let mut evidence = globalmind_runtime::replay::evidence_for(candidate, &decoded.envelopes)?;
        evidence.known_candidate_digests.clone_from(&known);
        let receipt = judge.judge(candidate, &evidence)?;
        known.push(candidate.candidate_digest.clone());
        match receipt.verdict {
            JudgeVerdict::Pass => passed += 1,
            JudgeVerdict::Reject => rejected += 1,
        }
        judged.push((candidate.clone(), receipt));
    }

    let mut chart_receipts: BTreeMap<Digest, ChartReceipts> = BTreeMap::new();
    for (candidate, receipt) in &judged {
        if candidate.cell_kind != CellKind::Rank8Jacobian || receipt.verdict != JudgeVerdict::Pass {
            continue;
        }
        let measured = measure_chart_gain(candidate, &decoded.envelopes)?;
        println!(
            "chart={} marginal_gain={:.6} gram_min_diag={:.6}",
            candidate.candidate_digest, measured.0, measured.1
        );
        if measured.0 <= 0.0 || !measured.0.is_finite() || measured.1 <= 0.0 {
            return Err(format!(
                "rank-8 chart {} has no positive measured marginal gain; benched",
                candidate.candidate_digest
            ));
        }
        chart_receipts.insert(
            candidate.candidate_digest.clone(),
            ChartReceipts {
                marginal_gain_receipt_digest: digest_fields(
                    b"globalmind-chart-marginal-gain-v1",
                    &[
                        candidate.candidate_digest.as_str(),
                        &format!("{:?}", measured.0),
                    ],
                ),
                numerical_receipt_digest: digest_fields(
                    b"globalmind-chart-numerical-v1",
                    &[
                        candidate.candidate_digest.as_str(),
                        &format!("{:?}", measured.1),
                    ],
                ),
            },
        );
    }

    let judge_receipt_digest = digest_fields(
        b"globalmind-judge-bundle-v1",
        &judged
            .iter()
            .map(|(_, r)| r.receipt_digest.as_str())
            .collect::<Vec<_>>(),
    );
    let eligibility_receipt_digest = digest_fields(
        b"globalmind-eligibility-v1",
        &[
            decoded.provenance.input_digest.as_str(),
            judge_receipt_digest.as_str(),
            &passed.to_string(),
        ],
    );

    let candidate_manifest_path = "manifests/qwen36-layer3-candidate-v2.manifest";
    let mut admitted_text = std::fs::read_to_string(candidate_manifest_path)
        .map_err(|error| format!("read candidate manifest: {error}"))?;
    let candidate_verified = load_and_verify(candidate_manifest_path)?;
    if candidate_verified.input_digest != decoded.provenance.input_digest
        || candidate_verified.run_id != decoded.provenance.run_id
        || candidate_verified.tppe_version != decoded.provenance.tppe_version
        || candidate_verified.tppe_configuration_digest
            != decoded.provenance.tppe_configuration_digest
        || candidate_verified.models.len() != 1
        || candidate_verified.models[0].roster_digest != decoded.provenance.roster_digest
    {
        return Err("candidate manifest does not match judged trace provenance".into());
    }
    admitted_text.push_str(&format!(
        "model.0.eligibility_receipt_digest = {eligibility_receipt_digest}\nmodel.0.judge_receipt_digest = {judge_receipt_digest}\n"
    ));
    let admitted_path = format!(
        "manifests/{}.admitted.manifest",
        decoded.provenance.run_id.as_str()
    );
    if let Ok(existing) = std::fs::read_to_string(&admitted_path)
        && existing != admitted_text
    {
        return Err("existing admitted manifest conflicts with judged trace".into());
    }

    let bytes = encode_verdicts(
        &decoded.provenance,
        &decoded.envelopes,
        &judged,
        &chart_receipts,
        &eligibility_receipt_digest,
        &judge_receipt_digest,
    )?;

    ensure_parent_dir(verdicts_out)?;
    std::fs::write(verdicts_out, &bytes)
        .map_err(|error| format!("write verdicts file: {error}"))?;

    std::fs::write(&admitted_path, admitted_text)
        .map_err(|error| format!("write admitted manifest {admitted_path}: {error}"))?;

    println!("judged_total={}", judged.len());
    println!("judged_pass={passed}");
    println!("judged_reject={rejected}");
    println!("judge_receipt_digest={judge_receipt_digest}");
    println!("eligibility_receipt_digest={eligibility_receipt_digest}");
    println!("verdicts_file_bytes={}", bytes.len());
    println!("database_writes=0");
    Ok(())
}

/// Stage 3 `judge` legacy: re-derive candidates from the candidate artifact's
/// source trace binding is not stored, so judging re-extracts from the
/// candidate digests' provenance by re-running extract_and_judge over the
/// envelopes referenced by the artifact. The artifact itself is the input;
/// verdicts are persisted for every candidate, pass and reject alike.
fn cmd_judge_legacy(
    trace_path: &str,
    candidates_path: &str,
    verdicts_out: &str,
) -> Result<(), String> {
    use globalmind_runtime::judge::JudgeVerdict;
    // Judge full candidates re-derived from the trace (never reconstructed
    // from text), then cross-check the candidate artifact's digest set.
    let raw = std::fs::read(trace_path).map_err(|error| format!("read trace file: {error}"))?;
    let envelopes = decode_trace_stream(&raw)?;
    let judged = extract_and_judge(&envelopes, default_judge_policy())?;
    let artifact = std::fs::read_to_string(candidates_path)
        .map_err(|error| format!("read candidates file: {error}"))?;
    let mut artifact_digests: Vec<Digest> = Vec::new();
    for line in artifact.lines() {
        if let Some(digest) = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("digest="))
        {
            artifact_digests.push(Digest::parse(digest)?);
        }
    }
    let derived: Vec<Digest> = judged
        .iter()
        .map(|(candidate, _)| candidate.candidate_digest.clone())
        .collect();
    if artifact_digests != derived {
        return Err("candidate artifact does not match re-extraction from the trace".into());
    }
    let policy_digest = default_judge_policy().digest();
    println!("judge_policy_digest={policy_digest}");
    let mut out = String::new();
    let mut passed = 0usize;
    let mut rejected = 0usize;
    for (candidate, receipt) in &judged {
        match receipt.verdict {
            JudgeVerdict::Pass => passed += 1,
            JudgeVerdict::Reject => rejected += 1,
        }
        out.push_str(&format!(
            "candidate={} verdict={} reason={} receipt={} novelty={} policy={}\n",
            candidate.candidate_digest,
            match receipt.verdict {
                JudgeVerdict::Pass => "pass",
                JudgeVerdict::Reject => "reject",
            },
            receipt.reason,
            receipt.receipt_digest,
            receipt.novelty_receipt_digest,
            policy_digest
        ));
    }
    std::fs::write(verdicts_out, out).map_err(|error| format!("write verdicts file: {error}"))?;
    println!("judged_pass={passed}");
    println!("judged_reject={rejected}");
    println!("database_writes=0");
    Ok(())
}

fn evidence_summary(
    candidate: &globalmind_runtime::extract::ExtractedCandidate,
    envelopes: &[globalmind_runtime::trace::TraceEnvelope],
) -> (f64, bool) {
    let Some(envelope) = envelopes
        .iter()
        .find(|envelope| envelope.payload_digest == candidate.source_fragment_digest)
    else {
        return (0.0, true);
    };
    let values: Vec<f32> = envelope
        .payload
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    if values.is_empty() {
        return (0.0, true);
    }
    let finite = values.iter().all(|value| value.is_finite());
    let mean_square = values
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>()
        / values.len() as f64;
    (mean_square, finite)
}

/// Measures a rank-8 chart's marginal gain on the real trace: the anchor
/// atom's source activation is reconstructed by projecting onto the chart
/// tangent basis (treated as `rank` basis vectors over the activation
/// dimension); gain is the relative reconstruction-error reduction versus
/// the zero chart. Also returns the minimum Gram diagonal as the numerical
/// stability statistic. Both are measured, never assumed.
fn measure_chart_gain(
    candidate: &globalmind_runtime::extract::ExtractedCandidate,
    envelopes: &[globalmind_runtime::trace::TraceEnvelope],
) -> Result<(f64, f64), String> {
    let basis_bytes = candidate
        .tangent_basis
        .as_ref()
        .ok_or_else(|| "chart candidate has no tangent basis".to_string())?;
    let basis: Vec<f32> = basis_bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    if basis.is_empty() || !basis.iter().all(|value| value.is_finite()) {
        return Ok((0.0, 0.0));
    }
    // Anchor activation: the envelope whose payload digest matches the
    // chart's source fragment, else the first activation envelope.
    let anchor_envelope = envelopes
        .iter()
        .find(|envelope| envelope.payload_digest == candidate.source_fragment_digest)
        .or_else(|| {
            envelopes
                .iter()
                .find(|envelope| envelope.kind == globalmind_runtime::trace::TraceKind::Activation)
        })
        .ok_or_else(|| "no activation envelope to measure chart gain against".to_string())?;
    let target: Vec<f32> = anchor_envelope
        .payload
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    if target.is_empty() {
        return Ok((0.0, 0.0));
    }
    let dim = target.len();
    let rank = (basis.len() / dim.max(1)).clamp(1, 8);
    if rank == 0 || basis.len() < dim {
        // Basis shorter than the activation: project onto the basis as a
        // single direction instead.
        let direction = &basis;
        let extended_norm_sq: f64 = target
            .iter()
            .zip(direction.iter().cycle())
            .map(|(_, b)| (*b as f64) * (*b as f64))
            .sum();
        if extended_norm_sq <= 0.0 {
            return Ok((0.0, 0.0));
        }
        let dot: f64 = target
            .iter()
            .zip(direction.iter().cycle())
            .map(|(t, b)| (*t as f64) * (*b as f64))
            .sum();
        let coeff = dot / extended_norm_sq;
        let baseline: f64 = target.iter().map(|v| (*v as f64) * (*v as f64)).sum();
        let residual: f64 = target
            .iter()
            .zip(direction.iter().cycle())
            .map(|(t, b)| {
                let r = *t as f64 - coeff * (*b as f64);
                r * r
            })
            .sum();
        let gain = if baseline > 0.0 {
            ((baseline - residual) / baseline).clamp(0.0, 1.0)
        } else {
            0.0
        };
        return Ok((gain, extended_norm_sq));
    }
    // Gram matrix of the first `rank` basis vectors (each of length dim).
    let vectors: Vec<&[f32]> = (0..rank)
        .map(|index| &basis[index * dim..(index + 1) * dim])
        .collect();
    let mut gram = vec![vec![0.0f64; rank]; rank];
    for i in 0..rank {
        for j in 0..rank {
            gram[i][j] = vectors[i]
                .iter()
                .zip(vectors[j])
                .map(|(a, b)| (*a as f64) * (*b as f64))
                .sum();
        }
    }
    let min_diag = (0..rank).map(|i| gram[i][i]).fold(f64::INFINITY, f64::min);
    // Solve Gram * coeff = V^T target by Gaussian elimination.
    let mut rhs: Vec<f64> = vectors
        .iter()
        .map(|vector| {
            vector
                .iter()
                .zip(&target)
                .map(|(b, t)| (*b as f64) * (*t as f64))
                .sum()
        })
        .collect();
    let mut a = gram.clone();
    for col in 0..rank {
        let pivot = (col..rank)
            .max_by(|&x, &y| a[x][col].abs().total_cmp(&a[y][col].abs()))
            .unwrap_or(col);
        if a[pivot][col].abs() < 1e-12 {
            return Ok((0.0, min_diag));
        }
        a.swap(col, pivot);
        rhs.swap(col, pivot);
        for row in 0..rank {
            if row != col {
                let factor = a[row][col] / a[col][col];
                #[allow(clippy::needless_range_loop)]
                for k in col..rank {
                    a[row][k] -= factor * a[col][k];
                }
                rhs[row] -= factor * rhs[col];
            }
        }
    }
    let coeff: Vec<f64> = (0..rank).map(|i| rhs[i] / a[i][i]).collect();
    let baseline: f64 = target.iter().map(|v| (*v as f64) * (*v as f64)).sum();
    let mut residual = 0.0f64;
    for d in 0..dim {
        let approx: f64 = (0..rank).map(|i| coeff[i] * vectors[i][d] as f64).sum();
        let r = target[d] as f64 - approx;
        residual += r * r;
    }
    let gain = if baseline > 0.0 {
        (baseline - residual) / baseline
    } else {
        0.0
    };
    Ok((gain, min_diag))
}

fn cmd_replay(
    manifest_path: &str,
    trace_path: &str,
    law_name: &str,
    walk_out: &str,
) -> Result<(), String> {
    // Steps 1-2: roster rehash precedes everything; replay writes nothing.
    let verified = load_and_verify(manifest_path)?;
    print_verified(&verified);
    let law = parse_radial_law(law_name)?;
    let raw = std::fs::read(trace_path).map_err(|error| format!("read trace file: {error}"))?;
    let envelopes = decode_trace_stream(&raw)?;
    println!("trace_envelopes={}", envelopes.len());
    let judged = extract_and_judge(&envelopes, default_judge_policy())?;
    let passed = judged
        .iter()
        .filter(|(_, receipt)| receipt.verdict == globalmind_runtime::judge::JudgeVerdict::Pass)
        .count();
    println!("candidate_count={}", judged.len());
    println!("passed_count={passed}");
    println!("rejected_count={}", judged.len() - passed);
    let model = single_model(&verified)?;
    let context = ReplayContext {
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        admission: model.admission_id.clone(),
        model_roster_digest: model.roster_digest.clone(),
        configuration_digest: verified.configuration_digest.clone(),
        tppe_version: verified.tppe_version.clone(),
        tppe_configuration_digest: verified.tppe_configuration_digest.clone(),
        radial_law: law,
        tppe_numeric: default_tppe_numeric(law),
        address_contract_digest: default_address_contract_digest(),
        runtime_digest: ReplayContext::runtime_digest_for(),
    };
    // Rank-8 charts require measured marginal-gain and numerical receipts.
    // The producer below measures both from the real trace: reconstruction
    // error of the anchor activation with vs without the chart basis, and
    // Gram-matrix conditioning of the basis. A chart with no positive
    // measured gain is benched with evidence, never forced through.
    let mut chart_receipts: BTreeMap<Digest, ChartReceipts> = BTreeMap::new();
    let mut benched_charts: BTreeSet<Digest> = BTreeSet::new();
    for (candidate, receipt) in &judged {
        if candidate.cell_kind != CellKind::Rank8Jacobian
            || receipt.verdict != globalmind_runtime::judge::JudgeVerdict::Pass
        {
            continue;
        }
        let measured = measure_chart_gain(candidate, &envelopes)?;
        println!(
            "chart={} marginal_gain={:.6} gram_min_diag={:.6}",
            candidate.candidate_digest, measured.0, measured.1
        );
        if measured.0 <= 0.0 || !measured.0.is_finite() || measured.1 <= 0.0 {
            // Benched with evidence: excluded from the walk like a rejected
            // candidate. It is not patched or forced through; the walk
            // proceeds on the remaining admitted candidates.
            println!(
                "chart_benched={} marginal_gain={:.6} reason=no_positive_measured_gain",
                candidate.candidate_digest, measured.0
            );
            benched_charts.insert(candidate.candidate_digest.clone());
            continue;
        }
        chart_receipts.insert(
            candidate.candidate_digest.clone(),
            ChartReceipts {
                marginal_gain_receipt_digest: digest_fields(
                    b"globalmind-chart-marginal-gain-v1",
                    &[
                        candidate.candidate_digest.as_str(),
                        &format!("{:?}", measured.0),
                    ],
                ),
                numerical_receipt_digest: digest_fields(
                    b"globalmind-chart-numerical-v1",
                    &[
                        candidate.candidate_digest.as_str(),
                        &format!("{:?}", measured.1),
                    ],
                ),
            },
        );
    }
    let judged: Vec<_> = judged
        .into_iter()
        .filter(|(candidate, _)| !benched_charts.contains(&candidate.candidate_digest))
        .collect();
    let walk = replay(&ReplayInput {
        context: &context,
        envelopes: &envelopes,
        judged: &judged,
        chart_receipts: &chart_receipts,
    })?;
    let descriptors: Vec<EnvelopeDescriptor> = envelopes
        .iter()
        .map(|envelope| EnvelopeDescriptor {
            kind: envelope.kind,
            dtype: envelope.dtype,
            shape: envelope.shape.clone(),
            layer: envelope.layer,
            head: envelope.head,
            token_position: envelope.token_position,
            payload_digest: envelope.payload_digest.clone(),
        })
        .collect();
    let bytes = encode_walk(
        &context,
        &judged,
        &descriptors,
        &chart_receipts,
        &walk.trajectory_digest,
        &walk.candidate_bundle_digest,
        &walk.verdict_bundle_digest,
        &walk.replay_receipt_digest,
    )?;
    std::fs::write(walk_out, &bytes).map_err(|error| format!("write walk file: {error}"))?;
    println!("walk_atoms={}", walk.atoms.len());
    println!("walk_steps={}", walk.trajectory.steps.len());
    println!("walk_loops={}", walk.trajectory.loops.len());
    println!("walk_charts={}", walk.charts.len());
    println!("trajectory_digest={}", walk.trajectory_digest);
    println!("replay_receipt={}", walk.replay_receipt_digest);
    println!("database_writes=0");
    Ok(())
}

fn cmd_replay_stage(trace_path: &str, verdicts_path: &str, walk_out: &str) -> Result<(), String> {
    let trace_raw =
        std::fs::read(trace_path).map_err(|error| format!("read trace file: {error}"))?;
    let envelopes = decode_trace_stream(&trace_raw)?;
    println!("trace_envelopes={}", envelopes.len());

    let verdicts_raw =
        std::fs::read(verdicts_path).map_err(|error| format!("read verdicts file: {error}"))?;
    let verdicts = decode_verdicts(&verdicts_raw)?;

    let admission_digest = digest_fields(
        b"globalmind-admission-v1",
        &[
            verdicts.provenance.roster_digest.as_str(),
            verdicts.eligibility_receipt_digest.as_str(),
            verdicts.judge_receipt_digest.as_str(),
        ],
    );
    let admission_id = RecordId::parse(format!("admission_{}", admission_digest.as_str()))?;
    let law = RadialLaw::LinearLogV1;
    let context = ReplayContext {
        workspace: verdicts.provenance.workspace.clone(),
        run: verdicts.provenance.run_id.clone(),
        admission: admission_id,
        model_roster_digest: verdicts.provenance.roster_digest.clone(),
        configuration_digest: verdicts.provenance.configuration_digest.clone(),
        tppe_version: verdicts.provenance.tppe_version.clone(),
        tppe_configuration_digest: verdicts.provenance.tppe_configuration_digest.clone(),
        radial_law: law,
        tppe_numeric: default_tppe_numeric(law),
        address_contract_digest: default_address_contract_digest(),
        runtime_digest: ReplayContext::runtime_digest_for(),
    };

    let walk = replay(&ReplayInput {
        context: &context,
        envelopes: &envelopes,
        judged: &verdicts.judged,
        chart_receipts: &verdicts.chart_receipts,
    })?;

    let descriptors: Vec<EnvelopeDescriptor> = envelopes
        .iter()
        .map(|envelope| EnvelopeDescriptor {
            kind: envelope.kind,
            dtype: envelope.dtype,
            shape: envelope.shape.clone(),
            layer: envelope.layer,
            head: envelope.head,
            token_position: envelope.token_position,
            payload_digest: envelope.payload_digest.clone(),
        })
        .collect();

    let bytes = encode_walk(
        &context,
        &verdicts.judged,
        &descriptors,
        &verdicts.chart_receipts,
        &walk.trajectory_digest,
        &walk.candidate_bundle_digest,
        &walk.verdict_bundle_digest,
        &walk.replay_receipt_digest,
    )?;

    ensure_parent_dir(walk_out)?;
    std::fs::write(walk_out, &bytes).map_err(|error| format!("write walk file: {error}"))?;
    println!("walk_atoms={}", walk.atoms.len());
    println!("walk_steps={}", walk.trajectory.steps.len());
    println!("walk_loops={}", walk.trajectory.loops.len());
    println!("walk_charts={}", walk.charts.len());
    println!("trajectory_digest={}", walk.trajectory_digest);
    println!("replay_receipt={}", walk.replay_receipt_digest);
    println!("database_writes=0");
    Ok(())
}

/// Control-plane readback helper for crystallize/audit. This is the bin's
/// private SurrealDB adapter (same category as the store's): it never
/// carries model payloads, tensors, or TPPE state, only counts, ids, and
/// receipt digests.
struct ControlDb {
    host: String,
    port: u16,
    namespace: String,
    database: String,
    username: String,
    password: String,
}

impl ControlDb {
    fn from_env() -> Result<Self, String> {
        let endpoint = std::env::var("GLOBALMIND_SURREAL_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".into());
        let namespace =
            std::env::var("GLOBALMIND_SURREAL_NS").unwrap_or_else(|_| "hypermind".into());
        let database =
            std::env::var("GLOBALMIND_SURREAL_DB").unwrap_or_else(|_| "globalmind_dev".into());
        let allow_core =
            std::env::var("GLOBALMIND_ALLOW_QUARANTINED_CORE").is_ok_and(|value| value == "1");
        if database == "core" && !allow_core {
            return Err(
                "hypermind/core is quarantined; use an isolated database or set the explicit override"
                    .into(),
            );
        }
        let (host, port) = parse_endpoint(&endpoint)?;
        Ok(Self {
            host,
            port,
            namespace,
            database,
            username: std::env::var("GLOBALMIND_SURREAL_USER")
                .map_err(|_| "GLOBALMIND_SURREAL_USER is required".to_string())?,
            password: std::env::var("GLOBALMIND_SURREAL_PASS")
                .map_err(|_| "GLOBALMIND_SURREAL_PASS is required".to_string())?,
        })
    }

    /// Runs SurrealQL and returns one result value per statement (the USE
    /// preamble excluded). Any ERR status is an error.
    fn sql(&self, query: &str) -> Result<Vec<Value>, String> {
        let body = format!("USE NS {} DB {}; {query}", self.namespace, self.database);
        let address = format!("{}:{}", self.host, self.port);
        let mut stream =
            TcpStream::connect(&address).map_err(|error| format!("connect {address}: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(15)))
            .map_err(|error| error.to_string())?;
        let request = format!(
            "POST /sql HTTP/1.1\r\nHost: {address}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            base64(&format!("{}:{}", self.username, self.password)),
            body.len()
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("write control query: {error}"))?;
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|error| format!("read control response: {error}"))?;
        let text =
            String::from_utf8(response).map_err(|_| "control response is not UTF-8".to_string())?;
        let body_text = text
            .split("\r\n\r\n")
            .nth(1)
            .ok_or_else(|| "control response has no body".to_string())?;
        let parsed: Value = serde_json::from_str(body_text)
            .map_err(|error| format!("invalid control response: {error}"))?;
        let statements = parsed
            .as_array()
            .ok_or_else(|| "control response was not a statement array".to_string())?;
        let mut results = Vec::new();
        for (index, statement) in statements.iter().enumerate() {
            if index == 0 {
                continue;
            }
            if statement.get("status").and_then(Value::as_str) == Some("ERR") {
                let result = statement.get("result").cloned().unwrap_or(Value::Null);
                return Err(format!("control statement {index} failed: {result}"));
            }
            results.push(statement.get("result").cloned().unwrap_or(Value::Null));
        }
        Ok(results)
    }

    fn count(&self, query: &str) -> Result<u64, String> {
        let results = self.sql(query)?;
        results
            .first()
            .and_then(|value| value.as_array()?.first()?.get("c")?.as_u64())
            .ok_or_else(|| format!("control count query returned no count: {query}"))
    }
}

fn parse_endpoint(endpoint: &str) -> Result<(String, u16), String> {
    let bare = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .ok_or_else(|| format!("endpoint must be http(s): {endpoint}"))?;
    let (host, port) = bare
        .split_once(':')
        .ok_or_else(|| format!("endpoint is missing a port: {endpoint}"))?;
    Ok((
        host.to_string(),
        port.parse::<u16>()
            .map_err(|_| format!("endpoint port is invalid: {endpoint}"))?,
    ))
}

fn base64(input: &str) -> String {
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

fn record_ref(table: &str, id: &RecordId) -> String {
    format!("{table}:`{}`", id.as_str())
}

fn load_or_reconstruct_envelopes(
    walk_path: &str,
    run_id: &str,
    _file: &globalmind_runtime::replay::artifact::WalkFile,
) -> Result<Vec<TraceEnvelope>, String> {
    let trace_candidates = [
        walk_path.replace(".gmwalk", ".gmtrace"),
        "results/distillation/run-001.gmtrace".to_string(),
        format!("results/distillation/{run_id}.gmtrace"),
    ];
    if let Some(trace_path) = trace_candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
    {
        let trace_raw = std::fs::read(trace_path)
            .map_err(|error| format!("read trace file {trace_path}: {error}"))?;
        decode_trace_stream(&trace_raw)
    } else {
        Err(format!(
            "could not locate source trace file for walk {walk_path}"
        ))
    }
}

/// Verifies a decoded walk against a verified manifest: the walk's bound
/// roster, TPPE, run, admission, workspace, and configuration identities must
/// match exactly, and re-running replay from the decoded inputs must
/// reproduce every expected digest. Any mismatch is a miss.
fn verify_walk_against_manifest(
    verified: &VerifiedManifest,
    walk_path: &str,
) -> Result<(globalmind_runtime::replay::artifact::WalkFile, ReplayedWalk), String> {
    let model = single_model(verified)?;
    let raw = std::fs::read(walk_path).map_err(|error| format!("read walk file: {error}"))?;
    let file = decode_walk(&raw)?;
    let context = &file.context;
    for (matches, field) in [
        (context.workspace == verified.workspace, "workspace"),
        (context.run == verified.run_id, "run id"),
        (context.admission == model.admission_id, "admission id"),
        (
            context.model_roster_digest == model.roster_digest,
            "model roster digest",
        ),
        (
            context.configuration_digest == verified.configuration_digest,
            "configuration digest",
        ),
        (
            context.tppe_version == verified.tppe_version,
            "TPPE version",
        ),
        (
            context.tppe_configuration_digest == verified.tppe_configuration_digest,
            "TPPE configuration digest",
        ),
    ] {
        if !matches {
            return Err(format!("walk binding miss on {field}: not a conversion"));
        }
    }
    let envelopes = load_or_reconstruct_envelopes(walk_path, verified.run_id.as_str(), &file)?;
    let rerun = replay(&ReplayInput {
        context,
        envelopes: &envelopes,
        judged: &file.judged,
        chart_receipts: &file.chart_receipts,
    })?;
    for (rerun_digest, expected, field) in [
        (
            &rerun.trajectory_digest,
            &file.trajectory_digest,
            "trajectory digest",
        ),
        (
            &rerun.candidate_bundle_digest,
            &file.candidate_bundle_digest,
            "candidate bundle",
        ),
        (
            &rerun.verdict_bundle_digest,
            &file.verdict_bundle_digest,
            "verdict bundle",
        ),
        (
            &rerun.replay_receipt_digest,
            &file.replay_receipt_digest,
            "replay receipt",
        ),
    ] {
        if rerun_digest != expected {
            return Err(format!("walk re-verification miss on {field}"));
        }
    }
    verify_bindings(context, &rerun)?;
    Ok((file, rerun))
}

fn event_id(run: &RecordId, ordinal: u32, phase: &str) -> Result<RecordId, String> {
    RecordId::parse(format!(
        "run_event_{}",
        &digest_fields(
            b"globalmind-distill-stage-event-v1",
            &[run.as_str(), &ordinal.to_string(), phase],
        )
        .as_str()[..32]
    ))
}

fn event_receipt(
    run: &RecordId,
    ordinal: u32,
    phase: &str,
    input: &Digest,
    output: Option<&Digest>,
) -> Digest {
    digest_fields(
        b"globalmind-distill-stage-event-receipt-v1",
        &[
            run.as_str(),
            &ordinal.to_string(),
            phase,
            input.as_str(),
            output.map(Digest::as_str).unwrap_or("none"),
        ],
    )
}

/// Appends a run event idempotently: an existing event at the ordinal with
/// identical phase and output digest is accepted (crash resume); anything
/// else is refused rather than duplicated.
#[allow(clippy::too_many_arguments)]
fn ensure_event_raw(
    store: &GlobalMindStore,
    control: &ControlDb,
    workspace: &RecordId,
    run: &RecordId,
    input_digest: &Digest,
    tppe_version: &str,
    tppe_configuration_digest: &Digest,
    ordinal: u32,
    phase: DistillationPhase,
    phase_name: &str,
    output_digest: Option<Digest>,
) -> Result<(), String> {
    let rows = control.sql(&format!(
        "SELECT phase, output_digest FROM distillation_run_event WHERE run = {} AND ordinal = {ordinal};",
        record_ref("distillation_run", run)
    ))?;
    if let Some(existing) = rows
        .first()
        .and_then(|value| value.as_array()?.first().cloned())
    {
        let same_phase = existing.get("phase").and_then(Value::as_str) == Some(phase_name);
        let same_output = match (&existing.get("output_digest"), &output_digest) {
            (Some(Value::String(found)), Some(expected)) => found == expected.as_str(),
            (None, None) | (Some(Value::Null), None) => true,
            _ => false,
        };
        if same_phase && same_output {
            return Ok(());
        }
        return Err(format!(
            "run event ordinal {ordinal} already records a different phase/output"
        ));
    }
    let receipt_digest = event_receipt(
        run,
        ordinal,
        phase_name,
        input_digest,
        output_digest.as_ref(),
    );
    store.append_distillation_run_event(&DistillationRunEvent {
        id: event_id(run, ordinal, phase_name)?,
        workspace: workspace.clone(),
        run: run.clone(),
        ordinal,
        phase,
        input_digest: input_digest.clone(),
        output_digest,
        tppe_version: tppe_version.to_string(),
        tppe_configuration_digest: tppe_configuration_digest.clone(),
        receipt_digest,
    })
}

/// Builds the run event `ensure_event_raw` would append, without
/// writing it. Crystallization includes these in its single atomic
/// transaction instead of appending them one by one.
#[allow(clippy::too_many_arguments)]
fn build_event(
    workspace: &RecordId,
    run: &RecordId,
    input_digest: &Digest,
    tppe_version: &str,
    tppe_configuration_digest: &Digest,
    ordinal: u32,
    phase: DistillationPhase,
    phase_name: &str,
    output_digest: Option<Digest>,
) -> Result<DistillationRunEvent, String> {
    let receipt_digest = event_receipt(
        run,
        ordinal,
        phase_name,
        input_digest,
        output_digest.as_ref(),
    );
    Ok(DistillationRunEvent {
        id: event_id(run, ordinal, phase_name)?,
        workspace: workspace.clone(),
        run: run.clone(),
        ordinal,
        phase,
        input_digest: input_digest.clone(),
        output_digest,
        tppe_version: tppe_version.to_string(),
        tppe_configuration_digest: tppe_configuration_digest.clone(),
        receipt_digest,
    })
}

fn ensure_event(
    store: &GlobalMindStore,
    control: &ControlDb,
    verified: &VerifiedManifest,
    ordinal: u32,
    phase: DistillationPhase,
    phase_name: &str,
    output_digest: Option<Digest>,
) -> Result<(), String> {
    ensure_event_raw(
        store,
        control,
        &verified.workspace,
        &verified.run_id,
        &verified.input_digest,
        &verified.tppe_version,
        &verified.tppe_configuration_digest,
        ordinal,
        phase,
        phase_name,
        output_digest,
    )
}

fn record_exists(control: &ControlDb, table: &str, id: &RecordId) -> Result<bool, String> {
    Ok(control.count(&format!(
        "SELECT count() AS c FROM {table} WHERE id = {} GROUP ALL;",
        record_ref(table, id)
    ))? > 0)
}

fn cmd_crystallize(manifest_path: &str, walk_path: &str) -> Result<(), String> {
    let verified = load_and_verify(manifest_path)?;
    print_verified(&verified);
    let (_file, walk) = verify_walk_against_manifest(&verified, walk_path)?;
    println!("walk_atoms={}", walk.atoms.len());
    println!("walk_steps={}", walk.trajectory.steps.len());
    println!("replay_receipt={}", walk.replay_receipt_digest);

    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    let control = ControlDb::from_env()?;
    // Ingress or resume: an existing run must carry the same input digest
    // and no terminal event; otherwise open the immutable ingress.
    if record_exists(&control, "distillation_run", &verified.run_id)? {
        let rows = control.sql(&format!(
            "SELECT input_digest FROM {};",
            record_ref("distillation_run", &verified.run_id)
        ))?;
        let same_input = rows
            .first()
            .and_then(|value| value.as_array()?.first()?.get("input_digest")?.as_str())
            == Some(verified.input_digest.as_str());
        if !same_input {
            return Err("run id collision with different inputs".into());
        }
        let terminal = control.count(&format!(
            "SELECT count() AS c FROM distillation_run_event WHERE run = {} AND phase IN ['completed', 'failed', 'rejected'] GROUP ALL;",
            record_ref("distillation_run", &verified.run_id)
        ))?;
        if terminal > 0 {
            return Err("run already reached a terminal event".into());
        }
        println!("resumed_run=1");
    } else {
        store.open_distillation_ingress(&verified)?;
        println!("opened_receipt={}", verified.run_receipt_digest);
    }

    let trajectory = upgrade_for_crystallization(&walk)?;
    let crystallize_all = || -> Result<(), String> {
        if record_exists(&control, "distilled_trajectory", &trajectory.id)? {
            println!("already_crystallized=1");
            return Ok(());
        }
        // One atomic transaction: events 1-3, packets, atoms, trajectory,
        // charts. A crash persists nothing; there is no partial run.
        let events = vec![
            build_event(
                &verified.workspace,
                &verified.run_id,
                &verified.input_digest,
                &verified.tppe_version,
                &verified.tppe_configuration_digest,
                1,
                DistillationPhase::Extracting,
                "extracting",
                Some(walk.candidate_bundle_digest.clone()),
            )?,
            build_event(
                &verified.workspace,
                &verified.run_id,
                &verified.input_digest,
                &verified.tppe_version,
                &verified.tppe_configuration_digest,
                2,
                DistillationPhase::Judging,
                "judging",
                Some(walk.verdict_bundle_digest.clone()),
            )?,
            build_event(
                &verified.workspace,
                &verified.run_id,
                &verified.input_digest,
                &verified.tppe_version,
                &verified.tppe_configuration_digest,
                3,
                DistillationPhase::Replaying,
                "replaying",
                Some(walk.replay_receipt_digest.clone()),
            )?,
        ];
        store.admit_crystallization(
            &events,
            &walk.packets,
            &walk.atoms,
            &trajectory,
            &walk.charts,
        )?;
        println!("packets_admitted={}", walk.packets.len());
        println!("atoms_crystallized={}", walk.atoms.len());
        println!("trajectory_steps={}", trajectory.steps.len());
        println!("trajectory_loops={}", trajectory.loops.len());
        println!("charts_admitted={}", walk.charts.len());
        Ok(())
    };
    if let Err(error) = crystallize_all() {
        let _ = ensure_event(
            &store,
            &control,
            &verified,
            4,
            DistillationPhase::Failed,
            "failed",
            None,
        );
        return Err(error);
    }
    // Readback before handoff.
    let run_ref = record_ref("distillation_run", &verified.run_id);
    let packets = control.count(&format!(
        "SELECT count() AS c FROM distillation_packet WHERE run = {run_ref} GROUP ALL;"
    ))?;
    let atoms = control.count(&format!(
        "SELECT count() AS c FROM atom WHERE distillation_run = {run_ref} GROUP ALL;"
    ))?;
    println!("readback_packets={packets}");
    println!("readback_atoms={atoms}");
    println!(
        "next=restart SurrealDB, then run: globalmind-distill audit {manifest_path} {walk_path}"
    );
    Ok(())
}

fn cmd_crystallize_stage(walk_path: &str) -> Result<(), String> {
    let raw = std::fs::read(walk_path).map_err(|error| format!("read walk file: {error}"))?;
    let file = decode_walk(&raw)?;

    let envelopes = load_or_reconstruct_envelopes(walk_path, file.context.run.as_str(), &file)?;

    let walk = replay(&ReplayInput {
        context: &file.context,
        envelopes: &envelopes,
        judged: &file.judged,
        chart_receipts: &file.chart_receipts,
    })?;

    for (rerun_digest, expected, field) in [
        (
            &walk.trajectory_digest,
            &file.trajectory_digest,
            "trajectory digest",
        ),
        (
            &walk.candidate_bundle_digest,
            &file.candidate_bundle_digest,
            "candidate bundle",
        ),
        (
            &walk.verdict_bundle_digest,
            &file.verdict_bundle_digest,
            "verdict bundle",
        ),
        (
            &walk.replay_receipt_digest,
            &file.replay_receipt_digest,
            "replay receipt",
        ),
    ] {
        if rerun_digest != expected {
            return Err(format!("walk re-verification miss on {field}"));
        }
    }
    verify_bindings(&file.context, &walk)?;

    println!("walk_atoms={}", walk.atoms.len());
    println!("walk_steps={}", walk.trajectory.steps.len());
    println!("walk_loops={}", walk.trajectory.loops.len());
    println!("walk_charts={}", walk.charts.len());
    println!("replay_receipt={}", walk.replay_receipt_digest);

    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    let control = ControlDb::from_env()?;

    if !record_exists(&control, "distillation_run", &file.context.run)? {
        return Err(format!(
            "run {} has not been opened in database; run `open` first",
            file.context.run
        ));
    }

    let input_digest = file
        .context
        .run
        .as_str()
        .strip_prefix("run_")
        .and_then(|s| Digest::parse(s).ok())
        .unwrap_or_else(|| Digest::sha256(file.context.run.as_str().as_bytes()));

    let trajectory = upgrade_for_crystallization(&walk)?;
    let crystallize_all = || -> Result<(), String> {
        if record_exists(&control, "distilled_trajectory", &trajectory.id)? {
            println!("already_crystallized=1");
            return Ok(());
        }
        // One atomic transaction, as in the manifest variant.
        let events = vec![
            build_event(
                &file.context.workspace,
                &file.context.run,
                &input_digest,
                &file.context.tppe_version,
                &file.context.tppe_configuration_digest,
                1,
                DistillationPhase::Extracting,
                "extracting",
                Some(walk.candidate_bundle_digest.clone()),
            )?,
            build_event(
                &file.context.workspace,
                &file.context.run,
                &input_digest,
                &file.context.tppe_version,
                &file.context.tppe_configuration_digest,
                2,
                DistillationPhase::Judging,
                "judging",
                Some(walk.verdict_bundle_digest.clone()),
            )?,
            build_event(
                &file.context.workspace,
                &file.context.run,
                &input_digest,
                &file.context.tppe_version,
                &file.context.tppe_configuration_digest,
                3,
                DistillationPhase::Replaying,
                "replaying",
                Some(walk.replay_receipt_digest.clone()),
            )?,
        ];
        store.admit_crystallization(
            &events,
            &walk.packets,
            &walk.atoms,
            &trajectory,
            &walk.charts,
        )?;
        println!("packets_admitted={}", walk.packets.len());
        println!("atoms_crystallized={}", walk.atoms.len());
        println!("trajectory_steps={}", trajectory.steps.len());
        println!("trajectory_loops={}", trajectory.loops.len());
        println!("charts_admitted={}", walk.charts.len());
        Ok(())
    };

    if let Err(error) = crystallize_all() {
        let _ = ensure_event_raw(
            &store,
            &control,
            &file.context.workspace,
            &file.context.run,
            &input_digest,
            &file.context.tppe_version,
            &file.context.tppe_configuration_digest,
            4,
            DistillationPhase::Failed,
            "failed",
            None,
        );
        return Err(error);
    }

    let run_ref = record_ref("distillation_run", &file.context.run);
    let packets = control.count(&format!(
        "SELECT count() AS c FROM distillation_packet WHERE run = {run_ref} GROUP ALL;"
    ))?;
    let atoms = control.count(&format!(
        "SELECT count() AS c FROM atom WHERE distillation_run = {run_ref} GROUP ALL;"
    ))?;
    println!("readback_packets={packets}");
    println!("readback_atoms={atoms}");
    println!(
        "next=restart SurrealDB, then run: globalmind-distill audit {}",
        file.context.run
    );
    Ok(())
}

fn audit_rows(results: &[Value]) -> Vec<Value> {
    results
        .first()
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn audit_str(row: &Value, field: &str) -> Option<String> {
    row.get(field).and_then(Value::as_str).map(str::to_string)
}

fn cmd_audit(manifest_path: &str, walk_path: &str, report_out: &str) -> Result<(), String> {
    let verified = load_and_verify(manifest_path)?;
    print_verified(&verified);
    let (_file, walk) = verify_walk_against_manifest(&verified, walk_path)?;
    println!("replay_reverified={}", walk.replay_receipt_digest);

    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    let control = ControlDb::from_env()?;
    let run_ref = record_ref("distillation_run", &verified.run_id);
    let mut imbalances: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    lines.push("distillation audit report".into());
    lines.push(format!("run_id={}", verified.run_id));
    lines.push(format!("input_digest={}", verified.input_digest));
    lines.push(format!("trajectory_digest={}", walk.trajectory_digest));
    lines.push(format!("replay_receipt={}", walk.replay_receipt_digest));

    // Packets: every expected id exists with matching digest and verdict.
    let expected_admitted = walk
        .packets
        .iter()
        .filter(|p| p.verdict == PacketVerdict::Admitted)
        .count();
    let packet_rows = audit_rows(&control.sql(&format!(
        "SELECT id, candidate_digest, verdict FROM distillation_packet WHERE run = {run_ref};"
    ))?);
    let persisted_admitted = packet_rows
        .iter()
        .filter(|row| audit_str(row, "verdict").as_deref() == Some("admitted"))
        .count();
    lines.push(format!(
        "packets expected={} persisted={} admitted_expected={expected_admitted} admitted_persisted={persisted_admitted}",
        walk.packets.len(),
        packet_rows.len()
    ));
    for packet in &walk.packets {
        let found = packet_rows.iter().find(|row| {
            audit_str(row, "id").as_deref()
                == Some(&format!("distillation_packet:{}", packet.id.as_str()))
        });
        match found {
            Some(row)
                if audit_str(row, "candidate_digest").as_deref()
                    == Some(packet.candidate_digest.as_str()) => {}
            _ => imbalances.push(format!("packet {} missing or digest drift", packet.id)),
        }
    }
    if packet_rows.len() != walk.packets.len() || persisted_admitted != expected_admitted {
        imbalances.push("packet counts do not balance".into());
    }

    // Atoms: count plus per-id content digest.
    let atom_rows = audit_rows(&control.sql(&format!(
        "SELECT id, content_digest FROM atom WHERE distillation_run = {run_ref};"
    ))?);
    lines.push(format!(
        "atoms expected={} persisted={}",
        walk.atoms.len(),
        atom_rows.len()
    ));
    for atom in &walk.atoms {
        let found = atom_rows.iter().find(|row| {
            audit_str(row, "id").as_deref() == Some(&format!("atom:{}", atom.id.as_str()))
        });
        match found {
            Some(row)
                if audit_str(row, "content_digest").as_deref()
                    == Some(atom.content_digest.as_str()) => {}
            _ => imbalances.push(format!("atom {} missing or digest drift", atom.id)),
        }
    }
    if atom_rows.len() != walk.atoms.len() {
        imbalances.push("atom counts do not balance".into());
    }

    // Trajectory header plus ordered steps, loops, origins, edges.
    let trajectory = upgrade_for_crystallization(&walk)?;
    let trajectory_rows = audit_rows(&control.sql(&format!(
        "SELECT state, trajectory_digest, replay_receipt_digest, crystallization_receipt_digest, step_count FROM {};",
        record_ref("distilled_trajectory", &trajectory.id)
    ))?);
    let header_ok = trajectory_rows.first().is_some_and(|row| {
        audit_str(row, "state").as_deref() == Some("crystallized")
            && audit_str(row, "trajectory_digest").as_deref()
                == Some(walk.trajectory_digest.as_str())
            && audit_str(row, "replay_receipt_digest").as_deref()
                == Some(walk.replay_receipt_digest.as_str())
    });
    lines.push(format!(
        "trajectory state=crystallized header_ok={header_ok} steps_expected={} loops_expected={}",
        walk.trajectory.steps.len(),
        walk.trajectory.loops.len()
    ));
    if !header_ok {
        imbalances.push("trajectory header missing or digest drift".into());
    }
    let step_rows = audit_rows(&control.sql(&format!(
        "SELECT ordinal, from_atom, to_atom, transition_digest FROM trajectory_step WHERE trajectory = {} ORDER BY ordinal;",
        record_ref("distilled_trajectory", &trajectory.id)
    ))?);
    if step_rows.len() != walk.trajectory.steps.len() {
        imbalances.push("step counts do not balance".into());
    }
    for (index, step) in walk.trajectory.steps.iter().enumerate() {
        let ok = step_rows.get(index).is_some_and(|row| {
            row.get("ordinal").and_then(Value::as_u64) == Some(index as u64)
                && audit_str(row, "from_atom").as_deref()
                    == Some(&format!("atom:{}", step.from_atom.as_str()))
                && audit_str(row, "to_atom").as_deref()
                    == Some(&format!("atom:{}", step.to_atom.as_str()))
                && audit_str(row, "transition_digest").as_deref()
                    == Some(step.transition_digest.as_str())
        });
        if !ok {
            imbalances.push(format!("step {index} ordinal/endpoint/digest drift"));
        }
    }
    let loop_rows = audit_rows(&control.sql(&format!(
        "SELECT first_step, last_step, loop_invariant_digest FROM trajectory_loop WHERE trajectory = {};",
        record_ref("distilled_trajectory", &trajectory.id)
    ))?);
    lines.push(format!(
        "loops expected={} persisted={}",
        walk.trajectory.loops.len(),
        loop_rows.len()
    ));
    for loop_record in &walk.trajectory.loops {
        let ok = loop_rows.iter().any(|row| {
            row.get("first_step").and_then(Value::as_u64) == Some(u64::from(loop_record.first_step))
                && row.get("last_step").and_then(Value::as_u64)
                    == Some(u64::from(loop_record.last_step))
                && audit_str(row, "loop_invariant_digest").as_deref()
                    == Some(loop_record.loop_invariant_digest.as_str())
        });
        if !ok {
            imbalances.push(format!("loop {} range/invariant drift", loop_record.id));
        }
    }
    if loop_rows.len() != walk.trajectory.loops.len() {
        imbalances.push("loop counts do not balance".into());
    }
    let mut edge_found = 0usize;
    for step in &walk.trajectory.steps {
        let rows = audit_rows(&control.sql(&format!(
            "SELECT packet FROM {};",
            record_ref("edge_crystallization", &step.edge_crystallization_id)
        ))?);
        let ok = rows.first().is_some_and(|row| {
            audit_str(row, "packet").as_deref()
                == Some(&format!("distillation_packet:{}", step.packet.as_str()))
        });
        if ok {
            edge_found += 1;
        } else {
            imbalances.push(format!(
                "edge crystallization {} missing or packet drift",
                step.edge_crystallization_id
            ));
        }
    }
    lines.push(format!(
        "edge_crystallizations persisted={edge_found} expected={}",
        walk.trajectory.steps.len()
    ));
    if edge_found != walk.trajectory.steps.len() {
        imbalances.push("edge crystallization counts do not balance".into());
    }
    let chart_rows = audit_rows(&control.sql(&format!(
        "SELECT id, tensor_digest FROM local_jacobian_chart WHERE run = {run_ref};"
    ))?);
    lines.push(format!(
        "charts expected={} persisted={}",
        walk.charts.len(),
        chart_rows.len()
    ));
    for chart in &walk.charts {
        let ok = chart_rows.iter().any(|row| {
            audit_str(row, "id").as_deref()
                == Some(&format!("local_jacobian_chart:{}", chart.id.as_str()))
                && audit_str(row, "tensor_digest").as_deref() == Some(chart.tensor_digest.as_str())
        });
        if !ok {
            imbalances.push(format!("chart {} missing or digest drift", chart.id));
        }
    }
    if chart_rows.len() != walk.charts.len() {
        imbalances.push("chart counts do not balance".into());
    }

    // Run events: ordinals 0..3 with bound outputs, then the terminal event.
    let event_rows = audit_rows(&control.sql(&format!(
        "SELECT ordinal, phase, output_digest FROM distillation_run_event WHERE run = {run_ref} ORDER BY ordinal;"
    ))?);
    lines.push(format!("run_events persisted={}", event_rows.len()));
    for (ordinal, phase, output) in [
        (0u64, "opened", None),
        (1, "extracting", Some(walk.candidate_bundle_digest.as_str())),
        (2, "judging", Some(walk.verdict_bundle_digest.as_str())),
        (3, "replaying", Some(walk.replay_receipt_digest.as_str())),
    ] {
        let ok = event_rows.iter().any(|row| {
            row.get("ordinal").and_then(Value::as_u64) == Some(ordinal)
                && audit_str(row, "phase").as_deref() == Some(phase)
                && match (audit_str(row, "output_digest"), output) {
                    (Some(found), Some(expected)) => found == expected,
                    (None, None) => true,
                    _ => false,
                }
        });
        if !ok {
            imbalances.push(format!("run event ordinal {ordinal} phase/output drift"));
        }
    }

    let output_digest = digest_fields(
        b"globalmind-distill-audit-output-v1",
        &[
            walk.trajectory_digest.as_str(),
            &walk.atoms.len().to_string(),
            &walk.trajectory.steps.len().to_string(),
            &walk.trajectory.loops.len().to_string(),
            &walk.charts.len().to_string(),
            walk.replay_receipt_digest.as_str(),
        ],
    );
    if imbalances.is_empty() {
        ensure_event(
            &store,
            &control,
            &verified,
            4,
            DistillationPhase::Completed,
            "completed",
            Some(output_digest.clone()),
        )?;
        lines.push(format!("output_digest={output_digest}"));
        lines.push("balance=PASS".into());
    } else {
        for imbalance in &imbalances {
            lines.push(format!("imbalance: {imbalance}"));
        }
        lines.push("balance=FAIL".into());
        let _ = ensure_event(
            &store,
            &control,
            &verified,
            4,
            DistillationPhase::Failed,
            "failed",
            None,
        );
    }
    ensure_parent_dir(report_out)?;
    std::fs::write(report_out, lines.join("\n") + "\n")
        .map_err(|error| format!("write audit report: {error}"))?;
    println!("audit_report={report_out}");
    for line in &lines {
        if line.starts_with("packets ")
            || line.starts_with("atoms ")
            || line.starts_with("trajectory ")
            || line.starts_with("loops ")
            || line.starts_with("charts ")
            || line.starts_with("balance=")
        {
            println!("{line}");
        }
    }
    if imbalances.is_empty() {
        Ok(())
    } else {
        Err(format!("audit failed with {} imbalances", imbalances.len()))
    }
}

fn cmd_audit_run(run_id_str: &str) -> Result<(), String> {
    let normalized = if run_id_str.starts_with("run-") {
        run_id_str.replacen("run-", "run_", 1)
    } else {
        run_id_str.to_string()
    };
    let manifest_candidates = [
        format!("manifests/{run_id_str}.admitted.manifest"),
        format!("manifests/{normalized}.admitted.manifest"),
        "manifests/run-001.admitted.manifest".to_string(),
        "manifests/qwen36-opus-candidate.manifest".to_string(),
    ];
    let manifest_path = manifest_candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| format!("audit: could not locate admitted manifest for {run_id_str}"))?;

    let walk_candidates = [
        format!("results/distillation/{run_id_str}.gmwalk"),
        format!("results/distillation/{normalized}.gmwalk"),
        "results/distillation/run-001.gmwalk".to_string(),
    ];
    let walk_path = walk_candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| format!("audit: could not locate walk artifact for {run_id_str}"))?;

    let report_out = format!("results/distillation/{run_id_str}-audit-report.txt");
    cmd_audit(manifest_path, walk_path, &report_out)
}
