//! Layer 3 — Immune System & structural compliance automatons (backend-native).
//!
//! Governs the Governor: the backend run governor cannot dispatch or route
//! without presenting a `CompliancePermit` minted by `ImmuneSystem`.
//! Structural gates, not prompt filters: every gate is a deterministic state
//! machine over digests, the overlap policy matrix, and view membership.
//!
//! Source design: `monster-hybrid-server/immune_system/implementation/src/
//! compliance.rs` (884 lines). This module implements the same law in the
//! backend's own types — it does not import the Monster crate.
//!
//! Two disclosed deviations from the source, both fail-closed:
//! 1. Gate 7 (governor supervision) ENFORCES the restart cap. The source
//!    records the parameters and discards them (`let _ = (...)`); the core
//!    governance spec forbids silently copying that gap.
//! 2. Human oversight (Gate 12, backend addition): core-scope changes
//!    require a human permit. The system can propose; it cannot self-approve.

use crate::digest::Sha256Digest;
use crate::fixed::Q32_31;
use crate::merkle::ComplianceMerkleTree;
use crate::mlir::{MlirAtomCatalog, MlirSsaBlock, MlirSsaValidationReceipt};
use crate::roles::{EpistemicMark, RoleId};
use crate::store::{
    AdapterLossReport, AtomId, BridgeLossReport, CarveLossReport, CarvedSwarmMap,
    EphemeralMapDeposit, FrozenBaseWeights, LorentzCoord, ProjectionLossReport,
    RefractionLossReport,
};
use std::fmt;

/// Preregistered falsification test fingerprint (`0x94ce_0283_17e0_ecac`),
/// kept verbatim from the extraction. A different fingerprint fails Gate 5
/// by design.
pub const PREREGISTERED_KILL_FINGERPRINT: u64 = 0x94ce_0283_17e0_ecac;

/// Pairwise overlap policy matrix Ω over the 12 carved lanes.
///
/// `matrix[r][s]` = allowed boundary expansion Ω(r,s) ≥ 0 beyond role r's
/// private home basin. Default policy: minimal lane adjacency — Ω = 0.08
/// only between workflow-adjacent lanes, Ω = 0 for all non-adjacent /
/// antipodal pairs. Strict policy: `zero_overlap()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlapPolicyMatrix {
    matrix: [[Q32_31; 12]; 12],
}

impl OverlapPolicyMatrix {
    pub const fn zero_overlap() -> Self {
        Self {
            matrix: [[Q32_31::ZERO; 12]; 12],
        }
    }

    /// Default minimal policy: `neighbour_omega` only between lanes that are
    /// neighbours in the backend's lane-adjacency relation.
    pub fn minimal_adjacent(neighbour_omega: Q32_31) -> Self {
        let mut matrix = [[Q32_31::ZERO; 12]; 12];
        for (r_idx, r) in RoleId::CARVED_12.iter().enumerate() {
            for (s_idx, s) in RoleId::CARVED_12.iter().enumerate() {
                if crate::roles::lane_neighbours(*r, *s) {
                    matrix[r_idx][s_idx] = neighbour_omega;
                }
            }
        }
        Self { matrix }
    }

    pub fn overlap_allowance(&self, a: RoleId, b: RoleId) -> Q32_31 {
        let (Some(i), Some(j)) = (a.carved_index(), b.carved_index()) else {
            return Q32_31::ZERO;
        };
        self.matrix[i][j]
    }

    /// Expand overlap symmetrically. Requires a non-empty architectural
    /// justification, or it fails with `UnjustifiedOverlapExpansion`.
    /// A negative Ω fails with `NegativeOverlapAllowance`.
    pub fn set_pair_overlap(
        &mut self,
        a: RoleId,
        b: RoleId,
        omega: Q32_31,
        justification: &str,
    ) -> Result<(), ComplianceViolation> {
        if justification.trim().is_empty() {
            return Err(ComplianceViolation::UnjustifiedOverlapExpansion {
                role_a: a,
                role_b: b,
            });
        }
        if omega.0 < 0 {
            return Err(ComplianceViolation::NegativeOverlapAllowance {
                role_a: a,
                role_b: b,
            });
        }
        let (Some(i), Some(j)) = (a.carved_index(), b.carved_index()) else {
            return Err(ComplianceViolation::InvalidSpecialistRole(
                RoleId::Coordinator,
            ));
        };
        self.matrix[i][j] = omega;
        self.matrix[j][i] = omega;
        Ok(())
    }
}

