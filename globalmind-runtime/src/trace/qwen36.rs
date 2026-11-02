//! Frozen-weight Qwen3.6 Layer-3 trace producer and binary token-tape codec.
//!
//! Runs a custom single-Layer-3 reconstruction with the rostered weights
//! twice on the same immutable
//! binary token tape:
//! 1. Layer-3 control pass (no observer tap).
//! 2. Observer-enabled extraction pass.
//!
//! Fails closed unless control versus observed hidden states and 248,320-dim
//! `lm_head` logits match with `0.0` maximum absolute difference (exact
//! IEEE-754 bit parity). Emits native `GMTR` binary [`TraceEnvelope`] frames.

use super::{TraceDtype, TraceEnvelope, TraceKind, decode_frame, encode};
use crate::{
    Digest, RadialLaw, RecordId, TppeConfig, TppeState, VerifiedArtifact, VerifiedManifest,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub const TOKEN_TAPE_MAGIC: [u8; 4] = *b"GMTT";
pub const TOKEN_TAPE_VERSION: u16 = 1;
pub const PROVENANCE_MAGIC: [u8; 4] = *b"GMPV";
pub const PROVENANCE_VERSION: u16 = 1;
pub const PROVENANCE_SENTINEL_LAYER: u32 = u32::MAX;
pub const PROVENANCE_SENTINEL_HEAD: u32 = u32::MAX;
pub const TPPE_LAYER3_CANDIDATE_VERSION: &str = "tppe-layer3-candidate-v2";
const TPPE_LAYER3_CANDIDATE_CONTRACT: &str =
    include_str!("../../contracts/tppe-layer3-candidate-v2.contract");

fn contract_value<'a>(contract: &'a str, key: &str) -> Result<&'a str, String> {
    contract
        .lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(name, value)| (name == key).then_some(value))
        .ok_or_else(|| format!("TPPE candidate contract missing {key}"))
}

fn verify_candidate_source_contract() -> Result<(), String> {
    let trace_digest = Digest::sha256(include_str!("qwen36.rs").as_bytes());
    let tppe_digest = Digest::sha256(include_str!("../tppe.rs").as_bytes());
    if contract_value(TPPE_LAYER3_CANDIDATE_CONTRACT, "contract")? != TPPE_LAYER3_CANDIDATE_VERSION
        || contract_value(TPPE_LAYER3_CANDIDATE_CONTRACT, "trace_source_sha256")?
            != trace_digest.as_str()
        || contract_value(TPPE_LAYER3_CANDIDATE_CONTRACT, "tppe_source_sha256")?
            != tppe_digest.as_str()
    {
        return Err("TPPE candidate contract does not match current Rust sources".into());
    }
    Ok(())
}

fn verify_candidate_identity(verified: &VerifiedManifest) -> Result<(), String> {
    verify_candidate_source_contract()?;
    if verified.tpe_version != TPPE_LAYER3_CANDIDATE_VERSION
        || verified.tpe_configuration_digest
            != Digest::sha256(TPPE_LAYER3_CANDIDATE_CONTRACT.as_bytes())
    {
        return Err("manifest TPPE identity does not match Layer-3 candidate runtime".into());
    }
    Ok(())
}

/// Default reviewed Qwen3.6 token tape:
/// `[151644 (<|im_start|>), 872 (user), 151644 (<|im_start|>)]`.
/// Token 0 and Token 2 revisit token `151644`, so the pre-positional Layer-3
/// input-normed activation tensor is bit-for-bit identical at positions 0 and 2
/// and closes a genuine rank-2 trajectory loop `A -> B -> A`.
pub const DEFAULT_REAL_TOKEN_TAPE: [u32; 3] = [151644, 872, 151644];

const SHARD1_HEADER_LEN: u64 = 63_300;
const SHARD4_HEADER_LEN: u64 = 37_985;

const HIDDEN_SIZE: usize = 2048;
const Q_HEADS: usize = 16;
const KV_HEADS: usize = 2;
const HEAD_DIM: usize = 256;
const VOCAB_SIZE: usize = 248_320;
const MOE_EXPERTS: usize = 256;
const MOE_TOP_K: usize = 8;
const MOE_INTERMEDIATE: usize = 512;
const QUANT_GROUP: usize = 64;
const RMS_NORM_EPS: f32 = 1e-6;
const TARGET_FULL_ATTN_LAYER: u32 = 3;
const TAPPED_ACTIVATION_DIM: usize = 64;
const TAPPED_WEIGHT_BASIS_DIM: usize = 8;
const TAPPED_QUANTIZED_WEIGHT_BYTES: usize = 37;

#[derive(Clone, Debug)]
pub struct TraceProvenance {
    pub workspace: RecordId,
    pub method: String,
    pub configuration_digest: Digest,
    pub tpe_version: String,
    pub tpe_configuration_digest: Digest,
    pub provider: String,
    pub model_name: String,
    pub revision: String,
    pub modality: String,
    pub roster_digest: Digest,
    pub input_digest: Digest,
    pub run_id: RecordId,
    pub model_digest: Digest,
    pub config_digest: Digest,
    pub tokenizer_digest: Digest,
    pub runtime_digest: Digest,
    pub hardware_digest: Digest,
    pub token_tape_digest: Digest,
    pub control_logits_digest: Digest,
    pub observed_logits_digest: Digest,
    pub control_hidden_digest: Digest,
    pub observed_hidden_digest: Digest,
    pub observer_receipt_digest: Digest,
    pub max_logit_abs_diff: f32,
    pub max_hidden_abs_diff: f32,
    pub artifacts: Vec<VerifiedArtifact>,
}

#[derive(Clone, Debug)]
pub struct ProducedTrace {
    pub envelopes: Vec<TraceEnvelope>,
    pub stream_bytes: Vec<u8>,
    pub trace_digest: Digest,
    pub provenance: TraceProvenance,
}

/// Position-encoder arms installed at the same real Layer-3 Q/K boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionEncoderArm {
    NativeRope,
    AngleOnlyToroidal,
    TppeLinearLogV1,
    TppeLogV1,
}

