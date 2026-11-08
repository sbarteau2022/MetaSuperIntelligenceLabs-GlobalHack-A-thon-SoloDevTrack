//! Model driver: one completion from Qwen3.6-MoE over the native binary seat (`serve_qwen`).
//! Flow: tokenize (GTK2/GTO2) -> complete (QNC1/QNR1) -> detokenize (GDT2/GDR2).
//! Every reply is checked against its SHA-256 receipt. Requests and replies are typed
//! bytes; no JSON crosses this boundary.

/// One model completion: text + hex of the weights digest reported for the turn.
/// Model-agnostic shape (formerly in the removed glimmer_driver).
#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub composite_hex: String,
}
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const TOKEN_REQUEST_MAGIC: &[u8; 4] = b"GTK2";
const TOKEN_RESPONSE_MAGIC: &[u8; 4] = b"GTO2";
const DETOK_MAGIC: &[u8; 4] = b"GDT2";
const DETOK_REPLY_MAGIC: &[u8; 4] = b"GDR2";
const COMPLETE_MAGIC: &[u8; 4] = b"QNC1";
const COMPLETE_REPLY_MAGIC: &[u8; 4] = b"QNR1";
const MAX_NEW_TOKENS: usize = 128;
const MAX_PROMPT_TOKENS: usize = 1024;

pub struct QwenDriver {
    addr: String,
}

impl QwenDriver {
    pub fn new(addr: &str) -> Self {
        Self { addr: addr.to_string() }
    }

    /// Greedy completion of `prompt`, at most `max_tokens` new tokens. The server resets its
    /// memory per call, so identical prompts give identical completions.
    pub fn complete(&self, prompt: &str, max_tokens: usize) -> Result<Completion, String> {
        if prompt.is_empty() {
            return Err("empty prompt".into());
        }
        if !(1..=MAX_NEW_TOKENS).contains(&max_tokens) {
            return Err("max_tokens out of bounds".into());
        }
        let ids = self.tokenize(prompt)?;
        if ids.len() > MAX_PROMPT_TOKENS {
            return Err(format!("prompt is {} tokens; limit is {MAX_PROMPT_TOKENS}", ids.len()));
        }
        let req = encode_complete(max_tokens, &ids);
        let resp = self.post("/v1/complete-native", &req)?;
        let (out_ids, receipt) = decode_complete_reply(&req, &resp)?;
        let text = self.detokenize(&out_ids)?;
        Ok(Completion { text, composite_hex: hex(&receipt) })
    }

    /// Text -> token ids, no BOS (Qwen's config has none). Receipt-checked.
    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>, String> {
        let t = text.as_bytes();
        if t.is_empty() {
            return Err("empty tokenize input".into());
        }
        let mut req = Vec::with_capacity(9 + t.len());
        req.extend_from_slice(TOKEN_REQUEST_MAGIC);
        req.extend_from_slice(&(t.len() as u32).to_le_bytes());
        req.push(0);
        req.extend_from_slice(t);
        let resp = self.post("/v1/tokenize-native", &req)?;
        if resp.len() < 76 || &resp[..4] != TOKEN_RESPONSE_MAGIC {
            return Err("tokenize reply header invalid".into());
        }
        let count = u32::from_le_bytes(resp[4..8].try_into().unwrap()) as usize;
        if count == 0 || resp.len() != 72 + count * 4 {
            return Err("tokenize reply length invalid".into());
        }
        check_receipt(&req, &resp, "tokenize")?;
        Ok(le_u32s(&resp[8..8 + count * 4]))
    }

    /// Token ids -> text, receipt-checked.
    pub fn detokenize(&self, ids: &[u32]) -> Result<String, String> {
        let mut req = DETOK_MAGIC.to_vec();
        req.extend_from_slice(&(ids.len() as u32).to_le_bytes());
        for id in ids {
            req.extend_from_slice(&id.to_le_bytes());
        }
        let resp = self.post("/v1/detokenize-native", &req)?;
        if resp.len() < 40 || &resp[..4] != DETOK_REPLY_MAGIC {
            return Err("detokenize reply header invalid".into());
        }
        let len = u32::from_le_bytes(resp[4..8].try_into().unwrap()) as usize;
        if resp.len() != 40 + len {
            return Err("detokenize reply length invalid".into());
        }
        check_receipt(&req, &resp, "detokenize")?;
        String::from_utf8(resp[8..8 + len].to_vec()).map_err(|e| e.to_string())
    }

    /// Minimal HTTP/1.1 POST over a fresh connection. The server always answers with Content-Length.
    fn post(&self, path: &str, body: &[u8]) -> Result<Vec<u8>, String> {
        let mut stream =
            TcpStream::connect(&self.addr).map_err(|e| format!("connect {}: {e}", self.addr))?;
        stream.set_read_timeout(Some(Duration::from_secs(900))).ok();
        let head = format!(
            "POST {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.addr,
            body.len()
        );
        stream
            .write_all(head.as_bytes())
            .and_then(|_| stream.write_all(body))
            .map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|e| format!("{path} read: {e}"))?;
        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or("malformed HTTP reply")?;
        let head = std::str::from_utf8(&raw[..split]).map_err(|_| "non-UTF8 HTTP head")?;
        let status = head.lines().next().unwrap_or("");
        if !status.contains(" 200 ") {
            let detail = String::from_utf8_lossy(&raw[split + 4..]).to_string();
            return Err(format!("{path}: {status}: {detail}"));
        }
        Ok(raw[split + 4..].to_vec())
    }
}

fn encode_complete(max_new: usize, ids: &[u32]) -> Vec<u8> {
    let mut out = COMPLETE_MAGIC.to_vec();
    out.extend_from_slice(&(max_new as u32).to_le_bytes());
    out.extend_from_slice(&(ids.len() as u32).to_le_bytes());
    for id in ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    out
}

/// Returns (output ids, receipt). The receipt is SHA-256 over request || reply-without-receipt.
fn decode_complete_reply(req: &[u8], resp: &[u8]) -> Result<(Vec<u32>, [u8; 32]), String> {
    if resp.len() < 40 || &resp[..4] != COMPLETE_REPLY_MAGIC {
        return Err("complete reply header invalid".into());
    }
    let n = u32::from_le_bytes(resp[4..8].try_into().unwrap()) as usize;
    if n > MAX_NEW_TOKENS || resp.len() != 8 + 4 * n + 32 {
        return Err("complete reply length invalid".into());
    }
    check_receipt(req, resp, "complete")?;
    let mut receipt = [0u8; 32];
    receipt.copy_from_slice(&resp[resp.len() - 32..]);
    Ok((le_u32s(&resp[8..8 + 4 * n]), receipt))
}

fn check_receipt(req: &[u8], resp: &[u8], what: &str) -> Result<(), String> {
    let split = resp.len() - 32;
    let mut bound = req.to_vec();
    bound.extend_from_slice(&resp[..split]);
    if Sha256::digest(&bound)[..] != resp[split..] {
        return Err(format!("{what} receipt mismatch"));
    }
    Ok(())
}

fn le_u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
