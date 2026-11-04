//! Live carved-swarm REPL on ONE frozen Muse Glimmer server (one weight set).
//!
//! Type a document or a task in real time: `ingest` carves text into atoms across
//! the workers' wells (real Surreal rows, real memberships), planned-prefills each
//! atom's exact K/V once, and `ask` / `task` route each question to the worker that
//! owns the best-matching atom. That worker recalls ONLY from its own well through
//! the real `globalmind-mcp` child (`mind.recall_scoped`), then answers by
//! continuing from the cached pages on the shared server. Disposable in-memory
//! Surreal; the persistent mind (8000) is never used.
//!
//! Env: ARES_GLIMMER_NATIVE_URL (isolated server), ARES_MCP_MIND_BIN, ARES_MCP_MIND_ROOT,
//!      GLOBALMIND_SCHEMA_DIR; optional SWARM_WORKERS (default 4), SWARM_MAX_TOKENS (24),
//!      SWARM_PRELOAD=demo|<file> (carve + planned-prefill before the demo starts).
//! Run:  cargo run --release --example swarm_live   (type `help`)

use ares_backend::harness::mcp::McpAuthority;
use ares_backend::harness::mind_recall_executor::MindRecallExecutor;
use ares_backend::harness::native_glimmer_executor::{
    decode_forward_response, decode_token_response, encode_forward_request, encode_token_request,
    verified_recall_text,
};
use ares_backend::harness::native_pages_client::{
    decode_prefill_response, encode_continue_request, encode_prefill_request, Pages, Shape,
    MAX_PAGE_TOKENS,
};
use ares_backend::harness::runtime::Executor;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const NS: &str = "globalmind_swarm_live";
const DB: &str = "swarm";
const SCHEMAS: &[&str] = &[
    "0001_hypermind_graph.surql",
    "0003_nested_hyperbolic_kv.surql",
    "0004_transformer_kv_tensor_pages.surql",
    "0005_universal_atom_lineage.surql",
    "0006_immutable_atom_address.surql",
    "0007_globalmind_distillation_trajectories.surql",
    "0008_superposition_holding.surql",
    "0009_immutable_tensor_kv.surql",
    "0010_native_distillation_ingress.surql",
    "0011_atomic_trajectory_cells.surql",
];
const WORKER_NAMES: [&str; 8] = [
    "scout",
    "analyst",
    "archivist",
    "engineer",
    "medic",
    "pilot",
    "cartographer",
    "warden",
];
const STOP: &[&str] = &[
    "the", "a", "an", "of", "is", "are", "was", "were", "to", "in", "on", "at", "for", "and", "or",
    "what", "which", "who", "how", "does", "do", "did", "where", "when", "why", "that", "this",
    "by", "with", "it", "its", "as", "from", "be", "has", "have", "had", "many", "much",
];
const DEMO_DOC: &str = "The Zorblax reactor at Quillan Station vents coolant every 47 minutes. \
Captain Elowen Marsh named her ship the Tern of Halvard. \
The village of Brindlemoor paints every door violet in the month of March. \
Professor Okonkwo-Reyes discovered the mineral talvorite in the Kesh Mountains. \
The annual Gantry Lantern Festival is held on the ninth day of Harvest Moon. \
Vessel Q-17 carries exactly 312 crates of saffron to Port Alderon. \
The lighthouse keeper at Cape Merrow rings a bronze bell at every low tide. \
Dr. Ilsabet Fenn invented the pressure loom that weaves glass thread.";

struct Surreal {
    child: Child,
    port: u16,
    pass: String,
}