impl PositionEncoderArm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeRope => "native_rope",
            Self::AngleOnlyToroidal => "angle_only_toroidal",
            Self::TppeLinearLogV1 => "tppe_linear_log_v1",
            Self::TppeLogV1 => "tppe_log_v1",
        }
    }

    pub fn encoder_state_bytes(self) -> usize {
        match self {
            Self::NativeRope => 0,
            Self::AngleOnlyToroidal => std::mem::size_of::<[f64; 2]>(),
            Self::TppeLinearLogV1 | Self::TppeLogV1 => std::mem::size_of::<TppeState>(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PositionAblationRow {
    pub arm: PositionEncoderArm,
    pub mean_next_token_nll: f64,
    pub top1_hits: u32,
    pub predictions: u32,
    pub mean_wall_us: f64,
    pub wall_stddev_us: f64,
    pub encoder_state_bytes: usize,
    pub mean_abs_logit_delta_from_rope: f64,
    pub max_abs_logit_delta_from_rope: f32,
    pub logits_digest: Digest,
    /// Paired per-token NLL difference against the native-RoPE arm on the
    /// same predictions: mean and 95% CI half-width. Zero-width for RoPE.
    pub paired_nll_delta_mean: f64,
    pub paired_nll_delta_ci95: f64,
    /// Per-token NLLs, retained so callers can run paired comparisons
    /// between non-reference arms (e.g. the two radial laws).
    pub per_token_nll: Vec<f64>,
}

/// Paired mean difference and 95% CI half-width between two per-token
/// NLL series measured on identical predictions.
pub fn paired_nll_stats(candidate: &[f64], reference: &[f64]) -> Result<(f64, f64), String> {
    if candidate.len() != reference.len() || candidate.len() < 2 {
        return Err("paired NLL comparison requires equal series of length >= 2".into());
    }
    let n = candidate.len() as f64;
    let diffs: Vec<f64> = candidate
        .iter()
        .zip(reference)
        .map(|(a, b)| a - b)
        .collect();
    let mean = diffs.iter().sum::<f64>() / n;
    let variance = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0);
    Ok((mean, 1.96 * (variance / n).sqrt()))
}

/// Encodes a sequence of token IDs into the bounded native `GMTT` binary tape format.
pub fn encode_token_tape(tokens: &[u32]) -> Result<Vec<u8>, String> {
    if tokens.is_empty() {
        return Err("token tape must contain at least one token".into());
    }
    for &tok in tokens {
        if (tok as usize) >= VOCAB_SIZE {
            return Err(format!(
                "token id {tok} exceeds Qwen3.6 vocab size {VOCAB_SIZE}"
            ));
        }
    }
    let mut payload = Vec::with_capacity(tokens.len() * 4);
    for &tok in tokens {
        payload.extend_from_slice(&tok.to_le_bytes());
    }
    let payload_digest = Digest::sha256(&payload);
    let mut out = Vec::with_capacity(4 + 2 + 4 + 32 + payload.len());
    out.extend_from_slice(&TOKEN_TAPE_MAGIC);
    out.extend_from_slice(&TOKEN_TAPE_VERSION.to_le_bytes());
    out.extend_from_slice(&(tokens.len() as u32).to_le_bytes());
    out.extend_from_slice(&digest_to_bytes(&payload_digest));
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Decodes and validates a `GMTT` binary token tape.
pub fn decode_token_tape(bytes: &[u8]) -> Result<(Vec<u32>, Digest), String> {
    if bytes.len() < 42 {
        return Err("token tape is truncated".into());
    }
    if bytes[..4] != TOKEN_TAPE_MAGIC {
        return Err("token tape magic mismatch (expected GMTT)".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != TOKEN_TAPE_VERSION {
        return Err(format!("unsupported token tape version: {version}"));
    }
    let count = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    if count == 0 {
        return Err("token tape count must be positive".into());
    }
    let declared_digest = digest_from_bytes(&bytes[10..42])?;
    let payload = &bytes[42..];
    if payload.len() != count * 4 {
        return Err("token tape byte length does not match token count".into());
    }
    let actual_digest = Digest::sha256(payload);
    if actual_digest != declared_digest {
        return Err("token tape payload digest mismatch".into());
    }
    let mut tokens = Vec::with_capacity(count);
    for chunk in payload.as_chunks::<4>().0 {
        let tok = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if (tok as usize) >= VOCAB_SIZE {
            return Err(format!(
                "token id {tok} exceeds Qwen3.6 vocab size {VOCAB_SIZE}"
            ));
        }
        tokens.push(tok);
    }
    Ok((tokens, Digest::sha256(bytes)))
}

/// Encodes a sequence of [`TraceEnvelope`] records as concatenated `GMTR` frames.
pub fn encode_trace_stream(envelopes: &[TraceEnvelope]) -> Result<Vec<u8>, String> {
    if envelopes.is_empty() {
        return Err("trace stream requires at least one envelope".into());
    }
    let mut out = Vec::new();
    for env in envelopes {
        let frame = encode(env)?;
        out.extend_from_slice(&frame);
    }
    Ok(out)
}

/// Decodes concatenated `GMTR` frames from a `.gmtrace` binary file.
pub fn decode_trace_stream(bytes: &[u8]) -> Result<Vec<TraceEnvelope>, String> {
    if bytes.is_empty() {
        return Err("trace stream is empty".into());
    }
    let mut offset = 0usize;
    let mut envelopes = Vec::new();
    while offset < bytes.len() {
        let (env, consumed) = decode_frame(&bytes[offset..])?;
        envelopes.push(env);
        offset += consumed;
    }
    if envelopes.is_empty() {
        return Err("trace stream decoded zero frames".into());
    }
    Ok(envelopes)
}

pub fn is_provenance_envelope(envelope: &TraceEnvelope) -> bool {
    envelope.kind == TraceKind::KeyValue
        && envelope.dtype == TraceDtype::Q8_0
        && envelope.layer == PROVENANCE_SENTINEL_LAYER
        && envelope.head == PROVENANCE_SENTINEL_HEAD
}

pub fn extract_provenance_from_stream(
    envelopes: &[TraceEnvelope],
) -> Result<TraceProvenance, String> {
    let env = envelopes
        .iter()
        .find(|e| is_provenance_envelope(e))
        .ok_or_else(|| "trace stream is missing its binary provenance envelope".to_string())?;
    decode_provenance_payload(&env.payload)
}

/// Runs the real frozen Qwen3.6 model twice (control vs. observer-enabled) over
/// `tokens` and emits verified [`TraceEnvelope`] frames.
pub fn produce_real_qwen36_trace(
    verified: &VerifiedManifest,
    tokens: &[u32],
    token_tape_digest: &Digest,
) -> Result<ProducedTrace, String> {
    verify_candidate_identity(verified)?;
    if tokens.is_empty() {
        return Err("trace requires at least one token".into());
    }
    let model = verified
        .models
        .first()
        .ok_or_else(|| "verified manifest has no model".to_string())?;

    let shard1_art = find_artifact(&model.artifacts, "weights-00001")?;
    let shard4_art = find_artifact(&model.artifacts, "weights-00004")?;
    let config_art = find_artifact(&model.artifacts, "model-config")?;
    let tokenizer_art = find_artifact(&model.artifacts, "tokenizer")?;

    let mut weights = Qwen36Layer3Weights::load(&shard1_art.path, &shard4_art.path)?;
    let tppe_cfg = default_tppe_config(RadialLaw::LinearLogV1);

    // Pass 1: Layer-3 control pass (no observer tap).
    let control = weights.run_sequence_forward(
        tokens,
        PositionEncoderArm::TppeLinearLogV1,
        &tppe_cfg,
        false,
    )?;

    // Pass 2: Observer-enabled extraction pass on the identical frozen weights and tape.
    let observed = weights.run_sequence_forward(
        tokens,
        PositionEncoderArm::TppeLinearLogV1,
        &tppe_cfg,
        true,
    )?;

    if control.logits_by_pos.len() != observed.logits_by_pos.len()
        || control.hidden_by_pos.len() != observed.hidden_by_pos.len()
    {
        return Err("control vs observed sequence length mismatch".into());
    }

    let mut max_logit_abs_diff = 0.0f32;
    for (c_logits, o_logits) in control.logits_by_pos.iter().zip(&observed.logits_by_pos) {
        for (&c, &o) in c_logits.iter().zip(o_logits) {
            let diff = (c - o).abs();
            if !diff.is_finite() || c.to_bits() != o.to_bits() {
                return Err("observer perturbed forward-pass logits".into());
            }
            if diff > max_logit_abs_diff {
                max_logit_abs_diff = diff;
            }
        }
    }

    let mut max_hidden_abs_diff = 0.0f32;
    for (c_hid, o_hid) in control.hidden_by_pos.iter().zip(&observed.hidden_by_pos) {
        for (&c, &o) in c_hid.iter().zip(o_hid) {
            let diff = (c - o).abs();
            if !diff.is_finite() || c.to_bits() != o.to_bits() {
                return Err("observer perturbed forward-pass hidden state".into());
            }
            if diff > max_hidden_abs_diff {
                max_hidden_abs_diff = diff;
            }
        }
    }

    let control_logits_digest = hash_f32_slices(&control.logits_by_pos);
    let observed_logits_digest = hash_f32_slices(&observed.logits_by_pos);
    let control_hidden_digest = hash_f32_slices(&control.hidden_by_pos);
    let observed_hidden_digest = hash_f32_slices(&observed.hidden_by_pos);

    if control_logits_digest != observed_logits_digest
        || control_hidden_digest != observed_hidden_digest
    {
        return Err("observer nonperturbation digest parity check failed".into());
    }

    let runtime_digest = digest_fields(
        b"globalmind-runtime-substrate-v1",
        &[
            crate::PROJECTOR_VERSION,
            verified.tpe_version.as_str(),
            verified.tpe_configuration_digest.as_str(),
            std::env::consts::OS,
            std::env::consts::ARCH,
        ],
    );
    let hardware_digest = digest_fields(
        b"globalmind-hardware-substrate-v1",
        &[
            std::env::consts::OS,
            std::env::consts::ARCH,
            "ieee754-little-endian-f32-bf16-oq4",
        ],
    );

    let observer_receipt_digest = digest_fields(
        b"globalmind-observer-nonperturbation-v1",
        &[
            model.roster_digest.as_str(),
            config_art.sha256.as_str(),
            tokenizer_art.sha256.as_str(),
            runtime_digest.as_str(),
            hardware_digest.as_str(),
            token_tape_digest.as_str(),
            control_logits_digest.as_str(),
            observed_logits_digest.as_str(),
            control_hidden_digest.as_str(),
            observed_hidden_digest.as_str(),
            &format!("{max_logit_abs_diff:?}"),
            &format!("{max_hidden_abs_diff:?}"),
        ],
    );

    let mut envelopes = Vec::new();

    // 1. Per-token real Layer-3 input-normed activation slices (head 0, 64 f32s).
    for (pos, slice) in observed.tapped_input_normed_by_pos.iter().enumerate() {
        let payload = f32_slice_to_le_bytes(slice);
        envelopes.push(TraceEnvelope {
            model_digest: model.roster_digest.clone(),
            config_digest: config_art.sha256.clone(),
            tokenizer_digest: tokenizer_art.sha256.clone(),
            kind: TraceKind::Activation,
            dtype: TraceDtype::F32,
            shape: vec![slice.len() as u32],
            layer: TARGET_FULL_ATTN_LAYER,
            head: 0,
            token_position: pos as u64,
            observer_receipt_digest: observer_receipt_digest.clone(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        });
    }

    // 2. Real Layer-3 q_norm weight tangent basis (rank 8, 8 f32s).
    {
        let payload = f32_slice_to_le_bytes(&observed.tapped_weight_basis);
        envelopes.push(TraceEnvelope {
            model_digest: model.roster_digest.clone(),
            config_digest: config_art.sha256.clone(),
            tokenizer_digest: tokenizer_art.sha256.clone(),
            kind: TraceKind::Weight,
            dtype: TraceDtype::F32,
            shape: vec![observed.tapped_weight_basis.len() as u32],
            layer: TARGET_FULL_ATTN_LAYER,
            head: 0,
            token_position: tokens.len() as u64,
            observer_receipt_digest: observer_receipt_digest.clone(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        });
    }

    // 3. Real Layer-3 raw quantized Q8 weight slice from mlp_gate (37 bytes, Q8_0),
    // which extracts a Rank-0 weight atom and a Rank-8 chart candidate that the
    // independent judge rejects because quantized tangent bases have no dense F32
    // chart statistic.
    {
        let payload = observed.tapped_quantized_weight_bytes.clone();
        envelopes.push(TraceEnvelope {
            model_digest: model.roster_digest.clone(),
            config_digest: config_art.sha256.clone(),
            tokenizer_digest: tokenizer_art.sha256.clone(),
            kind: TraceKind::Weight,
            dtype: TraceDtype::Q8_0,
            shape: vec![payload.len() as u32],
            layer: TARGET_FULL_ATTN_LAYER,
            head: 0,
            token_position: (tokens.len() + 1) as u64,
            observer_receipt_digest: observer_receipt_digest.clone(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        });
    }

    // 4. Real Layer-3 KV head 0 key cache across the sequence (TraceKind::KeyValue,
    // which semantic extraction must skip).
    {
        let payload = f32_slice_to_le_bytes(&observed.k_cache_head0);
        envelopes.push(TraceEnvelope {
            model_digest: model.roster_digest.clone(),
            config_digest: config_art.sha256.clone(),
            tokenizer_digest: tokenizer_art.sha256.clone(),
            kind: TraceKind::KeyValue,
            dtype: TraceDtype::F32,
            shape: vec![tokens.len() as u32, HEAD_DIM as u32],
            layer: TARGET_FULL_ATTN_LAYER,
            head: 0,
            token_position: 0,
            observer_receipt_digest: observer_receipt_digest.clone(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        });
    }

    let provenance = TraceProvenance {
        workspace: verified.workspace.clone(),
        method: verified.method.clone(),
        configuration_digest: verified.configuration_digest.clone(),
        tpe_version: verified.tpe_version.clone(),
        tpe_configuration_digest: verified.tpe_configuration_digest.clone(),
        provider: model.provider.clone(),
        model_name: model.name.clone(),
        revision: model.revision.clone(),
        modality: model.modality.clone(),
        roster_digest: model.roster_digest.clone(),
        input_digest: verified.input_digest.clone(),
        run_id: verified.run_id.clone(),
        model_digest: model.roster_digest.clone(),
        config_digest: config_art.sha256.clone(),
        tokenizer_digest: tokenizer_art.sha256.clone(),
        runtime_digest,
        hardware_digest,
        token_tape_digest: token_tape_digest.clone(),
        control_logits_digest,
        observed_logits_digest,
        control_hidden_digest,
        observed_hidden_digest,
        observer_receipt_digest: observer_receipt_digest.clone(),
        max_logit_abs_diff,
        max_hidden_abs_diff,
        artifacts: model.artifacts.clone(),
    };

    // 5. Binary provenance envelope (TraceKind::KeyValue + Q8_0 sentinel so semantic
    // extraction skips it while downstream binary stages recover full provenance).
    let prov_payload = encode_provenance_payload(&provenance);
    envelopes.push(TraceEnvelope {
        model_digest: model.roster_digest.clone(),
        config_digest: config_art.sha256.clone(),
        tokenizer_digest: tokenizer_art.sha256.clone(),
        kind: TraceKind::KeyValue,
        dtype: TraceDtype::Q8_0,
        shape: vec![prov_payload.len() as u32],
        layer: PROVENANCE_SENTINEL_LAYER,
        head: PROVENANCE_SENTINEL_HEAD,
        token_position: 0,
        observer_receipt_digest,
        payload_digest: Digest::sha256(&prov_payload),
        payload: prov_payload.into_boxed_slice(),
    });

    let stream_bytes = encode_trace_stream(&envelopes)?;
    let trace_digest = Digest::sha256(&stream_bytes);

    Ok(ProducedTrace {
        envelopes,
        stream_bytes,
        trace_digest,
        provenance,
    })
}

/// Matched position-encoder bakeoff on the real frozen Qwen execution path.
/// Every arm runs the same weights/tokens and differs only at the Q/K
/// positional boundary. Timings exclude artifact rehash and weight loading;
/// arm order rotates across repeats to reduce order/thermal bias.
pub fn qualify_position_encoders(
    verified: &VerifiedManifest,
    tokens: &[u32],
    repeats: usize,
) -> Result<Vec<PositionAblationRow>, String> {
    verify_candidate_identity(verified)?;
    if tokens.len() < 2 {
        return Err("position ablation requires at least two tokens".into());
    }
    if !(1..=20).contains(&repeats) {
        return Err("position ablation repeats must be in 1..=20".into());
    }
    let model = verified
        .models
        .first()
        .ok_or_else(|| "verified manifest has no model".to_string())?;
    let shard1 = find_artifact(&model.artifacts, "weights-00001")?;
    let shard4 = find_artifact(&model.artifacts, "weights-00004")?;
    let mut weights = Qwen36Layer3Weights::load(&shard1.path, &shard4.path)?;
    let arms = [
        PositionEncoderArm::NativeRope,
        PositionEncoderArm::AngleOnlyToroidal,
        PositionEncoderArm::TppeLinearLogV1,
        PositionEncoderArm::TppeLogV1,
    ];
    let config_for = |arm| match arm {
        PositionEncoderArm::TppeLogV1 => default_tppe_config(RadialLaw::LogV1),
        _ => default_tppe_config(RadialLaw::LinearLogV1),
    };

    // Untimed native warmup and independent reference for logit deltas.
    let reference = weights.run_sequence_forward(
        tokens,
        PositionEncoderArm::NativeRope,
        &config_for(PositionEncoderArm::NativeRope),
        false,
    )?;
    let mut wall_samples: [Vec<f64>; 4] = std::array::from_fn(|_| Vec::new());
    let mut first_outputs: [Option<SequencePassOutput>; 4] = std::array::from_fn(|_| None);

    for repeat in 0..repeats {
        for offset in 0..arms.len() {
            let index = (offset + repeat) % arms.len();
            let arm = arms[index];
            let started = std::time::Instant::now();
            let output = weights.run_sequence_forward(tokens, arm, &config_for(arm), false)?;
            wall_samples[index].push(started.elapsed().as_secs_f64() * 1_000_000.0);
            if let Some(first) = &first_outputs[index] {
                if hash_f32_slices(&first.logits_by_pos) != hash_f32_slices(&output.logits_by_pos) {
                    return Err(format!("{} logits drifted across repeats", arm.as_str()));
                }
            } else {
                first_outputs[index] = Some(output);
            }
        }
    }

    let (_, _, reference_predictions) = next_token_quality(&reference.logits_by_pos, tokens)?;
    let reference_per_token = per_token_nll(&reference.logits_by_pos, tokens)?;
    let mut rows = Vec::new();
    for (index, arm) in arms.into_iter().enumerate() {
        let output = first_outputs[index]
            .take()
            .ok_or_else(|| format!("{} produced no output", arm.as_str()))?;
        let (mean_nll, top1_hits, predictions) = next_token_quality(&output.logits_by_pos, tokens)?;
        if predictions != reference_predictions {
            return Err("arm prediction count differs from the RoPE reference".into());
        }
        let per_token = per_token_nll(&output.logits_by_pos, tokens)?;
        let (paired_mean, paired_ci95) = paired_nll_stats(&per_token, &reference_per_token)?;
        let (mean_delta, max_delta) = logit_delta(&reference.logits_by_pos, &output.logits_by_pos)?;
        let mean_wall_us = wall_samples[index].iter().sum::<f64>() / repeats as f64;
        let wall_variance = wall_samples[index]
            .iter()
            .map(|sample| (sample - mean_wall_us).powi(2))
            .sum::<f64>()
            / repeats as f64;
        rows.push(PositionAblationRow {
            arm,
            mean_next_token_nll: mean_nll,
            top1_hits,
            predictions,
            mean_wall_us,
            wall_stddev_us: wall_variance.sqrt(),
            encoder_state_bytes: arm.encoder_state_bytes(),
            mean_abs_logit_delta_from_rope: mean_delta,
            max_abs_logit_delta_from_rope: max_delta,
            logits_digest: hash_f32_slices(&output.logits_by_pos),
            paired_nll_delta_mean: paired_mean,
            paired_nll_delta_ci95: paired_ci95,
            per_token_nll: per_token,
        });
    }
    Ok(rows)
}

fn per_token_nll(logits_by_pos: &[Vec<f32>], tokens: &[u32]) -> Result<Vec<f64>, String> {
    if logits_by_pos.len() != tokens.len() || tokens.len() < 2 {
        return Err("next-token quality input length mismatch".into());
    }
    let mut series = Vec::with_capacity(tokens.len() - 1);
    for position in 0..tokens.len() - 1 {
        let logits = &logits_by_pos[position];
        let target = tokens[position + 1] as usize;
        if logits.is_empty() || target >= logits.len() || !logits.iter().all(|v| v.is_finite()) {
            return Err("invalid logits or next-token target".into());
        }
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let sum_exp = logits
            .iter()
            .map(|value| ((*value - max) as f64).exp())
            .sum::<f64>();
        series.push(sum_exp.ln() + max as f64 - logits[target] as f64);
    }
    Ok(series)
}

fn next_token_quality(
    logits_by_pos: &[Vec<f32>],
    tokens: &[u32],
) -> Result<(f64, u32, u32), String> {
    let series = per_token_nll(logits_by_pos, tokens)?;
    let mut hits = 0u32;
    for position in 0..tokens.len() - 1 {
        let logits = &logits_by_pos[position];
        let target = tokens[position + 1] as usize;
        let argmax = logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| index)
            .ok_or_else(|| "empty logits".to_string())?;
        hits += u32::from(argmax == target);
    }
    let predictions = series.len() as u32;
    Ok((
        series.iter().sum::<f64>() / predictions as f64,
        hits,
        predictions,
    ))
}

fn logit_delta(reference: &[Vec<f32>], candidate: &[Vec<f32>]) -> Result<(f64, f32), String> {
    if reference.len() != candidate.len() || reference.is_empty() {
        return Err("logit delta sequence length mismatch".into());
    }
    let mut sum = 0.0f64;
    let mut count = 0u64;
    let mut max = 0.0f32;
    for (left, right) in reference.iter().zip(candidate) {
        if left.len() != right.len() {
            return Err("logit delta vocabulary width mismatch".into());
        }
        for (a, b) in left.iter().zip(right) {
            let delta = (*a - *b).abs();
            if !delta.is_finite() {
                return Err("logit delta is non-finite".into());
            }
            sum += delta as f64;
            count += 1;
            max = max.max(delta);
        }
    }
    if count == 0 {
        return Err("logit delta compared zero values".into());
    }
    Ok((sum / count as f64, max))
}

pub fn default_tppe_config(law: RadialLaw) -> TppeConfig {
    // Both candidates start at u=0.05 and have the same radial endpoint
    // after one derived revolution. Their intermediate trajectories differ.
    let radial_span = match law {
        RadialLaw::LinearLogV1 => 0.12,
        RadialLaw::LogV1 => {
            let log_at_one = 2.0f64.ln();
            0.12 * (0.35 + 0.65 * log_at_one) / log_at_one
        }
    };
    TppeConfig {
        law,
        radial_start: 0.05,
        radial_span,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }
}

pub fn encode_provenance_payload(prov: &TraceProvenance) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&PROVENANCE_MAGIC);
    out.extend_from_slice(&PROVENANCE_VERSION.to_le_bytes());
    write_str(&mut out, prov.workspace.as_str());
    write_str(&mut out, &prov.method);
    out.extend_from_slice(&digest_to_bytes(&prov.configuration_digest));
    write_str(&mut out, &prov.tpe_version);
    out.extend_from_slice(&digest_to_bytes(&prov.tpe_configuration_digest));
    write_str(&mut out, &prov.provider);
    write_str(&mut out, &prov.model_name);
    write_str(&mut out, &prov.revision);
    write_str(&mut out, &prov.modality);
    out.extend_from_slice(&digest_to_bytes(&prov.roster_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.input_digest));
    write_str(&mut out, prov.run_id.as_str());
    out.extend_from_slice(&digest_to_bytes(&prov.model_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.config_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.tokenizer_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.runtime_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.hardware_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.token_tape_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.control_logits_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.observed_logits_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.control_hidden_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.observed_hidden_digest));
    out.extend_from_slice(&digest_to_bytes(&prov.observer_receipt_digest));
    out.extend_from_slice(&prov.max_logit_abs_diff.to_le_bytes());
    out.extend_from_slice(&prov.max_hidden_abs_diff.to_le_bytes());
    out.extend_from_slice(&(prov.artifacts.len() as u32).to_le_bytes());
    for art in &prov.artifacts {
        write_str(&mut out, &art.role);
        write_str(&mut out, &art.path);
        out.extend_from_slice(&digest_to_bytes(&art.sha256));
        out.extend_from_slice(&art.bytes.to_le_bytes());
    }
    out
}

pub fn decode_provenance_payload(bytes: &[u8]) -> Result<TraceProvenance, String> {
    let mut cur = BinCursor { bytes, offset: 0 };
    if cur.take(4)? != PROVENANCE_MAGIC {
        return Err("provenance payload magic mismatch".into());
    }
    let version = cur.take_u16()?;
    if version != PROVENANCE_VERSION {
        return Err(format!("unsupported provenance version: {version}"));
    }
    let workspace = RecordId::parse(cur.take_str()?)?;
    let method = cur.take_str()?;
    let configuration_digest = cur.take_digest()?;
    let tpe_version = cur.take_str()?;
    let tpe_configuration_digest = cur.take_digest()?;
    let provider = cur.take_str()?;
    let model_name = cur.take_str()?;
    let revision = cur.take_str()?;
    let modality = cur.take_str()?;
    let roster_digest = cur.take_digest()?;
    let input_digest = cur.take_digest()?;
    let run_id = RecordId::parse(cur.take_str()?)?;
    let model_digest = cur.take_digest()?;
    let config_digest = cur.take_digest()?;
    let tokenizer_digest = cur.take_digest()?;
    let runtime_digest = cur.take_digest()?;
    let hardware_digest = cur.take_digest()?;
    let token_tape_digest = cur.take_digest()?;
    let control_logits_digest = cur.take_digest()?;
    let observed_logits_digest = cur.take_digest()?;
    let control_hidden_digest = cur.take_digest()?;
    let observed_hidden_digest = cur.take_digest()?;
    let observer_receipt_digest = cur.take_digest()?;
    let max_logit_abs_diff = cur.take_f32()?;
    let max_hidden_abs_diff = cur.take_f32()?;
    let art_count = cur.take_u32()? as usize;
    let mut artifacts = Vec::with_capacity(art_count);
    for _ in 0..art_count {
        let role = cur.take_str()?;
        let path = cur.take_str()?;
        let sha256 = cur.take_digest()?;
        let byte_count = cur.take_u64()?;
        artifacts.push(VerifiedArtifact {
            role,
            path,
            sha256,
            bytes: byte_count,
        });
    }
    if cur.offset != bytes.len() {
        return Err("provenance payload has trailing bytes".into());
    }
    Ok(TraceProvenance {
        workspace,
        method,
        configuration_digest,
        tpe_version,
        tpe_configuration_digest,
        provider,
        model_name,
        revision,
        modality,
        roster_digest,
        input_digest,
        run_id,
        model_digest,
        config_digest,
        tokenizer_digest,
        runtime_digest,
        hardware_digest,
        token_tape_digest,
        control_logits_digest,
        observed_logits_digest,
        control_hidden_digest,
        observed_hidden_digest,
        observer_receipt_digest,
        max_logit_abs_diff,
        max_hidden_abs_diff,
        artifacts,
    })
}

fn find_artifact<'a>(
    artifacts: &'a [VerifiedArtifact],
    role: &str,
) -> Result<&'a VerifiedArtifact, String> {
    artifacts
        .iter()
        .find(|a| a.role == role)
        .ok_or_else(|| format!("manifest is missing required artifact role: {role}"))
}

#[derive(Clone, Copy, Debug)]
struct TensorSliceSpec {
    start: u64,
    end: u64,
}

impl TensorSliceSpec {
    const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    const fn byte_len(self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Clone, Copy, Debug)]
struct AffineQuantSpec {
    weight: TensorSliceSpec,
    scales: TensorSliceSpec,
    biases: TensorSliceSpec,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

const EMBED_TOKENS_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(4_590_448_704, 4_844_728_384),
    scales: TensorSliceSpec::new(4_844_728_384, 4_860_620_864),
    biases: TensorSliceSpec::new(3_780_340_096, 3_796_232_576),
    out_dim: VOCAB_SIZE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_INPUT_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(2_275_097_664, 2_275_101_760);

const L3_Q_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_363_184_960, 3_371_573_568),
    scales: TensorSliceSpec::new(4_414_260_288, 4_414_784_576),
    biases: TensorSliceSpec::new(3_008_045_312, 3_008_569_600),
    out_dim: 8192,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_Q_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(4_419_537_984, 4_419_538_496);

const L3_K_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_006_963_968, 3_007_488_256),
    scales: TensorSliceSpec::new(4_274_951_104, 4_274_983_872),
    biases: TensorSliceSpec::new(3_985_148_864, 3_985_181_632),
    out_dim: 512,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_K_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(2_678_836_608, 2_678_837_120);

const L3_V_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(979_633_728, 980_158_016),
    scales: TensorSliceSpec::new(1_598_257_536, 1_598_290_304),
    biases: TensorSliceSpec::new(3_964_078_656, 3_964_111_424),
    out_dim: 512,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_O_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(3_980_921_792, 3_985_116_096),
    scales: TensorSliceSpec::new(4_413_998_144, 4_414_260_288),
    biases: TensorSliceSpec::new(3_006_699_776, 3_006_961_920),
    out_dim: HIDDEN_SIZE,
    in_dim: 4096,
    bits: 4,
};

const L3_POST_ATTN_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(3_964_070_464, 3_964_074_560);

const L3_MLP_GATE_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(4_869_042_240, 4_869_566_528),
    scales: TensorSliceSpec::new(3_006_683_392, 3_006_699_776),
    biases: TensorSliceSpec::new(3_002_398_976, 3_002_415_360),
    out_dim: MOE_EXPERTS,
    in_dim: HIDDEN_SIZE,
    bits: 8,
};

const L3_SHARED_GATE_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_841_145_536, 2_841_669_824),
    scales: TensorSliceSpec::new(3_006_650_624, 3_006_683_392),
    biases: TensorSliceSpec::new(1_951_781_504, 1_951_814_272),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SHARED_UP_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(1_437_052_480, 1_437_576_768),
    scales: TensorSliceSpec::new(3_998_028_736, 3_998_061_504),
    biases: TensorSliceSpec::new(2_835_243_072, 2_835_275_840),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SHARED_DOWN_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(5_332_855_040, 5_333_379_328),
    scales: TensorSliceSpec::new(2_840_031_296, 2_840_064_064),
    biases: TensorSliceSpec::new(2_835_804_224, 2_835_836_992),
    out_dim: HIDDEN_SIZE,
    in_dim: MOE_INTERMEDIATE,
    bits: 4,
};

