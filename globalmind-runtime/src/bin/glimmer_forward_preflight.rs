//! Read-only Glimmer forward readiness gate. No inference or database writes.
use globalmind_runtime::Manifest;
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const EXPECTED_MODEL: &str = "Muse-Glimmer-q4km-gs128-MLX";
const EXPECTED_CONFIG: &str = "3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031";
const EXPECTED_TOKENIZER: &str = "c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d";

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 && !(args.len() == 3 && args[2] == "--native-baseline") {
        return Err(
            "usage: glimmer-forward-preflight <frozen-manifest> [--native-baseline]".into(),
        );
    }
    let native_baseline = args.len() == 3;
    let manifest = Manifest::load(&args[1])?;
    if manifest.models.len() != 1 || manifest.models[0].name != EXPECTED_MODEL {
        return Err("preflight requires the one frozen Glimmer model".into());
    }
    let model = &manifest.models[0];
    let config = model
        .artifacts
        .iter()
        .find(|a| a.role == "config")
        .ok_or("Glimmer manifest has no config artifact")?;
    let tokenizer = model
        .artifacts
        .iter()
        .find(|a| a.role == "tokenizer")
        .ok_or("Glimmer manifest has no tokenizer artifact")?;
    if config.sha256.as_str() != EXPECTED_CONFIG || tokenizer.sha256.as_str() != EXPECTED_TOKENIZER
    {
        return Err("Glimmer config/tokenizer identity drift".into());
    }
    let bytes = std::fs::read(&config.path).map_err(|e| format!("read Glimmer config: {e}"))?;
    if bytes.len() > 64 * 1024 {
        return Err("Glimmer config exceeds 64 KiB".into());
    }
    let cfg: Value = serde_json::from_slice(&bytes).map_err(|e| format!("Glimmer config: {e}"))?;
    for (key, expected) in [
        ("num_hidden_layers", 52),
        ("hidden_size", 6656),
        ("vocab_size", 202048),
        ("num_attention_heads", 32),
        ("num_key_value_heads", 2),
        ("head_dim", 128),
    ] {
        if cfg[key].as_u64() != Some(expected) {
            return Err(format!("Glimmer architecture mismatch: {key}"));
        }
    }
    if cfg["model_type"] != "muse_glimmer"
        || cfg["layer_types"].as_array().is_none_or(|v| v.len() != 52)
        || cfg["quantization"]["mode"] != "affine"
    {
        return Err("Glimmer architecture/quantization mismatch".into());
    }
    let verified = manifest.verify_artifacts()?;
    println!(
        "artifact_rehash=PASS roster_digest={} input_digest={}",
        verified.models[0].roster_digest, verified.input_digest
    );
    if !native_baseline
        && (manifest.tppe_version == "tppe-reference-v1"
            || !manifest.tppe_version.starts_with("tppe-glimmer-"))
    {
        return Err(
            "Glimmer TPPE contract is unqualified; reference-v1 cannot enter forward smoke".into(),
        );
    }
    if native_baseline {
        println!("position_mode=native_glimmer_rope_nope tppe_memory_address=not_exercised");
    }
    let bin = std::env::var("GLIMMER_NATIVE_FORWARD_BIN")
        .map_err(|_| "GLIMMER_NATIVE_FORWARD_BIN is unset".to_string())?;
    let expected = std::env::var("GLIMMER_NATIVE_FORWARD_SHA256")
        .map_err(|_| "GLIMMER_NATIVE_FORWARD_SHA256 is unset".to_string())?;
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("GLIMMER_NATIVE_FORWARD_SHA256 must be 64 hex characters".into());
    }
    let path = Path::new(&bin);
    if !path.is_absolute()
        || !std::fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .is_file()
    {
        return Err("Glimmer native executable must be an absolute regular file".into());
    }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    if format!("{:x}", hash.finalize()) != expected.to_ascii_lowercase() {
        return Err("Glimmer native executable digest mismatch".into());
    }
    println!("native_executor_identity=PASS path={bin}");
    println!("preflight=PASS inference_not_run=true");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("glimmer-forward-preflight: {error}");
        std::process::exit(1);
    }
}