impl Surreal {
    fn start() -> Result<Self, String> {
        let l = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = l.local_addr().map_err(|e| e.to_string())?.port();
        drop(l);
        if port == 8000 {
            return Err("refusing the live Surreal port".into());
        }
        let pass = format!("ephemeral_{}", std::process::id());
        let child = Command::new("/opt/homebrew/bin/surreal")
            .args([
                "start",
                "--no-banner",
                "--log",
                "none",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--user",
                "root",
                "--pass",
                &pass,
                "memory",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let me = Self { child, port, pass };
        for _ in 0..300 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return Ok(me);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err("isolated Surreal did not open".into())
    }

    fn sql(&self, sql: &str) -> Result<(), String> {
        let c = reqwest::blocking::Client::new();
        let base = format!("http://127.0.0.1:{}", self.port);
        let signin: serde_json::Value = c
            .post(format!("{base}/signin"))
            .header("accept", "application/json")
            .json(&serde_json::json!({"user":"root","pass":self.pass}))
            .send()
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;
        let token = signin["token"].as_str().ok_or("no Surreal token")?;
        let rows: serde_json::Value = c
            .post(format!("{base}/sql"))
            .header("accept", "application/json")
            .bearer_auth(token)
            .body(format!("USE NS {NS} DB {DB}; {sql}"))
            .send()
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;
        for (i, row) in rows
            .as_array()
            .ok_or("bad SQL result")?
            .iter()
            .enumerate()
            .skip(1)
        {
            if row["status"].as_str() == Some("ERR") {
                return Err(format!("statement {i}: {}", row["result"]));
            }
        }
        Ok(())
    }
}

impl Drop for Surreal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Atom {
    id: usize,
    text: String,
    worker: usize,
    a: f64,
    words: BTreeSet<String>,
    pages: Option<(Shape, Pages)>,
}

struct Swarm {
    url: String,
    db: Surreal,
    mind: MindRecallExecutor,
    workers: usize,
    max_tokens: usize,
    atoms: Vec<Atom>,
    composite: Option<[u8; 32]>,
    stats: HashMap<usize, (usize, f64)>,
    kv_hits: usize,
    kv_miss: usize,
    prefill: bool,
}

fn words(text: &str) -> BTreeSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| w.len() > 1 && !STOP.contains(w))
        .map(str::to_string)
        .collect()
}

const ABBREVIATIONS: &[&str] = &[
    "dr", "mr", "mrs", "ms", "prof", "st", "jr", "sr", "vs", "capt", "col",
];

