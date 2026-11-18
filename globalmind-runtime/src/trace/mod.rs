//! Lane 1 — native trace envelope.
//!
//! A fresh binary schema and codec for real model activation/weight traces.
//! No JSON, JSONL, Python, or text tensor transport crosses this boundary.
//! Every envelope binds exact model/config/tokenizer digests, tensor
//! shape/dtype, layer/head, token position, native payload bytes, and an
//! observer-nonperturbation receipt digest proving the trace tap did not
//! alter the forward pass it observed.
//!
//! Wire format (all integers little-endian):
//!   magic "GMTR" (4) | version u16 | header_len u32 | header | payload
//! The header is a fixed-layout binary record; `header_len` lets future
//! versions extend it without breaking this decoder, which rejects any
//! version or length it does not understand.

mod codec;
pub mod qwen36;

pub use codec::{decode, decode_frame, encode};
pub use qwen36::{
    DEFAULT_REAL_TOKEN_TAPE, PROVENANCE_MAGIC, PROVENANCE_SENTINEL_HEAD, PROVENANCE_SENTINEL_LAYER,
    PROVENANCE_VERSION, PositionAblationRow, PositionEncoderArm, ProducedTrace, TOKEN_TAPE_MAGIC,
    TOKEN_TAPE_VERSION, TraceProvenance, decode_provenance_payload, decode_token_tape,
    decode_trace_stream, default_tppe_config, encode_provenance_payload, encode_token_tape,
    encode_trace_stream, extract_provenance_from_stream, is_provenance_envelope, paired_nll_stats,
    produce_real_qwen36_trace, qualify_position_encoders, tppe_states_for_positions,
};

use crate::{Digest, RecordId};

pub const TRACE_MAGIC: [u8; 4] = *b"GMTR";
pub const TRACE_VERSION: u16 = 1;
/// magic(4) + version(2) + header_len(4)
pub const FRAME_PREFIX_LEN: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceDtype {
    F16,
    Bf16,
    F32,
    Q8_0,
    Q4_0,
}

impl TraceDtype {
    pub fn code(self) -> u8 {
        match self {
            Self::F16 => 1,
            Self::Bf16 => 2,
            Self::F32 => 3,
            Self::Q8_0 => 4,
            Self::Q4_0 => 5,
        }
    }

    pub fn from_code(code: u8) -> Result<Self, String> {
        match code {
            1 => Ok(Self::F16),
            2 => Ok(Self::Bf16),
            3 => Ok(Self::F32),
            4 => Ok(Self::Q8_0),
            5 => Ok(Self::Q4_0),
            _ => Err(format!("unknown trace dtype code: {code}")),
        }
    }

    /// Dense scalar byte width; quantized formats are block-packed and have
    /// no per-scalar width, so their payload length is receipt-checked only.
    pub fn scalar_bytes(self) -> Option<usize> {
        match self {
            Self::F16 | Self::Bf16 => Some(2),
            Self::F32 => Some(4),
            Self::Q8_0 | Self::Q4_0 => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceKind {
    Activation,
    Weight,
    KeyValue,
}

impl TraceKind {
    pub fn code(self) -> u8 {
        match self {
            Self::Activation => 1,
            Self::Weight => 2,
            Self::KeyValue => 3,
        }
    }

    pub fn from_code(code: u8) -> Result<Self, String> {
        match code {
            1 => Ok(Self::Activation),
            2 => Ok(Self::Weight),
            3 => Ok(Self::KeyValue),
            _ => Err(format!("unknown trace kind code: {code}")),
        }
    }
}

/// One immutable native trace record. The payload digest is always
/// recomputed over the payload bytes; a mismatch fails closed.
#[derive(Clone, Debug)]
pub struct TraceEnvelope {
    pub model_digest: Digest,
    pub config_digest: Digest,
    pub tokenizer_digest: Digest,
    pub kind: TraceKind,
    pub dtype: TraceDtype,
    pub shape: Vec<u32>,
    pub layer: u32,
    pub head: u32,
    pub token_position: u64,
    /// Digest of the observer-nonperturbation receipt: evidence the tap
    /// recorded without changing model outputs.
    pub observer_receipt_digest: Digest,
    pub payload: Box<[u8]>,
    pub payload_digest: Digest,
}

impl TraceEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        if self.shape.is_empty() || self.shape.len() > 8 {
            return Err("trace shape must have 1..=8 dimensions".into());
        }
        if self.shape.contains(&0) {
            return Err("trace shape dimensions must be positive".into());
        }
        if self.payload.is_empty() {
            return Err("trace payload must be non-empty".into());
        }
        if Digest::sha256(&self.payload) != self.payload_digest {
            return Err("trace payload digest mismatch".into());
        }
        if let Some(scalar_bytes) = self.dtype.scalar_bytes() {
            let elements = self.shape.iter().try_fold(1usize, |product, dimension| {
                product
                    .checked_mul(*dimension as usize)
                    .ok_or_else(|| "trace shape overflows usize".to_string())
            })?;
            let expected = elements
                .checked_mul(scalar_bytes)
                .ok_or_else(|| "trace byte length overflows usize".to_string())?;
            if expected != self.payload.len() {
                return Err("trace payload length does not match shape/dtype".into());
            }
        }
        Ok(())
    }

    /// Teacher-neutral content identity for this trace. Model, config, and
    /// tokenizer digests are provenance only and never enter this preimage,
    /// so identical tensor content from two teachers coalesces.
    pub fn content_digest(&self) -> Digest {
        let mut fields: Vec<String> = vec![
            self.kind.code().to_string(),
            self.dtype.code().to_string(),
            self.layer.to_string(),
            self.head.to_string(),
            self.token_position.to_string(),
        ];
        fields.extend(self.shape.iter().map(u32::to_string));
        fields.push(self.payload_digest.as_str().to_string());
        digest_fields(
            b"globalmind-trace-content-v1",
            &fields.iter().map(String::as_str).collect::<Vec<_>>(),
        )
    }
}

/// Envelope identity used for record ids in downstream lanes.
pub fn envelope_id(envelope: &TraceEnvelope) -> Result<RecordId, String> {
    RecordId::parse(format!(
        "trace_{}",
        &envelope.content_digest().as_str()[..32]
    ))
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    use sha2::{Digest as ShaDigest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}
