//! globalmind-mcp: the multimodel mind as an MCP server (stdio, JSON-RPC 2024-11-05).
//!
//! Tools, all from `globalmind-runtime` itself — no backend copy of the store:
//! - `mind.db_status`: authority attestation — mode, endpoint, namespace, database,
//!   SurrealDB reachability at this instant, and contract status. Mode is "On"
//!   only when the store connects and its contract verifies; otherwise "Unavailable".
//! - `mind.recall`: typed hyperbolic recall bundle via `retrieval::retrieve`.
//!   Read-only. Requires GLOBALMIND_SURREAL_USER/PASS; hypermind/core is
//!   quarantined unless GLOBALMIND_ALLOW_QUARANTINED_CORE=1.
//! - `mind.recall_scoped`: read-only persisted cognitive-well projection.
//!   The governor must authorize the well set; this child checks membership
//!   and edges but does not issue worker capabilities.
//! - `mind.kv_exact`: fetch a transformer K/V payload by ExactKvIdentity only,
//!   never by page ID alone. Returns key/value base64 only when the combined
//!   payload is at most 256 KiB; larger payloads return digests, lengths, and
//!   a clear `payload_too_large_for_mcp_text` note — raw tensor bytes stay in
//!   the store / model I/O path, they are never silently truncated here.
//! - `mind.automaton_decide`: deterministic, stateless decision (Admit/Clarify/
//!   Reject) using the backend automaton's rule shape — policy/prefix/map
//!   mismatch or expiry rejects; unresolved dependencies, entity ambiguity,
//!   or confidence below floor clarifies; otherwise admits. It holds NO
//!   confidence state and moves NO confidence; a decision is not execution,
//!   and a caller treating it as execution is misreading the tool.
//!   The receipt is a SHA-256 binding of tool + arguments + inner content,
//!   computed by this bin — it binds the call, it is not a claim that
//!   inner content is true (that is the caller's judge step).
use globalmind_runtime::{
    ExactKvIdentity, GlobalMindStore, RecordId, StoreConfig,
    retrieval::{RetrievalConfig, RetrievalMetric, retrieve, scoped::retrieve_scoped},
};
use serde_json::{Value, json};
use sha2::{Digest as ShaDigest, Sha256};
use std::io::{BufRead, Write};

const MCP_VERSION: &str = "2024-11-05";
const MAX_MCP_PAYLOAD_BYTES: usize = 256 * 1024;

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn receipt(tool: &str, args: &Value, inner: &Value) -> String {
    let mut h = Sha256::new();
    h.update(tool.as_bytes());
    h.update(b"\n");
    h.update(args.to_string().as_bytes());
    h.update(b"\n");
    h.update(inner.to_string().as_bytes());
    format!("{:x}", h.finalize())
}

fn text_result(id: &Value, tool: &str, args: &Value, inner: Value) -> Value {
    let r = receipt(tool, args, &inner);
    let mut obj = inner.as_object().cloned().unwrap_or_default();
    obj.insert("receipt_sha256".to_string(), Value::String(r));
    let text = Value::Object(obj).to_string();
    json!({"jsonrpc": "2.0", "id": id, "result": {"content": [{"type": "text", "text": text}]}})
}

