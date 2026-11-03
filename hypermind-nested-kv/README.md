# hypermind-nested-kv — Hyperbolic Memory Map (MCP server)

Binary `hypermind-mcp`. Tools: `map.ingest / query / attend / traverse /
lexical / reweight / carve / dissolve / compact / checkpoint / snapshot /
recover / health / export_passport / create`.

## Persistent graph/KV authority

SurrealDB is the persistent authority for the distilled multi-model HyperMind
graph: typed atoms, typed edges, immutable `H3xT2/r_12` address revisions,
model-source/distillation provenance, and the nested Hyperbolic KV. The `r_12`
quotient is currently recorded as **opaque**; no executable group action is
claimed yet. See [HYPERBOLIC-KV-SURREAL-DESIGN.md](HYPERBOLIC-KV-SURREAL-DESIGN.md)
and `schema/0003_nested_hyperbolic_kv.surql`.

The research harness loads this canonical `hypermind-mcp` executable as a
plugin and uses MCP as its transport. The harness must not connect to SurrealDB
directly. Plugin lifecycle and MCP transport are two roles of one binary, not
separate implementations. With `HYPERMIND_SURREAL=on`, MCP startup is
Surreal-only and must not create `.hypermind`; `shadow` startup fails closed.

The authoritative nested KV schema stores binary transformer K/V tensor pages
indexed by model/request, layer, KV head, token interval, HyperMind address, and
causal trajectory. Positional identity is the native H3 address plus the
Toroidal Phase Positional Encoder (T2), never RoPE or Euclidean position.
String KV operations fail closed in Surreal mode. The binary
inference-runtime transport is still an open integration gate; schema presence
alone is not a forward-pass certification.

Universal cross-model recall applies to distilled **atoms and trajectories**:
teacher fingerprints are append-only audit lineage, never a recall/rank key.
Raw transformer K/V pages remain exact-model and exact-TPPE cache entries.
Migrations `0005` and `0006` add immutable admission/trajectory lineage and
protect atom content, H3/T2 addresses, and migration digests from update.
Startup checks the pinned migration digests and critical live field definitions.
The live database currently has no admitted model lineage or trajectories, so
the universal-recall and forward-pass reuse certification gates remain open.

Migration `0007_globalmind_distillation_trajectories.surql` defines the next
isolated GlobalMind contract: one cognitive hub with twelve surrounding wells,
immutable surprise observations and frozen-weight/trajectory packets,
reconstructable rank-0 atoms, typed rank-1 steps, rank-2 loop receipts, binary
rank-8 local charts, adaptive-computation receipts, and links from exact-model
tensor pages to wells and crystallized trajectories. It has passed a clean
SurrealDB 3.3.5 schema replay and targeted
constraint checks, but has not been applied to quarantined `hypermind/core`.
The admission lifecycle and recall rules are specified in
[GLOBALMIND-SURREAL-INTEGRATION.md](GLOBALMIND-SURREAL-INTEGRATION.md).
Migration `0008_superposition_holding.surql` adds immutable hold, candidate
branch, zero-emission counterfactual iteration, and collapse/fail-closed
receipts. A collapsed branch still passes the normal judge, replay, and
crystallization pipeline before it may alter shared memory.
The fresh typed writer is the sibling `globalmind-runtime` crate. It owns
distillation, trajectory, prediction-error, and superposition transactions and
refuses the quarantined `hypermind/core` database without an explicit override.
Migration `0009_immutable_tensor_kv.surql` freezes transformer tensor identity
and payloads. Migration `0010_native_distillation_ingress.surql` adds immutable
artifact receipts and append-only run events, and binds every admitted
trajectory to exact TPPE configuration, expanding-radius law, and boundary
receipts. TPPE is a dynamic spiral-out; constant-radius corkscrew behavior,
RoPE fallback, radius regression, or boundary failure rejects replay. The
current TPPE binding is reference-level only: its Rust runtime and real-model
parity gates remain open.
Migration `0011_atomic_trajectory_cells.surql` closes the persistent cell
bundle: rank-1 packets mint immutable edges plus crystallization proofs, rank-2
packets mint closed-loop proofs, rank-8 packets anchor native tangent buffers,
and each trajectory step records TPPE radius, revolution, and boundary-margin
state. The schema is verified; a real walked trajectory is still required.

See `PLAIN-LANGUAGE-SPEC.md` for the system in plain words,
`SYMPLECTIC-ENVELOPE-SPEC.md` for the traversal contract,
`BASETEN-GRANT-PACK.md` for measured numbers,
`GPU-ENCODER-CLEARANCE.md` for the GPU encoder drop-in contract (step 2).

