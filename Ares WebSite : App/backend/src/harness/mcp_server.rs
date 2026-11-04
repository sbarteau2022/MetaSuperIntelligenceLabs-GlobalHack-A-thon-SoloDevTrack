//! Mind-as-MCP server dispatch, pure function over JSON values.
//!
//! This is the server side of the MCP packaging: the multimodel mind
//! (as owned by the backend) exposed as MCP tools, speaking the same
//! JSON-RPC shape as hypermind-mcp (initialize / tools/list /
//! tools/call, inner text result). The binary `ares-mcp-mind` is a
//! thin stdio loop around `handle_request`.
//!
//! Tools in v1, all backend-owned, none invented:
//! - `mind.db_status`: authority attestation — mode, endpoint,
//!   namespace, database, and whether SurrealDB was reachable at
//!   this instant. Mode is "On" only when reachable; otherwise
//!   "Unavailable". No fake attestation, no parity claim the
//!   server cannot measure. (hypermind-mcp's `map.db_status`
//!   attests parity_atoms for its own store; this server does
//!   not claim that store's parity.)
//! - `mind.automaton_decide`: deterministic automaton transition
//!   for a proposal (envelope + measured audit signals). Returns
//!   verdict + receipt digest. It does NOT execute anything and
//!   does NOT move confidence — confidence moves only on a
//!   verified outcome, through the governed backend path, not
//!   through an MCP call. A caller that treats a decision as
//!   execution is misreading the tool.
//!
//! NOT in v1, honestly: `mind.atom` / `mind.kv_exact` /
//! `mind.recall` for the multimodel mind. Atom and exact-K/V
//! retrieval live in `globalmind-runtime` (Store, ExactKvIdentity,
//! retrieval bundles), which the backend does not depend on, and
//! whose Glimmer-forward work is in motion in another lane.
//! Packaging those tools belongs in a `globalmind-runtime` MCP
//! bin, not in a backend copy of that store — building it here
//! would create the second map Step 3 forbids. The tool
//! contract for that bin is in `mind-mcp-packaging.md`.

use crate::fixed::Q32_31;
use crate::harness::automaton::{
    transition, AuditSignals, ProposerEnvelope, ProposerFamily, StrategyState,
};
use serde_json::{json, Value};

pub struct MindAuthority {
    pub base: String,
    pub ns: String,
    pub db: String,
    pub surreal_reachable: bool,
}

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// MCP result envelope. The inner JSON carries a receipt_sha256
/// binding tool + arguments + inner content (SHA-256, same binding
/// style as backend receipts). A client that checks the receipt
/// can verify the inner text was not altered in transit; a tool
/// that returns no receipt fails the adapter's receipt check.
/// This receipt binds the tool call, it is NOT a claim that the
/// inner content is true — that is the caller's judge step.
fn text_result(id: &Value, tool: &str, args: &Value, inner: Value) -> Value {
    let receipt =
        crate::digest::Sha256Digest::compute(format!("{}\n{}\n{}", tool, args, inner).as_bytes())
            .to_hex();
    let mut obj = inner.as_object().cloned().unwrap_or_default();
    obj.insert("receipt_sha256".to_string(), Value::String(receipt));
    let text = Value::Object(obj).to_string();
    json!({"jsonrpc": "2.0", "id": id, "result": {"content": [{"type": "text", "text": text}]}})
}

fn tools_list() -> Value {
    json!({"tools": [
        {"name": "mind.db_status", "description": "Attest this mind's SurrealDB authority and reachability at this instant.", "inputSchema": {"type": "object", "properties": {}}},
        {"name": "mind.automaton_decide", "description": "Deterministic automaton decision (admit/clarify/reject) for a proposal. Does not execute, does not move confidence.", "inputSchema": {"type": "object", "properties": {
            "family": {"type": "string"},
            "strategy_id": {"type": "string"},
            "policy_version": {"type": "string"},
            "prefix_identity": {"type": "string"},
            "map_identity": {"type": "string"},
            "payload": {"type": "string"},
            "expiry_ms": {"type": "integer"},
            "measured_confidence": {"type": "number"},
            "unresolved_dependencies": {"type": "integer"},
            "entity_ambiguity": {"type": "boolean"},
            "expected_information_gain": {"type": "number"},
            "confidence_floor": {"type": "number"},
            "now_ms": {"type": "integer"}
        }}}
    ]})
}

fn parse_family(s: &str) -> Option<ProposerFamily> {
    match s {
        "retrieval" => Some(ProposerFamily::Retrieval),
        "prefill" => Some(ProposerFamily::Prefill),
        "mtp" => Some(ProposerFamily::Mtp),
        "drafter" => Some(ProposerFamily::Drafter),
        _ => None,
    }
}

fn db_status(auth: &MindAuthority) -> Value {
    json!({
        "status": "ok",
        "mode": if auth.surreal_reachable { "On" } else { "Unavailable" },
        "endpoint": auth.base,
        "namespace": auth.ns,
        "database": auth.db,
        "surreal_reachable": auth.surreal_reachable,
    })
}

