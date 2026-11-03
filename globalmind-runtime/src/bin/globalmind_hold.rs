//! Lane C qualification runner: HoldingController <-> GlobalMindStore.
//!
//! Exercises every required gate against an isolated SurrealDB and prints a
//! plain receipt. It never targets the quarantined database (the store
//! refuses it) and never writes a collapsed bridge directly to memory: the
//! bridge is routed through packet admission, judge, replay, and
//! crystallization exactly like any other candidate (gate 7).

use globalmind_runtime::{
    BranchSource, BranchUpdate, CONTROLLER_VERSION, CellKind, ContentKind, CrystallizedAtom,
    Digest, DistillationPacket, DistillationPhase, DistillationRunEvent, ForwardMode,
    GlobalMindStore, HoldingPolicy, HoldingSession, Manifest, PacketVerdict,
    PredictionErrorObservation, RecallSignals, RecordId, Resolution, SessionOutcome, SourceKind,
    StoreConfig, SuperpositionHold, SuperpositionTrigger, Unit, branch, classify_trigger,
    controller_digest,
};
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    if let Err(error) = run() {
        eprintln!("globalmind-hold: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: globalmind-hold <manifest>".into());
    }
    // The manifest roster is rehashed before any database access, exactly as
    // in the distillation runner; the run it opens carries the bridge packet.
    let manifest = Manifest::load(&args[0])?;
    let verified = manifest.verify_artifacts()?;
    let model = verified
        .models
        .first()
        .ok_or_else(|| "manifest has no verified model".to_string())?;
    println!("verified_model_count={}", verified.models.len());
    println!("input_digest={}", verified.input_digest);

    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    store.verify_contract()?;
    println!("contract_verified=1");

    let workspace = verified.workspace.clone();
    let hub = RecordId::parse("hub")?;
    let well = RecordId::parse("well0")?;
    let policy = HoldingPolicy {
        readiness_floor: 0.5,
        suppression_delta: 0.5,
    };

    // Gate 1: every trigger class fires on its signal; clean signals do not.
    gate1_triggers()?;

    // Decision + trigger observation anchor the hold in the audit chain.
    let request_digest = digest_fields(
        b"globalmind-hold-request-v1",
        &[verified.input_digest.as_str()],
    );
    let decision_id = RecordId::parse(format!("decision_{}", &request_digest.as_str()[..32]))?;
    store.create_active_inference_decision(
        &decision_id,
        &workspace,
        &request_digest,
        &digest_fields(b"globalmind-hold-objective-v1", &[request_digest.as_str()]),
        std::slice::from_ref(&well),
        1,
        ForwardMode::AdaptiveForward,
        CONTROLLER_VERSION,
        &controller_digest(),
        &digest_fields(b"globalmind-hold-quality-v1", &[request_digest.as_str()]),
        &digest_fields(b"globalmind-hold-resource-v1", &[request_digest.as_str()]),
    )?;
    let observation_id =
        RecordId::parse(format!("observation_{}", &request_digest.as_str()[..32]))?;
    store.record_prediction_error(&PredictionErrorObservation {
        id: observation_id.clone(),
        workspace: workspace.clone(),
        decision: decision_id.clone(),
        hub: hub.clone(),
        well: Some(well.clone()),
        trajectory: None,
        prior_digest: digest_fields(b"globalmind-hold-prior-v1", &[request_digest.as_str()]),
        observation_digest: digest_fields(
            b"globalmind-hold-observation-v1",
            &[request_digest.as_str()],
        ),
        estimator_version: CONTROLLER_VERSION.to_string(),
        estimator_digest: controller_digest(),
        prediction_error: 0.9,
        precision: 1.0,
        precision_weighted_error: 0.9,
        surprisal: 0.9,
        unit: Unit::Nats,
        bayesian_surprise: None,
        resolution: Resolution::TrajectoryExtend,
        receipt_digest: digest_fields(b"globalmind-hold-surprise-v1", &[request_digest.as_str()]),
    })?;
    println!("decision_and_observation_persisted=1");

    // Seed the two atoms the branches reason between.
    seed_atoms(&store, &verified)?;

    // Main episode: ambiguity trigger, one iterate, then a gated collapse.
    let main_hold_id = RecordId::parse(format!("hold_{}", &request_digest.as_str()[..32]))?;
    let branches = vec![
        branch(
            &main_hold_id,
            0,
            BranchSource::OperatorBridge,
            "holdatom_a",
            "holdatom_b",
            1.0,
            0.4,
            0.6,
            true,
        )?,
        branch(
            &main_hold_id,
            1,
            BranchSource::PhaseNeighbor,
            "holdatom_a",
            "holdatom_b",
            1.2,
            0.5,
            0.55,
            true,
        )?,
        branch(
            &main_hold_id,
            2,
            BranchSource::FreshDerivation,
            "holdatom_a",
            "holdatom_b",
            3.0,
            0.9,
            0.3,
            true,
        )?,
    ];
    let hold = SuperpositionHold {
        id: main_hold_id,
        workspace: workspace.clone(),
        decision: decision_id.clone(),
        trigger_observation: Some(observation_id.clone()),
        hub: hub.clone(),
        well: Some(well.clone()),
        request_digest: request_digest.clone(),
        prefix_digest: digest_fields(b"globalmind-hold-prefix-v1", &[request_digest.as_str()]),
        trigger: SuperpositionTrigger::BranchAmbiguity,
        controller_version: CONTROLLER_VERSION.to_string(),
        controller_digest: controller_digest(),
        iteration_limit: 4,
        branches: branches.clone(),
    };
    let mut session = HoldingSession::open(&store, hold, policy)?;
    println!("hold_opened={}", session.hold_id());

    // Iteration 0: best branch is not prediction-ready, so no collapse.
    let outcome = session.advance(vec![
        update(&branches[0], 1.0, 0.6, false, true, true),
        update(&branches[1], 1.2, 0.55, false, true, true),
        update(&branches[2], 3.0, 0.3, false, true, true),
    ])?;
    match outcome {
        SessionOutcome::Iterate { surviving, .. } => {
            println!("iteration_0=iterate surviving={}", surviving.len());
            if surviving.len() != 2 {
                return Err("iteration 0 must suppress exactly the high-energy branch".into());
            }
        }
        _ => return Err("iteration 0 must iterate, not terminate".into()),
    }

    // Gate 3 negative: a missing update for an active branch is rejected and
    // persists nothing (the next successful iteration keeps ordinal 1).
    if session
        .advance(vec![update(&branches[0], 0.9, 0.7, true, true, true)])
        .is_ok()
    {
        return Err("an iteration missing an active-branch update must be rejected".into());
    }
    println!("gate3_missing_update_rejected=1");

    // Iteration 1: the leading branch now satisfies the full collapse gate.
    let outcome = session.advance(vec![
        update(&branches[0], 0.9, 0.8, true, true, true),
        update(&branches[1], 1.1, 0.6, true, true, true),
    ])?;
    let bridge = match outcome {
        SessionOutcome::Collapsed { branch, bridge } => {
            if branch != branches[0].id {
                return Err("collapse must select the lowest-energy ready branch".into());
            }
            println!("collapsed_branch={branch}");
            bridge
        }
        _ => return Err("iteration 1 must collapse".into()),
    };
    println!("bridge_digest={}", bridge.bridge_digest);
    println!("bridge_required_path={}", bridge.required_path);

    // Gate 5 negatives: each missing conjunct blocks collapse (pure
    // controller, no persistence).
    gate5_negatives(&branches, policy)?;
    // Gate 4: permuting branch sources cannot change suppression survivors.
    gate4_source_invariance(&branches, policy)?;

    // Fail-closed episode: no compliant branch.
    let closed_hold_id = RecordId::parse(format!("holdclosed_{}", &request_digest.as_str()[..32]))?;
    let closed_branches = vec![
        branch(
            &closed_hold_id,
            0,
            BranchSource::DirectTrajectory,
            "holdatom_a",
            "holdatom_b",
            1.0,
            0.4,
            0.6,
            false,
        )?,
        branch(
            &closed_hold_id,
            1,
            BranchSource::CrossWellBridge,
            "holdatom_a",
            "holdatom_b",
            1.1,
            0.5,
            0.5,
            false,
        )?,
    ];
    let closed_hold = SuperpositionHold {
        id: closed_hold_id,
        workspace: workspace.clone(),
        decision: decision_id.clone(),
        trigger_observation: Some(observation_id.clone()),
        hub: hub.clone(),
        well: Some(well.clone()),
        request_digest: digest_fields(
            b"globalmind-hold-request-closed-v1",
            &[request_digest.as_str()],
        ),
        prefix_digest: digest_fields(
            b"globalmind-hold-prefix-closed-v1",
            &[request_digest.as_str()],
        ),
        trigger: SuperpositionTrigger::ComplianceBoundary,
        controller_version: CONTROLLER_VERSION.to_string(),
        controller_digest: controller_digest(),
        iteration_limit: 2,
        branches: closed_branches.clone(),
    };
    let mut closed_session = HoldingSession::open(&store, closed_hold, policy)?;
    let outcome = closed_session.advance(vec![
        update(&closed_branches[0], 1.0, 0.6, true, true, true),
        update(&closed_branches[1], 1.1, 0.5, true, true, true),
    ])?;
    match outcome {
        SessionOutcome::FailClosed { reason } => println!("fail_closed_reason={reason}"),
        _ => return Err("a hold with no compliant branch must fail closed".into()),
    }

    // Gate 7: the collapsed bridge enters the normal distillation path. The
    // session itself wrote no atom for it; admission, judging, replay, and
    // crystallization happen here through the same store calls Lane A uses.
    route_bridge(&store, &verified, model, &bridge)?;
    println!("gate7_bridge_routed=packet>judge>replay>crystallization");
    println!("lane_c_qualification=complete");
    Ok(())
}

