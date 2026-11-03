use crate::{Digest, RecordId};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Text,
    Image,
    Audio,
    Video,
    Code,
    Mixed,
}

impl ContentKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Code => "code",
            Self::Mixed => "mixed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrystallizedAtom {
    pub id: RecordId,
    pub address_id: RecordId,
    pub origin_id: RecordId,
    pub crystallization_id: RecordId,
    pub membership_id: RecordId,
    pub workspace: RecordId,
    pub run: RecordId,
    pub packet: RecordId,
    pub admission: RecordId,
    pub geometry_space: RecordId,
    pub well: RecordId,
    pub external_id: String,
    pub content_kind: ContentKind,
    pub content_text: Option<String>,
    pub content_uri: Option<String>,
    pub content_digest: Digest,
    pub domain: String,
    pub salience: f64,
    pub provenance: String,
    pub taint: String,
    pub pinned: bool,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub geometry_model: String,
    pub geometry_version: String,
    pub address_revision: u32,
    pub source_step_digest: Digest,
    pub mint_receipt_digest: Digest,
    pub payload_digest: Digest,
    pub reconstruction_contract_digest: Digest,
    pub reconstruction_receipt_digest: Digest,
    pub crystallization_receipt_digest: Digest,
    pub classifier_version: String,
    pub classifier_digest: Digest,
    pub boundary_receipt_digest: Digest,
}

