//! Native Glimmer mission executor: one bound invocation =
//! MCP `mind.recall` -> verified atom text -> binary GTK2/GTO2 tokenize ->
//! binary GFX2/GFO2 exact forward, bound into one `BoundOutputReceipt`.
//!
//! JSON appears only on the MCP control plane. The model boundary carries
//! bounded binary frames; no prose `tool/args` prompt and no separate
//! generate-plus-forward digest. This module links no Glimmer tokenizer.
//! Every artifact digest is pinned by the caller and checked against the
//! server's response, and each wire receipt is recomputed over the exact
//! request bytes before it is accepted.

use crate::harness::mind_recall_executor::MindRecallExecutor;
use crate::harness::native_pages_client::{
    decode_prefill_response, encode_continue_request, encode_prefill_request, Pages, Shape,
    MAX_PAGE_TOKENS,
};
use crate::harness::output_receipt::{
    BoundCallOutcome, BoundOutputReceipt, ModelArtifactDigest, ModelInputBytes, ToolResultDigest,
};
use crate::harness::runtime::{CallOutcome, Executor};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub const TOKEN_REQUEST_MAGIC: &[u8; 4] = b"GTK2";
pub const TOKEN_RESPONSE_MAGIC: &[u8; 4] = b"GTO2";
pub const FORWARD_REQUEST_MAGIC: &[u8; 4] = b"GFX2";
pub const FORWARD_RESPONSE_MAGIC: &[u8; 4] = b"GFO2";
pub const MAX_TEXT_BYTES: usize = 16 * 1024;
pub const MAX_TOKEN_IDS: usize = 4096;
pub const MAX_OUTPUT_TOKENS: usize = 128;

fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_hex32(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("digest is not 64 lowercase hex".into());
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn ids_le(ids: &[u32]) -> Vec<u8> {
    ids.iter().flat_map(|id| id.to_le_bytes()).collect()
}

fn le_ids(bytes: &[u8]) -> Vec<u32> {
    (0..bytes.len() / 4)
        .map(|i| u32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap()))
        .collect()
}

fn composite(weights: &[u8; 32], tokenizer: &[u8; 32], config: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"glimmer-artifact-v1\0");
    h.update(weights);
    h.update(tokenizer);
    h.update(config);
    h.finalize().into()
}

/// Caller-pinned frozen Glimmer artifact identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactPin {
    pub weights: [u8; 32],
    pub tokenizer: [u8; 32],
    pub config: [u8; 32],
    pub composite: [u8; 32],
}

impl ArtifactPin {
    pub fn from_hex(weights: &str, tokenizer: &str, config: &str) -> Result<Self, String> {
        let (weights, tokenizer, config) = (
            parse_hex32(weights)?,
            parse_hex32(tokenizer)?,
            parse_hex32(config)?,
        );
        Ok(Self {
            composite: composite(&weights, &tokenizer, &config),
            weights,
            tokenizer,
            config,
        })
    }

    fn model_artifact(&self) -> ModelArtifactDigest {
        ModelArtifactDigest {
            weights_sha256: hex(&self.weights),
            tokenizer_sha256: hex(&self.tokenizer),
            config_sha256: hex(&self.config),
            composite_sha256: hex(&self.composite),
        }
    }
}

pub fn encode_token_request(text: &str, add_bos: bool) -> Result<Vec<u8>, String> {
    let t = text.as_bytes();
    if t.is_empty() || t.len() > MAX_TEXT_BYTES {
        return Err("native tokenize text length invalid".into());
    }
    let mut out = Vec::with_capacity(9 + t.len());
    out.extend_from_slice(TOKEN_REQUEST_MAGIC);
    out.extend_from_slice(&(t.len() as u32).to_le_bytes());
    out.push(u8::from(add_bos));
    out.extend_from_slice(t);
    Ok(out)
}

pub struct TokenResponse {
    pub token_ids: Vec<u32>,
    pub tokenizer: [u8; 32],
    pub receipt: [u8; 32],
}

