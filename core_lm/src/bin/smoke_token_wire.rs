//! Real local hardware probe for the binary GTK2/GTO2 tokenize route.
use core_lm::artifact_digest::ArtifactDigest;
use core_lm::glimmer_model::GlimmerConfig;
use core_lm::native_token_wire::{
    decode_response, encode_request, TokenizeRequest, TOKEN_REQUEST_MAGIC,
};
use core_lm::tokenizer::BpeTokenizer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct HttpBody {
    status_ok: bool,
    status_line: String,
    body: Vec<u8>,
}

fn call(addr: &str, request: &[u8]) -> Result<HttpBody, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(60)))
        .map_err(|e| e.to_string())?;
    let header = format!(
        "POST /v1/tokenize-native HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/vnd.glimmer.tokenize-v1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
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
    let header = String::from_utf8_lossy(&response[..split]);
    let status_line = header.lines().next().unwrap_or("missing status").to_string();
    let status_ok = response.starts_with(b"HTTP/1.1 200 ");
    Ok(HttpBody {
        status_ok,
        status_line,
        body: response[split + 4..].to_vec(),
    })
}

fn expect_reject(addr: &str, request: &[u8], label: &str) -> Result<String, String> {
    let http = call(addr, request)?;
    if http.status_ok {
        return Err(format!("{label}: server accepted malformed tokenize frame"));
    }
    Ok(format!("{label}: {}", http.status_line))
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
    let local_artifact = ArtifactDigest::from_dir(model_dir)?;
    let text = "The capital of France is";
    let expected = tokenizer.encode(text).map_err(|e| e.to_string())?;
    let request = encode_request(&TokenizeRequest {
        text: text.into(),
        add_bos: true,
    })?;
    let first_http = call(&addr, &request)?;
    if !first_http.status_ok {
        return Err(format!("tokenize-native HTTP failure: {}", first_http.status_line));
    }
    let first = decode_response(&request, &first_http.body)?;
    let second_http = call(&addr, &request)?;
    if !second_http.status_ok {
        return Err(format!(
            "repeat tokenize-native HTTP failure: {}",
            second_http.status_line
        ));
    }
    let second = decode_response(&request, &second_http.body)?;
    if first != second {
        return Err("repeated native tokenize changed token IDs or receipt".into());
    }
    if first.tokenizer_digest != local_artifact.tokenizer {
        return Err("server tokenizer digest differs from frozen tokenizer.json".into());
    }
    if first.token_ids.first().copied() != Some(config.bos_token_id) {
        return Err("add_bos=1 did not prepend model-owned BOS".into());
    }
    if first.token_ids[1..] != expected {
        return Err(format!(
            "server token IDs diverged from model-owned tokenizer: {:?} vs {:?}",
            first.token_ids, expected
        ));
    }
    let no_bos_request = encode_request(&TokenizeRequest {
        text: text.into(),
        add_bos: false,
    })?;
    let no_bos_http = call(&addr, &no_bos_request)?;
    if !no_bos_http.status_ok {
        return Err(format!(
            "add_bos=0 tokenize-native HTTP failure: {}",
            no_bos_http.status_line
        ));
    }
    let no_bos = decode_response(&no_bos_request, &no_bos_http.body)?;
    if no_bos.token_ids != expected {
        return Err("add_bos=0 token IDs diverged from model-owned tokenizer".into());
    }

    let mut truncated = request.clone();
    truncated.pop();
    let mut wrong_magic = request.clone();
    wrong_magic[0] ^= 1;
    let mut bad_bos = request.clone();
    bad_bos[8] = 2;
    let empty_claimed = {
        let mut bytes = TOKEN_REQUEST_MAGIC.to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.push(1);
        bytes
    };
    let rejects = [
        expect_reject(&addr, &truncated, "truncated GTK2")?,
        expect_reject(&addr, &wrong_magic, "wrong GTK2 magic")?,
        expect_reject(&addr, &bad_bos, "invalid add_bos flag")?,
        expect_reject(&addr, &empty_claimed, "empty text length")?,
        expect_reject(&addr, TOKEN_REQUEST_MAGIC, "header-only GTK2")?,
    ];
    println!(
        "PASS native tokenize: token_ids={:?} tokenizer={} receipt={} bos={} rejects={}",
        first.token_ids,
        hex(&first.tokenizer_digest),
        hex(&first.receipt_digest),
        config.bos_token_id,
        rejects.join(" | "),
    );
    Ok(())
}
