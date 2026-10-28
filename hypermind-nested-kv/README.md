# hypermind-nested-kv — Hyperbolic Memory Map (MCP server)

Binary `hypermind-mcp`. Tools: `map.ingest / query / attend / traverse /
lexical / reweight / carve / dissolve / compact / checkpoint / snapshot /
recover / health / export_passport / create`.

See `PLAIN-LANGUAGE-SPEC.md` for the system in plain words,
`SYMPLECTIC-ENVELOPE-SPEC.md` for the traversal contract,
`BASETEN-GRANT-PACK.md` for measured numbers,
`GPU-ENCODER-CLEARANCE.md` for the GPU encoder drop-in contract (step 2).

## Step log (goal bd01f172 — README updated per step)

### Step 1 — Salience normalization (DONE 2026-10-28)

- **Problem.** Query at fixed `qs=0.5` could never retrieve atoms stored at
  salience 0.9 (measured 0.00 top-1): radius encodes salience, so
  cross-shell content was geometrically far.
- **Fix.** `shell_project()` (direction only) + `shell_dist()` used by the
  VP index (build + search), retrieval `score()`, and dedup pre-check.
  Stored coordinates untouched; travel-cost paths (traverse, basins) keep
  raw distance where radius is real.
- **Proof.** `shell_dist_is_salience_invariant` + `cross_salience_recall`
  green; live 0.9-atom retrieved rank #0 from a flat query (was absent).
  26/26 tests. Shortlist widened 4k→8k (floor 256) so twins survive to fusion.
- **Contract (honest).** Geo answers exact twins cross-salience; extended
  paraphrases are the lexical arm's job (`map.lexical` ranks both #0/#1).

### Step 2 — GPU encoder clearance (DONE 2026-10-28)

- **Contract frozen:** `embed(text) -> unit [f64; 3]`, deterministic.
  Everything downstream (coords, shell, symplectic, receipts) unchanged.
- **Acceptance:** geo-alone paraphrase audit >= 6/12 (3x current), exact
  recall stays 1.00, weights hash pinned in reweight receipt, one-pass
  migration with zero text loss. See `GPU-ENCODER-CLEARANCE.md`.
- **Not cleared:** geometry/traversal/judge changes, network in hot path,
  silent substitution (probe audit gates dispatch).

### Hunger Ledger (DONE 2026-10-28, goal 134e5fbb)

- **Req 1 — Rules:** `RULES-HUNGER-LEDGER.md` (H-1..H-10): ignorance is
  first-class, no model in tap path, telemetry privacy, append-only,
  hunger ranks distill priority, consolidation is receipted sleep,
  zero-permission taps, aggregation prior, falsifiability.
- **Req 2 — Tap'n'Trace:** `TAP-N-TRACE.md`: taps watch receipt envelopes
  (never payloads); T&O SQLite log coalesces on gap_key; `map.hunger`
  serves pre-ranked ignorance. Worked example: mlir_op#26.
- **Req 3 — Tap structure live:** 5 taps (traverse_refused incl. missing
  endpoints, lexical_thin, query_thin, judge_rejected) + `map.hunger` +
  `map.consolidate`. In-server store mirrors the SQLite schema 1:1 —
  N³ upgrade with zero shape change. Same process = zero permissions
  by construction.
- **Proof live:** 3 refuses → hunger top-1 (priority 12.0) → consolidate
  (2 reinforced, 2072 decayed to 0.05 floor, 1 sated) → 0 open.
  29/29 tests (hunger accumulates/ranks, sleep sates, rows survive snapshot).

### Step 3 — Orchestration protocol (DONE 2026-10-28)

- **Pipeline:** Meta (decompose) → necessity filter → tactical mapping →
  orchestration with receipts. See `ORCHESTRATION-PROTOCOL.md`.
- **Min-info rule:** `I*(L)` = smallest set with coverage ≥ θ at depth
  ≥ δ(L). Necessary (absence voids the leaf), not merely sufficient.
- **Gradient:** no weight updates ever; descent on coverage (carve where
  retrieval failed), budget (variational depth gate), epsilons (sweep to
  operating point), IDF (reweight to usage). GPU encoder is the fifth.
- **Sundowning:** context-fill (capsule checkpoints), salience drift
  (recency + reweight), plan decay (re-check at dispatch). Failed
  traversal mid-plan IS the detector — re-ground, don't push through.
- **Superior Quality:** `E[Q] = cov × p_useful × (1-drift)`, all measured.
