//! MCP adapter: the client side of the MCP-to-harness seam.
//!
//! This is the ONLY governed path from the backend to an MCP child.
//! A raw MCP transport is never an `Executor` on its own: every call
//! goes through the automaton transition first, and confidence moves
//! only on a verified outcome (see `automaton.rs`). The adapter
//! never trusts the transport: the receipt it returns must be an
//! exact 64-hex digest, or the call fails with no confidence change.
//!
//! Authority, two databases on one server, distinct by role:
//! - Mind authority: the existing mind the MCP child serves,
//!   default (http://127.0.0.1:8000, hypermind, core) —
//!   hypermind-mcp's Surreal defaults, 2079 atoms. The child
//!   must point at THIS mind, never create a new one.
//! - Governance authority: the backend's Surreal substrate,
//!   default (http://127.0.0.1:8000, ares, ares) — receipts,
//!   permits, automaton confidence and transition receipts.
//! The adapter checks MCP authority against the EXPECTED MIND
//! authority, not against governance. An earlier version checked
//! MCP == harness (governance) authority; that was wrong for a
//! mind child, because mind and governance are different
//! databases by design. Persistence SQL is executed by the
//! caller against the governance client — the adapter never
//! writes confidence into the mind database, so atom identity
//! is never polluted by confidence state. The hypermind-mcp
//! child declares its authority the same way
//! (`HYPERMIND_SURREAL=on` + URL/NS/DB env), and the caller
//! verifies the live child with `map.db_status` attestation
//! (see research-grade-harness `globalmind_bridge.rs`) — this
//! adapter checks the configured authority; live attestation is
//! the caller's step, not a claim made here.
//!
//! Transport is a trait so tests can stub it. A stdio child
//! transport (spawning hypermind-mcp) is the next wiring step;
//! this module does not spawn processes and does not claim a
//! live child.

use crate::harness::automaton::{
    confidence_upsert_sql, transition, transition_insert_sql, update_from_outcome,
    verify_mcp_authority, AuditSignals, AutomatonTransition, ProposerEnvelope, StrategyState,
    Verdict,
};

/// Configured MCP authority: (base URL, namespace, database).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpAuthority {
    pub base: String,
    pub ns: String,
    pub db: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpCall {
    pub inner_text: String,
    pub receipt_sha256: String,
    pub dt_ns: u64,
}

pub trait McpTransport {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<McpCall, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpExecution {
    pub transition: AutomatonTransition,
    pub outcome: Option<McpCall>,
    /// Confidence transition, only when the outcome was verified.
    pub confidence_transition: Option<AutomatonTransition>,
    /// SQL text for persistence through the governed client.
    pub confidence_sql: Option<String>,
    pub receipt_sql: String,
}

/// Stdio MCP child transport: spawns an MCP server binary
/// (hypermind-mcp, ares-mcp-mind, or any stdio JSON-RPC MCP
/// server), sends `tools/call` lines, reads one response line
/// per call. The child is killed on drop. Credentials and
/// authority env vars are the caller's responsibility — they
/// are passed to the child process env, never printed or
/// logged by this module.
pub struct StdioMcpTransport {
    child: std::process::Child,
    reader: std::io::BufReader<std::process::ChildStdout>,
    writer: std::process::ChildStdin,
    stderr: Option<std::process::ChildStderr>,
    next_id: u64,
}

impl StdioMcpTransport {
    pub fn spawn(path: &str, root: &str, env_vars: &[(&str, &str)]) -> Result<Self, String> {
        let mut cmd = std::process::Command::new(path);
        cmd.env("HYPERMIND_ROOT", root);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {}: {}", path, e))?;
        let reader = std::io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| "child stdout unavailable".to_string())?,
        );
        let writer = child
            .stdin
            .take()
            .ok_or_else(|| "child stdin unavailable".to_string())?;
        let stderr = child.stderr.take();
        Ok(Self {
            child,
            reader,
            writer,
            stderr,
            next_id: 1,
        })
    }
}

