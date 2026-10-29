# Engine Build Checklist — Derived from audit, in-repo spec only

**Date.** 2026-10-28. **Status.** BUILD SPEC (no engine code edited; `monster-hybrid-server` treated as read-only reference per workspace rules).
**Companion.** `DistillationProtocol/DISTILLATION_V1_V2_DEPTH_PROOF.md`, `PER_MODEL_KNOWLEDGE_AND_TRAJECTORY_WALKS_PARAM_PROOF.md`, staged `research-grade-harness/src/distill_judge.rs`.

## Boundary

- We do NOT modify `/Users/stewartbarteau/Desktop/monster-hybrid-server`. The four items below are specified for the engine owner to implement in `immune_system/implementation/src/distill_engine.rs` and related modules. Any edit there is out of scope for this repo and must be done by its steward.
- Read-only anchors below are `VERIFIED` by grep/read this pass.

## The four build items (ordered)

### 1. DistillationJudge — enforcement, not just counters (Priority: first)

- **Gap.** 0 `judge` refs in `distill_engine.rs` + `refraction.rs` (grep-VERIFIED). Counters (`modes_scanned`, `novel_atoms_minted`, `coalesced_atoms`) exist in receipts; nothing enforces them.
- **In-repo template.** `research-grade-harness/src/distill_judge.rs::DistillationJudge` (staged, 5 tests pass). Structural gate:
  1. `knowledge_atoms_minted == modes_scanned`
  2. `novel_atoms_minted + coalesced_atoms == modes_scanned`
  3. `sum(role_counts) == knowledge_atoms_minted`
  4. `0 <= mean_retained_energy <= 1`
- **Engine work.** Add an analogous `DistillationJudge` in-engine; call it at the end of every `distill_*` entry point and on merge; emit `JudgeResult` into the receipt; reject on fail. Teacher identity must not be a field (§8).

### 2. `distill_live_step_novel_only` — chained continuous pass

- **Gap.** Only `distill_reasoning_chain` (static, pre-built chains) and `distill_safetensors_file_novel_only` (file-scanning) exist; nothing threads a live pass with `(prev_atom, prev_role)`.
- **Required signature (per existing plan).** `distill_live_step_novel_only(tap, …, previous_atom_and_role, …)` returning `LiveNoveltyDistillationReceipt{atom_was_novel, arc_was_novel}`.
- **Effect.** Single read per teacher element (O(h), no O(h²)); idempotent because repeats coalesce (Theorem 2); cross-band order preserved.
- **Walk depth.** Must accept steps spanning early/mid/late bands; must not collapse to one band or the 2-cell holonomy cannot close.

### 3. Depth-stratified budget allocator (not just scoring bonuses)

- **Current state (VERIFIED).** Bands exist as `+0.35` in-band scoring bonuses in `classify_icosahedral_well` (`:157–200`); greedy-by-energy happens _inside_ one layer’s top-k; nothing allocates budget _across_ bands.
- **Required allocator.** Input `(L, B)` → outputs `n_early/mid/late` (floor at 0.34/0.34/rest) and `B_early/mid/late = (0.20/0.50/0.30)*B`, with `k_band = max(1, round(B_band / n_band))`.
- **Must feed** all three pillars (`distill_static_weight_modes`, live step, quantized shard sieve) so `top_k_rows_per_layer` is band-aware rather than uniform.
- **Proof.** Theorem 3 (exchange argument): strictly more retained energy whenever band energies differ.

### 4. Explicit cosh coalescence gate

- **Current state (VERIFIED).** Dedup goes through `keep_and_carve_novel_only` with a spatial hash; no explicit `cosh(d) <= 1.0000001` predicate in the distill surface (the constant appears in `silo.rs:1226`).
- **Required.** Make the novelty decision call the cosh-distance threshold explicitly and return `(AtomId, bool)`. Thread the bool into receipts end-to-end (do not discard via `with_origin` as Pillar 1 currently does at `silo.rs:1315`).

## Supporting requirements (must hold for full rank to be measured)

- Retain ≥ 8 independent directions through projection (MGS, ε=0.08) so `active_rank` can reach 8; detect collapse via the engine’s `"low rank jacobian"` path.
- Judge gate (item 1) must be the only shared-scope entry condition.
- Twin-fingerprint invariance test must be run before declaring Theorem 5 proved (spec at `DistillationProtocol/TWIN_FINGERPRINT_TEST_SPEC.md`).
- A39-style ablation measuring ΔCap ≥ 0 remains the magnitude gate; mechanism claims (T1–T4) are already proved.

## What we (this repo) have already produced

- `DistillationProtocol/DISTILLATION_V1_V2_DEPTH_PROOF.md` (theorems T1–T5, proofs).
- `DistillationProtocol/PER_MODEL_KNOWLEDGE_AND_TRAJECTORY_WALKS_PARAM_PROOF.md` (per-model params, B/k tables, maximization checklist).
- `DistillationProtocol/model_roster.json` (Muse fingerprinted; 5 teachers).
- `DistillationProtocol/muse_fingerprint_receipt.json` (per-shard + combined hashes).
- `DistillationProtocol/scripts/validate_roster.py`, `validate_taps.py` (contracts).
- `research-grade-harness/src/distill_judge.rs` (structural judge template, 5 tests pass) — staged.
- `research-grade-harness/src/distillation.rs` (depth allocator + cosh gate + live threaded pass reference implementation, 5 tests pass) — staged.

## Nothing executed

No `distill_*` entry point invoked, no map written, no engine code modified.
