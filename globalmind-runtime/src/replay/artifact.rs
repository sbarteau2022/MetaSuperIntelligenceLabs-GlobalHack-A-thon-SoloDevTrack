//! Immutable replay walk artifact (`GMRW` v1, binary only).
//!
//! The artifact carries replay INPUTS plus expected digests — never derived
//! outputs. Crystallize re-runs [`crate::replay::replay`] from the decoded
//! inputs and compares digests, so any drift is a miss. Layout (all integers
//! little-endian):
//!
//! ```text
//! magic "GMRW" (4) | version u16 | section_count u32
//! per section: tag u16 | length u32 | bytes
//! trailing sha256 (32) over every preceding byte
//! ```
//!
//! Section tags: 1 context, 2 candidates, 3 receipts, 4 envelope descriptors,
//! 5 chart receipts, 6 expected digests. Field encoding inside sections uses
//!! the same length-prefixed primitives; f64 values are exact LE bytes.

use crate::extract::ExtractedCandidate;
use crate::judge::{JudgeReceipt, JudgeVerdict};
use crate::replay::{ChartReceipts, ReplayContext};
use crate::trace::{TraceDtype, TraceEnvelope, TraceKind, TraceProvenance};
use crate::{CellKind, Digest, RadialLaw, RecordId, TppeConfig};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;

pub const WALK_MAGIC: [u8; 4] = *b"GMRW";
pub const WALK_VERSION: u16 = 1;
pub const CANDIDATES_MAGIC: [u8; 4] = *b"GMCD";
pub const CANDIDATES_VERSION: u16 = 1;
pub const VERDICTS_MAGIC: [u8; 4] = *b"GMVD";
pub const VERDICTS_VERSION: u16 = 1;

#[derive(Clone, Debug)]
pub struct DecodedCandidates {
    pub provenance: TraceProvenance,
    pub envelopes: Vec<TraceEnvelope>,
    pub candidates: Vec<ExtractedCandidate>,
}

#[derive(Clone, Debug)]
pub struct DecodedVerdicts {
    pub provenance: TraceProvenance,
    pub envelopes: Vec<TraceEnvelope>,
    pub judged: Vec<(ExtractedCandidate, JudgeReceipt)>,
    pub chart_receipts: BTreeMap<Digest, ChartReceipts>,
    pub eligibility_receipt_digest: Digest,
    pub judge_receipt_digest: Digest,
}

/// Source envelope descriptor (no payload bytes): enough to match rank-8
/// tangent bases and to audit the walk without the full trace file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvelopeDescriptor {
    pub kind: TraceKind,
    pub dtype: TraceDtype,
    pub shape: Vec<u32>,
    pub layer: u32,
    pub head: u32,
    pub token_position: u64,
    pub payload_digest: Digest,
}

