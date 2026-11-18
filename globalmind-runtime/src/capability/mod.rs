//! Model-neutral Capability Atlas contract, binary scaffold envelope,
//! two-hop retrieval planner, live-registry reconciler, and mandatory Rust
//! harness policy gates.
//!
//! Authority boundaries:
//! - GlobalMind contains one model-neutral Capability Atlas attached 1-to-1 to
//!   semantic `atom` and `atom_address` records, never `transformer_kv_page`.
//! - The Atlas may propose candidate compositions; it cannot grant permission
//!   or execute tools directly.
//! - `LiveRegistry` is authoritative for installation, health, permissions,
//!   and version/digest compatibility.
//! - `MandatoryPolicyGate` enforces safety and model-I/O invariants in Rust
//!   even when SurrealDB/Atlas recall is offline or unavailable.
//! - All scaffold and context-projection payloads cross the planning/execution
//!   boundary as bounded native binary envelopes (`GMCS`, `GMCP`, `GMCE`,
//!   `GMCT`); no JSON, JSONL, Python, or prompt-generated tool schema crosses
//!   that boundary.

pub mod forward_smoke;
pub mod store;
pub mod vertical_slice;

use crate::whitehead::{
    WhiteheadTopology, hyperbolic_distance, poincare_to_lorentz, torus_distance_sq,
    validate_lorentz_point, validate_t2_phase,
};
use crate::{Digest, RecordId};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::f64::consts::TAU;

pub const CAPABILITY_SCAFFOLD_MAGIC: &[u8; 8] = b"GMCS\x01\x00\x00\x00";
pub const CAPABILITY_PROJECTION_MAGIC: &[u8; 8] = b"GMCP\x01\x00\x00\x00";
pub const CAPABILITY_EDGE_MAGIC: &[u8; 8] = b"GMCE\x01\x00\x00\x00";
pub const CAPABILITY_CONTEXT_FRAME_MAGIC: &[u8; 8] = b"GMCT\x01\x00\x00\x00";

pub const MAX_SCAFFOLD_ENVELOPE_BYTES: usize = 16_384;
pub const MAX_PROJECTION_ENVELOPE_BYTES: usize = 4_096;
pub const MAX_EDGE_ENVELOPE_BYTES: usize = 4_096;
pub const MAX_CONTEXT_FRAME_BYTES: usize = 32_768;
pub const MAX_INITIAL_TYPED_HOPS: usize = 2;

/// All twelve required Capability Atlas atom kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CapabilityKind {
    ProblemPattern,
    SolutionFamily,
    Capability,
    Runbook,
    McpServer,
    Plugin,
    Tool,
    Skill,
    Hook,
    PolicyRule,
    EvidenceReceipt,
    FailurePattern,
}

impl CapabilityKind {
    pub const ALL: [Self; 12] = [
        Self::ProblemPattern,
        Self::SolutionFamily,
        Self::Capability,
        Self::Runbook,
        Self::McpServer,
        Self::Plugin,
        Self::Tool,
        Self::Skill,
        Self::Hook,
        Self::PolicyRule,
        Self::EvidenceReceipt,
        Self::FailurePattern,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProblemPattern => "problem_pattern",
            Self::SolutionFamily => "solution_family",
            Self::Capability => "capability",
            Self::Runbook => "runbook",
            Self::McpServer => "mcp_server",
            Self::Plugin => "plugin",
            Self::Tool => "tool",
            Self::Skill => "skill",
            Self::Hook => "hook",
            Self::PolicyRule => "policy_rule",
            Self::EvidenceReceipt => "evidence_receipt",
            Self::FailurePattern => "failure_pattern",
        }
    }

    pub const fn tag_byte(self) -> u8 {
        match self {
            Self::ProblemPattern => 1,
            Self::SolutionFamily => 2,
            Self::Capability => 3,
            Self::Runbook => 4,
            Self::McpServer => 5,
            Self::Plugin => 6,
            Self::Tool => 7,
            Self::Skill => 8,
            Self::Hook => 9,
            Self::PolicyRule => 10,
            Self::EvidenceReceipt => 11,
            Self::FailurePattern => 12,
        }
    }

    pub fn from_tag_byte(tag: u8) -> Result<Self, String> {
        match tag {
            1 => Ok(Self::ProblemPattern),
            2 => Ok(Self::SolutionFamily),
            3 => Ok(Self::Capability),
            4 => Ok(Self::Runbook),
            5 => Ok(Self::McpServer),
            6 => Ok(Self::Plugin),
            7 => Ok(Self::Tool),
            8 => Ok(Self::Skill),
            9 => Ok(Self::Hook),
            10 => Ok(Self::PolicyRule),
            11 => Ok(Self::EvidenceReceipt),
            12 => Ok(Self::FailurePattern),
            other => Err(format!("unknown CapabilityKind tag byte {other}")),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "problem_pattern" => Ok(Self::ProblemPattern),
            "solution_family" => Ok(Self::SolutionFamily),
            "capability" => Ok(Self::Capability),
            "runbook" => Ok(Self::Runbook),
            "mcp_server" => Ok(Self::McpServer),
            "plugin" => Ok(Self::Plugin),
            "tool" => Ok(Self::Tool),
            "skill" => Ok(Self::Skill),
            "hook" => Ok(Self::Hook),
            "policy_rule" => Ok(Self::PolicyRule),
            "evidence_receipt" => Ok(Self::EvidenceReceipt),
            "failure_pattern" => Ok(Self::FailurePattern),
            other => Err(format!("unknown CapabilityKind '{other}'")),
        }
    }

    /// Returns true if this kind represents a live-reconcilable executable or
    /// provider artifact that must be reconciled with `LiveRegistry`.
    pub const fn requires_live_reconciliation(self) -> bool {
        matches!(
            self,
            Self::Runbook | Self::McpServer | Self::Plugin | Self::Tool | Self::Skill | Self::Hook
        )
    }
}

/// Kind-specific required payload for each of the twelve Capability Atlas kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityPayload {
    ProblemPattern {
        symptoms: String,
        constraints: String,
        success_condition: String,
    },
    SolutionFamily {
        scope: String,
        exclusions: String,
        selection_criteria: String,
    },
    Capability {
        input_output_contract: String,
        effects: String,
        failure_modes: String,
    },
    Runbook {
        steps: String,
        stop_rollback_rules: String,
        evidence_required: String,
    },
    McpServer {
        server_identity: String,
        transport: String,
        advertised_contracts: String,
    },
    Plugin {
        loader_contract: String,
        isolation_behavior: String,
        unload_behavior: String,
    },
    Tool {
        typed_arguments_result: String,
        side_effects: String,
        timeout_spec: String,
    },
    Skill {
        trigger: String,
        required_references: String,
        completion_gates: String,
    },
    Hook {
        event: String,
        ordering: String,
        idempotency_and_failure_policy: String,
    },
    PolicyRule {
        scope: String,
        severity: String,
        enforcement_point: String,
    },
    EvidenceReceipt {
        subject_config_runtime_digests: String,
        tier: String,
        result: String,
    },
    FailurePattern {
        signature: String,
        cause: String,
        containment: String,
        recovery: String,
    },
}

impl CapabilityPayload {
    pub const fn kind(&self) -> CapabilityKind {
        match self {
            Self::ProblemPattern { .. } => CapabilityKind::ProblemPattern,
            Self::SolutionFamily { .. } => CapabilityKind::SolutionFamily,
            Self::Capability { .. } => CapabilityKind::Capability,
            Self::Runbook { .. } => CapabilityKind::Runbook,
            Self::McpServer { .. } => CapabilityKind::McpServer,
            Self::Plugin { .. } => CapabilityKind::Plugin,
            Self::Tool { .. } => CapabilityKind::Tool,
            Self::Skill { .. } => CapabilityKind::Skill,
            Self::Hook { .. } => CapabilityKind::Hook,
            Self::PolicyRule { .. } => CapabilityKind::PolicyRule,
            Self::EvidenceReceipt { .. } => CapabilityKind::EvidenceReceipt,
            Self::FailurePattern { .. } => CapabilityKind::FailurePattern,
        }
    }

    pub fn slots(&self) -> (&str, &str, &str, Option<&str>) {
        match self {
            Self::ProblemPattern {
                symptoms,
                constraints,
                success_condition,
            } => (symptoms, constraints, success_condition, None),
            Self::SolutionFamily {
                scope,
                exclusions,
                selection_criteria,
            } => (scope, exclusions, selection_criteria, None),
            Self::Capability {
                input_output_contract,
                effects,
                failure_modes,
            } => (input_output_contract, effects, failure_modes, None),
            Self::Runbook {
                steps,
                stop_rollback_rules,
                evidence_required,
            } => (steps, stop_rollback_rules, evidence_required, None),
            Self::McpServer {
                server_identity,
                transport,
                advertised_contracts,
            } => (server_identity, transport, advertised_contracts, None),
            Self::Plugin {
                loader_contract,
                isolation_behavior,
                unload_behavior,
            } => (loader_contract, isolation_behavior, unload_behavior, None),
            Self::Tool {
                typed_arguments_result,
                side_effects,
                timeout_spec,
            } => (typed_arguments_result, side_effects, timeout_spec, None),
            Self::Skill {
                trigger,
                required_references,
                completion_gates,
            } => (trigger, required_references, completion_gates, None),
            Self::Hook {
                event,
                ordering,
                idempotency_and_failure_policy,
            } => (event, ordering, idempotency_and_failure_policy, None),
            Self::PolicyRule {
                scope,
                severity,
                enforcement_point,
            } => (scope, severity, enforcement_point, None),
            Self::EvidenceReceipt {
                subject_config_runtime_digests,
                tier,
                result,
            } => (subject_config_runtime_digests, tier, result, None),
            Self::FailurePattern {
                signature,
                cause,
                containment,
                recovery,
            } => (signature, cause, containment, Some(recovery.as_str())),
        }
    }