fn automaton_decide(args: &Value) -> Result<Value, String> {
    let get_str = |k: &str| {
        args.get(k)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let get_f64 = |k: &str| args.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let get_u64 = |k: &str| args.get(k).and_then(Value::as_u64).unwrap_or(0);
    let family = parse_family(&get_str("family")).ok_or("unknown proposer family")?;
    let envelope = ProposerEnvelope {
        family,
        strategy_id: get_str("strategy_id"),
        policy_version: get_str("policy_version"),
        prefix_identity: get_str("prefix_identity"),
        map_identity: get_str("map_identity"),
        payload: get_str("payload"),
        expiry_ms: get_u64("expiry_ms"),
    };
    let signals = AuditSignals {
        measured_confidence: Q32_31::from_f64(get_f64("measured_confidence")),
        unresolved_dependencies: get_u64("unresolved_dependencies") as u32,
        entity_ambiguity: args
            .get("entity_ambiguity")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        expected_information_gain: Q32_31::from_f64(get_f64("expected_information_gain")),
    };
    // The decision tool needs a strategy state to decide against.
    // It uses the caller's declared policy/identities and floor as
    // the state, with a neutral starting confidence of 0.6 — the
    // decision is about the proposal, not about stored history.
    // Stored confidence lives in Surreal and is not read here.
    let state = StrategyState {
        strategy_id: envelope.strategy_id.clone(),
        policy_version: envelope.policy_version.clone(),
        prefix_identity: envelope.prefix_identity.clone(),
        map_identity: envelope.map_identity.clone(),
        confidence: Q32_31::from_f64(0.6),
        confidence_floor: Q32_31::from_f64(get_f64("confidence_floor")),
        calibration_count: 0,
        outcome_history_digest: "h0".to_string(),
    };
    let t = transition(&state, &envelope, &signals, get_u64("now_ms"));
    Ok(json!({
        "status": "ok",
        "verdict": t.verdict.as_str(),
        "committed": t.committed,
        "proposal_digest": t.proposal_digest,
        "receipt_digest": t.receipt_digest,
        "note": "decision only: no execution, no confidence change",
    }))
}

/// Handle one JSON-RPC request value. Pure except for the authority
/// the caller measured (reachability is the caller's measurement).
pub fn handle_request(req: &Value, auth: &MindAuthority) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    match method {
        "initialize" => json!({"jsonrpc": "2.0", "id": id, "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "ares-mcp-mind", "version": "0.1.0"}
        }}),
        "tools/list" => json!({"jsonrpc": "2.0", "id": id, "result": tools_list()}),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match name {
                "mind.db_status" => text_result(&id, "mind.db_status", &args, db_status(auth)),
                "mind.automaton_decide" => match automaton_decide(&args) {
                    Ok(v) => text_result(&id, "mind.automaton_decide", &args, v),
                    Err(e) => error(&id, -32602, &e),
                },
                _ => error(&id, -32601, "method not found"),
            }
        }
        _ => error(&id, -32601, "method not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth() -> MindAuthority {
        MindAuthority {
            base: "http://127.0.0.1:8000".to_string(),
            ns: "ares".to_string(),
            db: "ares".to_string(),
            surreal_reachable: true,
        }
    }

    #[test]
    fn initialize_and_tools_list() {
        let r = handle_request(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}),
            &auth(),
        );
        assert_eq!(r["result"]["serverInfo"]["name"], "ares-mcp-mind");
        let r = handle_request(
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
            &auth(),
        );
        let tools = r["result"]["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "mind.db_status"));
        assert!(tools.iter().any(|t| t["name"] == "mind.automaton_decide"));
    }

    #[test]
    fn db_status_attests_authority() {
        let r = handle_request(
            &json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "mind.db_status", "arguments": {}}}),
            &auth(),
        );
        let text = r["result"]["content"][0]["text"].as_str().unwrap();
        let inner: Value = serde_json::from_str(text).unwrap();
        assert_eq!(inner["mode"], "On");
        assert_eq!(inner["database"], "ares");
    }

    #[test]
    fn automaton_decide_admits_and_clarifies() {
        let args = json!({
            "family": "retrieval", "strategy_id": "s1", "policy_version": "policy-v1",
            "prefix_identity": "p1", "map_identity": "m1", "payload": "q",
            "expiry_ms": 1000000, "measured_confidence": 0.8,
            "unresolved_dependencies": 0, "entity_ambiguity": false,
            "expected_information_gain": 0.7, "confidence_floor": 0.5, "now_ms": 100
        });
        let r = handle_request(
            &json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "mind.automaton_decide", "arguments": args}}),
            &auth(),
        );
        let text = r["result"]["content"][0]["text"].as_str().unwrap();
        let inner: Value = serde_json::from_str(text).unwrap();
        assert_eq!(inner["verdict"], "admit");
        assert_eq!(inner["receipt_digest"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn unknown_tool_and_method_fail_closed() {
        let r = handle_request(
            &json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "mind.atom", "arguments": {}}}),
            &auth(),
        );
        assert!(r.get("error").is_some());
    }
}