/// Sentence split that does not cut after titles such as "Dr." or "Prof.".
fn sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '\n' {
            if !cur.trim().is_empty() {
                out.push(cur.trim().to_string());
            }
            cur.clear();
            continue;
        }
        cur.push(c);
        if matches!(c, '.' | '!' | '?') {
            let next_upper = chars[i + 1..]
                .iter()
                .find(|x| !x.is_whitespace())
                .is_none_or(|x| x.is_uppercase());
            let at_break = chars.get(i + 1).is_none_or(|x| x.is_whitespace());
            let last_word = cur
                .trim_end_matches(['.', '!', '?'])
                .rsplit(char::is_whitespace)
                .next()
                .unwrap_or("")
                .to_lowercase();
            if at_break && next_upper && !ABBREVIATIONS.contains(&last_word.as_str()) {
                out.push(cur.trim().to_string());
                cur.clear();
            }
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn chunks(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for s in sentences(text) {
        if s.len() <= 170 {
            out.push(s);
        } else {
            // Over-long sentence: split on commas/semicolons, then hard-wrap.
            let mut cur = String::new();
            for part in s.split_inclusive([',', ';']) {
                if cur.len() + part.len() > 160 && !cur.is_empty() {
                    out.push(cur.trim().to_string());
                    cur.clear();
                }
                cur.push_str(part);
            }
            if !cur.trim().is_empty() {
                out.push(cur.trim().to_string());
            }
        }
    }
    out
}

impl Swarm {
    fn native_tokens(&self, text: &str, bos: bool) -> Result<Vec<u32>, String> {
        let req = encode_token_request(text, bos)?;
        let resp = reqwest::blocking::Client::new()
            .post(format!("{}/v1/tokenize-native", self.url))
            .body(req.clone())
            .send()
            .map_err(|e| e.to_string())?
            .bytes()
            .map_err(|e| e.to_string())?;
        Ok(decode_token_response(&req, &resp)?.token_ids)
    }

    fn post(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let resp = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(900))
            .build()
            .map_err(|e| e.to_string())?
            .post(format!("{}{path}", self.url))
            .body(body)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("{path}: {}", resp.status()));
        }
        Ok(resp.bytes().map_err(|e| e.to_string())?.to_vec())
    }

    fn detokenize(&self, ids: &[u32]) -> Result<String, String> {
        let mut req = b"GDT2".to_vec();
        req.extend_from_slice(&(ids.len() as u32).to_le_bytes());
        for id in ids {
            req.extend_from_slice(&id.to_le_bytes());
        }
        let resp = self.post("/v1/detokenize-native", req.clone())?;
        if resp.len() < 40 || &resp[..4] != b"GDR2" {
            return Err("detokenize header invalid".into());
        }
        let len = u32::from_le_bytes(resp[4..8].try_into().unwrap()) as usize;
        let mut bound = req;
        bound.extend_from_slice(&resp[..resp.len() - 32]);
        if resp.len() != 40 + len || Sha256::digest(&bound)[..] != resp[resp.len() - 32..] {
            return Err("detokenize receipt mismatch".into());
        }
        Ok(String::from_utf8_lossy(&resp[8..8 + len]).to_string())
    }

    fn ingest(&mut self, text: &str) -> Result<(), String> {
        let pieces = chunks(text);
        if pieces.is_empty() {
            return Err("nothing to ingest".into());
        }
        println!(
            "carving {} atoms across {} workers...",
            pieces.len(),
            self.workers
        );
        for piece in pieces {
            let id = self.atoms.len();
            let worker = id % self.workers;
            let a = 0.1 + 0.09 * id as f64;
            let digest = format!("{:x}", Sha256::digest(piece.as_bytes()));
            let esc = piece.replace('\'', "\\'");
            let well_digest = format!("{:x}", Sha256::digest(b"swarm-live-well"));
            let cls = format!("{:x}", Sha256::digest(format!("cls-{id}").as_bytes()));
            self.db.sql(&format!(
                "CREATE atom:`live_atom_{id}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'live_atom_{id}', content_kind: 'text', content_text: '{esc}', content_uri: NONE, content_digest: '{digest}', domain: 'live', rank: 0, salience: 1.0, provenance: 'operator-ingest', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
                 CREATE atom_address:`live_addr_{id}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`live_atom_{id}`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE; \
                 CREATE address_well_membership CONTENT {{ workspace: workspace:`globalmind`, address: atom_address:`live_addr_{id}`, well: cognitive_well:`live_well_{worker}`, classifier_version: 'swarm-live-v1', classifier_digest: '{cls}', boundary_receipt_digest: '{well_digest}' }} RETURN NONE;",
                a.cosh(), a.sinh()
            ))?;
            let mut pages = None;
            let mut note = String::from("no prefill");
            if self.prefill {
                let ids = self.native_tokens(&format!("{piece}\n"), true)?;
                if ids.len() <= MAX_PAGE_TOKENS {
                    let started = Instant::now();
                    let req = encode_prefill_request(&ids)?;
                    let resp = self.post("/v1/prefill-exact", req.clone())?;
                    let pre = decode_prefill_response(&req, &resp)?;
                    self.composite.get_or_insert(pre.composite);
                    note = format!(
                        "planned prefill {} tokens in {:.1}s",
                        ids.len(),
                        started.elapsed().as_secs_f64()
                    );
                    pages = Some((pre.shape, pre.pages));
                } else {
                    note = format!("{} tokens > {MAX_PAGE_TOKENS}: no prefill cache", ids.len());
                }
            }
            println!("  atom {id} -> {} [{note}]: {piece}", WORKER_NAMES[worker]);
            self.atoms.push(Atom {
                id,
                text: piece.clone(),
                worker,
                a,
                words: words(&piece),
                pages,
            });
        }
        Ok(())
    }

    fn route(&self, question: &str) -> Option<(usize, usize)> {
        let q = words(question);
        self.atoms
            .iter()
            .map(|a| (a.id, a.words.intersection(&q).count()))
            .filter(|(_, s)| *s > 0)
            .max_by_key(|(id, s)| (*s, usize::MAX - *id))
    }

    /// Answer `question` as `worker`, recalling only from that worker's well at `target`'s address.
    fn answer(&mut self, worker: usize, target: usize, question: &str) -> Result<(), String> {
        let a = self.atoms[target].a;
        let args = serde_json::json!({
            "workspace": "globalmind", "well_ids": [format!("live_well_{worker}")],
            "query_h3": [a.cosh(), a.sinh(), 0.0, 0.0], "query_t2": [0.1, 0.2],
            "query_provenance": "swarm-live", "k": 1
        })
        .to_string();
        let started = Instant::now();
        let out = self.mind.call("mind.recall_scoped", &args)?;
        let text = verified_recall_text(&out.inner_text)?;
        let atom_id = serde_json::from_str::<serde_json::Value>(&out.inner_text)
            .ok()
            .and_then(|v| v["members"][0]["atom"].as_str().map(str::to_string))
            .unwrap_or_default();
        let recalled = self.atoms.iter().position(|x| x.text == text);
        let cont = self.native_tokens(&format!("Question: {question}\nAnswer:"), false)?;
        let mut kv = "full forward (no cached pages)".to_string();
        let (req, response) = match recalled.and_then(|i| self.atoms[i].pages.clone()) {
            Some((shape, pages)) if cont.len() <= 64 => {
                self.kv_hits += 1;
                kv = format!(
                    "KV HIT: reused {} prefix tokens from planned-prefill pages (no re-prefill)",
                    shape.tokens
                );
                let req = encode_continue_request(&shape, &pages, &cont, self.max_tokens)?;
                let resp = self.post("/v1/continue-exact", req.clone())?;
                (req, resp)
            }
            _ => {
                self.kv_miss += 1;
                let mut ids = self.native_tokens(&format!("{text}\n"), true)?;
                ids.extend(&cont);
                let req = encode_forward_request(&ids, self.max_tokens)?;
                let resp = self.post("/v1/forward-exact", req.clone())?;
                (req, resp)
            }
        };
        let r = decode_forward_response(&req, &response)?;
        match self.composite {
            Some(c) if c != r.composite => return Err("weight identity changed!".into()),
            _ => self.composite = Some(r.composite),
        }
        let answer = self.detokenize(&r.output_ids)?;
        let elapsed = started.elapsed().as_secs_f64();
        let entry = self.stats.entry(worker).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += elapsed;
        let line = answer.split('\n').next().unwrap_or("").trim();
        println!(
            "  worker    : {} (scope: well {worker} only)",
            WORKER_NAMES[worker]
        );
        if recalled.map(|i| self.atoms[i].id) != Some(target) {
            println!(
                "  BLOCKED   : the best-matching atom {target} lives in {}'s well; {} can only recall its own",
                WORKER_NAMES[self.atoms[target].worker], WORKER_NAMES[worker]
            );
        }
        println!("  recalled  : {atom_id} \"{text}\"");
        println!("  path      : {kv}");
        println!("  answer    : {line}");
        println!(
            "  receipt   : weights {}…  kv {}…  ({elapsed:.1}s)",
            &hex(&r.composite)[..12],
            &hex(&r.kv)[..12]
        );
        Ok(())
    }

    fn ask(&mut self, question: &str) -> Result<(), String> {
        let Some((target, score)) = self.route(question) else {
            return Err("no atom shares words with that question; ingest more text".into());
        };
        let worker = self.atoms[target].worker;
        println!(
            "router: best atom {target} (word overlap {score}) belongs to {}",
            WORKER_NAMES[worker]
        );
        self.answer(worker, target, question)
    }

    fn status(&self) {
        println!(
            "one weight set: {}",
            self.composite
                .map(|c| hex(&c))
                .unwrap_or_else(|| "(not yet seen)".into())
        );
        println!(
            "atoms: {}  kv hits: {}  full forwards: {}",
            self.atoms.len(),
            self.kv_hits,
            self.kv_miss
        );
        for (w, name) in WORKER_NAMES.iter().enumerate().take(self.workers) {
            let n = self.atoms.iter().filter(|a| a.worker == w).count();
            let (q, t) = self.stats.get(&w).copied().unwrap_or((0, 0.0));
            println!("  {name:<12} well {w}: {n} atoms, {q} answers, {t:.1}s");
        }
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() -> Result<(), String> {
    let env = |k: &str| std::env::var(k).map_err(|_| format!("{k} required"));
    let url = env("ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let schema_dir = env("GLOBALMIND_SCHEMA_DIR")?;
    let workers: usize = std::env::var("SWARM_WORKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4)
        .clamp(1, 8);
    let max_tokens: usize = std::env::var("SWARM_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24)
        .clamp(1, 128);
    print!("booting disposable mind (Surreal + schemas + {workers} wells)... ");
    std::io::stdout().flush().ok();
    let db = Surreal::start()?;
    for name in SCHEMAS {
        db.sql(
            &std::fs::read_to_string(format!("{schema_dir}/{name}")).map_err(|e| e.to_string())?,
        )?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    db.sql("UPSERT geometry_space:`h3_t2_r12_v1` CONTENT { workspace: workspace:`globalmind` } RETURN NONE;").ok();
    let well_digest = format!("{:x}", Sha256::digest(b"swarm-live-well-contract"));
    for (w, name) in WORKER_NAMES.iter().enumerate().take(workers) {
        let a = 0.3 + 0.3 * w as f64;
        db.sql(&format!(
            "CREATE cognitive_well:`live_well_{w}` CONTENT {{ workspace: workspace:`globalmind`, geometry_space: geometry_space:`h3_t2_r12_v1`, ordinal: {w}, label: '{}', h3_center_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_sector: {w}, boundary_contract_digest: '{well_digest}', status: 'specified' }} RETURN NONE;",
            name, a.cosh(), a.sinh()
        ))?;
    }
    let authority = McpAuthority {
        base: format!("http://127.0.0.1:{}", db.port),
        ns: NS.into(),
        db: DB.into(),
    };
    let mind = MindRecallExecutor::from_state(authority, ("root".into(), db.pass.clone()))?;
    println!("ready.");
    let mut swarm = Swarm {
        url,
        db,
        mind,
        workers,
        max_tokens,
        atoms: Vec::new(),
        composite: None,
        stats: HashMap::new(),
        kv_hits: 0,
        kv_miss: 0,
        prefill: true,
    };
    // Planned work happens before the audience arrives: carve + prefill once.
    match std::env::var("SWARM_PRELOAD").as_deref() {
        Ok("demo") => swarm.ingest(DEMO_DOC)?,
        Ok(path) if !path.is_empty() => {
            swarm.ingest(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?
        }
        _ => {}
    }
    println!("CARVED SWARM on one frozen Muse Glimmer. Type `help`.");
    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        print!("swarm> ");
        std::io::stdout().flush().ok();
        line.clear();
        if stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?
            == 0
        {
            break;
        }
        let cmd = line.trim();
        let (head, rest) = cmd.split_once(' ').unwrap_or((cmd, ""));
        let result: Result<(), String> = match head {
            "" => continue,
            "help" => {
                println!("  demo                   load a built-in novel-facts document\n  ingest <text|@file>    carve text into atoms across workers (planned prefill)\n  ask <question>         route to the owning worker; scoped recall; answer\n  task <q1> ; <q2> ; ...  run several questions back to back (set it loose)\n  as <n> <question>      force worker n (cross-view: it can only see its own well)\n  atoms | status         inspect\n  prefill on|off         toggle planned prefill for future ingests\n  quit");
                Ok(())
            }
            "demo" => swarm.ingest(DEMO_DOC),
            "ingest" => match rest.strip_prefix('@') {
                Some(path) => std::fs::read_to_string(path.trim()).map_err(|e| e.to_string()).and_then(|t| swarm.ingest(&t)),
                None => swarm.ingest(rest),
            },
            "ask" => swarm.ask(rest),
            "task" => {
                let started = Instant::now();
                let questions: Vec<&str> = rest.split(';').map(str::trim).filter(|q| !q.is_empty()).collect();
                let mut res = Ok(());
                for (i, q) in questions.iter().enumerate() {
                    println!("\n[{}/{}] {q}", i + 1, questions.len());
                    if let Err(e) = swarm.ask(q) {
                        println!("  ! {e}");
                    }
                }
                println!("\ntask done: {} questions in {:.1}s", questions.len(), started.elapsed().as_secs_f64());
                swarm.status();
                if questions.is_empty() { res = Err("task needs questions separated by ;".into()); }
                res
            }
            "as" => match rest.split_once(' ') {
                Some((n, q)) => match (n.parse::<usize>(), swarm.route(q)) {
                    (Ok(w), Some((target, _))) if w < swarm.workers => swarm.answer(w, target, q),
                    _ => Err("usage: as <worker number> <question> (and the question must match an atom)".into()),
                },
                None => Err("usage: as <worker number> <question>".into()),
            },
            "atoms" => {
                for a in &swarm.atoms {
                    println!("  atom {:>2} [{}] {}", a.id, WORKER_NAMES[a.worker], a.text);
                }
                Ok(())
            }
            "status" => {
                swarm.status();
                Ok(())
            }
            "prefill" => {
                swarm.prefill = rest.trim() != "off";
                println!("planned prefill {}", if swarm.prefill { "on" } else { "off" });
                Ok(())
            }
            "quit" | "exit" => break,
            _ => Err("unknown command; type help".into()),
        };
        if let Err(e) = result {
            println!("  ! {e}");
        }
    }
    let _ = std::io::stdin().read(&mut []);
    Ok(())
}