fn gate1_triggers() -> Result<(), String> {
    let clean = RecallSignals {
        direct_trajectory_found: true,
        cycle_detected: false,
        ambiguous_branches: false,
        precision_weighted_error: 0.1,
        weighted_error_threshold: 0.5,
        readiness: 0.9,
        readiness_floor: 0.5,
        compliance_boundary: false,
    };
    if classify_trigger(&clean)?.is_some() {
        return Err("clean recall signals must not open a hold".into());
    }
    let cases = [
        (
            RecallSignals {
                compliance_boundary: true,
                ..clean
            },
            SuperpositionTrigger::ComplianceBoundary,
        ),
        (
            RecallSignals {
                cycle_detected: true,
                ..clean
            },
            SuperpositionTrigger::FixedPointOrCycle,
        ),
        (
            RecallSignals {
                direct_trajectory_found: false,
                ..clean
            },
            SuperpositionTrigger::NoDirectTrajectory,
        ),
        (
            RecallSignals {
                ambiguous_branches: true,
                ..clean
            },
            SuperpositionTrigger::BranchAmbiguity,
        ),
        (
            RecallSignals {
                precision_weighted_error: 0.9,
                ..clean
            },
            SuperpositionTrigger::HighPredictionError,
        ),
        (
            RecallSignals {
                readiness: 0.2,
                ..clean
            },
            SuperpositionTrigger::LowReadiness,
        ),
    ];
    for (signals, expected) in cases {
        if classify_trigger(&signals)? != Some(expected) {
            return Err("a trigger class did not fire on its signal".into());
        }
    }
    println!("gate1_triggers=6/6");
    Ok(())
}