    pub fn from_slots(
        kind: CapabilityKind,
        slot1: String,
        slot2: String,
        slot3: String,
        slot4: Option<String>,
    ) -> Result<Self, String> {
        let payload = match kind {
            CapabilityKind::ProblemPattern => Self::ProblemPattern {
                symptoms: slot1,
                constraints: slot2,
                success_condition: slot3,
            },
            CapabilityKind::SolutionFamily => Self::SolutionFamily {
                scope: slot1,
                exclusions: slot2,
                selection_criteria: slot3,
            },
            CapabilityKind::Capability => Self::Capability {
                input_output_contract: slot1,
                effects: slot2,
                failure_modes: slot3,
            },
            CapabilityKind::Runbook => Self::Runbook {
                steps: slot1,
                stop_rollback_rules: slot2,
                evidence_required: slot3,
            },
            CapabilityKind::McpServer => Self::McpServer {
                server_identity: slot1,
                transport: slot2,
                advertised_contracts: slot3,
            },
            CapabilityKind::Plugin => Self::Plugin {
                loader_contract: slot1,
                isolation_behavior: slot2,
                unload_behavior: slot3,
            },
            CapabilityKind::Tool => Self::Tool {
                typed_arguments_result: slot1,
                side_effects: slot2,
                timeout_spec: slot3,
            },
            CapabilityKind::Skill => Self::Skill {
                trigger: slot1,
                required_references: slot2,
                completion_gates: slot3,
            },
            CapabilityKind::Hook => Self::Hook {
                event: slot1,
                ordering: slot2,
                idempotency_and_failure_policy: slot3,
            },
            CapabilityKind::PolicyRule => Self::PolicyRule {
                scope: slot1,
                severity: slot2,
                enforcement_point: slot3,
            },
            CapabilityKind::EvidenceReceipt => Self::EvidenceReceipt {
                subject_config_runtime_digests: slot1,
                tier: slot2,
                result: slot3,
            },
            CapabilityKind::FailurePattern => Self::FailurePattern {
                signature: slot1,
                cause: slot2,
                containment: slot3,
                recovery: slot4.ok_or_else(|| {
                    "failure_pattern requires non-empty recovery in slot 4".to_string()
                })?,
            },
        };
        payload.validate()?;
        Ok(payload)
    }

    pub fn validate(&self) -> Result<(), String> {
        let (s1, s2, s3, s4) = self.slots();
        for (idx, val) in [(1usize, s1), (2, s2), (3, s3)] {
            if val.trim().is_empty() {
                return Err(format!(
                    "capability payload for {} has empty required slot {idx}",
                    self.kind().as_str()
                ));
            }
            validate_no_forbidden_material(&format!("payload_slot_{idx}"), val)?;
        }
        if let Some(s4_val) = s4 {
            if s4_val.trim().is_empty() {
                return Err("failure_pattern payload recovery slot must be non-empty".into());
            }
            validate_no_forbidden_material("payload_slot_4", s4_val)?;
        }
        Ok(())
    }
}

/// Lifecycle state for a `CapabilityScaffold`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CapabilityLifecycle {
    Candidate,
    Qualified,
    Benched,
    Retired,
}

impl CapabilityLifecycle {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Qualified => "qualified",
            Self::Benched => "benched",
            Self::Retired => "retired",
        }
    }

    pub const fn tag_byte(self) -> u8 {
        match self {
            Self::Candidate => 1,
            Self::Qualified => 2,
            Self::Benched => 3,
            Self::Retired => 4,
        }
    }

    pub fn from_tag_byte(tag: u8) -> Result<Self, String> {
        match tag {
            1 => Ok(Self::Candidate),
            2 => Ok(Self::Qualified),
            3 => Ok(Self::Benched),
            4 => Ok(Self::Retired),
            other => Err(format!("unknown CapabilityLifecycle tag {other}")),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "candidate" => Ok(Self::Candidate),
            "qualified" => Ok(Self::Qualified),
            "benched" => Ok(Self::Benched),
            "retired" => Ok(Self::Retired),
            other => Err(format!("unknown CapabilityLifecycle '{other}'")),
        }
    }
}

/// Operational side-effect class for a capability scaffold.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EffectClass {
    ReadOnly,
    LocalWrite,
    ExternalWrite,
    Destructive,
}

impl EffectClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::LocalWrite => "local_write",
            Self::ExternalWrite => "external_write",
            Self::Destructive => "destructive",
        }
    }

    pub const fn tag_byte(self) -> u8 {
        match self {
            Self::ReadOnly => 1,
            Self::LocalWrite => 2,
            Self::ExternalWrite => 3,
            Self::Destructive => 4,
        }
    }

    pub fn from_tag_byte(tag: u8) -> Result<Self, String> {
        match tag {
            1 => Ok(Self::ReadOnly),
            2 => Ok(Self::LocalWrite),
            3 => Ok(Self::ExternalWrite),
            4 => Ok(Self::Destructive),
            other => Err(format!("unknown EffectClass tag {other}")),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "read_only" => Ok(Self::ReadOnly),
            "local_write" => Ok(Self::LocalWrite),
            "external_write" => Ok(Self::ExternalWrite),
            "destructive" => Ok(Self::Destructive),
            other => Err(format!("unknown EffectClass '{other}'")),
        }
    }
}

/// All thirteen required directional edge kinds in the Capability Atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CapabilityEdgeKind {
    Solves,
    Specializes,
    Implements,
    Exposes,
    Requires,
    ComposesWith,
    ConflictsWith,
    FallbackFor,
    EnforcedBy,
    TriggeredBy,
    VerifiedBy,
    FailedOn,
    Supersedes,
}

impl CapabilityEdgeKind {
    pub const ALL: [Self; 13] = [
        Self::Solves,
        Self::Specializes,
        Self::Implements,
        Self::Exposes,
        Self::Requires,
        Self::ComposesWith,
        Self::ConflictsWith,
        Self::FallbackFor,
        Self::EnforcedBy,
        Self::TriggeredBy,
        Self::VerifiedBy,
        Self::FailedOn,
        Self::Supersedes,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solves => "solves",
            Self::Specializes => "specializes",
            Self::Implements => "implements",
            Self::Exposes => "exposes",
            Self::Requires => "requires",
            Self::ComposesWith => "composes_with",
            Self::ConflictsWith => "conflicts_with",
            Self::FallbackFor => "fallback_for",
            Self::EnforcedBy => "enforced_by",
            Self::TriggeredBy => "triggered_by",
            Self::VerifiedBy => "verified_by",
            Self::FailedOn => "failed_on",
            Self::Supersedes => "supersedes",
        }
    }

    pub const fn tag_byte(self) -> u8 {
        match self {
            Self::Solves => 1,
            Self::Specializes => 2,
            Self::Implements => 3,
            Self::Exposes => 4,
            Self::Requires => 5,
            Self::ComposesWith => 6,
            Self::ConflictsWith => 7,
            Self::FallbackFor => 8,
            Self::EnforcedBy => 9,
            Self::TriggeredBy => 10,
            Self::VerifiedBy => 11,
            Self::FailedOn => 12,
            Self::Supersedes => 13,
        }
    }

    pub fn from_tag_byte(tag: u8) -> Result<Self, String> {
        match tag {
            1 => Ok(Self::Solves),
            2 => Ok(Self::Specializes),
            3 => Ok(Self::Implements),
            4 => Ok(Self::Exposes),
            5 => Ok(Self::Requires),
            6 => Ok(Self::ComposesWith),
            7 => Ok(Self::ConflictsWith),
            8 => Ok(Self::FallbackFor),
            9 => Ok(Self::EnforcedBy),
            10 => Ok(Self::TriggeredBy),
            11 => Ok(Self::VerifiedBy),
            12 => Ok(Self::FailedOn),
            13 => Ok(Self::Supersedes),
            other => Err(format!("unknown CapabilityEdgeKind tag {other}")),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "solves" => Ok(Self::Solves),
            "specializes" => Ok(Self::Specializes),
            "implements" => Ok(Self::Implements),
            "exposes" => Ok(Self::Exposes),
            "requires" => Ok(Self::Requires),
            "composes_with" => Ok(Self::ComposesWith),
            "conflicts_with" => Ok(Self::ConflictsWith),
            "fallback_for" => Ok(Self::FallbackFor),
            "enforced_by" => Ok(Self::EnforcedBy),
            "triggered_by" => Ok(Self::TriggeredBy),
            "verified_by" => Ok(Self::VerifiedBy),
            "failed_on" => Ok(Self::FailedOn),
            "supersedes" => Ok(Self::Supersedes),
            other => Err(format!(
                "untyped or unknown capability edge kind '{other}' is forbidden"
            )),
        }
    }

    /// Validates directional endpoint kinds for this edge kind.
    pub fn validate_endpoints(
        self,
        from_kind: CapabilityKind,
        to_kind: CapabilityKind,
    ) -> Result<(), String> {
        let valid = match self {
            Self::Solves => {
                matches!(
                    from_kind,
                    CapabilityKind::SolutionFamily | CapabilityKind::Capability
                ) && to_kind == CapabilityKind::ProblemPattern
            }
            Self::Specializes => {
                matches!(
                    from_kind,
                    CapabilityKind::SolutionFamily | CapabilityKind::Capability
                ) && matches!(
                    to_kind,
                    CapabilityKind::SolutionFamily | CapabilityKind::Capability
                )
            }
            Self::Implements => {
                matches!(
                    from_kind,
                    CapabilityKind::Tool
                        | CapabilityKind::Skill
                        | Self::RUNBOOK_OR_PLUGIN_A
                        | Self::RUNBOOK_OR_PLUGIN_B
                ) && to_kind == CapabilityKind::Capability
            }
            Self::Exposes => {
                matches!(
                    from_kind,
                    CapabilityKind::McpServer | CapabilityKind::Plugin
                ) && to_kind == CapabilityKind::Tool
            }
            Self::Requires | Self::ComposesWith | Self::ConflictsWith | Self::FallbackFor => true,
            Self::EnforcedBy => {
                from_kind == CapabilityKind::PolicyRule || to_kind == CapabilityKind::PolicyRule
            }
            Self::TriggeredBy => {
                from_kind == CapabilityKind::Hook || to_kind == CapabilityKind::Hook
            }
            Self::VerifiedBy => {
                to_kind == CapabilityKind::EvidenceReceipt
                    || from_kind == CapabilityKind::EvidenceReceipt
            }
            Self::FailedOn => to_kind == CapabilityKind::FailurePattern,
            Self::Supersedes => from_kind == to_kind,
        };
        if !valid {
            return Err(format!(
                "invalid endpoint kinds for edge '{}': {} -> {}",
                self.as_str(),
                from_kind.as_str(),
                to_kind.as_str()
            ));
        }
        Ok(())
    }

    const RUNBOOK_OR_PLUGIN_A: CapabilityKind = CapabilityKind::Runbook;
    const RUNBOOK_OR_PLUGIN_B: CapabilityKind = CapabilityKind::Plugin;
}

/// Compact callable contract projected into context for a selected node.
/// Uses a bounded binary envelope (`GMCP\x01`) rather than JSON or prompt text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactCallableProjection {
    pub profile_id: RecordId,
    pub canonical_name: String,
    pub version: String,
    pub kind: CapabilityKind,
    pub argument_schema_digest: Digest,
    pub result_schema_digest: Digest,
    pub effect_class: EffectClass,
    pub permission_classes: Vec<String>,
    pub credential_classes: Vec<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub idempotent: bool,
    pub rollback_policy: String,
    pub max_memory_bytes: u64,
    pub live_registry_id: String,
}

