//! Boundary Symplectic Transducer — versioned typed ingress/egress contract.
//!
//! Frozen contract `bst-v1` (see `BOUNDARY-SYMPLECTIC-TRANSDUCER-SPEC.md`).
//! Native typed/binary representations only: no Python, no JSON on this path.
//! Canonical bytes below are hand-rolled little-endian encodings, NOT JSON.
//!
//! Honesty rule enforced by this module: a hyperbolic coordinate plus a phase
//! is a *phase-augmented* representation. The `SymplecticCanonical` claim is
//! only attached after [`check_canonical_transport`] proves preservation of
//! the frozen two-form within tolerance. Nothing here certifies a symplectic
//! map by construction.

use sha2::{Digest, Sha256};
use std::fmt;

// ---------------------------------------------------------------------------
// Frozen versions (change requires a new address revision + replay receipt)
// ---------------------------------------------------------------------------

/// Transducer implementation version.
pub const TRANSDUCER_VERSION: &str = "bst-v1";
/// Ingress schema version.
pub const INGRESS_SCHEMA_VERSION: u32 = 1;
/// Egress schema version.
pub const EGRESS_SCHEMA_VERSION: u32 = 1;
/// Semantic canonicalizer version.
pub const CANONICALIZER_VERSION: u32 = 1;
/// MLIR dialect version (`gm` dialect, ops below).
pub const DIALECT_VERSION: u32 = 1;
/// Geometry / projector version (H3xT2 chart + cotangent derivation).
pub const GEOMETRY_VERSION: u32 = 1;
/// Q32.31 numeric contract version.
pub const NUMERIC_CONTRACT_VERSION: u32 = 1;
/// Adapter contract version.
pub const ADAPTER_CONTRACT_VERSION: u32 = 1;
/// Address revision: bumped whenever ANY version above changes.
pub const ADDRESS_REVISION: u32 = 1;

// Bounds (ingress budget).
/// Maximum source bytes accepted per module.
pub const MAX_SOURCE_BYTES: usize = 1 << 20;
/// Maximum ops per module.
pub const MAX_OPS_PER_MODULE: usize = 4096;
/// Maximum tool args per tool-call op.
pub const MAX_TOOL_ARGS: usize = 64;
/// Maximum bytes per single tool arg.
pub const MAX_ARG_BYTES: usize = 65536;
/// Maximum provenance field length.
pub const MAX_PROVENANCE_LEN: usize = 1024;

// Geometry tolerances.
const TAU: f64 = std::f64::consts::TAU;
/// Hyperboloid domain residual tolerance: |<x,x>_L + 1| <= TOL_DOMAIN.
const TOL_DOMAIN: f64 = 1e-9;
/// Tangent-space residual tolerance: |<x,p>_L| <= TOL_TANGENT * (1 + |p|).
const TOL_TANGENT: f64 = 1e-9;
/// Default symplectic residual tolerance for canonical-transport claims.
pub const DEFAULT_SYMPLECTIC_TOL: f64 = 1e-9;
/// Maximum admissible tolerance a `SymplecticCanonical` claim may carry.
pub const MAX_SYMPLECTIC_TOL: f64 = 1e-6;

// ---------------------------------------------------------------------------
// Frozen Q32.31 numeric contract (independent copy; do not import live code)
// ---------------------------------------------------------------------------

/// Q32.31 policy scalar: represented value is `raw / 2^31`.
/// Decisions compare raw values exactly; this path performs no float math
/// except at the checked conversion boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Q32_31(i64);

impl Q32_31 {
    pub const SCALE: i64 = 1_i64 << 31;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_i64 << 31);
    /// Numeric contract version this type implements.
    pub const CONTRACT_VERSION: u32 = NUMERIC_CONTRACT_VERSION;

    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }
    pub const fn raw(self) -> i64 {
        self.0
    }
    /// Checked float conversion: rejects NaN/inf and out-of-range values.
    pub fn from_f64(v: f64) -> Result<Self, String> {
        if !v.is_finite() {
            return Err("Q32.31: non-finite value".to_string());
        }
        let scaled = v * (Self::SCALE as f64);
        if scaled > i64::MAX as f64 || scaled < i64::MIN as f64 {
            return Err("Q32.31: out of range".to_string());
        }
        Ok(Self(scaled.round() as i64))
    }
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }
}

// ---------------------------------------------------------------------------
// Dialect: effects, types, op names
// ---------------------------------------------------------------------------

/// Effect bits. Pure = 0.
pub const EFF_EMIT: u8 = 0x01;
pub const EFF_OBSERVE: u8 = 0x02;
pub const EFF_CALL: u8 = 0x04;
const EFF_ALL: u8 = EFF_EMIT | EFF_OBSERVE | EFF_CALL;

/// Typed result of an op.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MlirType {
    /// Strict UTF-8 text carrier.
    Utf8,
    /// Opaque bytes (multimodal observation payload).
    Bytes,
    /// Tool-call handle (typed args live in the payload).
    Handle,
    /// Unit (control only).
    Unit,
}

/// Tag universe for canonical bytes.
impl MlirType {
    fn tag(self) -> u8 {
        match self {
            MlirType::Utf8 => 1,
            MlirType::Bytes => 2,
            MlirType::Handle => 3,
            MlirType::Unit => 4,
        }
    }
}
/// Checked inverse of the canonical type tag: parses foreign bytes.
/// `pub` because the inverse direction is part of the frozen decoder
/// contract; pinned by `tags_roundtrip` so neither direction can drift.
pub fn mlir_type_from_tag(t: u8) -> Option<MlirType> {
    match t {
        1 => Some(MlirType::Utf8),
        2 => Some(MlirType::Bytes),
        3 => Some(MlirType::Handle),
        4 => Some(MlirType::Unit),
        _ => None,
    }
}

/// Dialect op names (`gm` dialect v1).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum OpName {
    TextEmit,
    CodeEmit,
    Observe,
    ToolCall,
}

impl OpName {
    fn tag(self) -> u8 {
        match self {
            OpName::TextEmit => 1,
            OpName::CodeEmit => 2,
            OpName::Observe => 3,
            OpName::ToolCall => 4,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            OpName::TextEmit => "gm.text_emit",
            OpName::CodeEmit => "gm.code_emit",
            OpName::Observe => "gm.observe",
            OpName::ToolCall => "gm.tool_call",
        }
    }
}
/// Checked inverse of the canonical op tag: parses foreign bytes.
/// `pub` because the inverse direction is part of the frozen decoder
/// contract; pinned by `tags_roundtrip` so neither direction can drift.
pub fn op_name_from_tag(t: u8) -> Option<OpName> {
    match t {
        1 => Some(OpName::TextEmit),
        2 => Some(OpName::CodeEmit),
        3 => Some(OpName::Observe),
        4 => Some(OpName::ToolCall),
        _ => None,
    }
}

/// (result type, max operands, allowed effect mask, required effect mask)
fn op_signature(name: OpName) -> (MlirType, usize, u8, u8) {
    match name {
        OpName::TextEmit | OpName::CodeEmit => (MlirType::Utf8, 1, EFF_EMIT, 0),
        OpName::Observe => (MlirType::Bytes, 1, EFF_OBSERVE, EFF_OBSERVE),
        OpName::ToolCall => (MlirType::Handle, 1, EFF_CALL, EFF_CALL),
    }
}

// ---------------------------------------------------------------------------
// Source, provenance, spans, geometry
// ---------------------------------------------------------------------------

/// Source format preserved verbatim from ingress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceFormat {
    TextUtf8,
    Code { language: String },
    OctetStream { mime: String },
    ToolEvent,
}

impl SourceFormat {
    fn tag(&self) -> u8 {
        match self {
            SourceFormat::TextUtf8 => 1,
            SourceFormat::Code { .. } => 2,
            SourceFormat::OctetStream { .. } => 3,
            SourceFormat::ToolEvent => 4,
        }
    }
}

/// Provenance: both fields required, length-bounded. Never empty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    pub source: String,
    pub actor: String,
    pub detail: String,
}

impl Provenance {
    pub fn new(source: &str, actor: &str, detail: &str) -> Result<Self, String> {
        if source.is_empty() {
            return Err("provenance: source must be non-empty".to_string());
        }
        if actor.is_empty() {
            return Err("provenance: actor must be non-empty".to_string());
        }
        if source.len() > MAX_PROVENANCE_LEN
            || actor.len() > MAX_PROVENANCE_LEN
            || detail.len() > MAX_PROVENANCE_LEN
        {
            return Err("provenance: field exceeds bound".to_string());
        }
        Ok(Self {
            source: source.to_string(),
            actor: actor.to_string(),
            detail: detail.to_string(),
        })
    }
}

/// Source bytes preserved verbatim. Language/script tags are opaque and
/// NEVER gate acceptance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBlob {
    pub bytes: Vec<u8>,
    pub format: SourceFormat,
    pub provenance: Provenance,
    /// Opaque BCP-47 (or other) tag; preserved, never used to exclude.
    pub language_tag: Option<String>,
}

/// Byte span into the source blob: [start, end).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

/// Position half: point on H3 (hyperboloid in R^{3,1}) x T2 (two-torus).
/// Invariant: x0 > 0 and <x,x>_L = -1; phases in [0, TAU).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub h: [f64; 4],
    pub t: [f64; 2],
}

