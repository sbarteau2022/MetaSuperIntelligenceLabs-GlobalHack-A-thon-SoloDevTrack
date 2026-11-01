# Design — Step 4 indexed retrieval (2026-11-01)

Objective: candidate lookup through SurrealDB, exact Rust H³×T² rerank,
recall benchmark vs exhaustive oracle.

Capability finding (probed, dictates the design): this SurrealDB 3.3.5 build
parses plain indexes but REJECTS `DEFINE INDEX ... SEARCH` and `... MTREE`
(parse errors captured). `CONTAINS`/`CONTAINSANY` work as full-scan
predicates. So: DB = lexical prefilter (bounded CONTAINSANY, LIMIT ≤16k),
Rust = all ranking. No server-side vector/FTS index is claimed; a future
capable build can accelerate the prefilter without changing the contract.

Changes (all in hypermind-nested-kv, schema untouched):
- `surreal.rs`: `candidates(terms, limit) -> Vec<u64>`; `counts()` now
  per-statement tolerant (fresh DB without tables reports 0, not error).
- `main.rs`: `rank_candidates()` extracted from map.query (shared rank —
  VP path and SDB path rank identically by construction); map.query rewired
  to it (ranking-neutral: full suite green); 3 new tools:
  `map.db_candidates`, `map.query_sdb` (prefilter→hydrate→rank_candidates),
  `map.recall_bench` (oracle + VP arm + SDB arm, recall@k + ms).
