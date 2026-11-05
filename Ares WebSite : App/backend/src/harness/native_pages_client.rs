//! Client side of the exact K/V page wire (GPF2/GPO2 prefill export, GCX2
//! continue). Mirrors `core_lm/src/native_pages_wire.rs`; the server is the
//! authority and the hardware gate runs both ends. Binary only, bounded.

use sha2::{Digest, Sha256};

pub const MAX_PAGE_TOKENS: usize = 160;
const MAX_LAYERS: usize = 256;
const MAX_KV_HEADS: usize = 64;
const MAX_HEAD_DIM: usize = 1024;

pub type Pages = Vec<(Vec<f32>, Vec<f32>)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub layers: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub tokens: usize,
}

impl Shape {
    fn validate(&self) -> Result<(), String> {
        if !(1..=MAX_LAYERS).contains(&self.layers)
            || !(1..=MAX_KV_HEADS).contains(&self.kv_heads)
            || !(1..=MAX_HEAD_DIM).contains(&self.head_dim)
            || !(1..=MAX_PAGE_TOKENS).contains(&self.tokens)
        {
            return Err("exact page shape out of bounds".into());
        }
        Ok(())
    }
    pub fn page_floats(&self) -> usize {
        self.tokens * self.head_dim
    }
    pub fn page_count(&self) -> usize {
        self.layers * self.kv_heads
    }
    fn payload_bytes(&self) -> usize {
        self.page_count() * self.page_floats() * 8
    }
}

fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn read_u32(bytes: &[u8], at: usize) -> usize {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

pub fn check_pages(shape: &Shape, pages: &Pages) -> Result<(), String> {
    shape.validate()?;
    if pages.len() != shape.page_count() {
        return Err("exact page count does not match shape".into());
    }
    for (k, v) in pages {
        if k.len() != shape.page_floats()
            || v.len() != shape.page_floats()
            || !k.iter().chain(v).all(|x| x.is_finite())
        {
            return Err("exact page tensor invalid for shape".into());
        }
    }
    Ok(())
}

pub fn encode_prefill_request(ids: &[u32]) -> Result<Vec<u8>, String> {
    if ids.is_empty() || ids.len() > MAX_PAGE_TOKENS {
        return Err("prefill token count out of bounds".into());
    }
    let mut out = b"GPF2".to_vec();
    out.extend_from_slice(&(ids.len() as u32).to_le_bytes());
    for id in ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    Ok(out)
}

pub struct PrefillResponse {
    pub shape: Shape,
    pub pages: Pages,
    pub kv_digest: [u8; 32],
    pub weights: [u8; 32],
    pub tokenizer: [u8; 32],
    pub config: [u8; 32],
    pub composite: [u8; 32],
    pub receipt: [u8; 32],
}

pub fn decode_prefill_response(request: &[u8], bytes: &[u8]) -> Result<PrefillResponse, String> {
    if bytes.len() < 20 + 32 * 6 || &bytes[..4] != b"GPO2" {
        return Err("prefill response header invalid".into());
    }
    let shape = Shape {
        layers: read_u32(bytes, 4),
        kv_heads: read_u32(bytes, 8),
        head_dim: read_u32(bytes, 12),
        tokens: read_u32(bytes, 16),
    };
    shape.validate()?;
    if bytes.len() != 20 + shape.payload_bytes() + 32 * 6 {
        return Err("prefill response length mismatch".into());
    }
    let split = bytes.len() - 32;
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes[..split]);
    if sha(&bound) != bytes[split..] {
        return Err("prefill response receipt mismatch".into());
    }
    let pages_end = 20 + shape.payload_bytes();
    let floats = |s: &[u8]| -> Result<Vec<f32>, String> {
        (0..s.len() / 4)
            .map(|i| {
                let v = f32::from_le_bytes(s[4 * i..4 * i + 4].try_into().unwrap());
                v.is_finite()
                    .then_some(v)
                    .ok_or_else(|| "exact page contains a non-finite value".to_string())
            })
            .collect()
    };
    let each = shape.page_floats() * 4;
    let pages = (0..shape.page_count())
        .map(|p| {
            let at = 20 + p * each * 2;
            Ok((
                floats(&bytes[at..at + each])?,
                floats(&bytes[at + each..at + 2 * each])?,
            ))
        })
        .collect::<Result<Pages, String>>()?;
    let d = |i: usize| -> [u8; 32] {
        bytes[pages_end + 32 * i..pages_end + 32 * (i + 1)]
            .try_into()
            .unwrap()
    };
    Ok(PrefillResponse {
        shape,
        pages,
        kv_digest: d(0),
        weights: d(1),
        tokenizer: d(2),
        config: d(3),
        composite: d(4),
        receipt: d(5),
    })
}