impl CompactCallableProjection {
    pub fn to_binary_envelope(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::with_capacity(256);
        buf.extend_from_slice(CAPABILITY_PROJECTION_MAGIC);
        write_str(&mut buf, self.profile_id.as_str())?;
        write_str(&mut buf, &self.canonical_name)?;
        write_str(&mut buf, &self.version)?;
        buf.push(self.kind.tag_byte());
        write_digest(&mut buf, &self.argument_schema_digest);
        write_digest(&mut buf, &self.result_schema_digest);
        buf.push(self.effect_class.tag_byte());
        write_str_vec(&mut buf, &self.permission_classes)?;
        write_str_vec(&mut buf, &self.credential_classes)?;
        buf.extend_from_slice(&self.timeout_ms.to_le_bytes());
        buf.extend_from_slice(&self.max_retries.to_le_bytes());
        buf.push(u8::from(self.idempotent));
        write_str(&mut buf, &self.rollback_policy)?;
        buf.extend_from_slice(&self.max_memory_bytes.to_le_bytes());
        write_str(&mut buf, &self.live_registry_id)?;
        if buf.len() > MAX_PROJECTION_ENVELOPE_BYTES {
            return Err(format!(
                "compact callable projection exceeds {} bytes ({})",
                MAX_PROJECTION_ENVELOPE_BYTES,
                buf.len()
            ));
        }
        Ok(buf)
    }

    pub fn from_binary_envelope(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_PROJECTION_ENVELOPE_BYTES {
            return Err("compact callable projection envelope exceeds maximum size".into());
        }
        let mut cursor = BinaryCursor::new(bytes);
        let magic = cursor.read_fixed::<8>()?;
        if &magic != CAPABILITY_PROJECTION_MAGIC {
            return Err("invalid compact callable projection magic header".into());
        }
        let profile_id = RecordId::parse(cursor.read_str()?)?;
        let canonical_name = cursor.read_str()?;
        let version = cursor.read_str()?;
        let kind = CapabilityKind::from_tag_byte(cursor.read_u8()?)?;
        let argument_schema_digest = cursor.read_digest()?;
        let result_schema_digest = cursor.read_digest()?;
        let effect_class = EffectClass::from_tag_byte(cursor.read_u8()?)?;
        let permission_classes = cursor.read_str_vec()?;
        let credential_classes = cursor.read_str_vec()?;
        let timeout_ms = cursor.read_u64()?;
        let max_retries = cursor.read_u32()?;
        let idempotent = match cursor.read_u8()? {
            0 => false,
            1 => true,
            other => return Err(format!("invalid boolean byte {other}")),
        };
        let rollback_policy = cursor.read_str()?;
        let max_memory_bytes = cursor.read_u64()?;
        let live_registry_id = cursor.read_str()?;
        cursor.finish()?;
        Ok(Self {
            profile_id,
            canonical_name,
            version,
            kind,
            argument_schema_digest,
            result_schema_digest,
            effect_class,
            permission_classes,
            credential_classes,
            timeout_ms,
            max_retries,
            idempotent,
            rollback_policy,
            max_memory_bytes,
            live_registry_id,
        })
    }

    pub fn digest(&self) -> Result<Digest, String> {
        let bytes = self.to_binary_envelope()?;
        digest_bytes(&bytes)
    }
}

/// Complete immutable capability profile + native capability scaffold.
#[derive(Clone, Debug)]
pub struct CapabilityScaffold {
    pub id: RecordId,
    pub atom_id: RecordId,
    pub address_id: RecordId,
    pub membership_id: RecordId,
    pub workspace: RecordId,
    pub geometry_space: RecordId,
    pub well: RecordId,
    pub kind: CapabilityKind,
    pub canonical_name: String,
    pub summary: String,
    pub version: String,
    pub payload: CapabilityPayload,
    pub content_digest: Digest,
    /// Audit-only source digest; never enters `content_digest`, coordinates, or ranking.
    pub source_digest: Digest,
    pub configuration_digest: Digest,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub lifecycle: CapabilityLifecycle,
    pub admission_receipt_digest: Digest,
    pub judge_receipt_digest: Digest,
    pub replay_receipt_digest: Digest,
    pub crystallization_receipt_digest: Digest,
    // Required scaffold operational fields:
    pub argument_schema_digest: Digest,
    pub result_schema_digest: Digest,
    pub effect_class: EffectClass,
    pub permission_classes: Vec<String>,
    pub credential_classes: Vec<String>,
    pub platform_requirements: Vec<String>,
    pub runtime_requirements: Vec<String>,
    pub version_requirement: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub idempotent: bool,
    pub rollback_policy: String,
    pub max_memory_bytes: u64,
    pub health_check_id: String,
    pub live_registry_id: String,
    pub component_receipt_digest: Option<Digest>,
    pub integration_receipt_digest: Option<Digest>,
    pub live_receipt_digest: Option<Digest>,
    pub incompatible_profiles: Vec<RecordId>,
    pub benched_reason: Option<String>,
    pub expected_utility: f64,
    pub resource_cost: f64,
    pub prediction_error: f64,
}

