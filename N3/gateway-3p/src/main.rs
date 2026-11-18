// gateway-3p: Anthropic-protocol gateway for cloud models plus the local oMLX lane.
// Listens only on loopback. Keys are read from the environment at startup,
// never written to this file, to git, or to any log.
// Claude Desktop requires an Anthropic-looking model ID; that ID maps to the
// local OpenAI-compatible oMLX server (127.0.0.1:8002) and its Qwen3.6-MoE-Combined model.
use axum::{
    body::Body,
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct Route {
    anthropic_id: &'static str,
    display: &'static str,
    provider_base: &'static str,
    provider_model: &'static str,
    /// Keys tried in order; a key that hits quota/rate limits is cooled down
    /// and the next one is used. Missing/empty env vars are skipped.
    key_envs: &'static [&'static str],
    /// Other Desktop model IDs to fail over to (in order) when every key for
    /// this route is exhausted, rate-limited, or the provider is down.
    fallbacks: &'static [&'static str],
}

const ROUTES: &[Route] = &[
    Route {
        anthropic_id: "claude-opus-4",
        display: "oMLX - Qwen3.6-MoE-Combined",
        provider_base: "http://127.0.0.1:8002/v1",
        provider_model: "Qwen3.6-MoE-Combined",
        key_envs: &["GATEWAY_OMLX_KEY"],
        fallbacks: &[],
    },
    Route {
        anthropic_id: "claude-sonnet-5",
        display: "Beam 1.0 501B A23B · Reflection AI",
        provider_base: "https://api.reflection.ai/openai/v1",
        provider_model: "Beam-1.0-501B-A23B",
        key_envs: &["GATEWAY_REFLECTION_KEY", "GATEWAY_REFLECTION2_KEY"],
        fallbacks: &["claude-3p-muse", "claude-haiku-4-5"],
    },
    Route {
        anthropic_id: "claude-3p-muse",
        display: "Muse Spark 1.3 Contributor · Meta",
        provider_base: "https://api.meta.ai/v1",
        provider_model: "muse-spark-1.3-contributor",
        key_envs: &["GATEWAY_META_KEY"],
        fallbacks: &["claude-sonnet-5", "claude-haiku-4-5"],
    },
    Route {
        anthropic_id: "claude-haiku-4-5",
        display: "Nemotron 3 Ultra 550B A55B · NVIDIA",
        provider_base: "https://integrate.api.nvidia.com/v1",
        provider_model: "nvidia/nemotron-3-ultra-550b-a55b",
        key_envs: &["GATEWAY_NVIDIA_KEY"],
        fallbacks: &["claude-3p-muse", "claude-sonnet-5"],
    },
];

fn find_route(id: &str) -> Option<&'static Route> {
    ROUTES.iter().find(|r| r.anthropic_id == id)
}

/// Primary route followed by its de-duplicated fallbacks.
fn route_chain(primary: &'static Route) -> Vec<&'static Route> {
    let mut chain = vec![primary];
    for id in primary.fallbacks {
        if let Some(r) = find_route(id) {
            if !chain.iter().any(|c| c.anthropic_id == r.anthropic_id) {
                chain.push(r);
            }
        }
    }
    chain
}

/// One pooled HTTP client for all providers: keeps TLS connections warm
/// instead of re-handshaking on every Claude turn.
fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            // Per-read idle limit (not total): reasoning models can pause long
            // before the first token, but a dead socket must not hang forever.
            .read_timeout(Duration::from_secs(300))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .expect("http client")
    })
}

/// Cooldowns keyed by "provider_model|KEY_ENV". Values are never stored.
fn cooldowns() -> &'static Mutex<HashMap<String, Instant>> {
    static MAP: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_cooling(slot: &str) -> bool {
    let mut m = cooldowns().lock().unwrap();
    match m.get(slot) {
        Some(until) if *until > Instant::now() => true,
        Some(_) => {
            m.remove(slot);
            false
        }
        None => false,
    }
}

fn cool_down(slot: &str, secs: u64) {
    cooldowns()
        .lock()
        .unwrap()
        .insert(slot.to_string(), Instant::now() + Duration::from_secs(secs));
}

fn secs_until_utc_midnight() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    86_400 - (now % 86_400) + 30
}