fn tools_list() -> Value {
    json!({"tools": [
        {"name": "mind.db_status", "description": "Attest this mind's SurrealDB authority, reachability, and contract status at this instant.", "inputSchema": {"type": "object", "properties": {}}},
        {"name": "mind.recall", "description": "Typed hyperbolic recall bundle (read-only). Query point must lie on the H3 sheet.", "inputSchema": {"type": "object", "properties": {
            "workspace": {"type": "string"},
            "query_h3": {"type": "array", "items": {"type": "number"}},
            "query_t2": {"type": "array", "items": {"type": "number"}},
            "query_provenance": {"type": "string"},
            "k": {"type": "integer"}
        }, "required": ["workspace", "query_h3", "query_t2", "query_provenance", "k"]}},
        {"name": "mind.recall_scoped", "description": "Read-only recall restricted to persisted cognitive-well memberships and internal edges. Caller must hold a governor-issued scope.", "inputSchema": {"type": "object", "properties": {
            "workspace": {"type": "string"}, "well_ids": {"type": "array", "items": {"type": "string"}},
            "query_h3": {"type": "array", "items": {"type": "number"}},
            "query_t2": {"type": "array", "items": {"type": "number"}},
            "query_provenance": {"type": "string"}, "k": {"type": "integer"}
        }, "required": ["workspace", "well_ids", "query_h3", "query_t2", "query_provenance", "k"]}},
        {"name": "mind.kv_exact", "description": "Fetch a transformer K/V payload by ExactKvIdentity only, never by page ID alone.", "inputSchema": {"type": "object", "properties": {
            "workspace": {"type": "string"}, "session": {"type": "string"}, "page": {"type": "string"},
            "model_id": {"type": "string"}, "model_revision": {"type": "string"},
            "model_digest": {"type": "string"}, "request_digest": {"type": "string"}, "prefix_digest": {"type": "string"},
            "tppe_version": {"type": "string"}, "tppe_configuration_digest": {"type": "string"},
            "address_contract_digest": {"type": "string"}, "tensor_contract_digest": {"type": "string"},
            "geometry_space": {"type": "string"}, "dtype": {"type": "string"},
            "layer_count": {"type": "integer"}, "kv_head_count": {"type": "integer"}, "head_dim": {"type": "integer"},
            "layer": {"type": "integer"}, "kv_head": {"type": "integer"},
            "token_start": {"type": "integer"}, "token_end": {"type": "integer"},
            "tensor_shape": {"type": "array", "items": {"type": "integer"}},
            "layout": {"type": "string"}, "address": {"type": "string"}, "phase_state_digest": {"type": "string"},
            "revision": {"type": "integer"}, "key_digest": {"type": "string"}, "value_digest": {"type": "string"}
        }}},
        {"name": "mind.automaton_decide", "description": "Deterministic, stateless automaton decision (Admit/Clarify/Reject). Does not execute, holds no confidence state, moves no confidence.", "inputSchema": {"type": "object", "properties": {
            "policy_version": {"type": "string"}, "state_policy_version": {"type": "string"},
            "prefix_identity": {"type": "string"}, "state_prefix_identity": {"type": "string"},
            "map_identity": {"type": "string"}, "state_map_identity": {"type": "string"},
            "expiry_ms": {"type": "integer"}, "now_ms": {"type": "integer"},
            "unresolved_dependencies": {"type": "integer"}, "entity_ambiguity": {"type": "boolean"},
            "measured_confidence": {"type": "number"}, "confidence_floor": {"type": "number"}
        }}}
    ]})
}

fn get_str(args: &Value, k: &str) -> Result<String, String> {
    args.get(k)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing string argument: {}", k))
}