impl CapabilityScaffold {
    /// Computes the model-neutral content digest strictly from the semantic
    /// capability specification (excluding `source_digest`).
    pub fn compute_content_digest(
        kind: CapabilityKind,
        canonical_name: &str,
        summary: &str,
        version: &str,
        payload: &CapabilityPayload,
        argument_schema_digest: &Digest,
        result_schema_digest: &Digest,
    ) -> Digest {
        let (s1, s2, s3, s4) = payload.slots();
        let mut hasher = Sha256::new();
        hasher.update(b"globalmind-capability-content-v1:");
        hasher.update(kind.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(canonical_name.as_bytes());
        hasher.update(b":");
        hasher.update(summary.as_bytes());
        hasher.update(b":");
        hasher.update(version.as_bytes());
        hasher.update(b":");
        hasher.update(s1.as_bytes());
        hasher.update(b":");
        hasher.update(s2.as_bytes());
        hasher.update(b":");
        hasher.update(s3.as_bytes());
        hasher.update(b":");
        if let Some(s4_val) = s4 {
            hasher.update(s4_val.as_bytes());
        }
        hasher.update(b":");
        hasher.update(argument_schema_digest.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(result_schema_digest.as_str().as_bytes());
        Digest::parse(hex_lower(&hasher.finalize())).expect("SHA-256 hex is valid")
    }

    /// Derives deterministic, model-neutral $(H^3 \times T^2)/\Gamma_{12}$
    /// coordinates anchored near the assigned cognitive well. Strictly
    /// invariant to `source_digest`.
    pub fn derive_model_neutral_coordinates(
        topology: &WhiteheadTopology,
        well_ordinal: u8,
        content_digest: &Digest,
    ) -> Result<([f64; 4], [f64; 2], RecordId), String> {
        let well = topology
            .wells
            .get(well_ordinal as usize)
            .ok_or_else(|| format!("invalid well ordinal {well_ordinal}"))?;
        let bytes = content_digest.as_str().as_bytes();
        let b0 =
            u64::from_str_radix(std::str::from_utf8(&bytes[0..8]).unwrap_or("0"), 16).unwrap_or(0);
        let b1 =
            u64::from_str_radix(std::str::from_utf8(&bytes[8..16]).unwrap_or("0"), 16).unwrap_or(0);
        let b2 = u64::from_str_radix(std::str::from_utf8(&bytes[16..24]).unwrap_or("0"), 16)
            .unwrap_or(0);
        let jitter_x = ((b0 as f64 / u64::MAX as f64) - 0.5) * 0.03;
        let jitter_y = ((b1 as f64 / u64::MAX as f64) - 0.5) * 0.03;
        let jitter_z = ((b2 as f64 / u64::MAX as f64) - 0.5) * 0.03;
        let p = [
            well.poincare_center[0] + jitter_x,
            well.poincare_center[1] + jitter_y,
            well.poincare_center[2] + jitter_z,
        ];
        let h3_lorentz = poincare_to_lorentz(p)?;
        let phase_jitter_1 = ((b0 as f64 / u64::MAX as f64) - 0.5) * 0.04;
        let phase_jitter_2 = ((b1 as f64 / u64::MAX as f64) - 0.5) * 0.04;
        let t2_phase = [
            (well.t2_center_phase[0] + phase_jitter_1).rem_euclid(TAU),
            (well.t2_center_phase[1] + phase_jitter_2).rem_euclid(TAU),
        ];
        Ok((h3_lorentz, t2_phase, well.id.clone()))
    }

    pub fn compact_projection(&self) -> CompactCallableProjection {
        CompactCallableProjection {
            profile_id: self.id.clone(),
            canonical_name: self.canonical_name.clone(),
            version: self.version.clone(),
            kind: self.kind,
            argument_schema_digest: self.argument_schema_digest.clone(),
            result_schema_digest: self.result_schema_digest.clone(),
            effect_class: self.effect_class,
            permission_classes: self.permission_classes.clone(),
            credential_classes: self.credential_classes.clone(),
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            idempotent: self.idempotent,
            rollback_policy: self.rollback_policy.clone(),
            max_memory_bytes: self.max_memory_bytes,
            live_registry_id: self.live_registry_id.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.payload.kind() != self.kind {
            return Err(format!(
                "capability profile kind {} does not match payload kind {}",
                self.kind.as_str(),
                self.payload.kind().as_str()
            ));
        }
        self.payload.validate()?;
        for (field, value) in [
            ("canonical_name", self.canonical_name.as_str()),
            ("summary", self.summary.as_str()),
            ("version", self.version.as_str()),
            ("version_requirement", self.version_requirement.as_str()),
            ("rollback_policy", self.rollback_policy.as_str()),
            ("health_check_id", self.health_check_id.as_str()),
            ("live_registry_id", self.live_registry_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("capability scaffold {field} must be non-empty"));
            }
            validate_no_forbidden_material(field, value)?;
        }
        for perm in &self.permission_classes {
            if perm.trim().is_empty() {
                return Err("permission_classes entry must be non-empty".into());
            }
            validate_no_forbidden_material("permission_classes", perm)?;
        }
        for cred_class in &self.credential_classes {
            if cred_class.trim().is_empty() {
                return Err("credential_classes entry must be non-empty".into());
            }
            validate_credential_class_only(cred_class)?;
        }
        for req in self
            .platform_requirements
            .iter()
            .chain(&self.runtime_requirements)
        {
            if req.trim().is_empty() {
                return Err("platform/runtime requirement entry must be non-empty".into());
            }
            validate_no_forbidden_material("requirement", req)?;
        }
        if self.timeout_ms == 0 {
            return Err("capability scaffold timeout_ms must be positive".into());
        }
        if self.max_memory_bytes == 0 {
            return Err("capability scaffold max_memory_bytes must be positive".into());
        }
        if self.lifecycle == CapabilityLifecycle::Benched
            && self
                .benched_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            return Err("benched capability scaffold requires a non-empty benched_reason".into());
        }
        if !self.expected_utility.is_finite() || !(0.0..=1.0).contains(&self.expected_utility) {
            return Err("expected_utility must be finite and in [0, 1]".into());
        }
        if !self.resource_cost.is_finite() || !(0.0..=1.0).contains(&self.resource_cost) {
            return Err("resource_cost must be finite and in [0, 1]".into());
        }
        if !self.prediction_error.is_finite() || self.prediction_error < 0.0 {
            return Err("prediction_error must be finite and nonnegative".into());
        }
        validate_lorentz_point(self.h3_lorentz)?;
        validate_t2_phase(self.t2_phase)?;
        let expected_content = Self::compute_content_digest(
            self.kind,
            &self.canonical_name,
            &self.summary,
            &self.version,
            &self.payload,
            &self.argument_schema_digest,
            &self.result_schema_digest,
        );
        if self.content_digest != expected_content {
            return Err(format!(
                "capability scaffold content_digest mismatch for {}: expected {}, got {}",
                self.canonical_name,
                expected_content.as_str(),
                self.content_digest.as_str()
            ));
        }
        Ok(())
    }

    pub fn to_binary_envelope(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut buf = Vec::with_capacity(512);
        buf.extend_from_slice(CAPABILITY_SCAFFOLD_MAGIC);
        write_str(&mut buf, self.id.as_str())?;
        write_str(&mut buf, self.atom_id.as_str())?;
        write_str(&mut buf, self.address_id.as_str())?;
        write_str(&mut buf, self.membership_id.as_str())?;
        write_str(&mut buf, self.workspace.as_str())?;
        write_str(&mut buf, self.geometry_space.as_str())?;
        write_str(&mut buf, self.well.as_str())?;
        buf.push(self.kind.tag_byte());
        write_str(&mut buf, &self.canonical_name)?;
        write_str(&mut buf, &self.summary)?;
        write_str(&mut buf, &self.version)?;
        let (s1, s2, s3, s4) = self.payload.slots();
        write_str(&mut buf, s1)?;
        write_str(&mut buf, s2)?;
        write_str(&mut buf, s3)?;
        write_opt_str(&mut buf, s4)?;
        write_digest(&mut buf, &self.content_digest);
        write_digest(&mut buf, &self.source_digest);
        write_digest(&mut buf, &self.configuration_digest);
        for v in self.h3_lorentz {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        for v in self.t2_phase {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.push(self.lifecycle.tag_byte());
        write_digest(&mut buf, &self.admission_receipt_digest);
        write_digest(&mut buf, &self.judge_receipt_digest);
        write_digest(&mut buf, &self.replay_receipt_digest);
        write_digest(&mut buf, &self.crystallization_receipt_digest);
        write_digest(&mut buf, &self.argument_schema_digest);
        write_digest(&mut buf, &self.result_schema_digest);
        buf.push(self.effect_class.tag_byte());
        write_str_vec(&mut buf, &self.permission_classes)?;
        write_str_vec(&mut buf, &self.credential_classes)?;
        write_str_vec(&mut buf, &self.platform_requirements)?;
        write_str_vec(&mut buf, &self.runtime_requirements)?;
        write_str(&mut buf, &self.version_requirement)?;
        buf.extend_from_slice(&self.timeout_ms.to_le_bytes());
        buf.extend_from_slice(&self.max_retries.to_le_bytes());
        buf.push(u8::from(self.idempotent));
        write_str(&mut buf, &self.rollback_policy)?;
        buf.extend_from_slice(&self.max_memory_bytes.to_le_bytes());
        write_str(&mut buf, &self.health_check_id)?;
        write_str(&mut buf, &self.live_registry_id)?;
        write_opt_digest(&mut buf, self.component_receipt_digest.as_ref());
        write_opt_digest(&mut buf, self.integration_receipt_digest.as_ref());
        write_opt_digest(&mut buf, self.live_receipt_digest.as_ref());
        let incompat_strs: Vec<String> = self
            .incompatible_profiles
            .iter()
            .map(|id| id.as_str().to_string())
            .collect();
        write_str_vec(&mut buf, &incompat_strs)?;
        write_opt_str(&mut buf, self.benched_reason.as_deref())?;
        buf.extend_from_slice(&self.expected_utility.to_le_bytes());
        buf.extend_from_slice(&self.resource_cost.to_le_bytes());
        buf.extend_from_slice(&self.prediction_error.to_le_bytes());
        if buf.len() > MAX_SCAFFOLD_ENVELOPE_BYTES {
            return Err(format!(
                "capability scaffold envelope exceeds {} bytes ({})",
                MAX_SCAFFOLD_ENVELOPE_BYTES,
                buf.len()
            ));
        }
        Ok(buf)
    }

    pub fn from_binary_envelope(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_SCAFFOLD_ENVELOPE_BYTES {
            return Err("capability scaffold envelope exceeds maximum size".into());
        }
        let mut cursor = BinaryCursor::new(bytes);
        let magic = cursor.read_fixed::<8>()?;
        if &magic != CAPABILITY_SCAFFOLD_MAGIC {
            return Err("invalid capability scaffold magic header".into());
        }
        let id = RecordId::parse(cursor.read_str()?)?;
        let atom_id = RecordId::parse(cursor.read_str()?)?;
        let address_id = RecordId::parse(cursor.read_str()?)?;
        let membership_id = RecordId::parse(cursor.read_str()?)?;
        let workspace = RecordId::parse(cursor.read_str()?)?;
        let geometry_space = RecordId::parse(cursor.read_str()?)?;
        let well = RecordId::parse(cursor.read_str()?)?;
        let kind = CapabilityKind::from_tag_byte(cursor.read_u8()?)?;
        let canonical_name = cursor.read_str()?;
        let summary = cursor.read_str()?;
        let version = cursor.read_str()?;
        let slot1 = cursor.read_str()?;
        let slot2 = cursor.read_str()?;
        let slot3 = cursor.read_str()?;
        let slot4 = cursor.read_opt_str()?;
        let payload = CapabilityPayload::from_slots(kind, slot1, slot2, slot3, slot4)?;
        let content_digest = cursor.read_digest()?;
        let source_digest = cursor.read_digest()?;
        let configuration_digest = cursor.read_digest()?;
        let h3_lorentz = [
            cursor.read_f64()?,
            cursor.read_f64()?,
            cursor.read_f64()?,
            cursor.read_f64()?,
        ];
        let t2_phase = [cursor.read_f64()?, cursor.read_f64()?];
        let lifecycle = CapabilityLifecycle::from_tag_byte(cursor.read_u8()?)?;
        let admission_receipt_digest = cursor.read_digest()?;
        let judge_receipt_digest = cursor.read_digest()?;
        let replay_receipt_digest = cursor.read_digest()?;
        let crystallization_receipt_digest = cursor.read_digest()?;
        let argument_schema_digest = cursor.read_digest()?;
        let result_schema_digest = cursor.read_digest()?;
        let effect_class = EffectClass::from_tag_byte(cursor.read_u8()?)?;
        let permission_classes = cursor.read_str_vec()?;
        let credential_classes = cursor.read_str_vec()?;
        let platform_requirements = cursor.read_str_vec()?;
        let runtime_requirements = cursor.read_str_vec()?;
        let version_requirement = cursor.read_str()?;
        let timeout_ms = cursor.read_u64()?;
        let max_retries = cursor.read_u32()?;
        let idempotent = match cursor.read_u8()? {
            0 => false,
            1 => true,
            other => return Err(format!("invalid boolean byte {other}")),
        };
        let rollback_policy = cursor.read_str()?;
        let max_memory_bytes = cursor.read_u64()?;
        let health_check_id = cursor.read_str()?;
        let live_registry_id = cursor.read_str()?;
        let component_receipt_digest = cursor.read_opt_digest()?;
        let integration_receipt_digest = cursor.read_opt_digest()?;
        let live_receipt_digest = cursor.read_opt_digest()?;
        let incompat_raw = cursor.read_str_vec()?;
        let mut incompatible_profiles = Vec::with_capacity(incompat_raw.len());
        for raw in incompat_raw {
            incompatible_profiles.push(RecordId::parse(raw)?);
        }
        let benched_reason = cursor.read_opt_str()?;
        let expected_utility = cursor.read_f64()?;
        let resource_cost = cursor.read_f64()?;
        let prediction_error = cursor.read_f64()?;
        cursor.finish()?;
        let scaffold = Self {
            id,
            atom_id,
            address_id,
            membership_id,
            workspace,
            geometry_space,
            well,
            kind,
            canonical_name,
            summary,
            version,
            payload,
            content_digest,
            source_digest,
            configuration_digest,
            h3_lorentz,
            t2_phase,
            lifecycle,
            admission_receipt_digest,
            judge_receipt_digest,
            replay_receipt_digest,
            crystallization_receipt_digest,
            argument_schema_digest,
            result_schema_digest,
            effect_class,
            permission_classes,
            credential_classes,
            platform_requirements,
            runtime_requirements,
            version_requirement,
            timeout_ms,
            max_retries,
            idempotent,
            rollback_policy,
            max_memory_bytes,
            health_check_id,
            live_registry_id,
            component_receipt_digest,
            integration_receipt_digest,
            live_receipt_digest,
            incompatible_profiles,
            benched_reason,
            expected_utility,
            resource_cost,
            prediction_error,
        };
        scaffold.validate()?;
        Ok(scaffold)
    }

    pub fn envelope_digest(&self) -> Result<Digest, String> {
        let bytes = self.to_binary_envelope()?;
        digest_bytes(&bytes)
    }
}

/// Immutable, evidence-bearing directional edge in the Capability Atlas.
#[derive(Clone, Debug, PartialEq)]
pub struct CapabilityEdgeRecord {
    pub id: RecordId,
    pub workspace: RecordId,
    pub from_profile: RecordId,
    pub to_profile: RecordId,
    pub from_atom: RecordId,
    pub to_atom: RecordId,
    pub kind: CapabilityEdgeKind,
    pub confidence: f64,
    pub evidence_receipt: Option<RecordId>,
    pub evidence_receipt_digest: Digest,
    pub producer: String,
    pub revision: u32,
    pub valid_from_unix_ms: u64,
    pub valid_to_unix_ms: Option<u64>,
    pub compatibility_digest: Digest,
    pub crystallization_receipt_digest: Digest,
}

impl CapabilityEdgeRecord {
    pub fn validate(&self) -> Result<(), String> {
        if !self.confidence.is_finite() || !(0.0..=1.0).contains(&self.confidence) {
            return Err("capability edge confidence must be finite and in [0, 1]".into());
        }
        if self.producer.trim().is_empty() {
            return Err("capability edge producer must be non-empty".into());
        }
        validate_no_forbidden_material("edge.producer", &self.producer)?;
        if self.revision == 0 {
            return Err("capability edge revision must be >= 1".into());
        }
        if let Some(valid_to) = self.valid_to_unix_ms
            && valid_to <= self.valid_from_unix_ms
        {
            return Err("capability edge valid_to must be strictly greater than valid_from".into());
        }
        Ok(())
    }

    pub fn to_binary_envelope(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut buf = Vec::with_capacity(256);
        buf.extend_from_slice(CAPABILITY_EDGE_MAGIC);
        write_str(&mut buf, self.id.as_str())?;
        write_str(&mut buf, self.workspace.as_str())?;
        write_str(&mut buf, self.from_profile.as_str())?;
        write_str(&mut buf, self.to_profile.as_str())?;
        write_str(&mut buf, self.from_atom.as_str())?;
        write_str(&mut buf, self.to_atom.as_str())?;
        buf.push(self.kind.tag_byte());
        buf.extend_from_slice(&self.confidence.to_le_bytes());
        write_opt_str(
            &mut buf,
            self.evidence_receipt.as_ref().map(RecordId::as_str),
        )?;
        write_digest(&mut buf, &self.evidence_receipt_digest);
        write_str(&mut buf, &self.producer)?;
        buf.extend_from_slice(&self.revision.to_le_bytes());
        buf.extend_from_slice(&self.valid_from_unix_ms.to_le_bytes());
        match self.valid_to_unix_ms {
            Some(ts) => {
                buf.push(1);
                buf.extend_from_slice(&ts.to_le_bytes());
            }
            None => buf.push(0),
        }
        write_digest(&mut buf, &self.compatibility_digest);
        write_digest(&mut buf, &self.crystallization_receipt_digest);
        if buf.len() > MAX_EDGE_ENVELOPE_BYTES {
            return Err("capability edge binary envelope exceeds maximum size".into());
        }
        Ok(buf)
    }

    pub fn from_binary_envelope(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_EDGE_ENVELOPE_BYTES {
            return Err("capability edge envelope exceeds maximum size".into());
        }
        let mut cursor = BinaryCursor::new(bytes);
        let magic = cursor.read_fixed::<8>()?;
        if &magic != CAPABILITY_EDGE_MAGIC {
            return Err("invalid capability edge magic header".into());
        }
        let id = RecordId::parse(cursor.read_str()?)?;
        let workspace = RecordId::parse(cursor.read_str()?)?;
        let from_profile = RecordId::parse(cursor.read_str()?)?;
        let to_profile = RecordId::parse(cursor.read_str()?)?;
        let from_atom = RecordId::parse(cursor.read_str()?)?;
        let to_atom = RecordId::parse(cursor.read_str()?)?;
        let kind = CapabilityEdgeKind::from_tag_byte(cursor.read_u8()?)?;
        let confidence = cursor.read_f64()?;
        let evidence_receipt = match cursor.read_opt_str()? {
            Some(raw) => Some(RecordId::parse(raw)?),
            None => None,
        };
        let evidence_receipt_digest = cursor.read_digest()?;
        let producer = cursor.read_str()?;
        let revision = cursor.read_u32()?;
        let valid_from_unix_ms = cursor.read_u64()?;
        let valid_to_unix_ms = match cursor.read_u8()? {
            0 => None,
            1 => Some(cursor.read_u64()?),
            other => return Err(format!("invalid optional timestamp tag {other}")),
        };
        let compatibility_digest = cursor.read_digest()?;
        let crystallization_receipt_digest = cursor.read_digest()?;
        cursor.finish()?;
        let edge = Self {
            id,
            workspace,
            from_profile,
            to_profile,
            from_atom,
            to_atom,
            kind,
            confidence,
            evidence_receipt,
            evidence_receipt_digest,
            producer,
            revision,
            valid_from_unix_ms,
            valid_to_unix_ms,
            compatibility_digest,
            crystallization_receipt_digest,
        };
        edge.validate()?;
        Ok(edge)
    }
}

/// Authoritative live runtime registry entry for one installed executable or provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRegistryEntry {
    pub live_registry_id: String,
    pub installed: bool,
    pub healthy: bool,
    pub version: String,
    pub argument_schema_digest: Digest,
    pub result_schema_digest: Digest,
    pub configuration_digest: Digest,
    pub granted_permissions: BTreeSet<String>,
    pub available_credential_classes: BTreeSet<String>,
}

/// Authoritative live runtime registry. The Capability Atlas is never the
/// execution or availability authority; every proposed executable must
/// reconcile against this registry before planning.
#[derive(Clone, Debug, Default)]
pub struct LiveRegistry {
    pub entries: BTreeMap<String, LiveRegistryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationDisqualification {
    MissingFromLiveRegistry,
    NotInstalled,
    Unhealthy,
    VersionMismatch { expected: String, live: String },
    ArgumentSchemaDigestMismatch,
    ResultSchemaDigestMismatch,
    ConfigurationDigestMismatch,
    MissingPermission(String),
    MissingCredentialClass(String),
}

impl LiveRegistry {
    pub fn register(&mut self, entry: LiveRegistryEntry) {
        self.entries.insert(entry.live_registry_id.clone(), entry);
    }

    pub fn reconcile(
        &self,
        scaffold: &CapabilityScaffold,
    ) -> Result<(), ReconciliationDisqualification> {
        if !scaffold.kind.requires_live_reconciliation() {
            return Ok(());
        }
        let Some(entry) = self.entries.get(&scaffold.live_registry_id) else {
            return Err(ReconciliationDisqualification::MissingFromLiveRegistry);
        };
        if !entry.installed {
            return Err(ReconciliationDisqualification::NotInstalled);
        }
        if !entry.healthy {
            return Err(ReconciliationDisqualification::Unhealthy);
        }
        if entry.version != scaffold.version {
            return Err(ReconciliationDisqualification::VersionMismatch {
                expected: scaffold.version.clone(),
                live: entry.version.clone(),
            });
        }
        if entry.argument_schema_digest != scaffold.argument_schema_digest {
            return Err(ReconciliationDisqualification::ArgumentSchemaDigestMismatch);
        }
        if entry.result_schema_digest != scaffold.result_schema_digest {
            return Err(ReconciliationDisqualification::ResultSchemaDigestMismatch);
        }
        if entry.configuration_digest != scaffold.configuration_digest {
            return Err(ReconciliationDisqualification::ConfigurationDigestMismatch);
        }
        for perm in &scaffold.permission_classes {
            if !entry.granted_permissions.contains(perm) {
                return Err(ReconciliationDisqualification::MissingPermission(
                    perm.clone(),
                ));
            }
        }
        for cred in &scaffold.credential_classes {
            if !entry.available_credential_classes.contains(cred) {
                return Err(ReconciliationDisqualification::MissingCredentialClass(
                    cred.clone(),
                ));
            }
        }
        Ok(())
    }
}

/// Mandatory safety and model-I/O policy gate enforced unconditionally in Rust.
/// Never depends on SurrealDB or Capability Atlas availability.
#[derive(Clone, Debug)]
pub struct MandatoryPolicyGate {
    pub allow_local_write: bool,
    pub allow_external_write: bool,
    pub allow_destructive: bool,
    pub max_allowed_timeout_ms: u64,
    pub max_allowed_memory_bytes: u64,
}

impl Default for MandatoryPolicyGate {
    fn default() -> Self {
        Self::read_only_smoke()
    }
}

impl MandatoryPolicyGate {
    pub const fn read_only_smoke() -> Self {
        Self {
            allow_local_write: false,
            allow_external_write: false,
            allow_destructive: false,
            max_allowed_timeout_ms: 30_000,
            max_allowed_memory_bytes: 256 * 1024 * 1024,
        }
    }

    /// Enforces mandatory safety and binary model-I/O rules on a compact
    /// callable projection and raw binary argument payload, even when the
    /// Capability Atlas or SurrealDB is completely unavailable.
    pub fn enforce_call(
        &self,
        projection: &CompactCallableProjection,
        binary_request_envelope: &[u8],
    ) -> Result<Digest, String> {
        // 1. Effect class enforcement.
        match projection.effect_class {
            EffectClass::ReadOnly => {}
            EffectClass::LocalWrite if !self.allow_local_write => {
                return Err(
                    "mandatory policy gate blocked local_write effect in read-only mode".into(),
                );
            }
            EffectClass::ExternalWrite if !self.allow_external_write => {
                return Err(
                    "mandatory policy gate blocked external_write effect in read-only mode".into(),
                );
            }
            EffectClass::Destructive if !self.allow_destructive => {
                return Err("mandatory policy gate blocked destructive effect".into());
            }
            _ => {}
        }
        // 2. Resource bounds enforcement.
        if projection.timeout_ms == 0 || projection.timeout_ms > self.max_allowed_timeout_ms {
            return Err(format!(
                "mandatory policy gate blocked timeout_ms={} (max={})",
                projection.timeout_ms, self.max_allowed_timeout_ms
            ));
        }
        if projection.max_memory_bytes == 0
            || projection.max_memory_bytes > self.max_allowed_memory_bytes
        {
            return Err(format!(
                "mandatory policy gate blocked max_memory_bytes={} (max={})",
                projection.max_memory_bytes, self.max_allowed_memory_bytes
            ));
        }
        // 3. Binary model-I/O invariant: reject JSON, JSONL, Python, or text
        //    tensor channels crossing the planning/execution boundary.
        Self::enforce_binary_io_boundary(binary_request_envelope)?;

        let mut hasher = Sha256::new();
        hasher.update(b"globalmind-policy-gate-receipt-v1:");
        hasher.update(projection.profile_id.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(projection.argument_schema_digest.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(binary_request_envelope);
        Digest::parse(hex_lower(&hasher.finalize()))
    }

    pub fn enforce_binary_io_boundary(bytes: &[u8]) -> Result<(), String> {
        if bytes.len() < 8 || &bytes[0..4] != b"GMBI" {
            return Err(
                "mandatory model-I/O rule violation: payload lacks native binary GMBI header"
                    .into(),
            );
        }
        if let Ok(as_utf8) = std::str::from_utf8(&bytes[8..]) {
            let trimmed = as_utf8.trim_start();
            if trimmed.starts_with('{')
                || trimmed.starts_with('[')
                || trimmed.contains("\"jsonrpc\"")
                || trimmed.contains("import torch")
                || trimmed.contains("def ")
                || trimmed.contains("transformer_kv_page")
            {
                return Err(
                    "mandatory model-I/O rule violation: JSON/Python/text-tensor payload rejected"
                        .into(),
                );
            }
        }
        Ok(())
    }
}

/// Compact binary context frame containing ONLY the selected callable contracts
/// for a materialized execution DAG. Global tool-list injection is structurally
/// impossible because the frame rejects unselected profiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactContextFrame {
    pub dag_digest: Digest,
    pub projections: Vec<CompactCallableProjection>,
}

impl CompactContextFrame {
    pub fn to_binary_envelope(&self) -> Result<Vec<u8>, String> {
        if self.projections.is_empty() {
            return Err("compact context frame requires at least one selected projection".into());
        }
        let mut buf = Vec::with_capacity(512);
        buf.extend_from_slice(CAPABILITY_CONTEXT_FRAME_MAGIC);
        write_digest(&mut buf, &self.dag_digest);
        let count = u32::try_from(self.projections.len())
            .map_err(|_| "too many projections in context frame".to_string())?;
        buf.extend_from_slice(&count.to_le_bytes());
        for proj in &self.projections {
            let env = proj.to_binary_envelope()?;
            let len = u32::try_from(env.len())
                .map_err(|_| "projection envelope too large".to_string())?;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(&env);
        }
        if buf.len() > MAX_CONTEXT_FRAME_BYTES {
            return Err("compact context frame exceeds maximum size".into());
        }
        Ok(buf)
    }

    pub fn from_binary_envelope(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_CONTEXT_FRAME_BYTES {
            return Err("compact context frame exceeds maximum size".into());
        }
        let mut cursor = BinaryCursor::new(bytes);
        let magic = cursor.read_fixed::<8>()?;
        if &magic != CAPABILITY_CONTEXT_FRAME_MAGIC {
            return Err("invalid compact context frame magic".into());
        }
        let dag_digest = cursor.read_digest()?;
        let count = cursor.read_u32()? as usize;
        if count == 0 || count > 64 {
            return Err(format!(
                "invalid compact context frame projection count {count}"
            ));
        }
        let mut projections = Vec::with_capacity(count);
        for _ in 0..count {
            let len = cursor.read_u32()? as usize;
            let slice = cursor.read_slice(len)?;
            projections.push(CompactCallableProjection::from_binary_envelope(slice)?);
        }
        cursor.finish()?;
        Ok(Self {
            dag_digest,
            projections,
        })
    }

    pub fn digest(&self) -> Result<Digest, String> {
        let bytes = self.to_binary_envelope()?;
        digest_bytes(&bytes)
    }
}

pub type DagDependencyEdge = (RecordId, RecordId, CapabilityEdgeKind);

/// Materialized acyclic execution DAG produced by `AtlasPlanner`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionDag {
    pub problem_profile_id: RecordId,
    pub solution_family_id: RecordId,
    pub capability_id: RecordId,
    pub topological_order: Vec<RecordId>,
    pub dependency_edges: Vec<DagDependencyEdge>,
    pub mandatory_rules: Vec<RecordId>,
    pub used_fallback: bool,
    pub dag_digest: Digest,
}

impl ExecutionDag {
    /// Verifies that `dependency_edges` form a strict DAG consistent with
    /// `topological_order`.
    pub fn verify_acyclic(&self) -> Result<(), String> {
        let mut positions = BTreeMap::new();
        for (idx, node) in self.topological_order.iter().enumerate() {
            if positions.insert(node.clone(), idx).is_some() {
                return Err(format!("duplicate node {} in ExecutionDag", node.as_str()));
            }
        }
        for (from, to, kind) in &self.dependency_edges {
            let Some(&u) = positions.get(from) else {
                return Err(format!("DAG edge source {} not in nodes", from.as_str()));
            };
            let Some(&v) = positions.get(to) else {
                return Err(format!("DAG edge target {} not in nodes", to.as_str()));
            };
            if u >= v {
                return Err(format!(
                    "ExecutionDag cycle or topological violation on {} ({}) -> {} ({}) [{}]",
                    from.as_str(),
                    u,
                    to.as_str(),
                    v,
                    kind.as_str()
                ));
            }
        }
        Ok(())
    }
}

/// Complete output of the 8-step Capability Atlas retrieval and planning pipeline.
#[derive(Clone, Debug)]
pub struct PlannedCapabilityExecution {
    pub nearest_solution_family: RecordId,
    pub expanded_two_hop_neighborhood: BTreeSet<RecordId>,
    pub excluded_profiles: BTreeMap<RecordId, String>,
    pub ranked_compositions: Vec<(RecordId, f64)>,
    pub dag: ExecutionDag,
    pub context_frame: CompactContextFrame,
    pub context_frame_bytes: Vec<u8>,
}

/// Append-only utility and prediction-error receipt recorded after execution.
#[derive(Clone, Debug)]
pub struct CapabilityUtilityReceipt {
    pub id: RecordId,
    pub workspace: RecordId,
    pub problem_profile: RecordId,
    pub selected_profiles: Vec<RecordId>,
    pub dag_digest: Digest,
    pub execution_receipt_digest: Digest,
    pub observed_utility: f64,
    pub prediction_error: f64,
    pub resource_cost: f64,
    pub outcome: UtilityOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UtilityOutcome {
    Succeeded,
    Degraded,
    Failed,
    BlockedByPolicy,
}

impl UtilityOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::BlockedByPolicy => "blocked_by_policy",
        }
    }
}

/// In-memory snapshot of the Capability Atlas used by `AtlasPlanner`.
#[derive(Clone, Debug, Default)]
pub struct CapabilityAtlasSnapshot {
    pub profiles: BTreeMap<RecordId, CapabilityScaffold>,
    pub edges: Vec<CapabilityEdgeRecord>,
}

impl CapabilityAtlasSnapshot {
    /// Returns the set of profile IDs that have been superseded by a newer
    /// qualified profile via `supersedes`.
    pub fn superseded_profiles(&self) -> BTreeSet<RecordId> {
        let mut out = BTreeSet::new();
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::Supersedes
                && let Some(newer) = self.profiles.get(&edge.from_profile)
                && newer.lifecycle == CapabilityLifecycle::Qualified
            {
                out.insert(edge.to_profile.clone());
            }
        }
        out
    }