fn gate5_negatives(
    branches: &[globalmind_runtime::SuperpositionBranch],
    policy: HoldingPolicy,
) -> Result<(), String> {
    // Each case removes exactly one collapse conjunct from the leader.
    let cases: [(&str, [bool; 4], f64); 5] = [
        ("not_compliant", [false, true, true, true], 0.9),
        ("not_prediction_ready", [true, false, true, true], 0.9),
        ("no_positive_progress", [true, true, false, true], 0.9),
        ("not_replayable", [true, true, true, false], 0.9),
        ("below_readiness_floor", [true, true, true, true], 0.4),
    ];
    for (name, [compliant, ready, progress, replayable], readiness) in cases {
        let hold = SuperpositionHold {
            id: RecordId::parse(format!("holdgate5_{name}"))?,
            workspace: RecordId::parse("globalmind")?,
            decision: RecordId::parse("decision_gate5")?,
            trigger_observation: None,
            hub: RecordId::parse("hub")?,
            well: None,
            request_digest: digest_fields(b"globalmind-gate5-v1", &[name]),
            prefix_digest: digest_fields(b"globalmind-gate5-prefix-v1", &[name]),
            trigger: SuperpositionTrigger::BranchAmbiguity,
            controller_version: CONTROLLER_VERSION.to_string(),
            controller_digest: controller_digest(),
            iteration_limit: 4,
            branches: branches.to_vec(),
        };
        let mut controller = globalmind_runtime::HoldingController::open(&hold, policy)?;
        let mut leader = update(&branches[0], 0.9, readiness, ready, progress, replayable);
        leader.compliance_permitted = compliant;
        let decision = controller.advance(vec![
            leader,
            update(&branches[1], 1.1, 0.6, true, true, true),
            update(&branches[2], 3.0, 0.3, true, true, true),
        ])?;
        // The weakened leader must never be the selected branch. Another
        // fully-gated branch may still collapse legitimately; what gate 5
        // forbids is selecting a branch missing any conjunct.
        if let globalmind_runtime::HoldingDecision::Collapse { branch, .. } = &decision
            && branch == &branches[0].id
        {
            return Err(format!(
                "gate 5 case {name} collapsed the branch missing a conjunct"
            ));
        }
    }
    println!("gate5_negatives=5/5");
    Ok(())
}

