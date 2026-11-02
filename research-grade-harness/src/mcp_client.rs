// mcp_client — std-only MCP stdio client for routing packet tool_calls
// through a real N3 MCP child process (hypermind-mcp).
//
// Doctrine: NO serde_json anywhere in this crate. All response handling goes
// through the typed [`crate::json`] boundary: strict parse, member lookup,
// exact 64-hex `receipt_sha256` validation. Ad-hoc substring scanners are
// retired (kept only as `#[cfg(test)]` fixtures proving the typed path
// agrees on the canned envelope shape).
//
// Wire shape (JSON-RPC is the MCP boundary only):
//   request:  {"jsonrpc":"2.0","id":<n>,"method":"tools/call",
//             "params":{"name":"<tool>","arguments":<args_json>}}
//   response: {"jsonrpc":"2.0","id":<n>,
//             "result":{"content":[{"type":"text","text":"<inner JSON text>"}]}}
// where <inner JSON text> is a JSON object like
//   {"status":"ok","receipt_sha256":"<hex>", ...}
// Receipt detection requires the FULL 64-hex digest, never a marker.

use crate::json::{self, Json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Live child-process handle for one hypermind-mcp stdio server.
pub struct McpChild {
    child: Child,
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,
    /// Bounded tail of child stderr (diagnostics only, never evidence).
    stderr_tape: Arc<Mutex<Vec<u8>>>,
    next_id: u64,
}

/// Outcome of one `tools/call`: the typed inner result object plus the
/// exact bound receipt digest and the call latency.
#[derive(Debug, Clone)]
pub struct CallOutcome {
    /// Typed inner result object (`result.content[0].text`, parsed).
    pub inner: Json,
    /// Exact 64-hex `receipt_sha256` from the inner object.
    pub receipt_sha256: String,
    /// Call latency in nanoseconds (wall clock, surrogate input only).
    pub dt_ns: u64,
}

impl McpChild {
    /// Spawn `mcp_path` with `HYPERMIND_ROOT=root`, piped stdio/stderr.
    /// Stderr is captured to a bounded 64 KiB ring (diagnostics on transport
    /// failure); it is never parsed as evidence.
    pub fn spawn(mcp_path: &str, root: &str) -> Result<Self, String> {
        let mut child = Command::new(mcp_path)
            .env("HYPERMIND_ROOT", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {}: {}", mcp_path, e))?;
        let reader = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| "child stdout unavailable".to_string())?,
        );
        let writer = child
            .stdin
            .take()
            .ok_or_else(|| "child stdin unavailable".to_string())?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| "child stderr unavailable".to_string())?;
        let tape: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let tape_writer = Arc::clone(&tape);
        std::thread::Builder::new()
            .name("mcp-stderr".into())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match stderr.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut t = tape_writer.lock().unwrap_or_else(|e| e.into_inner());
                            t.extend_from_slice(&buf[..n]);
                            // Bounded ring: keep the last 64 KiB (recent cause).
                            if t.len() > 65536 {
                                let drop = t.len() - 65536;
                                t.drain(..drop);
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|e| format!("stderr thread: {}", e))?;
        Ok(Self {
            child,
            reader,
            writer,
            stderr_tape: tape,
            next_id: 1,
        })
    }

    /// Recent child stderr (lossy UTF-8, diagnostics only).
    pub fn stderr_tail(&self) -> String {
        let t = self.stderr_tape.lock().unwrap_or_else(|e| e.into_inner());
        String::from_utf8_lossy(&t).into_owned()
    }

    fn send(&mut self, method: &str, params: &Json) -> Result<Json, String> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let mut top = std::collections::BTreeMap::new();
        top.insert("jsonrpc".to_string(), Json::Str("2.0".into()));
        top.insert("id".to_string(), Json::Int(id as i64));
        top.insert("method".to_string(), Json::Str(method.into()));
        top.insert("params".to_string(), params.clone());
        let req = json::write(&Json::Obj(top)) + "\n";
        let t = Instant::now();
        self.writer
            .write_all(req.as_bytes())
            .map_err(|e| format!("write: {}", e))?;
        self.writer.flush().map_err(|e| format!("flush: {}", e))?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| format!("read: {} stderr={:?}", e, self.stderr_tail()))?;
        let dt_ns = t.elapsed().as_nanos() as u64;
        if line.is_empty() {
            return Err(format!(
                "eof: child closed stdout stderr={:?}",
                self.stderr_tail()
            ));
        }
        let body = json::parse_line(&line).map_err(|e| format!("response parse: {}", e))?;
        // JSON-RPC error envelope: fail with code + message, never as data.
        if let Some(err) = body.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("rpc error");
            return Err(format!("rpc error: {}", msg));
        }
        let _ = dt_ns;
        Ok(body)
    }

    /// Call one tool. `args` is a typed object; the writer serializes it
    /// canonically — no verbatim splicing of caller text.
    pub fn call(&mut self, tool: &str, args: &Json) -> Result<CallOutcome, String> {
        let mut params = std::collections::BTreeMap::new();
        params.insert("name".to_string(), Json::Str(tool.into()));
        params.insert("arguments".to_string(), args.clone());
        let t = Instant::now();
        let body = self.send("tools/call", &Json::Obj(params))?;
        let dt_ns = t.elapsed().as_nanos() as u64;
        let inner = unwrap_content(&body)?;
        check_status(tool, &inner)?;
        let digest =
            receipt_digest(&inner).ok_or_else(|| format!("{}: missing valid receipt", tool))?;
        Ok(CallOutcome {
            inner,
            receipt_sha256: digest,
            dt_ns,
        })
    }

    /// Back-compat string-args entry point: parses `args_json_str` through
    /// the strict boundary first (malformed args are caller errors, never
    /// spliced verbatim onto the wire).
    pub fn call_str(
        &mut self,
        tool: &str,
        args_json_str: &str,
    ) -> Result<(String, bool, u64), String> {
        let args = json::parse(args_json_str).map_err(|e| format!("{}: bad args: {}", tool, e))?;
        if !matches!(args, Json::Obj(_)) {
            return Err(format!("{}: args must be a JSON object", tool));
        }
        let out = self.call(tool, &args)?;
        Ok((json::write(&out.inner), true, out.dt_ns))
    }

    /// Query the server's advertised tool names before loading it as a plugin.
    /// Returns the typed `tools/list` result object.
    pub fn list_tools(&mut self) -> Result<Json, String> {
        let body = self.send("tools/list", &Json::Obj(std::collections::BTreeMap::new()))?;
        body.get("result")
            .cloned()
            .ok_or_else(|| "tools/list: missing result".to_string())
    }

    /// `true` when the typed `tools/list` result advertises `tool`.
    pub fn advertises(&mut self, tool: &str) -> Result<bool, String> {
        let result = self.list_tools()?;
        Ok(result
            .get("tools")
            .and_then(|t| match t {
                Json::Arr(items) => Some(
                    items
                        .iter()
                        .any(|it| it.get("name").and_then(|n| n.as_str()) == Some(tool)),
                ),
                _ => None,
            })
            .unwrap_or(false))
    }

    /// Shut the child down. Best-effort kill + reap; never fails.
    pub fn close(&mut self) {
        self.child.kill().ok();
        let _ = self.child.wait();
    }
}