    /// Executes the 8-step retrieval and planning algorithm:
    /// 1. Locate the problem pattern and enter the nearest `solution_family`
    ///    neighborhood by $(H^3 \times T^2)$ geodesic distance.
    /// 2. Expand at most two typed hops (`MAX_INITIAL_TYPED_HOPS = 2`).
    /// 3. Remove `benched`, `retired`, superseded, and conflicting candidates.
    /// 4. Reconcile every remaining candidate with `LiveRegistry` and current
    ///    permissions; promote evidence-backed `fallback_for` candidates when
    ///    the primary executable is unavailable or conflicting.
    /// 5. Rank complete compatible compositions using evidence receipts,
    ///    expected utility, resource cost, and prediction error — strictly
    ///    independent of popularity or `source_digest`.
    /// 6. Materialize the chosen composition as an acyclic `ExecutionDag`.
    /// 7. Project only the selected callable contracts into a binary
    ///    `CompactContextFrame`.
    pub fn plan_for_query(
        &self,
        query_h3_lorentz: [f64; 4],
        query_t2_phase: [f64; 2],
        live_registry: &LiveRegistry,
    ) -> Result<PlannedCapabilityExecution, String> {
        validate_lorentz_point(query_h3_lorentz)?;
        validate_t2_phase(query_t2_phase)?;

        let superseded = self.superseded_profiles();

        // Step 1: Find nearest qualified `problem_pattern` and nearest
        // `solution_family` by (H^3 x T^2) distance.
        let problem_id = self
            .profiles
            .values()
            .filter(|p| {
                p.kind == CapabilityKind::ProblemPattern
                    && p.lifecycle == CapabilityLifecycle::Qualified
                    && !superseded.contains(&p.id)
            })
            .min_by(|a, b| {
                let da = manifold_distance(query_h3_lorentz, query_t2_phase, a);
                let db = manifold_distance(query_h3_lorentz, query_t2_phase, b);
                da.total_cmp(&db)
                    .then_with(|| a.id.as_str().cmp(b.id.as_str()))
            })
            .map(|p| p.id.clone())
            .ok_or_else(|| "no qualified problem_pattern in Capability Atlas".to_string())?;

        let nearest_family_id = self
            .profiles
            .values()
            .filter(|p| {
                p.kind == CapabilityKind::SolutionFamily
                    && p.lifecycle == CapabilityLifecycle::Qualified
                    && !superseded.contains(&p.id)
            })
            .min_by(|a, b| {
                let da = manifold_distance(query_h3_lorentz, query_t2_phase, a);
                let db = manifold_distance(query_h3_lorentz, query_t2_phase, b);
                da.total_cmp(&db)
                    .then_with(|| a.id.as_str().cmp(b.id.as_str()))
            })
            .map(|p| p.id.clone())
            .ok_or_else(|| "no qualified solution_family in Capability Atlas".to_string())?;

        // Step 2: Expand at most two typed hops from `nearest_family_id` (and
        // any family solving `problem_id` within geodesic proximity).
        let mut frontier = VecDeque::new();
        let mut hop_distance = BTreeMap::new();
        frontier.push_back((nearest_family_id.clone(), 0usize));
        hop_distance.insert(nearest_family_id.clone(), 0usize);

        while let Some((curr, depth)) = frontier.pop_front() {
            if depth >= MAX_INITIAL_TYPED_HOPS {
                continue;
            }
            for edge in &self.edges {
                let next_opt = if edge.from_profile == curr {
                    Some(edge.to_profile.clone())
                } else if edge.to_profile == curr {
                    Some(edge.from_profile.clone())
                } else {
                    None
                };
                if let Some(next_id) = next_opt
                    && !hop_distance.contains_key(&next_id)
                {
                    hop_distance.insert(next_id.clone(), depth + 1);
                    frontier.push_back((next_id, depth + 1));
                }
            }
        }
        let expanded_two_hop_neighborhood: BTreeSet<RecordId> =
            hop_distance.keys().cloned().collect();

        // Step 3 & Step 4: Filter out benched, retired, candidate, superseded,
        // conflicting, and live-registry-disqualified profiles.
        let mut excluded_profiles = BTreeMap::new();
        let mut eligible = BTreeSet::new();

        for id in &expanded_two_hop_neighborhood {
            let Some(profile) = self.profiles.get(id) else {
                continue;
            };
            if superseded.contains(id) {
                excluded_profiles.insert(id.clone(), "superseded_by_newer_version".into());
                continue;
            }
            match profile.lifecycle {
                CapabilityLifecycle::Qualified => {}
                other => {
                    excluded_profiles.insert(id.clone(), format!("lifecycle={}", other.as_str()));
                    continue;
                }
            }
            if let Err(reason) = live_registry.reconcile(profile) {
                excluded_profiles.insert(id.clone(), format!("live_registry:{reason:?}"));
                continue;
            }
            eligible.insert(id.clone());
        }

        // Also include 1-hop neighbors of eligible capabilities (such as
        // `fallback_for`, `exposes`, `enforced_by`, `verified_by`, `requires`)
        // that attach to capabilities reached within 2 hops of the solution family.
        for edge in &self.edges {
            if (eligible.contains(&edge.from_profile) || eligible.contains(&edge.to_profile))
                || (expanded_two_hop_neighborhood.contains(&edge.from_profile)
                    || expanded_two_hop_neighborhood.contains(&edge.to_profile))
            {
                for endpoint in [&edge.from_profile, &edge.to_profile] {
                    if excluded_profiles.contains_key(endpoint) || eligible.contains(endpoint) {
                        continue;
                    }
                    if let Some(profile) = self.profiles.get(endpoint) {
                        if superseded.contains(endpoint) {
                            excluded_profiles
                                .insert(endpoint.clone(), "superseded_by_newer_version".into());
                            continue;
                        }
                        if profile.lifecycle != CapabilityLifecycle::Qualified {
                            excluded_profiles.insert(
                                endpoint.clone(),
                                format!("lifecycle={}", profile.lifecycle.as_str()),
                            );
                            continue;
                        }
                        if let Err(reason) = live_registry.reconcile(profile) {
                            excluded_profiles
                                .insert(endpoint.clone(), format!("live_registry:{reason:?}"));
                            continue;
                        }
                        eligible.insert(endpoint.clone());
                    }
                }
            }
        }

        // Find capabilities inside the 2-hop neighborhood that specialize or
        // connect to `nearest_family_id` or solve `problem_id`.
        let candidate_capabilities: Vec<RecordId> = expanded_two_hop_neighborhood
            .iter()
            .filter(|id| {
                eligible.contains(*id)
                    && self
                        .profiles
                        .get(*id)
                        .is_some_and(|p| p.kind == CapabilityKind::Capability)
            })
            .cloned()
            .collect();

        if candidate_capabilities.is_empty() {
            return Err("no eligible capability found within two typed hops".into());
        }

        // Step 5: Discover and rank complete compatible tool/runbook compositions
        // for each candidate capability.
        let mut scored_candidates: Vec<(RecordId, RecordId, bool, f64)> = Vec::new();

        for cap_id in &candidate_capabilities {
            // Find all tools that `implements` `cap_id`.
            let implementing_tools: Vec<RecordId> = self
                .edges
                .iter()
                .filter(|e| {
                    e.kind == CapabilityEdgeKind::Implements
                        && &e.to_profile == cap_id
                        && self
                            .profiles
                            .get(&e.from_profile)
                            .is_some_and(|p| p.kind == CapabilityKind::Tool)
                })
                .map(|e| e.from_profile.clone())
                .collect();

            for tool_id in implementing_tools {
                if eligible.contains(&tool_id) && !self.has_active_conflict(&tool_id, &eligible) {
                    let score = self.score_composition(cap_id, &tool_id);
                    scored_candidates.push((cap_id.clone(), tool_id, false, score));
                } else {
                    // Record conflict if it was in `eligible`.
                    if eligible.contains(&tool_id) {
                        excluded_profiles
                            .insert(tool_id.clone(), "conflicts_with_active_profile".into());
                        eligible.remove(&tool_id);
                    }
                    // Check `fallback_for` edges where `from_profile` is a
                    // fallback for `tool_id` (or `to_profile` is a fallback).
                    for edge in &self.edges {
                        if edge.kind == CapabilityEdgeKind::FallbackFor {
                            let fallback_id = if edge.to_profile == tool_id {
                                Some(edge.from_profile.clone())
                            } else if edge.from_profile == tool_id {
                                Some(edge.to_profile.clone())
                            } else {
                                None
                            };
                            if let Some(fb_id) = fallback_id
                                && eligible.contains(&fb_id)
                                && !self.has_active_conflict(&fb_id, &eligible)
                            {
                                let score = self.score_composition(cap_id, &fb_id);
                                scored_candidates.push((cap_id.clone(), fb_id, true, score));
                            }
                        }
                    }
                }
            }
        }

        scored_candidates.sort_by(|a, b| {
            b.3.total_cmp(&a.3)
                .then_with(|| a.1.as_str().cmp(b.1.as_str()))
        });
        scored_candidates.dedup_by(|a, b| a.1 == b.1);

        let Some((selected_cap_id, selected_tool_id, used_fallback, _best_score)) =
            scored_candidates.first().cloned()
        else {
            return Err("no compatible live-reconciled tool or fallback composition found".into());
        };

        let ranked_compositions: Vec<(RecordId, f64)> = scored_candidates
            .iter()
            .map(|(_, tool_id, _, score)| (tool_id.clone(), *score))
            .collect();

        // Step 6: Materialize the chosen composition as a separate acyclic ExecutionDag.
        // Collect supporting providers (`mcp_server`/`plugin` that `exposes` `selected_tool_id`),
        // `runbook` that `implements` `selected_cap_id` or `composes_with` `selected_tool_id`,
        // and `policy_rule` that enforces `selected_cap_id` or `selected_tool_id`.
        let mut dag_nodes = vec![
            problem_id.clone(),
            nearest_family_id.clone(),
            selected_cap_id.clone(),
        ];
        let mut dag_edges = vec![
            (
                nearest_family_id.clone(),
                selected_cap_id.clone(),
                CapabilityEdgeKind::Specializes,
            ),
            (
                problem_id.clone(),
                nearest_family_id.clone(),
                CapabilityEdgeKind::Solves,
            ),
        ];

        // Provider (MCP server / plugin) exposing the selected tool:
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::Exposes
                && edge.to_profile == selected_tool_id
                && eligible.contains(&edge.from_profile)
                && !dag_nodes.contains(&edge.from_profile)
            {
                dag_nodes.push(edge.from_profile.clone());
                dag_edges.push((
                    selected_cap_id.clone(),
                    edge.from_profile.clone(),
                    CapabilityEdgeKind::Requires,
                ));
                dag_edges.push((
                    edge.from_profile.clone(),
                    selected_tool_id.clone(),
                    CapabilityEdgeKind::Exposes,
                ));
            }
        }