/// Momentum half: separately typed cotangent component.
/// Invariant: finite, and `lin` tangent to H3 at `h` (<h,lin>_L = 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cotangent {
    pub lin: [f64; 4],
    pub tor: [f64; 2],
}

/// Geometry claim attached to a module. Default is [`GeometryClaim::PhaseAugmented`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeometryClaim {
    /// Hyperbolic coordinate + phase WITHOUT a proven preserved two-form.
    /// Honest label; carries no symplectic claim.
    PhaseAugmented,
    /// Canonical transport proven by [`check_canonical_transport`].
    SymplecticCanonical { residual: f64, tol: f64 },
}

impl GeometryClaim {
    fn tag(self) -> u8 {
        match self {
            GeometryClaim::PhaseAugmented => 0,
            GeometryClaim::SymplecticCanonical { .. } => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Typed payloads
// ---------------------------------------------------------------------------

/// Typed tool argument value.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    Int(i64),
    RealBits(u64),
    Bool(bool),
    Text(String),
    Bytes(Vec<u8>),
}

impl ArgValue {
    /// Strict UTF-8 decoding applies to Text; Bytes may be arbitrary.
    fn validate(&self) -> Result<(), String> {
        match self {
            ArgValue::Text(s) => {
                if s.len() > MAX_ARG_BYTES {
                    return Err("tool arg text exceeds bound".to_string());
                }
                Ok(())
            }
            ArgValue::Bytes(b) => {
                if b.len() > MAX_ARG_BYTES {
                    return Err("tool arg bytes exceed bound".to_string());
                }
                Ok(())
            }
            ArgValue::RealBits(bits) => {
                let v = f64::from_bits(*bits);
                if !v.is_finite() {
                    return Err("tool arg real is non-finite".to_string());
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Typed tool argument.
#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    pub name: String,
    pub value: ArgValue,
}

/// Payload carried by an op (types, not carrier IDs).
#[derive(Clone, Debug, PartialEq)]
pub enum OpPayload {
    Text,
    Code { language: String },
    Observe { mime: String },
    Tool { tool: String, args: Vec<Arg> },
}

impl OpPayload {
    fn tag(&self) -> u8 {
        match self {
            OpPayload::Text => 1,
            OpPayload::Code { .. } => 2,
            OpPayload::Observe { .. } => 3,
            OpPayload::Tool { .. } => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// SSA op + module
// ---------------------------------------------------------------------------

/// One typed SSA operation in the single-block `gm` module.
#[derive(Clone, Debug, PartialEq)]
pub struct Op {
    /// Dense SSA id: ops are stored in id order 0..n.
    pub id: u32,
    pub name: OpName,
    pub result_type: MlirType,
    /// Dataflow/threading operands: must reference strictly smaller ids.
    pub operands: Vec<u32>,
    pub effects: u8,
    pub span: Span,
    pub pos: Position,
    pub mom: Cotangent,
    pub payload: OpPayload,
}

/// Frozen version set bound into every digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct VersionSet {
    pub ingress_schema: u32,
    pub egress_schema: u32,
    pub canonicalizer: u32,
    pub dialect: u32,
    pub geometry: u32,
    pub numeric: u32,
    pub adapter: u32,
}

impl VersionSet {
    pub const CURRENT: Self = Self {
        ingress_schema: INGRESS_SCHEMA_VERSION,
        egress_schema: EGRESS_SCHEMA_VERSION,
        canonicalizer: CANONICALIZER_VERSION,
        dialect: DIALECT_VERSION,
        geometry: GEOMETRY_VERSION,
        numeric: NUMERIC_CONTRACT_VERSION,
        adapter: ADAPTER_CONTRACT_VERSION,
    };
}

/// Versioned symplectic MLIR module (single block, straight-line).
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub versions: VersionSet,
    pub address_rev: u32,
    pub domain: String,
    pub scope: String,
    pub source: SourceBlob,
    pub ops: Vec<Op>,
    pub claim: GeometryClaim,
    /// Immutable digest over transducer version + schema + input + module.
    pub digest: [u8; 32],
}

// ---------------------------------------------------------------------------
// Receipts and failures (never silent)
// ---------------------------------------------------------------------------

/// Ingress rejection receipt: ambiguous or unrepresentable input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngressRejection {
    pub reason: String,
    pub span: Option<Span>,
    /// Digest of the rejected input bytes (receipted, not dropped).
    pub input_digest: [u8; 32],
}

impl fmt::Display for IngressRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ingress rejected: {}", self.reason)
    }
}

impl std::error::Error for IngressRejection {}

/// Validation receipt for a module that passed every check.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReceipt {
    pub digest: [u8; 32],
    pub ops: u32,
    pub bytes: u64,
    pub claim: GeometryClaim,
}

/// Model-native bytes: explicitly model-bound, NEVER part of the semantic key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelNative {
    pub bytes: Vec<u8>,
    pub shape: Vec<usize>,
    pub adapter_digest: [u8; 32],
    pub weights_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub execution_contract: String,
}

/// Adapter-level failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterError {
    pub reason: String,
}

/// One decoded model result per SSA op.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeResult {
    pub ssa_id: u32,
    pub payload: ResultPayload,
    pub effect_bits: u8,
}

/// Decoded payload typed per op.
#[derive(Clone, Debug, PartialEq)]
pub enum ResultPayload {
    TextBytes(Vec<u8>),
    RawBytes(Vec<u8>),
    ToolArgs(Vec<Arg>),
    Unit,
}

/// Egress failure: explicit, with per-category receipts. No plausible string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EgressFailure {
    pub unknown_ids: Vec<u32>,
    pub missing_ids: Vec<u32>,
    pub failed_inverse: Vec<u32>,
    pub effect_mismatch: Vec<u32>,
    pub loss_bytes: u64,
    pub reason: String,
}

impl fmt::Display for EgressFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "egress failed: {}", self.reason)
    }
}

impl std::error::Error for EgressFailure {}

/// Per-op evidence emitted on successful egress.
#[derive(Clone, Debug, PartialEq)]
pub struct OpEvidence {
    pub ssa_id: u32,
    pub name: String,
    pub payload_sha256: [u8; 32],
}

/// Loss accounting: all zeros on success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LossReceipt {
    pub unknown_ids: u64,
    pub failed_inverse: u64,
    pub loss_bytes: u64,
}

/// Successful semantic output.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticOutput {
    /// Rendered text for text/code modules; None for observation/tool modules.
    pub text: Option<String>,
    pub evidence: Vec<OpEvidence>,
    pub loss: LossReceipt,
}

/// Replay receipt: re-validation of a module at its frozen address revision.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayReceipt {
    pub module_digest: [u8; 32],
    pub address_rev: u32,
    pub versions: VersionSet,
    pub ops: u32,
    pub loss_bytes: u64,
}

// ---------------------------------------------------------------------------
// Hashing helpers
// ---------------------------------------------------------------------------

fn sha256_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    let o = h.finalize();
    let mut a = [0u8; 32];
    a.copy_from_slice(&o);
    a
}

fn u64_at(d: &[u8; 32], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[off..off + 8]);
    u64::from_le_bytes(b)
}

// ---------------------------------------------------------------------------
// Canonical (deterministic, non-JSON) byte encodings
// ---------------------------------------------------------------------------

fn push_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}
fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn push_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn push_i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn push_f64(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_bits().to_le_bytes());
}
fn push_bytes(out: &mut Vec<u8>, b: &[u8]) {
    push_u64(out, b.len() as u64);
    out.extend_from_slice(b);
}
fn push_str(out: &mut Vec<u8>, s: &str) {
    push_bytes(out, s.as_bytes());
}

fn canonical_arg_value(out: &mut Vec<u8>, v: &ArgValue) {
    match v {
        ArgValue::Int(i) => {
            push_u8(out, 1);
            push_i64(out, *i);
        }
        ArgValue::RealBits(b) => {
            push_u8(out, 2);
            push_u64(out, *b);
        }
        ArgValue::Bool(b) => {
            push_u8(out, 3);
            push_u8(out, u8::from(*b));
        }
        ArgValue::Text(s) => {
            push_u8(out, 4);
            push_str(out, s);
        }
        ArgValue::Bytes(b) => {
            push_u8(out, 5);
            push_bytes(out, b);
        }
    }
}

fn canonical_args(out: &mut Vec<u8>, args: &[Arg]) {
    push_u64(out, args.len() as u64);
    for a in args {
        push_str(out, &a.name);
        canonical_arg_value(out, &a.value);
    }
}

fn canonical_payload(out: &mut Vec<u8>, p: &OpPayload) {
    push_u8(out, p.tag());
    match p {
        OpPayload::Text => {}
        OpPayload::Code { language } => push_str(out, language),
        OpPayload::Observe { mime } => push_str(out, mime),
        OpPayload::Tool { tool, args } => {
            push_str(out, tool);
            canonical_args(out, args);
        }
    }
}