fn gate4_source_invariance(
    branches: &[globalmind_runtime::SuperpositionBranch],
    policy: HoldingPolicy,
) -> Result<(), String> {
    let sources = [
        BranchSource::DirectTrajectory,
        BranchSource::PhaseNeighbor,
        BranchSource::OperatorBridge,
        BranchSource::CrossWellBridge,
        BranchSource::FreshDerivation,
    ];
    let mut survivor_sets = Vec::new();
    for rotation in 0..sources.len() {
        let mut permuted = branches.to_vec();
        for (index, branch) in permuted.iter_mut().enumerate() {
            branch.source = sources[(index + rotation) % sources.len()];
        }
        let hold = SuperpositionHold {
            id: RecordId::parse(format!("holdgate4_{rotation}"))?,
            workspace: RecordId::parse("globalmind")?,
            decision: RecordId::parse("decision_gate4")?,
            trigger_observation: None,
            hub: RecordId::parse("hub")?,
            well: None,
            request_digest: digest_fields(b"globalmind-gate4-v1", &[&rotation.to_string()]),
            prefix_digest: digest_fields(b"globalmind-gate4-prefix-v1", &[&rotation.to_string()]),
            trigger: SuperpositionTrigger::BranchAmbiguity,
            controller_version: CONTROLLER_VERSION.to_string(),
            controller_digest: controller_digest(),
            iteration_limit: 4,
            branches: permuted.clone(),
        };
        let mut controller = globalmind_runtime::HoldingController::open(&hold, policy)?;
        let decision = controller.advance(vec![
            update(&permuted[0], 1.0, 0.6, false, true, true),
            update(&permuted[1], 1.2, 0.55, false, true, true),
            update(&permuted[2], 3.0, 0.3, false, true, true),
        ])?;
        let surviving = match decision {
            globalmind_runtime::HoldingDecision::Iterate { surviving, .. } => surviving,
            _ => return Err("gate 4 probe must iterate".into()),
        };
        survivor_sets.push(surviving);
    }
    if survivor_sets.windows(2).any(|pair| pair[0] != pair[1]) {
        return Err("source permutation changed the suppression survivor set".into());
    }
    println!(
        "gate4_source_invariance={}/{}",
        survivor_sets.len(),
        sources.len()
    );
    Ok(())
}

fn seed_atoms(
    store: &GlobalMindStore,
    verified: &globalmind_runtime::VerifiedManifest,
) -> Result<(), String> {
    store.open_distillation_ingress(verified)?;
    append_event(store, verified, 1, DistillationPhase::Extracting, None)?;
    append_event(store, verified, 2, DistillationPhase::Judging, None)?;
    for (name, text) in [("holdatom_a", "hold atom a"), ("holdatom_b", "hold atom b")] {
        let packet = seed_packet(verified, name, text);
        store.admit_packet(&packet)?;
    }
    append_event(store, verified, 3, DistillationPhase::Replaying, None)?;
    for (name, text) in [("holdatom_a", "hold atom a"), ("holdatom_b", "hold atom b")] {
        store.crystallize_atom(&seed_atom(verified, name, text))?;
    }
    println!("seed_atoms_crystallized=2");
    Ok(())
}

