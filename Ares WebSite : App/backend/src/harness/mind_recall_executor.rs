//! MCP-only tool executor. It cannot claim model-generated tokens or logits.
//! A bound model call must come from a native forward invocation instead.

use crate::harness::mcp::{attest_mind_child, McpAuthority, McpTransport, StdioMcpTransport};
use crate::harness::output_receipt::BoundCallOutcome;
use crate::harness::runtime::{CallOutcome, Executor};
use std::time::Instant;

pub struct MindRecallExecutor {
    authority: McpAuthority,
    credentials: (String, String),
    bin: String,
    root: String,
    identity: String,
}

impl MindRecallExecutor {
    pub fn from_state(
        authority: McpAuthority,
        credentials: (String, String),
    ) -> Result<Self, String> {
        if authority.base.is_empty()
            || authority.ns.is_empty()
            || authority.db.is_empty()
            || authority.db == "core"
        {
            return Err("MCP mind authority is missing or quarantined".into());
        }
        let bin = std::env::var("ARES_MCP_MIND_BIN").unwrap_or_else(|_| "globalmind-mcp".into());
        let root = std::env::var("ARES_MCP_MIND_ROOT").unwrap_or_else(|_| ".".into());
        let identity = format!("{}|{}|{}", authority.base, authority.ns, authority.db);
        Ok(Self {
            authority,
            credentials,
            bin,
            root,
            identity,
        })
    }

    pub fn seat_identity(&self) -> String {
        self.identity.clone()
    }
}

impl Executor for MindRecallExecutor {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        let started = Instant::now();
        let env_vars = [
            ("GLOBALMIND_SURREAL_URL", self.authority.base.as_str()),
            ("GLOBALMIND_SURREAL_NS", self.authority.ns.as_str()),
            ("GLOBALMIND_SURREAL_DB", self.authority.db.as_str()),
            ("GLOBALMIND_SURREAL_USER", self.credentials.0.as_str()),
            ("GLOBALMIND_SURREAL_PASS", self.credentials.1.as_str()),
            ("GLOBALMIND_ALLOW_QUARANTINED_CORE", "0"),
        ];
        let mut transport = StdioMcpTransport::spawn(&self.bin, &self.root, &env_vars)?;
        attest_mind_child(&mut transport, &self.authority)?;
        let result = transport.call(tool, args_json)?;
        Ok(CallOutcome {
            inner_text: result.inner_text,
            receipt_sha256: result.receipt_sha256,
            dt_ns: started.elapsed().as_nanos() as u64,
        })
    }

    fn call_bound(&mut self, _tool: &str, _args_json: &str) -> Result<BoundCallOutcome, String> {
        Err("MCP-only executor has no model forward; bound output requires real token IDs, logits, weights and exact K/V".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_model_output_without_forward() {
        let authority = McpAuthority {
            base: "http://127.0.0.1:8000".into(),
            ns: "hypermind".into(),
            db: "globalmind_dev".into(),
        };
        let mut executor =
            MindRecallExecutor::from_state(authority, ("root".into(), "unused".into())).unwrap();
        assert!(executor.call_bound("mind.recall", "{}").is_err());
    }
}
