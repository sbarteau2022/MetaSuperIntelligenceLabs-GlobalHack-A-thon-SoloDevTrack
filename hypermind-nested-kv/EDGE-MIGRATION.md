# Edge Migration — HyperMind on Cloudflare (goal 57167b0b)

**Direction.** The laptop MCP server is the oracle. The edge is the product.
TS→Rust port path proven here becomes the template for everything behind it.

**Why Rust, not TS, on the edge.** The entire decision core (coordinates,
gates, traversal contracts, hunger priority) is already Rust with
`#![forbid(unsafe_code)]`. Porting to TS would mean re-proving what the
29 tests already prove. `workers-rs` compiles this Rust to Wasm and runs it
on every edge PoP. The MCP JSON-RPC boundary becomes an HTTP Worker route;
the math doesn't move.

## Target topology (each maps 1:1 to a Cloudflare primitive)

```
USER REQUEST (any PoP worldwide)
  │
  ▼
WORKER (Rust, workers-rs → Wasm)          ← hypermind-nested-kv/src/main.rs,
  │                                          handle_tool becomes fetch routes
  │  map.query / map.lexical / map.attend
  │  map.traverse / map.hunger / map.consolidate
  ├──── VECTORIZE (atom embeddings)       ← 256d tf-idf today → learned 384d
  │         H³×T² coords as metadata,       (Workers AI encoder, GPU-side)
  │         cosine top-k, then geo re-rank
  │         in-Worker (shell_dist, f64)
  │
  ├──── D1 (SQLite)                       ← hunger ledger (schema mirrors
  │         hunger table = TAP-N-TRACE      TAP-N-TRACE.md §4 1:1) + receipts
  │         §4 SQL verbatim; receipts       + snapshot digests. D1 IS SQLite:
  │         table alongside.                zero shape change from in-server.
  │
  ├──── WORKERS AI (GPU encoder)          ← embed(text) -> unit [f64;3].
  │         @cf/baai/bge-base-en-v1.5       GPU-ENCODER-CLEARANCE.md
  │         or fine-tuned MiniLM.           acceptance bar unchanged.
  │
  ├──── R2 (snapshots)                    ← content-addressed .bin snapshots
  │         map.snapshot.bin +               + sha256 sidecars. R2 = cold
  │         .sha256, versioned.              archive; D1 = hot ledger.
  │
  └──── KV (hot cache)                    ← VP-tree shortlist cache, IDF table
            IDF table, reweight version,     (10,515 terms), reweight version.
            hunger top-k materialization.
```

## TS→Rust port path (the template this proves)

1. **TS is the harness, Rust is the decision.** TS (or Python today) drives
   tool calls, renders results, manages UX. Rust owns every number that
   matters: scores, gates, priorities, traversal contracts, fingerprints.
2. **The boundary is typed once.** `wrangler types` generates `Env` from
   `wrangler.jsonc` — never hand-written. The MCP tool schemas
   (`tools_list`) become Worker routes with the same input/output shapes.
3. **No re-proof.** The 29 Rust tests compile to the same logic under
   `wasm32-unknown-unknown`. `cargo test` green on laptop = green on edge
   for all pure functions (score, shell_dist, judge, priority, fusion).
   Only binding calls (D1/Vectorize/AI) need `vitest-pool-workers` coverage.
4. **Receipts cross the boundary.** Every edge response carries the same
   `receipt_sha256` chain. An edge answer and a laptop answer for the same
   query are byte-comparable — divergence is a bug, not a version.

## Migration order (dependency-ordered, each independently shippable)

1. **Worker shell** (this goal): Rust Worker exposing the read paths
   (`map.query`, `map.lexical`, `map.health`, `map.hunger`) over HTTP,
   backed by a seeded D1 + Vectorize from the live 2,074-atom export.
   Writes (`ingest`, `consolidate`) stay laptop-side until step 3.
2. **D1 hunger ledger**: `wrangler d1 execute` with the TAP-N-TRACE §4
   schema verbatim. Taps write from the Worker; `map.hunger` reads ranked.
3. **Write path**: `ingest` (judge-gated) + `consolidate` move to the
   Worker with D1 transactions. Laptop becomes a replica, not the primary.
4. **GPU encoder**: Workers AI binding replaces `embed_direction_idf`.
   Acceptance: geo-alone probe audit ≥ 6/12 (the clearance bar).
5. **R2 snapshots + KV cache**: cold archive + hot shortlist/IDF/top-k.

## What stays laptop-side forever

- The O(N) ceiling harness (`distill_ceiling.py`) — edge proves scale,
  laptop proves correctness.
- Held-out A39 runs (judged, receipted, then uploaded as evidence).
- The oracle binary itself: any edge answer must match the laptop oracle
  on the same snapshot, or the edge is wrong.

## Cost note (grant-relevant)

Reads (query/lexical/hunger) are the hot path: Vectorize + D1 reads +
Worker CPU — all within free-tier headroom at hackathon scale. Writes
(ingest/consolidate/reweight) are batched and infrequent. GPU encoder
(Workers AI) is the only metered hot-path call — batch embeds at ingest,
never at query (query embeds are the same call, cached in KV by text hash).
