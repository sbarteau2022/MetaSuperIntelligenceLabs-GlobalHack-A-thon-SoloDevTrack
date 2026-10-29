pub mod active_inference;
pub mod bench;
pub mod cache;
pub mod calibration;
pub mod cordis;
pub mod cpp_bridge;
pub mod distill_judge;
pub mod distillation;
pub mod energy;
pub mod engine;
pub mod evaluator;
pub mod evolution;
pub mod fep;
pub mod gate;
pub mod hear;
pub mod judge;
pub mod lanes;
pub mod mode;
pub mod mission;
pub mod registry;
pub mod plugins;
pub mod policy_kernel;
pub mod rag;
pub mod receipt;
pub mod recursive;
pub mod register;
pub mod rsi;
pub mod runbook;
pub mod session;
pub mod superposition;
pub mod telemetry;
pub mod temple;

pub use active_inference::{BeliefState, PolicyCandidate, PolicyRanker, RankedPolicy};
pub use bench::{BaselineRun, BenchmarkResult, HarnessBaseline, Task};
pub use cache::Cache;
pub use calibration::{
    HyperMapIngest, QualityReading, QualitySource, TwinResult, WattCalibration,
    twin_fingerprint_check,
};
pub use cordis::{Coeffect, Component, HmrGate, ReversibleEffect};
pub use cpp_bridge::{bridge_version, shell_dist_batch, shell_dist_rs};
pub use distill_judge::{
    DistillationJudge, DistillationJudgeResult, DistillationJudgeVerdict, DistillationRun,
    RoleCounts,
};
pub use distillation::{CoshCoalescenceGate, DepthAllocator, LiveThreadedPass};
pub use energy::{EnergyBudget, EnergyLedger};
pub use engine::{Engine, EngineReport};
pub use evaluator::{
    EvalVerdict, GeneratedApp, LiveCheck, LiveEvaluator, PlannedSprint, SprintContract,
};
pub use evolution::{
    ParetoPoint, leakage_critic, noise_band_accept, pareto_frontier, prune_dominated,
    regularized_adopt,
};
pub use fep::{FreeEnergyTerms, SessionBound, precision_update};
pub use gate::{InformationNeed, ResearchRecord, dynamic_iteration_directive, research_gate};
pub use hear::{CoordinationDecision, EngineRuntimeState, HearCoordinator, WorkflowIntent};
pub use judge::{Judge, JudgeKind, JudgeOutcome, JudgeResult};
pub use lanes::{
    ContributorLane, ContributorLaneMuse, DualLaneRun, LaneVerdict, NativeLane, TaskPacket,
    TaskReceipt,
};
pub use mode::{Command, TacticalLevel, allowed_commands, authorize, raise_level};
pub use plugins::{PluginReceipt, PluginRegistry, PluginSpec};
pub use policy_kernel::FrozenPolicyKernel;
pub use rag::{MockMap, RagQuery, RealVector, rag_coverage};
pub use receipt::{Receipt, ReceiptLog};
pub use recursive::{
    IterationMandate, OptimizerScheduler, SPIKE_SIGMA, SubsystemScore, mandate_iteration,
    optimize_cycle, rank_subsystems,
};
pub use register::{HypothesisRegister, ProbePlan, rank_probes};
pub use rsi::{Bundle, BundleDelta, GovernedLibrary, LibraryEntry, ProposeProbeCompose};
pub use runbook::{JudgeRef, Packet, PacketStatus, Plan, ToolCall};
pub use session::{SessionEvent, SessionEventKind, SessionLog};
pub use superposition::{
    BranchSource, DeliberationBranch, DeliberationOutcome, HoldingState, SeparationBounds,
    boltzmann_weights, check_separation, deliberate, irecall_check, suppress_isotropic,
};
pub use telemetry::TelemetryFrame;
pub use temple::TempleRecord;
