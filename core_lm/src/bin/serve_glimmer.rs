//! serve_glimmer — Glimmer HTTP seat, reconstructed 2026-11-03.
//!
//! The original serve_glimmer.rs, amst.rs, and sha256.rs were lost when
//! the live core_lm/src tree was replaced on 2026-11-03 at 22:52.
//! This file is a reconstruction from: the restored Nov 1 core_lm library
//! (glimmer_forward, glimmer_model, tokenizer), the backend AMST parser
//! (native_mtp.rs, stream_wire::Frame layout), and strings recovered
//! from the surviving release binary (endpoint names, error strings,
//! forward response field names). It is NOT claimed byte-identical to
//! the lost original. Differences are documented where they matter.
//!
//! Endpoints, all loopback only, sequential (one request at a time,
//! because the model is &mut and its geometric memory must not interleave):
//! - GET /health -> "ready" (plain text, only after model load; this
//!   binary loads the model before it starts listening, so it is always ready)
//! - GET /v1/info -> one AMST kind-0 frame, content-type
//!   application/vnd.rust-server.amst; version=1, identity = seat,
//!   profile = "glimmer"
//! - POST /v1/tokenize, JSON {"text": "..."} -> JSON array of token ids
//!   (encode only, no BOS inserted, matching the old tokenize output)
//! - POST /v1/tokenize-native, binary GTK2 body -> GTO2 token IDs plus the
//!   pinned tokenizer digest and a request-bound receipt. Model-owned
//!   tokenizer only; add_bos prepends config.bos_token_id.
//! - POST /v1/prefill-exact, binary GPF2 -> GPO2 with every exact K/V page of a
//!   short prefix; POST /v1/continue-exact, binary GCX2 (pages + continuation
//!   ids) -> GFO2. Shape mismatch is refused; the prefix is never recomputed.
//! - POST /v1/forward, JSON {"token_ids": [...], "max_tokens": N}
//!   (also accepts "input_ids" for the token list) -> JSON
//!   {"output_token_ids": [...], "logits_digest": "...", "receipt_sha256": "..."}
//!   Greedy argmax, EOS stop, max_tokens 1..=128.
//! - POST /v1/generate, text/plain body = prompt, headers
//!   x-rust-max-new-tokens (1..=128, default 32) -> AMST stream:
//!   kind-0 metadata, kind-1 token frames (text per token), kind-2 finish
//!   (stop 2 = EOS, 1 = max_tokens). Empty generation (first token EOS)
//!   is HTTP 500, because AMST cannot express 0 outputs.
//!
//! STATE-RESET FIX (the reason this file exists): every forward and
//! generate request calls model.reset_memory() BEFORE prefill, so no
//! geometric memory carries across requests. The old binary did not
//! reset, and identical forwards gave different tokens and digests
//! (verified 2026-11-03 ~22:35). After this fix, two identical forward
//! requests must return identical tokens AND identical digests.
//!
//! Receipt: receipt_sha256 = SHA-256 over
//!   input_ids (u32 LE bytes) || output_ids (u32 LE bytes) ||
//!   logits_digest (ASCII hex bytes).
//! logits_digest = SHA-256 over final logits (f32 LE bytes).
//! Seat: SHA-256 over config.json bytes + 0x00 + tokenizer.json bytes.
//!
//! Usage: serve_glimmer [127.0.0.1:8011] [--model-dir DIR]
//! Default model dir: $HOME/models/Muse-Glimmer-q4km-gs128-MLX

#[path = "../sha256.rs"]
mod sha256;
#[path = "../amst.rs"]
mod amst;

use core_lm::glimmer_forward::GlimmerModel;
use core_lm::exact_kv::ExactKvCache;
use core_lm::artifact_digest::ArtifactDigest;
use core_lm::native_exact_wire::{
    decode_request, decode_verify_request, encode_response, exact_kv_digest,
};
use core_lm::native_pages_wire::{
    decode_continue_request, decode_prefill_request, encode_prefill_response,
    encode_session_response, Shape,
};
use core_lm::native_token_wire::{
    decode_detok_request, decode_request as decode_token_request, encode_detok_response,
    encode_response as encode_token_response,
};
use core_lm::ops::argmax;
use core_lm::tokenizer::BpeTokenizer;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

