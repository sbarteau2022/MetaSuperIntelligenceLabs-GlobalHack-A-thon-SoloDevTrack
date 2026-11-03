//! SurrealDB persistence layer for the single HyperMind graph authority.
//!
//! Transport is sync std-only HTTP/1.1 POST to `/sql` (text/plain body), so
//! the binary gains zero new dependencies. Every method returns
//! `Result<Value, String>`; authority-mode callers fail closed on errors.
//!
//! Namespace/database/endpoint come from env at `Store::from_env` time:
//! `HYPERMIND_SURREAL_URL` (default `http://127.0.0.1:8000`),
//! `HYPERMIND_SURREAL_NS` (default `hypermind`),
//! `HYPERMIND_SURREAL_DB` (default `core`).

use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurrealMode {
    /// No SurrealDB traffic at all.
    Off,
    /// Migration-only mode. MCP startup rejects it to prevent split authority.
    Shadow,
    /// SurrealDB is canonical (set only after parity passes).
    On,
}

impl SurrealMode {
    pub fn parse(s: &str) -> SurrealMode {
        match s {
            "off" => SurrealMode::Off,
            "on" => SurrealMode::On,
            _ => SurrealMode::Shadow,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Store {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub ns: String,
    pub db: String,
    pub mode: SurrealMode,
    /// Step 7: optional DB credentials. When set, every /sql request carries
    /// a Bearer token from /signin (cached per Store, refreshed on 401).
    /// Env: HYPERMIND_SURREAL_USER / HYPERMIND_SURREAL_PASS.
    /// No creds = anonymous request (fails closed once the server drops
    /// --unauthenticated, as it now has).
    pub user: Option<String>,
    pub pass: Option<String>,
}

impl Store {
    pub fn from_env() -> Store {
        let url = std::env::var("HYPERMIND_SURREAL_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let (host, port) = parse_host_port(&url);
        Store {
            url,
            host,
            port,
            ns: std::env::var("HYPERMIND_SURREAL_NS").unwrap_or_else(|_| "hypermind".into()),
            db: std::env::var("HYPERMIND_SURREAL_DB").unwrap_or_else(|_| "core".into()),
            user: std::env::var("HYPERMIND_SURREAL_USER").ok(),
            pass: std::env::var("HYPERMIND_SURREAL_PASS").ok(),
            mode: SurrealMode::parse(
                &std::env::var("HYPERMIND_SURREAL").unwrap_or_else(|_| "shadow".into()),
            ),
        }
    }

    pub fn enabled(&self) -> bool {
        self.mode != SurrealMode::Off
    }

    /// Token for this Store's credentials (signin per call is cheap on
    /// loopback and avoids shared expiry state; 15s timeouts bound it).
    fn token(&self) -> Result<Option<String>, String> {
        let (user, pass) = match (&self.user, &self.pass) {
            (Some(u), Some(p)) => (u, p),
            _ => return Ok(None),
        };
        // Root users authenticate at root scope; adding ns/db changes the
        // requested auth scope and SurrealDB 3.3 rejects valid root creds.
        // Database-scoped users still require ns/db in the signin payload.
        let body = if user == "root" {
            format!("{{\"user\":{},\"pass\":{}}}", sq(user), sq(pass))
        } else {
            format!(
                "{{\"user\":{},\"pass\":{},\"ns\":{},\"db\":{}}}",
                sq(user),
                sq(pass),
                sq(&self.ns),
                sq(&self.db)
            )
        };
        let resp = http_post_raw(&self.host, self.port, "/signin", "application/json", &body, None)?;
        serde_json::from_str::<Value>(&resp)
            .ok()
            .and_then(|v| v.get("token").and_then(|t| t.as_str()).map(|x| x.to_string()))
            .map(Some)
            .ok_or_else(|| "surreal: signin failed — check user/pass".to_string())
    }

    /// Raw multi-statement SurrealQL; returns the per-statement results array.
    pub fn sql(&self, query: &str) -> Result<Vec<Value>, String> {
        if !self.enabled() {
            return Err("surreal: disabled (HYPERMIND_SURREAL=off)".into());
        }
        let body = format!("USE NS {} DB {}; {}", escape_ident(&self.ns), escape_ident(&self.db), query);
        let tok = self.token()?;
        let resp_text = http_post_raw(&self.host, self.port, "/sql", "text/plain", &body, tok.as_deref())?;
        let parsed: Value =
            serde_json::from_str(&resp_text).map_err(|e| format!("surreal: bad json: {e}"))?;
        let arr = parsed
            .as_array()
            .ok_or_else(|| format!("surreal: expected array, got: {}", trunc(&resp_text)))?;
        let mut out = Vec::with_capacity(arr.len().saturating_sub(1));
        for (i, stmt) in arr.iter().enumerate() {
            if i == 0 {
                continue; // USE statement result
            }
            if stmt.get("status").and_then(|s| s.as_str()) == Some("ERR") {
                return Err(format!(
                    "surreal: statement {} failed: {}",
                    i,
                    stmt.get("result").map(|v| v.to_string()).unwrap_or_default()
                ));
            }
            out.push(stmt.get("result").cloned().unwrap_or(Value::Null));
        }
        Ok(out)
    }

    /// Single-row helper: exactly one result statement expected.
    pub fn one(&self, query: &str) -> Result<Value, String> {
        let mut r = self.sql(query)?;
        r.into_iter().next().ok_or_else(|| "surreal: empty result".into())
    }

    /// Fail closed if an upgrade points this binary at a different memory
    /// contract. Historical migration rows are append-only; the original 0004
    /// row is retained and its TPPE correction has a separate pinned version.
    pub fn verify_contract(&self) -> Result<(), String> {
        for (version, expected) in [
            ("0004_tppe_contract_correction", "31c467e26dd81d9fddf61e10b605e6726a25b7c0cb573c399d73d79fec7f1246"),
            ("0005_universal_atom_lineage", "a2c4a85303874233048b73d18e36538f38c05e36b2503c37499b6489a92729c0"),
            ("0006_immutable_atom_address", "f91fb405f54b5bac454808e571c116819b6329061b6ce44817447559f1449a04"),
        ] {
            let rows = self.one(&format!(
                "SELECT digest FROM schema_migration WHERE version = {} LIMIT 1;",
                sq(version)
            ))?;
            let actual = rows.as_array().and_then(|r| r.first())
                .and_then(|r| r.get("digest")).and_then(Value::as_str);
            if actual != Some(expected) {
                return Err(format!("surreal: contract migration {version} missing or drifted"));
            }
        }
        for (table, fields) in [
            ("transformer_kv_session", &["model_digest", "toroidal_phase_encoder_version", "address_contract_digest"][..]),
            ("transformer_kv_page", &["key_tensor", "value_tensor", "tensor_shape", "address"][..]),
            ("distillation_admission", &["model_source", "source_artifact_digest", "eligibility_receipt_digest"][..]),
            ("atom_origin", &["atom", "admission", "source_step_digest"][..]),
            ("distilled_trajectory", &["atoms", "trajectory_digest", "judge_receipt_digest"][..]),
            ("trajectory_origin", &["trajectory", "admission", "source_walk_digest"][..]),
            ("atom", &["external_id", "content_text", "content_digest", "provenance"][..]),
            ("atom_address", &["atom", "h3_lorentz", "t2_phase", "revision"][..]),
            ("schema_migration", &["version", "digest"][..]),
        ] {
            let info = self.one(&format!("INFO FOR TABLE {table};"))?;
            let definitions = info.get("fields").and_then(Value::as_object)
                .ok_or_else(|| format!("surreal: contract table {table} missing"))?;
            for field in fields {
                let definition = definitions.get(*field).and_then(Value::as_str)
                    .ok_or_else(|| format!("surreal: contract field {table}.{field} missing"))?;
                if (table == "transformer_kv_page" && (*field == "key_tensor" || *field == "value_tensor"))
                    && !definition.contains("TYPE bytes") {
                    return Err(format!("surreal: contract field {table}.{field} is not binary"));
                }
                if (table == "schema_migration" || table == "distillation_admission"
                    || table == "atom_origin" || table == "distilled_trajectory"
                    || table == "trajectory_origin" || table == "atom"
                    || table == "atom_address") && !definition.contains("READONLY") {
                    return Err(format!("surreal: contract field {table}.{field} is mutable"));
                }
            }
            if table == "transformer_kv_session" && definitions.contains_key("position_encoding") {
                return Err("surreal: RoPE/Euclidean position field is inadmissible".into());
            }
        }
        Ok(())
    }

    pub fn upsert_atom(
        &self,
        id: u64,
        text: &str,
        salience: f64,
        rank: u8,
        provenance: &str,
        taint: &str,
        domain: &str,
        pinned: bool,
        digest: &str,
    ) -> Result<Value, String> {
        let q = format!(
            "CREATE atom:{} CONTENT {{ workspace: workspace:hypermind, external_id: {}, content_kind: 'text', content_text: {}, content_uri: NONE, content_digest: '{}', domain: {}, rank: {}, salience: {:?}, provenance: {}, taint: {}, pinned: {}, current_address: atom_address:{}, created_at: time::now(), updated_at: time::now() }} RETURN NONE;",
            id,
            sq(&id.to_string()),
            sq(text),
            digest,
            sq(domain),
            rank,
            salience,
            sq(provenance),
            sq(taint),
            pinned,
            id
        );
        self.one(&q)
    }

    pub fn upsert_address(
        &self,
        id: u64,
        coord: &[f64; 4],
        phase: &[f64; 2],
    ) -> Result<Value, String> {
        let q = format!(
            "CREATE atom_address:{} CONTENT {{ workspace: workspace:hypermind, atom: atom:{}, h3_lorentz: [{}], t2_phase: [{}], geometry_model: 'hash-idf-rank', geometry_version: 'H3xT2-Lorentz-canonical-v2', revision: 1, active: true, created_at: time::now() }} RETURN NONE;",
            id,
            id,
            floats(coord),
            floats(phase)
        );
        self.one(&q)
    }

    /// Admit a new atom and its first address in one transaction. CREATE
    /// refuses an existing id; neither record may be silently rewritten.
    pub fn create_atom_with_address(
        &self,
        id: u64,
        text: &str,
        salience: f64,
        rank: u8,
        provenance: &str,
        taint: &str,
        domain: &str,
        pinned: bool,
        digest: &str,
        coord: &[f64; 4],
        phase: &[f64; 2],
    ) -> Result<(), String> {
        let query = format!(
            "BEGIN TRANSACTION; \
             CREATE atom:{id} CONTENT {{ workspace: workspace:hypermind, external_id: {external_id}, content_kind: 'text', content_text: {text}, content_uri: NONE, content_digest: {digest}, domain: {domain}, rank: {rank}, salience: {salience:?}, provenance: {provenance}, taint: {taint}, pinned: {pinned}, current_address: atom_address:{id}, created_at: time::now(), updated_at: time::now() }} RETURN NONE; \
             CREATE atom_address:{id} CONTENT {{ workspace: workspace:hypermind, atom: atom:{id}, h3_lorentz: [{coord}], t2_phase: [{phase}], geometry_model: 'hash-idf-rank', geometry_version: 'H3xT2-Lorentz-canonical-v2', revision: 1, active: true, created_at: time::now() }} RETURN NONE; \
             COMMIT TRANSACTION;",
            external_id = sq(&id.to_string()),
            text = sq(text),
            digest = sq(digest),
            domain = sq(domain),
            provenance = sq(provenance),
            taint = sq(taint),
            coord = floats(coord),
            phase = floats(phase),
        );
        self.sql(&query)?;
        Ok(())
    }

    /// Explicit edge create. `kind` must already be validated by the caller
    /// against EXPLICIT_KINDS; this layer enforces the same gate defensively.
    /// `evidence` is required for `derived_from`; id is `from:to:kind:rev`.
    pub fn create_edge(
        &self,
        from: u64,
        to: u64,
        kind: &str,
        revision: u64,
        confidence: f64,
        weight: f64,
        producer: &str,
        evidence_digest: &str,
        evidence_atoms: &[u64],
    ) -> Result<Value, String> {
        if !EXPLICIT_KINDS.contains(&kind) && !GATED_KINDS.contains(&kind) {
            return Err(format!("surreal: kind '{kind}' is not creatable (explicit + gated-semantic only)"));
        }
        if !(0.0..=1.0).contains(&confidence) || !(0.0..=1.0).contains(&weight) {
            return Err("surreal: confidence and weight must be in [0,1]".into());
        }
        let ev: Vec<String> = evidence_atoms.iter().map(|a| format!("atom:{a}")).collect();
        let edge_id = format!("{from}_{to}_{kind}_{revision}");
        let q = format!(
            "RELATE atom:{from}->atom_edge:{edge_id}->atom:{to} CONTENT {{ workspace: workspace:hypermind, kind: '{kind}', weight: {weight:?}, confidence: {confidence:?}, evidence_atoms: [{ev}], evidence_digest: {evd}, producer: {prod}, revision: {revision}, active: true, valid_from: time::now(), valid_to: NONE, created_at: time::now() }} RETURN id;",
            from = from,
            to = to,
            edge_id = edge_id,
            kind = kind,
            weight = weight,
            confidence = confidence,
            ev = ev.join(", "),
            evd = sq(evidence_digest),
            prod = sq(producer),
            revision = revision
        );
        self.one(&q)
    }

    pub fn neighbors(&self, id: u64, kind: Option<&str>, limit: usize) -> Result<Value, String> {
        let kf = match kind {
            Some(k) => format!(" AND kind = '{}'", k.replace('\'', "")),
            None => String::new(),
        };
        let q = format!(
            "SELECT out.id AS id, kind, weight, confidence, revision FROM atom_edge WHERE in = atom:{}{} AND active = true LIMIT {};",
            id, kf, limit
        );
        self.one(&q)
    }

    /// Step 6: batched adjacency for viz. One round-trip: all active edges
    /// with `in` in `ids` (cap 2000). Returns (from, to, kind) triples.
    pub fn subgraph_edges(&self, ids: &[u64]) -> Result<Vec<(u64, u64, String)>, String> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let list: Vec<String> = ids.iter().take(2000).map(|id| format!("atom:{id}")).collect();
        let q = format!(
            "SELECT meta::id(in) AS f, meta::id(out) AS t, kind FROM atom_edge WHERE in IN [{}] AND active = true LIMIT 20000;",
            list.join(", ")
        );
        let v = self.one(&q)?;
        Ok(v
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|o| {
                        let num = |k: &str| -> Option<u64> {
                            o.get(k).and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse::<u64>().ok())))
                        };
                        Some((num("f")?, num("t")?, o.get("kind").and_then(|k| k.as_str()).unwrap_or("").to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Highest numeric external_id in the DB (0 when empty/error).
    /// Guards shadow dual-write against UPSERT-clobbering migrated rows
    /// with colliding fresh-map ids (incident 2026-11-01).
    pub fn max_external_id(&self) -> u64 {
        let rows = match self.one(
            "SELECT value external_id FROM atom;",
        ) {
            Ok(v) => v,
            Err(_) => return 0,
        };
        rows.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|x| x.parse::<u64>().ok())
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Candidate prefilter (step 4): lexical overlap via CONTAINSANY,
    /// bounded LIMIT. Returns external_ids in DB order; ranking ALWAYS
    /// happens in Rust (score() + BM25 fusion). No SEARCH/MTREE index exists
    /// in this SurrealDB build — this is a full-scan prefilter, honestly
    /// labeled. A future index-capable build can accelerate it without
    /// changing this contract.
    pub fn candidates(&self, terms: &[String], limit: usize) -> Result<Vec<u64>, String> {
        if terms.is_empty() || limit == 0 {
            return Ok(vec![]);
        }
        let capped: Vec<String> = terms.iter().take(16).map(|t| sq(t)).collect();
        let lim = limit.min(16000);
        let q = format!(
            "SELECT value external_id FROM atom WHERE content_text CONTAINSANY [{}] LIMIT {};",
            capped.join(", "),
            lim
        );
        let v = self.one(&q)?;
        Ok(v
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .filter_map(|x| x.parse::<u64>().ok())
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Step 5: upsert one canonical BGE-M3 text vector (1024D, normalized).
    /// Encoder/revision/dimensions are ASSERTed server-side by schema 0002.
    pub fn upsert_text_embedding(
        &self,
        external_id: &str,
        atom_id: u64,
        vector: &[f64],
        source_digest: &str,
    ) -> Result<Value, String> {
        if vector.len() != 1024 {
            return Err(format!("surreal: expected 1024D vector, got {}", vector.len()));
        }
        let vec = vector.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(", ");
        let q = format!(
            "UPSERT atom_embedding_text_v1:`{ext}` CONTENT {{ workspace: workspace:hypermind, atom: atom:{aid}, encoder: 'bge-m3', encoder_revision: '5617a9f61b028005a4858fdac845db406aefb181', dimensions: 1024, vector: [{vec}], normalized: true, source_digest: '{dg}', created_at: time::now() }} RETURN NONE;",
            ext = external_id.replace('`', ""),
            aid = atom_id,
            vec = vec,
            dg = source_digest
        );
        self.one(&q)
    }

    /// Step 5: brute-force cosine candidate lookup over the canonical text
    /// vectors (HNSW is defined as forward cover; the planner TableScans in
    /// this build — VERIFIED 2026-11-01). Returns (external_id, similarity).
    /// Exact Rust H3xT2 rerank stays authoritative downstream.
    pub fn vector_candidates(&self, vector: &[f64], limit: usize) -> Result<Vec<(u64, f64)>, String> {
        if vector.len() != 1024 {
            return Err(format!("surreal: expected 1024D query vector, got {}", vector.len()));
        }
        let vec = vector.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(", ");
        let lim = limit.min(16000).max(1);
        // NOTE: table rows are keyed atom_embedding_text_v1:`<external_id>`;
        // the id suffix IS the atom external id (no separate field).
        // NOTE 2: ORDER BY must reference a TOP-LEVEL projected field.
        // `SELECT value {..} ... ORDER BY sim` silently no-ops (sim is nested
        // inside VALUE) and returns storage order — caught by battery 2026-11-01
        // (identical top-k across queries). Project columns instead.
        let q = format!(
            "SELECT meta::id(id) AS ext, vector::similarity::cosine(vector, [{vec}]) AS sim FROM atom_embedding_text_v1 ORDER BY sim DESC LIMIT {lim};",
            vec = vec,
            lim = lim
        );
        let v = self.one(&q)?;
        Ok(v
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|o| {
                        let id = o.get("ext").and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse::<u64>().ok())))?;
                        let sim = o.get("sim").and_then(|x| x.as_f64())?;
                        Some((id, sim))
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Surreal-authority startup hydration. Addresses are dereferenced in the
    /// query so the MCP process needs no file snapshot or JSONL ledger.
    pub fn load_atoms(&self) -> Result<Value, String> {
        self.one(
            "SELECT external_id, content_text, salience, rank, provenance, taint, domain, pinned, current_address.h3_lorentz AS coord, current_address.t2_phase AS phase FROM atom ORDER BY external_id;",
        )
    }

    /// Auth errors must surface (fail LOUD); missing-table errors stay
    /// tolerable (fresh DB reports 0). Step-7 fix: on an authenticated
    /// server, swallowing 401/403 as zeros is indistinguishable from empty.
    fn is_auth_err(e: &str) -> bool {
        e.contains("401/403") || e.contains("unauthorized") || e.contains("Auth") || e.contains("Forbidden")
    }

    /// Step 5: (embedded vectors, total atoms) via two tolerant singles.
    pub fn counts_embed(&self) -> Result<(u64, u64), String> {
        let one = |q: &str| -> Result<u64, String> {
            match self.one(q) {
                Ok(v) => Ok(v
                    .as_array()
                    .and_then(|a| a.first().cloned())
                    .and_then(|o| o.get("n").and_then(|n| n.as_u64()))
                    .unwrap_or(0)),
                Err(e) if Self::is_auth_err(&e) => Err(e),
                Err(_) => Ok(0),
            }
        };
        Ok((
            one("SELECT count() AS n FROM atom_embedding_text_v1 GROUP ALL;")?,
            one("SELECT count() AS n FROM atom GROUP ALL;")?,
        ))
    }

    pub fn counts(&self) -> Result<(u64, u64, u64), String> {
        // Tolerant: a fresh DB may lack any of these tables (step-4 fix:
        // db_status errored on edge-less DBs instead of reporting 0).
        let one = |q: &str| -> Result<u64, String> {
            match self.one(q) {
                Ok(v) => Ok(v
                    .as_array()
                    .and_then(|a| a.first().cloned())
                    .and_then(|o| o.get("n").and_then(|n| n.as_u64()))
                    .unwrap_or(0)),
                Err(e) if Self::is_auth_err(&e) => Err(e),
                Err(_) => Ok(0),
            }
        };
        Ok((
            one("SELECT count() AS n FROM atom GROUP ALL;")?,
            one("SELECT count() AS n FROM atom_address GROUP ALL;")?,
            one("SELECT count() AS n FROM atom_edge GROUP ALL;")?,
        ))
    }
}

/// Edge kinds the runtime is allowed to create explicitly. `semantic` is
/// admitted only via the similarity+evidence gate in `handle_tool` (never by
/// free creation); `causal`, `supports`, `contradicts`, `temporal` are
/// FORBIDDEN to fabricate — requests for them are schema errors, not edges.
pub const EXPLICIT_KINDS: &[&str] = &["next", "references", "part_of", "derived_from"];
pub const GATED_KINDS: &[&str] = &["semantic"];
#[allow(dead_code)]
pub fn gated_kind(kind: &str) -> bool {
    GATED_KINDS.contains(&kind)
}
pub const FORBIDDEN_KINDS: &[&str] =
    &["causal", "temporal", "supports", "contradicts", "part_of_auto", "next_auto"];

pub fn sq(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
}

fn floats<const N: usize>(values: &[f64; N]) -> String {
    values.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(", ")
}

fn escape_ident(name: &str) -> String {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        name.to_string()
    } else {
        sq(name)
    }
}

fn trunc(s: &str) -> String {
    s.chars().take(300).collect()
}

fn parse_host_port(url: &str) -> (String, u16) {
    let noscheme = url.split("://").nth(1).unwrap_or(url);
    let hostport = noscheme.split('/').next().unwrap_or(noscheme);
    if let Some((h, p)) = hostport.split_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port);
        }
    }
    (hostport.to_string(), 8000)
}

fn http_post_raw(host: &str, port: u16, path: &str, content_type: &str, body: &str, auth: Option<&str>) -> Result<String, String> {
    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("surreal: connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let auth_h = auth.map(|t| format!("\r\nAuthorization: Bearer {t}")).unwrap_or_default();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: {content_type}\r\nAccept: application/json\r\nContent-Length: {}{auth_h}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
        addr = addr,
        path = path,
        content_type = content_type,
        auth_h = auth_h,
        body = body
    );
    stream.write_all(req.as_bytes()).map_err(|e| format!("surreal: write: {e}"))?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| format!("surreal: read: {e}"))?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    if text.starts_with("HTTP/1.1 401") || text.starts_with("HTTP/1.1 403") {
        return Err("surreal: unauthorized (401/403) — check HYPERMIND_SURREAL_USER/PASS".into());
    }
    let sep = text.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    Ok(text[sep..].to_string())
}

fn http_post_sql(host: &str, port: u16, body: &str) -> Result<String, String> {
    http_post_raw(host, port, "/sql", "text/plain", body, None)
}
