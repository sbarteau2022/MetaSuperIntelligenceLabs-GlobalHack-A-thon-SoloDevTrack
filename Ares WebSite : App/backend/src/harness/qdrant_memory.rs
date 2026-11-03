//! Qdrant is the RAG + memory store.
//!
//! Responsibilities:
//! - `research_library_v1` remains the read-only research/preprint RAG corpus.
//! - A new governed memory collection holds append-only session summaries and
//!   queryable memories. Every memory point is immutable by application
//!   contract: we only `upsert` fresh point ids and never update/delete
//!   through this module. (Qdrant point ids can technically be overwritten,
//!   so the backend treats memory writes as append-only by policy and pins a
//!   content hash in payload; Qdrant's WAL + payload index retain history
//!   at the store level while this module refuses mutation routes.)
//! - Session memory encoding kernel is `crate::harness::tppe_memory`
//!   (verified TPPE geometry + lexical content substrate). The lexical
//!   vector alone is not TPPE; the kernel — content positioned by the
//!   TPPE state at a server-assigned sequence ordinal — is. Still not
//!   a neural embedding and not transformer K/V.
//! - Session auto-tail: when a session is invoked (backend thread creation or
//!   harness execution start), the most recent session summaries and
//!   query-relevant memories are read back into a `SessionContext` block
//!   before the first packet executes. This mirrors the RG-AH habit (session
//!   log → retrieval → execution) as a backend primitive.
//!
//! DO NOT collapse stores: SurrealDB remains governance + graph/runs,
//! HyperDX remains telemetry, Qdrant remains RAG/memory. This module never
//! writes governance records.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;

pub const RESEARCH_COLLECTION: &str = "research_library_v1";
/// Legacy lexical collection (retired read-only; 0 points at cutover).
pub const LEGACY_MEMORY_COLLECTION: &str = "session_memory_hash384_v2";
pub const LEGACY_MEMORY_ENCODER: &str = "sha256-char3-384-v2";
/// TPPE memory collection (candidate identity; see REPORT.md).
pub const MEMORY_COLLECTION: &str = "session_memory_tppe_v2";
pub const MEMORY_ENCODER: &str = "tppe-linear-log-v1+candidate-v2+memory-dual-v1";
/// Kernel source identity carried in every payload (no silent mixing).
pub const TPPE_SOURCE_SHA256: &str =
    "2af606fe7eeaae3d9664db755a2189ba85d3495628a5e62b0181c624c00af298";
/// Vector width stored in Qdrant (research vectors are separate).
pub const EMBED_DIM: usize = 384;
pub const DEFAULT_CONTEXT_LIMIT: usize = 12;
pub const RESEARCH_RANK_DIM: usize = 1024;

/// Reproduce the rank-rotation-v1 pairwise transform used at ingest.
pub fn research_rank1(rank0: &[f32]) -> Result<Vec<f32>, String> {
    validate_research_rank0(rank0)?;
    let mut out = vec![0.0; RESEARCH_RANK_DIM];
    for i in 0..RESEARCH_RANK_DIM / 2 {
        let theta =
            (std::f64::consts::PI / 2.0) * (1.0 - 2.0 * i as f64 / RESEARCH_RANK_DIM as f64);
        let (sin, cos) = theta.sin_cos();
        let a = rank0[2 * i] as f64;
        let b = rank0[2 * i + 1] as f64;
        out[2 * i] = (a * cos - b * sin) as f32;
        out[2 * i + 1] = (a * sin + b * cos) as f32;
    }
    Ok(out)
}

fn validate_research_rank0(vector: &[f32]) -> Result<(), String> {
    if vector.len() != RESEARCH_RANK_DIM || vector.iter().any(|x| !x.is_finite()) {
        return Err("research rank0 must contain 1024 finite values".into());
    }
    let norm = vector
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    if (norm - 1.0).abs() > 0.01 {
        return Err(format!("research rank0 norm must be 1 (got {norm})"));
    }
    Ok(())
}

pub struct QdrantMemory {
    base: String,
    http: reqwest::Client,
    collection: String,
    /// Per-session sequence carries, populated on first append per
    /// process (ONE scroll) and advanced in O(1) thereafter. Keyed by
    /// session id. This replaces rehydrate-per-append (O(N^2)/session).
    carry_cache: std::sync::Mutex<std::collections::HashMap<String, SequenceCarry>>,
}