fn get_f64(args: &Value, k: &str) -> f64 {
    args.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

fn get_u64(args: &Value, k: &str) -> u64 {
    args.get(k).and_then(Value::as_u64).unwrap_or(0)
}

fn parse_f64_array(args: &Value, k: &str, n: usize) -> Result<Vec<f64>, String> {
    let arr = args
        .get(k)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array argument: {}", k))?;
    if arr.len() != n {
        return Err(format!("argument {} must have {} elements", k, n));
    }
    arr.iter()
        .map(|v| {
            v.as_f64()
                .ok_or_else(|| format!("argument {} must be numbers", k))
        })
        .collect()
}

fn db_status() -> Value {
    let cfg = StoreConfig::from_env();
    let (endpoint, ns, db) = (
        cfg.endpoint.clone(),
        cfg.namespace.clone(),
        cfg.database.clone(),
    );
    let (mode, contract): (&str, String) = match GlobalMindStore::connect(cfg) {
        Ok(store) => match store.verify_recall_kv_contract() {
            Ok(()) => ("On", "verified".to_string()),
            Err(e) => ("Unavailable", e),
        },
        Err(e) => ("Unavailable", e),
    };
    json!({
        "status": "ok",
        "mode": mode,
        "endpoint": endpoint,
        "namespace": ns,
        "database": db,
        "contract": contract,
        "contract_scope": "immutable atom_address and exact transformer K/V fields required by mind.recall and mind.kv_exact; full distillation contract is checked separately before writes",
    })
}

fn recall(args: &Value) -> Result<Value, String> {
    let workspace = RecordId::parse(get_str(args, "workspace")?)?;
    let h3v = parse_f64_array(args, "query_h3", 4)?;
    let t2v = parse_f64_array(args, "query_t2", 2)?;
    let provenance = get_str(args, "query_provenance")?;
    let k = get_u64(args, "k") as u32;
    let config = RetrievalConfig::from_env(workspace, RetrievalMetric::Hyperbolic)?;
    let bundle = retrieve(
        &config,
        [h3v[0], h3v[1], h3v[2], h3v[3]],
        [t2v[0], t2v[1]],
        &provenance,
        k,
    )?;
    Ok(json!({
        "workspace": bundle.workspace.as_str(),
        "query_digest": bundle.query_digest.as_str(),
        "query_provenance": bundle.query_provenance,
        "metric": bundle.metric.as_str(),
        "candidates_scanned": bundle.candidates_scanned,
        "ranking_digest": bundle.ranking_digest.as_str(),
        "members": bundle.members.iter().map(|m| json!({
            "atom": m.atom.as_str(),
            "address": m.address.as_str(),
            "content_digest": m.content_digest.as_str(),
            "salience": m.salience,
            "domain": m.domain,
            "score": m.score,
            "rank": m.rank,
        })).collect::<Vec<_>>(),
    }))
}

fn recall_scoped(args: &Value) -> Result<Value, String> {
    let workspace = RecordId::parse(get_str(args, "workspace")?)?;
    let wells = args
        .get("well_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing well_ids array".to_string())?;
    if wells.is_empty() || wells.len() > 12 {
        return Err("well_ids must contain 1..=12 entries".into());
    }
    let well_ids = wells
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| "well_ids must be strings".to_string())
                .and_then(RecordId::parse)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let h3v = parse_f64_array(args, "query_h3", 4)?;
    let t2v = parse_f64_array(args, "query_t2", 2)?;
    let provenance = get_str(args, "query_provenance")?;
    let k = u32::try_from(get_u64(args, "k")).map_err(|_| "k out of range".to_string())?;
    let config = RetrievalConfig::from_env(workspace, RetrievalMetric::Hyperbolic)?;
    let scoped = retrieve_scoped(
        &config,
        &well_ids,
        [h3v[0], h3v[1], h3v[2], h3v[3]],
        [t2v[0], t2v[1]],
        &provenance,
        k,
    )?;
    Ok(json!({
        "workspace": scoped.bundle.workspace.as_str(),
        "well_ids": scoped.well_ids.iter().map(RecordId::as_str).collect::<Vec<_>>(),
        "scope_digest": scoped.scope_digest.as_str(),
        "query_digest": scoped.bundle.query_digest.as_str(),
        "ranking_digest": scoped.bundle.ranking_digest.as_str(),
        "candidates_scanned": scoped.bundle.candidates_scanned,
        "members": scoped.bundle.members.iter().map(|m| json!({
            "atom": m.atom.as_str(), "address": m.address.as_str(),
            "content_digest": m.content_digest.as_str(), "score": m.score, "rank": m.rank,
        })).collect::<Vec<_>>(),
        "edges": scoped.edges.iter().map(|e| json!({
            "id": e.id.as_str(), "from": e.from.as_str(), "to": e.to.as_str(), "kind": e.kind,
        })).collect::<Vec<_>>(),
    }))
}

fn kv_exact(args: &Value) -> Result<Value, String> {
    use globalmind_runtime::{Digest, TensorDtype, TensorLayout};
    let dtype = match get_str(args, "dtype")?.as_str() {
        "f16" => TensorDtype::F16,
        "bf16" => TensorDtype::Bf16,
        "f32" => TensorDtype::F32,
        "q8_0" => TensorDtype::Q8_0,
        "q4_0" => TensorDtype::Q4_0,
        other => return Err(format!("unknown dtype: {}", other)),
    };
    let layout = match get_str(args, "layout")?.as_str() {
        "token_major" => TensorLayout::TokenMajor,
        "head_dim_major" => TensorLayout::HeadDimMajor,
        other => return Err(format!("unknown layout: {}", other)),
    };
    let shape = parse_f64_array(args, "tensor_shape", 2)?;
    let q = ExactKvIdentity {
        workspace: RecordId::parse(get_str(args, "workspace")?)?,
        session: RecordId::parse(get_str(args, "session")?)?,
        page: RecordId::parse(get_str(args, "page")?)?,
        model_id: get_str(args, "model_id")?,
        model_revision: get_str(args, "model_revision")?,
        model_digest: Digest::parse(get_str(args, "model_digest")?)?,
        request_digest: Digest::parse(get_str(args, "request_digest")?)?,
        prefix_digest: Digest::parse(get_str(args, "prefix_digest")?)?,
        tppe_version: get_str(args, "tppe_version")?,
        tppe_configuration_digest: Digest::parse(get_str(args, "tppe_configuration_digest")?)?,
        address_contract_digest: Digest::parse(get_str(args, "address_contract_digest")?)?,
        tensor_contract_digest: Digest::parse(get_str(args, "tensor_contract_digest")?)?,
        geometry_space: RecordId::parse(get_str(args, "geometry_space")?)?,
        dtype,
        layer_count: get_u64(args, "layer_count") as u32,
        kv_head_count: get_u64(args, "kv_head_count") as u32,
        head_dim: get_u64(args, "head_dim") as u32,
        layer: get_u64(args, "layer") as u32,
        kv_head: get_u64(args, "kv_head") as u32,
        token_start: get_u64(args, "token_start"),
        token_end: get_u64(args, "token_end"),
        tensor_shape: [shape[0] as u32, shape[1] as u32],
        layout,
        address: RecordId::parse(get_str(args, "address")?)?,
        phase_state_digest: Digest::parse(get_str(args, "phase_state_digest")?)?,
        revision: get_u64(args, "revision") as u32,
        key_digest: Digest::parse(get_str(args, "key_digest")?)?,
        value_digest: Digest::parse(get_str(args, "value_digest")?)?,
    };
    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    let payload = store.fetch_tensor_payload_exact(&q)?;
    let total = payload.key.len() + payload.value.len();
    let mut out = json!({
        "session": q.session.as_str(),
        "page": q.page.as_str(),
        "key_bytes": payload.key.len(),
        "value_bytes": payload.value.len(),
        "key_digest": q.key_digest.as_str(),
        "value_digest": q.value_digest.as_str(),
    });
    if total <= MAX_MCP_PAYLOAD_BYTES {
        use base64_helper::encode as b64;
        out["key_base64"] = Value::String(b64(&payload.key));
        out["value_base64"] = Value::String(b64(&payload.value));
    } else {
        out["payload_too_large_for_mcp_text"] = Value::Bool(true);
        out["payload_note"] = Value::String("Raw tensor bytes stay in the store / model I/O path; this text result carries digests and lengths only, never a silent truncation.".into());
    }
    Ok(out)
}