const L3_SHARED_EXPERT_GATE_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_834_651_200, 2_834_653_248),
    scales: TensorSliceSpec::new(2_834_356_224, 2_834_356_288),
    biases: TensorSliceSpec::new(1_951_847_040, 1_951_847_104),
    out_dim: 1,
    in_dim: HIDDEN_SIZE,
    bits: 8,
};

const L3_SWITCH_GATE_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_867_902_720, 3_002_120_448),
    scales: TensorSliceSpec::new(3_047_731_456, 3_056_120_064),
    biases: TensorSliceSpec::new(2_859_514_112, 2_867_902_720),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SWITCH_UP_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(5_197_846_784, 5_332_064_512),
    scales: TensorSliceSpec::new(1_952_111_296, 1_960_499_904),
    biases: TensorSliceSpec::new(4_886_874_176, 4_895_262_784),
    out_dim: MOE_INTERMEDIATE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

const L3_SWITCH_DOWN_PROJ_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(2_275_626_112, 2_409_843_840),
    scales: TensorSliceSpec::new(2_841_669_824, 2_850_058_432),
    biases: TensorSliceSpec::new(4_573_139_008, 4_581_527_616),
    out_dim: HIDDEN_SIZE,
    in_dim: MOE_INTERMEDIATE,
    bits: 4,
};

