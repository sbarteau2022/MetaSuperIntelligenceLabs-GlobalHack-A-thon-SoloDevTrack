//! Bounded, versioned native token/tensor receipt wire for exact Glimmer K/V.
//! This module contains no JSON or text-to-token conversion.

use crate::artifact_digest::ArtifactDigest;
use crate::exact_kv::ExactKvCache;
use crate::sha256::sha256;

pub const REQUEST_MAGIC: &[u8; 4] = b"GFX2";
pub const RESPONSE_MAGIC: &[u8; 4] = b"GFO2";
pub const VERIFY_REQUEST_MAGIC: &[u8; 4] = b"GVX2";
pub const MAX_DRAFT_TOKENS: usize = 16;
pub const MAX_INPUT_TOKENS: usize = 4096;
pub const MAX_OUTPUT_TOKENS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactForwardRequest {
    pub input_ids: Vec<u32>,
    pub max_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactForwardResponse {
    pub output_ids: Vec<u32>,
    pub logits_digest: [u8; 32],
    pub kv_digest: [u8; 32],
    pub artifact: ArtifactDigest,
    pub receipt_digest: [u8; 32],
}

pub fn decode_response(
    request_bytes: &[u8],
    response_bytes: &[u8],
) -> Result<ExactForwardResponse, String> {
    if response_bytes.len() < 232 || &response_bytes[..4] != RESPONSE_MAGIC {
        return Err("invalid exact-forward response header".into());
    }
    let count = u32::from_le_bytes(response_bytes[4..8].try_into().unwrap()) as usize;
    if !(1..=MAX_OUTPUT_TOKENS).contains(&count) || response_bytes.len() != 232 + count * 4 {
        return Err("exact-forward response length mismatch".into());
    }
    let mut bound = Vec::with_capacity(request_bytes.len() + response_bytes.len() - 32);
    bound.extend_from_slice(request_bytes);
    bound.extend_from_slice(&response_bytes[..response_bytes.len() - 32]);
    if sha256(&bound) != response_bytes[response_bytes.len() - 32..] {
        return Err("exact-forward response receipt mismatch".into());
    }
    let mut cursor = 8;
    let output_ids = response_bytes[cursor..cursor + count * 4]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    cursor += count * 4;
    let logits_digest = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let kv_digest = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let weights = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let tokenizer = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let config = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let composite = response_bytes[cursor..cursor + 32].try_into().unwrap();
    cursor += 32;
    let receipt_digest = response_bytes[cursor..cursor + 32].try_into().unwrap();
    let artifact = ArtifactDigest {
        weights,
        tokenizer,
        config,
        composite,
    };
    artifact.verify_composite()?;
    Ok(ExactForwardResponse {
        output_ids,
        logits_digest,
        kv_digest,
        artifact,
        receipt_digest,
    })
}

/// Speculative-verify request: the model predicts after `context`, then after
/// each draft prefix, stopping at the first draft token it disagrees with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyRequest {
    pub context_ids: Vec<u32>,
    pub draft_ids: Vec<u32>,
}

