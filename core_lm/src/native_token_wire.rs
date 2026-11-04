//! Native binary text-to-token boundary owned by the serving model adapter.
use crate::sha256::sha256;

pub const TOKEN_REQUEST_MAGIC: &[u8; 4] = b"GTK2";
pub const TOKEN_RESPONSE_MAGIC: &[u8; 4] = b"GTO2";
pub const MAX_TEXT_BYTES: usize = 16 * 1024;
pub const MAX_TOKEN_IDS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeRequest {
    pub text: String,
    pub add_bos: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeResponse {
    pub token_ids: Vec<u32>,
    pub tokenizer_digest: [u8; 32],
    pub receipt_digest: [u8; 32],
}

pub fn encode_request(request: &TokenizeRequest) -> Result<Vec<u8>, String> {
    let text = request.text.as_bytes();
    if text.is_empty() || text.len() > MAX_TEXT_BYTES {
        return Err("native tokenize text length invalid".into());
    }
    let mut bytes = Vec::with_capacity(9 + text.len());
    bytes.extend_from_slice(TOKEN_REQUEST_MAGIC);
    bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
    bytes.push(u8::from(request.add_bos));
    bytes.extend_from_slice(text);
    Ok(bytes)
}

pub fn decode_request(bytes: &[u8]) -> Result<TokenizeRequest, String> {
    if bytes.len() < 10 || &bytes[..4] != TOKEN_REQUEST_MAGIC {
        return Err("native tokenize request header invalid".into());
    }
    let length = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if length == 0 || length > MAX_TEXT_BYTES || bytes.len() != 9 + length || bytes[8] > 1 {
        return Err("native tokenize request bounds invalid".into());
    }
    let text = std::str::from_utf8(&bytes[9..]).map_err(|e| e.to_string())?;
    Ok(TokenizeRequest {
        text: text.to_string(),
        add_bos: bytes[8] == 1,
    })
}

pub fn encode_response(
    request_bytes: &[u8],
    token_ids: &[u32],
    tokenizer_digest: [u8; 32],
) -> Result<Vec<u8>, String> {
    if token_ids.is_empty() || token_ids.len() > MAX_TOKEN_IDS {
        return Err("native tokenize output count invalid".into());
    }
    let mut bytes = Vec::with_capacity(72 + token_ids.len() * 4);
    bytes.extend_from_slice(TOKEN_RESPONSE_MAGIC);
    bytes.extend_from_slice(&(token_ids.len() as u32).to_le_bytes());
    for id in token_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    bytes.extend_from_slice(&tokenizer_digest);
    let mut bound = Vec::with_capacity(request_bytes.len() + bytes.len());
    bound.extend_from_slice(request_bytes);
    bound.extend_from_slice(&bytes);
    bytes.extend_from_slice(&sha256(&bound));
    Ok(bytes)
}

pub fn decode_response(
    request_bytes: &[u8],
    bytes: &[u8],
) -> Result<TokenizeResponse, String> {
    if bytes.len() < 76 || &bytes[..4] != TOKEN_RESPONSE_MAGIC {
        return Err("native tokenize response header invalid".into());
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if count == 0 || count > MAX_TOKEN_IDS || bytes.len() != 72 + count * 4 {
        return Err("native tokenize response length invalid".into());
    }
    let mut bound = Vec::with_capacity(request_bytes.len() + bytes.len() - 32);
    bound.extend_from_slice(request_bytes);
    bound.extend_from_slice(&bytes[..bytes.len() - 32]);
    if sha256(&bound) != bytes[bytes.len() - 32..] {
        return Err("native tokenize receipt mismatch".into());
    }
    let token_ids = bytes[8..8 + count * 4]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    let tokenizer_digest = bytes[8 + count * 4..40 + count * 4].try_into().unwrap();
    let receipt_digest = bytes[40 + count * 4..].try_into().unwrap();
    Ok(TokenizeResponse {
        token_ids,
        tokenizer_digest,
        receipt_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_token_frames_reject_changed_text_and_ids() {
        let request = encode_request(&TokenizeRequest {
            text: "verified atom".into(),
            add_bos: true,
        })
        .unwrap();
        assert_eq!(decode_request(&request).unwrap().text, "verified atom");
        let response = encode_response(&request, &[1, 2], [3; 32]).unwrap();
        assert_eq!(decode_response(&request, &response).unwrap().token_ids, [1, 2]);
        let mut changed = response;
        changed[8] ^= 1;
        assert!(decode_response(&request, &changed).is_err());
        assert!(decode_response(b"other request", &changed).is_err());
    }

    #[test]
    fn native_token_frames_reject_malformed_and_truncated_requests() {
        let valid = encode_request(&TokenizeRequest {
            text: "verified atom".into(),
            add_bos: false,
        })
        .unwrap();
        assert!(decode_request(&valid[..valid.len() - 1]).is_err());
        assert!(decode_request(&valid[..4]).is_err());
        let mut wrong_magic = valid.clone();
        wrong_magic[0] = b'X';
        assert!(decode_request(&wrong_magic).is_err());
        let mut oversize = valid.clone();
        oversize[4..8].copy_from_slice(&(MAX_TEXT_BYTES as u32 + 1).to_le_bytes());
        assert!(decode_request(&oversize).is_err());
        let mut bad_bos = valid;
        bad_bos[8] = 2;
        assert!(decode_request(&bad_bos).is_err());
        assert!(encode_request(&TokenizeRequest {
            text: String::new(),
            add_bos: true,
        })
        .is_err());
    }
}
