//! Model seat: a typed HTTP seat for a Glimmer serving binary.
//!
//! Restored 2026-11-04, after the original model_seat.rs (Nov 3, 252 lines,
//! untracked) was lost with the same wipe that took core_lm/src and the
//! serve_glimmer source. This is a reconstruction from the surviving
//! backend contracts: native_mtp::parse_info_body for /v1/info, the
//! serve_glimmer JSON shapes (tokenize array, forward output_token_ids /
//! logits_digest / receipt_sha256), and the recorded kv_page refusal
//! string. It is not claimed byte-identical to the lost original.
//!
//! What this module is, and is not:
//! - It is a seat: identity attested by /v1/info, tokenize and forward
//!   over HTTP to a loopback Glimmer server, with digests validated
//!   (64 lowercase hex) before they are returned to any caller.
//! - Its ordinary call probes, generates text, then runs a separate
//!   tokenized forward. That receipt attests the second run only; it
//!   cannot bind the generated text to one model execution.
//! - kv_page is refused by this HTTP seat. Exact Glimmer tensor K/V
//!   exists in the separate native component, but is not exposed here.
//!
//! Step 3: Implements `call_bound` as a stub. The Glimmer HTTP seat
//! currently uses prose prompts + separate forward; a full bound-output
//! implementation would require a typed binary protocol to the Glimmer
//! server (removing prose tool/args from model I/O boundary). For now,
//! the live mission path uses an MCP-only executor and fails closed
//! for model-bound output. This seat remains for compatibility/testing.

