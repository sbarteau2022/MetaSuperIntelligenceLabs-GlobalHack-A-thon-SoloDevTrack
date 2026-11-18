//! One-page UI for the live carved swarm: IDE | chat | telemetry, loopback only.
//!
//! Drives the tested `swarm_live` REPL as a child process (nothing new touches the
//! model path). Chat lines go to its stdin; its stdout is logged and polled by the
//! page, which parses receipts into telemetry. The IDE pane browses and edits files
//! inside the repo only (no `.git`, `target`, `node_modules`; 400 KB cap).
//!
//! Env: REPO_ROOT (repo to expose), SWARM_LIVE_BIN (path to the swarm_live binary),
//!      SWARM_UI_PORT (default 4820); the swarm_live env (ARES_GLIMMER_NATIVE_URL etc.) is inherited.

use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

const PAGE: &str = include_str!("swarm_ui.html");
const MAX_FILE: usize = 400 * 1024;

struct State {
    log: Vec<String>,
    ready: bool,
    stdin: std::process::ChildStdin,
}

fn json_escape(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

fn respond(stream: &mut TcpStream, status: &str, ctype: &str, body: &str) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}

fn safe_path(root: &Path, rel: &str) -> Option<PathBuf> {
    if rel.contains("..")
        || rel
            .split('/')
            .any(|p| matches!(p, ".git" | "target" | "node_modules"))
    {
        return None;
    }
    let joined = root.join(rel.trim_start_matches('/'));
    let canon = joined.canonicalize().ok().or_else(|| {
        // A new file: canonicalize its parent instead.
        let parent = joined.parent()?.canonicalize().ok()?;
        Some(parent.join(joined.file_name()?))
    })?;
    canon.starts_with(root).then_some(canon)
}

fn query_param(query: &str, key: &str) -> String {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return percent_decode(v);
            }
        }
    }
    String::new()
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn handle(
    mut stream: TcpStream,
    root: Arc<PathBuf>,
    st: Arc<Mutex<State>>,
    port: u16,
    token: Arc<String>,
) {
    let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
    let mut first = String::new();
    if reader.read_line(&mut first).is_err() {
        return;
    }
    let mut parts = first.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut len = 0usize;
    let (mut host, mut origin, mut tok) = (String::new(), None::<String>, String::new());
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).is_err() || h.trim().is_empty() {
            break;
        }
        let lower = h.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            len = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = lower.strip_prefix("host:") {
            host = v.trim().to_string();
        } else if lower.starts_with("origin:") {
            origin = Some(h[7..].trim().to_string());
        } else if lower.starts_with("x-swarm-token:") {
            tok = h[14..].trim().to_string();
        }
    }
    // DNS-rebinding / CSRF guard: exact loopback Host, matching Origin when present,
    // and (for every API call) an unguessable per-launch token in a custom header.
    let host_ok = host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}");
    let origin_ok = origin.as_deref().is_none_or(|o| {
        o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}")
    });
    let is_page = method == "GET" && path == "/";
    if !host_ok || !origin_ok || (!is_page && tok != *token) {
        return respond(&mut stream, "403 Forbidden", "text/plain", "forbidden");
    }
    let mut body = vec![0u8; len.min(2 * MAX_FILE)];
    let _ = reader.read_exact(&mut body);
    let body = String::from_utf8_lossy(&body).to_string();

    match (method, path) {
        ("GET", "/") => {
            // SWARM_UI_HTML (a path) is read per request so the page can be edited without a restart.
            let page = std::env::var("SWARM_UI_HTML")
                .ok()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .unwrap_or_else(|| PAGE.to_string());
            let page = page
                .replace("__ROOT__", &root.display().to_string())
                .replace("__TOKEN__", &token);
            respond(&mut stream, "200 OK", "text/html; charset=utf-8", &page)
        }
        ("GET", "/api/log") => {
            let from: usize = query_param(query, "from").parse().unwrap_or(0);
            let s = st.lock().unwrap();
            let lines: Vec<&String> = s.log.iter().skip(from).collect();
            respond(
                &mut stream,
                "200 OK",
                "application/json",
                &format!(
                    "{{\"lines\":{},\"next\":{},\"ready\":{}}}",
                    serde_json::to_string(&lines).unwrap_or_else(|_| "[]".into()),
                    s.log.len(),
                    s.ready
                ),
            );
        }
        ("POST", "/api/send") => {
            let line = body.trim().replace('\n', " ");
            let mut s = st.lock().unwrap();
            if line.is_empty() {
                respond(&mut stream, "400 Bad Request", "text/plain", "empty");
            } else if !s.ready {
                respond(&mut stream, "409 Conflict", "text/plain", "swarm is busy");
            } else {
                s.ready = false;
                s.log.push(format!("\u{1}> {line}"));
                let ok = writeln!(s.stdin, "{line}")
                    .and_then(|_| s.stdin.flush())
                    .is_ok();
                respond(
                    &mut stream,
                    if ok {
                        "200 OK"
                    } else {
                        "500 Internal Server Error"
                    },
                    "text/plain",
                    "ok",
                );
            }
        }
        ("GET", "/api/files") => {
            let rel = query_param(query, "path");
            let Some(dir) = safe_path(&root, &rel).filter(|p| p.is_dir()) else {
                return respond(
                    &mut stream,
                    "404 Not Found",
                    "text/plain",
                    "not a directory",
                );
            };
            let mut items: Vec<(bool, String)> = std::fs::read_dir(dir)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            if name.starts_with('.')
                                || matches!(name.as_str(), "target" | "node_modules")
                            {
                                return None;
                            }
                            Some((e.path().is_dir(), name))
                        })
                        .collect()
                })
                .unwrap_or_default();
            items.sort_by(|a, b| {
                b.0.cmp(&a.0)
                    .then(a.1.to_lowercase().cmp(&b.1.to_lowercase()))
            });
            let json: Vec<String> = items
                .iter()
                .map(|(d, n)| format!("{{\"dir\":{d},\"name\":{}}}", json_escape(n)))
                .collect();
            respond(
                &mut stream,
                "200 OK",
                "application/json",
                &format!("[{}]", json.join(",")),
            );
        }
        ("GET", "/api/file") => {
            let rel = query_param(query, "path");
            match safe_path(&root, &rel).filter(|p| p.is_file()) {
                Some(p) => match std::fs::read(&p) {
                    Ok(b) if b.len() <= MAX_FILE && std::str::from_utf8(&b).is_ok() => respond(
                        &mut stream,
                        "200 OK",
                        "text/plain; charset=utf-8",
                        &String::from_utf8_lossy(&b),
                    ),
                    _ => respond(
                        &mut stream,
                        "413 Payload Too Large",
                        "text/plain",
                        "binary or over 400 KB",
                    ),
                },
                None => respond(&mut stream, "404 Not Found", "text/plain", "not a file"),
            }
        }
        ("POST", "/api/save") => {
            let rel = query_param(query, "path");
            match safe_path(&root, &rel) {
                Some(p) if body.len() <= MAX_FILE && !p.is_dir() => {
                    let tmp = p.with_extension("swarm-ui-tmp");
                    let ok = std::fs::write(&tmp, &body)
                        .and_then(|_| std::fs::rename(&tmp, &p))
                        .is_ok();
                    respond(
                        &mut stream,
                        if ok {
                            "200 OK"
                        } else {
                            "500 Internal Server Error"
                        },
                        "text/plain",
                        if ok { "saved" } else { "write failed" },
                    );
                }
                _ => respond(&mut stream, "400 Bad Request", "text/plain", "path refused"),
            }
        }
        _ => respond(&mut stream, "404 Not Found", "text/plain", "not found"),
    }
}