/// Structural compliance violations. Every violation is typed and names the
/// gate, the roles, and the atom it broke — never a stringly "failed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceViolation {
    BaseWeightsModified {
        expected: Sha256Digest,
        found: Sha256Digest,
    },
    InvalidSpecialistRole(RoleId),
    UnjustifiedOverlapExpansion {
        role_a: RoleId,
        role_b: RoleId,
    },
    NegativeOverlapAllowance {
        role_a: RoleId,
        role_b: RoleId,
    },
    UnauthorizedRolePair {
        from: RoleId,
        to: RoleId,
    },
    AtomInvisibleToSender {
        sender: RoleId,
        atom_id: AtomId,
    },
    AtomOutsideBoundaryOverlap {
        from: RoleId,
        to: RoleId,
        atom_id: AtomId,
    },
    SilentInformationLoss {
        stage: &'static str,
    },
    LensFalsifiedByKillCriteria {
        fingerprint: u64,
        families_passed: u8,
        required_families: u8,
        co_recall_ci_lower_q32: Q32_31,
    },
    InvalidMlirSsaBlock {
        reason: String,
    },
    /// Backend addition (Gate 7 enforcement): restart cap exceeded.
    RestartCapExceeded {
        active_restarts_in_window: u32,
        max_restarts: u32,
    },
    /// Backend addition (Gate 12): core-scope change without a human permit.
    HumanPermitRequired {
        scope: String,
    },
    /// Backend addition (Gate 12): human permit presented but not valid for
    /// this scope (wrong scope, or permit already consumed).
    HumanPermitInvalid {
        scope: String,
    },
}

impl fmt::Display for ComplianceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseWeightsModified { expected, found } => write!(
                f,
                "BaseWeightImmutabilityGate violation: base weights modified (expected {}, found {})",
                expected, found
            ),
            Self::InvalidSpecialistRole(r) => {
                write!(f, "Role {:?} is not one of the 12 carved lanes", r)
            }
            Self::UnjustifiedOverlapExpansion { role_a, role_b } => write!(
                f,
                "SiloBoundary law violation: overlap expansion between {:?} and {:?} lacks justification",
                role_a, role_b
            ),
            Self::NegativeOverlapAllowance { role_a, role_b } => write!(
                f,
                "Negative overlap allowance between {:?} and {:?}",
                role_a, role_b
            ),
            Self::UnauthorizedRolePair { from, to } => write!(
                f,
                "BoundaryHandoffGate: unauthorized handoff path {:?} -> {:?} (zero policy overlap — the path does not exist)",
                from, to
            ),
            Self::AtomInvisibleToSender { sender, atom_id } => write!(
                f,
                "BoundaryHandoffGate: atom {} does not exist in sender {:?} carved view",
                atom_id, sender
            ),
            Self::AtomOutsideBoundaryOverlap { from, to, atom_id } => write!(
                f,
                "BoundaryHandoffGate: atom {} is not in the shared boundary basin between {:?} and {:?}",
                atom_id, from, to
            ),
            Self::SilentInformationLoss { stage } => write!(
                f,
                "LossAccounting violation: silent/unmeasured information loss at {}",
                stage
            ),
            Self::LensFalsifiedByKillCriteria {
                fingerprint,
                families_passed,
                required_families,
                co_recall_ci_lower_q32,
            } => write!(
                f,
                "KillCriteriaGate TRIGGERED (test {:016x}): passed {}/24 families (need >= {}), co-recall 95% CI lower = {:.4}",
                fingerprint,
                families_passed,
                required_families,
                co_recall_ci_lower_q32.to_f64()
            ),
            Self::InvalidMlirSsaBlock { reason } => {
                write!(f, "MlirNativeSsaGate rejected block: {}", reason)
            }
            Self::RestartCapExceeded {
                active_restarts_in_window,
                max_restarts,
            } => write!(
                f,
                "GovernorSupervisionGate: restart cap exceeded ({}/{} restarts in window)",
                active_restarts_in_window, max_restarts
            ),
            Self::HumanPermitRequired { scope } => write!(
                f,
                "HumanOversightGate: core-scope change '{}' requires a human permit",
                scope
            ),
            Self::HumanPermitInvalid { scope } => write!(
                f,
                "HumanOversightGate: human permit invalid for core-scope change '{}'",
                scope
            ),
        }
    }
}

impl std::error::Error for ComplianceViolation {}

/// Input to the preregistered Kill Criteria gate (Gate 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreregisteredEvaluation {
    pub test_fingerprint: u64,
    /// Relative KL reduction per held-out family, Q32.31. Bar: ≥ 0.05 on ≥ 17 of 24.
    pub family_kl_reductions_q32: [Q32_31; 24],
    /// Lower bound of the 95% CI for co-recall difference, Q32.31. Bar: > 0.
    pub co_recall_ci95_lower_q32: Q32_31,
}