fn route_bridge(
    store: &GlobalMindStore,
    verified: &globalmind_runtime::VerifiedManifest,
    model: &globalmind_runtime::VerifiedModel,
    bridge: &globalmind_runtime::BridgeRoute,
) -> Result<(), String> {
    // Fresh run for the bridge so its packet lifecycle is independent. The
    // roster source/admission already exist and are reused, not recreated.
    let bridge_verified = bridge_manifest(verified, model)?;
    store.open_distillation_run(&bridge_verified)?;
    append_event(
        store,
        &bridge_verified,
        1,
        DistillationPhase::Extracting,
        None,
    )?;
    append_event(store, &bridge_verified, 2, DistillationPhase::Judging, None)?;
    let candidate_digest = digest_fields(
        b"globalmind-candidate-v1",
        &[
            "rank0_atom",
            "frozen_weight",
            bridge.bridge_digest.as_str(),
            bridge.bridge_digest.as_str(),
            globalmind_runtime::PROJECTOR_VERSION,
            bridge_verified.configuration_digest.as_str(),
        ],
    );
    let packet = DistillationPacket {
        id: RecordId::parse(format!("packet_{}", &candidate_digest.as_str()[..32]))?,
        workspace: bridge_verified.workspace.clone(),
        run: bridge_verified.run_id.clone(),
        admission: model.admission_id.clone(),
        source_kind: SourceKind::FrozenWeight,
        cell_kind: CellKind::Rank0Atom,
        source_fragment_digest: bridge.bridge_digest.clone(),
        teacher_neutral_input_digest: bridge.bridge_digest.clone(),
        projector_version: globalmind_runtime::PROJECTOR_VERSION.to_string(),
        projector_digest: digest_fields(
            b"globalmind-projector-v1",
            &[
                globalmind_runtime::PROJECTOR_VERSION,
                bridge_verified.configuration_digest.as_str(),
            ],
        ),
        configuration_digest: bridge_verified.configuration_digest.clone(),
        candidate_digest: candidate_digest.clone(),
        novelty_receipt_digest: digest_fields(
            b"globalmind-bridge-novelty-v1",
            &[bridge.bridge_digest.as_str()],
        ),
        judge_receipt_digest: digest_fields(
            b"globalmind-bridge-judge-v1",
            &[bridge.bridge_digest.as_str()],
        ),
        verdict: PacketVerdict::Admitted,
    };
    store.admit_packet(&packet)?;
    append_event(
        store,
        &bridge_verified,
        3,
        DistillationPhase::Replaying,
        None,
    )?;
    let text = "synthesized bridge atom";
    let slice = &candidate_digest.as_str()[..32];
    store.crystallize_atom(&CrystallizedAtom {
        id: RecordId::parse(format!("atom_{slice}"))?,
        address_id: RecordId::parse(format!("address_{slice}"))?,
        origin_id: RecordId::parse(format!("atomorigin_{slice}"))?,
        crystallization_id: RecordId::parse(format!("crystal_{slice}"))?,
        membership_id: RecordId::parse(format!("member_{slice}"))?,
        workspace: bridge_verified.workspace.clone(),
        run: bridge_verified.run_id.clone(),
        packet: packet.id.clone(),
        admission: model.admission_id.clone(),
        geometry_space: RecordId::parse("h3t2")?,
        well: RecordId::parse("well0")?,
        external_id: "bridge-atom".to_string(),
        content_kind: ContentKind::Text,
        content_text: Some(text.to_string()),
        content_uri: None,
        content_digest: Digest::sha256(text.as_bytes()),
        domain: "holding-bridge".to_string(),
        salience: 0.5,
        provenance: "superposition-bridge".to_string(),
        taint: "none".to_string(),
        pinned: false,
        h3_lorentz: [1.0, 0.0, 0.0, 0.0],
        t2_phase: [0.0, 0.0],
        geometry_model: "lorentz-t2".to_string(),
        geometry_version: "v1".to_string(),
        address_revision: 1,
        source_step_digest: bridge.bridge_digest.clone(),
        mint_receipt_digest: digest_fields(b"globalmind-bridge-mint-v1", &[slice]),
        payload_digest: Digest::sha256(text.as_bytes()),
        reconstruction_contract_digest: digest_fields(b"globalmind-bridge-contract-v1", &[slice]),
        reconstruction_receipt_digest: digest_fields(b"globalmind-bridge-recon-v1", &[slice]),
        crystallization_receipt_digest: digest_fields(b"globalmind-bridge-crystal-v1", &[slice]),
        classifier_version: "clf-v1".to_string(),
        classifier_digest: digest_fields(b"globalmind-bridge-clf-v1", &[slice]),
        boundary_receipt_digest: digest_fields(b"globalmind-bridge-boundary-v1", &[slice]),
    })?;
    let output_digest = digest_fields(
        b"globalmind-run-output-v1",
        &[candidate_digest.as_str(), slice],
    );
    append_event(
        store,
        &bridge_verified,
        4,
        DistillationPhase::Completed,
        Some(output_digest),
    )?;
    Ok(())
}