fn canonical_source(out: &mut Vec<u8>, s: &SourceBlob) {
    push_bytes(out, &s.bytes);
    push_u8(out, s.format.tag());
    match &s.format {
        SourceFormat::Code { language } => push_str(out, language),
        SourceFormat::OctetStream { mime } => push_str(out, mime),
        _ => {}
    }
    push_str(out, &s.provenance.source);
    push_str(out, &s.provenance.actor);
    push_str(out, &s.provenance.detail);
    match &s.language_tag {
        Some(t) => {
            push_u8(out, 1);
            push_str(out, t);
        }
        None => push_u8(out, 0),
    }
}

fn canonical_op(out: &mut Vec<u8>, o: &Op) {
    push_u32(out, o.id);
    push_u8(out, o.name.tag());
    push_u8(out, o.result_type.tag());
    push_u64(out, o.operands.len() as u64);
    for op in &o.operands {
        push_u32(out, *op);
    }
    push_u8(out, o.effects);
    push_u32(out, o.span.start);
    push_u32(out, o.span.end);
    for v in o.pos.h {
        push_f64(out, v);
    }
    for v in o.pos.t {
        push_f64(out, v);
    }
    for v in o.mom.lin {
        push_f64(out, v);
    }
    for v in o.mom.tor {
        push_f64(out, v);
    }
    canonical_payload(out, &o.payload);
}

fn canonical_claim(out: &mut Vec<u8>, c: GeometryClaim) {
    push_u8(out, c.tag());
    if let GeometryClaim::SymplecticCanonical { residual, tol } = c {
        push_f64(out, residual);
        push_f64(out, tol);
    }
}

fn canonical_versions(out: &mut Vec<u8>, v: VersionSet) {
    push_u32(out, v.ingress_schema);
    push_u32(out, v.egress_schema);
    push_u32(out, v.canonicalizer);
    push_u32(out, v.dialect);
    push_u32(out, v.geometry);
    push_u32(out, v.numeric);
    push_u32(out, v.adapter);
}

/// Canonical bytes of a module WITHOUT its digest (digest input).
fn canonical_module_nodigest(m: &Module) -> Vec<u8> {
    let mut out = Vec::new();
    push_str(&mut out, TRANSDUCER_VERSION);
    canonical_versions(&mut out, m.versions);
    push_u32(&mut out, m.address_rev);
    push_str(&mut out, &m.domain);
    push_str(&mut out, &m.scope);
    canonical_source(&mut out, &m.source);
    push_u64(&mut out, m.ops.len() as u64);
    for o in &m.ops {
        canonical_op(&mut out, o);
    }
    canonical_claim(&mut out, m.claim);
    out
}

/// Immutable digest over transducer version + schema + input + lowered module.
pub fn module_digest(m: &Module) -> [u8; 32] {
    let body = canonical_module_nodigest(m);
    sha256_parts(&[b"BST-MOD-v1", &body])
}

/// Digest of raw input bytes (for rejection receipts).
pub fn input_digest(bytes: &[u8]) -> [u8; 32] {
    sha256_parts(&[b"BST-IN-v1", TRANSDUCER_VERSION.as_bytes(), bytes])
}

// ---------------------------------------------------------------------------
// Geometry: Lorentz model, derivation, two-form, canonical-transport test
// ---------------------------------------------------------------------------

/// Lorentz inner product on R^{3,1}.
pub fn lorentz(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    -a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Derive a deterministic (H3 x T2) address plus cotangent component from
/// span bytes. Finite by construction; satisfies the H3 and tangent invariants
/// up to floating-point rounding (checked by validation within tolerance).
pub fn derive_address(span_bytes: &[u8], op_id: u32, name: OpName) -> (Position, Cotangent) {
    let mut id_le = [0u8; 4];
    id_le.copy_from_slice(&op_id.to_le_bytes());
    let d = sha256_parts(&[b"BST-ADDR-v1", span_bytes, &id_le, &[name.tag()]]);
    let sym = |w: u64| (w as f64) / (u64::MAX as f64) * 2.0 - 1.0;
    const K: f64 = 1.5;
    let x1 = sym(u64_at(&d, 0)) * K;
    let x2 = sym(u64_at(&d, 8)) * K;
    let x3 = sym(u64_at(&d, 16)) * K;
    let x0 = (1.0 + x1 * x1 + x2 * x2 + x3 * x3).sqrt();
    let h = [x0, x1, x2, x3];
    // Raw momentum, then projected to T_h H3: p = q + <h,q>_L * h.
    let q = [
        sym(u64_at(&d, 24).wrapping_mul(0x9E3779B97F4A7C15)) * 0.1,
        sym(u64_at(&d, 0).wrapping_mul(0xBF58476D1CE4E5B9)) * 0.1,
        sym(u64_at(&d, 8).wrapping_mul(0x94D049BB133111EB)) * 0.1,
        sym(u64_at(&d, 16).wrapping_mul(0x2545F4914F6CDD1D)) * 0.1,
    ];
    let m = lorentz(&h, &q);
    let lin = [
        q[0] + m * h[0],
        q[1] + m * h[1],
        q[2] + m * h[2],
        q[3] + m * h[3],
    ];
    let frac = |w: u64| (w as f64) / (u64::MAX as f64);
    let mut t0 = frac(u64_at(&d, 8) ^ u64_at(&d, 24).rotate_left(17)) * TAU;
    let mut t1 = frac(u64_at(&d, 0) ^ u64_at(&d, 16).rotate_left(31)) * TAU;
    if t0 >= TAU {
        t0 = 0.0;
    }
    if t1 >= TAU {
        t1 = 0.0;
    }
    let tor = [
        sym(u64_at(&d, 24) ^ 0xAAAAAAAAAAAAAAAA) * 0.1,
        sym(u64_at(&d, 16) ^ 0x5555555555555555) * 0.1,
    ];
    (Position { h, t: [t0, t1] }, Cotangent { lin, tor })
}

/// Frozen canonical two-form on the 10-D phase chart
/// (5 position + 5 momentum): OMEGA = [[0, I5], [-I5, 0]].
/// Returns the full 10x10 matrix.
pub fn symplectic_form() -> [[f64; 10]; 10] {
    let mut o = [[0.0f64; 10]; 10];
    for i in 0..5 {
        o[i][5 + i] = 1.0;
        o[5 + i][i] = -1.0;
    }
    o
}

/// Residual of a claimed canonical transport map with Jacobian J:
/// || J^T OMEGA J - OMEGA ||_F. Zero (within rounding) iff canonical.
pub fn canonical_residual(j: &[[f64; 10]; 10]) -> f64 {
    let o = symplectic_form();
    // jt_o = J^T * OMEGA
    let mut jt_o = [[0.0f64; 10]; 10];
    for (i, row) in jt_o.iter_mut().enumerate() {
        for (k, cell) in row.iter_mut().enumerate() {
            *cell = j
                .iter()
                .zip(o.iter())
                .map(|(jr, orow)| jr[i] * orow[k])
                .sum();
        }
    }
    // m = jt_o * J, residual vs OMEGA
    let mut acc = 0.0;
    for (i, (jt_row, o_row)) in jt_o.iter().zip(o.iter()).enumerate() {
        for (k, o_cell) in o_row.iter().enumerate() {
            let s: f64 = jt_row
                .iter()
                .zip(j.iter())
                .map(|(a, jrow)| a * jrow[k])
                .sum();
            let d = s - o_cell;
            let _ = i;
            acc += d * d;
        }
    }
    acc.sqrt()
}

/// Prove (or reject) a canonical-transport claim for Jacobian J.
/// Ok(residual) iff residual <= tol; Err names the residual otherwise.
/// Nondegeneracy of OMEGA itself is pinned by unit test
/// (`omega_is_nondegenrate`: skew + square = -I, hence det = +/-1 != 0).
pub fn check_canonical_transport(j: &[[f64; 10]; 10], tol: f64) -> Result<f64, String> {
    if !(tol.is_finite()) || tol < 0.0 || tol > MAX_SYMPLECTIC_TOL {
        return Err(format!(
            "symplectic tol {tol} out of admissible range [0, {MAX_SYMPLECTIC_TOL}]"
        ));
    }
    for row in j.iter() {
        for v in row.iter() {
            if !v.is_finite() {
                return Err("transport Jacobian is non-finite".to_string());
            }
        }
    }
    let r = canonical_residual(j);
    if !r.is_finite() {
        return Err("symplectic residual is non-finite".to_string());
    }
    if r <= tol {
        Ok(r)
    } else {
        Err(format!(
            "not canonical: symplectic residual {r} exceeds tol {tol}; label as phase-augmented"
        ))
    }
}

// ---------------------------------------------------------------------------
// Ingress
// ---------------------------------------------------------------------------

fn check_bounds(bytes_len: usize, n_ops_hint: usize) -> Result<(), String> {
    if bytes_len > MAX_SOURCE_BYTES {
        return Err(format!(
            "source {bytes_len} bytes exceeds bound {MAX_SOURCE_BYTES}"
        ));
    }
    if n_ops_hint > MAX_OPS_PER_MODULE {
        return Err(format!(
            "op count {n_ops_hint} exceeds bound {MAX_OPS_PER_MODULE}"
        ));
    }
    Ok(())
}

fn check_domain_scope(domain: &str, scope: &str) -> Result<(), String> {
    if domain.is_empty() || domain.len() > MAX_PROVENANCE_LEN {
        return Err("domain must be non-empty and bounded".to_string());
    }
    if scope.is_empty() || scope.len() > MAX_PROVENANCE_LEN {
        return Err("scope must be non-empty and bounded".to_string());
    }
    Ok(())
}

/// Split bytes after each `\n`, keeping delimiters (exact coverage).
fn split_lines_keep(bytes: &[u8]) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let mut s = 0u32;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            spans.push((s, (i + 1) as u32));
            s = (i + 1) as u32;
        }
    }
    if (s as usize) < bytes.len() {
        spans.push((s, bytes.len() as u32));
    }
    spans
}

