//! Bound output receipt: a single verifiable run artifact that binds
//! generated token IDs, logits digest, model artifact digest, exact
//! input bytes, tool result digest, and receipt chain — no separate
//! forward digest, no prose tool/args at the model I/O boundary.

use serde::{Deserialize, Serialize};
use sha2::Digest;

/// The exact input presented to the model (bytes, not prose).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelInputBytes {
    pub bytes: Vec<u8>,
    pub sha256: String, // 64-hex
}

impl ModelInputBytes {
    pub fn from_utf8(text: &str) -> Self {
        let bytes = text.as_bytes().to_vec();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        Self { bytes, sha256 }
    }
}

/// Result of a tool execution (mind.recall, etc.) with its exact digest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResultDigest {
    pub tool: String,
    pub args_sha256: String,    // 64-hex of args JSON
    pub result_sha256: String,  // 64-hex of tool's inner_text
    pub receipt_sha256: String, // 64-hex from executor (MCP/transport)
}

/// The model artifact identity (weights, tokenizer, config hash).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelArtifactDigest {
    pub weights_sha256: String,
    pub tokenizer_sha256: String,
    pub config_sha256: String,
    pub composite_sha256: String, // hash of the three above
}

/// One bound output receipt from a single verifiable run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundOutputReceipt {
    /// Generated token IDs from the model forward pass.
    pub generated_token_ids: Vec<u32>,
    /// Logits digest (64-hex) for the generated position.
    pub logits_digest: String,
    /// Exact K/V digest from this same native forward invocation.
    pub kv_digest: String,
    /// Model artifact digest (pinned at startup).
    pub model_artifact: ModelArtifactDigest,
    /// Exact input bytes fed to the model (no prose wrapper).
    pub input_bytes: ModelInputBytes,
    /// Tool result digests consumed in this run (ordered).
    pub tool_results: Vec<ToolResultDigest>,
    /// Receipt chain hash up to and including this receipt.
    pub chain_hash: String,
    /// Nanoseconds for this run.
    pub dt_ns: u64,
}

impl BoundOutputReceipt {
    pub fn validate(&self) -> Result<(), String> {
        let hex64 = |value: &str| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        };
        if self.generated_token_ids.is_empty() || self.input_bytes.bytes.is_empty() {
            return Err("bound receipt lacks model input or generated tokens".into());
        }
        for digest in [
            &self.logits_digest,
            &self.kv_digest,
            &self.model_artifact.weights_sha256,
            &self.model_artifact.tokenizer_sha256,
            &self.model_artifact.config_sha256,
            &self.model_artifact.composite_sha256,
            &self.input_bytes.sha256,
            &self.chain_hash,
        ] {
            if !hex64(digest) {
                return Err("bound receipt contains malformed digest".into());
            }
        }
        for tool in &self.tool_results {
            if !hex64(&tool.args_sha256)
                || !hex64(&tool.result_sha256)
                || !hex64(&tool.receipt_sha256)
            {
                return Err("bound receipt contains malformed tool digest".into());
            }
        }
        if format!("{:x}", sha2::Sha256::digest(&self.input_bytes.bytes)) != self.input_bytes.sha256
        {
            return Err("bound receipt input-byte digest mismatch".into());
        }
        if Self::compute_chain_hash(None, self) != self.chain_hash {
            return Err("bound receipt chain hash mismatch".into());
        }
        Ok(())
    }

    /// Compute the chain hash from previous head + this receipt's content.
    pub fn compute_chain_hash(prev_head: Option<&str>, receipt: &BoundOutputReceipt) -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        if let Some(prev) = prev_head {
            hasher.update(prev.as_bytes());
        }
        // Deterministic serialization of receipt content (excluding chain_hash itself)
        let content = serde_json::json!({
            "generated_token_ids": receipt.generated_token_ids,
            "logits_digest": receipt.logits_digest,
            "kv_digest": receipt.kv_digest,
            "model_artifact": receipt.model_artifact,
            "input_bytes": receipt.input_bytes,
            "tool_results": receipt.tool_results,
            "dt_ns": receipt.dt_ns,
        });
        let canon = serde_json::to_vec(&content).expect("json serialize");
        hasher.update(&canon);
        format!("{:x}", hasher.finalize())
    }
}

/// Extended CallOutcome that carries the bound receipt instead of a
/// separate forward digest. The executor MUST produce this atomically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundCallOutcome {
    pub inner_text: String,
    pub bound_receipt: BoundOutputReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_hash_deterministic() {
        let r = BoundOutputReceipt {
            generated_token_ids: vec![1, 2, 3],
            logits_digest: "a".repeat(64),
            kv_digest: "b".repeat(64),
            model_artifact: ModelArtifactDigest {
                weights_sha256: "w".repeat(64),
                tokenizer_sha256: "t".repeat(64),
                config_sha256: "c".repeat(64),
                composite_sha256: "z".repeat(64),
            },
            input_bytes: ModelInputBytes::from_utf8("test"),
            tool_results: vec![],
            chain_hash: String::new(),
            dt_ns: 123,
        };
        let h1 = BoundOutputReceipt::compute_chain_hash(None, &r);
        let h2 = BoundOutputReceipt::compute_chain_hash(None, &r);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        let mut valid = r.clone();
        valid.chain_hash = h1;
        assert!(valid.validate().is_err()); // placeholder artifact characters are not hex
        valid.model_artifact.weights_sha256 = "a".repeat(64);
        valid.model_artifact.tokenizer_sha256 = "b".repeat(64);
        valid.model_artifact.config_sha256 = "c".repeat(64);
        valid.model_artifact.composite_sha256 = "d".repeat(64);
        valid.chain_hash = BoundOutputReceipt::compute_chain_hash(None, &valid);
        assert!(valid.validate().is_ok());
        valid.kv_digest = "e".repeat(64);
        assert!(valid.validate().is_err());
    }
}
