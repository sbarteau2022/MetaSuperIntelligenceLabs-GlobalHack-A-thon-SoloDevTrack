---
name: allocating-weights
description: Allocate salience, IDF influence, fusion weights, and traversal budgets across the map — where gradient descent actually happens (never in weights). Use when tuning retrieval rank, spend, or learning dynamics.
---

# AllocatingWeights

Weights here are NEVER model weights (`dW = 0` by construction — frozen
reads only). The five allocatable weights, where gradient descent happens,
and the acceptance bar for each change.

## When to use

Retrieval ranks wrong, spend is off, learning stalls, or a sweep needs
running. Never touch all five at once — one allocator per change, measured
before/after.

## The five allocators

### 1. Salience (per-atom importance → shell radius)

- **Mechanism.** `radius = (1−s)×2.5 + 0.1`. High salience sits near origin.
- **Allocate.** Ingest `salience` by source authority (distilled arcs 0.7–0.9,
  doc flood 0.5, probes 0.1). Coalescence reinforces +0.05; consolidate
  decays −0.02 with a 0.05 floor (nothing starves to zero).
- **Compare invariantly.** Retrieval uses `shell_dist` (direction only), so
  salience never blocks recall — it ranks ties. Cross-salience recall is
  proven (`cross_salience_recall`, 0.9-atom rank #0 from qs=0.5).
- **Don't.** Use salience as a relevance score — it's storage geometry,
  not judgment. Judgment is the judge gate + BM25.

### 2. IDF influence (vocabulary discrimination)

- **Mechanism.** `embed_direction_idf`: hashed-TF × learned IDF → salted
  fold → unit 3D. Calibrated by `map.reweight` over live texts (10,515
  terms live). Rare terms pull harder; stopwords fade.
- **Allocate.** Reweight after every bulk ingest (O(N) once, O(1) per embed
  after). The GPU encoder (cleared, `GPU-ENCODER-CLEARANCE.md`) trains the
  256→3 projection against the 12-probe audit — that training IS gradient
  descent on this allocator.
- **Don't.** Hand-tune term weights. The table comes from the corpus or
  it doesn't ship.

### 3. Fusion weights (geo vs lexical)

- **Mechanism.** `map.query` fuses `gw×geo_norm + bw×bm25_norm` (min-max on
  the shortlist). Adaptive: BM25 signal present (max > 0.5) → 0.2/0.8;
  flat → 0.8/0.2. Neither arm drowns the other.
- **Allocate.** The 0.5 threshold and 0.2/0.8 splits are swept, not set —
  re-run the 12-probe audit to retune. Current: geo carries exact recall
  (1.00), lexical carries paraphrase (5/6 doc).
- **Don't.** Fix 50/50 — flat geo noise drowns BM25 signal half the time
  (measured 2026-10-28).

### 4. Traversal budgets (form/energy/frame gates)

- **Mechanism.** `map.traverse` checks form (tol 1e-9, proven non-binding),
  energy (cost vs budget), frame (eps 1e-2 default / 1e-3 audit).
- **Allocate.** Sweep table (40 live pairs × 16 configs): frame_eps is the
  binding gate (1e-6→0/40, 1e-3→7/40, 1e-2→36/40). Per-leaf budgets in
  `tactical-orchestration`; per-call overrides recorded in receipts.
- **Don't.** Loosen frame_eps to force dispatch — shear dispatched is
  corruption receipted. Fix the path, not the gate.

### 5. Coverage weight (what to distill next)

- **Mechanism.** Hunger priority `hits × depth × (1+recency)` ranks gaps;
  `map.consolidate` reinforces traversed (+0.05), decays untouched (−0.02).
- **Allocate.** Distill top-down from `map.hunger`. Sated gaps stay marked
  (the ledger remembers). This allocator IS the coverage gradient:
  `∂(success)/∂(atoms carved)` descended one sleep cycle at a time.
- **Don't.** Distill randomly or by recency alone. The ledger knows what's
  missing — read it first (H-5).

## Acceptance (every change)

1. State the allocator, the before number, the after number.
2. The number comes from a probe (`live_probe_audit_v2.py`), a sweep
   (`traverse_epsilon_sweep.py`), or a harness (`a39_heldout.py`) — never
   from reading code and feeling good.
3. Old receipts stay valid (additive changes; no re-minting history).
4. Record in the crate README step log.

## Native-stack note

All five allocators are `f64` fields in Rust structs, swept by Rust tests
and Python harnesses, persisted in bincode snapshots. JSON appears only
at the MCP boundary (tool args, receipts-in-transit). The direction of
travel: JSON shrinks to the boundary, Python shrinks to harnesses, and
the decision math — scores, gates, priorities, traversal contracts — is
native Rust today, native symplectic MLIR inside the model I/O boundary
tomorrow. That migration is the GPU-encoder + model-native-envelope work,
already scoped and gated (probe audit green or it doesn't ship).