use crate::harness::native_mtp::{parse_info_body, parse_stream};
use crate::harness::output_receipt::BoundCallOutcome;
use crate::harness::runtime::{CallOutcome, Executor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub const KV_PAGE_REFUSAL: &str =
    "kv_page not attested for Glimmer seat (GeometricMemoryMap; no standard KV tensor)";

fn is_loopback(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:")
}

fn is_lower_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardOutput {
    pub output_token_ids: Vec<u32>,
    pub logits_digest: String,
    pub receipt_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ForwardResponse {
    output_token_ids: Vec<u32>,
    logits_digest: String,
    receipt_sha256: String,
}

fn parse_forward_response(text: &str, input_ids: &[u32]) -> Result<ForwardOutput, String> {
    let r: ForwardResponse =
        serde_json::from_str(text).map_err(|e| format!("forward response json: {e}"))?;
    if r.output_token_ids.is_empty() {
        return Err("forward response has empty output_token_ids".to_string());
    }
    if !is_lower_hex64(&r.logits_digest) {
        return Err("forward logits_digest is not 64 lowercase hex".to_string());
    }
    if !is_lower_hex64(&r.receipt_sha256) {
        return Err("forward receipt_sha256 is not 64 lowercase hex".to_string());
    }
    let mut receipt = Sha256::new();
    for id in input_ids.iter().chain(r.output_token_ids.iter()) {
        receipt.update(id.to_le_bytes());
    }
    receipt.update(r.logits_digest.as_bytes());
    if format!("{:x}", receipt.finalize()) != r.receipt_sha256 {
        return Err(
            "forward receipt does not bind input/output token IDs and logits digest".into(),
        );
    }
    Ok(ForwardOutput {
        output_token_ids: r.output_token_ids,
        logits_digest: r.logits_digest,
        receipt_sha256: r.receipt_sha256,
    })
}

/// A model seat: attested identity, tokenize, forward, and an explicit
/// kv_page refusal. Implementations must not invent an identity, a
/// token list, or a KV page.
pub trait ModelSeat {
    fn base_url(&self) -> &str;
    fn identity(&self) -> Option<&str>;
    fn profile(&self) -> Option<&str>;
    fn probe(&mut self) -> Result<(String, String), String>;
    fn tokenize(&self, text: &str) -> Result<Vec<u32>, String>;
    fn forward(&self, input_ids: &[u32], max_tokens: usize) -> Result<ForwardOutput, String>;
    fn kv_page(&self, _address: &str) -> Result<Vec<u8>, String>;
}

pub struct GlimmerHttpSeat {
    base_url: String,
    client: reqwest::blocking::Client,
    identity: Option<String>,
    profile: Option<String>,
}

impl GlimmerHttpSeat {
    pub fn new(base_url: &str) -> Result<Self, String> {
        if !is_loopback(base_url) {
            return Err("glimmer seat URL must be loopback".to_string());
        }
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            identity: None,
            profile: None,
        })
    }

    pub fn from_env() -> Result<Self, String> {
        let url = std::env::var("ARES_NATIVE_MTP_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8011".to_string());
        Self::new(&url)
    }

    pub fn seat_identity(&self) -> Option<String> {
        self.identity.clone()
    }

    fn max_generate_tokens() -> usize {
        std::env::var("ARES_NATIVE_MTP_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| (1..=128).contains(v))
            .unwrap_or(32)
    }

    /// Blocking generate: text from /v1/generate, identity pinned to
    /// the probed identity. Returns (text, identity).
    fn generate_text(&mut self, tool: &str, args_json: &str) -> Result<(String, String), String> {
        let prompt = format!("tool: {}\nargs: {}\n", tool, args_json);
        let url = format!("{}/v1/generate", self.base_url);
        let resp = self
            .client
            .post(url)
            .timeout(Duration::from_secs(300))
            .header(
                "x-rust-max-new-tokens",
                Self::max_generate_tokens().to_string(),
            )
            .header("content-type", "text/plain")
            .body(prompt)
            .send()
            .map_err(|e| format!("glimmer generate transport: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("glimmer generate status {}", resp.status()));
        }
        let body = resp.bytes().map_err(|e| e.to_string())?;
        let frames = parse_stream(&body)?;
        let mut identity: Option<String> = None;
        let mut profile: Option<String> = None;
        let mut output = String::new();
        let mut finished = false;
        for f in frames {
            match f.kind {
                0 => {
                    identity = Some(f.model_identity);
                    profile = Some(f.profile);
                }
                1 => output.push_str(&f.text),
                2 => finished = true,
                3 => {
                    return Err(format!(
                        "glimmer generate error {}: {}",
                        f.error_code, f.text
                    ))
                }
                _ => return Err("glimmer generate unknown stream kind".to_string()),
            }
        }
        if !finished {
            return Err("glimmer generate stream ended without finish".to_string());
        }
        let identity =
            identity.ok_or_else(|| "glimmer generate stream without metadata".to_string())?;
        if let Some(pinned) = &self.identity {
            if pinned != &identity {
                return Err("glimmer model identity changed within mission".to_string());
            }
        } else {
            self.identity = Some(identity.clone());
            self.profile = profile;
        }
        Ok((output, identity))
    }

    /// One tool call through the seat: probe once, generate for text,
    /// then tokenize + forward the same prompt for the bound receipt.
    /// Forward uses 5 tokens: it is a digest attestation, not a second
    /// generation, so it does not need the full generate count.
    fn call_blocking(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        let started = Instant::now();
        if self.identity.is_none() {
            self.probe()?;
        }
        let prompt = format!("tool: {}\nargs: {}\n", tool, args_json);
        let (output, _identity) = self.generate_text(tool, args_json)?;
        let ids = self.tokenize(&prompt)?;
        let forward = self.forward(&ids, 5)?;
        Ok(CallOutcome {
            inner_text: output,
            receipt_sha256: forward.receipt_sha256,
            dt_ns: started.elapsed().as_nanos() as u64,
        })
    }
}

impl Executor for GlimmerHttpSeat {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        match tokio::runtime::Handle::try_current() {
            Ok(_) => tokio::task::block_in_place(|| self.call_blocking(tool, args_json)),
            Err(_) => self.call_blocking(tool, args_json),
        }
    }

    fn call_bound(&mut self, _tool: &str, _args_json: &str) -> Result<BoundCallOutcome, String> {
        // STUB: The Glimmer HTTP seat currently uses prose prompts + separate
        // forward call. A proper bound-output implementation requires:
        // 1. A typed binary protocol to the Glimmer server (no prose tool/args)
        // 2. Single verifiable run returning generated_token_ids, logits_digest,
        //    model_artifact, exact input bytes, tool_results, chain_hash
        // 3. Removal of the separate forward digest path
        //
        // For now, the live mission uses MindRecallExecutor which provides
        // the proper bound receipt. This stub returns an error to prevent
        // accidental use in the bound-output path.
        Err(
            "GlimmerHttpSeat::call_bound not implemented: requires typed binary protocol to Glimmer server (step 3 full implementation)".to_string()
        )
    }
}

impl ModelSeat for GlimmerHttpSeat {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn identity(&self) -> Option<&str> {
        self.identity.as_deref()
    }

    fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    fn probe(&mut self) -> Result<(String, String), String> {
        let url = format!("{}/v1/info", self.base_url);
        let resp = self
            .client
            .get(url)
            .timeout(Duration::from_millis(1500))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("glimmer probe status {}", resp.status()));
        }
        let body = resp.text().map_err(|e| e.to_string())?;
        let (identity, profile) = parse_info_body(body.as_bytes())?;
        if let Some(pinned) = &self.identity {
            if pinned != &identity {
                return Err("glimmer model identity changed within mission".to_string());
            }
        } else {
            self.identity = Some(identity.clone());
            self.profile = Some(profile.clone());
        }
        Ok((identity, profile))
    }

    fn tokenize(&self, text: &str) -> Result<Vec<u32>, String> {
        let url = format!("{}/v1/tokenize", self.base_url);
        let body_text = text.to_string();
        let resp = self
            .client
            .post(url)
            .timeout(Duration::from_secs(15))
            .header("content-type", "text/plain")
            .body(body_text)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("glimmer tokenize status {}", resp.status()));
        }
        let body = resp.text().map_err(|e| e.to_string())?;
        let ids: Vec<u32> =
            serde_json::from_str(&body).map_err(|e| format!("tokenize response json: {e}"))?;
        Ok(ids)
    }

    fn forward(&self, input_ids: &[u32], max_tokens: usize) -> Result<ForwardOutput, String> {
        let url = format!("{}/v1/forward", self.base_url);
        let body = serde_json::json!({
            "input_ids": input_ids,
            "max_tokens": max_tokens,
        });
        let resp = self
            .client
            .post(url)
            .timeout(Duration::from_secs(300))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("glimmer forward status {}", resp.status()));
        }
        let text = resp.text().map_err(|e| e.to_string())?;
        parse_forward_response(&text, input_ids)
    }

    fn kv_page(&self, _address: &str) -> Result<Vec<u8>, String> {
        Err(KV_PAGE_REFUSAL.to_string())
    }
}

#[cfg(test)]
mod receipt_tests {
    use super::*;

    #[test]
    fn forward_receipt_rejects_changed_input_or_output() {
        let input = [7_u32, 9];
        let output = [11_u32];
        let logits_digest = "a".repeat(64);
        let mut hasher = Sha256::new();
        for id in input.iter().chain(output.iter()) {
            hasher.update(id.to_le_bytes());
        }
        hasher.update(logits_digest.as_bytes());
        let receipt = format!("{:x}", hasher.finalize());
        let response = serde_json::json!({
            "output_token_ids": output,
            "logits_digest": logits_digest,
            "receipt_sha256": receipt,
        })
        .to_string();
        assert!(parse_forward_response(&response, &input).is_ok());
        assert!(parse_forward_response(&response, &[7, 10]).is_err());
        let tampered = response.replace("[11]", "[12]");
        assert!(parse_forward_response(&tampered, &input).is_err());
    }
}
