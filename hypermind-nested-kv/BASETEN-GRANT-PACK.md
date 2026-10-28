# Baseten Grant — Document-Heavy Pack (Goal 498fa956 a–c)

**Date.** 2026-10-28. **Live map.** `:8765`, 1,644 atoms, snapshot `bf42b7da`.
**Tests.** `cargo test -- --test-threads=1`: 24/24 green.

## What runs today (VERIFIED this session)

| Capability | Tool | Measured |
|---|---|---|
| Exact recall (matched salience) | `map.query` | 1.00 top-1 @ 100k atoms (ceiling harness) |
| Doc retrieval (paraphrase probe) | `map.lexical` (BM25 full-corpus) | 5/6 doc, 6/6 topic (6-probe spot check) |
| Semantic geo retrieval | `map.query` (IDF-weighted + BM25 fusion) | 0–2/12 doc (lexical carries; geo is recall support) |
| Traversal (symplectic envelope) | `map.traverse` | A39 `mlir_op#26` replays as fail-explained, not fail-silent |
| Judge gate | `map.ingest` mint-on-pass | structural + caller veto, every path receipted |
| Dedup | `map.ingest` coalescing | 12/12 rerun coalesced, salience +0.05, no swamp |
| `p_useful` (first measurement) | `a39_rerun.py` | 1.000 retrieved-useful, 12/12 task cover (see caveat) |
| Ingest | O(N) pass | ~9.9k atoms/s flat to 100k |

## Architecture (two-tier, as vault intended)

- **Geometric tier** (`query/attend/traverse`): exact recall, depth gate, form+energy+frame traversal checks. O(log N) queries, O(N) attend via VP shortlist.
- **Lexical tier** (`map.lexical`): full-corpus BM25, O(N) per query. For "which doc says X".
- **Learning tier** (`map.reweight`): IDF calibrated on live vocabulary (10,515 terms), coordinates reembedded. Rerunnable.

## Caveats (explicit, do not claim beyond)

1. `p_useful=1.000` is circular: tasks authored from map vocabulary. Proves the harness *measures*; magnitude needs held-out tasks (next goal).
2. Cross-salience recall still 0.00 (matched-salience only).
3. Geo retrieval alone is not semantic (fold 256→3). Lexical arm carries doc search.
4. No learned embeddings (no new deps by design). Baseten GPUs change this calculus — a real encoder is the highest-ROI next spend.

## Ask (Baseten)

GPU-backed encoder behind the `embed_direction_idf` interface (no coordinate-format change, old atoms stay valid) + held-out A39 with real judge model. Both are scoped, receipted, and falsifiable — the harness already exists.