pub fn decode_token_response(request: &[u8], bytes: &[u8]) -> Result<TokenResponse, String> {
    if bytes.len() < 76 || &bytes[..4] != TOKEN_RESPONSE_MAGIC {
        return Err("native tokenize response header invalid".into());
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if count == 0 || count > MAX_TOKEN_IDS || bytes.len() != 72 + count * 4 {
        return Err("native tokenize response length invalid".into());
    }
    let split = bytes.len() - 32;
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes[..split]);
    if sha(&bound) != bytes[split..] {
        return Err("native tokenize receipt mismatch".into());
    }
    Ok(TokenResponse {
        token_ids: le_ids(&bytes[8..8 + count * 4]),
        tokenizer: bytes[8 + count * 4..40 + count * 4].try_into().unwrap(),
        receipt: bytes[split..].try_into().unwrap(),
    })
}

pub fn encode_forward_request(input_ids: &[u32], max_tokens: usize) -> Result<Vec<u8>, String> {
    if input_ids.is_empty()
        || input_ids.len() > MAX_TOKEN_IDS
        || !(1..=MAX_OUTPUT_TOKENS).contains(&max_tokens)
    {
        return Err("exact-forward request bounds violated".into());
    }
    let mut out = Vec::with_capacity(12 + input_ids.len() * 4);
    out.extend_from_slice(FORWARD_REQUEST_MAGIC);
    out.extend_from_slice(&(input_ids.len() as u32).to_le_bytes());
    out.extend_from_slice(&(max_tokens as u32).to_le_bytes());
    out.extend_from_slice(&ids_le(input_ids));
    Ok(out)
}

pub struct ForwardResponse {
    pub output_ids: Vec<u32>,
    pub logits: [u8; 32],
    pub kv: [u8; 32],
    pub weights: [u8; 32],
    pub tokenizer: [u8; 32],
    pub config: [u8; 32],
    pub composite: [u8; 32],
    pub receipt: [u8; 32],
}

pub fn decode_forward_response(request: &[u8], bytes: &[u8]) -> Result<ForwardResponse, String> {
    if bytes.len() < 232 || &bytes[..4] != FORWARD_RESPONSE_MAGIC {
        return Err("exact-forward response header invalid".into());
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if !(1..=MAX_OUTPUT_TOKENS).contains(&count) || bytes.len() != 232 + count * 4 {
        return Err("exact-forward response length invalid".into());
    }
    let split = bytes.len() - 32;
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes[..split]);
    if sha(&bound) != bytes[split..] {
        return Err("exact-forward response receipt mismatch".into());
    }
    let mut at = 8 + count * 4;
    let mut take = || {
        let v: [u8; 32] = bytes[at..at + 32].try_into().unwrap();
        at += 32;
        v
    };
    let (logits, kv, weights, tokenizer, config, comp, receipt) =
        (take(), take(), take(), take(), take(), take(), take());
    if composite(&weights, &tokenizer, &config) != comp {
        return Err("exact-forward artifact composite mismatch".into());
    }
    Ok(ForwardResponse {
        output_ids: le_ids(&bytes[8..8 + count * 4]),
        logits,
        kv,
        weights,
        tokenizer,
        config,
        composite: comp,
        receipt,
    })
}

/// Join the ranked, digest-verified `content_text` of a `mind.recall`
/// control response. Each member's text must hash to its `content_digest`
/// again here; members with no inline text are ignored (metadata only).
pub fn verified_recall_text(recall_json: &str) -> Result<String, String> {
    let body: serde_json::Value =
        serde_json::from_str(recall_json).map_err(|e| format!("recall control json: {e}"))?;
    let members = body
        .get("members")
        .and_then(|m| m.as_array())
        .ok_or("recall response has no members array")?;
    let mut parts = Vec::new();
    for m in members {
        let Some(text) = m.get("content_text").and_then(|t| t.as_str()) else {
            continue;
        };
        let digest = m
            .get("content_digest")
            .and_then(|d| d.as_str())
            .ok_or("recall member has text but no content_digest")?;
        if hex(&sha(text.as_bytes())) != digest {
            return Err("recall member content fails SHA-256 verification".into());
        }
        parts.push(text);
    }
    if parts.is_empty() {
        return Err("recall returned no verified inline content to present to the model".into());
    }
    let joined = parts.join("\n\n");
    if joined.len() > MAX_TEXT_BYTES {
        return Err("verified recall text exceeds the model input bound".into());
    }
    Ok(joined)
}

/// Identity of a persisted prefix: the pinned model composite plus the exact
/// prefix token IDs. A store maps it to its own full exact-K/V identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvKey {
    pub artifact_composite: [u8; 32],
    pub prefix_ids: Vec<u32>,
}

