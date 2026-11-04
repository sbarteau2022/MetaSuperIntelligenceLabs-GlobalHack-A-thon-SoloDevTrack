//! Real local hardware probe for the binary exact-forward endpoint.
use core_lm::artifact_digest::ArtifactDigest;
use core_lm::glimmer_model::GlimmerConfig;
use core_lm::native_exact_wire::{decode_response, encode_request, ExactForwardRequest};
use core_lm::tokenizer::BpeTokenizer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn call(addr: &str, request: &[u8]) -> Result<Vec<u8>, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(300)))
        .map_err(|e| e.to_string())?;
    let header = format!(
        "POST /v1/forward-exact HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/vnd.glimmer.exact-v1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        request.len()
    );
    stream
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(request).map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| e.to_string())?;
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("HTTP response missing header terminator")?;
    if !response.starts_with(b"HTTP/1.1 200 ") {
        return Err(format!(
            "exact-forward HTTP failure: {}",
            String::from_utf8_lossy(&response[..split])
        ));
    }
    Ok(response[split + 4..].to_vec())
}

fn main() -> Result<(), String> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:18111".into());
    let model_dir = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX".into());
    let model_dir = Path::new(&model_dir);
    let config =
        GlimmerConfig::from_json_file(&model_dir.join("config.json")).map_err(|e| e.to_string())?;
    let tokenizer = BpeTokenizer::from_json_file(&model_dir.join("tokenizer.json"))
        .map_err(|e| e.to_string())?;
    let mut input_ids = tokenizer
        .encode("The capital of France is")
        .map_err(|e| e.to_string())?;
    input_ids.insert(0, config.bos_token_id);
    let request = encode_request(&ExactForwardRequest {
        input_ids,
        max_tokens: 1,
    })?;
    let first = decode_response(&request, &call(&addr, &request)?)?;
    let second = decode_response(&request, &call(&addr, &request)?)?;
    let local_artifact = ArtifactDigest::from_dir(model_dir)?;
    if first.artifact != local_artifact {
        return Err("server artifact digest differs from local frozen files".into());
    }
    if first != second {
        return Err("repeated exact forward changed token/logits/KV/seat receipt".into());
    }
    println!(
        "PASS native exact forward: token_ids={:?} logits={} kv={} weights={} tokenizer={} config={} composite={} receipt={}",
        first.output_ids,
        hex(&first.logits_digest),
        hex(&first.kv_digest),
        hex(&first.artifact.weights),
        hex(&first.artifact.tokenizer),
        hex(&first.artifact.config),
        hex(&first.artifact.composite),
        hex(&first.receipt_digest),
    );
    Ok(())
}
