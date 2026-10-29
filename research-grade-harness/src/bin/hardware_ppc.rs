// Hardware PPC probe: fixed-PPC gate instrument (HARDWARE-GATE-RUNBOOK.md).
//
// Runs ProposeProbeCompose::run() with a REAL model probe function against the
// live oMLX server (OpenAI-compatible /v1/chat/completions on 127.0.0.1:8000)
// and writes a plain-text receipt (no JSON, per the Mac no-JSON receipt rule).
//
// std-only HTTP: this crate carries NO serde_json/reqwest (doctrine), so the
// probe speaks raw HTTP/1.1 over TcpStream and extracts the two fields it
// needs (message content, total_tokens) with a minimal string scanner.
// Nothing here is JSON-schema parsing; the receipt is plain text.
//
// What it does:
// - 6 probe tasks + 4 held-out tasks (arithmetic, last_number_eq judges).
// - Each bundle is probed by rendering its components into the system prompt;
//   the base bundle and each candidate face the SAME tasks, so quality deltas
//   are measured, not word-counted.
// - Judge: deterministic last_number_eq (ported from the A39 canonical judge
//   pack; numeric comparison of the LAST number token in decoded text).
// - Adoption via leakage_critic + 0.02 noise band + regularized_adopt,
//   KEEP_BASE default (all inside run()).
//
// What it does NOT do:
// - This binary never ships synthetic numbers as measurements. If the model
//   server is unreachable it exits non-zero WITHOUT writing a pass receipt.
//
// Env: OMLX_URL (default http://127.0.0.1:8000/v1/chat/completions),
//      OMLX_MODEL (default Qwen3.6-MoE-Combined), OMLX_API_KEY (default dummy).
use rgah::energy::EnergyBudget;
use rgah::rsi::{Bundle, ProposeProbeCompose};
use std::io::{Read, Write};
use std::time::Duration;

struct Task {
    id: &'static str,
    prompt: &'static str,
    target: &'static str,
    #[allow(dead_code)]
    held_out: bool, // ledger tag: PROBE_TASKS=false, HELDOUT_TASKS=true
}

// 6 probe tasks: small arithmetic with distinct answers (no shared target
// that a prompt could memorize across tasks).
const PROBE_TASKS: &[Task] = &[
    Task {
        id: "probe-01",
        prompt: "What is 37 plus 48?",
        target: "85",
        held_out: false,
    },
    Task {
        id: "probe-02",
        prompt: "What is 112 minus 57?",
        target: "55",
        held_out: false,
    },
    Task {
        id: "probe-03",
        prompt: "What is 13 times 7?",
        target: "91",
        held_out: false,
    },
    Task {
        id: "probe-04",
        prompt: "What is 144 divided by 12?",
        target: "12",
        held_out: false,
    },
    Task {
        id: "probe-05",
        prompt: "What is 29 plus 71 minus 15?",
        target: "85",
        held_out: false,
    },
    // Same answer as probe-01 via a different path: catches copy-the-answer
    // prompts that a leakage probe would flag, not reward.
    Task {
        id: "probe-06",
        prompt: "What is 9 times 9 plus 4?",
        target: "85",
        held_out: false,
    },
    // Multi-step + distractor resistance: the answer must survive working.
    Task {
        id: "probe-07",
        prompt: "A train goes 60 km in 1 hour, then 90 km in the next 1.5 hours. What is the total distance?",
        target: "150",
        held_out: false,
    },
    Task {
        id: "probe-08",
        prompt: "Maya had 45 stickers, gave away 17, then bought 23 more. How many does she have now?",
        target: "51",
        held_out: false,
    },
];

// 4 held-out tasks: different operands/answers, run ONCE after adoption.
const HELDOUT_TASKS: &[Task] = &[
    Task {
        id: "held-01",
        prompt: "What is 53 plus 28?",
        target: "81",
        held_out: true,
    },
    Task {
        id: "held-02",
        prompt: "What is 200 minus 117?",
        target: "83",
        held_out: true,
    },
    Task {
        id: "held-03",
        prompt: "What is 11 times 11?",
        target: "121",
        held_out: true,
    },
    Task {
        id: "held-04",
        prompt: "What is 96 divided by 8?",
        target: "12",
        held_out: true,
    },
];

