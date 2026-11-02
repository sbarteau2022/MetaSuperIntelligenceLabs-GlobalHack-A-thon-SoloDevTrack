//! Native SSA block + catalog (backend-native equivalents of
//! `mlir_native::MlirSsaBlock`, `MlirAtomCatalog`, `MlirTypeSig`).
//!
//! Gate 11 checks, in the source's order: catalog SHA-256 integrity, SSA
//! dominance (use-before-def, duplicate def), exact operand arity, static
//! type compatibility, and specialist role capability ownership.

use crate::digest::Sha256Digest;
use crate::roles::RoleId;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MlirTypeSig {
    Void = 0,
    I64 = 1,
    Q32_31 = 2,
    F32Vec = 3,
    LorentzCoord = 4,
    PhaseCoord = 5,
    SiloAtomHandle = 6,
    CapabilityReceipt = 7,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlirOpDescriptor {
    pub op_id: u32,
    pub mnemonic: String,
    pub arg_types: Vec<MlirTypeSig>,
    pub ret_type: MlirTypeSig,
    pub required_role: Option<RoleId>,
}

#[derive(Debug, Clone)]
pub struct MlirAtomCatalog {
    pub version: u32,
    ops: BTreeMap<u32, MlirOpDescriptor>,
    catalog_sha256: Sha256Digest,
}

impl MlirAtomCatalog {
    pub fn new(version: u32, ops: Vec<MlirOpDescriptor>) -> Self {
        let mut map = BTreeMap::new();
        for op in ops {
            map.insert(op.op_id, op);
        }
        let digest = Self::digest_ops(&map);
        Self {
            version,
            ops: map,
            catalog_sha256: digest,
        }
    }

    fn digest_ops(ops: &BTreeMap<u32, MlirOpDescriptor>) -> Sha256Digest {
        let mut buf = Vec::new();
        for (id, op) in ops {
            buf.extend_from_slice(&id.to_le_bytes());
            buf.extend_from_slice(op.mnemonic.as_bytes());
            buf.push(op.arg_types.len() as u8);
            for t in &op.arg_types {
                buf.push(*t as u8);
            }
            buf.push(op.ret_type as u8);
            buf.push(op.required_role.map(|r| r.level()).unwrap_or(255));
        }
        Sha256Digest::compute(&buf)
    }

    pub fn catalog_sha256(&self) -> Sha256Digest {
        self.catalog_sha256
    }

    pub fn verify_integrity(&self) -> bool {
        Self::digest_ops(&self.ops) == self.catalog_sha256
    }

    /// Test/governance hook: corrupt the pinned digest (Gate 11's failing
    /// case). Not used by any dispatch path.
    pub fn tamper_digest_for_test(&mut self) {
        self.catalog_sha256 = Sha256Digest([0xAB; 32]);
    }

    pub fn get_op(&self, op_id: u32) -> Option<&MlirOpDescriptor> {
        self.ops.get(&op_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMlirInstruction {
    pub op_id: u32,
    pub dst_reg: u16,
    pub src_regs: [u16; 4],
    pub arity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlirSsaValidationReceipt {
    pub instruction_count: usize,
    pub final_return_type: MlirTypeSig,
    pub block_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MlirSsaError {
    EmptyBlock,
    CatalogIntegrityFailure,
    UnknownOpcode {
        step: usize,
        op_id: u32,
    },
    DuplicateReturn {
        step: usize,
    },
    MissingReturn,
    ReturnNotLast {
        step: usize,
    },
    RoleCapabilityViolation {
        step: usize,
        op_id: u32,
        required_role: RoleId,
        actual_sender: RoleId,
    },
    ArityMismatch {
        step: usize,
        op_id: u32,
        expected: usize,
        found: usize,
    },
    UseBeforeDef {
        step: usize,
        reg: u16,
    },
    TypeMismatch {
        step: usize,
        op_id: u32,
        arg_index: usize,
        expected: MlirTypeSig,
        found: MlirTypeSig,
    },
    DuplicateSsaDefinition {
        step: usize,
        reg: u16,
    },
}

impl fmt::Display for MlirSsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBlock => write!(f, "SSA block is empty"),
            Self::CatalogIntegrityFailure => {
                write!(f, "MlirAtomCatalog SHA-256 integrity check failed")
            }
            Self::UnknownOpcode { step, op_id } => {
                write!(f, "step {}: unknown opcode {}", step, op_id)
            }
            Self::DuplicateReturn { step } => write!(f, "step {}: duplicate return", step),
            Self::MissingReturn => write!(f, "SSA block has no return"),
            Self::ReturnNotLast { step } => {
                write!(f, "step {}: return is not the last instruction", step)
            }
            Self::RoleCapabilityViolation {
                step,
                op_id,
                required_role,
                actual_sender,
            } => write!(
                f,
                "step {}: op {} requires role {:?}, sender is {:?}",
                step, op_id, required_role, actual_sender
            ),
            Self::ArityMismatch {
                step,
                op_id,
                expected,
                found,
            } => write!(
                f,
                "step {}: op {} arity mismatch (expected {}, found {})",
                step, op_id, expected, found
            ),
            Self::UseBeforeDef { step, reg } => {
                write!(f, "step {}: register {} used before definition", step, reg)
            }
            Self::TypeMismatch {
                step,
                op_id,
                arg_index,
                expected,
                found,
            } => write!(
                f,
                "step {}: op {} arg {} type mismatch (expected {:?}, found {:?})",
                step, op_id, arg_index, expected, found
            ),
            Self::DuplicateSsaDefinition { step, reg } => {
                write!(f, "step {}: register {} defined twice", step, reg)
            }
        }
    }
}

impl std::error::Error for MlirSsaError {}

pub const MLIR_SSA_WIRE_MAGIC: &[u8; 8] = b"MLIRSSA1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlirSsaBlock {
    pub instructions: Vec<NativeMlirInstruction>,
}

impl MlirSsaBlock {
    pub fn new(instructions: Vec<NativeMlirInstruction>) -> Self {
        Self { instructions }
    }

    /// Deterministic binary wire frame: magic || count (u32 LE) || per-inst
    /// `op_id u32 | dst u16 | src[4] u16 | arity u8 | pad u8` (16 bytes).
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + self.instructions.len() * 16);
        buf.extend_from_slice(MLIR_SSA_WIRE_MAGIC);
        buf.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        for inst in &self.instructions {
            buf.extend_from_slice(&inst.op_id.to_le_bytes());
            buf.extend_from_slice(&inst.dst_reg.to_le_bytes());
            for r in inst.src_regs {
                buf.extend_from_slice(&r.to_le_bytes());
            }
            buf.push(inst.arity);
            buf.push(0);
        }
        buf
    }

    pub fn validate_ssa_and_types(
        &self,
        catalog: &MlirAtomCatalog,
        sender: RoleId,
    ) -> Result<MlirSsaValidationReceipt, MlirSsaError> {
        if self.instructions.is_empty() {
            return Err(MlirSsaError::EmptyBlock);
        }
        if !catalog.verify_integrity() {
            return Err(MlirSsaError::CatalogIntegrityFailure);
        }

        let mut reg_types: BTreeMap<u16, MlirTypeSig> = BTreeMap::new();
        let mut last_type = MlirTypeSig::Void;
        let mut return_step: Option<usize> = None;

        for (step, inst) in self.instructions.iter().enumerate() {
            let desc = catalog
                .get_op(inst.op_id)
                .ok_or(MlirSsaError::UnknownOpcode {
                    step,
                    op_id: inst.op_id,
                })?;

            if desc.mnemonic == "func.return_q32" {
                if return_step.is_some() {
                    return Err(MlirSsaError::DuplicateReturn { step });
                }
                return_step = Some(step);
            }

            if let Some(req_role) = desc.required_role {
                if sender != RoleId::Coordinator && sender != req_role {
                    return Err(MlirSsaError::RoleCapabilityViolation {
                        step,
                        op_id: inst.op_id,
                        required_role: req_role,
                        actual_sender: sender,
                    });
                }
            }

            let expected_arity = desc.arg_types.len();
            if (inst.arity as usize) != expected_arity {
                return Err(MlirSsaError::ArityMismatch {
                    step,
                    op_id: inst.op_id,
                    expected: expected_arity,
                    found: inst.arity as usize,
                });
            }

            for arg_idx in 0..expected_arity {
                let src_reg = inst.src_regs[arg_idx];
                let actual_ty = reg_types
                    .get(&src_reg)
                    .copied()
                    .ok_or(MlirSsaError::UseBeforeDef { step, reg: src_reg })?;
                let expected_ty = desc.arg_types[arg_idx];
                if actual_ty != expected_ty {
                    return Err(MlirSsaError::TypeMismatch {
                        step,
                        op_id: inst.op_id,
                        arg_index: arg_idx,
                        expected: expected_ty,
                        found: actual_ty,
                    });
                }
            }

            if reg_types.contains_key(&inst.dst_reg) {
                return Err(MlirSsaError::DuplicateSsaDefinition {
                    step,
                    reg: inst.dst_reg,
                });
            }
            reg_types.insert(inst.dst_reg, desc.ret_type);
            last_type = desc.ret_type;
        }

        let return_step = return_step.ok_or(MlirSsaError::MissingReturn)?;
        if return_step != self.instructions.len() - 1 {
            return Err(MlirSsaError::ReturnNotLast { step: return_step });
        }

        let wire = self.to_wire_bytes();
        Ok(MlirSsaValidationReceipt {
            instruction_count: self.instructions.len(),
            final_return_type: last_type,
            block_digest: Sha256Digest::compute(&wire),
        })
    }
}
