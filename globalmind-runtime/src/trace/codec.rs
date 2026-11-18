//! Binary codec for [`TraceEnvelope`]. See the module docs for the frame
//! layout. Encoding validates first; decoding revalidates, so a tampered
//! payload or digest fails closed on either side.

use super::{FRAME_PREFIX_LEN, TRACE_MAGIC, TRACE_VERSION, TraceDtype, TraceEnvelope, TraceKind};
use crate::Digest;

struct Cursor {
    offset: usize,
}

impl Cursor {
    fn take<'a>(&mut self, header: &'a [u8], len: usize) -> Result<&'a [u8], String> {
        let slice = header
            .get(self.offset..self.offset + len)
            .ok_or_else(|| "trace header is truncated".to_string())?;
        self.offset += len;
        Ok(slice)
    }
}

/// Fixed header size for version 1:
/// 3 digests (96) + kind/dtype/shape_len (3) + shape[8] (32) + layer/head (8)
/// + token_position (8) + observer digest (32) + payload_len (8)
/// + payload digest (32).
const HEADER_LEN: usize = 219;

pub fn encode(envelope: &TraceEnvelope) -> Result<Vec<u8>, String> {
    envelope.validate()?;
    let mut out = Vec::with_capacity(FRAME_PREFIX_LEN + HEADER_LEN + envelope.payload.len());
    out.extend_from_slice(&TRACE_MAGIC);
    out.extend_from_slice(&TRACE_VERSION.to_le_bytes());
    out.extend_from_slice(&(HEADER_LEN as u32).to_le_bytes());
    out.extend_from_slice(&digest_bytes(&envelope.model_digest));
    out.extend_from_slice(&digest_bytes(&envelope.config_digest));
    out.extend_from_slice(&digest_bytes(&envelope.tokenizer_digest));
    out.push(envelope.kind.code());
    out.push(envelope.dtype.code());
    out.push(envelope.shape.len() as u8);
    for index in 0..8 {
        let dimension = envelope.shape.get(index).copied().unwrap_or(0);
        out.extend_from_slice(&dimension.to_le_bytes());
    }
    out.extend_from_slice(&envelope.layer.to_le_bytes());
    out.extend_from_slice(&envelope.head.to_le_bytes());
    out.extend_from_slice(&envelope.token_position.to_le_bytes());
    out.extend_from_slice(&digest_bytes(&envelope.observer_receipt_digest));
    out.extend_from_slice(&(envelope.payload.len() as u64).to_le_bytes());
    out.extend_from_slice(&digest_bytes(&envelope.payload_digest));
    out.extend_from_slice(&envelope.payload);
    Ok(out)
}

/// Decodes one frame from the front of a stream and returns the envelope
/// plus the exact number of bytes consumed, so callers can walk
/// concatenated frames. Single-frame [`decode`] remains strict about
/// total length.
pub fn decode_frame(bytes: &[u8]) -> Result<(TraceEnvelope, usize), String> {
    let envelope = decode_prefix(bytes)?;
    let consumed = FRAME_PREFIX_LEN + HEADER_LEN + envelope.payload.len();
    Ok((envelope, consumed))
}

pub fn decode(bytes: &[u8]) -> Result<TraceEnvelope, String> {
    let envelope = decode_prefix(bytes)?;
    if bytes.len() != FRAME_PREFIX_LEN + HEADER_LEN + envelope.payload.len() {
        return Err("trace frame length does not match its payload length".into());
    }
    Ok(envelope)
}