/// Decoded walk file: replay inputs plus expected digests.
#[derive(Clone, Debug)]
pub struct WalkFile {
    pub context: ReplayContext,
    pub judged: Vec<(ExtractedCandidate, JudgeReceipt)>,
    pub envelopes: Vec<EnvelopeDescriptor>,
    pub chart_receipts: BTreeMap<Digest, ChartReceipts>,
    /// Source language the atoms carry, when the walk was built with one.
    pub source: Option<crate::replay::source::SourceText>,
    pub trajectory_digest: Digest,
    pub candidate_bundle_digest: Digest,
    pub verdict_bundle_digest: Digest,
    pub replay_receipt_digest: Digest,
}

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn bytes(&mut self, raw: &[u8]) {
        self.buf
            .extend_from_slice(&(raw.len() as u32).to_le_bytes());
        self.buf.extend_from_slice(raw);
    }

    fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    fn digest(&mut self, value: &Digest) {
        self.buf.extend_from_slice(&digest_bytes(value));
    }

    fn id(&mut self, value: &RecordId) {
        self.text(value.as_str());
    }

    fn f64_value(&mut self, value: f64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize, what: &str) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| format!("walk artifact overflows reading {what}"))?;
        if end > self.bytes.len() {
            return Err(format!("walk artifact truncated reading {what}"));
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u16_value(&mut self, what: &str) -> Result<u16, String> {
        let raw = self.take(2, what)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    fn u32_value(&mut self, what: &str) -> Result<u32, String> {
        let raw = self.take(4, what)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    fn u64_value(&mut self, what: &str) -> Result<u64, String> {
        let raw = self.take(8, what)?;
        Ok(u64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    fn f64_value(&mut self, what: &str) -> Result<f64, String> {
        let raw = self.take(8, what)?;
        Ok(f64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    fn bytes(&mut self, what: &str) -> Result<&'a [u8], String> {
        let len = self.u32_value(what)? as usize;
        self.take(len, what)
    }

    fn text(&mut self, what: &str) -> Result<String, String> {
        let raw = self.bytes(what)?;
        String::from_utf8(raw.to_vec()).map_err(|_| format!("walk artifact {what} is not UTF-8"))
    }

    fn digest(&mut self, what: &str) -> Result<Digest, String> {
        let raw = self.take(32, what)?;
        let mut hex = String::with_capacity(64);
        for byte in raw {
            use std::fmt::Write as _;
            write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        Digest::parse(hex)
    }

    fn id(&mut self, what: &str) -> Result<RecordId, String> {
        RecordId::parse(self.text(what)?)
    }

    fn finish(&self) -> Result<(), String> {
        if self.pos != self.bytes.len() {
            return Err(format!(
                "walk artifact has {} trailing bytes",
                self.bytes.len() - self.pos
            ));
        }
        Ok(())
    }
}

fn cell_kind_code(kind: CellKind) -> u8 {
    match kind {
        CellKind::Rank0Atom => 1,
        CellKind::Rank1Transition => 2,
        CellKind::Rank2Loop => 3,
        CellKind::Rank8Jacobian => 4,
    }
}

fn cell_kind_from(code: u8) -> Result<CellKind, String> {
    match code {
        1 => Ok(CellKind::Rank0Atom),
        2 => Ok(CellKind::Rank1Transition),
        3 => Ok(CellKind::Rank2Loop),
        4 => Ok(CellKind::Rank8Jacobian),
        _ => Err(format!("unknown cell kind code {code}")),
    }
}

fn reason_code(reason: &str) -> Result<u8, String> {
    match reason {
        "admitted by frozen policy" => Ok(1),
        "duplicate candidate digest" => Ok(2),
        "non-finite trace evidence" => Ok(3),
        "rank-0 energy below frozen floor" => Ok(4),
        "chart rank above frozen cap" => Ok(5),
        "chart without tangent basis" => Ok(6),
        _ => Err(format!("unknown judge reason {reason:?}")),
    }
}

fn reason_from(code: u8) -> Result<&'static str, String> {
    match code {
        1 => Ok("admitted by frozen policy"),
        2 => Ok("duplicate candidate digest"),
        3 => Ok("non-finite trace evidence"),
        4 => Ok("rank-0 energy below frozen floor"),
        5 => Ok("chart rank above frozen cap"),
        6 => Ok("chart without tangent basis"),
        _ => Err(format!("unknown judge reason code {code}")),
    }
}

fn encode_candidate(out: &mut Writer, candidate: &ExtractedCandidate) -> Result<(), String> {
    out.id(&candidate.id);
    out.buf.push(cell_kind_code(candidate.cell_kind));
    out.digest(&candidate.source_fragment_digest);
    out.digest(&candidate.teacher_neutral_input_digest);
    out.digest(&candidate.candidate_digest);
    match candidate.h3_lorentz {
        Some(h3) => {
            out.buf.push(1);
            for value in h3 {
                out.f64_value(value);
            }
        }
        None => out.buf.push(0),
    }
    match candidate.t2_phase {
        Some(t2) => {
            out.buf.push(1);
            for value in t2 {
                out.f64_value(value);
            }
        }
        None => out.buf.push(0),
    }
    match candidate.well_ordinal {
        Some(well) => {
            out.buf.push(1);
            out.buf.push(well);
        }
        None => out.buf.push(0),
    }
    for atom in [
        &candidate.from_atom,
        &candidate.to_atom,
        &candidate.anchor_atom,
    ] {
        match atom {
            Some(id) => {
                out.buf.push(1);
                out.id(id);
            }
            None => out.buf.push(0),
        }
    }
    for step in [&candidate.first_step, &candidate.last_step] {
        match step {
            Some(value) => {
                out.buf.push(1);
                out.buf.extend_from_slice(&value.to_le_bytes());
            }
            None => out.buf.push(0),
        }
    }
    match candidate.chart_rank {
        Some(rank) => {
            out.buf.push(1);
            out.buf.push(rank);
        }
        None => out.buf.push(0),
    }
    match &candidate.tangent_basis {
        Some(bytes) => {
            out.buf.push(1);
            out.bytes(bytes);
        }
        None => out.buf.push(0),
    }
    match &candidate.tensor_shape {
        Some(shape) => {
            out.buf.push(1);
            out.buf
                .extend_from_slice(&(shape.len() as u32).to_le_bytes());
            for dimension in shape {
                out.buf.extend_from_slice(&dimension.to_le_bytes());
            }
        }
        None => out.buf.push(0),
    }
    Ok(())
}

fn decode_candidate(reader: &mut Reader) -> Result<ExtractedCandidate, String> {
    let id = reader.id("candidate id")?;
    let kind = cell_kind_from(*reader.take(1, "cell kind")?.first().unwrap_or(&0))?;
    let source_fragment_digest = reader.digest("source fragment")?;
    let teacher_neutral_input_digest = reader.digest("neutral input")?;
    let candidate_digest = reader.digest("candidate digest")?;
    let h3_lorentz = match reader.take(1, "h3 flag")?[0] {
        1 => Some([
            reader.f64_value("h3[0]")?,
            reader.f64_value("h3[1]")?,
            reader.f64_value("h3[2]")?,
            reader.f64_value("h3[3]")?,
        ]),
        0 => None,
        flag => return Err(format!("invalid h3 flag {flag}")),
    };
    let t2_phase = match reader.take(1, "t2 flag")?[0] {
        1 => Some([reader.f64_value("t2[0]")?, reader.f64_value("t2[1]")?]),
        0 => None,
        flag => return Err(format!("invalid t2 flag {flag}")),
    };
    let well_ordinal = match reader.take(1, "well flag")?[0] {
        1 => Some(reader.take(1, "well")?[0]),
        0 => None,
        flag => return Err(format!("invalid well flag {flag}")),
    };
    let mut atoms = Vec::new();
    for name in ["from", "to", "anchor"] {
        match reader.take(1, name)?[0] {
            1 => atoms.push(Some(reader.id(name)?)),
            0 => atoms.push(None),
            flag => return Err(format!("invalid {name} flag {flag}")),
        }
    }
    let mut steps = Vec::new();
    for name in ["first_step", "last_step"] {
        match reader.take(1, name)?[0] {
            1 => steps.push(Some(reader.u32_value(name)?)),
            0 => steps.push(None),
            flag => return Err(format!("invalid {name} flag {flag}")),
        }
    }
    let chart_rank = match reader.take(1, "rank flag")?[0] {
        1 => Some(reader.take(1, "rank")?[0]),
        0 => None,
        flag => return Err(format!("invalid rank flag {flag}")),
    };
    let tangent_basis = match reader.take(1, "basis flag")?[0] {
        1 => Some(reader.bytes("tangent basis")?.to_vec().into_boxed_slice()),
        0 => None,
        flag => return Err(format!("invalid basis flag {flag}")),
    };
    let tensor_shape = match reader.take(1, "shape flag")?[0] {
        1 => {
            let len = reader.u32_value("shape len")? as usize;
            if len > 8 {
                return Err("walk shape exceeds 8 dimensions".into());
            }
            let mut shape = Vec::with_capacity(len);
            for _ in 0..len {
                shape.push(reader.u32_value("shape dim")?);
            }
            Some(shape)
        }
        0 => None,
        flag => return Err(format!("invalid shape flag {flag}")),
    };
    Ok(ExtractedCandidate {
        id,
        cell_kind: kind,
        source_fragment_digest,
        teacher_neutral_input_digest,
        candidate_digest,
        h3_lorentz,
        t2_phase,
        well_ordinal,
        from_atom: atoms[0].clone(),
        to_atom: atoms[1].clone(),
        first_step: steps[0],
        last_step: steps[1],
        anchor_atom: atoms[2].clone(),
        chart_rank,
        tangent_basis,
        tensor_shape,
    })
}

fn encode_receipt(out: &mut Writer, receipt: &JudgeReceipt) -> Result<(), String> {
    out.digest(&receipt.candidate_digest);
    out.buf.push(match receipt.verdict {
        JudgeVerdict::Pass => 1,
        JudgeVerdict::Reject => 2,
    });
    out.buf.push(reason_code(receipt.reason)?);
    out.digest(&receipt.policy_digest);
    out.digest(&receipt.evidence_digest);
    out.digest(&receipt.receipt_digest);
    out.digest(&receipt.novelty_receipt_digest);
    Ok(())
}

fn decode_receipt(reader: &mut Reader) -> Result<JudgeReceipt, String> {
    let candidate_digest = reader.digest("receipt candidate")?;
    let verdict = match reader.take(1, "verdict")?[0] {
        1 => JudgeVerdict::Pass,
        2 => JudgeVerdict::Reject,
        code => return Err(format!("unknown verdict code {code}")),
    };
    let reason = reason_from(reader.take(1, "reason")?[0])?;
    Ok(JudgeReceipt {
        candidate_digest,
        verdict,
        reason,
        policy_digest: reader.digest("policy")?,
        evidence_digest: reader.digest("evidence")?,
        receipt_digest: reader.digest("receipt")?,
        novelty_receipt_digest: reader.digest("novelty")?,
    })
}

fn encode_context(out: &mut Writer, context: &ReplayContext) {
    out.id(&context.workspace);
    out.id(&context.run);
    out.id(&context.admission);
    out.digest(&context.model_roster_digest);
    out.digest(&context.configuration_digest);
    out.text(&context.tppe_version);
    out.digest(&context.tppe_configuration_digest);
    out.buf.push(match context.radial_law {
        RadialLaw::LinearLogV1 => 1,
        RadialLaw::LogV1 => 2,
    });
    for value in [
        context.tppe_numeric.radial_start,
        context.tppe_numeric.radial_span,
        context.tppe_numeric.omega_fast,
        context.tppe_numeric.omega_slow,
        context.tppe_numeric.coupling_fast,
        context.tppe_numeric.coupling_slow,
    ] {
        out.f64_value(value);
    }
    out.digest(&context.address_contract_digest);
    out.digest(&context.runtime_digest);
}

fn decode_context(reader: &mut Reader) -> Result<ReplayContext, String> {
    let workspace = reader.id("workspace")?;
    let run = reader.id("run")?;
    let admission = reader.id("admission")?;
    let model_roster_digest = reader.digest("roster")?;
    let configuration_digest = reader.digest("configuration")?;
    let tppe_version = reader.text("tpe version")?;
    let tppe_configuration_digest = reader.digest("tpe configuration")?;
    let radial_law = match reader.take(1, "radial law")?[0] {
        1 => RadialLaw::LinearLogV1,
        2 => RadialLaw::LogV1,
        code => return Err(format!("unknown radial law code {code}")),
    };
    let numeric = [
        reader.f64_value("radial_start")?,
        reader.f64_value("radial_span")?,
        reader.f64_value("omega_fast")?,
        reader.f64_value("omega_slow")?,
        reader.f64_value("coupling_fast")?,
        reader.f64_value("coupling_slow")?,
    ];
    if !numeric.iter().all(|v| v.is_finite()) {
        return Err("walk TPPE numerics must be finite".into());
    }
    let context = ReplayContext {
        workspace,
        run,
        admission,
        model_roster_digest,
        configuration_digest,
        tppe_version,
        tppe_configuration_digest,
        radial_law,
        tppe_numeric: TppeConfig {
            law: radial_law,
            radial_start: numeric[0],
            radial_span: numeric[1],
            omega_fast: numeric[2],
            omega_slow: numeric[3],
            coupling_fast: numeric[4],
            coupling_slow: numeric[5],
        },
        address_contract_digest: reader.digest("address contract")?,
        runtime_digest: reader.digest("runtime")?,
    };
    context.validate()?;
    Ok(context)
}

fn encode_descriptor(out: &mut Writer, descriptor: &EnvelopeDescriptor) {
    out.buf.push(descriptor.kind.code());
    out.buf.push(descriptor.dtype.code());
    out.buf
        .extend_from_slice(&(descriptor.shape.len() as u32).to_le_bytes());
    for dimension in &descriptor.shape {
        out.buf.extend_from_slice(&dimension.to_le_bytes());
    }
    out.buf.extend_from_slice(&descriptor.layer.to_le_bytes());
    out.buf.extend_from_slice(&descriptor.head.to_le_bytes());
    out.buf
        .extend_from_slice(&descriptor.token_position.to_le_bytes());
    out.digest(&descriptor.payload_digest);
}

fn decode_descriptor(reader: &mut Reader) -> Result<EnvelopeDescriptor, String> {
    let kind = crate::trace::TraceKind::from_code(reader.take(1, "kind")?[0])?;
    let dtype = TraceDtype::from_code(reader.take(1, "dtype")?[0])?;
    let len = reader.u32_value("descriptor shape len")? as usize;
    if len == 0 || len > 8 {
        return Err("walk descriptor shape must have 1..=8 dimensions".into());
    }
    let mut shape = Vec::with_capacity(len);
    for _ in 0..len {
        shape.push(reader.u32_value("descriptor dim")?);
    }
    let layer = reader.u32_value("layer")?;
    let head = reader.u32_value("head")?;
    let token_position = reader.u64_value("token position")?;
    Ok(EnvelopeDescriptor {
        kind,
        dtype,
        shape,
        layer,
        head,
        token_position,
        payload_digest: reader.digest("payload")?,
    })
}

/// Encodes replay inputs plus expected digests as one immutable artifact.
#[allow(clippy::too_many_arguments)]
pub fn encode(
    context: &ReplayContext,
    judged: &[(ExtractedCandidate, JudgeReceipt)],
    envelopes: &[EnvelopeDescriptor],
    chart_receipts: &BTreeMap<Digest, ChartReceipts>,
    trajectory_digest: &Digest,
    candidate_bundle_digest: &Digest,
    verdict_bundle_digest: &Digest,
    replay_receipt_digest: &Digest,
    source: Option<&crate::replay::source::SourceText>,
) -> Result<Vec<u8>, String> {
    if judged.is_empty() {
        return Err("walk artifact requires at least one judged candidate".into());
    }
    let mut sections: Vec<(u16, Vec<u8>)> = Vec::new();
    let mut context_bytes = Writer { buf: Vec::new() };
    encode_context(&mut context_bytes, context);
    sections.push((1, context_bytes.buf));

    let mut candidate_bytes = Writer { buf: Vec::new() };
    candidate_bytes
        .buf
        .extend_from_slice(&(judged.len() as u32).to_le_bytes());
    let mut receipt_bytes = Writer { buf: Vec::new() };
    receipt_bytes
        .buf
        .extend_from_slice(&(judged.len() as u32).to_le_bytes());
    for (candidate, receipt) in judged {
        encode_candidate(&mut candidate_bytes, candidate)?;
        encode_receipt(&mut receipt_bytes, receipt)?;
    }
    sections.push((2, candidate_bytes.buf));
    sections.push((3, receipt_bytes.buf));

    let mut descriptor_bytes = Writer { buf: Vec::new() };
    descriptor_bytes
        .buf
        .extend_from_slice(&(envelopes.len() as u32).to_le_bytes());
    for descriptor in envelopes {
        encode_descriptor(&mut descriptor_bytes, descriptor);
    }
    sections.push((4, descriptor_bytes.buf));

    let mut chart_bytes = Writer { buf: Vec::new() };
    chart_bytes
        .buf
        .extend_from_slice(&(chart_receipts.len() as u32).to_le_bytes());
    for (candidate, receipts) in chart_receipts {
        chart_bytes.digest(candidate);
        chart_bytes.digest(&receipts.marginal_gain_receipt_digest);
        chart_bytes.digest(&receipts.numerical_receipt_digest);
    }
    sections.push((5, chart_bytes.buf));

    let mut digest_bytes_out = Writer { buf: Vec::new() };
    digest_bytes_out.digest(trajectory_digest);
    digest_bytes_out.digest(candidate_bundle_digest);
    digest_bytes_out.digest(verdict_bundle_digest);
    digest_bytes_out.digest(replay_receipt_digest);
    sections.push((6, digest_bytes_out.buf));
    if let Some(source) = source {
        sections.push((7, source.to_bytes()));
    }

    let mut out = Vec::new();
    out.extend_from_slice(&WALK_MAGIC);
    out.extend_from_slice(&WALK_VERSION.to_le_bytes());
    out.extend_from_slice(&(sections.len() as u32).to_le_bytes());
    for (tag, bytes) in &sections {
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
    }
    let file_digest = Sha256::digest(&out);
    out.extend_from_slice(&file_digest);
    Ok(out)
}

/// Decodes a walk artifact, verifying magic, version, section tags, and the
/// trailing file digest. Decoded inputs must still pass [`crate::replay::replay`].
pub fn decode(bytes: &[u8]) -> Result<WalkFile, String> {
    if bytes.len() < 4 + 2 + 4 + 32 {
        return Err("walk artifact is truncated".into());
    }
    if bytes[..4] != WALK_MAGIC {
        return Err("walk artifact magic mismatch".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != WALK_VERSION {
        return Err(format!("unsupported walk version: {version}"));
    }
    let body_end = bytes.len() - 32;
    if Sha256::digest(&bytes[..body_end])[..] != bytes[body_end..] {
        return Err("walk artifact file digest mismatch".into());
    }
    let mut reader = Reader {
        bytes: &bytes[..body_end],
        pos: 6,
    };
    let section_count = reader.u32_value("section count")? as usize;
    let mut sections: BTreeMap<u16, &[u8]> = BTreeMap::new();
    while reader.pos < body_end {
        let tag = reader.u16_value("section tag")?;
        let len = reader.u32_value("section len")? as usize;
        let data = reader.take(len, "section body")?;
        if sections.insert(tag, data).is_some() {
            return Err(format!("duplicate walk section {tag}"));
        }
    }
    if sections.len() != section_count {
        return Err("walk section count mismatch".into());
    }
    for required in [1u16, 2, 3, 4, 5, 6] {
        if !sections.contains_key(&required) {
            return Err(format!("walk artifact is missing section {required}"));
        }
    }
    let mut context_reader = Reader {
        bytes: sections[&1],
        pos: 0,
    };
    let context = decode_context(&mut context_reader)?;
    context_reader.finish()?;

    let mut candidate_reader = Reader {
        bytes: sections[&2],
        pos: 0,
    };
    let candidate_count = candidate_reader.u32_value("candidate count")? as usize;
    let mut receipt_reader = Reader {
        bytes: sections[&3],
        pos: 0,
    };
    let receipt_count = receipt_reader.u32_value("receipt count")? as usize;
    if candidate_count == 0 || candidate_count != receipt_count {
        return Err("walk candidate/receipt counts are invalid".into());
    }
    let mut judged = Vec::with_capacity(candidate_count);
    for _ in 0..candidate_count {
        judged.push((
            decode_candidate(&mut candidate_reader)?,
            decode_receipt(&mut receipt_reader)?,
        ));
    }
    candidate_reader.finish()?;
    receipt_reader.finish()?;

    let mut descriptor_reader = Reader {
        bytes: sections[&4],
        pos: 0,
    };
    let descriptor_count = descriptor_reader.u32_value("descriptor count")? as usize;
    let mut envelopes = Vec::with_capacity(descriptor_count);
    for _ in 0..descriptor_count {
        envelopes.push(decode_descriptor(&mut descriptor_reader)?);
    }
    descriptor_reader.finish()?;

    let mut chart_reader = Reader {
        bytes: sections[&5],
        pos: 0,
    };
    let chart_count = chart_reader.u32_value("chart count")? as usize;
    let mut chart_receipts = BTreeMap::new();
    for _ in 0..chart_count {
        let candidate = chart_reader.digest("chart candidate")?;
        let receipts = ChartReceipts {
            marginal_gain_receipt_digest: chart_reader.digest("marginal gain")?,
            numerical_receipt_digest: chart_reader.digest("numerical")?,
        };
        if chart_receipts.insert(candidate, receipts).is_some() {
            return Err("duplicate chart receipt entry".into());
        }
    }
    chart_reader.finish()?;

    let mut digest_reader = Reader {
        bytes: sections[&6],
        pos: 0,
    };
    let trajectory_digest = digest_reader.digest("trajectory")?;
    let candidate_bundle_digest = digest_reader.digest("candidates")?;
    let verdict_bundle_digest = digest_reader.digest("verdicts")?;
    let replay_receipt_digest = digest_reader.digest("replay receipt")?;
    digest_reader.finish()?;

    let source = match sections.get(&7) {
        Some(raw) => Some(crate::replay::source::SourceText::from_bytes(raw)?),
        None => None,
    };
    if sections.keys().any(|tag| !(1..=7).contains(tag)) {
        return Err("walk artifact has an unknown section".into());
    }
    Ok(WalkFile {
        context,
        judged,
        envelopes,
        chart_receipts,
        source,
        trajectory_digest,
        candidate_bundle_digest,
        verdict_bundle_digest,
        replay_receipt_digest,
    })
}

fn digest_bytes(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let mut out = [0u8; 32];
    for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

pub fn encode_candidates(
    provenance: &TraceProvenance,
    envelopes: &[TraceEnvelope],
    candidates: &[ExtractedCandidate],
) -> Result<Vec<u8>, String> {
    if candidates.is_empty() {
        return Err("candidates artifact requires at least one candidate".into());
    }
    let prov_bytes = crate::trace::encode_provenance_payload(provenance);
    let trace_bytes = crate::trace::encode_trace_stream(envelopes)?;
    let mut cand_writer = Writer { buf: Vec::new() };
    cand_writer
        .buf
        .extend_from_slice(&(candidates.len() as u32).to_le_bytes());
    for candidate in candidates {
        encode_candidate(&mut cand_writer, candidate)?;
    }
    let mut out = Vec::new();
    out.extend_from_slice(&CANDIDATES_MAGIC);
    out.extend_from_slice(&CANDIDATES_VERSION.to_le_bytes());
    out.extend_from_slice(&(prov_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&prov_bytes);
    out.extend_from_slice(&(trace_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&trace_bytes);
    out.extend_from_slice(&(cand_writer.buf.len() as u32).to_le_bytes());
    out.extend_from_slice(&cand_writer.buf);
    let checksum = Sha256::digest(&out);
    out.extend_from_slice(&checksum);
    Ok(out)
}

pub fn decode_candidates(bytes: &[u8]) -> Result<DecodedCandidates, String> {
    if bytes.len() < 4 + 2 + 4 + 4 + 4 + 32 {
        return Err("candidates artifact is truncated".into());
    }
    if bytes[..4] != CANDIDATES_MAGIC {
        return Err("candidates artifact magic mismatch".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != CANDIDATES_VERSION {
        return Err(format!("unsupported candidates version: {version}"));
    }
    let body_end = bytes.len() - 32;
    if Sha256::digest(&bytes[..body_end])[..] != bytes[body_end..] {
        return Err("candidates artifact file digest mismatch".into());
    }
    let mut reader = Reader {
        bytes: &bytes[..body_end],
        pos: 6,
    };
    let prov_len = reader.u32_value("prov len")? as usize;
    let prov_raw = reader.take(prov_len, "provenance payload")?;
    let provenance = crate::trace::decode_provenance_payload(prov_raw)?;

    let trace_len = reader.u32_value("trace len")? as usize;
    let trace_raw = reader.take(trace_len, "trace stream")?;
    let envelopes = crate::trace::decode_trace_stream(trace_raw)?;

    let cand_section_len = reader.u32_value("candidates section len")? as usize;
    let cand_raw = reader.take(cand_section_len, "candidates bytes")?;
    reader.finish()?;

    let mut cand_reader = Reader {
        bytes: cand_raw,
        pos: 0,
    };
    let count = cand_reader.u32_value("candidate count")? as usize;
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(decode_candidate(&mut cand_reader)?);
    }
    cand_reader.finish()?;

    Ok(DecodedCandidates {
        provenance,
        envelopes,
        candidates,
    })
}

pub fn encode_verdicts(
    provenance: &TraceProvenance,
    envelopes: &[TraceEnvelope],
    judged: &[(ExtractedCandidate, JudgeReceipt)],
    chart_receipts: &BTreeMap<Digest, ChartReceipts>,
    eligibility_receipt_digest: &Digest,
    judge_receipt_digest: &Digest,
) -> Result<Vec<u8>, String> {
    if judged.is_empty() {
        return Err("verdicts artifact requires at least one judged candidate".into());
    }
    let prov_bytes = crate::trace::encode_provenance_payload(provenance);
    let trace_bytes = crate::trace::encode_trace_stream(envelopes)?;
    let mut judged_writer = Writer { buf: Vec::new() };
    judged_writer
        .buf
        .extend_from_slice(&(judged.len() as u32).to_le_bytes());
    for (candidate, receipt) in judged {
        encode_candidate(&mut judged_writer, candidate)?;
        encode_receipt(&mut judged_writer, receipt)?;
    }
    let mut chart_writer = Writer { buf: Vec::new() };
    chart_writer
        .buf
        .extend_from_slice(&(chart_receipts.len() as u32).to_le_bytes());
    for (candidate, receipts) in chart_receipts {
        chart_writer.digest(candidate);
        chart_writer.digest(&receipts.marginal_gain_receipt_digest);
        chart_writer.digest(&receipts.numerical_receipt_digest);
    }
    let mut out = Vec::new();
    out.extend_from_slice(&VERDICTS_MAGIC);
    out.extend_from_slice(&VERDICTS_VERSION.to_le_bytes());
    out.extend_from_slice(&(prov_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&prov_bytes);
    out.extend_from_slice(&(trace_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&trace_bytes);
    out.extend_from_slice(&(judged_writer.buf.len() as u32).to_le_bytes());
    out.extend_from_slice(&judged_writer.buf);
    out.extend_from_slice(&(chart_writer.buf.len() as u32).to_le_bytes());
    out.extend_from_slice(&chart_writer.buf);
    out.extend_from_slice(&digest_bytes(eligibility_receipt_digest));
    out.extend_from_slice(&digest_bytes(judge_receipt_digest));
    let checksum = Sha256::digest(&out);
    out.extend_from_slice(&checksum);
    Ok(out)
}

pub fn decode_verdicts(bytes: &[u8]) -> Result<DecodedVerdicts, String> {
    if bytes.len() < 4 + 2 + 4 + 4 + 4 + 4 + 32 + 32 + 32 {
        return Err("verdicts artifact is truncated".into());
    }
    if bytes[..4] != VERDICTS_MAGIC {
        return Err("verdicts artifact magic mismatch".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != VERDICTS_VERSION {
        return Err(format!("unsupported verdicts version: {version}"));
    }
    let body_end = bytes.len() - 32;
    if Sha256::digest(&bytes[..body_end])[..] != bytes[body_end..] {
        return Err("verdicts artifact file digest mismatch".into());
    }
    let mut reader = Reader {
        bytes: &bytes[..body_end],
        pos: 6,
    };
    let prov_len = reader.u32_value("prov len")? as usize;
    let prov_raw = reader.take(prov_len, "provenance payload")?;
    let provenance = crate::trace::decode_provenance_payload(prov_raw)?;

    let trace_len = reader.u32_value("trace len")? as usize;
    let trace_raw = reader.take(trace_len, "trace stream")?;
    let envelopes = crate::trace::decode_trace_stream(trace_raw)?;

    let judged_section_len = reader.u32_value("judged section len")? as usize;
    let judged_raw = reader.take(judged_section_len, "judged bytes")?;

    let chart_section_len = reader.u32_value("chart section len")? as usize;
    let chart_raw = reader.take(chart_section_len, "chart bytes")?;

    let eligibility_receipt_digest = reader.digest("eligibility receipt")?;
    let judge_receipt_digest = reader.digest("judge receipt")?;
    reader.finish()?;

    let mut judged_reader = Reader {
        bytes: judged_raw,
        pos: 0,
    };
    let count = judged_reader.u32_value("judged count")? as usize;
    let mut judged = Vec::with_capacity(count);
    for _ in 0..count {
        let cand = decode_candidate(&mut judged_reader)?;
        let rec = decode_receipt(&mut judged_reader)?;
        judged.push((cand, rec));
    }
    judged_reader.finish()?;

    let mut chart_reader = Reader {
        bytes: chart_raw,
        pos: 0,
    };
    let chart_count = chart_reader.u32_value("chart count")? as usize;
    let mut chart_receipts = BTreeMap::new();
    for _ in 0..chart_count {
        let candidate = chart_reader.digest("chart candidate")?;
        let receipts = ChartReceipts {
            marginal_gain_receipt_digest: chart_reader.digest("marginal gain")?,
            numerical_receipt_digest: chart_reader.digest("numerical")?,
        };
        chart_receipts.insert(candidate, receipts);
    }
    chart_reader.finish()?;

    Ok(DecodedVerdicts {
        provenance,
        envelopes,
        judged,
        chart_receipts,
        eligibility_receipt_digest,
        judge_receipt_digest,
    })
}
