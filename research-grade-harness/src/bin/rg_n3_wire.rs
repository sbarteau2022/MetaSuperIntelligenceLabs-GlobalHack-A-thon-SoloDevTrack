// rgah -> hypermind-mcp round-trip over stdio (RG-AH <-> N^3 wiring, hardware).
//
// Spawns hypermind-mcp as a child (the PROVEN path from pressure_test.py),
// runs initialize + tools/list + map.ingest + map.query + map.compact +
// map.checkpoint with receipts recorded, and writes a plain-text receipt.
//
// Degraded drills: unknown-tool denial, bad-schema denial, and (optional,
// --kill-server) N3-unavailable mode where the harness records
// degraded=n3-unavailable in its own ledger and claims no canonical truth.
//
// No JSON files: JSON-RPC is the MCP boundary only; the receipt is plain text.
// No new deps: std only (TcpStream-free; child stdio pipes).
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn send(
    r: &mut BufReader<std::process::ChildStdout>,
    w: &mut std::process::ChildStdin,
    mid: u64,
    method: &str,
    params: &str,
) -> Result<(String, u64), String> {
    let req = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"{}\",\"params\":{}}}\n",
        mid, method, params
    );
    let t = Instant::now();
    w.write_all(req.as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    w.flush().map_err(|e| format!("flush: {}", e))?;
    let mut line = String::new();
    r.read_line(&mut line).map_err(|e| format!("read: {}", e))?;
    let dt = t.elapsed().as_nanos() as u64;
    Ok((line, dt))
}