const FINAL_NORM_SPEC: TensorSliceSpec = TensorSliceSpec::new(270_172_160, 270_176_256);

const LM_HEAD_SPEC: AffineQuantSpec = AffineQuantSpec {
    weight: TensorSliceSpec::new(15_892_480, 270_172_160),
    scales: TensorSliceSpec::new(433_565_824, 449_458_304),
    biases: TensorSliceSpec::new(0, 15_892_480),
    out_dim: VOCAB_SIZE,
    in_dim: HIDDEN_SIZE,
    bits: 4,
};

struct LoadedAffineMatrix {
    weight_words: Vec<u32>,
    scales_bf16: Vec<u16>,
    biases_bf16: Vec<u16>,
    out_dim: usize,
    in_dim: usize,
    bits: usize,
}

impl LoadedAffineMatrix {
    fn read_from(file: &mut File, data_base: u64, spec: AffineQuantSpec) -> Result<Self, String> {
        let weight_words = read_u32_vec(file, data_base, spec.weight)?;
        let scales_bf16 = read_u16_vec(file, data_base, spec.scales)?;
        let biases_bf16 = read_u16_vec(file, data_base, spec.biases)?;
        let words_per_row = (spec.in_dim * spec.bits) / 32;
        let groups_per_row = spec.in_dim / QUANT_GROUP;
        if weight_words.len() != spec.out_dim * words_per_row {
            return Err(format!(
                "packed weight word count mismatch: got {}, expected {}",
                weight_words.len(),
                spec.out_dim * words_per_row
            ));
        }
        if scales_bf16.len() != spec.out_dim * groups_per_row
            || biases_bf16.len() != spec.out_dim * groups_per_row
        {
            return Err("scales/biases length mismatch".into());
        }
        Ok(Self {
            weight_words,
            scales_bf16,
            biases_bf16,
            out_dim: spec.out_dim,
            in_dim: spec.in_dim,
            bits: spec.bits,
        })
    }

