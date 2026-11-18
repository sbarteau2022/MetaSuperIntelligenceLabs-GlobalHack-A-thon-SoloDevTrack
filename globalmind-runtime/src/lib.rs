//! Fresh GlobalMind admission runtime.
//!
//! This crate owns typed distillation, trajectory, prediction-error, and
//! SurrealDB admission contracts. It does not import the deleted CoreLM or the
//! legacy text-ingest MCP implementation.

pub mod capability;
pub mod exact_attention;
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
pub mod shared_knowledge;
pub mod retrieval;
mod store;
mod tppe;
pub mod trace;
pub mod universal_decoder;
pub mod whitehead;
pub mod position_transform;
pub mod model_loader;

pub use holding::{
    BranchUpdate, HoldingController, HoldingDecision, HoldingPolicy, HoldingSnapshot,
};
pub use holding_session::{
    BridgeRoute, CONTROLLER_VERSION, HoldingSession, RecallSignals, SessionOutcome, branch,
    classify_trigger, controller_digest,
};
pub use identity::{Digest, RecordId};
pub use kv_identity::ExactKvIdentity;
pub use manifest::{
    ArtifactEntry, Manifest, ModelEntry, PROJECTOR_VERSION, VerifiedArtifact, VerifiedManifest,
    VerifiedModel,
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
    BOUNDARY_TIE_EPSILON, CanonicalRepresentative, CognitiveHubSpec, CognitiveWellSpec,
    DEFAULT_HUB_HORIZON_RADIUS, DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN, DualityPole, GaussianInt,
    NamedGamma12Generator, ProjectivePointMod2Plus2i, Psl2Zi, TopologyStatus, UhsCoord,
    WHITEHEAD_CLASSIFIER_VERSION, WHITEHEAD_CONTRACT_VERSION, WHITEHEAD_COORDINATE_MODEL,
    WHITEHEAD_MANIFOLD, WHITEHEAD_QUOTIENT_SYMBOL, WHITEHEAD_SPACE_NAME, WellClassification,
    WhiteheadTopology, canonicalize, gamma12_generators, hyperbolic_distance, joint_well_cost,
    kuramoto_psi, lorentz_inner, lorentz_to_poincare, lorentz_to_uhs, poincare_to_lorentz,
    torus_distance_sq, uhs_to_lorentz, validate_lorentz_point, validate_t2_phase,
};
pub use position_transform::{
    LayerAttentionType, RopeConfig, RopeFrequencies, NoPeTransform, PositionTransform,
    GlimmerLayerConfig, build_glimmer_layer_configs, apply_position_transform_layer,
    apply_position_transform_layer_windowed,
};
pub use model_loader::{
    GLIMMER_HIDDEN_SIZE, GLIMMER_NUM_LAYERS, GLIMMER_NUM_Q_HEADS, GLIMMER_NUM_KV_HEADS,
    GLIMMER_HEAD_DIM, GLIMMER_VOCAB_SIZE, GLIMMER_INTERMEDIATE_SIZE,
    GLIMMER_QUANT_GROUP_SIZE, GLIMMER_RMS_NORM_EPS, GLIMMER_ROPE_THETA,
    GLIMMER_PARTIAL_ROTARY_FACTOR, GLIMMER_SLIDING_WINDOW,
    AffineQuantizedMatrix, DequantizedMatrix, RmsNorm, EmbeddingWeight, OutputProjection,
    GlimmerModel, GlimmerLayerWeights, load_glimmer_model, dequantize_matrix, project_dequantized,
    rms_norm,
    ForwardOutput, FullKVCache, LayerKVCache,
};