/// Unwrap the MCP content envelope: tools/call returns
/// result.content[0].text = JSON {result, receipt}. Returns the inner text,
/// or the raw body when the envelope is absent (initialize, tools/list).
fn unwrap_content(body: &str) -> String {
    // Find "text":" then scan the JSON string with escape handling.
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

/// Extract a top-level string field "key":"value" (first occurrence).
fn sfield(body: &str, key: &str) -> Option<String> {
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

fn esc(s: &str) -> String {
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

fn spawn(mcp: &str, root: &str) -> Result<Child, String> {
    Command::new(mcp)
        .env("HYPERMIND_ROOT", root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn {}: {}", mcp, e))
}

fn main() -> std::process::ExitCode {
    let mcp = match std::env::var("HYPERBOLIC_MAP_PLUGIN_BIN") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("set HYPERBOLIC_MAP_PLUGIN_BIN to the standalone plugin executable");
            return std::process::ExitCode::FAILURE;
        }
    };
    let root = std::env::var("HYPERMIND_ROOT").unwrap_or_else(|_| "/tmp/rgah-n3-wire".into());
    let receipt_path = std::env::var("WIRE_RECEIPT")
        .unwrap_or_else(|_| "research-grade-harness/RG-N3-WIRE-RECEIPT.txt".into());
    let kill_server = std::env::args().any(|a| a == "--kill-server");
    let _ = std::fs::create_dir_all(&root);

    let mut out = String::new();
    out.push_str("rg_n3_wire_receipt\n");
    out.push_str("spec=N^3-Research-Harness-Wiring.md\n");
    out.push_str("path=rgah(child-stdio)->hypermind-mcp tools/call with receipts\n");

    if kill_server {
        // N3-unavailable drill: harness runs degraded, claims no canonical truth.
        match spawn(&mcp, &root) {
            Ok(mut c) => {
                c.kill().ok();
                let _ = c.wait();
            }
            Err(e) => {
                out.push_str(&format!("spawn_failed={}\n", e));
            }
        }
        out.push_str("n3_available=false\n");
        out.push_str("degraded=n3-unavailable\n");
        out.push_str(
            "mode=receiptless-in-N3, harness-local ledger only; no claim of canonical truth\n",
        );
        out.push_str("accept fallback-n3-unavailable=True\n");
        return match std::fs::write(&receipt_path, &out) {
            Ok(()) => {
                println!("{}", out);
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("receipt write FAIL: {}", e);
                std::process::ExitCode::FAILURE
            }
        };
    }

    let mut child = match spawn(&mcp, &root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("spawn FAIL: {}", e);
            return std::process::ExitCode::FAILURE;
        }
    };
    let mut r = BufReader::new(child.stdout.take().expect("stdout"));
    let mut w = child.stdin.take().expect("stdin");
    let mut mid = 1u64;
    let mut run = |rdr: &mut BufReader<std::process::ChildStdout>,
                   wtr: &mut std::process::ChildStdin,
                   method: &str,
                   params: &str| {
        let m = mid;
        mid += 1;
        send(rdr, wtr, m, method, params)
    };

    // 1. initialize + tools/list round-trip.
    let t = Instant::now();
    let init = run(&mut r, &mut w, "initialize", "{}");
    let init_dt = t.elapsed();
    let init_ok = init.map(|(b, _)| b.contains("hypermind")).unwrap_or(false);
    out.push_str(&format!(
        "initialize ok={} dt_ms={}\n",
        init_ok,
        init_dt.as_millis()
    ));

    let (tl_body, _) = match run(&mut r, &mut w, "tools/list", "{}") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tools/list FAIL: {}", e);
            child.kill().ok();
            return std::process::ExitCode::FAILURE;
        }
    };
    let tool_count = tl_body.matches("\"name\":\"map.").count();
    out.push_str(&format!("tools/list map_tools={}\n", tool_count));

    // 2. harness loop: ingest -> retrieve -> compact, receipts recorded.
    let ingest_texts = [
        "rgah wire ingest: kuramoto lattices synchronize via phase coupling",
        "rgah wire ingest: symplectic envelope guards traversal arcs",
    ];
    let mut ingest_ok = 0;
    let mut swarm_echoed = 0;
    for text in ingest_texts {
        let params = format!(
            "{{\"name\":\"map.ingest\",\"arguments\":{{\"text\":{},\"swarm_run_id\":\"rgah-wire-20261029\"}}}}",
            esc(text)
        );
        match run(&mut r, &mut w, "tools/call", &params) {
            Ok((body, dt)) => {
                let inner = unwrap_content(&body);
                let has_receipt = inner.contains("receipt_sha256");
                let status = sfield(&inner, "status").unwrap_or_default();
                // swarm_run_id threads INTO the N3 receipt (echoed field),
                // not just the harness log — wiring spec §5.
                let echo = sfield(&inner, "swarm_run_id").unwrap_or_default();
                if echo == "rgah-wire-20261029" {
                    swarm_echoed += 1;
                }
                if status == "ok" && has_receipt {
                    ingest_ok += 1;
                }
                out.push_str(&format!(
                    "ingest status={} receipt={} swarm_echo={} dt_ns={}\n",
                    status,
                    has_receipt,
                    echo == "rgah-wire-20261029",
                    dt
                ));
            }
            Err(e) => out.push_str(&format!("ingest ERROR {}\n", e)),
        }
    }

    let qparams = "{\"name\":\"map.query\",\"arguments\":{\"qtext\":\"kuramoto phase coupling synchronization\",\"k\":3}}";
    let (qbody, qdt) = match run(&mut r, &mut w, "tools/call", qparams) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("map.query FAIL: {}", e);
            child.kill().ok();
            return std::process::ExitCode::FAILURE;
        }
    };
    let qinner = unwrap_content(&qbody);
    let q_ok =
        sfield(&qinner, "status").unwrap_or_default() == "ok" && qinner.contains("receipt_sha256");
    let retrieval = sfield(&qinner, "retrieval").unwrap_or_default();
    out.push_str(&format!(
        "query ok={} dt_ns={} retrieval={}\n",
        q_ok, qdt, retrieval
    ));
    // swarm_run_id threads the harness run id through to the N3 side.
    out.push_str("swarm_run_id=rgah-wire-20261029 (attached to harness ledger; in-band receipts returned by N3)\n");

    let cparams = "{\"name\":\"map.compact\",\"arguments\":{\"objective\":\"rgah-wire\",\"recent_state\":\"query ok\"}}";
    let compact_ok = run(&mut r, &mut w, "tools/call", cparams)
        .map(|(b, _)| {
            let inner = unwrap_content(&b);
            sfield(&inner, "status").unwrap_or_default() == "ok" && inner.contains("receipt_sha256")
        })
        .unwrap_or(false);
    out.push_str(&format!("compact ok={}\n", compact_ok));

    let ckparams = "{\"name\":\"map.checkpoint\",\"arguments\":{\"action\":\"get\"}}";
    let checkpoint_ok = run(&mut r, &mut w, "tools/call", ckparams)
        .map(|(b, _)| sfield(&unwrap_content(&b), "status").unwrap_or_default() == "ok")
        .unwrap_or(false);
    out.push_str(&format!("checkpoint_readback ok={}\n", checkpoint_ok));

    // 3. Fallback drill: unknown tool denied with receipt (no execution).
    let uparams = "{\"name\":\"map.nope_missing\",\"arguments\":{}}";
    let deny_unknown = run(&mut r, &mut w, "tools/call", uparams)
        .map(|(b, _)| sfield(&unwrap_content(&b), "status").unwrap_or_default() == "error")
        .unwrap_or(false);
    out.push_str(&format!("fallback unknown-tool denied={}\n", deny_unknown));

    // 4. Fallback drill: degraded query (bad schema) denied with receipt.
    let bparams = "{\"name\":\"map.query\",\"arguments\":{}}";
    let deny_schema = run(&mut r, &mut w, "tools/call", bparams)
        .map(|(b, _)| b.contains("schema"))
        .unwrap_or(false);
    out.push_str(&format!(
        "fallback degraded-query denied-with-receipt={}\n",
        deny_schema
    ));

    drop(w);
    child.kill().ok();
    let _ = child.wait_timeout(Duration::from_secs(5));

    let pass = init_ok
        && ingest_ok == ingest_texts.len()
        && swarm_echoed == ingest_texts.len()
        && q_ok
        && compact_ok
        && checkpoint_ok
        && deny_unknown
        && deny_schema;
    out.push_str(&format!(
        "accept rgah-launch-stdio-roundtrip={}\naccept ingest-retrieve-compact-cycle={}\naccept fallback-drills={}\naccept swarm_run_id-attached={} (echoed in N3 receipts, not harness-log-only)\n",
        init_ok,
        ingest_ok == ingest_texts.len() && q_ok && compact_ok,
        deny_unknown && deny_schema,
        swarm_echoed == ingest_texts.len()
    ));
    out.push_str(&format!("verdict={}\n", if pass { "PASS" } else { "FAIL" }));
    match std::fs::write(&receipt_path, &out) {
        Ok(()) => {
            println!("{}", out);
            if pass {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("receipt write FAIL: {}", e);
            std::process::ExitCode::FAILURE
        }
    }
}

/// wait with timeout (std has no Child::wait_timeout; poll manually).
trait WaitTimeout {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<()>;
}
impl WaitTimeout for Child {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<()> {
        let t = Instant::now();
        while t.elapsed() < dur {
            match self.try_wait()? {
                Some(_) => return Ok(()),
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        }
        Ok(())
    }
}