/// Unforgeable capability token minted exclusively by `ImmuneSystem`.
///
/// `sequence` and `chain_digest` are private to this module: neither the
/// Governor nor any lane can construct a permit without passing through
/// `ImmuneSystem`. The unauthorized path does not exist; it is not merely
/// blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompliancePermit {
    sequence: u64,
    action_summary: String,
    epistemic_mark: EpistemicMark,
    chain_digest: Sha256Digest,
}

impl CompliancePermit {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn action_summary(&self) -> &str {
        &self.action_summary
    }
    pub fn epistemic_mark(&self) -> EpistemicMark {
        self.epistemic_mark
    }
    pub fn chain_digest(&self) -> Sha256Digest {
        self.chain_digest
    }
}

/// Hash-chained audit entry recorded for every checked dispatch, pass or
/// reject. A rejection is an audit entry too, with `approved: false`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ComplianceAuditEntry {
    pub sequence: u64,
    pub action: String,
    pub approved: bool,
    pub mark: EpistemicMark,
    pub prev_digest: Sha256Digest,
    pub entry_digest: Sha256Digest,
}

impl serde::Serialize for Sha256Digest {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Sha256Digest {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Sha256Digest::from_hex(&s).ok_or_else(|| serde::de::Error::custom("bad digest hex"))
    }
}

/// Entry digest, exactly as implemented in the source:
/// `SHA-256( sequence LE || approved u8 || mark tag || prev_digest || action )`.
pub fn compute_entry_digest(
    sequence: u64,
    approved: bool,
    mark: EpistemicMark,
    prev_digest: &Sha256Digest,
    action: &str,
) -> Sha256Digest {
    let mut buf = Vec::with_capacity(48 + action.len());
    buf.extend_from_slice(&sequence.to_le_bytes());
    buf.push(u8::from(approved));
    buf.extend_from_slice(mark.as_tag().as_bytes());
    buf.extend_from_slice(&prev_digest.0);
    buf.extend_from_slice(action.as_bytes());
    Sha256Digest::compute(&buf)
}

/// Verify a full audit chain from genesis: sequences 1..=n in order, each
/// entry's digest recomputes, each `prev_digest` links to the prior entry,
/// and the final digest equals `expected_head`. This is the startup gate —
/// a broken chain is a startup failure, not a data issue.
pub fn verify_audit_chain(
    entries: &[ComplianceAuditEntry],
    expected_head: &Sha256Digest,
) -> Result<(), String> {
    let mut prev = Sha256Digest([0u8; 32]);
    for (i, e) in entries.iter().enumerate() {
        let want_seq = (i as u64) + 1;
        if e.sequence != want_seq {
            return Err(format!(
                "audit chain: entry {} has sequence {}, expected {}",
                i, e.sequence, want_seq
            ));
        }
        if e.prev_digest != prev {
            return Err(format!(
                "audit chain: entry {} prev_digest does not link to prior head",
                e.sequence
            ));
        }
        let recomputed =
            compute_entry_digest(e.sequence, e.approved, e.mark, &e.prev_digest, &e.action);
        if recomputed != e.entry_digest {
            return Err(format!(
                "audit chain: entry {} digest mismatch (log tampered)",
                e.sequence
            ));
        }
        prev = e.entry_digest;
    }
    if prev != *expected_head {
        return Err("audit chain: recomputed head does not equal stored head digest".to_string());
    }
    Ok(())
}

/// Core-scope change classes that require a human permit (Gate 12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreScopeChange {
    ThreadRunDefinition,
    ModelSwap,
    SiloViewChange,
    PolicyExpansion,
    ControlRegistryChange,
}

impl CoreScopeChange {
    pub fn scope_label(&self) -> String {
        match self {
            Self::ThreadRunDefinition => "thread_run_definition".to_string(),
            Self::ModelSwap => "model_swap".to_string(),
            Self::SiloViewChange => "silo_view_change".to_string(),
            Self::PolicyExpansion => "policy_expansion".to_string(),
            Self::ControlRegistryChange => "control_registry_change".to_string(),
        }
    }
}

/// A human-issued authorization for one core-scope change. Single-use:
/// minted by `issue_human_permit` (the operator action), consumed by
/// `authorize_core_scope_change`. The backend can propose a change; it
/// cannot mint this token for itself — issuance is an explicit operator
/// call, recorded in the audit chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanPermit {
    scope: String,
    issued_by: String,
    sequence: u64,
}