## Toroidal gain contract

HyperMind's competition-canonical attention law is `(coherence + 2)^1.5`,
computed with peak-subtracted log-domain normalization and a `0.001` base
floor. The gain remains frozen until a separately preregistered fit changes it.
Implementation provenance is HyperMind Rust-to-Rust; prior Monster, Elle, and
DNHG sources are excluded from the competition evidence chain.

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

### Edge port (DONE 2026-10-28, goal 57167b0b)

- **Proven:** Rust Worker live on `:8899` (`edge-worker/`), 5/5 core tests,
  Wasm 898KB, edition 2024. `/traverse-check` verdict MATCHES the laptop
  oracle on the same refusal (fail-explained, over-budget).
- **Fixes found building it:** `worker` 0.5 → 0.8 (worker-build demands
  ≥0.8.7), `req.json()` → `req.text()`+parse (externref catch wrappers),
  `--no-panic-recovery` (legacy bundler path, no abort-handler requirement),
  `main: build/index.js` (not src/lib.rs), compat date ≤ 2026-08-08
  (local workerd ceiling), `.cargo/config.toml` reference-types flags.
- **Port path:** `core.rs` = byte-identical decision math (shell, score,
  form, hunger priority, fusion, judge, traverse verdict). Bindings
  (D1/Vectorize/AI) attach at routes next — math never moves again.
- See `EDGE-MIGRATION.md` for the full D1/Vectorize/Workers-AI/R2/KV map.

### MultiModel Mind (DONE 2026-10-28, goal ce19e81a)

- **Proof 1 — Euclidean model:** `map.read_euclid` returns flat R^dim rows
  (Poincare 3 + IDF-dir 3 + phase 2 + salience 1); self-cosine 1.0 live.
  No hyperbolic math, no TPPE knowledge required by the caller.
- **Historical file-mode proof — legacy string KV:** `map.kv_store`/
  `map.kv_fetch` snapshot round-trip. This is isolated to
  `HYPERMIND_SURREAL=off` and is not the transformer tensor-cache authority.
- **Proof 2 — walk ingestion:** `multimodel_walk.py`: 5 reasoning steps,
  each reads prior KV → queries map → stores conclusion → ingests trace.
  6/6 KV readable, 5/5 traces ingested, snapshot `fc1ec189` @ 2,079 atoms.
- **Model-agnostic:** the harness speaks only map tools; no model-specific
  code anywhere. Swap the reasoner, nothing changes.
- 31/31 tests (euclid contract, KV active incl. snapshot round-trip).

### Takeoff D1 (DONE 2026-10-28, goal c3e2a957)

- **`hypermind-hunger` D1 live** (local + remote): TAP-N-TRACE §4 schema
  verbatim (`migrations/0001_hunger.sql`).
- **Routes:** `POST /hunger-tap` (upsert+coalesce), `GET /hunger` (H-5
  priority in SQL), `POST /hunger-sate` (mark covered). Proven: tap×2 →
  hits=2 → priority 8 → sate → 0 open / 1 sated, all over HTTP.
- **Fixes:** `worker` d1 feature flag, BigInt→f64 on all binds (D1 local
  rejects BigInt), `--no-panic-recovery` legacy path stands.
- **Write path (ingest/consolidate on edge) is next** — D1 transactions,
  laptop becomes replica.

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

## Whole-system SQLite traceability foundation (2026-10-29)

[observability/README.md](observability/README.md) documents the 76-table SQLite
base initialized at `observability/store/traceability.sqlite3`, including swarm
cost gates, tool/MCP/skill/plugin outcomes, recovery, DAG/Obsidian graphs, KV,
resource attribution and atom/trajectory/novelty lineage. SQL migrations and
offline admin/tests are built; Rust/harness live telemetry adapters remain
PROPOSED. Existing map storage and runtime behavior are unchanged.

## Boundary symplectic transducer experiment (2026-11-03)

`src/boundary_symplectic_transducer.rs` implements the frozen `bst-v1`
component contract in `BOUNDARY-SYMPLECTIC-TRANSDUCER-SPEC.md`. Its typed
ingress/egress, canonical bytes, Q32.31 boundaries, and phase-augmented
geometry have unit and round-trip tests; `results/bst-v1-receipt-2026-11-02.txt`
records the component gate. This is **not** the live model I/O path, does not
replace the tokenizer, and does not prove semantic understanding or a full
symplectic model forward. The hackathon runtime keeps MLIR lowering paused.