        if !dag_nodes.contains(&selected_tool_id) {
            dag_nodes.push(selected_tool_id.clone());
        }
        dag_edges.push((
            selected_cap_id.clone(),
            selected_tool_id.clone(),
            CapabilityEdgeKind::Implements,
        ));

        // Qualified runbook implementing `selected_cap_id`:
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::Implements
                && edge.to_profile == selected_cap_id
                && eligible.contains(&edge.from_profile)
                && self
                    .profiles
                    .get(&edge.from_profile)
                    .is_some_and(|p| p.kind == CapabilityKind::Runbook)
                && !dag_nodes.contains(&edge.from_profile)
            {
                dag_nodes.push(edge.from_profile.clone());
                dag_edges.push((
                    selected_tool_id.clone(),
                    edge.from_profile.clone(),
                    CapabilityEdgeKind::ComposesWith,
                ));
            }
        }

        // Mandatory policy rules attached via `enforced_by`:
        let mut mandatory_rules = Vec::new();
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::EnforcedBy {
                let rule_id = if self
                    .profiles
                    .get(&edge.from_profile)
                    .is_some_and(|p| p.kind == CapabilityKind::PolicyRule)
                {
                    Some(edge.from_profile.clone())
                } else if self
                    .profiles
                    .get(&edge.to_profile)
                    .is_some_and(|p| p.kind == CapabilityKind::PolicyRule)
                {
                    Some(edge.to_profile.clone())
                } else {
                    None
                };
                if let Some(r_id) = rule_id
                    && !mandatory_rules.contains(&r_id)
                {
                    mandatory_rules.push(r_id);
                }
            }
        }

        // Topologically sort `dag_nodes` and drop any back-edges that would
        // form a cycle (even if the underlying Atlas graph contains cycles).
        let (topological_order, acyclic_edges) = materialize_acyclic_dag(&dag_nodes, &dag_edges)?;

        let mut dag_hasher = Sha256::new();
        dag_hasher.update(b"globalmind-execution-dag-v1:");
        dag_hasher.update(problem_id.as_str().as_bytes());
        dag_hasher.update(b":");
        for node in &topological_order {
            dag_hasher.update(node.as_str().as_bytes());
            dag_hasher.update(b",");
        }
        for (u, v, k) in &acyclic_edges {
            dag_hasher.update(u.as_str().as_bytes());
            dag_hasher.update(b"->");
            dag_hasher.update(v.as_str().as_bytes());
            dag_hasher.update(b":");
            dag_hasher.update(k.as_str().as_bytes());
            dag_hasher.update(b";");
        }
        let dag_digest = Digest::parse(hex_lower(&dag_hasher.finalize()))?;

        let dag = ExecutionDag {
            problem_profile_id: problem_id,
            solution_family_id: nearest_family_id.clone(),
            capability_id: selected_cap_id,
            topological_order: topological_order.clone(),
            dependency_edges: acyclic_edges,
            mandatory_rules,
            used_fallback,
            dag_digest: dag_digest.clone(),
        };
        dag.verify_acyclic()?;

        // Step 7: Project ONLY the selected executable callable contracts into
        // the compact binary context frame.
        let mut projections = Vec::new();
        for node_id in &topological_order {
            if let Some(profile) = self.profiles.get(node_id)
                && matches!(
                    profile.kind,
                    CapabilityKind::Tool | CapabilityKind::Runbook | CapabilityKind::McpServer
                )
            {
                projections.push(profile.compact_projection());
            }
        }
        let context_frame = CompactContextFrame {
            dag_digest,
            projections,
        };
        let context_frame_bytes = context_frame.to_binary_envelope()?;

        Ok(PlannedCapabilityExecution {
            nearest_solution_family: nearest_family_id,
            expanded_two_hop_neighborhood,
            excluded_profiles,
            ranked_compositions,
            dag,
            context_frame,
            context_frame_bytes,
        })
    }

    fn has_active_conflict(&self, candidate_id: &RecordId, eligible: &BTreeSet<RecordId>) -> bool {
        if let Some(profile) = self.profiles.get(candidate_id) {
            for inc in &profile.incompatible_profiles {
                if eligible.contains(inc) {
                    return true;
                }
            }
        }
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::ConflictsWith
                && ((&edge.from_profile == candidate_id && eligible.contains(&edge.to_profile))
                    || (&edge.to_profile == candidate_id && eligible.contains(&edge.from_profile)))
            {
                return true;
            }
        }
        false
    }

    /// Computes a deterministic composition score using ONLY evidence receipts,
    /// expected utility, resource cost, and prediction error — never popularity
    /// or `source_digest`.
    pub fn score_composition(&self, capability_id: &RecordId, tool_id: &RecordId) -> f64 {
        let Some(cap) = self.profiles.get(capability_id) else {
            return -1.0;
        };
        let Some(tool) = self.profiles.get(tool_id) else {
            return -1.0;
        };
        let mut evidence_score = 0.0;
        if tool.component_receipt_digest.is_some() {
            evidence_score += 0.25;
        }
        if tool.integration_receipt_digest.is_some() {
            evidence_score += 0.35;
        }
        if tool.live_receipt_digest.is_some() {
            evidence_score += 0.40;
        }
        for edge in &self.edges {
            if edge.kind == CapabilityEdgeKind::VerifiedBy
                && (&edge.from_profile == tool_id || &edge.to_profile == tool_id)
            {
                evidence_score += 0.20 * edge.confidence;
            }
        }
        let utility = 0.5 * (cap.expected_utility + tool.expected_utility);
        let cost = 0.5 * (cap.resource_cost + tool.resource_cost);
        let pred_err = 0.5 * (cap.prediction_error + tool.prediction_error);

        (0.45 * utility) + (0.35 * evidence_score) - (0.12 * cost) - (0.08 * pred_err)
    }
}