/// How long to bench a provider key after a failed status, or None when the
/// failure is about the request itself (fail fast, don't burn other providers).
fn cooldown_for(status: u16, body: &str, retry_after: Option<u64>) -> Option<u64> {
    let daily = body.contains("daily_allowance_exhausted") || body.contains("insufficient_quota");
    match status {
        429 if daily => Some(secs_until_utc_midnight()),
        402 => Some(secs_until_utc_midnight()),
        429 => Some(retry_after.unwrap_or(30).clamp(5, 600)),
        401 | 403 => Some(600),
        408 | 500..=599 => Some(15),
        _ => None,
    }
}

/// Char-boundary-safe log/err preview.
fn preview(s: &str) -> String {
    s.chars().take(500).collect()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct AnthropicMsg {
    role: String,
    content: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct AnthropicReq {
    model: String,
    messages: Vec<AnthropicMsg>,
    #[serde(default)]
    system: Value,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    tools: Value,
    #[serde(default)]
    thinking: Value,
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

fn blocks_of(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a.clone(),
        Value::String(s) if !s.is_empty() => vec![json!({"type": "text", "text": s})],
        _ => vec![],
    }
}

fn provider_tool_name(name: &str) -> String {
    let already_valid = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if already_valid {
        return name.to_string();
    }

    // OpenAI-compatible servers cap function names at 64 ASCII characters,
    // while Claude Code can expose longer MCP/plugin names. Keep a readable
    // prefix and a deterministic FNV-1a suffix so aliases remain collision
    // resistant and can be mapped back before Claude executes the tool.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let mut prefix: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(46)
        .collect();
    if prefix.is_empty() {
        prefix.push_str("tool");
    }
    format!("{prefix}__{hash:016x}")
}

fn provider_to_anthropic_tool_names(tools: &Value) -> HashMap<String, String> {
    match tools {
        Value::Array(items) => items
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .map(|name| (provider_tool_name(name), name.to_string()))
            .collect(),
        _ => HashMap::new(),
    }
}

fn anthropic_tools_to_openai(tools: &Value) -> Vec<Value> {
    let arr = match tools {
        Value::Array(a) => a.clone(),
        _ => return vec![],
    };
    arr.into_iter().filter_map(|t| {
        let name = provider_tool_name(t.get("name").and_then(|n| n.as_str())?);
        let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
        let mut schema = t.get("input_schema").cloned().unwrap_or_else(|| json!({"type":"object"}));
        if !schema.is_object() {
            schema = json!({"type":"object"});
        }
        Some(json!({"type":"function","function":{"name":name,"description":desc,"parameters":schema}}))
    }).collect()
}

fn to_openai(req: &AnthropicReq, route: &Route) -> Value {
    let mut msgs: Vec<Value> = Vec::new();
    let sys = text_of(&req.system);
    if !sys.is_empty() {
        msgs.push(json!({"role": "system", "content": sys}));
    }
    let mut pending_tool_calls: Vec<String> = Vec::new();
    for m in &req.messages {
        let blocks = blocks_of(&m.content);
        let mut text_parts: Vec<String> = vec![];
        let mut reasoning_parts: Vec<String> = vec![];
        let mut tool_calls: Vec<Value> = vec![];
        let mut tool_results: Vec<Value> = vec![];
        for b in &blocks {
            let btype = b.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match btype {
                "text" => {
                    if let Some(t) = b.get("text").and_then(|t| t.as_str()) {
                        if !t.is_empty() {
                            text_parts.push(t.to_string());
                        }
                    }
                }
                "thinking" => {
                    if let Some(t) = b.get("thinking").and_then(|t| t.as_str()) {
                        if !t.is_empty() {
                            reasoning_parts.push(t.to_string());
                        }
                    }
                }
                "tool_use" => {
                    let id = b
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("call_0")
                        .to_string();
                    let name =
                        provider_tool_name(b.get("name").and_then(|v| v.as_str()).unwrap_or(""));
                    let input = b.get("input").cloned().unwrap_or(json!({}));
                    let args = serde_json::to_string(&input).unwrap_or("{}".to_string());
                    tool_calls.push(json!({"id":id,"type":"function","function":{"name":name,"arguments":args}}));
                }
                "tool_result" => {
                    let cid = b
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("call_0")
                        .to_string();
                    let c = b.get("content").cloned().unwrap_or(Value::Null);
                    tool_results
                        .push(json!({"tool_call_id":cid,"role":"tool","content":text_of(&c)}));
                }
                _ => {}
            }
        }

        // 1. If this message carries tool results, push them first so tool calls are answered immediately
        for tr in tool_results {
            if let Some(cid) = tr.get("tool_call_id").and_then(|v| v.as_str()) {
                if let Some(pos) = pending_tool_calls.iter().position(|id| id == cid) {
                    pending_tool_calls.remove(pos);
                }
            }
            msgs.push(tr);
        }

        // 2. OpenAI requires that any assistant tool calls be answered before another message
        if !pending_tool_calls.is_empty()
            && (!text_parts.is_empty() || !tool_calls.is_empty() || m.role != "user")
        {
            for missing_id in pending_tool_calls.drain(..) {
                msgs.push(json!({
                    "role": "tool",
                    "tool_call_id": missing_id,
                    "content": "ok"
                }));
            }
        }

        if !tool_calls.is_empty() {
            let assistant_text = text_parts.join("");
            let content_val = if assistant_text.is_empty() {
                Value::Null
            } else {
                Value::String(assistant_text.clone())
            };
            let mut amsg = json!({"role":"assistant","content": content_val});
            let reasoning = reasoning_parts.join("");
            if !reasoning.is_empty() {
                amsg["reasoning_content"] = Value::String(reasoning);
            } else if route.provider_model == "Beam-1.0-501B-A23B" && !assistant_text.is_empty() {
                // Beam requires the exact prior reasoning on subsequent turns.
                amsg["reasoning_content"] = Value::String(assistant_text);
            }
            if route.provider_model == "Beam-1.0-501B-A23B" && amsg.get("reasoning_content").is_none() {
                amsg["reasoning_content"] = Value::String("Executing tool calls.".to_string());
            }
            for tc in &tool_calls {
                if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                    pending_tool_calls.push(id.to_string());
                }
            }
            amsg["tool_calls"] = Value::Array(tool_calls);
            msgs.push(amsg);
        } else if m.role == "user" {
            let user_text = text_parts.join("");
            if !user_text.is_empty() {
                msgs.push(json!({"role": "user", "content": user_text}));
            }
        } else {
            let message_text = text_parts.join("");
            let content_val = if message_text.is_empty() {
                " ".to_string()
            } else {
                message_text.clone()
            };
            let mut message = json!({"role": m.role, "content": content_val});
            let reasoning = reasoning_parts.join("");
            if !reasoning.is_empty() {
                message["reasoning_content"] = Value::String(reasoning);
            } else if route.provider_model == "Beam-1.0-501B-A23B"
                && m.role == "assistant"
                && !message_text.is_empty()
            {
                message["reasoning_content"] = Value::String(message_text.clone());
            }
            if route.provider_model == "Beam-1.0-501B-A23B"
                && m.role == "assistant"
                && message.get("reasoning_content").is_none()
            {
                message["reasoning_content"] = Value::String(if message_text.is_empty() { "Thinking.".to_string() } else { message_text.clone() });
            }
            msgs.push(message);
        }
    }
    // Drain any remaining unanswered tool calls before finalizing body
    for missing_id in pending_tool_calls.drain(..) {
        msgs.push(json!({
            "role": "tool",
            "tool_call_id": missing_id,
            "content": "ok"
        }));
    }
    let mut max_tokens = match req.max_tokens {
        Some(m) if m < 8192 && req.stream => 16384,
        Some(m) => m,
        None => 32768,
    };
    if let Some(budget) = req.thinking.get("budget_tokens").and_then(|b| b.as_u64()) {
        max_tokens = max_tokens.max((budget as u32) + 4096);
    }
    let mut body = json!({
        "model": route.provider_model,
        "messages": msgs,
        "max_tokens": max_tokens,
        "stream": req.stream,
    });
    let otools = anthropic_tools_to_openai(&req.tools);
    if !otools.is_empty() {
        body["tools"] = Value::Array(otools);
        body["tool_choice"] = json!("auto");
    }
    body
}

async fn models() -> Json<Value> {
    Json(json!({
        "object": "list",
        "data": ROUTES.iter().map(|r| json!({
            "id": r.anthropic_id,
            "object": "model",
            "display_name": r.display,
        })).collect::<Vec<_>>()
    }))
}

async fn messages(headers: HeaderMap, Json(req): Json<AnthropicReq>) -> Response {
    let Some(primary) = find_route(&req.model) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"type": "error", "error": {"type": "not_found_error", "message": format!("no route for model {}", req.model)}})),
        )
            .into_response();
    };
    // Caller key, if any, is not forwarded and is never logged.
    let _ = &headers;
    let provider_to_anthropic = provider_to_anthropic_tool_names(&req.tools);
    let mut last_err: Option<(StatusCode, String)> = None;
    let mut chosen: Option<(&'static Route, reqwest::Response)> = None;
    'routes: for cand in route_chain(primary) {
        for key_env in cand.key_envs {
            let key = match std::env::var(key_env) {
                Ok(k) if !k.is_empty() => k,
                _ => continue,
            };
            let slot = format!("{}|{}", cand.provider_model, key_env);
            if is_cooling(&slot) {
                continue;
            }
            let body = to_openai(&req, cand);
            let url = format!("{}/chat/completions", cand.provider_base.trim_end_matches('/'));
            match http().post(&url).bearer_auth(&key).json(&body).send().await {
                Err(e) => {
                    eprintln!("gateway provider {} via {} transport error: {}", cand.provider_model, key_env, e);
                    cool_down(&slot, 15);
                    last_err = Some((StatusCode::BAD_GATEWAY, e.to_string()));
                }
                Ok(r) if r.status().is_success() => {
                    if cand.anthropic_id != primary.anthropic_id || *key_env != primary.key_envs[0] {
                        eprintln!("gateway failover: {} -> {} via {}", primary.provider_model, cand.provider_model, key_env);
                    }
                    chosen = Some((cand, r));
                    break 'routes;
                }
                Ok(r) => {
                    let code = r.status().as_u16();
                    let ra = r
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.trim().parse::<u64>().ok());
                    let text = r.text().await.unwrap_or_default();
                    eprintln!("gateway provider {} status {} body_preview {}", cand.provider_model, code, preview(&text));
                    eprintln!(
                        "gateway provider error: outgoing messages={} tools={} max_tokens={}",
                        body["messages"].as_array().map_or(0, |a| a.len()),
                        body["tools"].as_array().map_or(0, |a| a.len()),
                        body["max_tokens"]
                    );
                    let st = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_GATEWAY);
                    let msg = format!("provider {} status {}: {}", cand.provider_model, code, preview(&text));
                    match cooldown_for(code, &text, ra) {
                        Some(secs) => {
                            eprintln!("gateway cooldown {} for {}s", slot, secs);
                            cool_down(&slot, secs);
                            last_err = Some((st, msg));
                        }
                        None => {
                            return (
                                StatusCode::BAD_GATEWAY,
                                Json(json!({"type": "error", "error": {"type": "api_error", "message": msg}})),
                            )
                                .into_response();
                        }
                    }
                }
            }
        }
    }
    let Some((route, resp)) = chosen else {
        let (st, msg) = last_err.unwrap_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "every provider for this model is cooling down or has no key configured".to_string(),
        ));
        let etype = match st.as_u16() {
            429 => "rate_limit_error",
            401 | 403 => "authentication_error",
            _ => "api_error",
        };
        return (st, Json(json!({"type": "error", "error": {"type": etype, "message": msg}}))).into_response();
    };
    let provider_status = resp.status();
    if !req.stream {
        let status = provider_status;
        let text_body = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"type": "error", "error": {"type": "api_error", "message": e.to_string()}})),
                )
                    .into_response();
            }
        };
        // Provider diagnostics: status + first 500 chars of the body only.
        // Never log the key, the request body, or the full response.
        eprintln!(
            "gateway provider {} status {} body_preview {}",
            route.provider_model,
            status,
            preview(&text_body)
        );
        let v: Value = match serde_json::from_str(&text_body) {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"type": "error", "error": {"type": "api_error", "message": format!("provider status {}: {} (body preview: {})", status, e, preview(&text_body))}})),
                )
                    .into_response();
            }
        };
        let choice = v
            .get("choices")
            .and_then(|c| c.get(0))
            .cloned()
            .unwrap_or(Value::Null);
        let finish = choice
            .get("finish_reason")
            .and_then(|f| f.as_str())
            .unwrap_or("stop")
            .to_string();
        let msg = choice.get("message").cloned().unwrap_or(Value::Null);
        let text = msg
            .get("content")
            .and_then(|c| c.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| {
                msg.get("reasoning_content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();
        let mut content: Vec<Value> = vec![];
        if !text.is_empty() {
            content.push(json!({"type": "text", "text": text}));
        }
        let mut stop_reason = "end_turn";
        if let Some(tcs) = msg.get("tool_calls").and_then(|t| t.as_array()) {
            for tc in tcs {
                let cid = tc
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("call_0")
                    .to_string();
                let provider_fname = tc
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let fname = provider_to_anthropic
                    .get(&provider_fname)
                    .cloned()
                    .unwrap_or(provider_fname);
                let args_s = tc
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("{}");
                let input: Value = serde_json::from_str(args_s).unwrap_or(json!({}));
                content.push(json!({"type": "tool_use", "id": cid, "name": fname, "input": input}));
            }
            if !content
                .iter()
                .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
            {
            } else {
                stop_reason = "tool_use";
            }
        } else if finish == "tool_calls" {
            stop_reason = "tool_use";
        }
        if content.is_empty() {
            content.push(json!({"type": "text", "text": ""}));
        }
        let usage = v.get("usage").cloned().unwrap_or(json!({}));
        return Json(json!({
            "id": "msg_gateway",
            "type": "message",
            "role": "assistant",
            "content": content,
            "model": req.model,
            "stop_reason": stop_reason,
            "usage": {
                "input_tokens": usage.get("prompt_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
                "output_tokens": usage.get("completion_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
            }
        }))
        .into_response();
    }
    // Stream: translate OpenAI SSE deltas to Anthropic SSE events.
    let mut stream = resp.bytes_stream();
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<String, std::io::Error>>(32);
    tokio::spawn(async move {
        let _ = tx
            .send(Ok(format!(
                "event: message_start\ndata: {}\n\n",
                json!({"type": "message_start", "message": {"id": "msg_gateway", "type": "message", "role": "assistant", "content": [], "model": req.model, "stop_reason": null, "usage": {"input_tokens": 0, "output_tokens": 0}}})
            )))
            .await;
        let _ = tx
            .send(Ok(format!(
                "event: content_block_start\ndata: {}\n\n",
                json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}})
            )))
            .await;
        let mut buf = String::new();
        #[derive(Default)]
        struct ToolBlock {
            index: usize,
            id: String,
            name: String,
            pending_args: String,
            started: bool,
        }
        let mut tool_blocks: std::collections::BTreeMap<u64, ToolBlock> =
            std::collections::BTreeMap::new();
        let mut next_index: usize = 1;
        let mut saw_tool_call = false;
        while let Some(chunk) = stream.next().await {
            let bytes = match chunk {
                Ok(b) => b,
                Err(_) => break,
            };
            buf.push_str(&String::from_utf8_lossy(&bytes));
            while let Some(pos) = buf.find("\n\n") {
                let block = buf[..pos].to_string();
                buf = buf[pos + 2..].to_string();
                for line in block.lines() {
                    let line = line.trim_start_matches("data:").trim();
                    if line.is_empty() || line == "[DONE]" {
                        continue;
                    }
                    if let Ok(v) = serde_json::from_str::<Value>(line) {
                        let ch = v
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let delta = ch.get("delta").cloned().unwrap_or(Value::Null);
                        if let Some(t) = delta.get("content").and_then(|c| c.as_str()) {
                            if !t.is_empty() {
                                let _ = tx.send(Ok(format!("event: content_block_delta\ndata: {}\n\n", json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": t}})))).await;
                            }
                        }
                        // Provider reasoning_content is intentionally not forwarded as
                        // visible text; it flooded the reply with chain-of-thought.
                        if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                            for tc in tcs {
                                let idx = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                                let entry = tool_blocks.entry(idx).or_insert_with(|| {
                                    let index = next_index;
                                    next_index += 1;
                                    ToolBlock {
                                        index,
                                        ..ToolBlock::default()
                                    }
                                });
                                if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                                    entry.id.push_str(id);
                                }
                                if let Some(f) = tc.get("function") {
                                    if let Some(n) = f.get("name").and_then(|v| v.as_str()) {
                                        entry.name.push_str(n);
                                    }
                                    if let Some(a) = f.get("arguments").and_then(|v| v.as_str()) {
                                        entry.pending_args.push_str(a);
                                    }
                                }
                                // Anthropic requires content_block_start before any
                                // input_json_delta. OpenAI normally sends id/name in
                                // the first tool delta, but arguments may share it.
                                if !entry.started
                                    && !entry.id.is_empty()
                                    && !entry.name.is_empty()
                                    && !entry.pending_args.is_empty()
                                {
                                    let name = provider_to_anthropic
                                        .get(&entry.name)
                                        .cloned()
                                        .unwrap_or_else(|| entry.name.clone());
                                    let _ = tx.send(Ok(format!("event: content_block_start\ndata: {}\n\n", json!({"type": "content_block_start", "index": entry.index, "content_block": {"type": "tool_use", "id": entry.id, "name": name, "input": {}}})))).await;
                                    entry.started = true;
                                }
                                if entry.started && !entry.pending_args.is_empty() {
                                    let args = std::mem::take(&mut entry.pending_args);
                                    let _ = tx.send(Ok(format!("event: content_block_delta\ndata: {}\n\n", json!({"type": "content_block_delta", "index": entry.index, "delta": {"type": "input_json_delta", "partial_json": args}})))).await;
                                }
                                saw_tool_call = true;
                            }
                        }
                    }
                }
            }
        }
        for block in tool_blocks.values_mut() {
            if !block.started {
                let id = if block.id.is_empty() {
                    "call_0"
                } else {
                    &block.id
                };
                let provider_name = if block.name.is_empty() {
                    "unknown_tool".to_string()
                } else {
                    block.name.clone()
                };
                let name = provider_to_anthropic
                    .get(&provider_name)
                    .cloned()
                    .unwrap_or(provider_name);
                let _ = tx.send(Ok(format!("event: content_block_start\ndata: {}\n\n", json!({"type": "content_block_start", "index": block.index, "content_block": {"type": "tool_use", "id": id, "name": name, "input": {}}})))).await;
            }
            if !block.pending_args.is_empty() {
                let args = std::mem::take(&mut block.pending_args);
                let _ = tx.send(Ok(format!("event: content_block_delta\ndata: {}\n\n", json!({"type": "content_block_delta", "index": block.index, "delta": {"type": "input_json_delta", "partial_json": args}})))).await;
            }
            let _ = tx
                .send(Ok(format!(
                    "event: content_block_stop\ndata: {}\n\n",
                    json!({"type": "content_block_stop", "index": block.index})
                )))
                .await;
        }
        let _ = saw_tool_call;
        let _ = tx
            .send(Ok(format!(
                "event: content_block_stop\ndata: {}\n\n",
                json!({"type": "content_block_stop", "index": 0})
            )))
            .await;
        let _ = tx
            .send(Ok(format!(
                "event: message_delta\ndata: {}\n\n",
                json!({"type": "message_delta", "delta": {"stop_reason": if saw_tool_call { "tool_use" } else { "end_turn" }, "stop_sequence": null}, "usage": {"output_tokens": 0}})
            )))
            .await;
        let _ = tx
            .send(Ok(format!(
                "event: message_stop\ndata: {}\n\n",
                json!({"type": "message_stop"})
            )))
            .await;
    });
    let body_stream =
        tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| r.map(bytes::Bytes::from));
    // bytes crate is pulled via axum; use axum Body from stream
    Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(Body::from_stream(body_stream))
        .unwrap()
}

