//! Server-resident session wire. The server keeps each session's exact K/V cache;
//! the client holds only an opaque id and sends NEW tokens, so no pages cross the
//! wire per turn. Bounded, binary, receipt-bound to the request bytes.
//!   GSF2 fork:  [magic][id u64]
//!   GST2 turn:  [magic][id u64][cont u32][max u32][cont ids]
//!   GSR2 reply: [magic][id u64][tokens u32][out u32][out ids][logits][kv][w][t][c][composite][receipt]
//! Prefill reuses GPF2 (see `native_pages_wire`); its reply is also GSR2 with out=0.

use crate::artifact_digest::ArtifactDigest;
use crate::sha256::sha256;

pub const FORK_MAGIC: &[u8; 4] = b"GSF2";
pub const TURN_MAGIC: &[u8; 4] = b"GST2";
pub const REPLY_MAGIC: &[u8; 4] = b"GSR2";
pub const MAX_TURN_IDS: usize = 256;
pub const MAX_TURN_TOKENS: usize = 128;

fn u32_at(b: &[u8], at: usize) -> usize {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize
}

pub fn encode_fork(id: u64) -> Vec<u8> {
    let mut out = FORK_MAGIC.to_vec();
    out.extend_from_slice(&id.to_le_bytes());
    out
}

pub fn decode_fork(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() != 12 || &bytes[..4] != FORK_MAGIC {
        return Err("fork request invalid".into());
    }
    Ok(u64::from_le_bytes(bytes[4..12].try_into().unwrap()))
}

pub fn encode_turn(id: u64, cont: &[u32], max_tokens: usize) -> Result<Vec<u8>, String> {
    if cont.is_empty() || cont.len() > MAX_TURN_IDS || !(1..=MAX_TURN_TOKENS).contains(&max_tokens)
    {
        return Err("turn request bounds violated".into());
    }
    let mut out = TURN_MAGIC.to_vec();
    out.extend_from_slice(&id.to_le_bytes());
    out.extend_from_slice(&(cont.len() as u32).to_le_bytes());
    out.extend_from_slice(&(max_tokens as u32).to_le_bytes());
    for t in cont {
        out.extend_from_slice(&t.to_le_bytes());
    }
    Ok(out)
}

pub fn decode_turn(bytes: &[u8]) -> Result<(u64, Vec<u32>, usize), String> {
    if bytes.len() < 20 || &bytes[..4] != TURN_MAGIC {
        return Err("turn request header invalid".into());
    }
    let id = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
    let (n, max) = (u32_at(bytes, 12), u32_at(bytes, 16));
    if n == 0 || n > MAX_TURN_IDS || !(1..=MAX_TURN_TOKENS).contains(&max) || bytes.len() != 20 + n * 4
    {
        return Err("turn request bounds invalid".into());
    }
    Ok((id, (0..n).map(|i| u32_at(bytes, 20 + 4 * i) as u32).collect(), max))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub id: u64,
    pub tokens: usize,
    pub output_ids: Vec<u32>,
    pub logits_digest: [u8; 32],
    pub kv_digest: [u8; 32],
    pub artifact: ArtifactDigest,
    pub receipt: [u8; 32],
}

pub fn encode_reply(
    request: &[u8],
    id: u64,
    tokens: usize,
    output_ids: &[u32],
    logits_digest: [u8; 32],
    kv_digest: [u8; 32],
    artifact: ArtifactDigest,
) -> Vec<u8> {
    let mut out = REPLY_MAGIC.to_vec();
    out.extend_from_slice(&id.to_le_bytes());
    out.extend_from_slice(&(tokens as u32).to_le_bytes());
    out.extend_from_slice(&(output_ids.len() as u32).to_le_bytes());
    for t in output_ids {
        out.extend_from_slice(&t.to_le_bytes());
    }
    out.extend_from_slice(&logits_digest);
    out.extend_from_slice(&kv_digest);
    for d in [artifact.weights, artifact.tokenizer, artifact.config, artifact.composite] {
        out.extend_from_slice(&d);
    }
    let mut bound = request.to_vec();
    bound.extend_from_slice(&out);
    out.extend_from_slice(&sha256(&bound));
    out
}

pub fn decode_reply(request: &[u8], bytes: &[u8]) -> Result<Reply, String> {
    if bytes.len() < 20 + 32 * 7 || &bytes[..4] != REPLY_MAGIC {
        return Err("session reply header invalid".into());
    }
    let id = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
    let (tokens, n) = (u32_at(bytes, 12), u32_at(bytes, 16));
    if n > MAX_TURN_TOKENS || bytes.len() != 20 + n * 4 + 32 * 7 {
        return Err("session reply length invalid".into());
    }
    let split = bytes.len() - 32;
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes[..split]);
    if sha256(&bound) != bytes[split..] {
        return Err("session reply receipt mismatch".into());
    }
    let at = 20 + n * 4;
    let d = |i: usize| -> [u8; 32] { bytes[at + 32 * i..at + 32 * (i + 1)].try_into().unwrap() };
    let artifact = ArtifactDigest {
        weights: d(2),
        tokenizer: d(3),
        config: d(4),
        composite: d(5),
    };
    artifact.verify_composite()?;
    Ok(Reply {
        id,
        tokens,
        output_ids: (0..n).map(|i| u32_at(bytes, 20 + 4 * i) as u32).collect(),
        logits_digest: d(0),
        kv_digest: d(1),
        artifact,
        receipt: d(6),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_and_reject_tampering() {
        assert_eq!(decode_fork(&encode_fork(7)).unwrap(), 7);
        let t = encode_turn(9, &[1, 2, 3], 4).unwrap();
        assert_eq!(decode_turn(&t).unwrap(), (9, vec![1, 2, 3], 4));
        assert!(decode_turn(&t[..t.len() - 1]).is_err());
        assert!(encode_turn(1, &[], 4).is_err());
        let art = ArtifactDigest::from_parts([1; 32], [2; 32], [3; 32]);
        let r = encode_reply(&t, 9, 42, &[5, 6], [7; 32], [8; 32], art);
        let got = decode_reply(&t, &r).unwrap();
        assert_eq!((got.id, got.tokens, got.output_ids), (9, 42, vec![5, 6]));
        assert!(decode_reply(b"other", &r).is_err());
        let mut bad = r.clone();
        bad[21] ^= 1;
        assert!(decode_reply(&t, &bad).is_err());
    }
}
