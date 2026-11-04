//! Fresh GlobalMind admission runtime.
//!
//! This crate owns typed distillation, trajectory, prediction-error, and
//! SurrealDB admission contracts. It does not import the deleted CoreLM or the
//! legacy text-ingest MCP implementation.

pub mod capability;
pub mod extract;
mod holding;
mod holding_session;
mod identity;
pub mod inject;
pub mod judge;
mod kv_identity;
mod manifest;
mod model;
pub mod replay;
pub mod retrieval;
mod store;
mod tppe;
pub mod trace;
pub mod universal_decoder;
pub mod whitehead;

pub use holding::{
    BranchUpdate, HoldingController, HoldingDecision, HoldingPolicy, HoldingSnapshot,
};
pub use holding_session::{
    branch, classify_trigger, controller_digest, BridgeRoute, HoldingSession, RecallSignals,
    SessionOutcome, CONTROLLER_VERSION,
};
pub use identity::{Digest, RecordId};
pub use kv_identity::ExactKvIdentity;
pub use manifest::{
    ArtifactEntry, Manifest, ModelEntry, VerifiedArtifact, VerifiedManifest, VerifiedModel,
    PROJECTOR_VERSION,
};
pub use model::{
    BranchSource, CellKind, ChartDtype, ContentKind, CrystallizedAtom, DistillationPacket,
    DistillationPhase, DistillationRunEvent, EdgeKind, ForwardMode, LocalJacobianChart,
    PacketVerdict, PredictionErrorObservation, Resolution, ResolutionOutcome, SourceKind,
    SuperpositionBranch, SuperpositionHold, SuperpositionIteration, SuperpositionResolution,
    SuperpositionTrigger, TensorDtype, TensorLayout, TensorPage, TensorPayload, TensorSession,
    Trajectory, TrajectoryLoop, TrajectoryOrigin, TrajectoryState, TrajectoryStep, Unit,
};
pub use store::{GlobalMindStore, StoreConfig};
pub use tppe::{RadialLaw, TppeConfig, TppeState};
pub use whitehead::{
    canonicalize, gamma12_generators, hyperbolic_distance, joint_well_cost, kuramoto_psi,
    lorentz_inner, lorentz_to_poincare, lorentz_to_uhs, poincare_to_lorentz, torus_distance_sq,
    uhs_to_lorentz, validate_lorentz_point, validate_t2_phase, CanonicalRepresentative,
    CognitiveHubSpec, CognitiveWellSpec, DualityPole, GaussianInt, NamedGamma12Generator,
    ProjectivePointMod2Plus2i, Psl2Zi, TopologyStatus, UhsCoord, WellClassification,
    WhiteheadTopology, BOUNDARY_TIE_EPSILON, DEFAULT_HUB_HORIZON_RADIUS,
    DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN, WHITEHEAD_CLASSIFIER_VERSION, WHITEHEAD_CONTRACT_VERSION,
    WHITEHEAD_COORDINATE_MODEL, WHITEHEAD_MANIFOLD, WHITEHEAD_QUOTIENT_SYMBOL,
    WHITEHEAD_SPACE_NAME,
};