impl Drop for StdioMcpTransport {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl McpTransport for StdioMcpTransport {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<McpCall, String> {
        let id = self.next_id;
        self.next_id += 1;
        let args: serde_json::Value =
            serde_json::from_str(args_json).map_err(|e| format!("args_json parse: {}", e))?;
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {"name": tool, "arguments": args},
        });
        let started = std::time::Instant::now();
        use std::io::{BufRead, Write};
        self.writer
            .write_all(format!("{}\n", req).as_bytes())
            .map_err(|e| format!("write: {}", e))?;
        self.writer.flush().map_err(|e| format!("flush: {}", e))?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| format!("read: {}", e))?;
        if line.is_empty() {
            // Child exited: read its stderr tail for diagnosis
            // (diagnostics only, never evidence).
            let mut tail = String::new();
            if let Some(mut s) = self.stderr.take() {
                use std::io::Read;
                let _ = s.read_to_string(&mut tail);
            }
            return Err(format!("eof: child closed stdout stderr={:?}", tail));
        }
        let body: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| format!("response parse: {}", e))?;
        if let Some(err) = body.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("rpc error");
            return Err(format!("rpc error: {}", msg));
        }
        let text = body
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| "missing result content text".to_string())?;
        let inner: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("inner parse: {}", e))?;
        if let Some(status) = inner.get("status").and_then(|s| s.as_str()) {
            if status != "ok" {
                return Err(format!("mcp tool {} status: {}", tool, status));
            }
        }
        let receipt = inner
            .get("receipt_sha256")
            .and_then(|r| r.as_str())
            .ok_or_else(|| format!("{}: missing receipt_sha256", tool))?
            .to_string();
        Ok(McpCall {
            inner_text: text.to_string(),
            receipt_sha256: receipt,
            dt_ns: started.elapsed().as_nanos() as u64,
        })
    }
}

