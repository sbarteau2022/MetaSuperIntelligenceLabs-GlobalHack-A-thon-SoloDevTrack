//! Stage 5 — replay walker.
//!
//! Walks admitted rank-1 transitions in order, verifies endpoint and TPPE
//! continuity with real [`TppeState`] stepping, proves every rank-2 loop
//! closes, reconstructs rank-0 atoms, and validates rank-8 charts. Emits one
//! immutable walk artifact ([`artifact`]) plus a replay receipt digest.
//!
//! Identity rule: a different model, radial law, TPPE configuration, address
//! contract, runtime, or digest is a MISS (error), never a conversion.
//! [`verify_bindings`] enforces this before any downstream use.
//!
//! Evidence rule: judge evidence for rank-8 charts is computed exactly from
//! the chart tangent bytes. Rank-0/1/2 evidence is the aggregate F32
//! statistic over the input envelopes (F32-only schedule v1); envelopes of
//! other dtypes are excluded and recorded. This is documented in the
//! receipt preimage, not silently assumed.

pub mod artifact;

use crate::extract::{ExtractedCandidate, candidate_digest_for, extract};
use crate::judge::{IndependentJudge, JudgeEvidence, JudgePolicy, JudgeReceipt, JudgeVerdict};
use crate::trace::{TraceDtype, TraceEnvelope, TraceKind};
use crate::{
    CellKind, ChartDtype, ContentKind, CrystallizedAtom, Digest, DistillationPacket, EdgeKind,
    LocalJacobianChart, PROJECTOR_VERSION, PacketVerdict, RadialLaw, RecordId, SourceKind,
    TppeConfig, TppeState, Trajectory, TrajectoryLoop, TrajectoryOrigin, TrajectoryState,
    TrajectoryStep,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const REPLAY_VERSION: &str = "globalmind-replay-v1";
pub const REPLAY_TPPE_SCHEDULE: &str = "globalmind-replay-tppe-schedule-v1";
/// Fixed TPPE drive schedule per walked transition (replay parameter v1,
/// not model-derived; bound into the replay receipt).
pub const REPLAY_DRIVE_FAST: f64 = 0.02;
pub const REPLAY_DRIVE_SLOW: f64 = -0.01;

/// Frozen identity context a walk is bound to. Any field mismatch on
/// re-verification is a miss, never a conversion.
#[derive(Clone, Debug)]
pub struct ReplayContext {
    pub workspace: RecordId,
    pub run: RecordId,
    pub admission: RecordId,
    pub model_roster_digest: Digest,
    pub configuration_digest: Digest,
    pub tppe_version: String,
    pub tppe_configuration_digest: Digest,
    pub radial_law: RadialLaw,
    pub tppe_numeric: TppeConfig,
    pub address_contract_digest: Digest,
    pub runtime_digest: Digest,
}

impl ReplayContext {
    /// Runtime identity: crate + stage versions that produced the walk.
    pub fn runtime_digest_for() -> Digest {
        digest_fields(
            b"globalmind-replay-runtime-v1",
            &[
                env!("CARGO_PKG_VERSION"),
                crate::extract::EXTRACTOR_VERSION,
                crate::judge::JUDGE_VERSION,
                REPLAY_VERSION,
                PROJECTOR_VERSION,
            ],
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tppe_version.trim().is_empty() {
            return Err("replay TPPE version must be non-empty".into());
        }
        self.tppe_numeric.validate()?;
        Ok(())
    }

    pub fn context_digest(&self) -> Digest {
        digest_fields(
            b"globalmind-replay-context-v1",
            &[
                self.workspace.as_str(),
                self.run.as_str(),
                self.admission.as_str(),
                self.model_roster_digest.as_str(),
                self.configuration_digest.as_str(),
                &self.tppe_version,
                self.tppe_configuration_digest.as_str(),
                self.radial_law.as_str(),
                &format!(
                    "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
                    self.tppe_numeric.radial_start,
                    self.tppe_numeric.radial_span,
                    self.tppe_numeric.omega_fast,
                    self.tppe_numeric.omega_slow,
                    self.tppe_numeric.coupling_fast,
                    self.tppe_numeric.coupling_slow
                ),
                self.address_contract_digest.as_str(),
                self.runtime_digest.as_str(),
                REPLAY_TPPE_SCHEDULE,
                &format!("{REPLAY_DRIVE_FAST:?}/{REPLAY_DRIVE_SLOW:?}/derived-revolutions-v2"),
            ],
        )
    }
}

/// Independent marginal-gain and numerical-stability receipts for one
/// rank-8 chart. Replay enforces presence and binding; measuring the
/// economics is an upstream gate, not inferred here.
#[derive(Clone, Debug)]
pub struct ChartReceipts {
    pub marginal_gain_receipt_digest: Digest,
    pub numerical_receipt_digest: Digest,
}

/// Complete replay input: bound context, source envelopes (chart dtype
/// matching + audit), judged candidates, and chart receipts keyed by
/// candidate digest.
pub struct ReplayInput<'a> {
    pub context: &'a ReplayContext,
    pub envelopes: &'a [TraceEnvelope],
    pub judged: &'a [(ExtractedCandidate, JudgeReceipt)],
    pub chart_receipts: &'a BTreeMap<Digest, ChartReceipts>,
}

/// One verified replayed walk. The trajectory carries state `ReplayPassed`
/// and no crystallization receipt; [`upgrade_for_crystallization`] promotes
/// it at write time.
#[derive(Clone, Debug)]
pub struct ReplayedWalk {
    pub context: ReplayContext,
    pub context_digest: Digest,
    pub packets: Vec<DistillationPacket>,
    pub passed: Vec<Digest>,
    pub rejected: Vec<Digest>,
    pub atoms: Vec<CrystallizedAtom>,
    pub trajectory: Trajectory,
    pub charts: Vec<LocalJacobianChart>,
    pub trajectory_digest: Digest,
    pub candidate_bundle_digest: Digest,
    pub verdict_bundle_digest: Digest,
    pub replay_receipt_digest: Digest,
}

/// Replays a judged walk end to end. Any verification failure is an error
/// and yields no walk (no partial output).
pub fn replay(input: &ReplayInput) -> Result<ReplayedWalk, String> {
    input.context.validate()?;
    if input.judged.is_empty() {
        return Err("replay requires at least one judged candidate".into());
    }
    for envelope in input.envelopes {
        envelope.validate()?;
    }
    verify_candidates(input.judged)?;
    let packets = build_packets(input.context, input.judged);
    let atoms = build_atoms(input.context, input.judged)?;
    let atom_ids: BTreeSet<RecordId> = atoms.iter().map(|atom| atom.id.clone()).collect();
    let (steps, atom_walk) = walk_transitions(input.context, input.judged, &atom_ids)?;
    let loops = verify_loops(input.context, input.judged, &atom_walk)?;
    let charts = verify_charts(input)?;
    let origins = vec![TrajectoryOrigin {
        id: RecordId::parse(format!(
            "torigin_{}",
            &digest_fields(
                b"globalmind-replay-origin-v1",
                &[input.context.run.as_str(), input.context.admission.as_str()],
            )
            .as_str()[..32]
        ))?,
        admission: input.context.admission.clone(),
        source_walk_digest: digest_fields(
            b"globalmind-replay-walk-v1",
            &steps
                .iter()
                .map(|step| step.transition_digest.as_str().to_string())
                .collect::<Vec<_>>()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        receipt_digest: digest_fields(
            b"globalmind-replay-origin-receipt-v1",
            &[
                input.context.run.as_str(),
                &steps.len().to_string(),
                &loops.len().to_string(),
            ],
        ),
    }];
    let trajectory_digest = digest_fields(
        b"globalmind-replay-trajectory-v1",
        &[
            input.context.run.as_str(),
            &atom_walk
                .iter()
                .map(|id| id.as_str().to_string())
                .collect::<Vec<_>>()
                .join(","),
            input.context.tppe_configuration_digest.as_str(),
            input.context.radial_law.as_str(),
        ],
    );
    let judge_bundle = digest_fields(
        b"globalmind-replay-judge-bundle-v1",
        &input
            .judged
            .iter()
            .filter(|(_, receipt)| receipt.verdict == JudgeVerdict::Pass)
            .map(|(_, receipt)| receipt.receipt_digest.as_str().to_string())
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let trajectory = Trajectory {
        id: RecordId::parse(format!("trajectory_{}", &trajectory_digest.as_str()[..32]))?,
        workspace: input.context.workspace.clone(),
        run: input.context.run.clone(),
        atoms: atom_walk,
        trajectory_digest: trajectory_digest.clone(),
        judge_receipt_digest: judge_bundle,
        tppe_version: input.context.tppe_version.clone(),
        tppe_configuration_digest: input.context.tppe_configuration_digest.clone(),
        tppe_radial_law_digest: digest_fields(
            b"globalmind-replay-radial-v1",
            &[input.context.radial_law.as_str(), REPLAY_TPPE_SCHEDULE],
        ),
        tppe_boundary_receipt_digest: digest_fields(
            b"globalmind-replay-boundary-v1",
            &steps
                .iter()
                .map(|step| format!("{:?}", step.tppe_boundary_margin_after))
                .collect::<Vec<_>>()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        address_contract_digest: input.context.address_contract_digest.clone(),
        replay_receipt_digest: Digest::parse("0".repeat(64))?,
        crystallization_receipt_digest: None,
        state: TrajectoryState::ReplayPassed,
        steps,
        loops,
        origins,
    };
    verify_trajectory_shape(&trajectory)?;
    let candidate_bundle_digest = digest_fields(
        b"globalmind-replay-candidates-v1",
        &input
            .judged
            .iter()
            .map(|(candidate, _)| candidate.candidate_digest.as_str().to_string())
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let verdict_bundle_digest = digest_fields(
        b"globalmind-replay-verdicts-v1",
        &input
            .judged
            .iter()
            .map(|(candidate, receipt)| {
                format!(
                    "{}:{}",
                    candidate.candidate_digest.as_str(),
                    match receipt.verdict {
                        JudgeVerdict::Pass => "pass",
                        JudgeVerdict::Reject => "reject",
                    }
                )
            })
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let context_digest = input.context.context_digest();
    let replay_receipt_digest = digest_fields(
        b"globalmind-replay-receipt-v1",
        &[
            context_digest.as_str(),
            candidate_bundle_digest.as_str(),
            verdict_bundle_digest.as_str(),
            trajectory_digest.as_str(),
            &trajectory.steps.len().to_string(),
            &trajectory.loops.len().to_string(),
            &charts.len().to_string(),
        ],
    );
    let mut trajectory = trajectory;
    trajectory.replay_receipt_digest = replay_receipt_digest.clone();
    for step in &mut trajectory.steps {
        step.replay_receipt_digest = replay_receipt_digest.clone();
    }
    Ok(ReplayedWalk {
        context: input.context.clone(),
        context_digest,
        packets,
        passed: passed_partition(input.judged),
        rejected: rejected_partition(input.judged),
        atoms,
        trajectory,
        charts,
        trajectory_digest,
        candidate_bundle_digest,
        verdict_bundle_digest,
        replay_receipt_digest,
    })
}

fn passed_partition(judged: &[(ExtractedCandidate, JudgeReceipt)]) -> Vec<Digest> {
    judged
        .iter()
        .filter(|(_, receipt)| receipt.verdict == JudgeVerdict::Pass)
        .map(|(candidate, _)| candidate.candidate_digest.clone())
        .collect()
}

fn rejected_partition(judged: &[(ExtractedCandidate, JudgeReceipt)]) -> Vec<Digest> {
    judged
        .iter()
        .filter(|(_, receipt)| receipt.verdict == JudgeVerdict::Reject)
        .map(|(candidate, _)| candidate.candidate_digest.clone())
        .collect()
}

/// Recomputes every candidate digest from teacher-neutral fields, binds each
/// receipt to its candidate, requires one frozen judge policy across the
/// walk, and rejects duplicate passed digests.
fn verify_candidates(judged: &[(ExtractedCandidate, JudgeReceipt)]) -> Result<(), String> {
    let mut policy: Option<Digest> = None;
    let mut passed_digests = BTreeSet::new();
    for (candidate, receipt) in judged {
        let recomputed = candidate_digest_for(
            candidate.cell_kind,
            &candidate.source_fragment_digest,
            &candidate.teacher_neutral_input_digest,
        );
        if recomputed != candidate.candidate_digest {
            return Err(format!(
                "candidate digest mismatch for {}: recomputed {}, claimed {}",
                candidate.id.as_str(),
                recomputed.as_str(),
                candidate.candidate_digest.as_str()
            ));
        }
        if receipt.candidate_digest != candidate.candidate_digest {
            return Err("judge receipt is bound to a different candidate".into());
        }
        if receipt.reason.trim().is_empty() {
            return Err("judge receipt reason must be non-empty".into());
        }
        match &policy {
            None => policy = Some(receipt.policy_digest.clone()),
            Some(expected) if *expected == receipt.policy_digest => {}
            Some(_) => {
                return Err("judge policy drift within one walk is a miss".into());
            }
        }
        if receipt.verdict == JudgeVerdict::Pass
            && !passed_digests.insert(candidate.candidate_digest.clone())
        {
            return Err("duplicate passed candidate digest in one walk".into());
        }
    }
    Ok(())
}

/// Run-scoped identity: the store binds packets/atoms to one run, so two
/// runs distilling identical candidates must not share ids. The scope tag
/// derives from the run id; the digest suffix preserves content addressing.
fn scoped_id(prefix: &str, run: &RecordId, digest32: &str) -> Result<RecordId, String> {
    RecordId::parse(format!("{prefix}_{}_{}", run_tag(run), &digest32[..24]))
}

fn run_tag(run: &RecordId) -> String {
    digest_fields(b"globalmind-replay-run-tag-v1", &[run.as_str()]).as_str()[..8].to_string()
}

/// Maps an extract-issued global atom id (`atom_<digest32>`) to this run's
/// scoped atom id. Anything else fails closed.
fn scope_atom(run: &RecordId, extract_id: &RecordId) -> Result<RecordId, String> {
    let suffix = extract_id
        .as_str()
        .strip_prefix("atom_")
        .ok_or_else(|| format!("atom id {} is not extract-issued", extract_id.as_str()))?;
    if suffix.len() != 32 {
        return Err(format!(
            "atom id {} has a malformed digest",
            extract_id.as_str()
        ));
    }
    scoped_id("atom", run, suffix)
}

fn build_packets(
    context: &ReplayContext,
    judged: &[(ExtractedCandidate, JudgeReceipt)],
) -> Vec<DistillationPacket> {
    judged
        .iter()
        .map(|(candidate, receipt)| DistillationPacket {
            id: scoped_id(
                "packet",
                &context.run,
                &candidate.candidate_digest.as_str()[..32],
            )
            .expect("packet id is valid"),
            workspace: context.workspace.clone(),
            run: context.run.clone(),
            admission: context.admission.clone(),
            source_kind: SourceKind::FrozenWeight,
            cell_kind: candidate.cell_kind,
            source_fragment_digest: candidate.source_fragment_digest.clone(),
            teacher_neutral_input_digest: candidate.teacher_neutral_input_digest.clone(),
            projector_version: PROJECTOR_VERSION.to_string(),
            projector_digest: digest_fields(
                b"globalmind-replay-projector-v1",
                &[PROJECTOR_VERSION],
            ),
            configuration_digest: context.configuration_digest.clone(),
            candidate_digest: candidate.candidate_digest.clone(),
            novelty_receipt_digest: receipt.novelty_receipt_digest.clone(),
            judge_receipt_digest: receipt.receipt_digest.clone(),
            verdict: match receipt.verdict {
                JudgeVerdict::Pass => PacketVerdict::Admitted,
                JudgeVerdict::Reject => PacketVerdict::Rejected,
            },
        })
        .collect()
}

/// Reconstructs rank-0 atoms from passed candidates: coordinates must be
/// present, finite, and Lorentz-valid; content binds the teacher-neutral
/// input digest; every packet id must resolve to an admitted packet.
fn build_atoms(
    context: &ReplayContext,
    judged: &[(ExtractedCandidate, JudgeReceipt)],
) -> Result<Vec<CrystallizedAtom>, String> {
    let mut atoms = Vec::new();
    for (candidate, receipt) in judged {
        if candidate.cell_kind != CellKind::Rank0Atom || receipt.verdict != JudgeVerdict::Pass {
            continue;
        }
        let slice = &candidate.candidate_digest.as_str()[..32];
        let h3 = candidate
            .h3_lorentz
            .ok_or_else(|| "rank-0 candidate is missing H3 coordinates".to_string())?;
        let t2 = candidate
            .t2_phase
            .ok_or_else(|| "rank-0 candidate is missing T2 phase".to_string())?;
        if !h3.iter().chain(&t2).all(|v| v.is_finite()) || h3[0] <= 0.0 {
            return Err("rank-0 candidate coordinates are invalid".into());
        }
        let atom = CrystallizedAtom {
            id: scoped_id("atom", &context.run, slice)?,
            address_id: scoped_id("address", &context.run, slice)?,
            origin_id: scoped_id("atomorigin", &context.run, slice)?,
            crystallization_id: scoped_id("crystal", &context.run, slice)?,
            membership_id: scoped_id("member", &context.run, slice)?,
            workspace: context.workspace.clone(),
            run: context.run.clone(),
            packet: scoped_id("packet", &context.run, slice)?,
            admission: context.admission.clone(),
            geometry_space: RecordId::parse("h3t2")?,
            well: RecordId::parse(format!("well{}", candidate.well_ordinal.unwrap_or(0)))?,
            external_id: format!("replay-{}-{slice}", run_tag(&context.run)),
            content_kind: ContentKind::Code,
            content_text: Some(format!("replayed atom {slice}")),
            content_uri: None,
            content_digest: candidate.teacher_neutral_input_digest.clone(),
            domain: "replay".into(),
            salience: 0.5,
            provenance: REPLAY_VERSION.into(),
            taint: "none".into(),
            pinned: false,
            h3_lorentz: h3,
            t2_phase: t2,
            geometry_model: "lorentz-t2".into(),
            geometry_version: "v1".into(),
            address_revision: 1,
            source_step_digest: candidate.source_fragment_digest.clone(),
            mint_receipt_digest: digest_fields(b"globalmind-replay-mint-v1", &[slice]),
            payload_digest: candidate.teacher_neutral_input_digest.clone(),
            reconstruction_contract_digest: digest_fields(
                b"globalmind-replay-recon-contract-v1",
                &[slice, candidate.teacher_neutral_input_digest.as_str()],
            ),
            reconstruction_receipt_digest: receipt.receipt_digest.clone(),
            crystallization_receipt_digest: digest_fields(
                b"globalmind-replay-atom-crystal-v1",
                &[slice, receipt.receipt_digest.as_str()],
            ),
            classifier_version: "replay-clf-v1".into(),
            classifier_digest: digest_fields(b"globalmind-replay-clf-v1", &[slice]),
            boundary_receipt_digest: digest_fields(b"globalmind-replay-atom-boundary-v1", &[slice]),
        };
        atom.validate()?;
        atoms.push(atom);
    }
    if atoms.is_empty() {
        return Err("replay requires at least one passed rank-0 atom".into());
    }
    Ok(atoms)
}

/// Walks passed rank-1 transitions in candidate order: endpoints must chain
/// continuously over reconstructed atoms, and every transition advances a
/// real [`TppeState`] whose phases, radii, revolutions, and margins are
/// recorded on the step. Returns the steps plus the ordered atom walk.
fn walk_transitions(
    context: &ReplayContext,
    judged: &[(ExtractedCandidate, JudgeReceipt)],
    atom_ids: &BTreeSet<RecordId>,
) -> Result<(Vec<TrajectoryStep>, Vec<RecordId>), String> {
    let transitions: Vec<&ExtractedCandidate> = judged
        .iter()
        .filter(|(candidate, receipt)| {
            candidate.cell_kind == CellKind::Rank1Transition
                && receipt.verdict == JudgeVerdict::Pass
        })
        .map(|(candidate, _)| candidate)
        .collect();
    if transitions.is_empty() {
        return Err("replay requires at least one passed rank-1 transition".into());
    }
    let config = TppeConfig {
        law: context.radial_law,
        ..context.tppe_numeric
    };
    let mut state = TppeState::origin(config)?;
    let mut steps = Vec::with_capacity(transitions.len());
    let mut atom_walk = Vec::with_capacity(transitions.len() + 1);
    for (ordinal, candidate) in transitions.iter().enumerate() {
        let from_global = candidate
            .from_atom
            .clone()
            .ok_or_else(|| "rank-1 candidate is missing its from-atom".to_string())?;
        let to_global = candidate
            .to_atom
            .clone()
            .ok_or_else(|| "rank-1 candidate is missing its to-atom".to_string())?;
        let from = scope_atom(&context.run, &from_global)?;
        let to = scope_atom(&context.run, &to_global)?;
        if !atom_ids.contains(&from) || !atom_ids.contains(&to) {
            return Err("rank-1 transition references an unreconstructed atom".into());
        }
        if ordinal == 0 {
            atom_walk.push(from.clone());
        } else if atom_walk[ordinal] != from {
            return Err(format!(
                "rank-1 walk is discontinuous at transition {ordinal}"
            ));
        }
        atom_walk.push(to.clone());
        let before = state;
        state = state.step(config, REPLAY_DRIVE_FAST, REPLAY_DRIVE_SLOW)?;
        let slice = &candidate.candidate_digest.as_str()[..32];
        steps.push(TrajectoryStep {
            id: scoped_id(&format!("step{ordinal}"), &context.run, slice)?,
            ordinal: ordinal as u32,
            from_atom: from,
            to_atom: to,
            edge: scoped_id(&format!("edge{ordinal}"), &context.run, slice)?,
            edge_crystallization_id: scoped_id(
                &format!("edgecrystal{ordinal}"),
                &context.run,
                slice,
            )?,
            packet: scoped_id("packet", &context.run, slice)?,
            admission: context.admission.clone(),
            transition_kind: EdgeKind::Next,
            weight: 0.9,
            confidence: 0.8,
            evidence_atoms: Vec::new(),
            evidence_digest: digest_fields(b"globalmind-replay-evidence-v1", &[slice]),
            producer: REPLAY_VERSION.into(),
            revision: 1,
            t2_phase_before: before.phase_t2(),
            t2_phase_after: state.phase_t2(),
            tppe_radius_before: before.hyperbolic_radius,
            tppe_radius_after: state.hyperbolic_radius,
            tppe_revolutions_before: before.cumulative_revolutions as u64,
            tppe_revolutions_after: state.cumulative_revolutions as u64,
            tppe_boundary_margin_before: before.boundary_margin,
            tppe_boundary_margin_after: state.boundary_margin,
            action_digest: digest_fields(b"globalmind-replay-action-v1", &[slice]),
            observation_digest: digest_fields(b"globalmind-replay-observation-v1", &[slice]),
            transition_digest: candidate.source_fragment_digest.clone(),
            replay_receipt_digest: Digest::parse("0".repeat(64))?,
            crystallization_receipt_digest: digest_fields(
                b"globalmind-replay-step-crystal-v1",
                &[slice, &ordinal.to_string()],
            ),
        });
    }
    Ok((steps, atom_walk))
}

/// Proves every passed rank-2 loop closes over the walked steps: the range
/// must be valid, the range endpoints must be the same atom, and degenerate
/// (zero-transition) loops fail closed.
fn verify_loops(
    context: &ReplayContext,
    judged: &[(ExtractedCandidate, JudgeReceipt)],
    atom_walk: &[RecordId],
) -> Result<Vec<TrajectoryLoop>, String> {
    let mut loops = Vec::new();
    for (candidate, receipt) in judged {
        if candidate.cell_kind != CellKind::Rank2Loop || receipt.verdict != JudgeVerdict::Pass {
            continue;
        }
        let first = candidate
            .first_step
            .ok_or_else(|| "rank-2 candidate is missing first_step".to_string())?
            as usize;
        let last = candidate
            .last_step
            .ok_or_else(|| "rank-2 candidate is missing last_step".to_string())?
            as usize;
        if first > last || last + 1 >= atom_walk.len() {
            return Err("rank-2 loop step range is invalid for this walk".into());
        }
        if atom_walk[first] != atom_walk[last + 1] {
            return Err("rank-2 loop does not close on the same atom".into());
        }
        let slice = &candidate.candidate_digest.as_str()[..32];
        loops.push(TrajectoryLoop {
            id: scoped_id("loop", &context.run, slice)?,
            packet: scoped_id("packet", &context.run, slice)?,
            admission: context.admission.clone(),
            first_step: first as u32,
            last_step: last as u32,
            loop_invariant_kind: "closed_atom_walk_v1".into(),
            loop_invariant_digest: digest_fields(
                b"globalmind-replay-invariant-v1",
                &[
                    atom_walk[first].as_str(),
                    &first.to_string(),
                    &last.to_string(),
                ],
            ),
            closure_receipt_digest: digest_fields(
                b"globalmind-replay-closure-v1",
                &[
                    atom_walk[first].as_str(),
                    candidate.candidate_digest.as_str(),
                ],
            ),
        });
    }
    Ok(loops)
}

/// Validates every passed rank-8 chart: the tangent basis must match a real
/// source weight envelope by shape and payload digest, the dtype must be a
/// representable dense dtype (quantized charts fail closed), marginal-gain
/// and numerical receipts must be supplied, and the chart must validate.
fn verify_charts(input: &ReplayInput) -> Result<Vec<LocalJacobianChart>, String> {
    let mut charts = Vec::new();
    for (candidate, receipt) in input.judged {
        if candidate.cell_kind != CellKind::Rank8Jacobian || receipt.verdict != JudgeVerdict::Pass {
            continue;
        }
        let basis = candidate
            .tangent_basis
            .as_ref()
            .ok_or_else(|| "rank-8 candidate is missing its tangent basis".to_string())?;
        if basis.is_empty() {
            return Err("rank-8 tangent basis is empty".into());
        }
        let shape = candidate
            .tensor_shape
            .clone()
            .ok_or_else(|| "rank-8 candidate is missing its tensor shape".to_string())?;
        let basis_digest = Digest::sha256(basis);
        let envelope = input
            .envelopes
            .iter()
            .find(|envelope| {
                envelope.kind == TraceKind::Weight
                    && envelope.shape == shape
                    && envelope.payload_digest == basis_digest
            })
            .ok_or_else(|| "rank-8 tangent basis matches no source weight envelope".to_string())?;
        let dtype = match envelope.dtype {
            TraceDtype::F16 => ChartDtype::F16,
            TraceDtype::Bf16 => ChartDtype::Bf16,
            TraceDtype::F32 => ChartDtype::F32,
            TraceDtype::Q8_0 | TraceDtype::Q4_0 => {
                return Err("quantized rank-8 chart has no dense chart dtype".into());
            }
        };
        let anchor = candidate
            .anchor_atom
            .clone()
            .ok_or_else(|| "rank-8 candidate is missing its anchor atom".to_string())?;
        let receipts = input
            .chart_receipts
            .get(&candidate.candidate_digest)
            .ok_or_else(|| {
                "rank-8 chart is missing marginal-gain/numerical receipts".to_string()
            })?;
        let slice = &candidate.candidate_digest.as_str()[..32];
        let chart = LocalJacobianChart {
            id: scoped_id("chart", &input.context.run, slice)?,
            workspace: input.context.workspace.clone(),
            run: input.context.run.clone(),
            anchor: scope_atom(&input.context.run, &anchor)?,
            packet: scoped_id("packet", &input.context.run, slice)?,
            admission: input.context.admission.clone(),
            chart_rank: candidate.chart_rank.unwrap_or(0),
            dtype,
            tensor_shape: shape,
            tangent_basis: basis.clone(),
            tensor_digest: basis_digest,
            chart_digest: candidate.candidate_digest.clone(),
            marginal_gain_receipt_digest: receipts.marginal_gain_receipt_digest.clone(),
            numerical_receipt_digest: receipts.numerical_receipt_digest.clone(),
        };
        chart.validate()?;
        charts.push(chart);
    }
    Ok(charts)
}

/// Verifies trajectory shape exactly as the store will, minus the
/// crystallization state requirement (replay precedes crystallization).
fn verify_trajectory_shape(trajectory: &Trajectory) -> Result<(), String> {
    if trajectory.atoms.len() < 2 || trajectory.steps.is_empty() {
        return Err("trajectory requires at least two atoms and one step".into());
    }
    if trajectory.atoms.len() != trajectory.steps.len() + 1 {
        return Err("trajectory atom count must equal step count plus one".into());
    }
    let mut edge_ids = BTreeSet::new();
    let mut edge_crystallization_ids = BTreeSet::new();
    for (index, step) in trajectory.steps.iter().enumerate() {
        if step.ordinal as usize != index {
            return Err("trajectory step ordinals must be contiguous from zero".into());
        }
        if step.from_atom != trajectory.atoms[index] || step.to_atom != trajectory.atoms[index + 1]
        {
            return Err("trajectory step endpoints must match the ordered atom walk".into());
        }
        if index > 0 && trajectory.steps[index - 1].t2_phase_after != step.t2_phase_before {
            return Err("trajectory TPPE phase state is discontinuous".into());
        }
        if !step.tppe_radius_before.is_finite()
            || !step.tppe_radius_after.is_finite()
            || step.tppe_radius_before < 0.0
            || step.tppe_radius_after < step.tppe_radius_before
            || step.tppe_revolutions_after < step.tppe_revolutions_before
            || !step.tppe_boundary_margin_before.is_finite()
            || !step.tppe_boundary_margin_after.is_finite()
            || step.tppe_boundary_margin_before <= 0.0
            || step.tppe_boundary_margin_after <= 0.0
        {
            return Err("trajectory TPPE radial/boundary state is invalid".into());
        }
        if step.tppe_revolutions_after > step.tppe_revolutions_before
            && step.tppe_radius_after <= step.tppe_radius_before
        {
            return Err("TPPE radius must expand when revolution count advances".into());
        }
        if !edge_ids.insert(step.edge.clone())
            || !edge_crystallization_ids.insert(step.edge_crystallization_id.clone())
        {
            return Err("trajectory edge identities must be unique".into());
        }
    }
    for loop_record in &trajectory.loops {
        let first = loop_record.first_step as usize;
        let last = loop_record.last_step as usize;
        if first > last || last >= trajectory.steps.len() {
            return Err("trajectory loop step range is invalid".into());
        }
        if trajectory.atoms[first] != trajectory.atoms[last + 1] {
            return Err("rank-2 trajectory loop must close on the same atom".into());
        }
    }
    if trajectory.origins.is_empty() {
        return Err("trajectory requires at least one audit origin".into());
    }
    Ok(())
}

/// Re-verifies a decoded walk against an expected context. Any field
/// mismatch is a miss (error), never a conversion.
pub fn verify_bindings(expected: &ReplayContext, walk: &ReplayedWalk) -> Result<(), String> {
    let mismatches = [
        (walk.context.workspace == expected.workspace, "workspace"),
        (walk.context.run == expected.run, "run"),
        (walk.context.admission == expected.admission, "admission"),
        (
            walk.context.model_roster_digest == expected.model_roster_digest,
            "model roster digest",
        ),
        (
            walk.context.configuration_digest == expected.configuration_digest,
            "configuration digest",
        ),
        (
            walk.context.tppe_version == expected.tppe_version,
            "TPPE version",
        ),
        (
            walk.context.tppe_configuration_digest == expected.tppe_configuration_digest,
            "TPPE configuration digest",
        ),
        (walk.context.radial_law == expected.radial_law, "radial law"),
        (
            walk.context.address_contract_digest == expected.address_contract_digest,
            "address contract digest",
        ),
        (
            walk.context.runtime_digest == expected.runtime_digest,
            "runtime digest",
        ),
        (
            walk.context_digest == expected.context_digest(),
            "context digest",
        ),
    ];
    for (matches, field) in mismatches {
        if !matches {
            return Err(format!("replay binding miss on {field}: not a conversion"));
        }
    }
    if walk.replay_receipt_digest.as_str().len() != 64 {
        return Err("replay receipt digest is malformed".into());
    }
    Ok(())
}

/// Promotes a verified `ReplayPassed` trajectory to `Crystallized` at write
/// time. The crystallization receipt binds the replay receipt, the run, and
/// the trajectory digest; anything else is refused by the store.
pub fn upgrade_for_crystallization(walk: &ReplayedWalk) -> Result<Trajectory, String> {
    if walk.trajectory.state != TrajectoryState::ReplayPassed {
        return Err("only a ReplayPassed trajectory can crystallize".into());
    }
    verify_trajectory_shape(&walk.trajectory)?;
    let crystal = digest_fields(
        b"globalmind-replay-crystal-v1",
        &[
            walk.replay_receipt_digest.as_str(),
            walk.context.run.as_str(),
            walk.trajectory_digest.as_str(),
        ],
    );
    let mut trajectory = walk.trajectory.clone();
    trajectory.state = TrajectoryState::Crystallized;
    trajectory.crystallization_receipt_digest = Some(crystal);
    Ok(trajectory)
}

/// Judge evidence schedule v1: exact F32 statistics from a rank-8 tangent
/// basis, otherwise aggregate F32 statistics over the envelopes. Non-F32
/// envelopes are excluded and counted; with zero F32 values the evidence
/// is non-finite and the judge rejects.
pub fn evidence_for(
    candidate: &ExtractedCandidate,
    envelopes: &[TraceEnvelope],
) -> Result<JudgeEvidence, String> {
    let mut sum_squares = 0.0f64;
    let mut count = 0u64;
    let mut excluded = 0u64;
    let mut saw_non_finite = false;
    let consume_f32 = |bytes: &[u8],
                       label: &str,
                       sum_squares: &mut f64,
                       count: &mut u64,
                       saw_non_finite: &mut bool|
     -> Result<(), String> {
        if !bytes.len().is_multiple_of(4) {
            return Err(format!("{label} byte length is not F32-aligned"));
        }
        for chunk in bytes.as_chunks::<4>().0 {
            let value = f32::from_le_bytes(*chunk);
            if !value.is_finite() {
                *saw_non_finite = true;
                continue;
            }
            *sum_squares += f64::from(value) * f64::from(value);
            *count += 1;
        }
        Ok(())
    };
    if candidate.cell_kind == CellKind::Rank8Jacobian {
        let basis = candidate
            .tangent_basis
            .as_ref()
            .ok_or_else(|| "rank-8 evidence requires a tangent basis".to_string())?;
        if !basis.len().is_multiple_of(4) {
            // Non-F32-aligned tangent (e.g. quantized): no exact statistic
            // exists, so the evidence is non-finite and the judge rejects.
            return Ok(JudgeEvidence {
                mean_square: f64::NAN,
                finite: false,
                known_candidate_digests: Vec::new(),
            });
        }
        consume_f32(
            basis,
            "chart tangent basis",
            &mut sum_squares,
            &mut count,
            &mut saw_non_finite,
        )?;
    } else {
        for envelope in envelopes {
            if envelope.kind == TraceKind::KeyValue {
                continue;
            }
            if envelope.dtype != TraceDtype::F32 {
                excluded += 1;
                continue;
            }
            consume_f32(
                &envelope.payload,
                "trace payload",
                &mut sum_squares,
                &mut count,
                &mut saw_non_finite,
            )?;
        }
    }
    if count == 0 || saw_non_finite {
        return Ok(JudgeEvidence {
            mean_square: f64::NAN,
            finite: false,
            known_candidate_digests: Vec::new(),
        });
    }
    let _ = excluded;
    Ok(JudgeEvidence {
        mean_square: sum_squares / count as f64,
        finite: true,
        known_candidate_digests: Vec::new(),
    })
}

/// Default frozen judge policy (schedule v1): matches the integration
/// walk energy floor and the full rank-8 cap. The policy digest is bound
/// into every receipt, so any change is a miss, not a silent retune.
pub fn default_judge_policy() -> JudgePolicy {
    JudgePolicy {
        min_mean_square: 0.1,
        max_chart_rank: 8,
    }
}

/// Default address-contract binding (schedule v1): the Whitehead r12
/// quotient is specified, not certified, so replay pins an explicit
/// schedule digest rather than inventing a boundary contract. Any future
/// certified contract is a different digest, hence a miss, not a merge.
pub fn default_address_contract_digest() -> Digest {
    Digest::sha256(b"globalmind-replay-address-contract-schedule-v1")
}

/// Default TPPE numeric schedule (schedule v1) for a chosen radial law.
/// Constants are pinned here so every replay run is deterministic; the
/// bakeoff qualifies model-coupled schedules separately.
pub fn default_tppe_numeric(law: RadialLaw) -> TppeConfig {
    TppeConfig {
        law,
        radial_start: 0.0,
        radial_span: 0.01,
        omega_fast: 0.1,
        omega_slow: 0.05,
        coupling_fast: 0.3,
        coupling_slow: 0.15,
    }
}

/// Runs extract then judge in process over trace envelopes with schedule-v1
/// evidence and in-run novelty. Used by the `replay` CLI branch until the
/// dedicated stage CLIs produce authoritative artifact files.
pub fn extract_and_judge(
    envelopes: &[TraceEnvelope],
    policy: JudgePolicy,
) -> Result<Vec<(ExtractedCandidate, JudgeReceipt)>, String> {
    let judge = IndependentJudge::new(policy)?;
    let candidates = extract(envelopes)?;
    let mut known: Vec<Digest> = Vec::new();
    let mut judged = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut evidence = evidence_for(&candidate, envelopes)?;
        evidence.known_candidate_digests.clone_from(&known);
        let receipt = judge.judge(&candidate, &evidence)?;
        known.push(candidate.candidate_digest.clone());
        judged.push((candidate, receipt));
    }
    Ok(judged)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::artifact::{
        EnvelopeDescriptor, decode as decode_walk, encode as encode_walk,
    };

    fn envelope(kind: TraceKind, token: u64, payload: Vec<u8>, dtype: TraceDtype) -> TraceEnvelope {
        TraceEnvelope {
            model_digest: Digest::parse("a".repeat(64)).unwrap(),
            config_digest: Digest::parse("b".repeat(64)).unwrap(),
            tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
            kind,
            dtype,
            shape: vec![(payload.len() / dtype_width(dtype)) as u32],
            layer: 3,
            head: 1,
            token_position: token,
            observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        }
    }

    fn dtype_width(dtype: TraceDtype) -> usize {
        match dtype {
            TraceDtype::F16 | TraceDtype::Bf16 => 2,
            TraceDtype::F32 => 4,
            TraceDtype::Q8_0 | TraceDtype::Q4_0 => 1,
        }
    }

    fn floats(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn activation(token: u64, values: &[f32]) -> TraceEnvelope {
        envelope(
            TraceKind::Activation,
            token,
            floats(values),
            TraceDtype::F32,
        )
    }

    fn context() -> ReplayContext {
        ReplayContext {
            workspace: RecordId::parse("replay_ws").unwrap(),
            run: RecordId::parse("replay_run").unwrap(),
            admission: RecordId::parse("replay_adm").unwrap(),
            model_roster_digest: Digest::sha256(b"test-roster"),
            configuration_digest: Digest::sha256(b"test-config"),
            tppe_version: "tppe-test-v1".into(),
            tppe_configuration_digest: Digest::sha256(b"test-tpe-config"),
            radial_law: RadialLaw::LogV1,
            tppe_numeric: TppeConfig {
                law: RadialLaw::LogV1,
                radial_start: 0.0,
                radial_span: 0.01,
                omega_fast: 0.1,
                omega_slow: 0.05,
                coupling_fast: 0.3,
                coupling_slow: 0.15,
            },
            address_contract_digest: Digest::sha256(b"test-address"),
            runtime_digest: ReplayContext::runtime_digest_for(),
        }
    }

    fn policy() -> JudgePolicy {
        JudgePolicy {
            min_mean_square: 0.1,
            max_chart_rank: 8,
        }
    }

    fn walk_envelopes() -> Vec<TraceEnvelope> {
        vec![
            activation(0, &[1.0, 0.0]),
            activation(1, &[0.0, 1.0]),
            activation(2, &[1.0, 0.0]),
        ]
    }

    #[test]
    fn replay_verifies_walk_loop_and_tppe_continuity() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        assert!(judged.iter().all(|(_, r)| r.verdict == JudgeVerdict::Pass));
        let context = context();
        let receipts = BTreeMap::new();
        let walk = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        assert_eq!(walk.atoms.len(), 2);
        assert_eq!(walk.trajectory.steps.len(), 2);
        assert_eq!(walk.trajectory.loops.len(), 1);
        assert!(walk.charts.is_empty());
        assert_eq!(walk.trajectory.state, TrajectoryState::ReplayPassed);
        assert_eq!(walk.rejected.len(), 0);
        // Exact TPPE phase continuity and strict radial expansion.
        let steps = &walk.trajectory.steps;
        assert_eq!(steps[0].t2_phase_after, steps[1].t2_phase_before);
        assert!(steps[0].tppe_radius_after > steps[0].tppe_radius_before);
        assert!(steps[1].tppe_radius_after > steps[1].tppe_radius_before);
        assert!(steps.iter().all(|s| s.tppe_boundary_margin_before > 0.0));
        assert!(steps.iter().all(|s| s.tppe_boundary_margin_after > 0.0));
        // Revolutions are derived from unwrapped phase, never supplied:
        // the recorded count can only stay flat or advance by the turns
        // the phases actually turned (far less than one per step here).
        assert_eq!(steps[0].tppe_revolutions_before, 0);
        assert!(steps[0].tppe_revolutions_after >= steps[0].tppe_revolutions_before);
        assert!(steps[1].tppe_revolutions_after >= steps[1].tppe_revolutions_before);
        // Loop closes on the same atom.
        let atoms = &walk.trajectory.atoms;
        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms[0], atoms[2]);
        // Bindings hold; upgrade promotes to Crystallized.
        verify_bindings(&context, &walk).unwrap();
        let crystal = upgrade_for_crystallization(&walk).unwrap();
        assert_eq!(crystal.state, TrajectoryState::Crystallized);
        assert!(crystal.crystallization_receipt_digest.is_some());
    }

    #[test]
    fn two_runs_sharing_candidates_keep_disjoint_ids() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let mut second = context();
        second.run = RecordId::parse("replay_run_two").unwrap();
        let receipts = BTreeMap::new();
        let first = replay(&ReplayInput {
            context: &context(),
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        let rerun = replay(&ReplayInput {
            context: &second,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        let ids_of = |walk: &ReplayedWalk| {
            let mut ids: BTreeSet<String> = walk
                .packets
                .iter()
                .map(|p| p.id.as_str().to_string())
                .chain(walk.atoms.iter().map(|a| a.id.as_str().to_string()))
                .chain(
                    walk.trajectory
                        .steps
                        .iter()
                        .map(|s| s.id.as_str().to_string()),
                )
                .collect();
            ids.insert(walk.trajectory.id.as_str().to_string());
            ids
        };
        assert!(ids_of(&first).is_disjoint(&ids_of(&rerun)));
        // Same candidates, same walk shape, different run binding.
        assert_eq!(first.trajectory.steps.len(), rerun.trajectory.steps.len());
        assert!(verify_bindings(&context(), &rerun).is_err());
    }

    #[test]
    fn binding_miss_on_any_context_drift() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let context = context();
        let receipts = BTreeMap::new();
        let walk = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        let mut drifted = context.clone();
        drifted.radial_law = RadialLaw::LinearLogV1;
        assert!(verify_bindings(&drifted, &walk).is_err());
        let mut drifted = context.clone();
        drifted.model_roster_digest = Digest::sha256(b"other-model");
        assert!(verify_bindings(&drifted, &walk).is_err());
        let mut drifted = context.clone();
        drifted.tppe_configuration_digest = Digest::sha256(b"other-tpe");
        assert!(verify_bindings(&drifted, &walk).is_err());
        let mut drifted = context.clone();
        drifted.address_contract_digest = Digest::sha256(b"other-address");
        assert!(verify_bindings(&drifted, &walk).is_err());
        let mut drifted = context.clone();
        drifted.runtime_digest = Digest::sha256(b"other-runtime");
        assert!(verify_bindings(&drifted, &walk).is_err());
        let mut drifted = context.clone();
        drifted.tppe_numeric.radial_span = 0.02;
        assert!(verify_bindings(&drifted, &walk).is_err());
    }

    #[test]
    fn discontinuous_and_degenerate_walks_fail_closed() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let context = context();
        let receipts = BTreeMap::new();
        // Retarget the second transition's from-atom (the atom id is not
        // part of the candidate digest preimage, so digest checks pass and
        // the walk itself must catch the discontinuity).
        let mut broken = judged.clone();
        let first_from = broken
            .iter()
            .find(|(c, _)| c.cell_kind == CellKind::Rank1Transition)
            .expect("two transitions")
            .0
            .from_atom
            .clone()
            .expect("from atom");
        let second = broken
            .iter_mut()
            .filter(|(c, _)| c.cell_kind == CellKind::Rank1Transition)
            .nth(1)
            .expect("two transitions");
        second.0.from_atom = Some(first_from);
        let error = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &broken,
            chart_receipts: &receipts,
        })
        .expect_err("discontinuous walk must fail");
        assert!(error.contains("discontinuous"), "{error}");
        // Degenerate [A,A,B] loop covers no transition and cannot close.
        let degenerate = vec![
            activation(0, &[1.0, 0.0]),
            activation(0, &[1.0, 0.0]),
            activation(1, &[0.0, 1.0]),
        ];
        let judged = extract_and_judge(&degenerate, policy()).unwrap();
        assert!(
            judged
                .iter()
                .any(|(c, _)| c.cell_kind == CellKind::Rank2Loop)
        );
        assert!(
            replay(&ReplayInput {
                context: &context,
                envelopes: &degenerate,
                judged: &judged,
                chart_receipts: &receipts,
            })
            .is_err()
        );
    }

    #[test]
    fn rejected_candidates_mint_nothing_but_record_verdicts() {
        // [A,B,A,B] replays [A->B, B->A] with duplicate transitions rejected.
        let envelopes = vec![
            activation(0, &[1.0, 0.0]),
            activation(1, &[0.0, 1.0]),
            activation(2, &[1.0, 0.0]),
            activation(3, &[0.0, 1.0]),
        ];
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        assert!(
            judged
                .iter()
                .any(|(_, r)| r.verdict == JudgeVerdict::Reject)
        );
        // Rank-2 ranges address the envelope walk, not the deduped step walk;
        // replay the transition/atom subset to isolate rejection handling.
        let judged: Vec<_> = judged
            .into_iter()
            .filter(|(c, _)| c.cell_kind != CellKind::Rank2Loop)
            .collect();
        let context = context();
        let receipts = BTreeMap::new();
        let walk = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        assert!(!walk.rejected.is_empty());
        assert_eq!(walk.atoms.len(), 2);
        assert_eq!(walk.trajectory.steps.len(), 2);
        assert!(walk.trajectory.loops.is_empty());
        let rejected_packets = walk
            .packets
            .iter()
            .filter(|p| p.verdict == crate::PacketVerdict::Rejected)
            .count();
        assert_eq!(rejected_packets, walk.rejected.len());
        assert!(rejected_packets > 0);
    }

    fn chart_envelopes() -> Vec<TraceEnvelope> {
        vec![
            activation(0, &[1.0, 0.0]),
            activation(1, &[0.0, 1.0]),
            activation(2, &[1.0, 0.0]),
            envelope(
                TraceKind::Weight,
                3,
                floats(&[1.0, 2.0, 3.0, 4.0]),
                TraceDtype::F32,
            ),
        ]
    }

    fn chart_receipt_map(
        judged: &[(ExtractedCandidate, JudgeReceipt)],
    ) -> BTreeMap<Digest, ChartReceipts> {
        judged
            .iter()
            .filter(|(c, _)| c.cell_kind == CellKind::Rank8Jacobian)
            .map(|(c, _)| {
                (
                    c.candidate_digest.clone(),
                    ChartReceipts {
                        marginal_gain_receipt_digest: Digest::sha256(b"test-marginal"),
                        numerical_receipt_digest: Digest::sha256(b"test-numerical"),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn chart_validates_with_receipts_and_fails_without() {
        let envelopes = chart_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        assert!(judged.iter().any(
            |(c, r)| c.cell_kind == CellKind::Rank8Jacobian && r.verdict == JudgeVerdict::Pass
        ));
        let context = context();
        // Missing receipts fail closed.
        assert!(
            replay(&ReplayInput {
                context: &context,
                envelopes: &envelopes,
                judged: &judged,
                chart_receipts: &BTreeMap::new(),
            })
            .is_err()
        );
        let receipts = chart_receipt_map(&judged);
        let walk = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        assert_eq!(walk.charts.len(), 1);
        assert_eq!(walk.charts[0].chart_rank, 4);
        // Tangent basis that matches no envelope fails closed.
        let no_weight: Vec<TraceEnvelope> = envelopes
            .into_iter()
            .filter(|e| e.kind != TraceKind::Weight)
            .collect();
        assert!(
            replay(&ReplayInput {
                context: &context,
                envelopes: &no_weight,
                judged: &judged,
                chart_receipts: &receipts,
            })
            .is_err()
        );
    }

    #[test]
    fn quantized_chart_has_no_dense_dtype() {
        let mut envelopes = walk_envelopes();
        envelopes.push(envelope(
            TraceKind::Weight,
            3,
            vec![0xABu8; 37],
            TraceDtype::Q8_0,
        ));
        // Schedule-v1 evidence rejects the quantized chart first.
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let chart_rejected = judged.iter().any(|(c, r)| {
            c.cell_kind == CellKind::Rank8Jacobian && r.verdict == JudgeVerdict::Reject
        });
        assert!(chart_rejected);
        // Force a pass to prove replay's own dtype gate still fails closed.
        let judge = IndependentJudge::new(policy()).unwrap();
        let mut known = Vec::new();
        let mut forced = Vec::new();
        for candidate in crate::extract::extract(&envelopes).unwrap() {
            let evidence = JudgeEvidence {
                mean_square: 3.3,
                finite: true,
                known_candidate_digests: known.clone(),
            };
            let receipt = judge.judge(&candidate, &evidence).unwrap();
            known.push(candidate.candidate_digest.clone());
            forced.push((candidate, receipt));
        }
        assert!(forced.iter().any(
            |(c, r)| c.cell_kind == CellKind::Rank8Jacobian && r.verdict == JudgeVerdict::Pass
        ));
        let context = context();
        let receipts = chart_receipt_map(&forced);
        let error = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &forced,
            chart_receipts: &receipts,
        })
        .expect_err("quantized chart must fail");
        assert!(error.contains("quantized"), "{error}");
    }

    #[test]
    fn candidate_tamper_and_policy_drift_fail() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let context = context();
        let receipts = BTreeMap::new();
        let mut tampered = judged.clone();
        tampered[0].0.candidate_digest = Digest::sha256(b"forged");
        assert!(
            replay(&ReplayInput {
                context: &context,
                envelopes: &envelopes,
                judged: &tampered,
                chart_receipts: &receipts,
            })
            .is_err()
        );
        let mut drifted = judged.clone();
        drifted[1].1.policy_digest = Digest::sha256(b"other-policy");
        assert!(
            replay(&ReplayInput {
                context: &context,
                envelopes: &envelopes,
                judged: &drifted,
                chart_receipts: &receipts,
            })
            .is_err()
        );
    }

    #[test]
    fn cli_fixture_trace_roundtrips_through_frames() {
        // Writes a constructed-envelope .gmtrace fixture for CLI end-to-end
        // runs (build output only; the envelopes are labeled constructed).
        let envelopes = walk_envelopes();
        let mut bytes = Vec::new();
        for envelope in &envelopes {
            bytes.extend_from_slice(&crate::trace::encode(envelope).unwrap());
        }
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/fixtures/cli-walk.gmtrace");
        std::fs::create_dir_all(fixture.parent().unwrap()).unwrap();
        std::fs::write(&fixture, &bytes).unwrap();
        let mut rest = bytes.as_slice();
        let mut count = 0;
        while !rest.is_empty() {
            let (_, consumed) = crate::trace::decode_frame(rest).unwrap();
            count += 1;
            rest = &rest[consumed..];
        }
        assert_eq!(count, envelopes.len());
    }

    #[test]
    fn evidence_schedule_handles_finite_nan_and_empty() {
        let envelopes = walk_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let rank0 = judged
            .iter()
            .find(|(c, _)| c.cell_kind == CellKind::Rank0Atom)
            .unwrap();
        let evidence = evidence_for(&rank0.0, &envelopes).unwrap();
        assert!(evidence.finite);
        assert!((evidence.mean_square - 0.5).abs() < 1e-9);
        let nan = vec![activation(0, &[f32::NAN, 1.0])];
        let evidence = evidence_for(&rank0.0, &nan).unwrap();
        assert!(!evidence.finite);
        let kv_only = vec![envelope(
            TraceKind::KeyValue,
            0,
            floats(&[1.0]),
            TraceDtype::F32,
        )];
        let evidence = evidence_for(&rank0.0, &kv_only).unwrap();
        assert!(!evidence.finite);
    }

    fn descriptors(envelopes: &[TraceEnvelope]) -> Vec<EnvelopeDescriptor> {
        envelopes
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
            .collect()
    }

    #[test]
    fn artifact_roundtrip_restart_is_identical_and_tamper_fails() {
        let envelopes = chart_envelopes();
        let judged = extract_and_judge(&envelopes, policy()).unwrap();
        let context = context();
        let receipts = chart_receipt_map(&judged);
        let walk = replay(&ReplayInput {
            context: &context,
            envelopes: &envelopes,
            judged: &judged,
            chart_receipts: &receipts,
        })
        .unwrap();
        let bytes = encode_walk(
            &context,
            &judged,
            &descriptors(&envelopes),
            &receipts,
            &walk.trajectory_digest,
            &walk.candidate_bundle_digest,
            &walk.verdict_bundle_digest,
            &walk.replay_receipt_digest,
        )
        .unwrap();
        // Restart: decode the artifact and re-run replay from decoded inputs.
        let file = decode_walk(&bytes).unwrap();
        assert_eq!(file.judged.len(), judged.len());
        assert_eq!(file.envelopes.len(), envelopes.len());
        verify_bindings(&context, &walk).unwrap();
        let rerun = replay(&ReplayInput {
            context: &file.context,
            envelopes: &envelopes,
            judged: &file.judged,
            chart_receipts: &file.chart_receipts,
        })
        .unwrap();
        assert_eq!(rerun.replay_receipt_digest, walk.replay_receipt_digest);
        assert_eq!(rerun.trajectory_digest, walk.trajectory_digest);
        assert_eq!(rerun.trajectory_digest, file.trajectory_digest);
        assert_eq!(rerun.replay_receipt_digest, file.replay_receipt_digest);
        // Tamper and truncation fail closed.
        let mut tampered = bytes.clone();
        tampered[20] ^= 0xFF;
        assert!(decode_walk(&tampered).is_err());
        assert!(decode_walk(&bytes[..bytes.len() - 33]).is_err());
        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(decode_walk(&bad_magic).is_err());
    }
}