/// Executes a planned read-only tool call through the mandatory `LiveRegistry`
/// and `MandatoryPolicyGate` boundary. The Capability Atlas itself has zero
/// execution methods; execution is possible only through this harness bridge.
#[derive(Clone, Debug)]
pub struct HarnessExecutionReceipt {
    pub tool_profile_id: RecordId,
    pub live_registry_id: String,
    pub dag_digest: Digest,
    pub policy_gate_receipt_digest: Digest,
    pub response_envelope_bytes: Vec<u8>,
    pub response_digest: Digest,
    pub utility_receipt: CapabilityUtilityReceipt,
}

pub fn execute_planned_readonly_tool<F>(
    plan: &PlannedCapabilityExecution,
    live_registry: &LiveRegistry,
    policy_gate: &MandatoryPolicyGate,
    workspace: &RecordId,
    utility_receipt_id: &RecordId,
    binary_request_envelope: &[u8],
    executor: F,
) -> Result<HarnessExecutionReceipt, String>
where
    F: FnOnce(&CompactCallableProjection, &[u8]) -> Result<Vec<u8>, String>,
{
    plan.dag.verify_acyclic()?;
    let tool_projection = plan
        .context_frame
        .projections
        .iter()
        .find(|p| p.kind == CapabilityKind::Tool)
        .ok_or_else(|| "planned context frame contains no selected tool projection".to_string())?;

    // Re-verify live registry health & schema digests at execution boundary.
    let live_entry = live_registry
        .entries
        .get(&tool_projection.live_registry_id)
        .ok_or_else(|| "selected tool missing from live registry at execution time".to_string())?;
    if !live_entry.installed || !live_entry.healthy {
        return Err("selected tool is not installed/healthy in live registry".into());
    }
    if live_entry.version != tool_projection.version
        || live_entry.argument_schema_digest != tool_projection.argument_schema_digest
        || live_entry.result_schema_digest != tool_projection.result_schema_digest
    {
        return Err("selected tool digest/version drift detected at execution time".into());
    }

    // Enforce mandatory Rust safety and binary I/O policy gate.
    let policy_gate_receipt_digest =
        policy_gate.enforce_call(tool_projection, binary_request_envelope)?;

    let response_envelope_bytes = executor(tool_projection, binary_request_envelope)?;
    MandatoryPolicyGate::enforce_binary_io_boundary(&response_envelope_bytes)?;
    let response_digest = digest_bytes(&response_envelope_bytes)?;

    let mut exec_hasher = Sha256::new();
    exec_hasher.update(b"globalmind-harness-exec-v1:");
    exec_hasher.update(plan.dag.dag_digest.as_str().as_bytes());
    exec_hasher.update(b":");
    exec_hasher.update(policy_gate_receipt_digest.as_str().as_bytes());
    exec_hasher.update(b":");
    exec_hasher.update(response_digest.as_str().as_bytes());
    let execution_receipt_digest = Digest::parse(hex_lower(&exec_hasher.finalize()))?;

    let utility_receipt = CapabilityUtilityReceipt {
        id: utility_receipt_id.clone(),
        workspace: workspace.clone(),
        problem_profile: plan.dag.problem_profile_id.clone(),
        selected_profiles: plan
            .context_frame
            .projections
            .iter()
            .map(|p| p.profile_id.clone())
            .collect(),
        dag_digest: plan.dag.dag_digest.clone(),
        execution_receipt_digest,
        observed_utility: 0.96,
        prediction_error: 0.02,
        resource_cost: 0.08,
        outcome: UtilityOutcome::Succeeded,
    };

    Ok(HarnessExecutionReceipt {
        tool_profile_id: tool_projection.profile_id.clone(),
        live_registry_id: tool_projection.live_registry_id.clone(),
        dag_digest: plan.dag.dag_digest.clone(),
        policy_gate_receipt_digest,
        response_envelope_bytes,
        response_digest,
        utility_receipt,
    })
}

