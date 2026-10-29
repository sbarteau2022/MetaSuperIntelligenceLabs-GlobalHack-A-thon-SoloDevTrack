// mcp_client — std-only MCP stdio client for routing packet tool_calls
// through a real N3 MCP child process (hypermind-mcp).
//
// Doctrine: NO serde_json VALUE parsing anywhere in this crate. The helpers
// below are hand-rolled string scanners ported from the reference pattern in
// src/bin/rg_n3_wire.rs (`unwrap_content`, `sfield`, `esc`). Serde derive on
// structs is fine; serde_json is not a dependency and must not become one.
//
// Wire shape (JSON-RPC is the MCP boundary only):
//   request:  {"jsonrpc":"2.0","id":<n>,"method":"tools/call",
//             "params":{"name":"<tool>","arguments":<args_json>}}
//   response: {"jsonrpc":"2.0","id":<n>,
//             "result":{"content":[{"type":"text","text":"<inner JSON text>"}]}}
// where <inner JSON text> is a JSON string like
//   {"status":"ok","receipt_sha256":"<hex>", ...}
// Receipt detection is a plain substring check for "receipt_sha256".

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

/// Live child-process handle for one hypermind-mcp stdio server.
pub struct McpChild {
    child: Child,
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,
    next_id: u64,
}

impl McpChild {
    /// Spawn `mcp_path` with `HYPERMIND_ROOT=root`, piped stdio.
    pub fn spawn(mcp_path: &str, root: &str) -> Result<Self, String> {
        let mut child = Command::new(mcp_path)
            .env("HYPERMIND_ROOT", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
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
        Ok(Self {
            child,
            reader,
            writer,
            next_id: 1,
        })
    }

    /// Call one tool. `args_json_str` must already be a JSON object literal
    /// (e.g. `{"text":"hi"}`); it is spliced verbatim into `params.arguments`.
    ///
    /// Returns `(inner_result_json_text, has_receipt_sha256, dt_ns)`.
    pub fn call(&mut self, tool: &str, args_json_str: &str) -> Result<(String, bool, u64), String> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let params = format!("{{\"name\":{},\"arguments\":{}}}", esc(tool), args_json_str);
        let req = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"tools/call\",\"params\":{}}}\n",
            id, params
        );
        let t = Instant::now();
        self.writer
            .write_all(req.as_bytes())
            .map_err(|e| format!("write: {}", e))?;
        self.writer.flush().map_err(|e| format!("flush: {}", e))?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| format!("read: {}", e))?;
        let dt_ns = t.elapsed().as_nanos() as u64;
        if line.is_empty() {
            return Err("eof: child closed stdout".to_string());
        }
        let inner = unwrap_content(&line);
        let has_receipt = has_receipt_sha256(&inner);
        Ok((inner, has_receipt, dt_ns))
    }

    /// Shut the child down. Best-effort kill + reap; never fails.
    pub fn close(&mut self) {
        self.child.kill().ok();
        let _ = self.child.wait();
    }
}

/// Unwrap the MCP content envelope: tools/call returns
/// `result.content[0].text` = inner JSON text (e.g. `{result, receipt}`).
/// Returns the inner text, or the raw body when the envelope is absent
/// (initialize, tools/list responses).
pub fn unwrap_content(body: &str) -> String {
    // Find `"text":"` then scan the JSON string with escape handling.
    let pat = "\"text\":\"";
    let start = match body.find(pat) {
        Some(i) => i + pat.len(),
        None => return body.to_string(),
    };
    let bytes = body.as_bytes();
    let mut out = String::new();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => {
                match bytes[i + 1] {
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    c => {
                        out.push('\\');
                        out.push(c as char);
                    }
                }
                i += 2;
            }
            b'"' => return out,
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

/// Receipt presence check: plain substring scan, no JSON parsing.
pub fn has_receipt_sha256(inner: &str) -> bool {
    inner.contains("receipt_sha256")
}

/// Extract a top-level string field `"key":"value"` (first occurrence).
/// Handles the standard `\"` `\\` `\n` `\r` `\t` escapes.
pub fn sfield(body: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\":\"", key);
    let start = body.find(&pat)? + pat.len();
    let bytes = body.as_bytes();
    let mut out = String::new();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => {
                match bytes[i + 1] {
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    c => {
                        out.push('\\');
                        out.push(c as char);
                    }
                }
                i += 2;
            }
            b'"' => return Some(out),
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    None
}

/// Quote a string as a JSON string literal (`"..."` with escapes).
pub fn esc(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            _ => o.push(c),
        }
    }
    o.push('"');
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canned tools/call response line copied from the real envelope shape
    // documented in rg_n3_wire.rs (`result.content[0].text` = inner JSON
    // `{result, receipt}`), plus a tools/list-style body with no envelope.
    const INGEST_LINE: &str = "{\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"{\\\"status\\\":\\\"ok\\\",\\\"receipt_sha256\\\":\\\"abc123\\\",\\\"swarm_run_id\\\":\\\"rgah-wire-20261029\\\"}\"}]}}";
    const LIST_BODY: &str =
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"map.ingest\"}]}}";

    #[test]
    fn envelope_helpers_behave() {
        // 1. unwrap peels the content envelope to the inner JSON text.
        let inner = unwrap_content(INGEST_LINE);
        assert!(
            inner.contains("\"status\":\"ok\""),
            "unwrap must expose inner status, got: {}",
            inner
        );
        // 2. receipt detection fires on the inner text.
        assert!(
            has_receipt_sha256(&inner),
            "inner text must carry receipt_sha256"
        );
        // 3. sfield extracts status from the inner text.
        assert_eq!(sfield(&inner, "status").as_deref(), Some("ok"));
        // 4. sfield extracts the threaded swarm_run_id echo.
        assert_eq!(
            sfield(&inner, "swarm_run_id").as_deref(),
            Some("rgah-wire-20261029")
        );
        // 5. bodies without an envelope pass through untouched.
        assert_eq!(unwrap_content(LIST_BODY), LIST_BODY);
        // 6. no receipt substring means no receipt.
        assert!(!has_receipt_sha256(LIST_BODY));
        // 7. esc round-trips through sfield for tricky values.
        let tricky = "a\"b\\c\nd";
        let doc = format!("{{\"note\":{}}}", esc(tricky));
        assert_eq!(sfield(&doc, "note").as_deref(), Some(tricky));
        // 8. missing keys yield None rather than panicking.
        assert_eq!(sfield(&inner, "no_such_key"), None);
    }
}
