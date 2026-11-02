//! Phase 0 core-governance acceptance tests. Each maps to an acceptance line
//! in the core governance spec. Failing test first, per gate.

use ares_backend::compliance::{
    surgical_erase_atom, ComplianceViolation, CoreScopeChange, ImmuneSystem, OverlapPolicyMatrix,
    PreregisteredEvaluation, PREREGISTERED_KILL_FINGERPRINT,
};
use ares_backend::digest::Sha256Digest;
use ares_backend::first_principles::{FirstPrinciplesComplianceEngine, FIRST_PRINCIPLE_STATUTES};
use ares_backend::fixed::Q32_31;
use ares_backend::merkle::ComplianceMerkleTree;
use ares_backend::mlir::{
    MlirAtomCatalog, MlirOpDescriptor, MlirSsaBlock, MlirTypeSig, NativeMlirInstruction,
};
use ares_backend::roles::RoleId;
use ares_backend::store::{
    AdapterLossReport, BridgeLossReport, CarveLossReport, CarvedSwarmMap, EphemeralMapDeposit,
    FrozenBaseWeights, LorentzCoord, ProjectionLossReport, RefractionLossReport,
};

fn base_map() -> (CarvedSwarmMap, Sha256Digest) {
    let base = FrozenBaseWeights::freeze("test-model", 2, vec![0.25f32; 64]);
    let fp = base.fingerprint;
    (CarvedSwarmMap::new(base, Q32_31::from_f64(1.22)), fp)
}

fn immune(fp: Sha256Digest) -> ImmuneSystem {
    ImmuneSystem::new(
        OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08)),
        fp,
    )
}

#[test]
fn dispatch_without_permit_impossible_and_chain_recomputes() {
    let (map, fp) = base_map();
    let mut sys = immune(fp);
    // A permit can only come from the gate.
    let permit = sys
        .authorize_role_spawn(RoleId::Judge, &map)
        .expect("spawn should pass");
    assert_eq!(permit.sequence(), sys.sequence());
    sys.verify_chain().expect("chain must recompute to head");

    // Tamper with the log: chain verification must fail (startup fail-closed).
    let mut entries = sys.audit_log().to_vec();
    // Clone-and-tamper via a fresh system replaying a corrupted entry is not
    // constructible externally (fields are pub for audit, digest is checked).
    entries[0].action = "FORGED PASS".to_string();
    let head = sys.head_digest();
    assert!(ares_backend::compliance::verify_audit_chain(&entries, &head).is_err());
}

#[test]
fn modified_weight_fingerprint_fails_gate1() {
    let (mut map, fp) = base_map();
    // Mutate the frozen base by replacing it with different weights.
    let tampered = FrozenBaseWeights::freeze("test-model", 2, vec![0.5f32; 64]);
    assert_ne!(tampered.fingerprint, fp);
    map.base_weights = std::sync::Arc::new(tampered);
    let mut sys = immune(fp);
    let err = sys.check_base_weights(&map).unwrap_err();
    assert!(matches!(
        err,
        ComplianceViolation::BaseWeightsModified { .. }
    ));
    // The rejection is itself an audit entry.
    assert!(sys.audit_log().last().unwrap().action.contains("REJECT"));
    assert!(!sys.audit_log().last().unwrap().approved);
}

#[test]
fn planted_out_of_view_row_returns_zero_hits() {
    let (mut map, fp) = base_map();
    let policy = OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08));
    // Plant an atom homed at MissionIntake (L1).
    let atom = map.keep_and_carve(
        LorentzCoord::origin(),
        b"secret".to_vec(),
        RoleId::MissionIntake,
        &policy,
    );
    // Antipodal view (GateKeeper L12, non-adjacent, Omega = 0) must not see it.
    assert!(!map.silo(RoleId::GateKeeper).unwrap().contains_atom(atom));
    // And a handoff L1 -> L12 carrying it must fail: the path does not exist.
    let mut sys = immune(fp);
    let err = sys
        .authorize_handoff(RoleId::MissionIntake, RoleId::GateKeeper, atom, &map)
        .unwrap_err();
    assert!(matches!(
        err,
        ComplianceViolation::AtomInvisibleToSender { .. }
            | ComplianceViolation::UnauthorizedRolePair { .. }
            | ComplianceViolation::AtomOutsideBoundaryOverlap { .. }
    ));
    // Adjacent handoff L1 -> L2 (PacketFormer) with Omega > 0 passes.
    let mut sys2 = immune(fp);
    sys2.authorize_handoff(RoleId::MissionIntake, RoleId::PacketFormer, atom, &map)
        .expect("adjacent handoff should pass");
}

