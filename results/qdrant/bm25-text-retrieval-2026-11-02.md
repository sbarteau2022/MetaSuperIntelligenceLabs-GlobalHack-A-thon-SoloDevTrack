# Qdrant fact-library text retrieval receipt — 2026-11-02

Scope: local Qdrant 1.20.0 `research_library_v1`, scored BM25 index on existing `text` payload; no point upserts/deletes, no vector changes. This is lexical retrieval. It does not claim Qwen query encoder parity or PAMI completion.

Frozen gates before the change: disposable 3-point collection query ranks the repeated-term document first; empty query refused by the Rust route; live point count unchanged; silo-filtered results carry source provenance; vector self-retrieval still passes. The disposable collection was deleted after the probe.

Observed: index creation completed in 2.126 s. Exact point count before and after: 48,422. Text index metadata reports 48,367 indexed points, 55 fewer than the collection count. A Qdrant `is_empty(text)` count returned zero, so missing/empty payload does not explain the discrepancy; index accounting or tokenization must be audited before a coverage claim. Live `hyperbolic memory` query with silo `specops` returned scored hits with source paths; `POST /api/rag/query` returned `query_mode=qdrant-bm25-text`, three hits, and HTTP 422 for blank text. Ten repeated loopback Qdrant requests after index creation took 40.545–57.877 ms end to end; median approximately 41.5 ms. This is one warm query on one local machine, not a p95 service-level claim.

Gates: `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` passed in the backend; explicit live `qdrant_research_live` had 2/2 pass. The backend was started locally for HTTP smoke and stopped cleanly. Official API contract: https://qdrant.tech/documentation/search/text-search/full-text-search/ .

Open: Rust-native Qwen3-VL-Embedding-2B-6bit text encoder matching the first-1024-final-token ingest recipe, versioned known-vector parity, dense text route, PAMI document-rank aggregation, and broader retrieval quality/latency evaluation. The BM25 lane is an operational fallback and future hybrid prefetch input, not a substitute for those gates.
