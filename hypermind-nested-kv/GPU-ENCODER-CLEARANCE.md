# GPU Encoder Clearance (Step 2, goal bd01f172) — cleared for takeoff

**Status.** CLEAR. The interface is frozen; any GPU encoder that honors it
drops in with no coordinate-format change and no live-atom migration.

## The contract (frozen — do not widen)

```
embed(text: &str) -> [f64; 3]     // unit direction, deterministic
```

That's it. Everything downstream is fixed:

- `coord_from_text_idf(text, salience, idf)` wraps the direction in the
  salience radius + T² phase. Unchanged.
- `shell_project / shell_dist / score` compare directions. Unchanged.
- `symplectic_from_triple` derives momentum from (coord, phase). Unchanged.
- Snapshots, receipts, VP index, traversal contracts: all consume `[f64; 4]
  + [f64; 2]`, never the embedder internals.

## What the current CPU embedder does (the bar to beat)

`embed_direction_idf`: hashed-TF 256d × learned IDF → salted fold → unit 3D.
Measured: exact recall 1.00, paraphrase doc retrieval 0–2/12 geo (lexical
arm carries at 5/6). The fold 256→3 is the information bottleneck — no
weighting recovers what folding destroys.

## What the GPU encoder must deliver (acceptance)

1. **Same signature.** `String -> unit [f64; 3]`, deterministic
   (same text → bit-identical direction), no network at query time
   (weights local, Baseten or bundled).
2. **Beat the bar.** Paraphrase probe audit (`live_probe_audit_v2.py`,
   12 cases) ≥ 6/12 doc-correct with the geo arm ALONE (BM25 off) —
   triple the current 2/12. Exact recall stays 1.00 (`distill_ceiling.py`).
3. **Receipt it.** Encoder weights hash pinned in the reweight receipt
   (`map.reweight` gains an `encoder_digest` field); `map.health` reports
   which encoder produced the live coordinates.
4. **Migrate cleanly.** `map.reweight`-style one-pass reembed of all live
   atoms under the new encoder; old snapshot hash retires, new snapshot
   receipts the migration. Zero atom-text loss (coordinates change, texts
   don't).

## Suggested shape (PROPOSED, not prescribed)

Small sentence-encoder (e.g. 384d MiniLM-class, Baseten-hosted or
distilled-local) → learned 384→3 projection (calibrated on live vocab,
same IDF philosophy) → L2 normalize. The projection trains against the
12-probe audit as its loss: paraphrase pairs near, strangers far.

## What is NOT cleared

- No change to H³×T² geometry, traversal contracts, judge, or dedup.
- No network dependency in the query hot path (batch/embed offline or
  sidecar; the MCP server never blocks on a socket for coordinates).
- No silent substitution: a new encoder without a passing probe audit
  does not ship (same rule as microkernels: oracle first, dispatch second).