/// Deterministic last_number_eq judge (A39 canonical semantics): the LAST
/// number token in decoded text must equal target numerically.
fn last_number_eq(decoded: &str, target: &str) -> (bool, String) {
    let mut last: Option<(String, f64)> = None;
    let mut cur = String::new();
    let flush = |cur: &mut String, last: &mut Option<(String, f64)>| {
        if !cur.is_empty() {
            if let Ok(v) = cur.parse::<f64>() {
                *last = Some((cur.clone(), v));
            }
            cur.clear();
        }
    };
    for c in decoded.chars() {
        if c.is_ascii_digit()
            || (c == '.' && cur.chars().any(|d| d.is_ascii_digit()))
            || ((c == '-' || c == '+') && cur.is_empty())
        {
            cur.push(c);
        } else {
            flush(&mut cur, &mut last);
        }
    }
    flush(&mut cur, &mut last);
    match (last, target.parse::<f64>()) {
        (Some((s, v)), Ok(w)) => (
            v == w,
            format!(
                "last number {} {} {}",
                s,
                if v == w { "==" } else { "!=" },
                target
            ),
        ),
        _ => (false, "no number in output or bad target".to_string()),
    }
}

/// Minimal JSON string escaper for the request body (request side only).
fn jstr(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Extract a JSON string value for `key` (first `"key":"..."` occurrence),
/// honouring backslash escapes. Returns None when absent/malformed.
fn extract_str(body: &str, key: &str) -> Option<String> {
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
                    b'/' => out.push('/'),
                    b'u' if i + 5 < bytes.len() => {
                        let hex = &body[i + 2..i + 6];
                        if let Ok(cp) = u32::from_str_radix(hex, 16) {
                            if let Some(c) = char::from_u32(cp) {
                                out.push(c);
                            }
                        }
                        i += 4;
                    }
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

/// Extract a JSON integer value for `key` (first `"key":<digits>` occurrence).
fn extract_u64(body: &str, key: &str) -> Option<u64> {
    let pat = format!("\"{}\":", key);
    let start = body.find(&pat)? + pat.len();
    let tail = body[start..].trim_start();
    let end = tail
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(tail.len());
    if end == 0 {
        return None;
    }
    tail[..end].parse().ok()
}

fn chat_once(
    host: &str,
    port: u16,
    path: &str,
    model: &str,
    key: &str,
    system: &str,
    user: &str,
) -> Result<(String, u64), String> {
    let body = format!(
        "{{\"model\":{},\"messages\":[{{\"role\":\"system\",\"content\":{}}},{{\"role\":\"user\",\"content\":{}}}],\"max_tokens\":256,\"temperature\":0}}",
        jstr(model),
        jstr(system),
        jstr(user)
    );
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path,
        host,
        key,
        body.len(),
        body
    );
    let addr = format!("{}:{}", host, port);
    let mut stream =
        std::net::TcpStream::connect(&addr).map_err(|e| format!("connect {}: {}", addr, e))?;
    stream.set_read_timeout(Some(Duration::from_secs(120))).ok();
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read: {}", e))?;
    let resp = String::from_utf8_lossy(&raw).into_owned();
    let status_line = resp.lines().next().unwrap_or("");
    if !status_line.contains("200") {
        return Err(format!("server status: {}", status_line));
    }
    let body_start = resp.find("\r\n\r\n").map(|i| i + 4).unwrap_or(resp.len());
    let rbody = &resp[body_start..];
    let content = extract_str(rbody, "content").unwrap_or_default();
    let total = extract_u64(rbody, "total_tokens").unwrap_or(0);
    Ok((content, total))
}

/// Split OMLX_URL into (host, port, path). http localhost only.
fn split_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or("only http:// URLs supported")?;
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].to_string()),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match hostport.rfind(':') {
        Some(i) => (
            hostport[..i].to_string(),
            hostport[i + 1..]
                .parse::<u16>()
                .map_err(|e| e.to_string())?,
        ),
        None => (hostport.to_string(), 80),
    };
    Ok((host, port, path))
}

/// Bundle text rendered into the probe's system prompt: the ONLY channel by
/// which a candidate bundle can affect probe quality.
fn bundle_system_prompt(b: &Bundle) -> String {
    format!(
        "You are an arithmetic answerer. Follow these instructions exactly:\n{}\nEnd every reply with: FINAL <number> and nothing after it.",
        b.components.join("\n")
    )
}