fn decode_prefix(bytes: &[u8]) -> Result<TraceEnvelope, String> {
    if bytes.len() < FRAME_PREFIX_LEN + HEADER_LEN {
        return Err("trace frame is truncated".into());
    }
    if bytes[..4] != TRACE_MAGIC {
        return Err("trace frame magic mismatch".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != TRACE_VERSION {
        return Err(format!("unsupported trace version: {version}"));
    }
    let header_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    if header_len != HEADER_LEN {
        return Err(format!("unsupported trace header length: {header_len}"));
    }
    let header = &bytes[FRAME_PREFIX_LEN..FRAME_PREFIX_LEN + HEADER_LEN];
    let mut cursor = Cursor { offset: 0 };
    let model_digest = digest_from(cursor.take(header, 32)?)?;
    let config_digest = digest_from(cursor.take(header, 32)?)?;
    let tokenizer_digest = digest_from(cursor.take(header, 32)?)?;
    let kind = TraceKind::from_code(cursor.take(header, 1)?[0])?;
    let dtype = TraceDtype::from_code(cursor.take(header, 1)?[0])?;
    let shape_len = cursor.take(header, 1)?[0] as usize;
    if !(1..=8).contains(&shape_len) {
        return Err("trace shape length is invalid".into());
    }
    let mut shape = Vec::with_capacity(shape_len);
    for index in 0..8 {
        let raw = cursor.take(header, 4)?;
        let dimension = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        if index < shape_len {
            shape.push(dimension);
        } else if dimension != 0 {
            return Err("trace shape padding must be zero".into());
        }
    }
    let layer_raw = cursor.take(header, 4)?;
    let layer = u32::from_le_bytes([layer_raw[0], layer_raw[1], layer_raw[2], layer_raw[3]]);
    let head_raw = cursor.take(header, 4)?;
    let head = u32::from_le_bytes([head_raw[0], head_raw[1], head_raw[2], head_raw[3]]);
    let token_raw = cursor.take(header, 8)?;
    let token_position = u64::from_le_bytes([
        token_raw[0],
        token_raw[1],
        token_raw[2],
        token_raw[3],
        token_raw[4],
        token_raw[5],
        token_raw[6],
        token_raw[7],
    ]);
    let observer_receipt_digest = digest_from(cursor.take(header, 32)?)?;
    let payload_len_raw = cursor.take(header, 8)?;
    let payload_len_u64 = u64::from_le_bytes([
        payload_len_raw[0],
        payload_len_raw[1],
        payload_len_raw[2],
        payload_len_raw[3],
        payload_len_raw[4],
        payload_len_raw[5],
        payload_len_raw[6],
        payload_len_raw[7],
    ]);
    let payload_len = usize::try_from(payload_len_u64)
        .map_err(|_| "trace payload length overflows usize".to_string())?;
    let payload_digest = digest_from(cursor.take(header, 32)?)?;
    let payload_start = FRAME_PREFIX_LEN + HEADER_LEN;
    if bytes.len() < payload_start + payload_len {
        return Err("trace frame is truncated before its payload ends".into());
    }
    let envelope = TraceEnvelope {
        model_digest,
        config_digest,
        tokenizer_digest,
        kind,
        dtype,
        shape,
        layer,
        head,
        token_position,
        observer_receipt_digest,
        payload: bytes[payload_start..payload_start + payload_len]
            .to_vec()
            .into_boxed_slice(),
        payload_digest,
    };
    envelope.validate()?;
    Ok(envelope)
}

fn digest_bytes(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let mut out = [0u8; 32];
    for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

fn digest_from(bytes: &[u8]) -> Result<Digest, String> {
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

#[cfg(test)]
mod tests {
    use super::super::{TraceDtype, TraceKind};
    use super::*;

    fn envelope() -> TraceEnvelope {
        let payload: Vec<u8> = (0..16).flat_map(|v| (v as f32).to_le_bytes()).collect();
        TraceEnvelope {
            model_digest: Digest::parse("a".repeat(64)).unwrap(),
            config_digest: Digest::parse("b".repeat(64)).unwrap(),
            tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
            kind: TraceKind::Activation,
            dtype: TraceDtype::F32,
            shape: vec![4, 4],
            layer: 3,
            head: 1,
            token_position: 42,
            observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        }
    }

    #[test]
    fn roundtrip_preserves_all_fields() {
        let original = envelope();
        let decoded = decode(&encode(&original).unwrap()).unwrap();
        assert_eq!(decoded.model_digest, original.model_digest);
        assert_eq!(decoded.config_digest, original.config_digest);
        assert_eq!(decoded.tokenizer_digest, original.tokenizer_digest);
        assert_eq!(decoded.kind, original.kind);
        assert_eq!(decoded.dtype, original.dtype);
        assert_eq!(decoded.shape, original.shape);
        assert_eq!(decoded.layer, original.layer);
        assert_eq!(decoded.head, original.head);
        assert_eq!(decoded.token_position, original.token_position);
        assert_eq!(
            decoded.observer_receipt_digest,
            original.observer_receipt_digest
        );
        assert_eq!(decoded.payload, original.payload);
        assert_eq!(decoded.content_digest(), original.content_digest());
    }

    #[test]
    fn tampered_payload_or_digest_fails_closed() {
        let mut bytes = encode(&envelope()).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        assert!(decode(&bytes).is_err());
        let mut bytes = encode(&envelope()).unwrap();
        bytes[FRAME_PREFIX_LEN] ^= 0xFF; // model digest byte: provenance change still decodes
        assert!(decode(&bytes).is_ok());
        let mut bytes = encode(&envelope()).unwrap();
        bytes[FRAME_PREFIX_LEN + HEADER_LEN - 1] ^= 0xFF; // payload digest tail
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn bad_magic_version_and_truncation_fail_closed() {
        let bytes = encode(&envelope()).unwrap();
        let mut bad = bytes.clone();
        bad[0] = b'X';
        assert!(decode(&bad).is_err());
        let mut bad = bytes.clone();
        bad[4] = 9;
        assert!(decode(&bad).is_err());
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
        assert!(decode(&bytes[..20]).is_err());
    }

    #[test]
    fn content_digest_is_teacher_neutral() {
        let mut twin = envelope();
        twin.model_digest = Digest::parse("e".repeat(64)).unwrap();
        twin.config_digest = Digest::parse("f".repeat(64)).unwrap();
        twin.tokenizer_digest = Digest::parse("0".repeat(64)).unwrap();
        assert_eq!(twin.content_digest(), envelope().content_digest());
    }

    #[test]
    fn dense_shape_mismatch_fails_closed() {
        let mut bad = envelope();
        bad.shape = vec![5, 4];
        assert!(encode(&bad).is_err());
    }

    #[test]
    fn nonzero_shape_padding_fails_closed() {
        let mut bytes = encode(&envelope()).unwrap();
        // shape[2] slot: header offset 96 + 3 + 2*4
        let slot = FRAME_PREFIX_LEN + 96 + 3 + 8;
        bytes[slot] = 7;
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn concatenated_frames_walk_exactly() {
        let first = encode(&envelope()).unwrap();
        let mut second_env = envelope();
        second_env.token_position = 43;
        second_env.layer = 9;
        let second = encode(&second_env).unwrap();
        let mut stream = first.clone();
        stream.extend_from_slice(&second);
        assert!(decode(&stream).is_err()); // strict single-frame decode rejects trailing bytes
        let (decoded_first, consumed) = decode_frame(&stream).unwrap();
        assert_eq!(consumed, first.len());
        assert_eq!(decoded_first.token_position, 42);
        let (decoded_second, consumed_second) = decode_frame(&stream[consumed..]).unwrap();
        assert_eq!(consumed + consumed_second, stream.len());
        assert_eq!(decoded_second.token_position, 43);
        assert_eq!(decoded_second.layer, 9);
    }

    #[test]
    fn all_kinds_and_dtypes_roundtrip() {
        for kind in [
            TraceKind::Activation,
            TraceKind::Weight,
            TraceKind::KeyValue,
        ] {
            for dtype in [
                TraceDtype::F16,
                TraceDtype::Bf16,
                TraceDtype::F32,
                TraceDtype::Q8_0,
                TraceDtype::Q4_0,
            ] {
                let mut env = envelope();
                env.kind = kind;
                env.dtype = dtype;
                // resize payload to shape/dtype: 16 elements dense,
                // receipt-checked only for quantized formats
                let len = match dtype.scalar_bytes() {
                    Some(width) => 16 * width,
                    None => 37,
                };
                env.payload = vec![0xABu8; len].into_boxed_slice();
                env.payload_digest = Digest::sha256(&env.payload);
                let decoded = decode(&encode(&env).unwrap()).unwrap();
                assert_eq!(decoded.kind, kind);
                assert_eq!(decoded.dtype, dtype);
            }
        }
        assert!(TraceKind::from_code(0).is_err());
        assert!(TraceDtype::from_code(0).is_err());
        assert!(TraceDtype::from_code(255).is_err());
    }

    #[test]
    fn shape_bounds_are_enforced() {
        let mut env = envelope();
        env.shape = vec![2, 2, 2, 2, 2, 2, 2, 2]; // 8 dims ok
        env.payload = vec![0u8; 256 * 4].into_boxed_slice();
        env.payload_digest = Digest::sha256(&env.payload);
        assert!(encode(&env).is_ok());
        let mut bad = envelope();
        bad.shape = vec![1; 9];
        assert!(encode(&bad).is_err());
        let mut bad = envelope();
        bad.shape = vec![];
        assert!(encode(&bad).is_err());
        let mut bad = envelope();
        bad.shape = vec![4, 0];
        assert!(encode(&bad).is_err());
        let mut bad = envelope();
        bad.payload = Vec::new().into_boxed_slice();
        bad.payload_digest = Digest::sha256(&bad.payload);
        assert!(encode(&bad).is_err());
    }
}