fn build_module(
    source: SourceBlob,
    pieces: Vec<(OpName, Span, OpPayload, u8)>,
    domain: &str,
    scope: &str,
) -> Result<Module, IngressRejection> {
    let rej = |reason: String, span: Option<Span>| IngressRejection {
        reason,
        span,
        input_digest: input_digest(&source.bytes),
    };
    if source.bytes.len() > MAX_SOURCE_BYTES {
        return Err(rej(
            format!(
                "source {} bytes exceeds bound {MAX_SOURCE_BYTES}",
                source.bytes.len()
            ),
            None,
        ));
    }
    if pieces.len() > MAX_OPS_PER_MODULE {
        return Err(rej(
            format!(
                "op count {} exceeds bound {MAX_OPS_PER_MODULE}",
                pieces.len()
            ),
            None,
        ));
    }
    if let Err(e) = check_domain_scope(domain, scope) {
        return Err(rej(e, None));
    }
    let mut ops = Vec::with_capacity(pieces.len());
    for (i, (name, span, payload, effects)) in pieces.into_iter().enumerate() {
        let id = i as u32;
        let (ty, _, _, _) = op_signature(name);
        let end = span.end as usize;
        let start = span.start as usize;
        if end > source.bytes.len() || start > end {
            return Err(rej(
                format!("span [{start},{end}) out of bounds"),
                Some(span),
            ));
        }
        let (pos, mom) = derive_address(&source.bytes[start..end], id, name);
        let operands = if id == 0 { vec![] } else { vec![id - 1] };
        ops.push(Op {
            id,
            name,
            result_type: ty,
            operands,
            effects,
            span,
            pos,
            mom,
            payload,
        });
    }
    let mut m = Module {
        versions: VersionSet::CURRENT,
        address_rev: ADDRESS_REVISION,
        domain: domain.to_string(),
        scope: scope.to_string(),
        source,
        ops,
        // Default honest label: phase-augmented. Canonical claims require
        // check_canonical_transport + attach_canonical_claim.
        claim: GeometryClaim::PhaseAugmented,
        digest: [0u8; 32],
    };
    m.digest = module_digest(&m);
    Ok(m)
}

/// Ingress: bounded semantic text. Strict UTF-8; language tag opaque.
pub fn ingress_text(
    bytes: &[u8],
    language_tag: Option<&str>,
    provenance: Provenance,
    domain: &str,
    scope: &str,
) -> Result<Module, IngressRejection> {
    let rej = |reason: String| IngressRejection {
        reason,
        span: None,
        input_digest: input_digest(bytes),
    };
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(rej(format!(
            "source {} bytes exceeds bound {MAX_SOURCE_BYTES}",
            bytes.len()
        )));
    }
    if std::str::from_utf8(bytes).is_err() {
        return Err(rej("text ingress: invalid UTF-8 (strict)".to_string()));
    }
    if language_tag.is_some_and(|t| t.len() > MAX_PROVENANCE_LEN) {
        return Err(rej("language tag exceeds bound".to_string()));
    }
    let source = SourceBlob {
        bytes: bytes.to_vec(),
        format: SourceFormat::TextUtf8,
        provenance,
        language_tag: language_tag.map(str::to_string),
    };
    let spans = split_lines_keep(bytes);
    if spans.len() > MAX_OPS_PER_MODULE {
        return Err(rej(format!(
            "line-op count {} exceeds bound {MAX_OPS_PER_MODULE}",
            spans.len()
        )));
    }
    let pieces = spans
        .into_iter()
        .map(|(s, e)| {
            (
                OpName::TextEmit,
                Span { start: s, end: e },
                OpPayload::Text,
                EFF_EMIT,
            )
        })
        .collect();
    build_module(source, pieces, domain, scope)
}

/// Ingress: code text with a declared language. Bytes preserved verbatim.
pub fn ingress_code(
    bytes: &[u8],
    language: &str,
    provenance: Provenance,
    domain: &str,
    scope: &str,
) -> Result<Module, IngressRejection> {
    let rej = |reason: String| IngressRejection {
        reason,
        span: None,
        input_digest: input_digest(bytes),
    };
    if language.is_empty() || language.len() > MAX_PROVENANCE_LEN {
        return Err(rej(
            "code ingress: language must be non-empty and bounded".to_string()
        ));
    }
    if std::str::from_utf8(bytes).is_err() {
        return Err(rej("code ingress: invalid UTF-8 (strict)".to_string()));
    }
    let source = SourceBlob {
        bytes: bytes.to_vec(),
        format: SourceFormat::Code {
            language: language.to_string(),
        },
        provenance,
        language_tag: None,
    };
    let spans = split_lines_keep(bytes);
    if spans.len() > MAX_OPS_PER_MODULE {
        return Err(rej(format!(
            "line-op count {} exceeds bound {MAX_OPS_PER_MODULE}",
            spans.len()
        )));
    }
    let pieces = spans
        .into_iter()
        .map(|(s, e)| {
            (
                OpName::CodeEmit,
                Span { start: s, end: e },
                OpPayload::Code {
                    language: language.to_string(),
                },
                EFF_EMIT,
            )
        })
        .collect();
    build_module(source, pieces, domain, scope)
}

/// Ingress: multimodal observation. Bytes may be arbitrary (mime-typed).
pub fn ingress_observation(
    bytes: &[u8],
    mime: &str,
    provenance: Provenance,
    domain: &str,
    scope: &str,
) -> Result<Module, IngressRejection> {
    let rej = |reason: String| IngressRejection {
        reason,
        span: None,
        input_digest: input_digest(bytes),
    };
    if mime.is_empty() || mime.len() > MAX_PROVENANCE_LEN {
        return Err(rej(
            "observation ingress: mime must be non-empty and bounded".to_string(),
        ));
    }
    let source = SourceBlob {
        bytes: bytes.to_vec(),
        format: SourceFormat::OctetStream {
            mime: mime.to_string(),
        },
        provenance,
        language_tag: None,
    };
    let pieces = vec![(
        OpName::Observe,
        Span {
            start: 0,
            end: bytes.len() as u32,
        },
        OpPayload::Observe {
            mime: mime.to_string(),
        },
        EFF_OBSERVE,
    )];
    build_module(source, pieces, domain, scope)
}

/// Ingress: typed tool event. No JSON: args are typed values.
pub fn ingress_tool_event(
    tool: &str,
    args: Vec<Arg>,
    provenance: Provenance,
    domain: &str,
    scope: &str,
) -> Result<Module, IngressRejection> {
    if tool.is_empty() || tool.len() > MAX_PROVENANCE_LEN {
        return Err(IngressRejection {
            reason: "tool ingress: tool name must be non-empty and bounded".to_string(),
            span: None,
            input_digest: input_digest(tool.as_bytes()),
        });
    }
    if args.len() > MAX_TOOL_ARGS {
        return Err(IngressRejection {
            reason: format!(
                "tool ingress: {} args exceeds bound {MAX_TOOL_ARGS}",
                args.len()
            ),
            span: None,
            input_digest: input_digest(tool.as_bytes()),
        });
    }
    let mut seen = std::collections::BTreeSet::new();
    for a in &args {
        if a.name.is_empty() || !seen.insert(a.name.as_str()) {
            return Err(IngressRejection {
                reason: "tool ingress: arg names must be non-empty and unique".to_string(),
                span: None,
                input_digest: input_digest(tool.as_bytes()),
            });
        }
        if let Err(e) = a.value.validate() {
            return Err(IngressRejection {
                reason: format!("tool ingress: arg '{}': {e}", a.name),
                span: None,
                input_digest: input_digest(tool.as_bytes()),
            });
        }
    }
    // Source bytes = deterministic canonical encoding of the typed event.
    let mut src = Vec::new();
    push_str(&mut src, tool);
    canonical_args(&mut src, &args);
    let end = src.len() as u32;
    let source = SourceBlob {
        bytes: src,
        format: SourceFormat::ToolEvent,
        provenance,
        language_tag: None,
    };
    let pieces = vec![(
        OpName::ToolCall,
        Span { start: 0, end },
        OpPayload::Tool {
            tool: tool.to_string(),
            args,
        },
        EFF_CALL,
    )];
    build_module(source, pieces, domain, scope)
}