fn automaton_decide(args: &Value) -> Result<Value, String> {
    let policy = get_str(args, "policy_version").unwrap_or_default();
    let state_policy = get_str(args, "state_policy_version").unwrap_or_default();
    let prefix = get_str(args, "prefix_identity").unwrap_or_default();
    let state_prefix = get_str(args, "state_prefix_identity").unwrap_or_default();
    let map = get_str(args, "map_identity").unwrap_or_default();
    let state_map = get_str(args, "state_map_identity").unwrap_or_default();
    let expiry = get_u64(args, "expiry_ms");
    let now = get_u64(args, "now_ms");
    let unresolved = get_u64(args, "unresolved_dependencies");
    let ambiguity = args
        .get("entity_ambiguity")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let confidence = get_f64(args, "measured_confidence");
    let floor = get_f64(args, "confidence_floor");
    let verdict =
        if policy != state_policy || prefix != state_prefix || map != state_map || now > expiry {
            "Reject"
        } else if unresolved > 0 || ambiguity || confidence < floor {
            "Clarify"
        } else {
            "Admit"
        };
    Ok(json!({
        "verdict": verdict,
        "committed": verdict == "Admit",
        "confidence_moved": false,
        "note": "Stateless decision only. No confidence state is held or moved by this tool.",
    }))
}

fn handle_request(req: &Value) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    match method {
        "initialize" => json!({"jsonrpc": "2.0", "id": id, "result": {
            "protocolVersion": MCP_VERSION,
            "serverInfo": {"name": "globalmind-mcp", "version": "0.1.0"},
            "capabilities": {"tools": {}},
        }}),
        "tools/list" => json!({"jsonrpc": "2.0", "id": id, "result": tools_list()}),
        "ping" => json!({"jsonrpc": "2.0", "id": id, "result": {}}),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let inner = match name {
                "mind.db_status" => Ok(db_status()),
                "mind.recall" => recall(&args),
                "mind.recall_scoped" => recall_scoped(&args),
                "mind.kv_exact" => kv_exact(&args),
                "mind.automaton_decide" => automaton_decide(&args),
                _ => Err(format!("unknown tool: {}", name)),
            };
            match inner {
                Ok(v) => text_result(&id, name, &args, v),
                Err(e) => error(&id, -32602, &e),
            }
        }
        "notifications/initialized" | "notifications/cancelled" => Value::Null,
        _ => error(&id, -32601, &format!("unknown method: {}", method)),
    }
}

// Minimal base64 (no extra crate dependency in globalmind-runtime).
mod base64_helper {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    pub fn encode(data: &[u8]) -> String {
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let b = [
                chunk[0],
                *chunk.get(1).unwrap_or(&0),
                *chunk.get(2).unwrap_or(&0),
            ];
            let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
            out.push(TABLE[(n >> 18) as usize & 63] as char);
            out.push(TABLE[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                TABLE[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(req) => handle_request(&req),
            Err(_) => {
                json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}})
            }
        };
        if resp != Value::Null {
            let _ = writeln!(stdout, "{}", resp);
            let _ = stdout.flush();
        }
    }
}
