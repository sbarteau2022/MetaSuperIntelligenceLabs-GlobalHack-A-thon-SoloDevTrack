//! MCP adapter: the client side of the MCP-to-harness seam.
//!
//! This is the ONLY governed path from the backend to an MCP child.
//! A raw MCP transport is never an `Executor` on its own: every call
//! goes through the automaton transition first, and confidence moves
//! only on a verified outcome (see `automaton.rs`). The adapter
//! never trusts the transport: the receipt it returns must be an
//! exact 64-hex digest, or the call fails with no confidence change.
//!
//! Authority: the MCP target must resolve to exactly the harness
//! GlobalMind authority (base, namespace, database). Any mismatch
//! fails before any transport call, so no MCP path can create a
//! second map. The hypermind-mcp child declares its authority the
//! same way (`HYPERMIND_SURREAL=on` + URL/NS/DB env), and the
//! caller verifies the live child with `map.db_status` attestation
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
    harness_authority: &McpAuthority,
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
        &harness_authority.base,
        &harness_authority.ns,
        &harness_authority.db,
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