#[derive(Serialize)]
struct Health {
    ok: bool,
    routes: Vec<&'static str>,
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        routes: ROUTES.iter().map(|r| r.anthropic_id).collect(),
    })
}

// Keys file: the server carries its own keys, so no per-launch export
// is needed. Path: $GATEWAY_KEYS_FILE if set, else
// $HOME/.config/gateway-3p/keys.env. Format: one KEY=value per line,
// optional single/double quotes around the value, # comments allowed.
// An already-set environment variable wins over the file. Key names
// only are logged; values are never logged, printed, or returned.
fn load_keys_file() {
    let path = std::env::var("GATEWAY_KEYS_FILE")
        .ok()
        .filter(|p| !p.is_empty())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/.config/gateway-3p/keys.env", h))
        });
    let Some(path) = path else { return };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        eprintln!("gateway keys file not found: {}", path);
        return;
    };
    let mut found: Vec<String> = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim().trim_matches(|c| c == '\'' || c == '"');
        if k.is_empty() || v.is_empty() {
            continue;
        }
        if std::env::var(k).is_err() {
            std::env::set_var(k, v);
        }
        found.push(k.to_string());
    }
    eprintln!(
        "gateway keys file loaded: {} ({} keys)",
        path,
        found.join(",")
    );
}

#[tokio::main]
async fn main() {
    load_keys_file();
    let app = Router::new()
        .route("/v1/models", get(models))
        .route("/v1/messages", post(messages))
        .route("/health", get(health));
    let addr: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    println!("gateway-3p listening on http://{}", addr);
    println!("routes loaded: {}", ROUTES.len());
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_routes_match_provider_contracts() {
        assert_eq!(ROUTES.len(), 3);
        assert_eq!(
            find_route("claude-sonnet-5").unwrap().provider_base,
            "https://api.reflection.ai/openai/v1"
        );
        assert_eq!(
            find_route("claude-3p-muse")
                .unwrap()
                .provider_base,
            "https://api.meta.ai/v1"
        );
        assert_eq!(
            find_route("claude-haiku-4-5")
                .unwrap()
                .provider_base,
            "https://integrate.api.nvidia.com/v1"
        );
    }

    #[test]
    fn tool_use_and_result_survive_anthropic_to_openai_translation() {
        let req = AnthropicReq {
            model: "claude-sonnet-5".into(),
            system: json!("system"),
            max_tokens: Some(256),
            stream: true,
            thinking: Value::Null,
            tools: json!([{
                "name": "read_file",
                "description": "read",
                "input_schema": {"type": "object"}
            }]),
            messages: vec![
                AnthropicMsg {
                    role: "assistant".into(),
                    content: json!([{
                        "type": "tool_use",
                        "id": "call_7",
                        "name": "read_file",
                        "input": {"path": "README.md"}
                    }]),
                },
                AnthropicMsg {
                    role: "user".into(),
                    content: json!([{
                        "type": "tool_result",
                        "tool_use_id": "call_7",
                        "content": "ok"
                    }]),
                },
            ],
        };

        let body = to_openai(&req, &ROUTES[0]);
        assert_eq!(body["messages"][1]["tool_calls"][0]["id"], "call_7");
        assert_eq!(body["messages"][2]["role"], "tool");
        assert_eq!(body["messages"][2]["tool_call_id"], "call_7");
        assert_eq!(body["tools"][0]["function"]["name"], "read_file");
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn unknown_desktop_model_is_rejected() {
        assert!(find_route("claude-unknown-99").is_none());
    }

    #[test]
    fn long_plugin_tool_names_round_trip_through_provider_aliases() {
        let original = "mcp__atoma_research_grade_harness__read_workspace_artifact_with_receipt";
        let alias = provider_tool_name(original);
        assert!(alias.len() <= 64);
        assert!(alias
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'));
        assert_ne!(alias, original);

        let tools = json!([{
            "name": original,
            "description": "read",
            "input_schema": {"type": "object"}
        }]);
        let reverse = provider_to_anthropic_tool_names(&tools);
        assert_eq!(reverse.get(&alias).map(String::as_str), Some(original));
        assert_eq!(
            anthropic_tools_to_openai(&tools)[0]["function"]["name"],
            alias
        );
    }

    #[test]
    fn beam_history_echoes_reasoning_for_followup_turns() {
        let req = AnthropicReq {
            model: "claude-sonnet-5".into(),
            system: Value::Null,
            max_tokens: Some(64),
            stream: false,
            thinking: Value::Null,
            tools: json!([{
                "name": "get_value",
                "input_schema": {"type": "object"}
            }]),
            messages: vec![AnthropicMsg {
                role: "assistant".into(),
                content: json!([{
                    "type": "text",
                    "text": "I need to call get_value."
                }, {
                    "type": "tool_use",
                    "id": "call_1",
                    "name": "get_value",
                    "input": {"key": "alpha"}
                }]),
            }],
        };
        let beam = find_route("claude-sonnet-5").unwrap();
        let body = to_openai(&req, beam);
        assert_eq!(
            body["messages"][0]["reasoning_content"],
            "I need to call get_value."
        );
    }
}
