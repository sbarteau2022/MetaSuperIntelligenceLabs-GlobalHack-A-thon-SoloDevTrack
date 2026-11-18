//! Pinned Rust tokenizer for the admitted Qwen3 VL ingest recipe.
//! Native model execution is a separate gate; this module never fabricates vectors.
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const TOKENIZER_SHA256: &str = "93623af029cdc69b87f2864d3b2cc2424fdf16684f15e139b5b9d08ec34ced91";
const MAX_TOKENIZER_BYTES: u64 = 16 * 1024 * 1024;
const MAX_QUERY_BYTES: usize = 3200;

pub struct QwenVlTokenizer(tokenizers::Tokenizer);

impl QwenVlTokenizer {
    pub fn load(directory: &Path) -> Result<Self, String> {
        let path = directory.join("tokenizer.json");
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > MAX_TOKENIZER_BYTES {
            return Err("Qwen VL tokenizer must be a bounded regular file".into());
        }
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        if bytes.len() as u64 != metadata.len()
            || format!("{:x}", Sha256::digest(&bytes)) != TOKENIZER_SHA256
        {
            return Err("Qwen VL tokenizer digest mismatch".into());
        }
        let tokenizer = tokenizers::Tokenizer::from_bytes(&bytes).map_err(|e| e.to_string())?;
        Ok(Self(tokenizer))
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>, String> {
        if text.trim().is_empty() || text.len() > MAX_QUERY_BYTES {
            return Err("Qwen VL query is empty or exceeds the admitted byte cap".into());
        }
        let encoding = self.0.encode(text, true).map_err(|e| e.to_string())?;
        let ids = encoding.get_ids().to_vec();
        if ids.is_empty() || ids.len() > 1024 {
            return Err("Qwen VL query exceeds the token cap".into());
        }
        Ok(ids)
    }
}