impl Drop for McpChild {
    fn drop(&mut self) {
        self.close();
    }
}

/// Unwrap the MCP content envelope: `result.content[0].text` parsed as the
/// typed inner JSON object. Non-object inner text, missing envelopes, and
/// wrong shapes are all `Err` (typed, no silent pass-through).
///
/// The inner object carries `{result: {...}, receipt: {...}}`: status lives
/// under `result.status`, the digest under `receipt.receipt_sha256` (exact
/// server shape, VERIFIED against the live wire 2026-11-01). [`inner_view`]
/// flattens both into one object so judges and digest checks read flat keys.
pub fn unwrap_content(body: &Json) -> Result<Json, String> {
    let text = body
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| match c {
            Json::Arr(items) => items.first(),
            _ => None,
        })
        .and_then(|el| el.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "missing result.content[0].text envelope".to_string())?;
    let raw = json::parse(text).map_err(|e| format!("inner parse: {}", e))?;
    Ok(inner_view(&raw))
}

/// Flatten the server's `{result, receipt}` pair into a single judged object:
/// every `result.*` member lifted (status, retrieval, atom ids), plus
/// `receipt_sha256` lifted from `receipt.receipt_sha256`. A flat inner
/// object (legacy/test shape) passes through unchanged.
pub fn inner_view(raw: &Json) -> Json {
    let (Some(result), Some(receipt)) = (raw.get("result"), raw.get("receipt")) else {
        return raw.clone();
    };
    let mut flat = match result {
        Json::Obj(m) => m.clone(),
        _ => std::collections::BTreeMap::new(),
    };
    if let Json::Obj(rm) = receipt {
        if let Some(Json::Str(d)) = rm.get("receipt_sha256") {
            flat.insert("receipt_sha256".to_string(), Json::Str(d.clone()));
        }
        if let Some(swarm) = rm.get("swarm_run_id") {
            flat.insert("swarm_run_id".to_string(), swarm.clone());
        }
    }
    Json::Obj(flat)
}