#[test]
fn policy_expansion_without_justification_fails() {
    let mut policy = OverlapPolicyMatrix::zero_overlap();
    let err = policy
        .set_pair_overlap(
            RoleId::MissionIntake,
            RoleId::GateKeeper,
            Q32_31::from_f64(0.1),
            "  ",
        )
        .unwrap_err();
    assert!(matches!(
        err,
        ComplianceViolation::UnjustifiedOverlapExpansion { .. }
    ));
    let err2 = policy
        .set_pair_overlap(
            RoleId::MissionIntake,
            RoleId::GateKeeper,
            Q32_31::from_f64(-0.1),
            "reason",
        )
        .unwrap_err();
    assert!(matches!(
        err2,
        ComplianceViolation::NegativeOverlapAllowance { .. }
    ));
}

#[test]
fn core_scope_change_without_human_permit_fails() {
    let (_map, fp) = base_map();
    let mut sys = immune(fp);
    let err = sys
        .authorize_core_scope_change(&CoreScopeChange::ModelSwap, None)
        .unwrap_err();
    assert!(matches!(
        err,
        ComplianceViolation::HumanPermitRequired { .. }
    ));

    // With a human permit it passes — exactly once (single-use).
    let hp = sys.issue_human_permit(&CoreScopeChange::ModelSwap, "operator-stewart");
    sys.authorize_core_scope_change(&CoreScopeChange::ModelSwap, Some(&hp))
        .expect("human-permitted change should pass");
    let err2 = sys
        .authorize_core_scope_change(&CoreScopeChange::ModelSwap, Some(&hp))
        .unwrap_err();
    assert!(matches!(
        err2,
        ComplianceViolation::HumanPermitInvalid { .. }
    ));

    // Wrong-scope permit fails.
    let hp2 = sys.issue_human_permit(&CoreScopeChange::PolicyExpansion, "operator-stewart");
    let err3 = sys
        .authorize_core_scope_change(&CoreScopeChange::ModelSwap, Some(&hp2))
        .unwrap_err();
    assert!(matches!(
        err3,
        ComplianceViolation::HumanPermitInvalid { .. }
    ));
}

#[test]
fn erasure_produces_pre_post_roots_and_zero_residual() {
    let (mut map, _fp) = base_map();
    let policy = OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08));
    let atom = map.keep_and_carve(
        LorentzCoord::origin(),
        b"erase-me".to_vec(),
        RoleId::Judge,
        &policy,
    );
    let receipt = surgical_erase_atom(&mut map, atom, &policy).expect("erasure should succeed");
    assert!(receipt.excised_from_global);
    assert_eq!(receipt.residual_occurrences_across_13_maps, 0);
    assert_ne!(
        receipt.pre_erasure_merkle_root,
        receipt.post_erasure_merkle_root
    );
    // Erasing an absent atom fails.
    assert!(surgical_erase_atom(&mut map, atom, &policy).is_err());
}

#[test]
fn merkle_vault_odd_carry_forward_and_verify() {
    // Odd leaf count exercises carry-forward (no self-duplication).
    let payloads: Vec<Vec<u8>> = (0..5u8).map(|i| vec![i; 8]).collect();
    let tree = ComplianceMerkleTree::from_payloads(&payloads);
    assert!(ComplianceMerkleTree::verify(&tree.root, &tree.leaves));
    // A tampered leaf must not verify against the root.
    let mut bad = tree.leaves.clone();
    bad[0] = Sha256Digest([9u8; 32]);
    assert!(!ComplianceMerkleTree::verify(&tree.root, &bad));
    // Single leaf: root is the leaf itself under carry-forward.
    let one = ComplianceMerkleTree::from_payloads(&[b"x".to_vec()]);
    assert_eq!(one.root, one.leaves[0]);
    // Empty: domain seed.
    let empty = ComplianceMerkleTree::from_payloads(&[]);
    assert_eq!(
        empty.root,
        Sha256Digest::compute(b"EMPTY_FIRST_PRINCIPLES_MERKLE_VAULT")
    );
}

