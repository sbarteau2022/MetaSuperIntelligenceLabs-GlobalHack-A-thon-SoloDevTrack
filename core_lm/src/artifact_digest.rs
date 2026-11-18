//! Frozen Glimmer artifact identity, streamed from the actual local files.
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactDigest {
    pub weights: [u8; 32],
    pub tokenizer: [u8; 32],
    pub config: [u8; 32],
    pub composite: [u8; 32],
}

fn file_digest(path: &Path) -> Result<[u8; 32], String> {
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().into())
}

impl ArtifactDigest {
    pub fn from_parts(weights: [u8; 32], tokenizer: [u8; 32], config: [u8; 32]) -> Self {
        let mut composite = Sha256::new();
        composite.update(b"glimmer-artifact-v1\0");
        composite.update(weights);
        composite.update(tokenizer);
        composite.update(config);
        Self {
            weights,
            tokenizer,
            config,
            composite: composite.finalize().into(),
        }
    }

    pub fn verify_composite(&self) -> Result<(), String> {
        let mut composite = Sha256::new();
        composite.update(b"glimmer-artifact-v1\0");
        composite.update(self.weights);
        composite.update(self.tokenizer);
        composite.update(self.config);
        if self.composite != <[u8; 32]>::from(composite.finalize()) {
            return Err("Glimmer artifact composite digest mismatch".into());
        }
        Ok(())
    }

    pub fn from_dir(dir: &Path) -> Result<Self, String> {
        let config = file_digest(&dir.join("config.json"))?;
        let tokenizer = file_digest(&dir.join("tokenizer.json"))?;
        let mut roster = Sha256::new();
        roster.update(b"glimmer-weights-roster-v1\0");
        for name in [
            "model.safetensors.index.json",
            "model-00001-of-00004.safetensors",
            "model-00002-of-00004.safetensors",
            "model-00003-of-00004.safetensors",
            "model-00004-of-00004.safetensors",
        ] {
            roster.update((name.len() as u64).to_le_bytes());
            roster.update(name.as_bytes());
            roster.update(file_digest(&dir.join(name))?);
        }
        let weights: [u8; 32] = roster.finalize().into();
        let result = Self::from_parts(weights, tokenizer, config);
        result.verify_composite()?;
        Ok(result)
    }
}