    fn read_expert_from(
        file: &mut File,
        data_base: u64,
        spec: AffineQuantSpec,
        expert_idx: usize,
    ) -> Result<Self, String> {
        let words_per_expert = spec.out_dim * ((spec.in_dim * spec.bits) / 32);
        let groups_per_expert = spec.out_dim * (spec.in_dim / QUANT_GROUP);
        let weight_byte_len = (words_per_expert * 4) as u64;
        let sb_byte_len = (groups_per_expert * 2) as u64;
        let w_start = spec.weight.start + (expert_idx as u64) * weight_byte_len;
        let s_start = spec.scales.start + (expert_idx as u64) * sb_byte_len;
        let b_start = spec.biases.start + (expert_idx as u64) * sb_byte_len;
        let expert_spec = AffineQuantSpec {
            weight: TensorSliceSpec::new(w_start, w_start + weight_byte_len),
            scales: TensorSliceSpec::new(s_start, s_start + sb_byte_len),
            biases: TensorSliceSpec::new(b_start, b_start + sb_byte_len),
            out_dim: spec.out_dim,
            in_dim: spec.in_dim,
            bits: spec.bits,
        };
        Self::read_from(file, data_base, expert_spec)
    }

    fn dequantize_row(&self, row: usize) -> Vec<f32> {
        let words_per_row = (self.in_dim * self.bits) / 32;
        let groups_per_row = self.in_dim / QUANT_GROUP;
        let mask = (1u32 << self.bits) - 1;
        let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
        let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
        let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
        let mut out = vec![0.0f32; self.in_dim];
        for (col, slot) in out.iter_mut().enumerate() {
            let bit = col * self.bits;
            let word = w_row[bit / 32];
            let shift = bit % 32;
            let q = ((word >> shift) & mask) as f32;
            let g = col / QUANT_GROUP;
            let scale = bf16_to_f32(s_row[g]);
            let bias = bf16_to_f32(b_row[g]);
            *slot = scale * q + bias;
        }
        out
    }

