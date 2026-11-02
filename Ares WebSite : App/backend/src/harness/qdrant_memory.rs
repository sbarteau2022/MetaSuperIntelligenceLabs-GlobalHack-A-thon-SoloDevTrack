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
//! - Encoding: memory text is encoded by the **toroidal positional
//!   encoder (TPE)** — the real kernel set (`crate::harness::tpe`): a
//!   coupled 2:1 resonant T² phase carrier (`TpeState`) whose positions
//!   advance per token, a radius law `r(w)`, and the rank-5 trig kernel
//!   `k(d) = (1 + cos d)²` with its O(1) recurrent linear-attention
//!   readout (`Rank5State`). Position is part of the vector: the same
//!   text at different positions encodes differently. The TPE is a
//!   positional encoder over pseudo-hidden word vectors — NOT a neural
//!   embedder and NOT a hyperbolic KV; its measured e2e verdict on
//!   record is 3/4 PASS, 1 MISS (seq-1 denominator pin, frozen 1e-9
//!   floor), cited in `harness::tpe` and never hidden. Payload filters
//!   (session/thread/document ids) remain exact.
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
/// Legacy hashed-encoder collection. It is retained for historical recall
/// only; no new write or TPE query may enter this collection.
pub const LEGACY_MEMORY_COLLECTION: &str = "session_memory_v1";
/// TPE-encoded memory collection. The encoder name and vector contract live
/// here so startup can refuse to create or inspect it with silently
/// incompatible settings.
pub const MEMORY_COLLECTION: &str = "session_memory_tpe_v1";
/// Vector width scored/stored in Qdrant. Under the TPE this is the
/// retrieval width; the familiar 384 was inherited from the makeshift
/// hashed encoder and intentionally not preserved.
pub const EMBED_DIM: usize = 384;
pub const DEFAULT_CONTEXT_LIMIT: usize = 12;

#[derive(Debug, Clone)]
pub struct QdrantMemory {
    base: String,
    http: reqwest::Client,
    collection: String,
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

/// Encode text with the toroidal positional encoder (TPE) into the
/// memory vector of width `EMBED_DIM`. Kernel path: per word-position,
/// `TpeState::evolve(0, 0, arc=1, 0)` advances the 2:1 resonant carrier;
/// the word's pseudo-hidden vector feeds `Rank5State::step_and_readout`
/// (decay 1.0); the readout is scaled by `radius_for_rotations` and
/// woven with the phase fingerprint (theta_f, theta_s, psi, amplitude,
/// radius, phi) so cosine is position-sensitive. Replaces the former
/// hashed n-gram encoder (removed) — the TPE is the positional encoder,
/// not a makeshift hash and never presented as a neural embedder.
pub fn embed_text(text: &str) -> Vec<f32> {
    crate::harness::tpe::embed_tpe(text, EMBED_DIM)
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

    /// Ensure the governed memory collection exists. Created idempotently.
    pub async fn ensure_collection(&self) -> Result<bool, String> {
        if self.collection_exists(&self.collection).await? {
            return Ok(false);
        }
        let body = json!({
            "vectors": {
                "memory_vector": { "size": EMBED_DIM, "distance": "Cosine" }
            },
            "on_disk_payload": true
        });
        let _ = self
            .req_json(
                reqwest::Method::PUT,
                &format!("/collections/{}", self.collection),
                Some(body),
            )
            .await?;
        // Payload filters need indexes for exact tail/filter queries.
        for (field, schema) in [
            ("kind", "keyword"),
            ("session_id", "keyword"),
            ("thread_id", "keyword"),
            ("mission_id", "keyword"),
            ("created_at_ms", "integer"),
        ] {
            let _ = self
                .req_json(
                    reqwest::Method::PUT,
                    &format!("/collections/{}/index", self.collection),
                    Some(json!({ "field_name": field, "field_schema": schema })),
                )
                .await;
        }
        Ok(true)
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
        let ms = now_ms();
        let session_key = session_id.unwrap_or("global");
        let id = memory_point_id(kind, session_key, ms, text);
        let hash = content_hash(text);
        let vector = embed_text(text);
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
        payload.insert("encoder".into(), json!("tpe-toroidal-rank5-v1"));
        let body = json!({
            "points": [{
                "id": id,
                "vector": { "memory_vector": vector },
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
            Value::Null
        } else {
            json!({ "must": must })
        };
        let body = json!({
            "query": embed_text(query),
            "using": "memory_vector",
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
        let filter = if must.is_empty() {
            Value::Null
        } else {
            json!({ "must": must })
        };
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
    /// memories, deduped by content hash and text. Every item keeps its
    /// id + provenance so a capability can show where context came from.
    pub async fn load_session_context(
        &self,
        query: Option<&str>,
        thread_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<SessionContext, String> {
        let bounded = limit.min(DEFAULT_CONTEXT_LIMIT.max(1));
        let mut ordered: Vec<MemoryEntry> = Vec::new();
        let mut seen_hashes = std::collections::HashSet::new();

        // 1) recency tail: newest session summaries/memories first
        let tail = self
            .tail_recent(bounded, thread_id, session_id)
            .await
            .unwrap_or_default();
        for item in tail {
            if seen_hashes.insert(item.content_hash.clone()) {
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
                    if seen_hashes.insert(item.content_hash.clone()) {
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