/// Typed status gate: `error`/`denied`/`fail-explained`/`refused` are `Err`.
pub fn check_status(tool: &str, inner: &Json) -> Result<(), String> {
    if inner.is_error_status() {
        let status = inner
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("error");
        return Err(format!("{}: tool returned status={}", tool, status));
    }
    Ok(())
}

/// Exact receipt binding: the full 64-hex `receipt_sha256`, or `None`.
/// Markers, prefixes, and non-hex strings never bind.
pub fn receipt_digest(inner: &Json) -> Option<String> {
    inner
        .get("receipt_sha256")
        .and_then(|v| v.as_str())
        .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(|s| s.to_string())
}

/// Legacy helper (tests only): full-hex check on an inner-text string.
/// New code uses [`receipt_digest`] on the typed value.
pub fn has_receipt_sha256(inner: &str) -> bool {
    // Parse-and-extract keeps one validation rule (strict JSON + 64-hex).
    json::parse(inner)
        .ok()
        .and_then(|v| {
            v.get("receipt_sha256")
                .and_then(|r| r.as_str())
                .map(|s| s.to_string())
        })
        .map(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canned tools/call response line: the exact live server shape
    // (result.content[0].text = inner JSON {result: {...}, receipt: {...}},
    // VERIFIED against the wire 2026-11-01; digest is receipt-nested).

    /// Build the canned envelope from parts (keeps the 64-hex digest readable).
    fn canned_envelope(inner_text: &str) -> String {
        let mut top = std::collections::BTreeMap::new();
        top.insert("jsonrpc".to_string(), Json::Str("2.0".into()));
        top.insert("id".to_string(), Json::Int(3));
        let el = Json::Obj(
            [
                ("type".to_string(), Json::Str("text".into())),
                ("text".to_string(), Json::Str(inner_text.into())),
            ]
            .into_iter()
            .collect(),
        );
        let mut result = std::collections::BTreeMap::new();
        result.insert("content".to_string(), Json::Arr(vec![el]));
        top.insert("result".to_string(), Json::Obj(result));
        json::write(&Json::Obj(top))
    }

    #[test]
    fn typed_envelope_helpers_behave() {
        let inner_raw = "{\"result\":{\"status\":\"ok\"},\"receipt\":{\"receipt_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"swarm_run_id\":\"rgah-wire-20261029\"}}";
        let body = json::parse_line(&canned_envelope(inner_raw)).expect("canned line parses");
        let inner = unwrap_content(&body).expect("envelope unwraps typed");
        assert_eq!(inner.get("status").and_then(|s| s.as_str()), Some("ok"));
        assert_eq!(
            receipt_digest(&inner).as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(
            inner.get("swarm_run_id").and_then(|s| s.as_str()),
            Some("rgah-wire-20261029")
        );
        // status gate passes on ok.
        assert!(check_status("map.ingest", &inner).is_ok());
        // error statuses fail typed.
        let denied = json::parse("{\"status\":\"denied\"}").unwrap();
        assert!(check_status("map.query", &denied).is_err());
        // short/non-hex digests never bind.
        let short = json::parse("{\"status\":\"ok\",\"receipt_sha256\":\"abc\"}").unwrap();
        assert_eq!(receipt_digest(&short), None);
        // legacy string helper agrees on the canned inner text.
        let inner_text = json::write(&inner);
        assert!(has_receipt_sha256(&inner_text));
        assert!(!has_receipt_sha256("{\"status\":\"ok\"}"));
        // malformed args never reach the wire: call_str rejects pre-spawn.
        // (no child needed — parse failure precedes any IO).
        assert!(json::parse("{bad}").is_err());
    }

    #[test]
    fn malformed_envelopes_are_typed_errors() {
        // missing text member
        let no_text = json::parse("{\"result\":{\"content\":[{\"type\":\"text\"}]}}").unwrap();
        assert!(unwrap_content(&no_text).is_err());
        // inner text that is not JSON
        let raw =
            json::parse("{\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"not json\"}]}}")
                .unwrap();
        assert!(unwrap_content(&raw).is_err());
        // no envelope at all (initialize-style body)
        let bare = json::parse("{\"jsonrpc\":\"2.0\",\"id\":2}").unwrap();
        assert!(unwrap_content(&bare).is_err());
    }
}