    fn project_vec(&self, x: &[f32]) -> Vec<f32> {
        assert_eq!(x.len(), self.in_dim);
        let words_per_row = (self.in_dim * self.bits) / 32;
        let groups_per_row = self.in_dim / QUANT_GROUP;
        let mut out = vec![0.0f32; self.out_dim];
        if self.bits == 4 {
            for (row, out_slot) in out.iter_mut().enumerate() {
                let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
                let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let mut sum = 0.0f32;
                for g in 0..groups_per_row {
                    let scale = bf16_to_f32(s_row[g]);
                    let bias = bf16_to_f32(b_row[g]);
                    let w_group = &w_row[g * 8..g * 8 + 8];
                    let x_group = &x[g * 64..g * 64 + 64];
                    for w_idx in 0..8 {
                        let word = w_group[w_idx];
                        let xb = &x_group[w_idx * 8..w_idx * 8 + 8];
                        sum += (scale * ((word & 0xF) as f32) + bias) * xb[0];
                        sum += (scale * (((word >> 4) & 0xF) as f32) + bias) * xb[1];
                        sum += (scale * (((word >> 8) & 0xF) as f32) + bias) * xb[2];
                        sum += (scale * (((word >> 12) & 0xF) as f32) + bias) * xb[3];
                        sum += (scale * (((word >> 16) & 0xF) as f32) + bias) * xb[4];
                        sum += (scale * (((word >> 20) & 0xF) as f32) + bias) * xb[5];
                        sum += (scale * (((word >> 24) & 0xF) as f32) + bias) * xb[6];
                        sum += (scale * (((word >> 28) & 0xF) as f32) + bias) * xb[7];
                    }
                }
                *out_slot = sum;
            }
        } else if self.bits == 8 {
            for (row, out_slot) in out.iter_mut().enumerate() {
                let w_row = &self.weight_words[row * words_per_row..(row + 1) * words_per_row];
                let s_row = &self.scales_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let b_row = &self.biases_bf16[row * groups_per_row..(row + 1) * groups_per_row];
                let mut sum = 0.0f32;
                for g in 0..groups_per_row {
                    let scale = bf16_to_f32(s_row[g]);
                    let bias = bf16_to_f32(b_row[g]);
                    let w_group = &w_row[g * 16..g * 16 + 16];
                    let x_group = &x[g * 64..g * 64 + 64];
                    for w_idx in 0..16 {
                        let word = w_group[w_idx];
                        let xb = &x_group[w_idx * 4..w_idx * 4 + 4];
                        sum += (scale * ((word & 0xFF) as f32) + bias) * xb[0];
                        sum += (scale * (((word >> 8) & 0xFF) as f32) + bias) * xb[1];
                        sum += (scale * (((word >> 16) & 0xFF) as f32) + bias) * xb[2];
                        sum += (scale * (((word >> 24) & 0xFF) as f32) + bias) * xb[3];
                    }
                }
                *out_slot = sum;
            }
        }
        out
    }
}

struct SequencePassOutput {
    hidden_by_pos: Vec<Vec<f32>>,
    logits_by_pos: Vec<Vec<f32>>,
    tapped_input_normed_by_pos: Vec<Vec<f32>>,
    tapped_weight_basis: Vec<f32>,
    tapped_quantized_weight_bytes: Vec<u8>,
    k_cache_head0: Vec<f32>,
}

struct Qwen36Layer3Weights {
    shard1_file: File,
    shard1_data_base: u64,
    input_norm: Vec<f32>,
    q_proj: LoadedAffineMatrix,
    q_norm: Vec<f32>,
    k_proj: LoadedAffineMatrix,
    k_norm: Vec<f32>,
    v_proj: LoadedAffineMatrix,
    o_proj: LoadedAffineMatrix,
    post_attn_norm: Vec<f32>,
    mlp_gate: LoadedAffineMatrix,
    shared_gate_proj: LoadedAffineMatrix,
    shared_up_proj: LoadedAffineMatrix,
    shared_down_proj: LoadedAffineMatrix,
    shared_expert_gate: LoadedAffineMatrix,
    final_norm: Vec<f32>,
    lm_head: LoadedAffineMatrix,
}