/// Exact K/V page persistence. `fetch` must return pages only when every
/// layer/head page matches the stored identity (model, shape, TPPE and address
/// contract, page digest); otherwise `None` or an error — never a partial set.
pub trait KvPageStore: Send {
    fn fetch(&mut self, key: &KvKey) -> Result<Option<(Shape, Pages)>, String>;
    fn store(&mut self, key: &KvKey, shape: &Shape, pages: &Pages) -> Result<(), String>;
}

fn pages_digest(pages: &Pages) -> [u8; 32] {
    let mut h = Sha256::new();
    for (k, v) in pages {
        for x in k.iter().chain(v) {
            h.update(x.to_le_bytes());
        }
    }
    h.finalize().into()
}

pub struct NativeGlimmerExecutor {
    kv: Option<Box<dyn KvPageStore>>,
    mind: MindRecallExecutor,
    base_url: String,
    pin: ArtifactPin,
    max_tokens: usize,
    client: reqwest::blocking::Client,
}

fn check_hit_shape(shape: &Shape, prefix_len: usize) -> Result<(), String> {
    if shape.tokens != prefix_len {
        return Err("stored K/V pages cover a different prefix length".into());
    }
    Ok(())
}

fn is_loopback(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:")
}

impl NativeGlimmerExecutor {
    pub fn new(
        mind: MindRecallExecutor,
        base_url: &str,
        pin: ArtifactPin,
        max_tokens: usize,
    ) -> Result<Self, String> {
        if !is_loopback(base_url) {
            return Err("native glimmer URL must be loopback".into());
        }
        if !(1..=MAX_OUTPUT_TOKENS).contains(&max_tokens) {
            return Err("max_tokens out of range".into());
        }
        Ok(Self {
            kv: None,
            mind,
            base_url: base_url.trim_end_matches('/').to_string(),
            pin,
            max_tokens,
            client: reqwest::blocking::Client::builder()
                .build()
                .map_err(|e| e.to_string())?,
        })
    }