fn is_64_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Governed MCP execution: authority → automaton transition →
/// transport (only if Admit) → verified-outcome confidence update.
/// No transport call happens on Clarify or Reject, and no
/// confidence change happens on an unverified outcome.
#[allow(clippy::too_many_arguments)]
pub fn execute_via_automaton<T: McpTransport>(
    state: &mut StrategyState,
    envelope: &ProposerEnvelope,
    signals: &AuditSignals,
    now_ms: u64,
    mcp_authority: &McpAuthority,
    expected_mind_authority: &McpAuthority,
    tool: &str,
    args_json: &str,
    transport: &mut T,
    verified: bool,
    success: bool,
) -> Result<McpExecution, String> {
    verify_mcp_authority(
        &mcp_authority.base,
        &mcp_authority.ns,
        &mcp_authority.db,
        &expected_mind_authority.base,
        &expected_mind_authority.ns,
        &expected_mind_authority.db,
    )?;
    let transition_record = transition(state, envelope, signals, now_ms);
    let receipt_sql = transition_insert_sql(&transition_record);
    if transition_record.verdict != Verdict::Admit {
        return Ok(McpExecution {
            transition: transition_record,
            outcome: None,
            confidence_transition: None,
            confidence_sql: None,
            receipt_sql,
        });
    }
    let call = transport.call(tool, args_json)?;
    if !is_64_hex(&call.receipt_sha256) {
        return Err(format!(
            "MCP call returned a non-64-hex receipt for tool {}",
            tool
        ));
    }
    let (confidence_transition, confidence_sql) = if verified {
        let ct = update_from_outcome(state, true, success)?;
        let sql = confidence_upsert_sql(state);
        (Some(ct), Some(sql))
    } else {
        (None, None)
    };
    Ok(McpExecution {
        transition: transition_record,
        outcome: Some(call),
        confidence_transition,
        confidence_sql,
        receipt_sql,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Q32_31;
    use crate::harness::automaton::{ProposerEnvelope, ProposerFamily};

    struct StubTransport {
        calls: usize,
        receipt: String,
    }

    impl McpTransport for StubTransport {
        fn call(&mut self, _tool: &str, _args_json: &str) -> Result<McpCall, String> {
            self.calls += 1;
            Ok(McpCall {
                inner_text: "{\"status\":\"ok\"}".to_string(),
                receipt_sha256: self.receipt.clone(),
                dt_ns: 42,
            })
        }
    }

    fn state() -> StrategyState {
        StrategyState {
            strategy_id: "mcp-retrieval".to_string(),
            policy_version: "policy-v1".to_string(),
            prefix_identity: "prefix-1".to_string(),
            map_identity: "globalmind-ares".to_string(),
            confidence: Q32_31::from_f64(0.6),
            confidence_floor: Q32_31::from_f64(0.5),
            calibration_count: 0,
            outcome_history_digest: "h0".to_string(),
        }
    }

    fn envelope() -> ProposerEnvelope {
        ProposerEnvelope {
            family: ProposerFamily::Retrieval,
            strategy_id: "mcp-retrieval".to_string(),
            policy_version: "policy-v1".to_string(),
            prefix_identity: "prefix-1".to_string(),
            map_identity: "globalmind-ares".to_string(),
            payload: "query: test".to_string(),
            expiry_ms: 1_000_000,
        }
    }

    fn signals(conf: f64) -> AuditSignals {
        AuditSignals {
            measured_confidence: Q32_31::from_f64(conf),
            unresolved_dependencies: 0,
            entity_ambiguity: false,
            expected_information_gain: Q32_31::from_f64(0.7),
        }
    }

    fn authority() -> McpAuthority {
        McpAuthority {
            base: "http://127.0.0.1:8000".to_string(),
            ns: "ares".to_string(),
            db: "ares".to_string(),
        }
    }

    fn good_receipt() -> String {
        "a".repeat(64)
    }

    #[test]
    fn admit_calls_transport_once_and_updates_confidence_on_verified_outcome() {
        let mut s = state();
        let mut t = StubTransport {
            calls: 0,
            receipt: good_receipt(),
        };
        let ex = execute_via_automaton(
            &mut s,
            &envelope(),
            &signals(0.8),
            100,
            &authority(),
            &authority(),
            "map.recall",
            "{}",
            &mut t,
            true,
            true,
        )
        .expect("admit execution");
        assert_eq!(t.calls, 1);
        assert!(ex.outcome.is_some());
        assert!(ex.confidence_transition.is_some());
        assert!(s.confidence > Q32_31::from_f64(0.6));
    }

    #[test]
    fn clarify_and_reject_never_call_transport() {
        let mut s = state();
        let mut t = StubTransport {
            calls: 0,
            receipt: good_receipt(),
        };
        let ex = execute_via_automaton(
            &mut s,
            &envelope(),
            &signals(0.4),
            100,
            &authority(),
            &authority(),
            "map.recall",
            "{}",
            &mut t,
            true,
            true,
        )
        .expect("clarify execution");
        assert_eq!(ex.transition.verdict, Verdict::Clarify);
        assert_eq!(t.calls, 0);
        assert!(ex.outcome.is_none());
    }

    #[test]
    fn authority_mismatch_fails_before_transport() {
        let mut s = state();
        let mut t = StubTransport {
            calls: 0,
            receipt: good_receipt(),
        };
        let other = McpAuthority {
            base: "http://127.0.0.1:8000".to_string(),
            ns: "hypermind".to_string(),
            db: "core".to_string(),
        };
        assert!(execute_via_automaton(
            &mut s,
            &envelope(),
            &signals(0.8),
            100,
            &other,
            &authority(),
            "map.recall",
            "{}",
            &mut t,
            true,
            true,
        )
        .is_err());
        assert_eq!(t.calls, 0);
    }

    #[test]
    fn bad_receipt_fails_with_no_confidence_change() {
        let mut s = state();
        let before = s.clone();
        let mut t = StubTransport {
            calls: 0,
            receipt: "not-a-digest".to_string(),
        };
        assert!(execute_via_automaton(
            &mut s,
            &envelope(),
            &signals(0.8),
            100,
            &authority(),
            &authority(),
            "map.recall",
            "{}",
            &mut t,
            true,
            true,
        )
        .is_err());
        assert_eq!(s, before);
    }

    #[test]
    fn unverified_outcome_calls_transport_but_moves_no_confidence() {
        let mut s = state();
        let before = s.clone();
        let mut t = StubTransport {
            calls: 0,
            receipt: good_receipt(),
        };
        let ex = execute_via_automaton(
            &mut s,
            &envelope(),
            &signals(0.8),
            100,
            &authority(),
            &authority(),
            "map.recall",
            "{}",
            &mut t,
            false,
            true,
        )
        .expect("unverified execution");
        assert_eq!(t.calls, 1);
        assert!(ex.confidence_transition.is_none());
        assert_eq!(s, before);
    }
}