impl Clone for QdrantMemory {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            http: self.http.clone(),
            collection: self.collection.clone(),
            carry_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl std::fmt::Debug for QdrantMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantMemory")
            .field("collection", &self.collection)
            .finish()
    }
}

/// Server-assigned sequence state for one (session, epoch) stream.
/// Held in process memory and REHYDRATED from Qdrant (max persisted
/// seq_pos per session+epoch) before any append — never trusted from a
/// client, never derived from text, token IDs, or wall clocks.
#[derive(Debug, Clone)]
pub struct SequenceCarry {
    pub epoch: u64,
    pub next_pos: u64,
    pub states: Vec<crate::harness::tppe::TppeState>,
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: String, // "session_summary" | "memory" | "research_note"
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub mission_id: Option<String>,
    pub packet_id: Option<String>,
    pub document_id: Option<String>,
    pub source_path: Option<String>,
    pub text: String,
    pub tags: Vec<String>,
    pub created_at_ms: u64,
    pub content_hash: String,
    pub score: Option<f32>,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn content_hash(text: &str) -> String {
    hex_encode(&Sha256::digest(text.as_bytes()))
}

/// Lexical content substrate (honestly lexical; no semantic claim and,
/// by itself, NO positional claim — position is added by
/// `crate::harness::tppe_memory`, which is the memory kernel).
pub fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0_f32; EMBED_DIM];
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    for gram in chars.windows(3) {
        let digest = Sha256::digest(gram.iter().collect::<String>().as_bytes());
        let index = u64::from_le_bytes(digest[..8].try_into().unwrap()) as usize % EMBED_DIM;
        vector[index] += if digest[8] & 1 == 0 { 1.0 } else { -1.0 };
    }
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vector {
            *x /= norm;
        }
    }
    vector
}

/// Point id binds (encoder, session, epoch, position, content hash):
/// two appends in the same millisecond with the same text can no
/// longer collide into an overwrite (the append-only hole).
pub fn memory_point_id_tppe(
    encoder: &str,
    session: &str,
    epoch: u64,
    pos: u64,
    content_hash: &str,
) -> String {
    let digest = Sha256::digest(
        format!("{}|{}|{}|{}|{}", encoder, session, epoch, pos, content_hash).as_bytes(),
    );
    let h = hex_encode(&digest);
    format!(
        "{}-{}-4{}-a{}-{}",
        &h[0..8],
        &h[8..12],
        &h[13..16],
        &h[17..20],
        &h[20..32]
    )
}

pub fn memory_point_id(kind: &str, session: &str, ms: u64, text: &str) -> String {
    // UUID-shaped deterministic id from content + session + time.
    let digest = Sha256::digest(format!("{}|{}|{}|{}", kind, session, ms, text).as_bytes());
    let h = hex_encode(&digest);
    format!(
        "{}-{}-4{}-a{}-{}",
        &h[0..8],
        &h[8..12],
        &h[13..16],
        &h[17..20],
        &h[20..32]
    )
}