#[test]
fn kill_criteria_preregistered_fingerprint_and_bars() {
    let (_map, fp) = base_map();
    let mut sys = immune(fp);
    let mut kl = [Q32_31::from_f64(0.10); 24];
    kl[20] = Q32_31::from_f64(0.01);
    kl[21] = Q32_31::from_f64(0.01);
    kl[22] = Q32_31::from_f64(0.01);
    kl[23] = Q32_31::from_f64(0.01);
    // 20/24 pass >= 5%, CI > 0, correct fingerprint -> PASS.
    let good = PreregisteredEvaluation {
        test_fingerprint: PREREGISTERED_KILL_FINGERPRINT,
        family_kl_reductions_q32: kl,
        co_recall_ci95_lower_q32: Q32_31::from_f64(0.02),
    };
    sys.evaluate_kill_criteria(&good)
        .expect("kill criteria should pass");

    // Wrong fingerprint fails by design, even with passing numbers.
    let bad_fp = PreregisteredEvaluation {
        test_fingerprint: 0x1234,
        ..good.clone()
    };
    let mut sys2 = immune(fp);
    assert!(matches!(
        sys2.evaluate_kill_criteria(&bad_fp).unwrap_err(),
        ComplianceViolation::LensFalsifiedByKillCriteria { .. }
    ));

    // Too few families fails.
    let mut kl2 = [Q32_31::from_f64(0.01); 24];
    kl2[0] = Q32_31::from_f64(0.5);
    let few = PreregisteredEvaluation {
        test_fingerprint: PREREGISTERED_KILL_FINGERPRINT,
        family_kl_reductions_q32: kl2,
        co_recall_ci95_lower_q32: Q32_31::from_f64(0.02),
    };
    let mut sys3 = immune(fp);
    assert!(sys3.evaluate_kill_criteria(&few).is_err());
}

#[test]
fn loss_accounting_and_forward_pass_and_supervision_cap() {
    let (map, fp) = base_map();
    let mut sys = immune(fp);
    // Gate 4: zero input bits = silent loss.
    let err = sys
        .verify_loss_accounting(
            &ProjectionLossReport {
                input_state_bits: 0,
                output_tangent_bits: 256,
                discarded_dimensions: 1,
            },
            &RefractionLossReport {
                refractive_index_q32: Q32_31::from_f64(2.0),
            },
            &CarveLossReport {
                global_atom_count: 1,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        ComplianceViolation::SilentInformationLoss { .. }
    ));

    // Gate 6: RoPE integrity 0 = silent loss.
    let err6 = sys
        .authorize_forward_pass(
            RoleId::LaneBuilder,
            7,
            0,
            &map,
            &BridgeLossReport {
                input_dimensions: 128,
                rope_pair_integrity_q32: Q32_31::ZERO,
                retained_5d_energy_ratio_q32: Q32_31::HALF,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err6,
        ComplianceViolation::SilentInformationLoss { .. }
    ));

    // Gate 7 enforcement (source gap closed): over-cap restarts rejected.
    let err7 = sys
        .authorize_governor_supervision(1, 4, 3, &map)
        .unwrap_err();
    assert!(matches!(
        err7,
        ComplianceViolation::RestartCapExceeded { .. }
    ));
    sys.authorize_governor_supervision(2, 3, 3, &map)
        .expect("at-cap supervision passes");
}

#[test]
fn deposit_and_cross_epoch_and_hot_swap_gates() {
    let (mut map, fp) = base_map();
    let mut sys = immune(fp);

    // Gate 8: invalid hyperboloid coordinate rejected.
    let bad = EphemeralMapDeposit {
        from_role: RoleId::LaneBuilder,
        dissolved_pid: 1,
        records: vec![LorentzCoord {
            x0: 2.0,
            x1: 0.0,
            x2: 0.0,
            x3: 0.0,
        }],
        forward_steps_completed: 1,
    };
    assert!(matches!(
        sys.authorize_deposit_compilation(&bad, &map).unwrap_err(),
        ComplianceViolation::SilentInformationLoss { .. }
    ));
    let good = EphemeralMapDeposit {
        from_role: RoleId::LaneBuilder,
        dissolved_pid: 1,
        records: vec![LorentzCoord::origin()],
        forward_steps_completed: 1,
    };
    sys.authorize_deposit_compilation(&good, &map)
        .expect("valid deposit passes");

    // Gate 10: odd head dim rejected; unregistered lineage rejected.
    let loss = AdapterLossReport {
        origin_fingerprint: fp,
        target_fingerprint: fp,
        source_head_dim: 63,
        target_head_dim: 64,
        rope_pair_integrity_q32: Q32_31::ONE,
        retained_k_energy_ratio_q32: Q32_31::HALF,
        retained_v_energy_ratio_q32: Q32_31::HALF,
        is_zero_copy_passthrough: false,
    };
    assert!(sys
        .authorize_cross_epoch_adaptation(RoleId::Judge, 1, &loss, &map)
        .is_err());
    let loss_ok = AdapterLossReport {
        source_head_dim: 64,
        ..loss
    };
    sys.authorize_cross_epoch_adaptation(RoleId::Judge, 1, &loss_ok, &map)
        .expect("valid cross-epoch passes");
    map.register_model_fingerprint(fp);

    // Gate 9: hot swap re-pins; externally-fingerprinted weights fail closed.
    let new_weights = FrozenBaseWeights::freeze("new-model", 2, vec![0.75f32; 32]);
    let new_fp = new_weights.fingerprint;
    sys.authorize_model_hot_swap(&new_weights, 3)
        .expect("hot swap passes");
    assert_eq!(sys.pinned_weight_fingerprint, new_fp);
    let ext =
        FrozenBaseWeights::freeze_with_fingerprint("ext", 1, vec![1.0], Sha256Digest([7u8; 32]));
    assert!(!ext.verify_integrity());
}