fn seat_identity(model_dir: &PathBuf) -> Result<String, String> {
    let cfg = std::fs::read(model_dir.join("config.json")).map_err(|e| e.to_string())?;
    let tok = std::fs::read(model_dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
    let mut data = cfg;
    data.push(0u8);
    data.extend_from_slice(&tok);
    Ok(sha256::sha256_hex(&data))
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

struct GenOut {
    output_ids: Vec<u32>,
    final_logits: Vec<f32>,
    eos: bool,
}

fn generate(
    model: &mut GlimmerModel,
    input_ids: &[u32],
    max_tokens: usize,
) -> Result<GenOut, String> {
    if input_ids.is_empty() {
        return Err("Empty input".to_string());
    }
    if !(1..=128).contains(&max_tokens) {
        return Err("max_tokens must be 1..=128".to_string());
    }
    // STATE RESET: fresh geometric memory for every request.
    model.reset_memory();
    let eos_ids = model.config.eos_token_ids.clone();
    let mut last_logits: Vec<f32> = Vec::new();
    for pos in 0..input_ids.len() {
        last_logits = model
            .forward(input_ids[pos], pos)
            .map_err(|e| format!("forward prefill: {e}"))?;
    }
    let first = argmax(&last_logits).map_err(|e| format!("argmax: {e}"))? as u32;
    let mut output_ids = vec![first];
    let mut final_logits = last_logits;
    let mut eos = eos_ids.contains(&first);
    let mut i = 1usize;
    while !eos && i < max_tokens {
        let pos = input_ids.len() + i - 1;
        let prev = output_ids[i - 1];
        let logits = model
            .forward(prev, pos)
            .map_err(|e| format!("forward gen: {e}"))?;
        let next = argmax(&logits).map_err(|e| format!("argmax: {e}"))? as u32;
        final_logits = logits;
        output_ids.push(next);
        eos = eos_ids.contains(&next);
        i += 1;
    }
    Ok(GenOut {
        output_ids,
        final_logits,
        eos,
    })
}

fn logits_digest_hex(logits: &[f32]) -> String {
    let bytes: Vec<u8> = logits.iter().flat_map(|v| v.to_le_bytes()).collect();
    sha256::sha256_hex(&bytes)
}

fn receipt_hex(input_ids: &[u32], output_ids: &[u32], logits_digest: &str) -> String {
    let mut data = Vec::new();
    for id in input_ids {
        data.extend_from_slice(&id.to_le_bytes());
    }
    for id in output_ids {
        data.extend_from_slice(&id.to_le_bytes());
    }
    data.extend_from_slice(logits_digest.as_bytes());
    sha256::sha256_hex(&data)
}

fn handle_tokenize_native(
    body: &[u8],
    tok: &BpeTokenizer,
    bos_token_id: u32,
    tokenizer_digest: [u8; 32],
) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let request = decode_token_request(body)?;
        let mut token_ids = tok.encode(&request.text).map_err(|e| e.to_string())?;
        if request.add_bos {
            token_ids.insert(0, bos_token_id);
        }
        encode_token_response(body, &token_ids, tokenizer_digest)
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.tokenize-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

fn handle_detokenize(body: &[u8], tok: &BpeTokenizer) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let ids = decode_detok_request(body)?;
        encode_detok_response(body, &tok.decode(&ids))
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.detokenize-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

fn handle_exact(body: &[u8], model: &mut GlimmerModel, artifact: ArtifactDigest) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let request = decode_request(body)?;
        let mut cache = ExactKvCache::new(
            model.config.num_hidden_layers,
            model.config.num_attention_heads,
            model.config.num_key_value_heads,
            model.config.head_dim,
        )?;
        model.reset_memory();
        let mut logits = Vec::new();
        for (position, &id) in request.input_ids.iter().enumerate() {
            logits = model
                .forward_with_backend(id, position, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let mut output = Vec::new();
        loop {
            let next = argmax(&logits).map_err(|e| e.to_string())? as u32;
            output.push(next);
            if output.len() >= request.max_tokens || model.config.eos_token_ids.contains(&next) {
                break;
            }
            let position = request.input_ids.len() + output.len() - 1;
            logits = model
                .forward_with_backend(next, position, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let mut logits_bytes = Vec::with_capacity(logits.len() * 4);
        for value in logits {
            logits_bytes.extend_from_slice(&value.to_le_bytes());
        }
        let kv_digest = exact_kv_digest(
            &cache,
            model.config.num_hidden_layers,
            model.config.num_key_value_heads,
            request.input_ids.len() + output.len() - 1,
        )?;
        encode_response(
            body,
            &output,
            sha256::sha256(&logits_bytes),
            kv_digest,
            artifact,
        )
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.exact-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

fn handle_prefill(body: &[u8], model: &mut GlimmerModel, artifact: ArtifactDigest) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let ids = decode_prefill_request(body)?;
        let c = &model.config;
        let (layers, kv_heads, head_dim) =
            (c.num_hidden_layers, c.num_key_value_heads, c.head_dim);
        let mut cache = ExactKvCache::new(layers, c.num_attention_heads, kv_heads, head_dim)?;
        model.reset_memory();
        for (position, &id) in ids.iter().enumerate() {
            model
                .forward_with_backend(id, position, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        cache.verify_complete_prefix(ids.len())?;
        let mut pages = Vec::with_capacity(layers * kv_heads);
        for layer in 0..layers {
            for head in 0..kv_heads {
                let (k, v) = cache.head_page(layer, head).ok_or("missing K/V page")?;
                pages.push((k.to_vec(), v.to_vec()));
            }
        }
        let kv_digest = exact_kv_digest(&cache, layers, kv_heads, ids.len())?;
        let shape = Shape {
            layers,
            kv_heads,
            head_dim,
            tokens: ids.len(),
        };
        encode_prefill_response(body, &shape, &pages, kv_digest, artifact)
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.prefill-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

/// Continue from caller-supplied exact pages. The shape must equal the
/// loaded model's shape; there is no fallback to recomputing the prefix.
fn handle_continue(body: &[u8], model: &mut GlimmerModel, artifact: ArtifactDigest) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let request = decode_continue_request(body)?;
        let c = &model.config;
        let (layers, kv_heads, head_dim) =
            (c.num_hidden_layers, c.num_key_value_heads, c.head_dim);
        if (request.shape.layers, request.shape.kv_heads, request.shape.head_dim)
            != (layers, kv_heads, head_dim)
        {
            return Err("page shape differs from loaded model".into());
        }
        let prefix = request.shape.tokens;
        let mut cache = ExactKvCache::from_head_pages(
            layers,
            c.num_attention_heads,
            kv_heads,
            head_dim,
            prefix,
            request.pages,
        )?;
        model.reset_memory();
        let mut logits = Vec::new();
        for (offset, &id) in request.continuation_ids.iter().enumerate() {
            logits = model
                .forward_with_backend(id, prefix + offset, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let input_len = prefix + request.continuation_ids.len();
        let mut output = Vec::new();
        loop {
            let next = argmax(&logits).map_err(|e| e.to_string())? as u32;
            output.push(next);
            if output.len() >= request.max_tokens || model.config.eos_token_ids.contains(&next) {
                break;
            }
            logits = model
                .forward_with_backend(next, input_len + output.len() - 1, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let mut logits_bytes = Vec::with_capacity(logits.len() * 4);
        for value in logits {
            logits_bytes.extend_from_slice(&value.to_le_bytes());
        }
        let kv_digest = exact_kv_digest(
            &cache,
            model.config.num_hidden_layers,
            model.config.num_key_value_heads,
            input_len + output.len() - 1,
        )?;
        encode_response(body, &output, sha256::sha256(&logits_bytes), kv_digest, artifact)
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.exact-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

/// Speculative verify: predict after the context, then after each draft
/// prefix, stopping at the first disagreement (that prediction is the
/// correction token). With no draft this is one plain greedy step.
fn handle_verify(body: &[u8], model: &mut GlimmerModel, artifact: ArtifactDigest) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let request = decode_verify_request(body)?;
        let mut cache = ExactKvCache::new(
            model.config.num_hidden_layers,
            model.config.num_attention_heads,
            model.config.num_key_value_heads,
            model.config.head_dim,
        )?;
        model.reset_memory();
        let mut logits = Vec::new();
        for (position, &id) in request.context_ids.iter().enumerate() {
            logits = model
                .forward_with_backend(id, position, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let mut consumed = request.context_ids.len();
        let mut preds = Vec::new();
        for step in 0..=request.draft_ids.len() {
            let predicted = argmax(&logits).map_err(|e| e.to_string())? as u32;
            preds.push(predicted);
            if step == request.draft_ids.len() || predicted != request.draft_ids[step] {
                break;
            }
            logits = model
                .forward_with_backend(request.draft_ids[step], consumed, &mut cache)
                .map_err(|e| e.to_string())?;
            consumed += 1;
        }
        let mut logits_bytes = Vec::with_capacity(logits.len() * 4);
        for value in logits {
            logits_bytes.extend_from_slice(&value.to_le_bytes());
        }
        let kv_digest = exact_kv_digest(
            &cache,
            model.config.num_hidden_layers,
            model.config.num_key_value_heads,
            consumed,
        )?;
        encode_response(body, &preds, sha256::sha256(&logits_bytes), kv_digest, artifact)
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.exact-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

/// Continue from pages and return the generated tokens PLUS the updated pages
/// (all tokens now in the cache), so the caller keeps the state as tensors.
fn handle_continue_session(
    body: &[u8],
    model: &mut GlimmerModel,
    artifact: ArtifactDigest,
) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let request = decode_continue_request(body)?;
        let c = &model.config;
        let (layers, kv_heads, head_dim) =
            (c.num_hidden_layers, c.num_key_value_heads, c.head_dim);
        if (request.shape.layers, request.shape.kv_heads, request.shape.head_dim)
            != (layers, kv_heads, head_dim)
        {
            return Err("page shape differs from loaded model".into());
        }
        let prefix = request.shape.tokens;
        let mut cache = ExactKvCache::from_head_pages(
            layers,
            c.num_attention_heads,
            kv_heads,
            head_dim,
            prefix,
            request.pages,
        )?;
        model.reset_memory();
        let mut logits = Vec::new();
        for (offset, &id) in request.continuation_ids.iter().enumerate() {
            logits = model
                .forward_with_backend(id, prefix + offset, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let input_len = prefix + request.continuation_ids.len();
        let mut output = Vec::new();
        loop {
            let next = argmax(&logits).map_err(|e| e.to_string())? as u32;
            output.push(next);
            if output.len() >= request.max_tokens || model.config.eos_token_ids.contains(&next) {
                break;
            }
            logits = model
                .forward_with_backend(next, input_len + output.len() - 1, &mut cache)
                .map_err(|e| e.to_string())?;
        }
        let mut logits_bytes = Vec::with_capacity(logits.len() * 4);
        for value in logits {
            logits_bytes.extend_from_slice(&value.to_le_bytes());
        }
        let tokens = input_len + output.len() - 1;
        let kv_digest = exact_kv_digest(&cache, layers, kv_heads, tokens)?;
        let mut pages = Vec::with_capacity(layers * kv_heads);
        for layer in 0..layers {
            for head in 0..kv_heads {
                let (k, v) = cache.head_page(layer, head).ok_or("missing K/V page")?;
                pages.push((k.to_vec(), v.to_vec()));
            }
        }
        let shape = Shape {
            layers,
            kv_heads,
            head_dim,
            tokens,
        };
        encode_session_response(
            body,
            &output,
            &shape,
            &pages,
            sha256::sha256(&logits_bytes),
            kv_digest,
            artifact,
        )
    })();
    match result {
        Ok(bytes) => http_response("200 OK", "application/vnd.glimmer.session-v1", &bytes),
        Err(error) => http_response("400 Bad Request", "text/plain", error.as_bytes()),
    }
}

fn http_response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

fn json_error(msg: &str) -> Vec<u8> {
    let body = format!("{{\"error\": \"{}\"}}", json_escape(msg));
    http_response("400 Bad Request", "application/json", body.as_bytes())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut bind = "127.0.0.1:8011".to_string();
    let mut model_dir: Option<String> = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--model-dir" => {
                i += 1;
                model_dir = Some(args.get(i).ok_or("--model-dir needs a value")?.clone());
            }
            "--help" => {
                println!("Usage: serve_glimmer [127.0.0.1:8011] [--model-dir DIR]");
                return Ok(());
            }
            other => {
                if other.starts_with("--") {
                    return Err(format!("unknown flag: {other}").into());
                }
                bind = other.to_string();
            }
        }
        i += 1;
    }
    let host = bind.split(':').next().unwrap_or("");
    if host != "127.0.0.1" && host != "localhost" {
        return Err("serve_glimmer is loopback only".into());
    }
    let model_dir = match model_dir {
        Some(d) => PathBuf::from(d),
        None => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".to_string());
            PathBuf::from(home).join("models/Muse-Glimmer-q4km-gs128-MLX")
        }
    };
    println!("Loading tokenizer from {}", model_dir.join("tokenizer.json").display());
    let tok = BpeTokenizer::from_json_file(&model_dir.join("tokenizer.json"))?;
    let seat = seat_identity(&model_dir)?;
    let artifact = ArtifactDigest::from_dir(&model_dir)?;
    println!("Seat: {seat}");
    println!("Loading Glimmer model (this takes a while) ...");
    let mut model = GlimmerModel::load(&model_dir)?;
    if ArtifactDigest::from_dir(&model_dir)? != artifact {
        return Err("Glimmer artifact changed during model load".into());
    }
    model.weights.retain_cache = false;
    println!(
        "Model loaded (q_fused={}, layers={}, hidden={})",
        model.q_fused, model.config.num_hidden_layers, model.config.hidden_size
    );
    let listener = TcpListener::bind(&bind)?;
    println!("serve_glimmer listening on {bind}");
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept: {e}");
                continue;
            }
        };
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        // Read headers, then body per Content-Length.
        let mut header_end: Option<usize> = None;
        while header_end.is_none() {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                        header_end = Some(pos + 4);
                    }
                    if buf.len() > 1_048_576 {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("read: {e}");
                    break;
                }
            }
        }
        let Some(hend) = header_end else {
            continue;
        };
        let header_text = String::from_utf8_lossy(&buf[..hend]).to_string();
        let mut lines = header_text.lines();
        let request_line = lines.next().unwrap_or("");
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");
        let mut content_length: usize = 0;
        let mut max_new_tokens: Option<usize> = None;
        for line in lines {
            let lower = line.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            }
            if let Some(v) = lower.strip_prefix("x-rust-max-new-tokens:") {
                max_new_tokens = v.trim().parse().ok();
            }
        }
        if content_length > 64 * 1024 * 1024 {
            let resp = http_response("413 Payload Too Large", "text/plain", b"request too large");
            let _ = stream.write_all(&resp);
            continue;
        }
        while buf.len() < hend + content_length {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        let body = &buf[hend..std::cmp::min(buf.len(), hend + content_length)];
        let resp = if method == "POST" && path == "/v1/forward-exact" {
            handle_exact(body, &mut model, artifact)
        } else if method == "POST" && path == "/v1/continue-session" {
            handle_continue_session(body, &mut model, artifact)
        } else if method == "POST" && path == "/v1/detokenize-native" {
            handle_detokenize(body, &tok)
        } else if method == "POST" && path == "/v1/verify-exact" {
            handle_verify(body, &mut model, artifact)
        } else if method == "POST" && path == "/v1/prefill-exact" {
            handle_prefill(body, &mut model, artifact)
        } else if method == "POST" && path == "/v1/continue-exact" {
            handle_continue(body, &mut model, artifact)
        } else if method == "POST" && path == "/v1/tokenize-native" {
            handle_tokenize_native(body, &tok, model.config.bos_token_id, artifact.tokenizer)
        } else {
            let body_text = String::from_utf8_lossy(body);
            handle(&method, &path, &body_text, max_new_tokens, &mut model, &tok, &seat)
        };
        if let Err(e) = stream.write_all(&resp) {
            eprintln!("write: {e}");
        }
    }
    Ok(())
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn parse_json_u32_array(text: &str, key: &str) -> Option<Vec<u32>> {
    // Minimal JSON: find "key": [ ... ] and parse integers inside.
    let pat = format!("\"{key}\"");
    let start = text.find(&pat)?;
    let after = &text[start + pat.len()..];
    let bracket = after.find('[')?;
    let inner = &after[bracket + 1..];
    let end = inner.find(']')?;
    let inner = &inner[..end];
    let mut out = Vec::new();
    for piece in inner.split(',') {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        out.push(piece.parse::<u32>().ok()?);
    }
    Some(out)
}

fn parse_json_usize(text: &str, key: &str) -> Option<usize> {
    let pat = format!("\"{key}\"");
    let start = text.find(&pat)?;
    let after = &text[start + pat.len()..];
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    let mut digits = String::new();
    for c in rest.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            break;
        }
    }
    digits.parse::<usize>().ok()
}

fn parse_json_string(text: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let start = text.find(&pat)?;
    let after = &text[start + pat.len()..];
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for c in rest[1..].chars() {
        if escaped {
            match c {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                _ => out.push(c),
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn handle(
    method: &str,
    path: &str,
    body_text: &str,
    max_new_tokens: Option<usize>,
    model: &mut GlimmerModel,
    tok: &BpeTokenizer,
    seat: &str,
) -> Vec<u8> {
    match (method, path) {
        ("GET", "/health") => http_response("200 OK", "text/plain", b"ready"),
        ("GET", "/v1/info") => {
            let frame = amst::frame(0, 0, "", seat, "glimmer", 0, 0, 0);
            http_response(
                "200 OK",
                "application/vnd.rust-server.amst; version=1",
                &frame,
            )
        }
        ("POST", "/v1/tokenize") => {
            let Some(text) = parse_json_string(body_text, "text") else {
                return json_error("text required");
            };
            match tok.encode(&text) {
                Ok(ids) => {
                    let body = format!(
                        "[{}]",
                        ids.iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    http_response("200 OK", "application/json", body.as_bytes())
                }
                Err(e) => {
                    let body = format!("{{\"error\": \"tokenizer encode: {}\"}}", json_escape(&e.to_string()));
                    http_response("500 Internal Server Error", "application/json", body.as_bytes())
                }
            }
        }
        ("POST", "/v1/forward") => {
            let ids = parse_json_u32_array(body_text, "token_ids")
                .or_else(|| parse_json_u32_array(body_text, "input_ids"));
            let Some(ids) = ids else {
                return json_error("token_ids required");
            };
            let max_tokens = parse_json_usize(body_text, "max_tokens").unwrap_or(32);
            if !(1..=128).contains(&max_tokens) {
                return json_error("max_tokens must be 1..=128");
            }
            match generate(model, &ids, max_tokens) {
                Ok(out) => {
                    let digest = logits_digest_hex(&out.final_logits);
                    let receipt = receipt_hex(&ids, &out.output_ids, &digest);
                    let body = format!(
                        "{{\"output_token_ids\":[{}],\"logits_digest\":\"{}\",\"receipt_sha256\":\"{}\"}}",
                        out.output_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                        digest,
                        receipt
                    );
                    http_response("200 OK", "application/json", body.as_bytes())
                }
                Err(e) => {
                    let body = format!("{{\"error\": \"{}\"}}", json_escape(&e));
                    http_response("500 Internal Server Error", "application/json", body.as_bytes())
                }
            }
        }
        ("POST", "/v1/generate") => {
            if body_text.len() > 4096 {
                return json_error("prompt exceeds 4096 chars");
            }
            let max_tokens = match max_new_tokens {
                Some(n) if (1..=128).contains(&n) => n,
                Some(_) => {
                    return json_error("max-new-tokens must be 1..=128");
                }
                None => 32,
            };
            let ids = match tok.encode(&body_text) {
                Ok(ids) => ids,
                Err(e) => {
                    let body = format!("{{\"error\": \"tokenizer encode: {}\"}}", json_escape(&e.to_string()));
                    return http_response("500 Internal Server Error", "application/json", body.as_bytes());
                }
            };
            match generate(model, &ids, max_tokens) {
                Ok(out) => {
                    // Token frames: non-EOS tokens only. If the first token
                    // is EOS, there is no text to stream, and AMST cannot
                    // express 0 outputs, so this is a 500, matching the old server.
                    let text_ids: Vec<u32> = if out.eos {
                        out.output_ids[..out.output_ids.len().saturating_sub(1)].to_vec()
                    } else {
                        out.output_ids.clone()
                    };
                    if text_ids.is_empty() {
                        let body = "{\"error\": \"empty generation: first token was EOS\"}";
                        return http_response("500 Internal Server Error", "application/json", body.as_bytes());
                    }
                    let mut stream = Vec::new();
                    stream.extend_from_slice(&amst::frame(0, 0, "", seat, "glimmer", 0, 0, 0));
                    for (idx, id) in text_ids.iter().enumerate() {
                        let text = tok.decode(&[*id]);
                        stream.extend_from_slice(&amst::frame(
                            1,
                            (idx + 1) as u64,
                            &text,
                            "",
                            "",
                            0,
                            0,
                            0,
                        ));
                    }
                    let stop: u8 = if out.eos { 2 } else { 1 };
                    stream.extend_from_slice(&amst::frame(
                        2,
                        (text_ids.len() + 1) as u64,
                        "",
                        "",
                        "",
                        text_ids.len() as u32,
                        stop,
                        0,
                    ));
                    http_response(
                        "200 OK",
                        "application/vnd.rust-server.amst; version=1",
                        &stream,
                    )
                }
                Err(e) => {
                    let body = format!("{{\"error\": \"{}\"}}", json_escape(&e));
                    http_response("500 Internal Server Error", "application/json", body.as_bytes())
                }
            }
        }
        _ => {
            let body = "{\"error\": \"unknown route\"}";
            http_response("404 Not Found", "application/json", body.as_bytes())
        }
    }
}