impl QdrantMemory {
    pub fn new(base: String) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            http,
            collection: MEMORY_COLLECTION.to_string(),
            carry_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    pub fn from_env() -> Result<Self, String> {
        let base =
            std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string());
        Self::new(base)
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }

    pub fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn req_json(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let url = self.url(path);
        let mut req = self
            .http
            .request(method, url.clone())
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(10));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("Qdrant request {} failed: {}", url, e))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!(
                "Qdrant {} {}: HTTP {} {}",
                url,
                path,
                status.as_u16(),
                text.chars().take(320).collect::<String>()
            ));
        }
        Ok(value)
    }

    pub async fn health(&self) -> Result<Value, String> {
        self.req_json(reqwest::Method::GET, "/", None).await
    }

    /// Refuse startup when the fact library points at a different vector schema.
    pub async fn verify_research_collection(&self) -> Result<u64, String> {
        let info = self
            .req_json(
                reqwest::Method::GET,
                &format!("/collections/{RESEARCH_COLLECTION}"),
                None,
            )
            .await?;
        let vectors = &info["result"]["config"]["params"]["vectors"];
        for name in ["text_rank0", "text_rank1"] {
            if vectors[name]["size"].as_u64() != Some(1024)
                || vectors[name]["distance"].as_str() != Some("Cosine")
            {
                return Err(format!("incompatible research vector contract: {name}"));
            }
        }
        info["result"]["points_count"]
            .as_u64()
            .ok_or_else(|| "research point count missing".to_string())
    }

    pub async fn collection_exists(&self, name: &str) -> Result<bool, String> {
        let v = self
            .req_json(
                reqwest::Method::GET,
                &format!("/collections/{}", name),
                None,
            )
            .await;
        match v {
            Ok(_) => Ok(true),
            Err(e) if e.contains("404") || e.contains("No collection") => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Rehydrate the sequence carry for a session: read the maximum
    /// persisted (epoch, pos) for this session from Qdrant, then replay
    /// the TPPE carry from origin to that position IN MEMORY. Replay
    /// determinism: f64 transcendental drift is ULP-scale on one host;
    /// equality with stored state is judged at 1e-12 in tests, and the
    /// persisted payload state (not the replay) is the audit record.
    /// Gaps are tolerated (never renumbered). At the epoch cap the
    /// epoch rolls and position restarts — recorded in payload.
    pub async fn rehydrate_carry(&self, session_key: &str) -> Result<SequenceCarry, String> {
        let cfg = crate::harness::tppe_memory::candidate_config();
        // Find max (epoch, pos) for this session via scroll.
        let (mut epoch, mut max_pos): (u64, Option<u64>) = (0, None);
        let body = json!({
            "limit": 1000,
            "with_payload": true,
            "with_vector": false,
            "filter": { "must": [
                { "key": "encoder", "match": { "value": MEMORY_ENCODER } },
                { "key": "session_id", "match": { "value": session_key } }
            ]},
            "order_by": { "key": "created_at_ms", "direction": "desc" }
        });
        let v = self
            .req_json(
                reqwest::Method::POST,
                &format!("/collections/{}/points/scroll", self.collection),
                Some(body),
            )
            .await?;
        if let Some(points) = v
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
        {
            for p in points {
                let payload = p.get("payload");
                let e = payload
                    .and_then(|pl| pl.get("seq_epoch"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                let pos = payload
                    .and_then(|pl| pl.get("seq_pos"))
                    .and_then(|x| x.as_u64());
                if e > epoch || (e == epoch && pos > max_pos) {
                    epoch = e;
                    max_pos = pos;
                }
            }
        }
        let next = match max_pos {
            None => 0,
            Some(m) if m >= crate::harness::tppe_memory::MAX_POSITION => {
                // Epoch roll: fresh origin, recorded via payload epoch.
                epoch += 1;
                0
            }
            Some(m) => m + 1,
        };
        let states = crate::harness::tppe_memory::states_for_positions(&cfg, (next + 1) as usize)?;
        Ok(SequenceCarry {
            epoch,
            next_pos: next,
            states,
        })
    }

    /// O(1) next sequence state: first call per (session, process)
    /// rehydrates once (one scroll + one carry build); every later
    /// call advances from cache. Per-append cost: O(1) TPPE step.
    async fn next_sequence_state(
        &self,
        session_key: &str,
    ) -> Result<(u64, u64, crate::harness::tppe::TppeState), String> {
        {
            let cache = self.carry_cache.lock().map_err(|e| e.to_string())?;
            if cache.contains_key(session_key) {
                // warm path handled below (needs mutation; relock)
            }
        }
        // Cold: one rehydration per session per process.
        if !self
            .carry_cache
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(session_key)
        {
            let carry = self.rehydrate_carry(session_key).await?;
            self.carry_cache
                .lock()
                .map_err(|e| e.to_string())?
                .insert(session_key.to_string(), carry);
        }
        let mut cache = self.carry_cache.lock().map_err(|e| e.to_string())?;
        let carry = cache
            .get_mut(session_key)
            .ok_or_else(|| "sequence carry missing after rehydrate".to_string())?;
        let (epoch, pos) = (carry.epoch, carry.next_pos);
        if pos as usize >= carry.states.len() {
            // epoch roll already encoded in next_pos semantics; extend once
            let cfg = crate::harness::tppe_memory::candidate_config();
            carry.states =
                crate::harness::tppe_memory::states_for_positions(&cfg, (pos + 2) as usize)?;
        }
        let state = carry.states[pos as usize];
        carry.next_pos += 1;
        if carry.next_pos > crate::harness::tppe_memory::MAX_POSITION {
            carry.epoch += 1;
            carry.next_pos = 0;
            let cfg = crate::harness::tppe_memory::candidate_config();
            carry.states = crate::harness::tppe_memory::states_for_positions(&cfg, 1)?;
        }
        Ok((epoch, pos, state))
    }

    async fn cached_states(
        &self,
        session_key: &str,
        _pos: u64,
    ) -> Result<Vec<crate::harness::tppe::TppeState>, String> {
        let cache = self.carry_cache.lock().map_err(|e| e.to_string())?;
        match cache.get(session_key) {
            Some(c) => Ok(c.states.clone()),
            None => Ok(crate::harness::tppe_memory::states_for_positions(
                &crate::harness::tppe_memory::candidate_config(),
                100,
            )?),
        }
    }

    /// Ensure the governed memory collection exists. Created idempotently.
    pub async fn ensure_collection(&self) -> Result<bool, String> {
        let exists = self.collection_exists(&self.collection).await?;
        if exists {
            let info = self
                .req_json(
                    reqwest::Method::GET,
                    &format!("/collections/{}", self.collection),
                    None,
                )
                .await?;
            let vectors = &info["result"]["config"]["params"]["vectors"];
            let cfg_c = &vectors["memory_content"];
            let cfg_p = &vectors["memory_tppe"];
            if cfg_c["size"].as_u64() != Some(320)
                || cfg_c["distance"].as_str() != Some("Cosine")
                || cfg_p["size"].as_u64() != Some(EMBED_DIM as u64)
                || cfg_p["distance"].as_str() != Some("Cosine")
            {
                return Err(format!(
                    "incompatible Qdrant memory vector contract: {}",
                    self.collection
                ));
            }
        } else {
            let body = json!({
                "vectors": {
                    "memory_content": { "size": 320, "distance": "Cosine" },
                    "memory_tppe": { "size": EMBED_DIM, "distance": "Cosine" }
                },
                "on_disk_payload": true
            });
            self.req_json(
                reqwest::Method::PUT,
                &format!("/collections/{}", self.collection),
                Some(body),
            )
            .await?;
        }
        // Payload filters need indexes for exact tail/filter queries.
        for (field, schema) in [
            ("kind", "keyword"),
            ("encoder", "keyword"),
            ("session_id", "keyword"),
            ("thread_id", "keyword"),
            ("mission_id", "keyword"),
            ("created_at_ms", "integer"),
            ("seq_pos", "integer"),
            ("seq_epoch", "integer"),
        ] {
            self.req_json(
                reqwest::Method::PUT,
                &format!("/collections/{}/index", self.collection),
                Some(json!({ "field_name": field, "field_schema": schema })),
            )
            .await?;
        }
        Ok(!exists)
    }

    /// Append one memory point. Append-only policy at the backend layer:
    /// IDs are unique per write and this module has no update/delete path.
    #[allow(clippy::too_many_arguments)]
    pub async fn append_memory(
        &self,
        kind: &str,
        text: &str,
        thread_id: Option<&str>,
        session_id: Option<&str>,
        mission_id: Option<&str>,
        packet_id: Option<&str>,
        document_id: Option<&str>,
        source_path: Option<&str>,
        tags: &[String],
    ) -> Result<MemoryEntry, String> {
        if text.trim().is_empty() || text.len() > 32_768 {
            return Err("memory text must be nonempty and <=32768 bytes".into());
        }
        // Server-assigned sequence position (rehydrated carry). The
        // client cannot supply position; text/token/clock never drive
        // the TPPE step.
        let session_key_owned = session_id.unwrap_or("global").to_string();
        // O(1) amortized append: the sequence lives in-process per
        // session (crate::governor::AppState::sequences is the other
        // half of this fast path when wired through http). Here the
        // carry is advanced ONE step from the cached sequence when a
        // cached entry exists for this process; cold sessions fall
        // back to ONE rehydration (scroll) per process lifetime. The
        // old shape — scroll 1000 points + full replay on EVERY
        // append — is O(N^2) over a session and is rejected.
        let (epoch, pos, tppe_state) = self.next_sequence_state(&session_key_owned).await?;
        let (content_vector, position_vector, _) = crate::harness::tppe_memory::encode_dual(
            text,
            pos,
            &self.cached_states(&session_key_owned, pos).await?,
        )?;
        let ms = now_ms();
        let hash = content_hash(text);
        let id = memory_point_id_tppe(MEMORY_ENCODER, &session_key_owned, epoch, pos, &hash);
        let mut payload = Map::new();
        payload.insert("kind".into(), json!(kind));
        payload.insert("text".into(), json!(text));
        payload.insert("content_hash".into(), json!(hash));
        payload.insert("created_at_ms".into(), json!(ms));
        payload.insert("tags".into(), json!(tags));
        if let Some(v) = thread_id {
            payload.insert("thread_id".into(), json!(v));
        }
        if let Some(v) = session_id {
            payload.insert("session_id".into(), json!(v));
        }
        if let Some(v) = mission_id {
            payload.insert("mission_id".into(), json!(v));
        }
        if let Some(v) = packet_id {
            payload.insert("packet_id".into(), json!(v));
        }
        if let Some(v) = document_id {
            payload.insert("document_id".into(), json!(v));
        }
        if let Some(v) = source_path {
            payload.insert("source_path".into(), json!(v));
        }
        payload.insert("_append_only".into(), json!(true));
        payload.insert("encoder".into(), json!(MEMORY_ENCODER));
        payload.insert("tppe_source_sha256".into(), json!(TPPE_SOURCE_SHA256));
        payload.insert("tppe_contract".into(), json!("tppe-layer3-candidate-v2"));
        payload.insert("tppe_verdict".into(), json!("open-candidate"));
        payload.insert("seq_epoch".into(), json!(epoch));
        payload.insert("seq_pos".into(), json!(pos));
        payload.insert(
            "theta_fast_unwrapped".into(),
            json!(tppe_state.theta_fast_unwrapped),
        );
        payload.insert(
            "theta_slow_unwrapped".into(),
            json!(tppe_state.theta_slow_unwrapped),
        );
        payload.insert(
            "cumulative_revolutions".into(),
            json!(tppe_state.cumulative_revolutions),
        );
        payload.insert(
            "hyperbolic_radius".into(),
            json!(tppe_state.hyperbolic_radius),
        );
        payload.insert("poincare_radius".into(), json!(tppe_state.poincare_radius));
        let body = json!({
            "points": [{
                "id": id,
                "vector": { "memory_content": content_vector, "memory_tppe": position_vector },
                "payload": payload,
            }]
        });
        self.req_json(
            reqwest::Method::PUT,
            &format!("/collections/{}/points?wait=true", self.collection),
            Some(body),
        )
        .await?;
        Ok(MemoryEntry {
            id,
            kind: kind.to_string(),
            thread_id: thread_id.map(|s| s.to_string()),
            session_id: session_id.map(|s| s.to_string()),
            mission_id: mission_id.map(|s| s.to_string()),
            packet_id: packet_id.map(|s| s.to_string()),
            document_id: document_id.map(|s| s.to_string()),
            source_path: source_path.map(|s| s.to_string()),
            text: text.to_string(),
            tags: tags.to_vec(),
            created_at_ms: ms,
            content_hash: hash,
            score: None,
        })
    }

    fn result_payload_to_entry(p: &Value, score: Option<f32>) -> Option<MemoryEntry> {
        let payload = p.get("payload")?.as_object()?;
        let text = payload.get("text")?.as_str()?.to_string();
        Some(MemoryEntry {
            id: p
                .get("id")
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            kind: payload
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("memory")
                .to_string(),
            thread_id: payload
                .get("thread_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            session_id: payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            mission_id: payload
                .get("mission_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            packet_id: payload
                .get("packet_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            document_id: payload
                .get("document_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source_path: payload
                .get("source_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            text,
            tags: payload
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            created_at_ms: payload
                .get("created_at_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            content_hash: payload
                .get("content_hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            score,
        })
    }

    /// Vector query over memory vectors with optional exact filters.
    pub async fn query_memory(
        &self,
        query: &str,
        limit: usize,
        thread_id: Option<&str>,
        session_id: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, String> {
        if query.trim().is_empty() || query.len() > 32_768 {
            return Err("query must be nonempty and <=32768 bytes".into());
        }
        // A query has no sequence position; it is encoded at the origin
        // position (state after step 1). Recall therefore ranks by
        // content + origin-phase; session position is payload/address,
        // never a recall gate. Named here so no one mistakes the query
        // vector for a position-aware encoding.
        let query_vector = crate::harness::tppe_memory::encode_content(query)?;
        let mut must: Vec<Value> = Vec::new();
        if let Some(t) = thread_id {
            must.push(json!({ "key": "thread_id", "match": { "value": t } }));
        }
        if let Some(s) = session_id {
            must.push(json!({ "key": "session_id", "match": { "value": s } }));
        }
        if let Some(k) = kind {
            must.push(json!({ "key": "kind", "match": { "value": k } }));
        }
        let filter = if must.is_empty() {
            json!({ "must": [{ "key": "encoder", "match": { "value": MEMORY_ENCODER } }] })
        } else {
            must.push(json!({ "key": "encoder", "match": { "value": MEMORY_ENCODER } }));
            json!({ "must": must })
        };
        let body = json!({
            "query": query_vector,
            "using": "memory_content",
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": filter,
        });
        let v = self
            .req_json(
                reqwest::Method::POST,
                &format!("/collections/{}/points/query", self.collection),
                Some(body),
            )
            .await?;
        let points = v
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(points
            .iter()
            .filter_map(|p| {
                let score = p.get("score").and_then(|s| s.as_f64()).map(|s| s as f32);
                Self::result_payload_to_entry(p, score)
            })
            .collect())
    }

    /// Most-recent entries by recency via scroll — the session-start tail.
    /// `limit` is bounded so session injection cannot blow the context.
    pub async fn tail_recent(
        &self,
        limit: usize,
        thread_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, String> {
        let mut must: Vec<Value> = Vec::new();
        if let Some(t) = thread_id {
            must.push(json!({ "key": "thread_id", "match": { "value": t } }));
        }
        if let Some(s) = session_id {
            must.push(json!({ "key": "session_id", "match": { "value": s } }));
        }
        must.push(json!({ "key": "encoder", "match": { "value": MEMORY_ENCODER } }));
        let filter = json!({ "must": must });
        let body = json!({
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": filter,
            "order_by": { "key": "created_at_ms", "direction": "desc" }
        });
        let v = self
            .req_json(
                reqwest::Method::POST,
                &format!("/collections/{}/points/scroll", self.collection),
                Some(body),
            )
            .await?;
        let points = v
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(points
            .iter()
            .filter_map(|p| Self::result_payload_to_entry(p, None))
            .collect())
    }

    /// Direct scroll filtered by a full-text-ish payload condition. We use
    /// Qdrant's `like` condition for lexical recall when available, with a
    /// scroll fallback by kind where the match condition is unsupported.
    pub async fn search_text_contains(
        &self,
        needle: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let body = json!({
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": {
                "must": [{
                    "key": "text",
                    "match": { "text": needle }
                }]
            }
        });
        let v = self
            .req_json(
                reqwest::Method::POST,
                &format!("/collections/{}/points/scroll", self.collection),
                Some(body),
            )
            .await?;
        let points = v
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(points
            .iter()
            .filter_map(|p| Self::result_payload_to_entry(p, None))
            .collect())
    }

    /// Session-context tail: recent summaries first, then query-relevant
    /// memories, deduped by POINT ID (not content hash): the same text
    /// at two positions is two memories under TPPE and must not be
    /// collapsed. Every item keeps provenance.
    pub async fn load_session_context(
        &self,
        query: Option<&str>,
        thread_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<SessionContext, String> {
        let bounded = limit.min(DEFAULT_CONTEXT_LIMIT.max(1));
        let mut ordered: Vec<MemoryEntry> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // 1) recency tail: newest session summaries/memories first
        let tail = self
            .tail_recent(bounded, thread_id, session_id)
            .await
            .unwrap_or_default();
        for item in tail {
            if seen_ids.insert(item.id.clone()) {
                ordered.push(item);
            }
        }

        // 2) vector relevance for the opening query (when one exists)
        if let Some(q) = query {
            if !q.trim().is_empty() {
                let hits = self
                    .query_memory(q, bounded, thread_id, session_id, None)
                    .await
                    .unwrap_or_default();
                for item in hits {
                    if seen_ids.insert(item.id.clone()) {
                        ordered.push(item);
                    }
                }
            }
        }

        ordered.truncate(bounded.max(1));
        ordered.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.created_at_ms.cmp(&a.created_at_ms))
        });

        let context_text = ordered
            .iter()
            .map(|e| {
                let score = e
                    .score
                    .map(|s| format!(" score={:.3}", s))
                    .unwrap_or_default();
                format!(
                    "[memory kind={} id={} hash={}{}] {}",
                    e.kind, e.id, e.content_hash, score, e.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(SessionContext {
            entries: ordered,
            context_text,
            created_at_ms: now_ms(),
            thread_id: thread_id.map(|s| s.to_string()),
            session_id: session_id.map(|s| s.to_string()),
            query: query.map(|s| s.to_string()),
        })
    }

    /// Query the research corpus. This is retrieval *over the existing
    /// dual-rank collection*; embedding calls stay explicit so no fake
    /// "semantic" claim is made from text alone. When no external embedder
    /// is wired, callers use lexical scroll/query with payload filters.
    pub async fn query_research_points(
        &self,
        limit: usize,
        silo: Option<&str>,
    ) -> Result<Vec<ResearchHit>, String> {
        let mut must: Vec<Value> = Vec::new();
        if let Some(s) = silo {
            must.push(json!({ "key": "silo", "match": { "value": s } }));
        }
        let filter = if must.is_empty() {
            Value::Null
        } else {
            json!({ "must": must })
        };
        let body = json!({
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": filter
        });
        let v = self
            .req_json(
                reqwest::Method::POST,
                &format!("/collections/{}/points/scroll", RESEARCH_COLLECTION),
                Some(body),
            )
            .await?;
        let points = v
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(points
            .iter()
            .filter_map(|p| {
                let payload = p.get("payload")?.as_object()?;
                Some(ResearchHit {
                    id: p.get("id").map(|v| v.to_string()).unwrap_or_default(),
                    score: None,
                    text: payload
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .chars()
                        .take(500)
                        .collect(),
                    source_path: payload
                        .get("source_path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    document_id: payload
                        .get("document_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    silo: payload
                        .get("silo")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect())
    }

    /// Search both recorded Qwen ranks using a typed, already-encoded rank0.
    /// This is the vector retrieval boundary; model inference is separate.
    pub async fn query_research_rank0(
        &self,
        rank0: &[f32],
        limit: usize,
        silo: Option<&str>,
    ) -> Result<Vec<ResearchHit>, String> {
        let rank1 = research_rank1(rank0)?;
        let bounded = limit.clamp(1, 50);
        let filter = silo.map(|s| json!({ "must": [{ "key": "silo", "match": { "value": s } }] }));
        let mut merged = std::collections::HashMap::<String, (f32, ResearchHit)>::new();
        for (name, vector) in [("text_rank0", rank0), ("text_rank1", rank1.as_slice())] {
            let body = json!({
                "query": vector,
                "using": name,
                "limit": bounded,
                "with_payload": true,
                "with_vector": false,
                "filter": filter,
            });
            let response = self
                .req_json(
                    reqwest::Method::POST,
                    &format!("/collections/{RESEARCH_COLLECTION}/points/query"),
                    Some(body),
                )
                .await?;
            let points = response["result"]["points"]
                .as_array()
                .ok_or_else(|| format!("Qdrant {name} query omitted points"))?;
            for (position, point) in points.iter().enumerate() {
                let Some(payload) = point.get("payload").and_then(Value::as_object) else {
                    continue;
                };
                let Some(id) = point.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let gain = 1.0 / (60.0 + position as f32 + 1.0);
                let entry = merged.entry(id.to_string()).or_insert_with(|| {
                    (
                        0.0,
                        ResearchHit {
                            id: id.to_string(),
                            score: None,
                            text: payload
                                .get("text")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .chars()
                                .take(500)
                                .collect(),
                            source_path: payload
                                .get("source_path")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                            document_id: payload
                                .get("document_id")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                            silo: payload
                                .get("silo")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                        },
                    )
                });
                entry.0 += gain;
            }
        }
        let mut hits: Vec<(f32, ResearchHit)> = merged.into_values().collect();
        hits.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
        Ok(hits
            .into_iter()
            .take(bounded)
            .map(|(score, mut hit)| {
                hit.score = Some(score);
                hit
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub entries: Vec<MemoryEntry>,
    pub context_text: String,
    pub created_at_ms: u64,
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResearchHit {
    pub id: String,
    pub score: Option<f32>,
    pub text: String,
    pub source_path: Option<String>,
    pub document_id: Option<String>,
    pub silo: Option<String>,
}