#[test]
fn mlir_ssa_gate_checks_dominance_arity_types_role() {
    let (map, fp) = base_map();
    let catalog = MlirAtomCatalog::new(
        1,
        vec![
            MlirOpDescriptor {
                op_id: 1,
                mnemonic: "const.q32".into(),
                arg_types: vec![],
                ret_type: MlirTypeSig::Q32_31,
                required_role: None,
            },
            MlirOpDescriptor {
                op_id: 2,
                mnemonic: "func.return_q32".into(),
                arg_types: vec![MlirTypeSig::Q32_31],
                ret_type: MlirTypeSig::Q32_31,
                required_role: None,
            },
            MlirOpDescriptor {
                op_id: 3,
                mnemonic: "judge.only".into(),
                arg_types: vec![MlirTypeSig::Q32_31],
                ret_type: MlirTypeSig::Q32_31,
                required_role: Some(RoleId::Judge),
            },
        ],
    );
    let good = MlirSsaBlock::new(vec![
        NativeMlirInstruction {
            op_id: 1,
            dst_reg: 0,
            src_regs: [0; 4],
            arity: 0,
        },
        NativeMlirInstruction {
            op_id: 2,
            dst_reg: 1,
            src_regs: [0, 0, 0, 0],
            arity: 1,
        },
    ]);
    let mut sys = immune(fp);
    let (receipt, _permit) = sys
        .authorize_mlir_ssa_block(RoleId::LaneBuilder, RoleId::Judge, &good, &catalog, &map)
        .expect("valid SSA block passes");
    assert_eq!(receipt.instruction_count, 2);

    // Use-before-def fails.
    let bad = MlirSsaBlock::new(vec![NativeMlirInstruction {
        op_id: 2,
        dst_reg: 1,
        src_regs: [9, 0, 0, 0],
        arity: 1,
    }]);
    let mut sys2 = immune(fp);
    assert!(matches!(
        sys2.authorize_mlir_ssa_block(RoleId::LaneBuilder, RoleId::Judge, &bad, &catalog, &map)
            .unwrap_err(),
        ComplianceViolation::InvalidMlirSsaBlock { .. }
    ));

    // Role capability: judge-only op from a non-judge sender fails.
    let role_bad = MlirSsaBlock::new(vec![
        NativeMlirInstruction {
            op_id: 1,
            dst_reg: 0,
            src_regs: [0; 4],
            arity: 0,
        },
        NativeMlirInstruction {
            op_id: 3,
            dst_reg: 1,
            src_regs: [0, 0, 0, 0],
            arity: 1,
        },
        NativeMlirInstruction {
            op_id: 2,
            dst_reg: 2,
            src_regs: [1, 0, 0, 0],
            arity: 1,
        },
    ]);
    let mut sys3 = immune(fp);
    assert!(sys3
        .authorize_mlir_ssa_block(
            RoleId::LaneBuilder,
            RoleId::Judge,
            &role_bad,
            &catalog,
            &map
        )
        .is_err());
}

#[test]
fn statutory_registry_twelve_rules_evaluated_from_live_state() {
    assert_eq!(FIRST_PRINCIPLE_STATUTES.len(), 12);
    let (map, _fp) = base_map();
    let policy = OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08));
    let digests = vec![Sha256Digest::compute(b"audit_event_1")];
    let report = FirstPrinciplesComplianceEngine::prove_from_live_state(&map, &policy, &digests);
    // All 8 invariants provable from live backend state in this fixture.
    assert_eq!(
        report.verified_invariants.len(),
        8,
        "invariants: {:?}",
        report.verified_invariants
    );
    assert!(
        report.all_statutes_satisfied,
        "verdicts: {:?}",
        report.rule_verdicts
    );
    // With no audit digests, the Merkle invariant is unproven and its rules
    // are NOT credited (no label registry).
    let report2 = FirstPrinciplesComplianceEngine::prove_from_live_state(&map, &policy, &[]);
    assert!(!report2.all_statutes_satisfied);
    let aia12 = report2
        .rule_verdicts
        .iter()
        .find(|v| v.id == "AIA-ART-12")
        .unwrap();
    assert!(!aia12.satisfied);
}