impl Qwen36Layer3Weights {
    fn load(shard1_path: &str, shard4_path: &str) -> Result<Self, String> {
        let mut shard1 = File::open(shard1_path).map_err(|e| format!("open {shard1_path}: {e}"))?;
        let mut header_len_bytes = [0u8; 8];
        shard1
            .read_exact(&mut header_len_bytes)
            .map_err(|e| format!("read shard1 header len: {e}"))?;
        let shard1_header_len = u64::from_le_bytes(header_len_bytes);
        if shard1_header_len != SHARD1_HEADER_LEN {
            return Err(format!(
                "shard1 header length mismatch: got {shard1_header_len}, expected {SHARD1_HEADER_LEN}"
            ));
        }
        let shard1_data_base = 8 + shard1_header_len;

        let mut shard4 = File::open(shard4_path).map_err(|e| format!("open {shard4_path}: {e}"))?;
        shard4
            .read_exact(&mut header_len_bytes)
            .map_err(|e| format!("read shard4 header len: {e}"))?;
        let shard4_header_len = u64::from_le_bytes(header_len_bytes);
        if shard4_header_len != SHARD4_HEADER_LEN {
            return Err(format!(
                "shard4 header length mismatch: got {shard4_header_len}, expected {SHARD4_HEADER_LEN}"
            ));
        }
        let shard4_data_base = 8 + shard4_header_len;

        let input_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_INPUT_NORM_SPEC)?;
        let q_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_Q_PROJ_SPEC)?;
        let q_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_Q_NORM_SPEC)?;
        let k_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_K_PROJ_SPEC)?;
        let k_norm = read_bf16_as_f32(&mut shard1, shard1_data_base, L3_K_NORM_SPEC)?;
        let v_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_V_PROJ_SPEC)?;
        let o_proj = LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_O_PROJ_SPEC)?;
        let post_attn_norm =
            read_bf16_as_f32(&mut shard1, shard1_data_base, L3_POST_ATTN_NORM_SPEC)?;
        let mlp_gate =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_MLP_GATE_SPEC)?;
        let shared_gate_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_GATE_PROJ_SPEC)?;
        let shared_up_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_UP_PROJ_SPEC)?;
        let shared_down_proj =
            LoadedAffineMatrix::read_from(&mut shard1, shard1_data_base, L3_SHARED_DOWN_PROJ_SPEC)?;
        let shared_expert_gate = LoadedAffineMatrix::read_from(
            &mut shard1,
            shard1_data_base,
            L3_SHARED_EXPERT_GATE_SPEC,
        )?;
        let final_norm = read_bf16_as_f32(&mut shard4, shard4_data_base, FINAL_NORM_SPEC)?;
        let lm_head = LoadedAffineMatrix::read_from(&mut shard4, shard4_data_base, LM_HEAD_SPEC)?;

        Ok(Self {
            shard1_file: shard1,
            shard1_data_base,
            input_norm,
            q_proj,
            q_norm,
            k_proj,
            k_norm,
            v_proj,
            o_proj,
            post_attn_norm,
            mlp_gate,
            shared_gate_proj,
            shared_up_proj,
            shared_down_proj,
            shared_expert_gate,
            final_norm,
            lm_head,
        })
    }

    fn embed_token(&mut self, token_id: u32) -> Result<Vec<f32>, String> {
        let row = token_id as usize;
        if row >= VOCAB_SIZE {
            return Err(format!("token_id {token_id} out of range"));
        }
        let words_per_row = (HIDDEN_SIZE * EMBED_TOKENS_SPEC.bits) / 32;
        let groups_per_row = HIDDEN_SIZE / QUANT_GROUP;
        let w_start = EMBED_TOKENS_SPEC.weight.start + (row * words_per_row * 4) as u64;
        let s_start = EMBED_TOKENS_SPEC.scales.start + (row * groups_per_row * 2) as u64;
        let b_start = EMBED_TOKENS_SPEC.biases.start + (row * groups_per_row * 2) as u64;
        let row_spec = AffineQuantSpec {
            weight: TensorSliceSpec::new(w_start, w_start + (words_per_row * 4) as u64),
            scales: TensorSliceSpec::new(s_start, s_start + (groups_per_row * 2) as u64),
            biases: TensorSliceSpec::new(b_start, b_start + (groups_per_row * 2) as u64),
            out_dim: 1,
            in_dim: HIDDEN_SIZE,
            bits: EMBED_TOKENS_SPEC.bits,
        };
        let row_mat =
            LoadedAffineMatrix::read_from(&mut self.shard1_file, self.shard1_data_base, row_spec)?;
        Ok(row_mat.dequantize_row(0))
    }

    fn run_sequence_forward(
        &mut self,
        tokens: &[u32],
        position_arm: PositionEncoderArm,
        tppe_config: &TppeConfig,
        observe: bool,
    ) -> Result<SequencePassOutput, String> {
        let mut full_k_by_head = [Vec::new(), Vec::new()];
        let mut full_v_by_head = [Vec::new(), Vec::new()];
        let mut hidden_by_pos = Vec::with_capacity(tokens.len());
        let mut logits_by_pos = Vec::with_capacity(tokens.len());
        let mut tapped_input_normed_by_pos = Vec::new();
        let mut tapped_quantized_weight_bytes = Vec::new();

        // One sequential carry for the whole sequence: the state at each
        // position continues the previous position's state. Nothing is
        // rebuilt per token and no token ID drives any step.
        let tppe_states = match position_arm {
            PositionEncoderArm::NativeRope => Vec::new(),
            PositionEncoderArm::AngleOnlyToroidal
            | PositionEncoderArm::TppeLinearLogV1
            | PositionEncoderArm::TppeLogV1 => {
                tppe_states_for_positions(tppe_config, tokens.len())?
            }
        };

        for (pos, &tok) in tokens.iter().enumerate() {
            let hidden = self.embed_token(tok)?;
            let normed = rms_norm_shifted(&hidden, &self.input_norm);
            if observe {
                tapped_input_normed_by_pos.push(normed[..TAPPED_ACTIVATION_DIM].to_vec());
            }

            let tppe_state = tppe_states.get(pos);
            let qg_raw = self.q_proj.project_vec(&normed);
            let k_raw = self.k_proj.project_vec(&normed);
            let v_raw = self.v_proj.project_vec(&normed);

            let mut q_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
            let mut gate_heads = vec![[0.0f32; HEAD_DIM]; Q_HEADS];
            for h in 0..Q_HEADS {
                let base = h * (2 * HEAD_DIM);
                let q_slice = &qg_raw[base..base + HEAD_DIM];
                let g_slice = &qg_raw[base + HEAD_DIM..base + 2 * HEAD_DIM];
                let mut q_normed = rms_norm_shifted(q_slice, &self.q_norm);
                apply_position_encoder_in_place(&mut q_normed, position_arm, tppe_state, pos)?;
                q_heads[h].copy_from_slice(&q_normed);
                gate_heads[h].copy_from_slice(g_slice);
            }

            for kv_h in 0..KV_HEADS {
                let base = kv_h * HEAD_DIM;
                let mut k_normed = rms_norm_shifted(&k_raw[base..base + HEAD_DIM], &self.k_norm);
                apply_position_encoder_in_place(&mut k_normed, position_arm, tppe_state, pos)?;
                full_k_by_head[kv_h].extend_from_slice(&k_normed);
                full_v_by_head[kv_h].extend_from_slice(&v_raw[base..base + HEAD_DIM]);
            }

            if observe && pos == 0 {
                // Tap a raw quantized Q8 weight slice from Layer-3 mlp_gate (37 bytes).
                let quant_spec = TensorSliceSpec::new(
                    L3_MLP_GATE_SPEC.weight.start,
                    L3_MLP_GATE_SPEC.weight.start + TAPPED_QUANTIZED_WEIGHT_BYTES as u64,
                );
                tapped_quantized_weight_bytes =
                    read_exact_slice(&mut self.shard1_file, self.shard1_data_base, quant_spec)?;
            }

            let seq_len = pos + 1;
            let (layer_out, logits) = self.finish_step_from_kv(
                &hidden,
                &q_heads,
                &gate_heads,
                &full_k_by_head,
                &full_v_by_head,
                seq_len,
            )?;
            hidden_by_pos.push(layer_out);
            logits_by_pos.push(logits);
        }

        let tapped_weight_basis = if observe {
            self.q_norm[..TAPPED_WEIGHT_BASIS_DIM].to_vec()
        } else {
            Vec::new()
        };

        Ok(SequencePassOutput {
            hidden_by_pos,
            logits_by_pos,
            tapped_input_normed_by_pos,
            tapped_weight_basis,
            tapped_quantized_weight_bytes,
            k_cache_head0: full_k_by_head[0].clone(),
        })
    }

    fn finish_step_from_kv(
        &mut self,
        decode_hidden_in: &[f32],
        q_heads: &[[f32; HEAD_DIM]],
        gate_heads: &[[f32; HEAD_DIM]],
        full_k_by_head: &[Vec<f32>; KV_HEADS],
        full_v_by_head: &[Vec<f32>; KV_HEADS],
        seq_len: usize,
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let scale = 1.0f32 / (HEAD_DIM as f32).sqrt();
        let heads_per_kv = Q_HEADS / KV_HEADS;
        let mut gated_concat = vec![0.0f32; Q_HEADS * HEAD_DIM];

        for q_h in 0..Q_HEADS {
            let kv_h = q_h / heads_per_kv;
            let k_cache = &full_k_by_head[kv_h];
            let v_cache = &full_v_by_head[kv_h];
            let q_vec = &q_heads[q_h];
            let mut scores = vec![0.0f32; seq_len];
            let mut max_score = f32::NEG_INFINITY;
            for (t, score_slot) in scores.iter_mut().enumerate() {
                let k_vec = &k_cache[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                let mut dot = 0.0f32;
                for d in 0..HEAD_DIM {
                    dot += q_vec[d] * k_vec[d];
                }
                let s = dot * scale;
                if s > max_score {
                    max_score = s;
                }
                *score_slot = s;
            }
            let mut denom = 0.0f32;
            for s in &mut scores {
                *s = (*s - max_score).exp();
                denom += *s;
            }
            for s in &mut scores {
                *s /= denom;
            }

            let mut attn_out = [0.0f32; HEAD_DIM];
            for (t, &weight) in scores.iter().enumerate() {
                let v_vec = &v_cache[t * HEAD_DIM..(t + 1) * HEAD_DIM];
                for d in 0..HEAD_DIM {
                    attn_out[d] += weight * v_vec[d];
                }
            }

            let gate_vec = &gate_heads[q_h];
            let out_slice = &mut gated_concat[q_h * HEAD_DIM..(q_h + 1) * HEAD_DIM];
            for d in 0..HEAD_DIM {
                out_slice[d] = attn_out[d] * sigmoid(gate_vec[d]);
            }
        }

        let o_out = self.o_proj.project_vec(&gated_concat);
        let mut post_attn_hidden = vec![0.0f32; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            post_attn_hidden[i] = decode_hidden_in[i] + o_out[i];
        }

        let mlp_in = rms_norm_shifted(&post_attn_hidden, &self.post_attn_norm);
        let router_logits = self.mlp_gate.project_vec(&mlp_in);
        let router_probs = softmax(&router_logits);

        let mut indexed: Vec<(usize, f32)> = router_probs.into_iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top_k = &indexed[..MOE_TOP_K];
        let weight_sum: f32 = top_k.iter().map(|(_, w)| *w).sum();

        let mut moe_accum = vec![0.0f32; HIDDEN_SIZE];
        for &(expert_idx, raw_weight) in top_k {
            let norm_weight = raw_weight / weight_sum;
            let gate_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_GATE_PROJ_SPEC,
                expert_idx,
            )?;
            let up_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_UP_PROJ_SPEC,
                expert_idx,
            )?;
            let down_mat = LoadedAffineMatrix::read_expert_from(
                &mut self.shard1_file,
                self.shard1_data_base,
                L3_SWITCH_DOWN_PROJ_SPEC,
                expert_idx,
            )?;
            let gate_act = gate_mat.project_vec(&mlp_in);
            let up_act = up_mat.project_vec(&mlp_in);
            let mut swiglu = vec![0.0f32; MOE_INTERMEDIATE];
            for i in 0..MOE_INTERMEDIATE {
                swiglu[i] = silu(gate_act[i]) * up_act[i];
            }
            let expert_out = down_mat.project_vec(&swiglu);
            for i in 0..HIDDEN_SIZE {
                moe_accum[i] += norm_weight * expert_out[i];
            }
        }

        let sh_gate = self.shared_gate_proj.project_vec(&mlp_in);
        let sh_up = self.shared_up_proj.project_vec(&mlp_in);
        let mut sh_swiglu = vec![0.0f32; MOE_INTERMEDIATE];
        for i in 0..MOE_INTERMEDIATE {
            sh_swiglu[i] = silu(sh_gate[i]) * sh_up[i];
        }
        let sh_down = self.shared_down_proj.project_vec(&sh_swiglu);
        let sh_scale = sigmoid(self.shared_expert_gate.project_vec(&mlp_in)[0]);

        let mut layer_out = vec![0.0f32; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            layer_out[i] = post_attn_hidden[i] + moe_accum[i] + sh_scale * sh_down[i];
        }

        let final_normed = rms_norm_shifted(&layer_out, &self.final_norm);
        let logits = self.lm_head.project_vec(&final_normed);
        Ok((layer_out, logits))
    }
}

/// Sequential TPPE states for positions `0..count`, computed in one
/// forward carry: state at position `p` is the state after `p + 1`
/// positional steps from the origin. Drives are purely positional
/// (`(step + 1) * rate`); a token ID never drives earlier steps, and no
/// state is rebuilt from scratch per position. The state at a position
/// is therefore a deterministic function of position alone, as a
/// positional encoder requires.
pub fn tppe_states_for_positions(
    config: &TppeConfig,
    count: usize,
) -> Result<Vec<TppeState>, String> {
    let mut states = Vec::with_capacity(count);
    let mut state = TppeState::origin(*config)?;
    for step_idx in 0..count {
        let drive_fast = (step_idx as f64 + 1.0) * 0.017;
        let drive_slow = (step_idx as f64 + 1.0) * 0.009;
        state = state.step(*config, drive_fast, drive_slow)?;
        states.push(state);
    }
    Ok(states)
}