    /// Requires `ARES_GLIMMER_NATIVE_URL` and `ARES_GLIMMER_WEIGHTS_SHA256`;
    /// tokenizer and config digests default to the frozen checkpoint pins.
    pub fn from_env(mind: MindRecallExecutor) -> Result<Self, String> {
        let url = std::env::var("ARES_GLIMMER_NATIVE_URL")
            .map_err(|_| "ARES_GLIMMER_NATIVE_URL required".to_string())?;
        let weights = std::env::var("ARES_GLIMMER_WEIGHTS_SHA256")
            .map_err(|_| "ARES_GLIMMER_WEIGHTS_SHA256 required".to_string())?;
        let tokenizer = std::env::var("ARES_GLIMMER_TOKENIZER_SHA256").unwrap_or_else(|_| {
            "c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d".into()
        });
        let config = std::env::var("ARES_GLIMMER_CONFIG_SHA256").unwrap_or_else(|_| {
            "3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031".into()
        });
        let max_tokens = std::env::var("ARES_GLIMMER_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8);
        Self::new(
            mind,
            &url,
            ArtifactPin::from_hex(&weights, &tokenizer, &config)?,
            max_tokens,
        )
    }

    /// Route prefix-eligible calls through exact K/V persistence: a hit
    /// hydrates stored pages and continues; a miss prefills, persists, then
    /// continues from the exported pages.
    pub fn with_kv_store(mut self, store: Box<dyn KvPageStore>) -> Self {
        self.kv = Some(store);
        self
    }

    pub fn seat_identity(&self) -> String {
        hex(&self.pin.composite)
    }

    fn post(&self, path: &str, body: &[u8], content_type: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .client
            .post(format!("{}{path}", self.base_url))
            .timeout(Duration::from_secs(300))
            .header("content-type", content_type)
            .body(body.to_vec())
            .send()
            .map_err(|e| format!("glimmer {path} transport: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("glimmer {path} status {}", resp.status()));
        }
        Ok(resp.bytes().map_err(|e| e.to_string())?.to_vec())
    }

    fn call_blocking(&mut self, tool: &str, args_json: &str) -> Result<BoundCallOutcome, String> {
        let started = Instant::now();
        let mcp = self.mind.call(tool, args_json)?;
        let text = verified_recall_text(&mcp.inner_text)?;

        let token_req = encode_token_request(&text, true)?;
        let token_bytes = self.post(
            "/v1/tokenize-native",
            &token_req,
            "application/vnd.glimmer.tokenize-v1",
        )?;
        let tokens = decode_token_response(&token_req, &token_bytes)?;
        if tokens.tokenizer != self.pin.tokenizer {
            return Err("tokenizer digest differs from pinned artifact".into());
        }

        let ids = &tokens.token_ids;
        let mut tool_results = vec![
            ToolResultDigest {
                tool: tool.to_string(),
                args_sha256: hex(&sha(args_json.as_bytes())),
                result_sha256: hex(&sha(mcp.inner_text.as_bytes())),
                receipt_sha256: mcp.receipt_sha256.clone(),
            },
            ToolResultDigest {
                tool: "native.tokenize".into(),
                args_sha256: hex(&sha(&token_req)),
                result_sha256: hex(&sha(&ids_le(ids))),
                receipt_sha256: hex(&tokens.receipt),
            },
        ];
        let kv_eligible = self.kv.is_some() && ids.len() >= 2 && ids.len() - 1 <= MAX_PAGE_TOKENS;
        let (fwd_req, fwd_name, fwd) = if kv_eligible {
            let (prefix, rest) = ids.split_at(ids.len() - 1);
            let key = KvKey {
                artifact_composite: self.pin.composite,
                prefix_ids: prefix.to_vec(),
            };
            let key_digest = hex(&sha(
                &[&key.artifact_composite[..], &ids_le(prefix)].concat()
            ));
            let cached = self.kv.as_mut().unwrap().fetch(&key)?;
            let (shape, pages) = match cached {
                Some(hit) => {
                    check_hit_shape(&hit.0, prefix.len())?;
                    let pd = hex(&pages_digest(&hit.1));
                    tool_results.push(ToolResultDigest {
                        tool: "native.kv-hit".into(),
                        args_sha256: key_digest.clone(),
                        result_sha256: pd.clone(),
                        receipt_sha256: hex(&sha(format!("{key_digest}{pd}").as_bytes())),
                    });
                    hit
                }
                None => {
                    let preq = encode_prefill_request(prefix)?;
                    let pbytes = self.post(
                        "/v1/prefill-exact",
                        &preq,
                        "application/vnd.glimmer.prefill-v1",
                    )?;
                    let pre = decode_prefill_response(&preq, &pbytes)?;
                    if (pre.weights, pre.tokenizer, pre.config, pre.composite)
                        != (
                            self.pin.weights,
                            self.pin.tokenizer,
                            self.pin.config,
                            self.pin.composite,
                        )
                    {
                        return Err("prefill artifact digests differ from pinned artifact".into());
                    }
                    check_hit_shape(&pre.shape, prefix.len())?;
                    self.kv
                        .as_mut()
                        .unwrap()
                        .store(&key, &pre.shape, &pre.pages)?;
                    tool_results.push(ToolResultDigest {
                        tool: "native.kv-prefill".into(),
                        args_sha256: hex(&sha(&preq)),
                        result_sha256: hex(&pages_digest(&pre.pages)),
                        receipt_sha256: hex(&pre.receipt),
                    });
                    (pre.shape, pre.pages)
                }
            };
            let creq = encode_continue_request(&shape, &pages, rest, self.max_tokens)?;
            let cbytes = self.post(
                "/v1/continue-exact",
                &creq,
                "application/vnd.glimmer.continue-v1",
            )?;
            let fwd = decode_forward_response(&creq, &cbytes)?;
            (creq, "native.continue-exact", fwd)
        } else {
            let freq = encode_forward_request(ids, self.max_tokens)?;
            let fbytes = self.post(
                "/v1/forward-exact",
                &freq,
                "application/vnd.glimmer.forward-exact-v1",
            )?;
            let fwd = decode_forward_response(&freq, &fbytes)?;
            (freq, "native.forward-exact", fwd)
        };
        if (fwd.weights, fwd.tokenizer, fwd.config, fwd.composite)
            != (
                self.pin.weights,
                self.pin.tokenizer,
                self.pin.config,
                self.pin.composite,
            )
        {
            return Err("forward artifact digests differ from pinned artifact".into());
        }
        tool_results.push(ToolResultDigest {
            tool: fwd_name.into(),
            args_sha256: hex(&sha(&fwd_req)),
            result_sha256: hex(&sha(&ids_le(&fwd.output_ids))),
            receipt_sha256: hex(&fwd.receipt),
        });
        let mut receipt = BoundOutputReceipt {
            generated_token_ids: fwd.output_ids,
            logits_digest: hex(&fwd.logits),
            kv_digest: hex(&fwd.kv),
            model_artifact: self.pin.model_artifact(),
            input_bytes: ModelInputBytes::from_utf8(&text),
            tool_results,
            chain_hash: String::new(),
            dt_ns: started.elapsed().as_nanos() as u64,
        };
        receipt.chain_hash = BoundOutputReceipt::compute_chain_hash(None, &receipt);
        receipt.validate()?;
        Ok(BoundCallOutcome {
            // Tool evidence only: no detokenizer route exists yet, so the
            // generated tokens are not rendered as text here.
            inner_text: mcp.inner_text,
            bound_receipt: receipt,
        })
    }
}

impl Executor for NativeGlimmerExecutor {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        let bound = self.call_bound(tool, args_json)?;
        Ok(CallOutcome {
            inner_text: bound.inner_text,
            receipt_sha256: bound.bound_receipt.chain_hash,
            dt_ns: bound.bound_receipt.dt_ns,
        })
    }

    fn call_bound(&mut self, tool: &str, args_json: &str) -> Result<BoundCallOutcome, String> {
        match tokio::runtime::Handle::try_current() {
            Ok(_) => tokio::task::block_in_place(|| self.call_blocking(tool, args_json)),
            Err(_) => self.call_blocking(tool, args_json),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recall(text: &str, digest: &str) -> String {
        serde_json::json!({"members": [{"content_text": text, "content_digest": digest}]})
            .to_string()
    }

    #[test]
    fn recall_text_requires_matching_digest_and_content() {
        let good = hex(&sha(b"atom"));
        assert_eq!(
            verified_recall_text(&recall("atom", &good)).unwrap(),
            "atom"
        );
        assert!(verified_recall_text(&recall("atom!", &good)).is_err());
        assert!(verified_recall_text(&serde_json::json!({"members": []}).to_string()).is_err());
        assert!(verified_recall_text(
            &serde_json::json!({"members": [{"content_digest": good}]}).to_string()
        )
        .is_err());
    }

    #[test]
    fn token_response_rejects_other_request_and_tampering() {
        let req = encode_token_request("hi", true).unwrap();
        let mut body = Vec::new();
        body.extend_from_slice(TOKEN_RESPONSE_MAGIC);
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&7u32.to_le_bytes());
        body.extend_from_slice(&[9u8; 32]);
        let mut bound = req.clone();
        bound.extend_from_slice(&body);
        body.extend_from_slice(&sha(&bound));
        assert_eq!(decode_token_response(&req, &body).unwrap().token_ids, [7]);
        assert!(decode_token_response(b"other", &body).is_err());
        body[8] ^= 1;
        assert!(decode_token_response(&req, &body).is_err());
    }

    #[test]
    fn forward_response_rejects_false_composite() {
        let req = encode_forward_request(&[1], 1).unwrap();
        let pin = ArtifactPin::from_hex(&"a".repeat(64), &"b".repeat(64), &"c".repeat(64)).unwrap();
        let build = |comp: [u8; 32]| {
            let mut body = Vec::new();
            body.extend_from_slice(FORWARD_RESPONSE_MAGIC);
            body.extend_from_slice(&1u32.to_le_bytes());
            body.extend_from_slice(&5u32.to_le_bytes());
            for d in [
                [1u8; 32],
                [2u8; 32],
                pin.weights,
                pin.tokenizer,
                pin.config,
                comp,
            ] {
                body.extend_from_slice(&d);
            }
            let mut bound = req.clone();
            bound.extend_from_slice(&body);
            body.extend_from_slice(&sha(&bound));
            body
        };
        let ok = decode_forward_response(&req, &build(pin.composite)).unwrap();
        assert_eq!(ok.output_ids, [5]);
        let mut wrong = pin.composite;
        wrong[0] ^= 1;
        assert!(decode_forward_response(&req, &build(wrong)).is_err());
    }

    #[test]
    fn pin_rejects_non_hex() {
        assert!(ArtifactPin::from_hex("xyz", &"b".repeat(64), &"c".repeat(64)).is_err());
    }
}