/// Attach a proven canonical-transport claim to a module.
/// Recomputes the digest; the returned module differs from the input.
pub fn attach_canonical_claim(m: &Module, j: &[[f64; 10]; 10], tol: f64) -> Result<Module, String> {
    let residual = check_canonical_transport(j, tol)?;
    let mut out = m.clone();
    out.claim = GeometryClaim::SymplecticCanonical { residual, tol };
    out.digest = module_digest(&out);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn check_finite(v: f64) -> bool {
    v.is_finite()
}

/// Full module validation. Returns a receipt iff every check passes.
pub fn validate_module(m: &Module) -> Result<ValidationReceipt, String> {
    // Versions frozen: reject unknown schema/dialect/geometry/numeric.
    if m.versions != VersionSet::CURRENT {
        return Err(format!(
            "version mismatch: module {:?} != frozen {:?}; version change requires a new address revision",
            m.versions,
            VersionSet::CURRENT
        ));
    }
    if m.address_rev != ADDRESS_REVISION {
        return Err(format!(
            "address revision {} != frozen {ADDRESS_REVISION}",
            m.address_rev
        ));
    }
    check_domain_scope(&m.domain, &m.scope)?;
    check_bounds(m.source.bytes.len(), m.ops.len())?;
    if m.source.provenance.source.is_empty() || m.source.provenance.actor.is_empty() {
        return Err("provenance must be non-empty".to_string());
    }
    if m.ops.len() > MAX_OPS_PER_MODULE {
        return Err("op count exceeds bound".to_string());
    }
    // SSA: dense ids 0..n in order (straight-line dominance).
    for (i, o) in m.ops.iter().enumerate() {
        if o.id != i as u32 {
            return Err(format!(
                "ssa: op at position {i} carries id {}; ids must be dense 0..n in order",
                o.id
            ));
        }
        // Type/arity/effect signature.
        let (ty, max_ops, allowed, required) = op_signature(o.name);
        if o.result_type != ty {
            return Err(format!(
                "type: {} must return {:?}, got {:?}",
                o.name.as_str(),
                ty,
                o.result_type
            ));
        }
        if o.operands.len() > max_ops {
            return Err(format!(
                "arity: {} takes <= {max_ops} operands",
                o.name.as_str()
            ));
        }
        if o.effects & !EFF_ALL != 0 {
            return Err(format!(
                "effect: {} carries unknown effect bits",
                o.name.as_str()
            ));
        }
        if o.effects & !allowed != 0 {
            return Err(format!(
                "effect: {} allows {:#04x}, got {:#04x}",
                o.name.as_str(),
                allowed,
                o.effects
            ));
        }
        if o.effects & required != required {
            return Err(format!(
                "effect: {} requires {:#04x}, got {:#04x}",
                o.name.as_str(),
                required,
                o.effects
            ));
        }
        // Dominance: operands strictly dominate (defined earlier, exist).
        for op_id in &o.operands {
            if *op_id >= o.id {
                return Err(format!(
                    "dominance: op {} uses operand {op_id} which does not dominate it",
                    o.id
                ));
            }
        }
        // Span bounds. Empty spans are legal (empty observation payload);
        // inverted spans and overruns are not. Coverage below still
        // forbids gaps and overlaps, so no silent loss is possible.
        let s = o.span.start as usize;
        let e = o.span.end as usize;
        if s > e {
            return Err(format!("span: op {} has inverted span", o.id));
        }
        if e > m.source.bytes.len() {
            return Err(format!("span: op {} out of source bounds", o.id));
        }
        // Payload matches op name; typed arg values finite/bounded.
        match (&o.name, &o.payload) {
            (OpName::TextEmit, OpPayload::Text) => {
                let bytes = &m.source.bytes[s..e];
                if std::str::from_utf8(bytes).is_err() {
                    return Err(format!("span: op {} text is not strict UTF-8", o.id));
                }
            }
            (OpName::CodeEmit, OpPayload::Code { .. }) => {
                let bytes = &m.source.bytes[s..e];
                if std::str::from_utf8(bytes).is_err() {
                    return Err(format!("span: op {} code is not strict UTF-8", o.id));
                }
            }
            (OpName::Observe, OpPayload::Observe { mime }) => {
                if mime.is_empty() {
                    return Err(format!("payload: op {} mime empty", o.id));
                }
            }
            (OpName::ToolCall, OpPayload::Tool { tool, args }) => {
                if tool.is_empty() {
                    return Err(format!("payload: op {} tool empty", o.id));
                }
                if args.len() > MAX_TOOL_ARGS {
                    return Err(format!("payload: op {} too many args", o.id));
                }
                for a in args {
                    a.value
                        .validate()
                        .map_err(|e| format!("op {}: {e}", o.id))?;
                }
            }
            _ => {
                return Err(format!(
                    "payload: op {} name/payload mismatch ({})",
                    o.id,
                    o.name.as_str()
                ));
            }
        }
        // Coordinate domain + finite checks.
        for v in o.pos.h.iter().chain(o.pos.t.iter()) {
            if !check_finite(*v) {
                return Err(format!("coordinate: op {} non-finite position", o.id));
            }
        }
        for v in o.mom.lin.iter().chain(o.mom.tor.iter()) {
            if !check_finite(*v) {
                return Err(format!("coordinate: op {} non-finite momentum", o.id));
            }
        }
        if o.pos.h[0] <= 0.0 {
            return Err(format!("coordinate: op {} H3 x0 must be > 0", o.id));
        }
        let l = lorentz(&o.pos.h, &o.pos.h);
        if (l + 1.0).abs() > TOL_DOMAIN {
            return Err(format!(
                "coordinate: op {} H3 residual |<x,x>+1| = {} exceeds {TOL_DOMAIN}",
                o.id,
                (l + 1.0).abs()
            ));
        }
        for (k, p) in o.pos.t.iter().enumerate() {
            if *p < 0.0 || *p >= TAU {
                return Err(format!(
                    "coordinate: op {} T2 phase {k} = {p} outside [0, TAU)",
                    o.id
                ));
            }
        }
        let tl = lorentz(&o.pos.h, &o.mom.lin);
        let pnorm: f64 = o.mom.lin.iter().map(|v| v * v).sum::<f64>().sqrt();
        if tl.abs() > TOL_TANGENT * (1.0 + pnorm) {
            return Err(format!(
                "coordinate: op {} cotangent not tangent to H3 (|{}|)",
                o.id, tl
            ));
        }
    }
    // Coverage: spans partition [0, source.len) exactly — zero silent loss.
    let mut spans: Vec<(u32, u32)> = m.ops.iter().map(|o| (o.span.start, o.span.end)).collect();
    spans.sort();
    let mut cursor: u32 = 0;
    let len = m.source.bytes.len() as u32;
    for (s, e) in spans {
        if s != cursor {
            return Err(format!(
                "coverage: gap or overlap at byte {cursor} (next span starts at {s}); silent loss forbidden"
            ));
        }
        if e < s {
            return Err("coverage: inverted span".to_string());
        }
        cursor = e;
    }
    if cursor != len {
        return Err(format!(
            "coverage: spans cover {cursor} of {len} bytes; silent loss forbidden"
        ));
    }
    // Symplectic-form claim check.
    match m.claim {
        GeometryClaim::PhaseAugmented => {}
        GeometryClaim::SymplecticCanonical { residual, tol } => {
            if !residual.is_finite() || !tol.is_finite() {
                return Err("symplectic claim carries non-finite residual/tol".to_string());
            }
            if tol < 0.0 || tol > MAX_SYMPLECTIC_TOL {
                return Err("symplectic claim tol outside admissible range".to_string());
            }
            if residual > tol {
                return Err(format!(
                    "symplectic claim violated: residual {residual} > tol {tol}"
                ));
            }
        }
    }
    // Immutable digest check.
    let expect = module_digest(m);
    if expect != m.digest {
        return Err("digest mismatch: module bytes do not match sealed digest".to_string());
    }
    Ok(ValidationReceipt {
        digest: m.digest,
        ops: m.ops.len() as u32,
        bytes: m.source.bytes.len() as u64,
        claim: m.claim,
    })
}

// ---------------------------------------------------------------------------
// Semantic key (shared) vs model-bound bytes (never shared)
// ---------------------------------------------------------------------------

/// Canonical semantic payload: source bytes + op names/types/spans/payloads.
/// EXCLUDES geometry addresses, momenta, digests, adapter and model bytes.
fn canonical_semantic_payload(m: &Module) -> Vec<u8> {
    let mut out = Vec::new();
    push_bytes(&mut out, &m.source.bytes);
    push_u8(&mut out, m.source.format.tag());
    push_u64(&mut out, m.ops.len() as u64);
    for o in &m.ops {
        push_u32(&mut out, o.id);
        push_u8(&mut out, o.name.tag());
        push_u8(&mut out, o.result_type.tag());
        push_u64(&mut out, o.operands.len() as u64);
        for op in &o.operands {
            push_u32(&mut out, *op);
        }
        push_u32(&mut out, o.span.start);
        push_u32(&mut out, o.span.end);
        canonical_payload(&mut out, &o.payload);
    }
    out
}

/// Stable shared semantic key:
/// H("GMSEM-v1" || canonicalizer_version || domain || scope || payload).
/// Model-bound bytes (adapter digests, native encodings, tensor shapes,
/// weights) are NEVER inputs here.
pub fn semantic_key(m: &Module) -> [u8; 32] {
    let payload = canonical_semantic_payload(m);
    let mut ver = [0u8; 4];
    ver.copy_from_slice(&CANONICALIZER_VERSION.to_le_bytes());
    sha256_parts(&[
        b"GMSEM-v1",
        &ver,
        m.domain.as_bytes(),
        b"\x00",
        m.scope.as_bytes(),
        b"\x00",
        &payload,
    ])
}

// ---------------------------------------------------------------------------
// Model adapter seam (last model-specific step only)
// ---------------------------------------------------------------------------

/// Frozen-model adapter contract. Implementations translate the typed
/// module to the model's native input representation and back.
pub trait ModelAdapter {
    fn adapter_digest(&self) -> [u8; 32];
    fn weights_digest(&self) -> [u8; 32];
    fn config_digest(&self) -> [u8; 32];
    fn shape_contract(&self) -> String;
    fn execution_contract(&self) -> String;
    /// Disclosure: how this adapter bridges typed IR to native input,
    /// and whether any tokenization/encoding stands between them.
    fn disclosure(&self) -> String;
    fn encode(&self, m: &Module) -> Result<ModelNative, AdapterError>;
    fn decode(&self, native: &ModelNative) -> Result<Vec<NativeResult>, AdapterError>;
}

/// Test-only passthrough adapter: the "model" consumes typed canonical
/// bytes directly. DISCLOSED as test-only; a frozen model that cannot
/// consume typed IR directly MUST NOT use this to claim tokenless execution.
pub struct PassthroughTestAdapter {
    pub weights_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub shape: Vec<usize>,
}

impl PassthroughTestAdapter {
    pub fn new(weights_digest: [u8; 32], config_digest: [u8; 32]) -> Self {
        Self {
            weights_digest,
            config_digest,
            shape: vec![],
        }
    }
    pub fn adapter_digest_static() -> [u8; 32] {
        sha256_parts(&[
            b"BST-ADAPTER-v1",
            b"passthrough-test",
            &ADAPTER_CONTRACT_VERSION.to_le_bytes(),
        ])
    }
}

impl ModelAdapter for PassthroughTestAdapter {
    fn adapter_digest(&self) -> [u8; 32] {
        Self::adapter_digest_static()
    }
    fn weights_digest(&self) -> [u8; 32] {
        self.weights_digest
    }
    fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }
    fn shape_contract(&self) -> String {
        "test-only: native shape mirrors canonical module bytes 1:1".to_string()
    }
    fn execution_contract(&self) -> String {
        "test-only loopback: decode echoes the encoded module; no frozen-model forward pass"
            .to_string()
    }
    fn disclosure(&self) -> String {
        "TEST-ONLY adapter. No tokenizer, no embedding lookup: the test model \
         consumes typed canonical bytes directly. A frozen model that requires \
         native IDs/embeddings needs a real adapter; do not claim tokenless \
         end-to-end execution on its behalf."
            .to_string()
    }
    fn encode(&self, m: &Module) -> Result<ModelNative, AdapterError> {
        validate_module(m).map_err(|e| AdapterError {
            reason: format!("adapter refuses invalid module: {e}"),
        })?;
        let mut bytes = Vec::new();
        let ad = self.adapter_digest();
        push_bytes(&mut bytes, &ad);
        let body = canonical_module_nodigest(m);
        push_bytes(&mut bytes, &body);
        Ok(ModelNative {
            bytes,
            shape: self.shape.clone(),
            adapter_digest: ad,
            weights_digest: self.weights_digest,
            config_digest: self.config_digest,
            execution_contract: self.execution_contract(),
        })
    }
    fn decode(&self, native: &ModelNative) -> Result<Vec<NativeResult>, AdapterError> {
        let fail = |r: String| AdapterError { reason: r };
        if native.adapter_digest != self.adapter_digest() {
            return Err(fail("adapter digest mismatch on decode".to_string()));
        }
        if native.weights_digest != self.weights_digest {
            return Err(fail("weights digest mismatch on decode".to_string()));
        }
        // Parse header: [ad_len u64][ad][body_len u64][body].
        let b = &native.bytes;
        let read_u64 = |off: usize| -> Result<u64, AdapterError> {
            if off + 8 > b.len() {
                return Err(fail("truncated native header".to_string()));
            }
            let mut t = [0u8; 8];
            t.copy_from_slice(&b[off..off + 8]);
            Ok(u64::from_le_bytes(t))
        };
        let ad_len = read_u64(0)? as usize;
        if 8 + ad_len + 8 > b.len() {
            return Err(fail("truncated native body".to_string()));
        }
        let body_len = read_u64(8 + ad_len)? as usize;
        let body_off = 8 + ad_len + 8;
        if body_off + body_len != b.len() {
            return Err(fail("native body length mismatch".to_string()));
        }
        // Test loopback cannot reconstruct ops from bytes alone without the
        // module; the caller pairs decode with the originating module via
        // loopback_results. Direct decode of opaque bytes is refused rather
        // than guessed: return an explicit marker error.
        let _ = (body_len, body_off);
        Err(fail(
            "passthrough decode requires the originating module; use loopback_results + egress_decode"
                .to_string(),
        ))
    }
}
/// Opaque frozen-model adapter: the model cannot consume typed IR directly
/// and no real adapter exists yet. Encode always refuses with disclosure
/// instead of pretending a tokenless path exists.
pub struct OpaqueFrozenAdapter {
    pub weights_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub reason: String,
}

