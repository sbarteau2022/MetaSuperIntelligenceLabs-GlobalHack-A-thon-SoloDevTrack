//! Shared live-model probe instrument for the hardware PPC binaries.
//!
//! Extracted verbatim from `src/bin/hardware_ppc.rs` (task set v2, judges,
//! std-only HTTP, prompt rendering) so new PPC instruments reuse the filed
//! logic instead of copying it. `hardware_ppc.rs` keeps its own copies —
//! the filed binary is frozen by its receipts; this module serves NEW bins.
//!
//! Doctrine: std-only, no serde_json. Receipts are plain text.

use std::io::{Read, Write};
use std::time::Duration;

pub struct PpcTask {
    pub id: &'static str,
    pub prompt: &'static str,
    pub target: &'static str,
    pub held_out: bool, // ledger tag: PROBE_TASKS=false, HELDOUT_TASKS=true
}

// TASK SET v2 (discriminating), 2026-10-29. Verbatim from hardware_ppc.rs:
// v1 (single-op arithmetic) saturated, so v2 uses multi-step word problems
// with distractor numbers; every intermediate result is a plausible wrong
// LAST number. All targets python3-verified before check-in.
pub const PROBE_TASKS: &[PpcTask] = &[
    PpcTask {
        id: "probe-01",
        prompt: "A warehouse stocks 240 crates, ships 84 on Monday and half the remainder on Tuesday. How many remain?",
        target: "78",
        held_out: false,
    },
    PpcTask {
        id: "probe-02",
        prompt: "A film runs 2 hours, plus a 37-minute featurette and a 15-minute intermission. How many minutes is that in total?",
        target: "172",
        held_out: false,
    },
    PpcTask {
        id: "probe-03",
        prompt: "137 marbles are packed into boxes holding 12 each. How many marbles are left over unpacked?",
        target: "5",
        held_out: false,
    },
    PpcTask {
        id: "probe-04",
        prompt: "A jacket costs 200 dollars, is marked down 20 percent, then the sale price is taxed 5 percent. What is the final price in dollars?",
        target: "168",
        held_out: false,
    },
    PpcTask {
        id: "probe-05",
        prompt: "A truck drives 100 km in the first hour and 120 km over the next 3 hours. What is its average speed in km/h across the whole trip?",
        target: "55",
        held_out: false,
    },
    PpcTask {
        id: "probe-06",
        prompt: "Maya has 340 dollars, spends 127 on rent, earns 58 babysitting, then spends 96 on groceries. How much money is left?",
        target: "175",
        held_out: false,
    },
    PpcTask {
        id: "probe-07",
        prompt: "A depot holds 96 crates. 38 ship out Monday, leaving 58. Then 25 more ship Tuesday. First confirm Monday's count of 58, then give Tuesday's remainder. How many remain after Tuesday?",
        target: "33",
        held_out: false,
    },
    PpcTask {
        id: "probe-08",
        prompt: "A library holds 210 books, lends out 86, receives 47 donations, then lends out 59 more. How many books are on the shelves now?",
        target: "112",
        held_out: false,
    },
];

// 4 held-out tasks (set v2): same families as the probes, different
// numbers/stories. Run ONCE on the adopted bundle after adoption.
pub const HELDOUT_TASKS: &[PpcTask] = &[
    PpcTask {
        id: "held-01",
        prompt: "A depot stocks 300 pallets, ships 96 on Monday and half the remainder on Tuesday. How many remain?",
        target: "102",
        held_out: true,
    },
    PpcTask {
        id: "held-02",
        prompt: "A roast cooks for 1 hour, rests 52 minutes, then reheats for 18 minutes. How many minutes is that in total?",
        target: "130",
        held_out: true,
    },
    PpcTask {
        id: "held-03",
        prompt: "154 bolts are packed into bags holding 12 each. How many bolts are left over unpacked?",
        target: "10",
        held_out: true,
    },
    PpcTask {
        id: "held-04",
        prompt: "A bicycle costs 250 dollars, is marked down 20 percent, then the sale price is taxed 6 percent. What is the final price in dollars?",
        target: "212",
        held_out: true,
    },
];

/// Deterministic last-number judge (ported from the A39 canonical judge
/// pack; numeric comparison of the LAST number token in decoded text).
pub fn last_number_eq(decoded: &str, target: &str) -> (bool, String) {
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
pub fn jstr(s: &str) -> String {
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
pub fn extract_str(body: &str, key: &str) -> Option<String> {
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
pub fn extract_u64(body: &str, key: &str) -> Option<u64> {
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

/// Split OMLX_URL into (host, port, path). http localhost only.
pub fn split_url(url: &str) -> Result<(String, u16, String), String> {
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

/// One chat completion. Returns (content text, total_tokens).
pub fn chat_once(
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

/// Bundle text rendered into the probe's system prompt: the ONLY channel by
/// which a candidate bundle can affect probe quality.
pub fn bundle_system_prompt(b: &crate::rsi::Bundle) -> String {
    format!(
        "You are an arithmetic answerer. Follow these instructions exactly:\n{}\nEnd every reply with: FINAL <number> and nothing after it.",
        b.components.join("\n")
    )
}

/// Human-readable bundle instruction text for receipts (components joined).
pub fn bundle_text(b: &crate::rsi::Bundle) -> String {
    b.components.join(" | ")
}