/// Rebinds the verified roster to a fresh run identity for the bridge, the
/// same way a second roster revision would. Artifact digests are unchanged;
/// only the method/configuration binding (and therefore the run) differs.
fn bridge_manifest(
    verified: &globalmind_runtime::VerifiedManifest,
    model: &globalmind_runtime::VerifiedModel,
) -> Result<globalmind_runtime::VerifiedManifest, String> {
    let manifest = globalmind_runtime::Manifest {
        workspace: verified.workspace.clone(),
        workspace_slug: verified.workspace.as_str().to_string(),
        method: "fresh-globalmind-holding-bridge-v1".to_string(),
        configuration_digest: digest_fields(
            b"globalmind-bridge-configuration-v1",
            &[verified.configuration_digest.as_str()],
        ),
        tppe_version: verified.tppe_version.clone(),
        tppe_configuration_digest: verified.tppe_configuration_digest.clone(),
        models: vec![globalmind_runtime::ModelEntry {
            provider: model.provider.clone(),
            name: model.name.clone(),
            revision: model.revision.clone(),
            modality: model.modality.clone(),
            eligibility_receipt_digest: model.eligibility_receipt_digest.clone(),
            judge_receipt_digest: model.judge_receipt_digest.clone(),
            artifacts: model
                .artifacts
                .iter()
                .map(|artifact| globalmind_runtime::ArtifactEntry {
                    role: artifact.role.clone(),
                    path: artifact.path.clone(),
                    sha256: artifact.sha256.clone(),
                })
                .collect(),
        }],
    };
    // Rehash again: the bridge run must not trust the earlier verification.
    manifest.verify_artifacts()
}

fn seed_packet(
    verified: &globalmind_runtime::VerifiedManifest,
    name: &str,
    text: &str,
) -> DistillationPacket {
    let model = &verified.models[0];
    let fragment = Digest::sha256(text.as_bytes());
    let candidate_digest = digest_fields(
        b"globalmind-candidate-v1",
        &[
            "rank0_atom",
            "frozen_weight",
            fragment.as_str(),
            fragment.as_str(),
            globalmind_runtime::PROJECTOR_VERSION,
            verified.configuration_digest.as_str(),
        ],
    );
    DistillationPacket {
        id: RecordId::parse(format!("packet_{}", &candidate_digest.as_str()[..32]))
            .expect("digest-derived packet id"),
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        admission: model.admission_id.clone(),
        source_kind: SourceKind::FrozenWeight,
        cell_kind: CellKind::Rank0Atom,
        source_fragment_digest: fragment.clone(),
        teacher_neutral_input_digest: fragment,
        projector_version: globalmind_runtime::PROJECTOR_VERSION.to_string(),
        projector_digest: digest_fields(
            b"globalmind-projector-v1",
            &[
                globalmind_runtime::PROJECTOR_VERSION,
                verified.configuration_digest.as_str(),
            ],
        ),
        configuration_digest: verified.configuration_digest.clone(),
        candidate_digest,
        novelty_receipt_digest: digest_fields(b"globalmind-seed-novelty-v1", &[name]),
        judge_receipt_digest: digest_fields(b"globalmind-seed-judge-v1", &[name]),
        verdict: PacketVerdict::Admitted,
    }
}