pub fn encode_verify_request(req: &VerifyRequest) -> Result<Vec<u8>, String> {
    if req.context_ids.is_empty()
        || req.context_ids.len() + req.draft_ids.len() > MAX_INPUT_TOKENS
        || req.draft_ids.len() > MAX_DRAFT_TOKENS
    {
        return Err("verify request bounds violated".into());
    }
    let mut bytes = Vec::with_capacity(12 + (req.context_ids.len() + req.draft_ids.len()) * 4);
    bytes.extend_from_slice(VERIFY_REQUEST_MAGIC);
    bytes.extend_from_slice(&(req.context_ids.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(req.draft_ids.len() as u32).to_le_bytes());
    for id in req.context_ids.iter().chain(&req.draft_ids) {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    Ok(bytes)
}

pub fn decode_verify_request(bytes: &[u8]) -> Result<VerifyRequest, String> {
    if bytes.len() < 16 || &bytes[..4] != VERIFY_REQUEST_MAGIC {
        return Err("invalid verify request header".into());
    }
    let ctx = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let draft = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if ctx == 0
        || draft > MAX_DRAFT_TOKENS
        || ctx + draft > MAX_INPUT_TOKENS
        || bytes.len() != 12 + (ctx + draft) * 4
    {
        return Err("verify request bounds violated".into());
    }
    let ids: Vec<u32> = (0..ctx + draft)
        .map(|i| u32::from_le_bytes(bytes[12 + 4 * i..16 + 4 * i].try_into().unwrap()))
        .collect();
    Ok(VerifyRequest {
        context_ids: ids[..ctx].to_vec(),
        draft_ids: ids[ctx..].to_vec(),
    })
}

pub fn decode_request(bytes: &[u8]) -> Result<ExactForwardRequest, String> {
    if bytes.len() < 12 || &bytes[..4] != REQUEST_MAGIC {
        return Err("invalid exact-forward request header".into());
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let max_tokens = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if count == 0 || count > MAX_INPUT_TOKENS || !(1..=MAX_OUTPUT_TOKENS).contains(&max_tokens) {
        return Err("exact-forward request bounds violated".into());
    }
    if bytes.len() != 12 + count * 4 {
        return Err("exact-forward request length mismatch".into());
    }
    let input_ids = bytes[12..]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    Ok(ExactForwardRequest {
        input_ids,
        max_tokens,
    })
}

pub fn encode_request(req: &ExactForwardRequest) -> Result<Vec<u8>, String> {
    if req.input_ids.is_empty()
        || req.input_ids.len() > MAX_INPUT_TOKENS
        || !(1..=MAX_OUTPUT_TOKENS).contains(&req.max_tokens)
    {
        return Err("exact-forward request bounds violated".into());
    }
    let mut bytes = Vec::with_capacity(12 + req.input_ids.len() * 4);
    bytes.extend_from_slice(REQUEST_MAGIC);
    bytes.extend_from_slice(&(req.input_ids.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(req.max_tokens as u32).to_le_bytes());
    for id in &req.input_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    Ok(bytes)
}

/// Digest every K and V head page, plus its layer/head identity and shape.
pub fn exact_kv_digest(
    cache: &ExactKvCache,
    layers: usize,
    kv_heads: usize,
    tokens: usize,
) -> Result<[u8; 32], String> {
    cache.verify_complete_prefix(tokens)?;
    let mut roster = Vec::with_capacity(32 + layers * kv_heads * 80);
    roster.extend_from_slice(b"glimmer-exact-kv-v1\0");
    for n in [layers, kv_heads, tokens] {
        roster.extend_from_slice(&(n as u64).to_le_bytes());
    }
    for layer in 0..layers {
        for head in 0..kv_heads {
            let (keys, values) = cache.head_page(layer, head).ok_or("missing K/V page")?;
            roster.extend_from_slice(&(layer as u64).to_le_bytes());
            roster.extend_from_slice(&(head as u64).to_le_bytes());
            for values in [keys, values] {
                let mut page = Vec::with_capacity(values.len() * 4);
                for value in values {
                    page.extend_from_slice(&value.to_le_bytes());
                }
                roster.extend_from_slice(&(page.len() as u64).to_le_bytes());
                roster.extend_from_slice(&sha256(&page));
            }
        }
    }
    Ok(sha256(&roster))
}

pub fn encode_response(
    request_bytes: &[u8],
    output_ids: &[u32],
    logits_digest: [u8; 32],
    kv_digest: [u8; 32],
    artifact: ArtifactDigest,
) -> Result<Vec<u8>, String> {
    if output_ids.is_empty() || output_ids.len() > MAX_OUTPUT_TOKENS {
        return Err("exact-forward response token count invalid".into());
    }
    let mut bytes = Vec::with_capacity(232 + output_ids.len() * 4);
    bytes.extend_from_slice(RESPONSE_MAGIC);
    bytes.extend_from_slice(&(output_ids.len() as u32).to_le_bytes());
    for id in output_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    bytes.extend_from_slice(&logits_digest);
    bytes.extend_from_slice(&kv_digest);
    bytes.extend_from_slice(&artifact.weights);
    bytes.extend_from_slice(&artifact.tokenizer);
    bytes.extend_from_slice(&artifact.config);
    bytes.extend_from_slice(&artifact.composite);
    let mut bound = Vec::with_capacity(request_bytes.len() + bytes.len());
    bound.extend_from_slice(request_bytes);
    bound.extend_from_slice(&bytes);
    bytes.extend_from_slice(&sha256(&bound));
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_request_rejects_truncation_and_unknown_version() {
        let request = ExactForwardRequest {
            input_ids: vec![1, 2],
            max_tokens: 1,
        };
        let bytes = encode_request(&request).unwrap();
        assert_eq!(decode_request(&bytes).unwrap(), request);
        assert!(decode_request(&bytes[..bytes.len() - 1]).is_err());
        let mut unknown = bytes;
        unknown[3] = b'3';
        assert!(decode_request(&unknown).is_err());
    }

    #[test]
    fn verify_request_round_trips_and_rejects_bad_frames() {
        let req = VerifyRequest {
            context_ids: vec![1, 2, 3],
            draft_ids: vec![4, 5],
        };
        let bytes = encode_verify_request(&req).unwrap();
        assert_eq!(decode_verify_request(&bytes).unwrap(), req);
        assert!(decode_verify_request(&bytes[..bytes.len() - 1]).is_err());
        assert!(encode_verify_request(&VerifyRequest {
            context_ids: vec![],
            draft_ids: vec![1],
        })
        .is_err());
        assert!(encode_verify_request(&VerifyRequest {
            context_ids: vec![1],
            draft_ids: vec![0; MAX_DRAFT_TOKENS + 1],
        })
        .is_err());
        let mut wrong = bytes;
        wrong[3] = b'9';
        assert!(decode_verify_request(&wrong).is_err());
    }

    #[test]
    fn response_receipt_rejects_changed_output() {
        let request = encode_request(&ExactForwardRequest {
            input_ids: vec![1],
            max_tokens: 1,
        })
        .unwrap();
        let artifact = ArtifactDigest::from_parts([5; 32], [6; 32], [7; 32]);
        let response = encode_response(&request, &[2], [3; 32], [4; 32], artifact).unwrap();
        assert_eq!(
            decode_response(&request, &response).unwrap().output_ids,
            [2]
        );
        let mut tampered = response;
        tampered[8] ^= 1;
        assert!(decode_response(&request, &tampered).is_err());
        let mut false_artifact = artifact;
        false_artifact.composite[0] ^= 1;
        let false_response =
            encode_response(&request, &[2], [3; 32], [4; 32], false_artifact).unwrap();
        assert!(decode_response(&request, &false_response).is_err());
    }
}
