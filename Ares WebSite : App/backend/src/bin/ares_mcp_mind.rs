//! ares-mcp-mind: the backend-owned mind as an MCP server (stdio).
//!
//! Thin stdio loop around `ares_backend::harness::mcp_server::handle_request`.
//! Authority is measured once at startup from the governed SurrealDB
//! client configuration; reachability is a live `RETURN 1` query at
//! startup, not a cached claim. If SurrealDB is unreachable, the
//! server still answers initialize/tools/list/db_status, with mode
//! "Unavailable" — it never attests "On" it cannot measure.

use ares_backend::harness::mcp_server::{handle_request, MindAuthority};
use ares_backend::surreal::SurrealClient;
use std::io::{BufRead, Write};

fn main() {
    let (base, ns, db, reachable) = match SurrealClient::from_env() {
        Ok(client) => {
            let (b, n, d) = client.authority();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let ok = match rt {
                Ok(rt) => rt.block_on(client.sql("RETURN 1;")).is_ok(),
                Err(_) => false,
            };
            (b, n, d, ok)
        }
        Err(_) => (
            "http://127.0.0.1:8000".to_string(),
            "ares".to_string(),
            "ares".to_string(),
            false,
        ),
    };
    let auth = MindAuthority {
        base,
        ns,
        db,
        surreal_reachable: reachable,
    };
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(req) => handle_request(&req, &auth),
            Err(_) => {
                serde_json::json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}})
            }
        };
        let _ = writeln!(stdout, "{}", resp);
        let _ = stdout.flush();
    }
}