impl ModelAdapter for OpaqueFrozenAdapter {
    fn adapter_digest(&self) -> [u8; 32] {
        sha256_parts(&[b"BST-ADAPTER-v1", b"opaque-frozen"])
    }
    fn weights_digest(&self) -> [u8; 32] {
        self.weights_digest
    }
    fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }
    fn shape_contract(&self) -> String {
        "unknown: frozen model shape not yet bound".to_string()
    }
    fn execution_contract(&self) -> String {
        "refused: no forward pass without a real adapter".to_string()
    }
    fn disclosure(&self) -> String {
        format!(
            "DISCLOSED GAP: frozen model cannot consume typed IR directly ({})",
            self.reason
        )
    }
    fn encode(&self, _m: &Module) -> Result<ModelNative, AdapterError> {
        Err(AdapterError {
            reason: self.disclosure(),
        })
    }
    fn decode(&self, _native: &ModelNative) -> Result<Vec<NativeResult>, AdapterError> {
        Err(AdapterError {
            reason: self.disclosure(),
        })
    }
}

/// Adapter against `rust-server` typed model IR
/// (`Rust-TS-Tokio-Inference-Engine/src/serving/model_ir.rs`:
/// [`ModelStepIr::compile`](../../../Rust-TS-Tokio-Inference-Engine/src/serving/model_ir.rs)
/// + [`HybridProgram::compile`](../../../Rust-TS-Tokio-Inference-Engine/src/serving/model_ir.rs)).
///
/// Honest seam statement: the frozen model's native input is token IDs
/// against vocabulary 248320 (`native_qwen.rs` workhorse config), NOT typed
/// `gm` ops. So `encode` maps each `Utf8` op span to the span's raw bytes
/// hashed into bounded carrier IDs `< vocab`, packs them into prefill steps
/// of `<= 8` tokens each (`ModelStepIr` bound), and records the tokenizer
/// gap explicitly: byte-hash carriers are NOT the model's trained token IDs.
/// A trained-tokenizer encoding step is still required before any forward
/// pass that claims trained-model behavior. `decode` refuses: reversing
/// model token IDs into typed SSA results without the trained detokenizer
/// plus the originating module would be a guess, and guesses are egress
/// failures, not plausible strings.
pub struct ModelIrAdapter {
    /// Vocabulary of the frozen model (e.g. 248320 for Combined).
    pub vocab: u32,
    /// Max context positions (e.g. 512 for the native smoke profile).
    pub max_context: usize,
    /// Digest of the frozen weights this adapter was qualified against.
    pub weights_digest: [u8; 32],
    /// Digest of the frozen model config this adapter was qualified against.
    pub config_digest: [u8; 32],
    /// Layers / full-attention interval of the bound HybridProgram.
    pub layers: usize,
    pub interval: usize,
}

impl ModelIrAdapter {
    pub fn combined_smoke(weights_digest: [u8; 32], config_digest: [u8; 32]) -> Self {
        Self {
            vocab: 248_320,
            max_context: 512,
            weights_digest,
            config_digest,
            layers: 40,
            interval: 4,
        }
    }
    pub fn adapter_digest_static() -> [u8; 32] {
        sha256_parts(&[
            b"BST-ADAPTER-v1",
            b"rust-server-model-ir-v1",
            &ADAPTER_CONTRACT_VERSION.to_le_bytes(),
        ])
    }
    /// Deterministic byte-span to carrier-ID map. NOT a tokenizer: output
    /// IDs are content hashes mod vocab, recorded as unvalidated carriers.
    /// They satisfy `ModelStepIr` geometry (each `< vocab`, `<= 8` per step)
    /// but carry NO trained-model token identity.
    pub fn carrier_ids_for(&self, m: &Module) -> Vec<u32> {
        let mut ids = Vec::new();
        for o in &m.ops {
            let bytes = &m.source.bytes[o.span.start as usize..o.span.end as usize];
            if bytes.is_empty() {
                continue;
            }
            let d = sha256_parts(&[b"BST-CARRIER-v1", bytes, &o.id.to_le_bytes()]);
            let mut off = 0;
            while off < bytes.len() {
                let end = (off + 8).min(bytes.len());
                let mut b8 = [0u8; 8];
                b8[..end - off].copy_from_slice(&d[..end - off]);
                let mut mixed =
                    u64::from_le_bytes(b8) ^ (off as u64).wrapping_mul(0x9E3779B97F4A7C15);
                mixed %= self.vocab as u64;
                ids.push(mixed as u32);
                off += 8;
            }
        }
        ids
    }
    fn steps_for_ids(&self, ids: &[u32]) -> Result<Vec<StepSpec>, AdapterError> {
        let fail = |r: &str| AdapterError {
            reason: r.to_string(),
        };
        if ids.is_empty() {
            return Err(fail("model-ir adapter: module yields no carrier ids"));
        }
        if ids.iter().any(|t| *t >= self.vocab) {
            return Err(fail("model-ir adapter: carrier id out of vocabulary"));
        }
        // Mirror ModelStepIr bounds: <= 8 tokens per step, position + len <= max_context.
        let mut steps = Vec::new();
        let mut pos = 0usize;
        for chunk in ids.chunks(8) {
            if pos + chunk.len() > self.max_context {
                return Err(fail("model-ir adapter: carriers exceed max_context"));
            }
            steps.push(StepSpec {
                position: pos,
                tokens: chunk.to_vec(),
            });
            pos += chunk.len();
        }
        Ok(steps)
    }
}

