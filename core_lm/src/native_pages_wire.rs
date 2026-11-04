//! Bounded binary wire for exporting and re-hydrating exact Glimmer K/V pages.
//! GPF2/GPO2: prefill a short prefix and return every layer/head page.
//! GCX2: continue from caller-supplied pages (the response is a GFO2 frame).
//! No JSON, no text. Page order is layer-major then head-major, K then V.

use crate::artifact_digest::ArtifactDigest;
use crate::sha256::sha256;

pub const PREFILL_REQUEST_MAGIC: &[u8; 4] = b"GPF2";
pub const PREFILL_RESPONSE_MAGIC: &[u8; 4] = b"GPO2";
pub const CONTINUE_REQUEST_MAGIC: &[u8; 4] = b"GCX2";
pub const MAX_PAGE_TOKENS: usize = 16;
pub const MAX_CONTINUE_IDS: usize = 64;
pub const MAX_MAX_TOKENS: usize = 128;
const MAX_LAYERS: usize = 256;
const MAX_KV_HEADS: usize = 64;
const MAX_HEAD_DIM: usize = 1024;

pub type Pages = Vec<(Vec<f32>, Vec<f32>)>;

#[derive(Debug, Clone, PartialEq)]
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
    fn page_floats(&self) -> usize {
        self.tokens * self.head_dim
    }
    fn page_count(&self) -> usize {
        self.layers * self.kv_heads
    }
    fn payload_bytes(&self) -> usize {
        self.page_count() * self.page_floats() * 2 * 4
    }
}

