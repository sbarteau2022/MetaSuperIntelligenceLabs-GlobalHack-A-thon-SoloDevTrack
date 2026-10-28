# Tap'n'Trace — Formalization (goal 134e5fbb, Req 2)

**One sentence.** Taps watch receipts, not payloads; the T&O log counts
before any model looks; `map.hunger` serves pre-ranked ignorance.

## 1. What a tap is (structure)

```
TOOL RESULT (receipt envelope)
  { tool, status, atom_ids, scores, depth, receipt_sha256, timestamp }
       │
       │  tap subscribes (read-only, zero permissions per H-7)
       ▼
TAP (deterministic function, no model, no network)
  in:  receipt envelope
  out: hunger event { gap_key, hits:+1, depth, terms_hash, timestamp }
       │
       ▼
T&O LOG (SQLite, append-only, N³-T&O-Log store)
  coalesce on gap_key: hits++, last_seen=now
       │
       ▼
map.hunger (read path)
  rank by priority = hits × depth × (1 + recency)   [H-5]
  return top-k (hashes + ids + ranks, never raw text) [H-3]
```

## 2. Aggregation prior (the key guarantee)

The model's first sight of hunger is `map.hunger` output. Between the
failure and the model sit TWO deterministic stages (tap function + T&O
coalesce) that contain zero inference. By the time a model could ask "what
are we missing," the answer is already counted, de-duplicated, and ranked.
The model never scans failure streams; it reads a leaderboard.

**Why not the model?** Three reasons, all load-bearing:
1. **Cost.** Scanning 2,074 atoms of failure context per question is the
   O(N) mistake `map.attend` just stopped making.
2. **Trust.** A model summarizing its own ignorance is circular (the same
   circularity that voided the first `p_useful=1.000`). Counters don't flatter.
3. **Determinism.** Same failures → same ledger, bit-identical. A model in
   the path breaks reproducibility (C31 Rule 5).

## 3. Tap points (where taps attach — all receipt envelopes, zero new plumbing)

| Tap | Attaches at | Gap key | Depth |
|---|---|---|---|
| `hunger.traverse_refused` | `map.traverse` fail-explained | `(from_atom, reason_class)` | hop depth requested |
| `hunger.lexical_thin` | `map.lexical` top bm25 < threshold | `terms_hash(query)` | 1 |
| `hunger.query_thin` | `map.query` fused top < threshold | `terms_hash(query)` | depth_k used |
| `hunger.judge_rejected` | `map.ingest` judge-rejected | `terms_hash(text)` | 0 (pre-storage) |
| `hunger.uncovered_task` | A39-style coverage miss | `task_id` | chain depth needed |

Thresholds are preregistered (defaults: bm25 < 5.0 = thin; fused < 0.3 = thin)
and reported in every hunger receipt. Tuning them is a sweep, not a judgment.

## 4. T&O Log store (SQLite, N³-T&O-Log)

```sql
-- One row per distinct gap. hits counts repeats. Nothing deleted ever.
CREATE TABLE hunger (
  gap_key     TEXT PRIMARY KEY,   -- deterministic from tap (see §3)
  tap         TEXT NOT NULL,      -- which tap minted it
  terms_hash  TEXT NOT NULL,      -- sha256 of sorted query terms (H-3)
  atom_ids    TEXT NOT NULL,      -- JSON array of involved atom ids
  hits        INTEGER NOT NULL DEFAULT 1,
  depth       INTEGER NOT NULL DEFAULT 1,
  first_seen  INTEGER NOT NULL,   -- unix seconds
  last_seen   INTEGER NOT NULL,
  sated       INTEGER NOT NULL DEFAULT 0,  -- 1 when a later distill covers it
  sated_by    TEXT                -- distill receipt that sated it
);
CREATE INDEX hunger_priority ON hunger (sated, hits DESC, last_seen DESC);
```

**No permissions.** The T&O log lives inside the server process (same
address space as the receipt stream). Taps are function calls, not IPC —
there is nothing to grant because nothing crosses a boundary. When N³
installs as a separate process, taps connect over the read-only receipt
stream (receipts are already public to any MCP client); the SQLite file is
server-local, never shared. Zero-permission holds in both topologies.

## 5. Worked example (mlir_op#26, the canonical hunger)

1. `map.traverse(1142→1305, op 26, budget 0)` → fail-explained (over budget).
2. Tap `hunger.traverse_refused` fires on the receipt envelope:
   `gap_key = traverse:1142:over-budget`, hits +1.
3. Twelve more identical failures → `hits = 13`, priority climbs.
4. `map.hunger` top-1: "13 failures traversing 1142→*, all over-budget."
5. Distill decision (human/GPU): carve the missing arc OR raise the budget.
6. Distill lands → coverage probe passes → gap marked `sated`, `sated_by`
   = the distill receipt. The ledger remembers it was hungry.