/// One prefill step bound for `ModelStepIr::compile(kind=Prefill,
/// position, tokens, vocab)`: `tokens.len() <= 8`, each `< vocab`,
/// `position + len <= max_context`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepSpec {
    pub position: usize,
    pub tokens: Vec<u32>,
}

impl ModelAdapter for ModelIrAdapter {
    fn adapter_digest(&self) -> [u8; 32] {
        Self::adapter_digest_static()
    }
    fn weights_digest(&self) -> [u8; 32] {
        self.weights_digest
    }
    fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }
    fn shape_contract(&self) -> String {
        format!(
            "rust-server ModelStepIr prefill: vocab={} max_context={} chunk<=8; HybridProgram layers={} interval={}",
            self.vocab, self.max_context, self.layers, self.interval
        )
    }
    fn execution_contract(&self) -> String {
        "encode-only: builds geometry-valid ModelStepIr prefill steps from unvalidated byte-hash carriers; \
         NO forward pass executed here, NO trained-token claim. Pair with the originating module and a \
         qualified tokenizer/detokenizer before any generation claim."
            .to_string()
    }
    fn disclosure(&self) -> String {
        "DISCLOSED SEAM (rust-server model_ir): typed gm ops are lowered to byte-hash carrier IDs \
         (< vocab, <= 8 per prefill step) that satisfy ModelStepIr geometry but are NOT the frozen \
         model's trained token IDs. A trained tokenizer encoding step stands between this adapter's \
         output and any forward pass that claims trained-model behavior. Tokenless end-to-end \
         execution is NOT claimed."
            .to_string()
    }
    fn encode(&self, m: &Module) -> Result<ModelNative, AdapterError> {
        validate_module(m).map_err(|e| AdapterError {
            reason: format!("adapter refuses invalid module: {e}"),
        })?;
        // Reject non-text-bearing modules: byte-hash carriers are only
        // defined over Utf8 spans here; observation/tool modules have no
        // honest carrier mapping yet (refuse, don't guess).
        for o in &m.ops {
            match (&o.name, &o.payload) {
                (OpName::TextEmit, OpPayload::Text)
                | (OpName::CodeEmit, OpPayload::Code { .. }) => {}
                _ => {
                    return Err(AdapterError {
                        reason: format!(
                            "model-ir adapter: op {} ({}) has no carrier mapping; refuses non-text payload",
                            o.id,
                            o.name.as_str()
                        ),
                    });
                }
            }
        }
        let ids = self.carrier_ids_for(m);
        let steps = self.steps_for_ids(&ids)?;
        // Native bytes: canonical encoding of the bound geometry + steps.
        // Explicitly model-bound: adapter/weights/config digests + vocab +
        // steps. NEVER an input to semantic_key.
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, &self.adapter_digest());
        push_bytes(&mut bytes, &self.weights_digest);
        push_bytes(&mut bytes, &self.config_digest);
        push_u32(&mut bytes, self.vocab);
        push_u64(&mut bytes, self.max_context as u64);
        push_u64(&mut bytes, steps.len() as u64);
        for s in &steps {
            push_u64(&mut bytes, s.position as u64);
            push_u64(&mut bytes, s.tokens.len() as u64);
            for t in &s.tokens {
                push_u32(&mut bytes, *t);
            }
        }
        Ok(ModelNative {
            bytes,
            shape: vec![steps.len(), 8],
            adapter_digest: self.adapter_digest(),
            weights_digest: self.weights_digest,
            config_digest: self.config_digest,
            execution_contract: self.execution_contract(),
        })
    }
    fn decode(&self, _native: &ModelNative) -> Result<Vec<NativeResult>, AdapterError> {
        Err(AdapterError {
            reason: "model-ir adapter decode refused: model token IDs reverse to typed SSA results only \
                through a qualified detokenizer plus the originating module; use loopback_results + \
                egress_decode for tests, never guess"
                .to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Egress
// ---------------------------------------------------------------------------

/// Build loopback results for a module (perfect-model simulation for tests:
/// every SSA result echoes the module's own typed payload).
pub fn loopback_results(m: &Module) -> Vec<NativeResult> {
    m.ops
        .iter()
        .map(|o| {
            let payload = match &o.payload {
                OpPayload::Text | OpPayload::Code { .. } => {
                    let s = o.span.start as usize;
                    let e = o.span.end as usize;
                    ResultPayload::TextBytes(m.source.bytes[s..e].to_vec())
                }
                OpPayload::Observe { .. } => {
                    let s = o.span.start as usize;
                    let e = o.span.end as usize;
                    ResultPayload::RawBytes(m.source.bytes[s..e].to_vec())
                }
                OpPayload::Tool { args, .. } => ResultPayload::ToolArgs(args.clone()),
            };
            NativeResult {
                ssa_id: o.id,
                payload,
                effect_bits: o.effects,
            }
        })
        .collect()
}

/// Decode model-native results against the module: per-carrier/SSA checks,
/// effect validation, strict UTF-8, then semantic rendering.
/// ANY unmapped output is an explicit boxed [`EgressFailure`], never a guess.
/// (Boxed: the failure carries four id-lists; callers use `unwrap_err()` /
/// field access exactly as before via auto-deref.)
pub fn egress_decode(
    m: &Module,
    results: &[NativeResult],
) -> Result<SemanticOutput, Box<EgressFailure>> {
    let fail = |reason: String,
                unknown_ids: Vec<u32>,
                missing_ids: Vec<u32>,
                failed_inverse: Vec<u32>,
                effect_mismatch: Vec<u32>,
                loss_bytes: u64| {
        Box::new(EgressFailure {
            unknown_ids,
            missing_ids,
            failed_inverse,
            effect_mismatch,
            loss_bytes,
            reason,
        })
    };
    // Index results; duplicates are explicit failures.
    let mut by_id: std::collections::BTreeMap<u32, &NativeResult> =
        std::collections::BTreeMap::new();
    let mut dup: Vec<u32> = Vec::new();
    for r in results {
        if by_id.insert(r.ssa_id, r).is_some() {
            dup.push(r.ssa_id);
        }
    }
    if !dup.is_empty() {
        return Err(fail(
            format!("duplicate results for SSA ids {dup:?}"),
            vec![],
            vec![],
            dup,
            vec![],
            0,
        ));
    }
    let known: std::collections::BTreeSet<u32> = m.ops.iter().map(|o| o.id).collect();
    let got: std::collections::BTreeSet<u32> = by_id.keys().copied().collect();
    let unknown_ids: Vec<u32> = got.difference(&known).copied().collect();
    let missing_ids: Vec<u32> = known.difference(&got).copied().collect();
    let mut failed_inverse: Vec<u32> = Vec::new();
    let mut effect_mismatch: Vec<u32> = Vec::new();
    let mut evidence: Vec<OpEvidence> = Vec::new();
    let mut text_out: Vec<u8> = Vec::new();
    let mut is_text_module = true;
    for o in &m.ops {
        match o.name {
            OpName::TextEmit | OpName::CodeEmit => {}
            _ => is_text_module = false,
        }
    }
    for o in &m.ops {
        let r = match by_id.get(&o.id) {
            Some(r) => r,
            None => continue, // counted in missing_ids
        };
        let (_, _, allowed, required) = op_signature(o.name);
        if r.effect_bits & !allowed != 0 || r.effect_bits & required != required {
            effect_mismatch.push(o.id);
            continue;
        }
        let payload_bytes: Vec<u8> = match (&o.payload, &r.payload) {
            (OpPayload::Text, ResultPayload::TextBytes(b))
            | (OpPayload::Code { .. }, ResultPayload::TextBytes(b))
            | (OpPayload::Observe { .. }, ResultPayload::RawBytes(b)) => b.clone(),
            (OpPayload::Tool { .. }, ResultPayload::ToolArgs(_)) => {
                // Canonical-encode the returned args for evidence hashing.
                let mut c = Vec::new();
                if let ResultPayload::ToolArgs(a) = &r.payload {
                    canonical_args(&mut c, a);
                }
                c
            }
            _ => {
                failed_inverse.push(o.id);
                continue;
            }
        };
        // Strict inverse mapping per type.
        match (&o.payload, &r.payload) {
            (OpPayload::Text, ResultPayload::TextBytes(_))
            | (OpPayload::Code { .. }, ResultPayload::TextBytes(_)) => {
                if std::str::from_utf8(&payload_bytes).is_err() {
                    failed_inverse.push(o.id);
                    continue;
                }
                text_out.extend_from_slice(&payload_bytes);
            }
            (OpPayload::Observe { .. }, ResultPayload::RawBytes(_)) => {}
            (OpPayload::Tool { args, .. }, ResultPayload::ToolArgs(back)) => {
                // Arg names and values must round-trip exactly.
                if back.len() != args.len()
                    || back
                        .iter()
                        .zip(args.iter())
                        .any(|(b, a)| b.name != a.name || b.value != a.value)
                {
                    failed_inverse.push(o.id);
                    continue;
                }
            }
            _ => {
                failed_inverse.push(o.id);
                continue;
            }
        }
        evidence.push(OpEvidence {
            ssa_id: o.id,
            name: o.name.as_str().to_string(),
            payload_sha256: sha256_parts(&[b"BST-EV-v1", &payload_bytes]),
        });
    }
    // Loss = source bytes of ops without a valid mapping.
    let bad: std::collections::BTreeSet<u32> = missing_ids
        .iter()
        .chain(failed_inverse.iter())
        .chain(effect_mismatch.iter())
        .copied()
        .collect();
    let mut loss_bytes: u64 = 0;
    for o in &m.ops {
        if bad.contains(&o.id) {
            loss_bytes += (o.span.end - o.span.start) as u64;
        }
    }
    if !unknown_ids.is_empty()
        || !missing_ids.is_empty()
        || !failed_inverse.is_empty()
        || !effect_mismatch.is_empty()
    {
        return Err(fail(
            "unmapped or invalid SSA results; output refused, not guessed".to_string(),
            unknown_ids,
            missing_ids,
            failed_inverse,
            effect_mismatch,
            loss_bytes,
        ));
    }
    let text = if is_text_module {
        match String::from_utf8(text_out) {
            Ok(t) => Some(t),
            Err(_) => {
                return Err(fail(
                    "rendered text is not strict UTF-8".to_string(),
                    vec![],
                    vec![],
                    m.ops.iter().map(|o| o.id).collect(),
                    vec![],
                    m.source.bytes.len() as u64,
                ));
            }
        }
    } else {
        None
    };
    Ok(SemanticOutput {
        text,
        evidence,
        loss: LossReceipt {
            unknown_ids: 0,
            failed_inverse: 0,
            loss_bytes: 0,
        },
    })
}

// ---------------------------------------------------------------------------
// Replay (version-change gate)
// ---------------------------------------------------------------------------

/// Re-validate a module at its frozen address revision and receipt it.
/// A version change must produce a NEW module + receipt, never an edit.
pub fn replay(m: &Module) -> Result<ReplayReceipt, String> {
    let v = validate_module(m)?;
    Ok(ReplayReceipt {
        module_digest: v.digest,
        address_rev: m.address_rev,
        versions: m.versions,
        ops: v.ops,
        loss_bytes: 0,
    })
}

/// Hex helper for receipts/logs.
pub fn hex32(d: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// Unit tests (contract pins)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn prov() -> Provenance {
        Provenance::new("test-source", "test-actor", "unit").unwrap()
    }

    fn ident() -> [[f64; 10]; 10] {
        let mut j = [[0.0f64; 10]; 10];
        for (i, row) in j.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        j
    }

    #[test]
    fn omega_is_nondegenerate() {
        // Skew-symmetry + square = -I  =>  det = +/-1 != 0. The two-form is
        // nondegenerate by construction, independent of any data.
        let o = symplectic_form();
        for (i, row) in o.iter().enumerate() {
            for (k, cell) in row.iter().enumerate() {
                assert_eq!(*cell, -o[k][i], "skew-symmetry");
            }
        }
        for (i, row) in o.iter().enumerate() {
            for (k, _) in row.iter().enumerate() {
                let s: f64 = row.iter().zip(o.iter()).map(|(a, orow)| a * orow[k]).sum();
                let want = if i == k { -1.0 } else { 0.0 };
                assert!((s - want).abs() < 1e-15, "square = -I");
            }
        }
    }

    #[test]
    fn identity_transport_is_canonical() {
        let r = check_canonical_transport(&ident(), DEFAULT_SYMPLECTIC_TOL).unwrap();
        assert!(r < 1e-12, "identity residual {r}");
    }

    #[test]
    fn shear_transport_is_not_canonical() {
        // Hyperbolic-coordinate shear: scales one position axis. Must FAIL
        // the symplectic test -> label stays phase-augmented.
        let mut j = ident();
        j[0][0] = 2.0;
        let e = check_canonical_transport(&j, DEFAULT_SYMPLECTIC_TOL).unwrap_err();
        assert!(e.contains("phase-augmented"), "got: {e}");
    }

    #[test]
    fn tags_roundtrip() {
        for t in [
            MlirType::Utf8,
            MlirType::Bytes,
            MlirType::Handle,
            MlirType::Unit,
        ] {
            assert_eq!(mlir_type_from_tag(t.tag()), Some(t));
        }
        for n in [
            OpName::TextEmit,
            OpName::CodeEmit,
            OpName::Observe,
            OpName::ToolCall,
        ] {
            assert_eq!(op_name_from_tag(n.tag()), Some(n));
        }
        assert_eq!(mlir_type_from_tag(0), None);
        assert_eq!(mlir_type_from_tag(99), None);
        assert_eq!(op_name_from_tag(0), None);
        assert_eq!(op_name_from_tag(99), None);
    }

    #[test]
    fn default_claim_is_phase_augmented() {
        let m = ingress_text(b"hello", None, prov(), "d", "s").unwrap();
        assert_eq!(m.claim, GeometryClaim::PhaseAugmented);
        let v = validate_module(&m).unwrap();
        assert_eq!(v.claim, GeometryClaim::PhaseAugmented);
    }

    #[test]
    fn q3231_rejects_nonfinite() {
        assert!(Q32_31::from_f64(f64::NAN).is_err());
        assert!(Q32_31::from_f64(f64::INFINITY).is_err());
        let one = Q32_31::from_f64(1.0).unwrap();
        assert_eq!(one, Q32_31::ONE);
        assert!((one.to_f64() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ingress_rejects_invalid_utf8() {
        let r = ingress_text(&[0xFF, 0xFE], None, prov(), "d", "s").unwrap_err();
        assert!(r.reason.contains("UTF-8"));
        assert_eq!(r.input_digest, input_digest(&[0xFF, 0xFE]));
    }

    #[test]
    fn text_roundtrip_exact() {
        let text = "La résonance — 捕获共振 — يحدث: κ(T,t) → 0 ✓\nline two\n";
        let m = ingress_text(text.as_bytes(), Some("mul"), prov(), "d", "s").unwrap();
        validate_module(&m).unwrap();
        let res = loopback_results(&m);
        let out = egress_decode(&m, &res).unwrap();
        assert_eq!(out.text.unwrap(), text);
        assert_eq!(out.loss.loss_bytes, 0);
    }

    #[test]
    fn semantic_key_excludes_model_bytes() {
        let m = ingress_text(b"key test", None, prov(), "d", "s").unwrap();
        let k1 = semantic_key(&m);
        // Geometry + digests + adapter bytes must not move the shared key.
        let mut m2 = m.clone();
        let (p, mo) = derive_address(b"other", 0, OpName::TextEmit);
        m2.ops[0].pos = p;
        m2.ops[0].mom = mo;
        m2.digest = module_digest(&m2);
        assert_eq!(k1, semantic_key(&m2));
    }

    #[test]
    fn opaque_adapter_discloses() {
        let a = OpaqueFrozenAdapter {
            weights_digest: [7u8; 32],
            config_digest: [8u8; 32],
            reason: "needs native embedding IDs".to_string(),
        };
        let m = ingress_text(b"x", None, prov(), "d", "s").unwrap();
        let e = a.encode(&m).unwrap_err();
        assert!(e.reason.contains("DISCLOSED"), "got: {}", e.reason);
    }

    #[test]
    fn egress_refuses_unknown_ids() {
        let m = ingress_text(b"ab\n", None, prov(), "d", "s").unwrap();
        let mut res = loopback_results(&m);
        res.push(NativeResult {
            ssa_id: 9999,
            payload: ResultPayload::TextBytes(b"evil".to_vec()),
            effect_bits: EFF_EMIT,
        });
        let e = egress_decode(&m, &res).unwrap_err();
        assert_eq!(e.unknown_ids, vec![9999]);
        assert!(e.loss_bytes == 0, "unknown ids carry no source loss");
    }

    #[test]
    fn egress_refuses_bad_utf8_payload() {
        let m = ingress_text(b"ab\n", None, prov(), "d", "s").unwrap();
        let mut res = loopback_results(&m);
        res[0].payload = ResultPayload::TextBytes(vec![0xFF]);
        let e = egress_decode(&m, &res).unwrap_err();
        assert_eq!(e.failed_inverse, vec![0]);
        assert!(e.loss_bytes > 0);
    }
}