impl CrystallizedAtom {
    pub(crate) fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("external_id", self.external_id.as_str()),
            ("domain", self.domain.as_str()),
            ("provenance", self.provenance.as_str()),
            ("taint", self.taint.as_str()),
            ("geometry_model", self.geometry_model.as_str()),
            ("geometry_version", self.geometry_version.as_str()),
            ("classifier_version", self.classifier_version.as_str()),
        ] {
            require_finite_nonempty(field, value)?;
        }
        if self.content_text.is_none() && self.content_uri.is_none() {
            return Err("crystallized atom requires content_text or content_uri".into());
        }
        if !self.salience.is_finite() || !(0.0..=1.0).contains(&self.salience) {
            return Err("atom salience must be finite and in [0,1]".into());
        }
        if self.address_revision == 0 {
            return Err("atom address revision must be positive".into());
        }
        if !self
            .h3_lorentz
            .iter()
            .chain(&self.t2_phase)
            .all(|v| v.is_finite())
        {
            return Err("atom address coordinates must be finite".into());
        }
        if self.h3_lorentz[0] <= 0.0 {
            return Err("Lorentz time component must be positive".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceKind {
    FrozenWeight,
    WalkedTrajectory,
}

impl SourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::FrozenWeight => "frozen_weight",
            Self::WalkedTrajectory => "walked_trajectory",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellKind {
    Rank0Atom,
    Rank1Transition,
    Rank2Loop,
    Rank8Jacobian,
}

impl CellKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Rank0Atom => "rank0_atom",
            Self::Rank1Transition => "rank1_transition",
            Self::Rank2Loop => "rank2_loop",
            Self::Rank8Jacobian => "rank8_jacobian",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketVerdict {
    Admitted,
    Rejected,
}

impl PacketVerdict {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DistillationPacket {
    pub id: RecordId,
    pub workspace: RecordId,
    pub run: RecordId,
    pub admission: RecordId,
    pub source_kind: SourceKind,
    pub cell_kind: CellKind,
    pub source_fragment_digest: Digest,
    pub teacher_neutral_input_digest: Digest,
    pub projector_version: String,
    pub projector_digest: Digest,
    pub configuration_digest: Digest,
    pub candidate_digest: Digest,
    pub novelty_receipt_digest: Digest,
    pub judge_receipt_digest: Digest,
    pub verdict: PacketVerdict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeKind {
    Semantic,
    Causal,
    Temporal,
    Supports,
    Contradicts,
    PartOf,
    DerivedFrom,
    References,
    Next,
}

impl EdgeKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Causal => "causal",
            Self::Temporal => "temporal",
            Self::Supports => "supports",
            Self::Contradicts => "contradicts",
            Self::PartOf => "part_of",
            Self::DerivedFrom => "derived_from",
            Self::References => "references",
            Self::Next => "next",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrajectoryState {
    Candidate,
    ReplayPassed,
    Crystallized,
    Rejected,
}

impl TrajectoryState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::ReplayPassed => "replay_passed",
            Self::Crystallized => "crystallized",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrajectoryStep {
    pub id: RecordId,
    pub ordinal: u32,
    pub from_atom: RecordId,
    pub to_atom: RecordId,
    pub edge: RecordId,
    pub edge_crystallization_id: RecordId,
    pub packet: RecordId,
    pub admission: RecordId,
    pub transition_kind: EdgeKind,
    pub weight: f64,
    pub confidence: f64,
    pub evidence_atoms: Vec<RecordId>,
    pub evidence_digest: Digest,
    pub producer: String,
    pub revision: u32,
    pub t2_phase_before: [f64; 2],
    pub t2_phase_after: [f64; 2],
    pub tppe_radius_before: f64,
    pub tppe_radius_after: f64,
    pub tppe_revolutions_before: u64,
    pub tppe_revolutions_after: u64,
    pub tppe_boundary_margin_before: f64,
    pub tppe_boundary_margin_after: f64,
    pub action_digest: Digest,
    pub observation_digest: Digest,
    pub transition_digest: Digest,
    pub replay_receipt_digest: Digest,
    pub crystallization_receipt_digest: Digest,
}

#[derive(Clone, Debug)]
pub struct TrajectoryLoop {
    pub id: RecordId,
    pub packet: RecordId,
    pub admission: RecordId,
    pub first_step: u32,
    pub last_step: u32,
    pub loop_invariant_kind: String,
    pub loop_invariant_digest: Digest,
    pub closure_receipt_digest: Digest,
}

#[derive(Clone, Debug)]
pub struct TrajectoryOrigin {
    pub id: RecordId,
    pub admission: RecordId,
    pub source_walk_digest: Digest,
    pub receipt_digest: Digest,
}

#[derive(Clone, Debug)]
pub struct Trajectory {
    pub id: RecordId,
    pub workspace: RecordId,
    pub run: RecordId,
    pub atoms: Vec<RecordId>,
    pub trajectory_digest: Digest,
    pub judge_receipt_digest: Digest,
    pub tppe_version: String,
    pub tppe_configuration_digest: Digest,
    pub tppe_radial_law_digest: Digest,
    pub tppe_boundary_receipt_digest: Digest,
    pub address_contract_digest: Digest,
    pub replay_receipt_digest: Digest,
    pub crystallization_receipt_digest: Option<Digest>,
    pub state: TrajectoryState,
    pub steps: Vec<TrajectoryStep>,
    pub loops: Vec<TrajectoryLoop>,
    pub origins: Vec<TrajectoryOrigin>,
}

impl Trajectory {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.atoms.len() < 2 || self.steps.is_empty() {
            return Err("trajectory requires at least two atoms and one step".into());
        }
        if self.atoms.len() != self.steps.len() + 1 {
            return Err("trajectory atom count must equal step count plus one".into());
        }
        let mut edge_ids = BTreeSet::new();
        let mut edge_crystallization_ids = BTreeSet::new();
        let origin_admissions = self
            .origins
            .iter()
            .map(|origin| origin.admission.clone())
            .collect::<BTreeSet<_>>();
        if origin_admissions.len() != self.origins.len() {
            return Err("trajectory audit origins must have unique admissions".into());
        }
        for (index, step) in self.steps.iter().enumerate() {
            if step.ordinal as usize != index {
                return Err("trajectory step ordinals must be contiguous from zero".into());
            }
            if step.from_atom != self.atoms[index] || step.to_atom != self.atoms[index + 1] {
                return Err("trajectory step endpoints must match the ordered atom walk".into());
            }
            if !step
                .t2_phase_before
                .iter()
                .chain(&step.t2_phase_after)
                .all(|v| v.is_finite())
            {
                return Err("trajectory phases must be finite".into());
            }
            if index > 0 && self.steps[index - 1].t2_phase_after != step.t2_phase_before {
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
            if !step.weight.is_finite()
                || !(0.0..=1.0).contains(&step.weight)
                || !step.confidence.is_finite()
                || !(0.0..=1.0).contains(&step.confidence)
            {
                return Err("trajectory edge weight/confidence must be finite and in [0,1]".into());
            }
            if step.producer.trim().is_empty() || step.revision == 0 {
                return Err("trajectory edge producer and positive revision are required".into());
            }
            if !edge_ids.insert(step.edge.clone())
                || !edge_crystallization_ids.insert(step.edge_crystallization_id.clone())
            {
                return Err("trajectory edge identities must be unique".into());
            }
            if !origin_admissions.contains(&step.admission) {
                return Err("trajectory step admission is missing from audit origins".into());
            }
        }
        for loop_record in &self.loops {
            let first = loop_record.first_step as usize;
            let last = loop_record.last_step as usize;
            if first > last || last >= self.steps.len() {
                return Err("trajectory loop step range is invalid".into());
            }
            if self.atoms[first] != self.atoms[last + 1] {
                return Err("rank-2 trajectory loop must close on the same atom".into());
            }
            if loop_record.loop_invariant_kind.trim().is_empty() {
                return Err("trajectory loop invariant kind must be non-empty".into());
            }
            if !origin_admissions.contains(&loop_record.admission) {
                return Err("trajectory loop admission is missing from audit origins".into());
            }
        }
        if self.origins.is_empty() {
            return Err("trajectory requires at least one audit origin".into());
        }
        if self.state != TrajectoryState::Crystallized
            || self.crystallization_receipt_digest.is_none()
        {
            return Err("recall admission requires a crystallized trajectory receipt".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChartDtype {
    F16,
    Bf16,
    F32,
}

impl ChartDtype {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::F16 => "f16",
            Self::Bf16 => "bf16",
            Self::F32 => "f32",
        }
    }

    fn element_bytes(self) -> usize {
        match self {
            Self::F16 | Self::Bf16 => 2,
            Self::F32 => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalJacobianChart {
    pub id: RecordId,
    pub workspace: RecordId,
    pub run: RecordId,
    pub anchor: RecordId,
    pub packet: RecordId,
    pub admission: RecordId,
    pub chart_rank: u8,
    pub dtype: ChartDtype,
    pub tensor_shape: Vec<u32>,
    pub tangent_basis: Box<[u8]>,
    pub tensor_digest: Digest,
    pub chart_digest: Digest,
    pub marginal_gain_receipt_digest: Digest,
    pub numerical_receipt_digest: Digest,
}

impl LocalJacobianChart {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.chart_rank) || self.tensor_shape.is_empty() {
            return Err("local Jacobian chart requires rank 1..=8 and a shape".into());
        }
        let elements = self
            .tensor_shape
            .iter()
            .try_fold(1usize, |product, dimension| {
                if *dimension == 0 {
                    return Err("local Jacobian tensor dimensions must be positive".to_string());
                }
                product
                    .checked_mul(*dimension as usize)
                    .ok_or_else(|| "local Jacobian tensor shape overflows usize".to_string())
            })?;
        let expected_bytes = elements
            .checked_mul(self.dtype.element_bytes())
            .ok_or_else(|| "local Jacobian byte length overflows usize".to_string())?;
        if expected_bytes != self.tangent_basis.len() {
            return Err("local Jacobian tensor byte length does not match shape/dtype".into());
        }
        if Digest::sha256(&self.tangent_basis) != self.tensor_digest {
            return Err("local Jacobian tensor digest mismatch".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistillationPhase {
    Opened,
    Extracting,
    Judging,
    Replaying,
    Completed,
    Failed,
    Rejected,
}

impl DistillationPhase {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::Extracting => "extracting",
            Self::Judging => "judging",
            Self::Replaying => "replaying",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }

    pub(crate) fn can_follow(self, prior: Self) -> bool {
        matches!(
            (prior, self),
            (
                Self::Opened,
                Self::Extracting | Self::Failed | Self::Rejected
            ) | (
                Self::Extracting,
                Self::Judging | Self::Failed | Self::Rejected
            ) | (
                Self::Judging,
                Self::Replaying | Self::Failed | Self::Rejected
            ) | (
                Self::Replaying,
                Self::Completed | Self::Failed | Self::Rejected
            )
        )
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "opened" => Ok(Self::Opened),
            "extracting" => Ok(Self::Extracting),
            "judging" => Ok(Self::Judging),
            "replaying" => Ok(Self::Replaying),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "rejected" => Ok(Self::Rejected),
            _ => Err(format!("unknown distillation phase: {value}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DistillationRunEvent {
    pub id: RecordId,
    pub workspace: RecordId,
    pub run: RecordId,
    pub ordinal: u32,
    pub phase: DistillationPhase,
    pub input_digest: Digest,
    pub output_digest: Option<Digest>,
    pub tppe_version: String,
    pub tppe_configuration_digest: Digest,
    pub receipt_digest: Digest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unit {
    Nats,
    Bits,
}

impl Unit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Nats => "nats",
            Self::Bits => "bits",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resolution {
    Reuse,
    DeeperRetrieval,
    TrajectoryExtend,
    FullForward,
    Quarantine,
}

impl Resolution {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Reuse => "reuse",
            Self::DeeperRetrieval => "deeper_retrieval",
            Self::TrajectoryExtend => "trajectory_extend",
            Self::FullForward => "full_forward",
            Self::Quarantine => "quarantine",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForwardMode {
    RecallOnly,
    AdaptiveForward,
    FullForward,
}

#[derive(Clone, Debug)]
pub struct PredictionErrorObservation {
    pub id: RecordId,
    pub workspace: RecordId,
    pub decision: RecordId,
    pub hub: RecordId,
    pub well: Option<RecordId>,
    pub trajectory: Option<RecordId>,
    pub prior_digest: Digest,
    pub observation_digest: Digest,
    pub estimator_version: String,
    pub estimator_digest: Digest,
    pub prediction_error: f64,
    pub precision: f64,
    pub precision_weighted_error: f64,
    pub surprisal: f64,
    pub unit: Unit,
    pub bayesian_surprise: Option<f64>,
    pub resolution: Resolution,
    pub receipt_digest: Digest,
}

impl PredictionErrorObservation {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let scalars = [
            self.prediction_error,
            self.precision,
            self.precision_weighted_error,
            self.surprisal,
        ];
        if !scalars.into_iter().all(f64::is_finite)
            || self
                .bayesian_surprise
                .is_some_and(|value| !value.is_finite())
        {
            return Err("prediction-error measurements must be finite".into());
        }
        if self.precision <= 0.0 || self.surprisal < 0.0 {
            return Err("precision must be positive and surprisal nonnegative".into());
        }
        if self.bayesian_surprise.is_some_and(|value| value < 0.0) {
            return Err("Bayesian surprise must be nonnegative when present".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuperpositionTrigger {
    NoDirectTrajectory,
    FixedPointOrCycle,
    BranchAmbiguity,
    HighPredictionError,
    LowReadiness,
    ComplianceBoundary,
}

impl SuperpositionTrigger {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NoDirectTrajectory => "no_direct_trajectory",
            Self::FixedPointOrCycle => "fixed_point_or_cycle",
            Self::BranchAmbiguity => "branch_ambiguity",
            Self::HighPredictionError => "high_prediction_error",
            Self::LowReadiness => "low_readiness",
            Self::ComplianceBoundary => "compliance_boundary",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchSource {
    DirectTrajectory,
    PhaseNeighbor,
    OperatorBridge,
    CrossWellBridge,
    FreshDerivation,
}

impl BranchSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DirectTrajectory => "direct_trajectory",
            Self::PhaseNeighbor => "phase_neighbor",
            Self::OperatorBridge => "operator_bridge",
            Self::CrossWellBridge => "cross_well_bridge",
            Self::FreshDerivation => "fresh_derivation",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SuperpositionBranch {
    pub id: RecordId,
    pub ordinal: u8,
    pub source: BranchSource,
    pub source_trajectory: Option<RecordId>,
    pub from_atom: RecordId,
    pub candidate_atom: RecordId,
    pub prediction_digest: Digest,
    pub scorer_version: String,
    pub scorer_digest: Digest,
    pub free_energy_value: f64,
    pub precision_weighted_error: f64,
    pub readiness: f64,
    pub compliance_permitted: bool,
    pub score_receipt_digest: Digest,
    pub compliance_receipt_digest: Digest,
}

#[derive(Clone, Debug)]
pub struct SuperpositionHold {
    pub id: RecordId,
    pub workspace: RecordId,
    pub decision: RecordId,
    pub trigger_observation: Option<RecordId>,
    pub hub: RecordId,
    pub well: Option<RecordId>,
    pub request_digest: Digest,
    pub prefix_digest: Digest,
    pub trigger: SuperpositionTrigger,
    pub controller_version: String,
    pub controller_digest: Digest,
    pub iteration_limit: u32,
    pub branches: Vec<SuperpositionBranch>,
}

impl SuperpositionHold {
    pub(crate) fn validate(&self) -> Result<(), String> {
        require_finite_nonempty("controller_version", &self.controller_version)?;
        if !(2..=12).contains(&self.branches.len()) {
            return Err("superposition hold requires 2..=12 branches".into());
        }
        if self.iteration_limit == 0 {
            return Err("superposition hold requires a positive iteration limit".into());
        }
        for (index, branch) in self.branches.iter().enumerate() {
            if branch.ordinal as usize != index {
                return Err("superposition branch ordinals must be contiguous from zero".into());
            }
            require_finite_nonempty("scorer_version", &branch.scorer_version)?;
            if ![
                branch.free_energy_value,
                branch.precision_weighted_error,
                branch.readiness,
            ]
            .into_iter()
            .all(f64::is_finite)
            {
                return Err("superposition branch measurements must be finite".into());
            }
            if !(0.0..=1.0).contains(&branch.readiness) {
                return Err("superposition readiness must be in [0,1]".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SuperpositionIteration {
    pub id: RecordId,
    pub workspace: RecordId,
    pub hold: RecordId,
    pub ordinal: u32,
    pub input_branch_digest: Digest,
    pub surviving_branches: Vec<RecordId>,
    pub rollout_receipt_digest: Digest,
    pub loop_receipt_digest: Option<Digest>,
    pub suppression_receipt_digest: Digest,
    pub output_branch_digest: Digest,
}

impl SuperpositionIteration {
    pub(crate) fn validate(&self) -> Result<(), String> {
        // An empty survivor set is the legitimate terminal state of a
        // fail-closed iteration (every branch suppressed or non-compliant);
        // the schema likewise places no minimum on the array. Non-terminal
        // iterations always carry at least one survivor by construction.
        if self.surviving_branches.len() > 12 {
            return Err("superposition iteration requires at most 12 surviving branches".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome {
    Collapsed,
    Iterate,
    FailClosed,
}

impl ResolutionOutcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Collapsed => "collapsed",
            Self::Iterate => "iterate",
            Self::FailClosed => "fail_closed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SuperpositionResolution {
    pub id: RecordId,
    pub workspace: RecordId,
    pub hold: RecordId,
    pub outcome: ResolutionOutcome,
    pub selected_branch: Option<RecordId>,
    pub resulting_trajectory: Option<RecordId>,
    pub resulting_atom: Option<RecordId>,
    pub judge_receipt_digest: Digest,
    pub replay_receipt_digest: Option<Digest>,
    pub crystallization_receipt_digest: Option<Digest>,
    pub resolution_receipt_digest: Digest,
}

impl SuperpositionResolution {
    pub(crate) fn validate(&self) -> Result<(), String> {
        match self.outcome {
            ResolutionOutcome::Collapsed if self.selected_branch.is_none() => {
                Err("collapsed superposition requires a selected branch".into())
            }
            ResolutionOutcome::Iterate | ResolutionOutcome::FailClosed
                if self.selected_branch.is_some()
                    || self.resulting_trajectory.is_some()
                    || self.resulting_atom.is_some() =>
            {
                Err("non-collapsed resolution cannot claim a selected or resulting record".into())
            }
            _ => Ok(()),
        }
    }
}

fn require_finite_nonempty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must be non-empty"))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TensorDtype {
    F16,
    Bf16,
    F32,
    Q8_0,
    Q4_0,
}

impl TensorDtype {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::F16 => "f16",
            Self::Bf16 => "bf16",
            Self::F32 => "f32",
            Self::Q8_0 => "q8_0",
            Self::Q4_0 => "q4_0",
        }
    }

    fn scalar_bytes(self) -> Option<usize> {
        match self {
            Self::F16 | Self::Bf16 => Some(2),
            Self::F32 => Some(4),
            Self::Q8_0 | Self::Q4_0 => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TensorLayout {
    TokenMajor,
    HeadDimMajor,
}

impl TensorLayout {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::TokenMajor => "token_major",
            Self::HeadDimMajor => "head_dim_major",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TensorSession {
    pub id: RecordId,
    pub workspace: RecordId,
    pub model_id: String,
    pub model_revision: String,
    pub model_digest: Digest,
    pub request_digest: Digest,
    pub prefix_digest: Digest,
    pub dtype: TensorDtype,
    pub layer_count: u32,
    pub kv_head_count: u32,
    pub head_dim: u32,
    pub geometry_space: RecordId,
    pub tppe_version: String,
    pub tppe_configuration_digest: Digest,
    pub address_contract_digest: Digest,
    pub tensor_contract_digest: Digest,
}

impl TensorSession {
    pub(crate) fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("model_id", self.model_id.as_str()),
            ("model_revision", self.model_revision.as_str()),
            ("tppe_version", self.tppe_version.as_str()),
        ] {
            require_finite_nonempty(field, value)?;
        }
        if self.layer_count == 0 || self.kv_head_count == 0 || self.head_dim == 0 {
            return Err("tensor session dimensions must be positive".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TensorPage {
    pub id: RecordId,
    pub workspace: RecordId,
    pub session: RecordId,
    pub layer: u32,
    pub kv_head: u32,
    pub token_start: u64,
    pub token_end: u64,
    pub tensor_shape: [u32; 2],
    pub dtype: TensorDtype,
    pub layout: TensorLayout,
    pub key_tensor: Box<[u8]>,
    pub value_tensor: Box<[u8]>,
    pub key_digest: Digest,
    pub value_digest: Digest,
    pub address: RecordId,
    pub trajectory_atoms: Vec<RecordId>,
    pub well: Option<RecordId>,
    pub trajectory: Option<RecordId>,
    pub phase_state_digest: Digest,
    pub receipt_digest: Digest,
    pub revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TensorPayload {
    pub key: Box<[u8]>,
    pub value: Box<[u8]>,
}

impl TensorPage {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.token_end <= self.token_start {
            return Err("tensor page token interval must be non-empty".into());
        }
        if self.tensor_shape[0] == 0 || self.tensor_shape[1] == 0 {
            return Err("tensor page shape dimensions must be positive".into());
        }
        if self.key_tensor.is_empty() || self.value_tensor.is_empty() {
            return Err("tensor page K and V payloads must be non-empty".into());
        }
        if self.key_digest != Digest::sha256(&self.key_tensor)
            || self.value_digest != Digest::sha256(&self.value_tensor)
        {
            return Err("tensor page K/V digest mismatch".into());
        }
        if self.revision == 0 {
            return Err("tensor page revision must be positive".into());
        }
        if let Some(scalar_bytes) = self.dtype.scalar_bytes() {
            let scalar_count = (self.tensor_shape[0] as usize)
                .checked_mul(self.tensor_shape[1] as usize)
                .ok_or_else(|| "tensor page shape overflow".to_string())?;
            let expected_bytes = scalar_count
                .checked_mul(scalar_bytes)
                .ok_or_else(|| "tensor page byte count overflow".to_string())?;
            if self.key_tensor.len() != expected_bytes || self.value_tensor.len() != expected_bytes
            {
                return Err("dense tensor payload size does not match shape and dtype".into());
            }
        }
        Ok(())
    }
}