fn main() -> Result<(), String> {
    let root = PathBuf::from(std::env::var("REPO_ROOT").map_err(|_| "REPO_ROOT required")?)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let bin = std::env::var("SWARM_LIVE_BIN").map_err(|_| "SWARM_LIVE_BIN required")?;
    let port: u16 = std::env::var("SWARM_UI_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4820);
    let mut child = Command::new(&bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn {bin}: {e}"))?;
    let stdin = child.stdin.take().ok_or("no child stdin")?;
    let mut stdout = child.stdout.take().ok_or("no child stdout")?;
    let st = Arc::new(Mutex::new(State {
        log: vec!["starting the swarm (loading the mind, carving the document)...".into()],
        ready: false,
        stdin,
    }));
    {
        let st = st.clone();
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stdout.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                let mut s = st.lock().unwrap();
                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=pos).collect();
                    let mut text = String::from_utf8_lossy(&line).trim_end().to_string();
                    while let Some(rest) = text.strip_prefix("swarm> ") {
                        text = rest.to_string();
                    }
                    if !text.is_empty() {
                        s.log.push(text);
                    }
                }
                if String::from_utf8_lossy(&buf).trim_end().ends_with("swarm>") {
                    buf.clear();
                    s.ready = true;
                }
            }
            let mut s = st.lock().unwrap();
            s.log.push("(swarm process exited)".into());
            s.ready = false;
        });
    }
    let token = Arc::new({
        let mut raw = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut raw))
            .map_err(|e| format!("urandom: {e}"))?;
        raw.iter().map(|b| format!("{b:02x}")).collect::<String>()
    });
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    println!(
        "swarm UI: http://127.0.0.1:{port}  (repo: {})",
        root.display()
    );
    let root = Arc::new(root);
    for stream in listener.incoming().flatten() {
        let (root, st, token) = (root.clone(), st.clone(), token.clone());
        std::thread::spawn(move || handle(stream, root, st, port, token));
    }
    let _ = child.kill();
    Ok(())
}