/// Continue-with-state response (GCO2): generated tokens plus the updated exact
/// K/V pages for a worker session. Mirrors core_lm `decode_session_response`.
pub struct SessionResponse {
    pub output_ids: Vec<u32>,
    pub shape: Shape,
    pub pages: Pages,
    pub logits_digest: [u8; 32],
    pub kv_digest: [u8; 32],
    pub weights: [u8; 32],
    pub tokenizer: [u8; 32],
    pub config: [u8; 32],
    pub composite: [u8; 32],
    pub receipt: [u8; 32],
}

pub fn decode_session_response(request: &[u8], bytes: &[u8]) -> Result<SessionResponse, String> {
    if bytes.len() < 8 + 16 + 32 * 7 || &bytes[..4] != b"GCO2" {
        return Err("session response header invalid".into());
    }
    let count = read_u32(bytes, 4);
    if count == 0 || count > 128 || bytes.len() < 8 + count * 4 + 16 {
        return Err("session response count invalid".into());
    }
    let mut at = 8 + count * 4;
    let shape = Shape {
        layers: read_u32(bytes, at),
        kv_heads: read_u32(bytes, at + 4),
        head_dim: read_u32(bytes, at + 8),
        tokens: read_u32(bytes, at + 12),
    };
    shape.validate()?;
    at += 16;
    if bytes.len() != at + shape.payload_bytes() + 32 * 7 {
        return Err("session response length mismatch".into());
    }
    let split = bytes.len() - 32;
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes[..split]);
    if sha(&bound) != bytes[split..] {
        return Err("session response receipt mismatch".into());
    }
    let floats = |s: &[u8]| -> Result<Vec<f32>, String> {
        (0..s.len() / 4)
            .map(|i| {
                let v = f32::from_le_bytes(s[4 * i..4 * i + 4].try_into().unwrap());
                v.is_finite()
                    .then_some(v)
                    .ok_or_else(|| "exact page contains a non-finite value".to_string())
            })
            .collect()
    };
    let each = shape.page_floats() * 4;
    let pages = (0..shape.page_count())
        .map(|p| {
            let o = at + p * each * 2;
            Ok((
                floats(&bytes[o..o + each])?,
                floats(&bytes[o + each..o + 2 * each])?,
            ))
        })
        .collect::<Result<Pages, String>>()?;
    at += shape.payload_bytes();
    let d = |i: usize| -> [u8; 32] { bytes[at + 32 * i..at + 32 * (i + 1)].try_into().unwrap() };
    Ok(SessionResponse {
        output_ids: (0..count)
            .map(|i| read_u32(bytes, 8 + 4 * i) as u32)
            .collect(),
        shape,
        pages,
        logits_digest: d(0),
        kv_digest: d(1),
        weights: d(2),
        tokenizer: d(3),
        config: d(4),
        composite: d(5),
        receipt: d(6),
    })
}

pub fn encode_continue_request(
    shape: &Shape,
    pages: &Pages,
    continuation_ids: &[u32],
    max_tokens: usize,
) -> Result<Vec<u8>, String> {
    check_pages(shape, pages)?;
    if continuation_ids.is_empty()
        || continuation_ids.len() > 64
        || !(1..=128).contains(&max_tokens)
    {
        return Err("continue request bounds violated".into());
    }
    let mut out = b"GCX2".to_vec();
    for n in [
        shape.layers,
        shape.kv_heads,
        shape.head_dim,
        shape.tokens,
        continuation_ids.len(),
        max_tokens,
    ] {
        out.extend_from_slice(&(n as u32).to_le_bytes());
    }
    for (k, v) in pages {
        for x in k.iter().chain(v) {
            out.extend_from_slice(&x.to_le_bytes());
        }
    }
    for id in continuation_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_request_rejects_wrong_page_set() {
        let shape = Shape {
            layers: 1,
            kv_heads: 2,
            head_dim: 2,
            tokens: 1,
        };
        let page = (vec![0.0f32; 2], vec![1.0f32; 2]);
        let ok = vec![page.clone(), page.clone()];
        assert!(encode_continue_request(&shape, &ok, &[3], 1).is_ok());
        assert!(encode_continue_request(&shape, &ok[..1].to_vec(), &[3], 1).is_err());
        let mut nan = ok.clone();
        nan[0].0[0] = f32::NAN;
        assert!(encode_continue_request(&shape, &nan, &[3], 1).is_err());
        assert!(encode_continue_request(&shape, &ok, &[], 1).is_err());
    }

    #[test]
    fn prefill_response_rejects_bad_header_and_receipt() {
        let req = encode_prefill_request(&[1]).unwrap();
        assert!(decode_prefill_response(&req, b"short").is_err());
        assert!(encode_prefill_request(&[]).is_err());
        assert!(encode_prefill_request(&[0; MAX_PAGE_TOKENS + 1]).is_err());
    }
}