fn seed_atom(
    verified: &globalmind_runtime::VerifiedManifest,
    name: &str,
    text: &str,
) -> CrystallizedAtom {
    let packet = seed_packet(verified, name, text);
    let slice = packet.candidate_digest.as_str()[..32].to_string();
    CrystallizedAtom {
        id: RecordId::parse(name).expect("seed atom id"),
        address_id: RecordId::parse(format!("address_{slice}")).expect("address id"),
        origin_id: RecordId::parse(format!("atomorigin_{slice}")).expect("origin id"),
        crystallization_id: RecordId::parse(format!("crystal_{slice}")).expect("crystal id"),
        membership_id: RecordId::parse(format!("member_{slice}")).expect("membership id"),
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        packet: packet.id,
        admission: packet.admission,
        geometry_space: RecordId::parse("h3t2").expect("geometry id"),
        well: RecordId::parse("well0").expect("well id"),
        external_id: name.to_string(),
        content_kind: ContentKind::Text,
        content_text: Some(text.to_string()),
        content_uri: None,
        content_digest: Digest::sha256(text.as_bytes()),
        domain: "holding-seed".to_string(),
        salience: 0.5,
        provenance: "lane-c-seed".to_string(),
        taint: "none".to_string(),
        pinned: false,
        h3_lorentz: [1.0, 0.0, 0.0, 0.0],
        t2_phase: [0.0, 0.0],
        geometry_model: "lorentz-t2".to_string(),
        geometry_version: "v1".to_string(),
        address_revision: 1,
        source_step_digest: Digest::sha256(text.as_bytes()),
        mint_receipt_digest: digest_fields(b"globalmind-seed-mint-v1", &[name]),
        payload_digest: Digest::sha256(text.as_bytes()),
        reconstruction_contract_digest: digest_fields(b"globalmind-seed-contract-v1", &[name]),
        reconstruction_receipt_digest: digest_fields(b"globalmind-seed-recon-v1", &[name]),
        crystallization_receipt_digest: digest_fields(b"globalmind-seed-crystal-v1", &[name]),
        classifier_version: "clf-v1".to_string(),
        classifier_digest: digest_fields(b"globalmind-seed-clf-v1", &[name]),
        boundary_receipt_digest: digest_fields(b"globalmind-seed-boundary-v1", &[name]),
    }
}

fn append_event(
    store: &GlobalMindStore,
    verified: &globalmind_runtime::VerifiedManifest,
    ordinal: u32,
    phase: DistillationPhase,
    output_digest: Option<Digest>,
) -> Result<(), String> {
    let receipt = digest_fields(
        b"globalmind-run-event-v1",
        &[
            verified.run_id.as_str(),
            &ordinal.to_string(),
            phase_name(phase),
            output_digest.as_ref().map_or("", Digest::as_str),
            verified.input_digest.as_str(),
        ],
    );
    store.append_distillation_run_event(&DistillationRunEvent {
        id: RecordId::parse(format!("run_event_{}", receipt.as_str()))?,
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        ordinal,
        phase,
        input_digest: verified.input_digest.clone(),
        output_digest,
        tppe_version: verified.tppe_version.clone(),
        tppe_configuration_digest: verified.tppe_configuration_digest.clone(),
        receipt_digest: receipt,
    })
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

fn update(
    branch: &globalmind_runtime::SuperpositionBranch,
    free_energy_value: f64,
    readiness: f64,
    prediction_ready: bool,
    positive_progress: bool,
    replayable: bool,
) -> BranchUpdate {
    BranchUpdate {
        branch: branch.id.clone(),
        free_energy_value,
        precision_weighted_error: 0.4,
        readiness,
        compliance_permitted: branch.compliance_permitted,
        prediction_ready,
        positive_progress,
        replayable,
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