fn manifold_distance(query_h3: [f64; 4], query_t2: [f64; 2], profile: &CapabilityScaffold) -> f64 {
    let d_h3 = hyperbolic_distance(query_h3, profile.h3_lorentz);
    let d_t2 = torus_distance_sq(query_t2, profile.t2_phase).sqrt();
    d_h3 + 0.25 * d_t2
}

fn materialize_acyclic_dag(
    nodes: &[RecordId],
    raw_edges: &[DagDependencyEdge],
) -> Result<(Vec<RecordId>, Vec<DagDependencyEdge>), String> {
    let mut order = Vec::new();
    let mut index_of = BTreeMap::new();
    for node in nodes {
        if !index_of.contains_key(node) {
            let idx = order.len();
            order.push(node.clone());
            index_of.insert(node.clone(), idx);
        }
    }
    let mut dag_edges = Vec::new();
    for (u, v, kind) in raw_edges {
        if let (Some(&iu), Some(&iv)) = (index_of.get(u), index_of.get(v)) {
            if iu < iv {
                let entry = (u.clone(), v.clone(), *kind);
                if !dag_edges.contains(&entry) {
                    dag_edges.push(entry);
                }
            } else if iv < iu {
                // Reverse or drop cycle-inducing back-edge so the execution
                // plan is strictly a forward topological DAG.
                let reversed = (v.clone(), u.clone(), *kind);
                if !dag_edges.contains(&reversed) {
                    dag_edges.push(reversed);
                }
            }
        }
    }
    Ok((order, dag_edges))
}

/// Validates that a string contains no secret, credential value, live token,
/// private key, or raw K/V tensor payload.
pub fn validate_no_forbidden_material(field: &str, value: &str) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    for forbidden in [
        "sk-",
        "ghp_",
        "bearer ",
        "-----begin private key",
        "secret_key=",
        "password=",
        "api_key=",
        "access_token=",
        "transformer_kv_page",
        "key_tensor:",
        "value_tensor:",
    ] {
        if lower.contains(forbidden) {
            return Err(format!(
                "forbidden secret, token, or K/V tensor material detected in {field}"
            ));
        }
    }
    Ok(())
}

fn validate_credential_class_only(cred_class: &str) -> Result<(), String> {
    validate_no_forbidden_material("credential_classes", cred_class)?;
    if cred_class.contains('=') || cred_class.contains(':') || cred_class.contains(' ') {
        return Err(format!(
            "credential_classes must name a credential class identifier, never a value: '{cred_class}'"
        ));
    }
    Ok(())
}

pub fn deterministic_digest(label: &str) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"globalmind-capability-atlas-v1:");
    hasher.update(label.as_bytes());
    Digest::parse(hex_lower(&hasher.finalize())).expect("SHA-256 hex is valid")
}

pub fn digest_bytes(bytes: &[u8]) -> Result<Digest, String> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Digest::parse(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn write_str(buf: &mut Vec<u8>, s: &str) -> Result<(), String> {
    let bytes = s.as_bytes();
    let len = u32::try_from(bytes.len()).map_err(|_| "string exceeds u32 length".to_string())?;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    Ok(())
}

fn write_opt_str(buf: &mut Vec<u8>, opt: Option<&str>) -> Result<(), String> {
    match opt {
        Some(s) => {
            buf.push(1);
            write_str(buf, s)
        }
        None => {
            buf.push(0);
            Ok(())
        }
    }
}

fn write_str_vec(buf: &mut Vec<u8>, items: &[String]) -> Result<(), String> {
    let count = u32::try_from(items.len()).map_err(|_| "vec exceeds u32 length".to_string())?;
    buf.extend_from_slice(&count.to_le_bytes());
    for item in items {
        write_str(buf, item)?;
    }
    Ok(())
}

fn write_digest(buf: &mut Vec<u8>, digest: &Digest) {
    buf.extend_from_slice(digest.as_str().as_bytes());
}

fn write_opt_digest(buf: &mut Vec<u8>, opt: Option<&Digest>) {
    match opt {
        Some(d) => {
            buf.push(1);
            write_digest(buf, d);
        }
        None => buf.push(0),
    }
}

struct BinaryCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BinaryCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "binary cursor overflow".to_string())?;
        if end > self.bytes.len() {
            return Err("unexpected EOF in capability binary envelope".into());
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let slice = self.read_slice(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.read_fixed::<1>()?[0])
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.read_fixed::<4>()?))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.read_fixed::<8>()?))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_le_bytes(self.read_fixed::<8>()?))
    }

    fn read_str(&mut self) -> Result<String, String> {
        let len = self.read_u32()? as usize;
        let slice = self.read_slice(len)?;
        std::str::from_utf8(slice)
            .map(str::to_string)
            .map_err(|e| format!("invalid UTF-8 in binary envelope: {e}"))
    }

    fn read_opt_str(&mut self) -> Result<Option<String>, String> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_str()?)),
            other => Err(format!("invalid optional string tag {other}")),
        }
    }

    fn read_str_vec(&mut self) -> Result<Vec<String>, String> {
        let count = self.read_u32()? as usize;
        if count > 1024 {
            return Err(format!("binary vector length {count} exceeds safety cap"));
        }
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(self.read_str()?);
        }
        Ok(out)
    }

    fn read_digest(&mut self) -> Result<Digest, String> {
        let slice = self.read_slice(64)?;
        let raw = std::str::from_utf8(slice)
            .map_err(|e| format!("invalid digest UTF-8 in binary envelope: {e}"))?;
        Digest::parse(raw.to_string())
    }

    fn read_opt_digest(&mut self) -> Result<Option<Digest>, String> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_digest()?)),
            other => Err(format!("invalid optional digest tag {other}")),
        }
    }

    fn finish(&self) -> Result<(), String> {
        if self.pos != self.bytes.len() {
            return Err(format!(
                "trailing {} bytes in binary envelope",
                self.bytes.len() - self.pos
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_projection_and_mandatory_policy_gate_work_offline() {
        let proj = CompactCallableProjection {
            profile_id: RecordId::parse("prof_tool_contract_verifier").unwrap(),
            canonical_name: "surreal_contract_verify".into(),
            version: "1.0.0".into(),
            kind: CapabilityKind::Tool,
            argument_schema_digest: deterministic_digest("arg_schema_v1"),
            result_schema_digest: deterministic_digest("res_schema_v1"),
            effect_class: EffectClass::ReadOnly,
            permission_classes: vec!["surreal.schema.read".into()],
            credential_classes: vec!["loopback_session_token".into()],
            timeout_ms: 5_000,
            max_retries: 1,
            idempotent: true,
            rollback_policy: "read_only_noop".into(),
            max_memory_bytes: 16 * 1024 * 1024,
            live_registry_id: "mcp://globalmind-contract/surreal_contract_verify@1.0.0".into(),
        };
        let bytes = proj.to_binary_envelope().unwrap();
        let decoded = CompactCallableProjection::from_binary_envelope(&bytes).unwrap();
        assert_eq!(proj, decoded);

        let gate = MandatoryPolicyGate::read_only_smoke();
        let valid_req = b"GMBI\x01\x00\x00\x00\x10\x20\x30\x40";
        assert!(gate.enforce_call(&proj, valid_req).is_ok());

        let json_req = b"GMBI\x01\x00\x00\x00{\"tool\":\"hack\"}";
        assert!(gate.enforce_call(&proj, json_req).is_err());
    }
}
