//! serve_qwen — native binary seat for Qwen3.6-MoE (qwen3_5_moe) for the research-grade harness.
//!
//! Loopback only, sequential (the model is &mut and its geometric memory must not interleave).
//! Every request and reply is typed bytes. No JSON crosses this boundary.
//!
//! Endpoints:
//! - GET  /health                 -> "ready" (model is loaded before the listener opens)
//! - POST /v1/tokenize-native     GTK2 request -> GTO2 reply (native_token_wire)
//! - POST /v1/detokenize-native   GDT2 request -> GDR2 reply (native_token_wire)
//! - POST /v1/complete-native     QNC1 request -> QNR1 reply (stateless, greedy, EOS stops)
//!     request : "QNC1" | u32 max_new (1..=128) | u32 n (1..=1024) | n x u32 prompt ids
//!     reply   : "QNR1" | u32 n_out | n_out x u32 output ids | 32-byte receipt
//!     receipt : SHA-256( request bytes || "QNR1" ... output ids )
//!
//! Each complete call resets geometric memory, prefills the prompt token by token,
//! then decodes greedily. Two identical requests return identical ids.
//!
//! Usage: serve_qwen [127.0.0.1:8003] [--model-dir DIR]
//! Default model dir: $HOME/models/fused/Qwen3.6-MoE-Combined

use core_lm::native_token_wire as wire;
use core_lm::ops::argmax;
use core_lm::qwen_forward::QwenModel;
use core_lm::sha256::sha256;
use core_lm::tokenizer::BpeTokenizer;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

const COMPLETE_REQ: &[u8; 4] = b"QNC1";
const COMPLETE_RESP: &[u8; 4] = b"QNR1";
const MAX_NEW: usize = 128;
const MAX_PROMPT: usize = 1024;

fn decode_complete(body: &[u8]) -> Result<(usize, Vec<u32>), String> {
    if body.len() < 12 || &body[..4] != COMPLETE_REQ {
        return Err("complete request header invalid".into());
    }
    let max_new = u32::from_le_bytes(body[4..8].try_into().unwrap()) as usize;
    let n = u32::from_le_bytes(body[8..12].try_into().unwrap()) as usize;
    if !(1..=MAX_NEW).contains(&max_new) || n == 0 || n > MAX_PROMPT || body.len() != 12 + 4 * n {
        return Err("complete request bounds invalid".into());
    }
    let ids = body[12..]
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    Ok((max_new, ids))
}

fn complete(model: &mut QwenModel, prompt: &[u32], max_new: usize) -> Result<Vec<u32>, String> {
    let t0 = std::time::Instant::now();
    model.reset_memory();
    let mut logits = Vec::new();
    for (pos, &id) in prompt.iter().enumerate() {
        logits = model.forward(id, pos).map_err(|e| e.to_string())?;
    }
    eprintln!("prefill: {} tokens in {:.2}s", prompt.len(), t0.elapsed().as_secs_f32());
    let eos = model.config.eos_token_ids.clone();
    let mut out = Vec::new();
    for i in 0..max_new {
        let next = argmax(&logits).map_err(|e| format!("argmax: {e}"))? as u32;
        if eos.contains(&next) {
            break;
        }
        out.push(next);
        if i + 1 < max_new {
            logits = model
                .forward(next, prompt.len() + i)
                .map_err(|e| e.to_string())?;
        }
    }
    eprintln!("complete: {} new tokens, total {:.2}s", out.len(), t0.elapsed().as_secs_f32());
    Ok(out)
}

fn encode_complete(request: &[u8], out: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + 4 * out.len() + 32);
    bytes.extend_from_slice(COMPLETE_RESP);
    bytes.extend_from_slice(&(out.len() as u32).to_le_bytes());
    for id in out {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    let mut bound = request.to_vec();
    bound.extend_from_slice(&bytes);
    bytes.extend_from_slice(&sha256(&bound));
    bytes
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

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut bind = "127.0.0.1:8003".to_string();
    let mut model_dir: Option<String> = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--model-dir" => {
                i += 1;
                model_dir = Some(args.get(i).ok_or("--model-dir needs a value")?.clone());
            }
            "--help" => {
                println!("Usage: serve_qwen [127.0.0.1:8003] [--model-dir DIR]");
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
        return Err("serve_qwen is loopback only".into());
    }
    let model_dir = match model_dir {
        Some(d) => PathBuf::from(d),
        None => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/stewartbarteau".to_string());
            PathBuf::from(home).join("models/fused/Qwen3.6-MoE-Combined")
        }
    };
    let tok_bytes = std::fs::read(model_dir.join("tokenizer.json"))?;
    let tok = BpeTokenizer::from_json_file(&model_dir.join("tokenizer.json"))?;
    let tokenizer_digest = sha256(&tok_bytes);
    println!("Loading Qwen model from {} (this takes a while) ...", model_dir.display());
    let mut model = QwenModel::load(&model_dir)?;
    println!(
        "Model loaded (layers={}, hidden={}, eos={:?})",
        model.config.num_hidden_layers, model.config.hidden_size, model.config.eos_token_ids
    );
    let listener = TcpListener::bind(&bind)?;
    println!("serve_qwen listening on {bind}");

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
        let Some(hend) = header_end else { continue };
        let header_text = String::from_utf8_lossy(&buf[..hend]).to_string();
        let mut lines = header_text.lines();
        let mut parts = lines.next().unwrap_or("").split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("").to_string();
        let mut content_length: usize = 0;
        for line in lines {
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
        if content_length > 16 * 1024 * 1024 {
            let _ = stream.write_all(&http_response("413 Payload Too Large", "text/plain", b"request too large"));
            continue;
        }
        while buf.len() < hend + content_length {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        let body = buf[hend..std::cmp::min(buf.len(), hend + content_length)].to_vec();
        let ctype = "application/octet-stream";
        let resp = match (method.as_str(), path.as_str()) {
            ("GET", "/health") => http_response("200 OK", "text/plain", b"ready"),
            ("POST", "/v1/tokenize-native") => match wire::decode_request(&body)
                .and_then(|req| tok.encode(&req.text).map_err(|e| e.to_string()))
                .and_then(|ids| wire::encode_response(&body, &ids, tokenizer_digest))
            {
                Ok(bytes) => http_response("200 OK", ctype, &bytes),
                Err(e) => http_response("400 Bad Request", "text/plain", e.as_bytes()),
            },
            ("POST", "/v1/detokenize-native") => match wire::decode_detok_request(&body)
                .and_then(|ids| wire::encode_detok_response(&body, &tok.decode(&ids)))
            {
                Ok(bytes) => http_response("200 OK", ctype, &bytes),
                Err(e) => http_response("400 Bad Request", "text/plain", e.as_bytes()),
            },
            ("POST", "/v1/complete-native") => match decode_complete(&body)
                .and_then(|(max_new, prompt)| complete(&mut model, &prompt, max_new))
            {
                Ok(out) => http_response("200 OK", ctype, &encode_complete(&body, &out)),
                Err(e) => http_response("500 Internal Server Error", "text/plain", e.as_bytes()),
            },
            _ => http_response("404 Not Found", "text/plain", b"unknown endpoint"),
        };
        if let Err(e) = stream.write_all(&resp) {
            eprintln!("write: {e}");
        }
    }
    Ok(())
}