fn put_pages(out: &mut Vec<u8>, shape: &Shape, pages: &Pages) -> Result<(), String> {
    if pages.len() != shape.page_count() {
        return Err("exact page count does not match shape".into());
    }
    for (keys, values) in pages {
        if keys.len() != shape.page_floats() || values.len() != shape.page_floats() {
            return Err("exact page tensor length does not match shape".into());
        }
        for v in keys.iter().chain(values) {
            if !v.is_finite() {
                return Err("exact page contains a non-finite value".into());
            }
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    Ok(())
}

fn get_pages(bytes: &[u8], shape: &Shape) -> Result<Pages, String> {
    if bytes.len() != shape.payload_bytes() {
        return Err("exact page payload length mismatch".into());
    }
    let floats = |slice: &[u8]| -> Result<Vec<f32>, String> {
        (0..slice.len() / 4)
            .map(|i| {
                let v = f32::from_le_bytes(slice[4 * i..4 * i + 4].try_into().unwrap());
                v.is_finite()
                    .then_some(v)
                    .ok_or_else(|| "exact page contains a non-finite value".to_string())
            })
            .collect()
    };
    let each = shape.page_floats() * 4;
    (0..shape.page_count())
        .map(|p| {
            let at = p * each * 2;
            Ok((
                floats(&bytes[at..at + each])?,
                floats(&bytes[at + each..at + 2 * each])?,
            ))
        })
        .collect()
}

fn read_u32(bytes: &[u8], at: usize) -> usize {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

pub fn encode_prefill_request(input_ids: &[u32]) -> Result<Vec<u8>, String> {
    if input_ids.is_empty() || input_ids.len() > MAX_PAGE_TOKENS {
        return Err("prefill token count out of bounds".into());
    }
    let mut out = Vec::with_capacity(8 + input_ids.len() * 4);
    out.extend_from_slice(PREFILL_REQUEST_MAGIC);
    out.extend_from_slice(&(input_ids.len() as u32).to_le_bytes());
    for id in input_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    Ok(out)
}

pub fn decode_prefill_request(bytes: &[u8]) -> Result<Vec<u32>, String> {
    if bytes.len() < 12 || &bytes[..4] != PREFILL_REQUEST_MAGIC {
        return Err("prefill request header invalid".into());
    }
    let count = read_u32(bytes, 4);
    if count == 0 || count > MAX_PAGE_TOKENS || bytes.len() != 8 + count * 4 {
        return Err("prefill request bounds invalid".into());
    }
    Ok((0..count).map(|i| read_u32(bytes, 8 + 4 * i) as u32).collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefillResponse {
    pub shape: Shape,
    pub pages: Pages,
    pub kv_digest: [u8; 32],
    pub artifact: ArtifactDigest,
    pub receipt: [u8; 32],
}

pub fn encode_prefill_response(
    request: &[u8],
    shape: &Shape,
    pages: &Pages,
    kv_digest: [u8; 32],
    artifact: ArtifactDigest,
) -> Result<Vec<u8>, String> {
    shape.validate()?;
    let mut out = Vec::with_capacity(24 + shape.payload_bytes() + 192);
    out.extend_from_slice(PREFILL_RESPONSE_MAGIC);
    for n in [shape.layers, shape.kv_heads, shape.head_dim, shape.tokens] {
        out.extend_from_slice(&(n as u32).to_le_bytes());
    }
    put_pages(&mut out, shape, pages)?;
    out.extend_from_slice(&kv_digest);
    for d in [
        artifact.weights,
        artifact.tokenizer,
        artifact.config,
        artifact.composite,
    ] {
        out.extend_from_slice(&d);
    }
    let mut bound = request.to_vec();
    bound.extend_from_slice(&out);
    out.extend_from_slice(&sha256(&bound));
    Ok(out)
}

pub fn decode_prefill_response(request: &[u8], bytes: &[u8]) -> Result<PrefillResponse, String> {
    if bytes.len() < 20 + 32 * 6 || &bytes[..4] != PREFILL_RESPONSE_MAGIC {
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
    if sha256(&bound) != bytes[split..] {
        return Err("prefill response receipt mismatch".into());
    }
    let pages_end = 20 + shape.payload_bytes();
    let pages = get_pages(&bytes[20..pages_end], &shape)?;
    let d = |i: usize| -> [u8; 32] {
        bytes[pages_end + 32 * i..pages_end + 32 * (i + 1)]
            .try_into()
            .unwrap()
    };
    let artifact = ArtifactDigest {
        weights: d(1),
        tokenizer: d(2),
        config: d(3),
        composite: d(4),
    };
    artifact.verify_composite()?;
    Ok(PrefillResponse {
        shape,
        pages,
        kv_digest: d(0),
        artifact,
        receipt: d(5),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinueRequest {
    pub shape: Shape,
    pub pages: Pages,
    pub continuation_ids: Vec<u32>,
    pub max_tokens: usize,
}

pub fn encode_continue_request(req: &ContinueRequest) -> Result<Vec<u8>, String> {
    req.shape.validate()?;
    if req.continuation_ids.is_empty()
        || req.continuation_ids.len() > MAX_CONTINUE_IDS
        || !(1..=MAX_MAX_TOKENS).contains(&req.max_tokens)
    {
        return Err("continue request bounds violated".into());
    }
    let mut out = Vec::new();
    out.extend_from_slice(CONTINUE_REQUEST_MAGIC);
    for n in [
        req.shape.layers,
        req.shape.kv_heads,
        req.shape.head_dim,
        req.shape.tokens,
        req.continuation_ids.len(),
        req.max_tokens,
    ] {
        out.extend_from_slice(&(n as u32).to_le_bytes());
    }
    put_pages(&mut out, &req.shape, &req.pages)?;
    for id in &req.continuation_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    Ok(out)
}

pub fn decode_continue_request(bytes: &[u8]) -> Result<ContinueRequest, String> {
    if bytes.len() < 28 || &bytes[..4] != CONTINUE_REQUEST_MAGIC {
        return Err("continue request header invalid".into());
    }
    let shape = Shape {
        layers: read_u32(bytes, 4),
        kv_heads: read_u32(bytes, 8),
        head_dim: read_u32(bytes, 12),
        tokens: read_u32(bytes, 16),
    };
    shape.validate()?;
    let (count, max_tokens) = (read_u32(bytes, 20), read_u32(bytes, 24));
    if count == 0 || count > MAX_CONTINUE_IDS || !(1..=MAX_MAX_TOKENS).contains(&max_tokens) {
        return Err("continue request bounds invalid".into());
    }
    let pages_end = 28 + shape.payload_bytes();
    if bytes.len() != pages_end + count * 4 {
        return Err("continue request length mismatch".into());
    }
    Ok(ContinueRequest {
        pages: get_pages(&bytes[28..pages_end], &shape)?,
        continuation_ids: (0..count)
            .map(|i| read_u32(bytes, pages_end + 4 * i) as u32)
            .collect(),
        shape,
        max_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> (Shape, Pages) {
        let shape = Shape {
            layers: 2,
            kv_heads: 2,
            head_dim: 3,
            tokens: 2,
        };
        let pages = (0..4)
            .map(|p| {
                (
                    (0..6).map(|i| (p * 10 + i) as f32).collect(),
                    (0..6).map(|i| (p * 10 + i) as f32 + 0.5).collect(),
                )
            })
            .collect();
        (shape, pages)
    }

    #[test]
    fn prefill_response_round_trips_and_rejects_tampering() {
        let (shape, pages) = tiny();
        let req = encode_prefill_request(&[1, 2]).unwrap();
        assert_eq!(decode_prefill_request(&req).unwrap(), [1, 2]);
        let art = ArtifactDigest::from_parts([1; 32], [2; 32], [3; 32]);
        let resp = encode_prefill_response(&req, &shape, &pages, [9; 32], art).unwrap();
        let got = decode_prefill_response(&req, &resp).unwrap();
        assert_eq!(got.pages, pages);
        let mut bad = resp.clone();
        bad[24] ^= 1;
        assert!(decode_prefill_response(&req, &bad).is_err());
        assert!(decode_prefill_response(b"other", &resp).is_err());
        assert!(decode_prefill_response(&req, &resp[..resp.len() - 1]).is_err());
    }

    #[test]
    fn continue_request_round_trips_and_rejects_bad_frames() {
        let (shape, pages) = tiny();
        let req = ContinueRequest {
            shape,
            pages,
            continuation_ids: vec![7],
            max_tokens: 3,
        };
        let bytes = encode_continue_request(&req).unwrap();
        assert_eq!(decode_continue_request(&bytes).unwrap(), req);
        assert!(decode_continue_request(&bytes[..bytes.len() - 1]).is_err());
        let mut nan = bytes.clone();
        nan[28..32].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_continue_request(&nan).is_err());
        let mut short = req.clone();
        short.pages.pop();
        assert!(encode_continue_request(&short).is_err());
        let mut zero = bytes;
        zero[20..24].copy_from_slice(&0u32.to_le_bytes());
        assert!(decode_continue_request(&zero).is_err());
    }
}