/// Applies one arm at the Layer-3 Q/K boundary.
///
/// - `NativeRope`: the model's own RoPE — half-split pair rotation by
///   `position * theta^(-2p/64)` with `theta = 10,000,000` over the
///   64-wide rotary domain (`head_dim 256`, `partial_rotary_factor
///   0.25`). This is the control; it is never perturbed by TPPE state.
/// - `AngleOnlyToroidal`: pair rotation by the sequential state's T2
///   phase alone (fast phase on even pairs, slow on odd pairs). No
///   position/frequency product, no radial term.
/// - TPPE arms: the same phase rotation, then a radial scale of
///   `1 + 0.05 * poincare_radius` from the state's derived revolutions.
///   No RoPE frequency schedule enters a TPPE arm; the former rotary
///   hybrid (`position * inv_freq * radial_modulation + 0.01 * phase`)
///   is removed, not reparameterized.
fn apply_position_encoder_in_place(
    vec: &mut [f32],
    arm: PositionEncoderArm,
    state: Option<&TppeState>,
    token_pos: usize,
) -> Result<(), String> {
    let rope_pairs = 32usize;
    if vec.len() < 2 * rope_pairs {
        return Err("position encoder vector is shorter than the rotary domain".into());
    }
    for pair in 0..rope_pairs {
        let (angle, scale) = match arm {
            PositionEncoderArm::NativeRope => {
                let base_theta = 10_000_000.0f64;
                let inv_freq = base_theta.powf(-((2 * pair) as f64) / 64.0);
                (((token_pos as f64) * inv_freq) as f32, 1.0f32)
            }
            PositionEncoderArm::AngleOnlyToroidal => {
                let state =
                    state.ok_or_else(|| "angle-only arm is missing T2 state".to_string())?;
                let t2 = state.phase_t2();
                (t2[pair % 2] as f32, 1.0f32)
            }
            PositionEncoderArm::TppeLinearLogV1 | PositionEncoderArm::TppeLogV1 => {
                let state = state.ok_or_else(|| "TPPE arm is missing TPPE state".to_string())?;
                let t2 = state.phase_t2();
                (
                    t2[pair % 2] as f32,
                    (1.0 + 0.05 * state.poincare_radius) as f32,
                )
            }
        };
        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = vec[pair];
        let x1 = vec[pair + rope_pairs];
        vec[pair] = scale * (x0 * cos_a - x1 * sin_a);
        vec[pair + rope_pairs] = scale * (x1 * cos_a + x0 * sin_a);
    }
    Ok(())
}

fn bf16_to_f32(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

fn rms_norm_shifted(x: &[f32], weight: &[f32]) -> Vec<f32> {
    assert_eq!(x.len(), weight.len());
    let mut sum_sq = 0.0f32;
    for &v in x {
        sum_sq += v * v;
    }
    let inv_rms = 1.0f32 / (sum_sq / (x.len() as f32) + RMS_NORM_EPS).sqrt();
    let mut out = vec![0.0f32; x.len()];
    for i in 0..x.len() {
        out[i] = x[i] * inv_rms * (1.0f32 + weight[i]);
    }
    out
}

fn sigmoid(x: f32) -> f32 {
    1.0f32 / (1.0f32 + (-x).exp())
}

fn silu(x: f32) -> f32 {
    x * sigmoid(x)
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let mut max_v = f32::NEG_INFINITY;
    for &v in logits {
        if v > max_v {
            max_v = v;
        }
    }
    let mut out = vec![0.0f32; logits.len()];
    let mut sum = 0.0f32;
    for (i, &v) in logits.iter().enumerate() {
        let e = (v - max_v).exp();
        out[i] = e;
        sum += e;
    }
    for v in &mut out {
        *v /= sum;
    }
    out
}

fn read_exact_slice(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(data_base + spec.start))
        .map_err(|e| format!("seek safetensors: {e}"))?;
    let mut buf = vec![0u8; spec.byte_len()];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read safetensors slice: {e}"))?;
    Ok(buf)
}

fn read_u16_vec(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u16>, String> {
    let bytes = read_exact_slice(file, data_base, spec)?;
    if bytes.len() % 2 != 0 {
        return Err("u16 slice byte length is odd".into());
    }
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect())
}

fn read_u32_vec(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<u32>, String> {
    let bytes = read_exact_slice(file, data_base, spec)?;
    if bytes.len() % 4 != 0 {
        return Err("u32 slice byte length is not a multiple of 4".into());
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| u32::from_le_bytes(*c))
        .collect())
}

fn read_bf16_as_f32(
    file: &mut File,
    data_base: u64,
    spec: TensorSliceSpec,
) -> Result<Vec<f32>, String> {
    let u16s = read_u16_vec(file, data_base, spec)?;
    Ok(u16s.into_iter().map(bf16_to_f32).collect())
}

fn f32_slice_to_le_bytes(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn hash_f32_slices(slices: &[Vec<f32>]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"globalmind-f32-sequence-v1");
    hasher.update((slices.len() as u64).to_le_bytes());
    for slice in slices {
        hasher.update((slice.len() as u64).to_le_bytes());
        for &v in slice {
            hasher.update(v.to_le_bytes());
        }
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(hex).expect("SHA-256 is a valid Digest")
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
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

pub(crate) fn digest_to_bytes(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let mut out = [0u8; 32];
    for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

pub(crate) fn digest_from_bytes(bytes: &[u8]) -> Result<Digest, String> {
    if bytes.len() != 32 {
        return Err("digest byte slice must be 32 bytes".into());
    }
    let mut hex = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(hex)
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

struct BinCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinCursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
        let slice = self
            .bytes
            .get(self.offset..self.offset + len)
            .ok_or_else(|| "binary payload is truncated".to_string())?;
        self.offset += len;
        Ok(slice)
    }

    fn take_u16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn take_u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn take_u64(&mut self) -> Result<u64, String> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn take_f32(&mut self) -> Result<f32, String> {
        let b = self.take(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn take_digest(&mut self) -> Result<Digest, String> {
        digest_from_bytes(self.take(32)?)
    }

    fn take_str(&mut self) -> Result<String, String> {
        let len = self.take_u32()? as usize;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("invalid UTF-8 string in binary payload: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_contract_matches_current_sources() {
        verify_candidate_source_contract().unwrap();
    }

    #[test]
    fn radial_candidates_match_at_one_derived_revolution() {
        let linear = default_tppe_config(RadialLaw::LinearLogV1);
        let log = default_tppe_config(RadialLaw::LogV1);
        let linear_end = linear.radial_start + linear.radial_span * (0.35 + 0.65 * 2.0f64.ln());
        let log_end = log.radial_start + log.radial_span * 2.0f64.ln();
        assert!((linear_end - log_end).abs() <= 64.0 * f64::EPSILON);
        assert_eq!(linear.radial_start, log.radial_start);
        let zero_phase = |mut cfg: TppeConfig| {
            cfg.omega_fast = 0.0;
            cfg.omega_slow = 0.0;
            cfg.coupling_fast = 0.0;
            cfg.coupling_slow = 0.0;
            cfg
        };
        let linear = zero_phase(linear);
        let log = zero_phase(log);
        let linear_state = TppeState::origin(linear)
            .unwrap()
            .step(linear, std::f64::consts::TAU, 0.0)
            .unwrap();
        let log_state = TppeState::origin(log)
            .unwrap()
            .step(log, std::f64::consts::TAU, 0.0)
            .unwrap();
        assert!(
            (linear_state.hyperbolic_radius - log_state.hyperbolic_radius).abs()
                <= 64.0 * f64::EPSILON
        );
    }

    #[test]
    fn position_ablation_metrics_use_real_targets_and_logits() {
        let logits = vec![vec![0.0, 1.0, 2.0], vec![3.0, 1.0, 0.0]];
        let (nll, hits, predictions) = next_token_quality(&logits, &[0, 2]).unwrap();
        let expected = ((0.0f64 - 2.0).exp() + (1.0f64 - 2.0).exp() + 1.0).ln();
        assert!((nll - expected).abs() < 1e-12);
        assert_eq!(hits, 1);
        assert_eq!(predictions, 1);
        assert!(next_token_quality(&logits, &[0]).is_err());
        assert!(next_token_quality(&[vec![f32::NAN]], &[0, 0]).is_err());

        let (mean, max) = logit_delta(&[vec![1.0, 2.0]], &[vec![2.0, 4.0]]).unwrap();
        assert_eq!(mean, 1.5);
        assert_eq!(max, 2.0);
        assert!(logit_delta(&[vec![1.0]], &[vec![1.0, 2.0]]).is_err());
    }

    #[test]
    fn test_token_tape_roundtrip_and_save() {
        let bytes = encode_token_tape(&DEFAULT_REAL_TOKEN_TAPE).unwrap();
        let (tokens, digest) = decode_token_tape(&bytes).unwrap();
        assert_eq!(tokens, DEFAULT_REAL_TOKEN_TAPE);
        assert_eq!(digest, Digest::sha256(&bytes));

        let inputs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("inputs");
        std::fs::create_dir_all(&inputs_dir).unwrap();
        let tape_path = inputs_dir.join("real-token-tape.bin");
        std::fs::write(&tape_path, &bytes).unwrap();
    }
}