impl HumanPermit {
    /// Reconstruct a token from an audit-chain issuance entry (HTTP layer).
    /// Forgery is still impossible: `authorize_core_scope_change` re-checks
    /// that the sequence exists in the chain as an ISSUE entry and that it
    /// has not been consumed. A client-invented sequence fails both checks.
    #[doc(hidden)]
    pub fn __from_chain_unchecked(scope: String, issued_by: String, sequence: u64) -> Self {
        Self {
            scope,
            issued_by,
            sequence,
        }
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }
    pub fn issued_by(&self) -> &str {
        &self.issued_by
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// Layer 3 — the Immune System. Sole minter of permits.
#[derive(Debug)]
pub struct ImmuneSystem {
    pub policy: OverlapPolicyMatrix,
    pub pinned_weight_fingerprint: Sha256Digest,
    sequence: u64,
    head_digest: Sha256Digest,
    audit_log: Vec<ComplianceAuditEntry>,
}

impl ImmuneSystem {
    pub fn new(policy: OverlapPolicyMatrix, pinned_weight_fingerprint: Sha256Digest) -> Self {
        Self {
            policy,
            pinned_weight_fingerprint,
            sequence: 0,
            head_digest: Sha256Digest([0u8; 32]),
            audit_log: Vec::new(),
        }
    }

    /// Restore from a persisted audit log (startup). Verifies the chain
    /// before returning; a broken chain is an error, never a warning.
    pub fn restore(
        policy: OverlapPolicyMatrix,
        pinned_weight_fingerprint: Sha256Digest,
        entries: Vec<ComplianceAuditEntry>,
        stored_head: Sha256Digest,
    ) -> Result<Self, String> {
        verify_audit_chain(&entries, &stored_head)?;
        let sequence = entries.last().map(|e| e.sequence).unwrap_or(0);
        Ok(Self {
            policy,
            pinned_weight_fingerprint,
            sequence,
            head_digest: stored_head,
            audit_log: entries,
        })
    }

    pub fn audit_log(&self) -> &[ComplianceAuditEntry] {
        &self.audit_log
    }

    pub fn head_digest(&self) -> Sha256Digest {
        self.head_digest
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Recompute the chain from the in-memory log and compare to head.
    pub fn verify_chain(&self) -> Result<(), String> {
        verify_audit_chain(&self.audit_log, &self.head_digest)
    }

    /// Gate 1: BaseWeightImmutability. SHA-256 of base weights (layer count
    /// + weights, -0.0 normalised) must equal the pinned fingerprint.
    pub fn check_base_weights(
        &mut self,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let actual = FrozenBaseWeights::hash_f32_slice(
            map.base_weights.num_layers,
            map.base_weights.raw_weights(),
        );
        if actual != self.pinned_weight_fingerprint {
            let err = ComplianceViolation::BaseWeightsModified {
                expected: self.pinned_weight_fingerprint,
                found: actual,
            };
            self.record_audit(
                format!("REJECT base_weights: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!("PASS BaseWeightImmutabilityGate ({})", actual),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 2: SiloBoundary (role spawn). Base weights pass; role is the
    /// Coordinator or a carved lane with a silo; the silo's weight view is
    /// the same `Arc` as the frozen base (zero-copy).
    pub fn authorize_role_spawn(
        &mut self,
        role: RoleId,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        self.check_base_weights(map)?;
        if role == RoleId::Coordinator {
            return Ok(self.mint_permit(
                "PASS SpawnRole(L0 Coordinator)".to_string(),
                EpistemicMark::Verified,
            ));
        }
        let Some(silo) = map.silo(role) else {
            let err = ComplianceViolation::InvalidSpecialistRole(role);
            self.record_audit(
                format!("REJECT spawn: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        };
        if !std::sync::Arc::ptr_eq(&silo.weight_view.base, &map.base_weights) {
            let err = ComplianceViolation::BaseWeightsModified {
                expected: self.pinned_weight_fingerprint,
                found: silo.weight_view.base.fingerprint,
            };
            self.record_audit(
                format!("REJECT spawn weight ptr: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!(
                "PASS SiloBoundaryGate SpawnRole(L{} {}, atoms={})",
                role.level(),
                role.id_slug(),
                silo.atom_count()
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 3: BoundaryHandoff. Base weights pass; from/to distinct; atom
    /// exists in sender's view; Ω(from,to) > 0; atom exists in receiver's
    /// view (shared boundary basin). Coordinator handoffs still require the
    /// specialist side to contain the atom.
    pub fn authorize_handoff(
        &mut self,
        from: RoleId,
        to: RoleId,
        atom_id: AtomId,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        self.check_base_weights(map)?;

        if from == RoleId::Coordinator || to == RoleId::Coordinator {
            let specialist = if from == RoleId::Coordinator {
                to
            } else {
                from
            };
            if specialist != RoleId::Coordinator {
                let silo = map
                    .silo(specialist)
                    .ok_or(ComplianceViolation::InvalidSpecialistRole(specialist))?;
                if !silo.contains_atom(atom_id) {
                    let err = ComplianceViolation::AtomOutsideBoundaryOverlap { from, to, atom_id };
                    self.record_audit(
                        format!("REJECT coordinator handoff: {}", err),
                        false,
                        EpistemicMark::Verified,
                    );
                    return Err(err);
                }
            }
            return Ok(self.mint_permit(
                format!(
                    "PASS BoundaryHandoffGate L{} -> L{} atom={}",
                    from.level(),
                    to.level(),
                    atom_id
                ),
                EpistemicMark::Verified,
            ));
        }

        if from == to {
            let err = ComplianceViolation::UnauthorizedRolePair { from, to };
            self.record_audit(
                format!("REJECT handoff self-pair: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        let silo_from = map
            .silo(from)
            .ok_or(ComplianceViolation::InvalidSpecialistRole(from))?;
        if !silo_from.contains_atom(atom_id) {
            let err = ComplianceViolation::AtomInvisibleToSender {
                sender: from,
                atom_id,
            };
            self.record_audit(
                format!("REJECT handoff sender: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        let omega = self.policy.overlap_allowance(from, to);
        if omega.0 <= 0 {
            let err = ComplianceViolation::UnauthorizedRolePair { from, to };
            self.record_audit(
                format!("REJECT handoff policy: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        let silo_to = map
            .silo(to)
            .ok_or(ComplianceViolation::InvalidSpecialistRole(to))?;
        if !silo_to.contains_atom(atom_id) {
            let err = ComplianceViolation::AtomOutsideBoundaryOverlap { from, to, atom_id };
            self.record_audit(
                format!("REJECT handoff boundary: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        Ok(self.mint_permit(
            format!(
                "PASS BoundaryHandoffGate L{}({}) -> L{}({}) atom={} omega={:.4}",
                from.level(),
                from.id_slug(),
                to.level(),
                to.id_slug(),
                atom_id,
                omega.to_f64()
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 4: LossAccounting. Projection, refraction, and carve stages all
    /// explicitly name and measure loss.
    pub fn verify_loss_accounting(
        &mut self,
        proj: &ProjectionLossReport,
        refr: &RefractionLossReport,
        carve: &CarveLossReport,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        if proj.input_state_bits == 0 || proj.output_tangent_bits == 0 {
            let err = ComplianceViolation::SilentInformationLoss {
                stage: "Stage 3 (Project)",
            };
            self.record_audit(
                format!("REJECT loss gate: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        if refr.refractive_index_q32.0 < Q32_31::ONE.0 {
            let err = ComplianceViolation::SilentInformationLoss {
                stage: "Stage 4 (Refract optical index n(r) < 1)",
            };
            self.record_audit(
                format!("REJECT loss gate: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        if carve.global_atom_count == 0 {
            let err = ComplianceViolation::SilentInformationLoss {
                stage: "Stage 5 (Carve empty global map)",
            };
            self.record_audit(
                format!("REJECT loss gate: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!(
                "PASS LossAccountingGate (in={}b -> out={}b, discarded_dims={}, global_atoms={})",
                proj.input_state_bits,
                proj.output_tangent_bits,
                proj.discarded_dimensions,
                carve.global_atom_count
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 5: KillCriteria. Preregistered fingerprint exactly
    /// `0x94ce_0283_17e0_ecac`; ≥17 of 24 families with ≥5% KL reduction;
    /// co-recall 95% CI lower bound > 0.
    pub fn evaluate_kill_criteria(
        &mut self,
        eval: &PreregisteredEvaluation,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let five_percent = Q32_31::from_f64(0.05);
        let families_passed = eval
            .family_kl_reductions_q32
            .iter()
            .filter(|&&kl_red| kl_red.0 >= five_percent.0)
            .count() as u8;

        if eval.test_fingerprint != PREREGISTERED_KILL_FINGERPRINT
            || families_passed < 17
            || eval.co_recall_ci95_lower_q32.0 <= 0
        {
            let err = ComplianceViolation::LensFalsifiedByKillCriteria {
                fingerprint: eval.test_fingerprint,
                families_passed,
                required_families: 17,
                co_recall_ci_lower_q32: eval.co_recall_ci95_lower_q32,
            };
            self.record_audit(
                format!("KILL_CRITERIA_FALSIFIED: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        Ok(self.mint_permit(
            format!(
                "PASS KillCriteriaGate ({:016x}: {}/24 families >= 5% KL reduction, CI95 > 0)",
                eval.test_fingerprint, families_passed
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 6: ForwardPass. Base weights pass; bridge loss input dimensions
    /// > 0 and RoPE pair integrity > 0.
    pub fn authorize_forward_pass(
        &mut self,
        role: RoleId,
        token_id: u32,
        pos: usize,
        map: &CarvedSwarmMap,
        bridge_loss: &BridgeLossReport,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let _weight_permit = self.check_base_weights(map)?;
        if bridge_loss.input_dimensions == 0 || bridge_loss.rope_pair_integrity_q32.0 <= 0 {
            let err = ComplianceViolation::SilentInformationLoss {
                stage: "bridge::f32_to_atom_address (Forward Pass)",
            };
            self.record_audit(
                format!(
                    "REJECT forward pass role={} pos={}: {}",
                    role.name(),
                    pos,
                    err
                ),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!(
                "PASS ForwardPassGate(role={}, token={}, pos={}, dims={}->5, retained_5d={:.4})",
                role.name(),
                token_id,
                pos,
                bridge_loss.input_dimensions,
                bridge_loss.retained_5d_energy_ratio_q32.to_f64()
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 7: GovernorSupervision. Base weights pass; supervision tick
    /// recorded with restarts-in-window.
    ///
    /// DEVIATION FROM SOURCE (fail-closed, disclosed): the source records
    /// `active_restarts_in_window`/`max_restarts` and discards them. This
    /// backend enforces the cap: restarts above the cap are rejected with
    /// `RestartCapExceeded`, and the rejection is audited.
    pub fn authorize_governor_supervision(
        &mut self,
        tick: u64,
        active_restarts_in_window: u32,
        max_restarts: u32,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let _weight_permit = self.check_base_weights(map)?;
        if active_restarts_in_window > max_restarts {
            let err = ComplianceViolation::RestartCapExceeded {
                active_restarts_in_window,
                max_restarts,
            };
            self.record_audit(
                format!("REJECT supervision tick={}: {}", tick, err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!(
                "PASS GovernorSupervisionGate(tick={}, restarts_in_window={}/{})",
                tick, active_restarts_in_window, max_restarts
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 8: DepositCompilation. Base weights pass; every deposited atom
    /// validates on the hyperboloid (x0² − ‖x‖² = 1).
    pub fn authorize_deposit_compilation(
        &mut self,
        deposit: &EphemeralMapDeposit,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let _weight_permit = self.check_base_weights(map)?;
        for rec in &deposit.records {
            if rec.validate().is_err() {
                let err = ComplianceViolation::SilentInformationLoss {
                    stage: "EphemeralMapDeposit invalid hyperboloid coordinate",
                };
                self.record_audit(
                    format!(
                        "REJECT DepositCompilationGate role={} pid={}: {}",
                        deposit.from_role.name(),
                        deposit.dissolved_pid,
                        err
                    ),
                    false,
                    EpistemicMark::Verified,
                );
                return Err(err);
            }
        }
        Ok(self.mint_permit(
            format!(
                "PASS DepositCompilationGate(role={}, pid={}, atoms={}, steps={})",
                deposit.from_role.name(),
                deposit.dissolved_pid,
                deposit.records.len(),
                deposit.forward_steps_completed
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 9: ModelHotSwap. New weights verify integrity; the pinned
    /// fingerprint is then re-pinned to the new model, with preserved-atom
    /// count recorded in the permit.
    pub fn authorize_model_hot_swap(
        &mut self,
        new_weights: &FrozenBaseWeights,
        preserved_atoms: usize,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        if !new_weights.verify_integrity() {
            let err = ComplianceViolation::BaseWeightsModified {
                expected: new_weights.fingerprint,
                found: Sha256Digest([0u8; 32]),
            };
            self.record_audit(
                format!("REJECT ModelHotSwapGate: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        let prev_fp = self.pinned_weight_fingerprint;
        self.pinned_weight_fingerprint = new_weights.fingerprint;
        Ok(self.mint_permit(
            format!(
                "PASS ModelHotSwapGate(new_model={}, layers={}, prev_fp={}..->new_fp={}.., preserved_atoms={})",
                new_weights.model_name,
                new_weights.num_layers,
                &prev_fp.to_hex()[..12],
                &new_weights.fingerprint.to_hex()[..12],
                preserved_atoms
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 10: CrossEpochAdapter. Base weights pass; origin and target
    /// fingerprints are in the registered model lineage; head dims are
    /// positive even numbers; RoPE pair integrity == 1; retained K energy > 0.
    pub fn authorize_cross_epoch_adaptation(
        &mut self,
        role: RoleId,
        atom_id: AtomId,
        loss: &AdapterLossReport,
        map: &CarvedSwarmMap,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let _weight_permit = self.check_base_weights(map)?;

        if !map.is_registered_model_fingerprint(&loss.origin_fingerprint)
            || loss.target_fingerprint != map.base_weights.fingerprint
        {
            let err = ComplianceViolation::BaseWeightsModified {
                expected: map.base_weights.fingerprint,
                found: loss.origin_fingerprint,
            };
            self.record_audit(
                format!(
                    "REJECT CrossEpochAdapterGate role={} atom={}: unregistered origin/target model lineage",
                    role.name(),
                    atom_id
                ),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        if loss.source_head_dim == 0
            || !loss.source_head_dim.is_multiple_of(2)
            || loss.target_head_dim == 0
            || !loss.target_head_dim.is_multiple_of(2)
            || loss.rope_pair_integrity_q32 != Q32_31::ONE
            || loss.retained_k_energy_ratio_q32.0 <= 0
        {
            let err = ComplianceViolation::SilentInformationLoss {
                stage: "ModelAdapterLens::adapt_kv_payload (CrossEpochAdapterGate)",
            };
            self.record_audit(
                format!(
                    "REJECT CrossEpochAdapterGate role={} atom={}: {}",
                    role.name(),
                    atom_id,
                    err
                ),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        Ok(self.mint_permit(
            format!(
                "PASS CrossEpochAdapterGate(role={}, atom={}, {}d->{}d, retained_k={:.4}, retained_v={:.4}, passthrough={})",
                role.name(),
                atom_id,
                loss.source_head_dim,
                loss.target_head_dim,
                loss.retained_k_energy_ratio_q32.to_f64(),
                loss.retained_v_energy_ratio_q32.to_f64(),
                loss.is_zero_copy_passthrough
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Gate 11: MlirNativeSsaGate. Base weights pass; atom catalog SHA-256
    /// verifies; block passes SSA dominance, exact operand arity, and static
    /// type compatibility, for the owning role. Returns receipt + permit.
    pub fn authorize_mlir_ssa_block(
        &mut self,
        sender: RoleId,
        receiver: RoleId,
        block: &MlirSsaBlock,
        catalog: &MlirAtomCatalog,
        map: &CarvedSwarmMap,
    ) -> Result<(MlirSsaValidationReceipt, CompliancePermit), ComplianceViolation> {
        let _weight_permit = self.check_base_weights(map)?;
        if !catalog.verify_integrity() {
            let err = ComplianceViolation::InvalidMlirSsaBlock {
                reason: "MlirAtomCatalog SHA-256 integrity check failed".to_string(),
            };
            self.record_audit(
                format!("REJECT MlirNativeSsaGate: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }

        let receipt = block.validate_ssa_and_types(catalog, sender).map_err(|e| {
            let err = ComplianceViolation::InvalidMlirSsaBlock {
                reason: e.to_string(),
            };
            self.record_audit(
                format!(
                    "REJECT MlirNativeSsaGate L{} -> L{}: {}",
                    sender.level(),
                    receiver.level(),
                    err
                ),
                false,
                EpistemicMark::Verified,
            );
            err
        })?;

        let permit = self.mint_permit(
            format!(
                "PASS MlirNativeSsaGate(L{}->L{}, ops={}, ret={:?}, digest={}..)",
                sender.level(),
                receiver.level(),
                receipt.instruction_count,
                receipt.final_return_type,
                &receipt.block_digest.to_hex()[..12]
            ),
            EpistemicMark::Verified,
        );
        Ok((receipt, permit))
    }

    /// Issue a human permit for one core-scope change (Gate 12 issuance).
    /// This is the operator action; it is recorded in the audit chain.
    /// The backend never calls this on its own behalf.
    pub fn issue_human_permit(&mut self, scope: &CoreScopeChange, issued_by: &str) -> HumanPermit {
        let label = scope.scope_label();
        let entry = self.record_audit(
            format!(
                "ISSUE HumanPermit(scope={}, issued_by={})",
                label, issued_by
            ),
            true,
            EpistemicMark::Verified,
        );
        HumanPermit {
            scope: label,
            issued_by: issued_by.to_string(),
            sequence: entry.sequence,
        }
    }

    /// Gate 12: HumanOversight. A core-scope change proceeds only with a
    /// valid, unconsumed human permit for exactly that scope. Permits are
    /// single-use: the audit entry that consumes one is the proof.
    pub fn authorize_core_scope_change(
        &mut self,
        change: &CoreScopeChange,
        human_permit: Option<&HumanPermit>,
    ) -> Result<CompliancePermit, ComplianceViolation> {
        let label = change.scope_label();
        let Some(hp) = human_permit else {
            let err = ComplianceViolation::HumanPermitRequired {
                scope: label.clone(),
            };
            self.record_audit(
                format!("REJECT core-scope change: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        };
        if hp.scope != label {
            let err = ComplianceViolation::HumanPermitInvalid {
                scope: label.clone(),
            };
            self.record_audit(
                format!("REJECT core-scope change: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        // Single-use: the permit's issuance entry must exist in the chain
        // and must not already have been consumed by an earlier PASS.
        let issued = self
            .audit_log
            .iter()
            .any(|e| e.sequence == hp.sequence && e.action.starts_with("ISSUE HumanPermit"));
        let consumed = self.audit_log.iter().any(|e| {
            e.approved
                && e.action
                    .contains(&format!("human_permit_seq={}", hp.sequence))
                && e.action.starts_with("PASS HumanOversightGate")
        });
        if !issued || consumed {
            let err = ComplianceViolation::HumanPermitInvalid {
                scope: label.clone(),
            };
            self.record_audit(
                format!("REJECT core-scope change: {}", err),
                false,
                EpistemicMark::Verified,
            );
            return Err(err);
        }
        Ok(self.mint_permit(
            format!(
                "PASS HumanOversightGate(scope={}, human_permit_seq={}, issued_by={})",
                label, hp.sequence, hp.issued_by
            ),
            EpistemicMark::Verified,
        ))
    }

    /// Record a governance incident (broken chain, modified fingerprint,
    /// failed residual scan). GDPR Art. 33/34 posture: an incident is an
    /// audit entry with a receipt, never a silent log line.
    pub fn record_governance_incident(&mut self, description: &str) -> ComplianceAuditEntry {
        self.record_audit(
            format!("GOVERNANCE_INCIDENT: {}", description),
            false,
            EpistemicMark::Verified,
        )
    }

    fn mint_permit(&mut self, action_summary: String, mark: EpistemicMark) -> CompliancePermit {
        let entry = self.record_audit(action_summary.clone(), true, mark);
        CompliancePermit {
            sequence: entry.sequence,
            action_summary,
            epistemic_mark: mark,
            chain_digest: entry.entry_digest,
        }
    }

    fn record_audit(
        &mut self,
        action: String,
        approved: bool,
        mark: EpistemicMark,
    ) -> ComplianceAuditEntry {
        self.sequence += 1;
        let prev_digest = self.head_digest;
        let entry_digest =
            compute_entry_digest(self.sequence, approved, mark, &prev_digest, &action);
        self.head_digest = entry_digest;
        let entry = ComplianceAuditEntry {
            sequence: self.sequence,
            action,
            approved,
            mark,
            prev_digest,
            entry_digest,
        };
        self.audit_log.push(entry.clone());
        entry
    }
}

/// Surgical erasure receipt (GDPR Art. 17 shape, from the extraction).
/// The receipt carries the residual count either way — it does not hide a
/// non-zero residual.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SurgicalErasureReceipt {
    pub erased_atom_id: AtomId,
    pub excised_from_global: bool,
    pub silos_excised_count: usize,
    pub residual_occurrences_across_13_maps: usize,
    pub pre_erasure_merkle_root: Sha256Digest,
    pub post_erasure_merkle_root: Sha256Digest,
    pub mark: EpistemicMark,
}

/// Execute surgical erasure against the carved store:
/// record pre-root → count silos containing the atom → remove from global
/// (fail if absent) → re-carve all 12 views → count residuals across
/// global + 12 views (a correct erasure reports 0) → record post-root.
/// The (pre, post) pair is the transition proof.
pub fn surgical_erase_atom(
    map: &mut CarvedSwarmMap,
    atom_id: AtomId,
    policy: &OverlapPolicyMatrix,
) -> Result<SurgicalErasureReceipt, &'static str> {
    let pre_root = map.compute_swarm_merkle_root();
    let mut s_count = 0usize;
    for role in RoleId::CARVED_12 {
        if let Some(silo) = map.silo(role) {
            if silo.contains_atom(atom_id) {
                s_count += 1;
            }
        }
    }

    if !map.remove_global_atom_internal(atom_id) {
        return Err("AtomId not present in global map");
    }
    map.recarve_all_twelve(policy);

    let mut residual = usize::from(map.global_atom(atom_id).is_some());
    for role in RoleId::CARVED_12 {
        if let Some(silo) = map.silo(role) {
            if silo.contains_atom(atom_id) {
                residual += 1;
            }
        }
    }
    let post_root = map.compute_swarm_merkle_root();

    Ok(SurgicalErasureReceipt {
        erased_atom_id: atom_id,
        excised_from_global: true,
        silos_excised_count: s_count,
        residual_occurrences_across_13_maps: residual,
        pre_erasure_merkle_root: pre_root,
        post_erasure_merkle_root: post_root,
        mark: EpistemicMark::Verified,
    })
}

/// Merkle root over the swarm (kept for parity with the source API).
pub fn compute_swarm_merkle_root(map: &CarvedSwarmMap) -> Sha256Digest {
    ComplianceMerkleTree::from_payloads(&map.collect_merkle_leaf_bytes()).root
}

/// A deposit coordinate helper so HTTP/tests can build valid deposits
/// without depending on geometry modules.
pub fn origin_coord() -> LorentzCoord {
    LorentzCoord::origin()
}