fn main() -> std::process::ExitCode {
    let url = std::env::var("OMLX_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/v1/chat/completions".into());
    let model = std::env::var("OMLX_MODEL").unwrap_or_else(|_| "Qwen3.6-MoE-Combined".into());
    let key = std::env::var("OMLX_API_KEY").unwrap_or_else(|_| "dummy".into());
    let k: usize = std::env::var("PPC_K")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let (host, port, path) = match split_url(&url) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bad OMLX_URL: {}", e);
            return std::process::ExitCode::FAILURE;
        }
    };

    // Preflight: the gate needs a LIVE model. No server -> fail, no receipt.
    if let Err(e) = chat_once(
        &host,
        port,
        &path,
        &model,
        &key,
        "You are a probe.",
        "Reply with: FINAL 1",
    ) {
        eprintln!("PREFLIGHT FAIL: model server unreachable: {}", e);
        eprintln!("Set OMLX_URL/OMLX_MODEL/OMLX_API_KEY or start omlx. No receipt written.");
        return std::process::ExitCode::FAILURE;
    }

    let base = Bundle::base();
    let probe = |b: &Bundle| -> (usize, usize, u64, Vec<String>) {
        let sys = bundle_system_prompt(b);
        let mut passes = 0usize;
        let mut tokens = 0u64;
        for t in PROBE_TASKS {
            match chat_once(&host, port, &path, &model, &key, &sys, t.prompt) {
                Ok((text, tok)) => {
                    tokens += tok;
                    if last_number_eq(&text, t.target).0 {
                        passes += 1;
                    }
                }
                Err(e) => {
                    eprintln!("probe task {} error: {}", t.id, e);
                }
            }
        }
        (
            passes,
            PROBE_TASKS.len(),
            tokens,
            vec!["last_number_eq".to_string()],
        )
    };

    let mut budget = EnergyBudget::new(50.0);
    let ppc = ProposeProbeCompose::new(k);
    let (adopted, m) = ppc.run(&base, &mut budget, probe);

    // Held-out: run ONCE on the adopted bundle (or base if kept).
    let sys = bundle_system_prompt(&adopted);
    let mut held_passes = 0usize;
    let mut held_tokens = 0u64;
    let mut held_rows: Vec<String> = vec![];
    for t in HELDOUT_TASKS {
        match chat_once(&host, port, &path, &model, &key, &sys, t.prompt) {
            Ok((text, tok)) => {
                held_tokens += tok;
                let (ok, why) = last_number_eq(&text, t.target);
                if ok {
                    held_passes += 1;
                }
                held_rows.push(format!(
                    "heldout task={} target={} pass={} why={}",
                    t.id, t.target, ok, why
                ));
            }
            Err(e) => held_rows.push(format!("heldout task={} ERROR {}", t.id, e)),
        }
    }

    let mut out = String::new();
    out.push_str("hardware_ppc_receipt\n");
    out.push_str("runbook=research-grade-harness/HARDWARE-GATE-RUNBOOK.md\n");
    out.push_str(&format!("model={}\n", model));
    out.push_str(&format!("server={}\n", url));
    out.push_str(&format!("real_probe={}\n", m.real_probe));
    out.push_str(&format!(
        "k={} rollouts={} sessions={}\n",
        m.k, m.rollouts, m.sessions
    ));
    out.push_str(&format!(
        "adopted_version={} adopted_fingerprint={}\n",
        m.adopted_version, m.adopted_fingerprint
    ));
    out.push_str(&format!(
        "kept_base={} abstained={}\n",
        m.kept_base, m.abstained
    ));
    out.push_str(&format!("quality_gain={:.4}\n", m.quality_gain));
    out.push_str(&format!(
        "energy_spent_j={:.4}\n",
        m.energy_spent_j + held_tokens as f64 * 0.0005
    ));
    out.push_str(&format!("judges={}\n", m.judge_ids.join(",")));
    out.push_str("regulators:\n");
    for (name, passed) in &m.regulators {
        out.push_str(&format!("regulator {} passed={}\n", name, passed));
    }
    out.push_str("proposals:\n");
    for p in &m.proposals {
        out.push_str(&format!(
            "proposal fp={} v={} passes={}/{} quality={:.4} tokens={} energy_j={:.4}\n",
            p.fingerprint, p.version, p.passes, p.total, p.quality, p.tokens, p.energy_j
        ));
    }
    out.push_str(&format!(
        "heldout passes={}/{} tokens={}\n",
        held_passes,
        HELDOUT_TASKS.len(),
        held_tokens
    ));
    for r in &held_rows {
        out.push_str(&format!("{}\n", r));
    }

    let path = std::env::var("PPC_RECEIPT")
        .unwrap_or_else(|_| "research-grade-harness/HARDWARE-PPC-RECEIPT.txt".into());
    match std::fs::File::create(&path).and_then(|mut f| f.write_all(out.as_bytes()).map(|_| ())) {
        Ok(()) => {
            println!("{}", out);
            eprintln!("receipt written to {}", path);
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("receipt write FAIL {}: {}", path, e);
            std::process::ExitCode::FAILURE
        }
    }
}
