//! Frozen Glimmer tokenizer for the shared swarm model seat.
//! The 6656-wide hidden state is a model property, not a tokenizer dimension.
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const TOKENIZER_SHA256: &str = "c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d";
const CONFIG_SHA256: &str = "3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031";
const MAX_TOKENIZER_BYTES: u64 = 32 * 1024 * 1024;
const MAX_QUERY_BYTES: usize = 3200;

pub struct GlimmerTokenizer(tokenizers::Tokenizer);

fn pinned_file(path: &Path, max_bytes: u64, digest: &str) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err("Glimmer checkpoint file is not a bounded regular file".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() as u64 != metadata.len() || format!("{:x}", Sha256::digest(&bytes)) != digest {
        return Err("Glimmer checkpoint digest mismatch".into());
    }
    Ok(bytes)
}

impl GlimmerTokenizer {
    pub fn load(directory: &Path) -> Result<Self, String> {
        pinned_file(&directory.join("config.json"), 64 * 1024, CONFIG_SHA256)?;
        let bytes = pinned_file(
            &directory.join("tokenizer.json"),
            MAX_TOKENIZER_BYTES,
            TOKENIZER_SHA256,
        )?;
        let tokenizer = tokenizers::Tokenizer::from_bytes(&bytes).map_err(|e| e.to_string())?;
        Ok(Self(tokenizer))
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>, String> {
        if text.trim().is_empty() || text.len() > MAX_QUERY_BYTES {
            return Err("Glimmer input is empty or exceeds byte cap".into());
        }
        let encoding = self.0.encode(text, true).map_err(|e| e.to_string())?;
        let ids = encoding.get_ids().to_vec();
        if ids.is_empty() || ids.len() > 1024 {
            return Err("Glimmer input exceeds token cap".into());
        }
        Ok(ids)
    }
}
