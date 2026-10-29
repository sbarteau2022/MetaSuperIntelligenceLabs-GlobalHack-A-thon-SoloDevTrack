# Per-Model Parameter Proof — Knowledge + Trajectory Walks at Depth

## DistillationProtocol (companion to DISTILLATION_V1_V2_DEPTH_PROOF.md)

**Date.** 2026-10-28. **Status.** PROPOSED parameter set + VERIFIED code facts (per-model configs from local model dirs) + INFERRED allocation math. No distillation executed; no live map touched.
**Scope.** Answers: for each teacher model available locally, what numeric parameters should be set to (a) maximize retained _knowledge_ (0-cells/weight modes) and (b) maximize _trajectory walks_ (1-cells/arcs) at depth, with proof.
**Source of truth.** `distill_engine.rs` constants and formulas (`monster-hybrid-server`, VERIFIED this pass); per-model `config.json` (VERIFIED by read); theorems T1–T5 of `DISTILLATION_V1_V2_DEPTH_PROOF.md`.
**Standards.** Doc-standards §7: every claim labeled VERIFIED/REPORTED/INFERRED/PROPOSED; corrections appended, never rewritten.

---

## 0. Parameter vocabulary (read first)

All parameters below are the knobs that appear (or are derived from) the engine. Nothing new is invented as code; derived values are labeled INFERRED/PROPOSED.

| Parameter               | Symbol                               | Engine anchor                                                 | Role                                                                                    |
| ----------------------- | ------------------------------------ | ------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Entropy gate            | `Hmax = 2.75 bits`                   | `max_entropy_bits_q32` (distill_engine.rs:95–104)             | Rejects high-entropy teacher guesses (SIEVE). Raising admits more walks but adds noise. |
| Top-k rows/layer        | `k`                                  | `top_k_rows` / `top_k_rows_per_layer` (:533,:977,:1108)       | Modes retained per layer (energy budget per layer).                                     |
| Depth bands             | `early/mid/late`                     | boundaries `0.34, 0.68` of `depth_ratio` (:157–200)           | Budget spent where reasoning lives.                                                     |
| In-band bonus           | `+0.35`                              | `classify_icosahedral_well` (:157–200)                        | Certainty bonus for the role matching the band.                                         |
| Greedy vs stride        | select-greatest                      | analytic sieve `E_est(r)` (:550–567) + sort desc (:574)       | T1: greedy dominates stride (9.90× on Zipf s=1).                                        |
| Novelty coalescence     | `cosh(d) ≤ 1.0000001`                | `silo.rs:1226`                                                | Cross-teacher duplicates coalesce (§8 invariance).                                      |
| MGS epsilon             | `ε = 0.08`                           | `CONVERSION_MATH.md` §1                                       | Projection orthogonalization.                                                           |
| Trajectory loop closure | `closed_roles` + first role appended | distill_engine.rs Pillar 2 (end of `distill_reasoning_chain`) | Enables Gauss–Bonnet holonomy → salience +2 on `DialecticalSynthesisVerified`.          |
| Walk chaining           | `(prev_atom, prev_role)` thread      | `distill_live_step_novel_only` (:1997–2085)                   | Continuous single pass; idempotent (repeats coalesce).                                  |

---

## 1. General law (PROVED by T1 + T3 + T4)

1. **Knowledge maximize rule.** Replace uniform stride with greedy top-k by analytical spectral energy `E_est(r)` (T1). `k` is the per-layer budget; energy share follows salience, not scan order.
2. **Depth maximize rule.** Allocate `k` across layer bands in proportion to band energy prior `(early 0.2, mid 0.5, late 0.3)` (T3). Mid-heavy teachers (the normal case: mid/late carry reasoning) yield strictly more retained energy than a uniform sweep for any differing band energies.
3. **Trajectory-walk maximize rule.** Keep the entropy gate at `2.75` (the engine's built-in sieve) and capture walks that **span all three bands** (early→mid→late) so the closed role loop fires holonomy reinforcement. A walk wholly inside one band never receives the +2 salience boost.
4. **No fine-tuning.** Every update is `M ∪ {mint(project(content))}` or `salience += 1`; `dW = 0` identically (T4). So these parameters can be run in a single continuous pass with zero gradient accumulation.

---

## 2. Band allocation math (INFERRED from T3; executable)

For a model with `L` layers and per-pass budget `B` (total top-k across layers), the per-band layer counts are:

```
n_early = floor(0.34 · L)
n_mid   = floor(0.34 · L)
n_late  = L - n_early - n_mid
```

and the per-band mode budgets (PROPOSED, proportion = band energy prior):

```
B_early = round(0.20 · B)
B_mid   = round(0.50 · B)
B_late  = round(0.30 · B)
k_band  = round(B_band / n_band)   (modes per layer within that band; ≥1)
```

Depth bonus `+0.35` applies only to the role whose band matches the layer (engine `:160–200`); off-band layers get the role by content only.

---

## 3. Per-model table (configs VERIFIED by reading each `config.json`)

Teachers are local models under `~/models`. Embedding-only models are **not** reasoning teachers (mark: weight-modes only, no trajectory chains). All figures below are PROPOSED recommendations; labels attached per cell.

| Model                                  | Type           | L               | hidden d | GQA (h/kv) | Experts (act/tot) | Quant                     | Role in distillation                                                 | Band layer counts (E/M/L) | Recommended B (total top-k) | Per-band B (E/M/L) | k per layer (E/M/L) | Trajectory-walk prescription                                                                                                                                                               |
| -------------------------------------- | -------------- | --------------- | -------- | ---------- | ----------------- | ------------------------- | -------------------------------------------------------------------- | ------------------------- | --------------------------- | ------------------ | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Muse-Glimmer-q4km-gs128-MLX            | dense LM       | 52              | 6656     | 32/2       | —                 | 4-bit (gs128)             | **Full teacher** (knowledge + walks)                                 | 18 / 18 / 16              | 128 (PROPOSED)              | 26 / 64 / 38       | 2 / 4 / 3 (min 1)   | Capture 1 step/layer across all 52 (full continuous pass); arc chain length 51; require early+mid+late steps present so holonomy closes.                                                   |
| NVIDIA Nemotron-3.5-Lightning-30B-A3B  | MoE LM         | 52              | 2688     | 32/2       | 6/128 (VERIFIED)  | 4-bit (gs64, affine)      | **Full teacher**                                                     | 18 / 18 / 16              | 128 (PROPOSED)              | 26 / 64 / 38       | 2 / 4 / 3           | Same as above; MoE routing gives multiple expert trajectories per layer — capture the _active-expert_ path only (one walk per token) to avoid duplicate walks across co-activated experts. |
| Qwen3.6 MoE Combined (Qwen3.6-35B-A3B) | MoE+VL         | 40              | 2048     | 16/2       | 8/256             | 4-bit (gs64; gates 8-bit) | **Full teacher** (text path)                                         | 14 / 14 / 12              | 96 (PROPOSED, scaled to L)  | 19 / 48 / 29       | 2 / 4 / 3           | 40-step chains (39 arcs). Note 8-bit gates → higher retained energy on early layers; do NOT override depth prior; let Fisher bonus (engine :644) raise early-layer certainty.              |
| Qwen3-Embedding-8B-4bit-DWQ            | dense embedder | 36              | 4096     | 32 (kv?)   | —                 | 4-bit DWQ                 | **Weight-modes only** (no reasoning chains: no next-op distribution) | 13 / 12 / 11              | 64 (PROPOSED)               | 13 / 32 / 19       | 2 / 3 / 2           | N/A — trajectory arcs require `next_op_probs`; embedding hidden states carry no op entropy. Pillar 2 not applicable. Pillar 1 top-k only.                                                  |
| Qwen3-VL-Embedding-2B-6bit             | VL embedder    | ? (config null) | ?        | ?          | —                 | 6-bit                     | **Weight-modes only** (config incomplete — UNVERIFIED)               | UNVERIFIED                | UNVERIFIED                  | —                  | —                   | Same caveat as above; do not distill until config verified.                                                                                                                                |

Notes:

- `num_hidden_layers` for the VL-Embedding RUN config parsed as `None` (nested schema) — REPORTED as UNVERIFIED; do not set parameters from it yet.
- For the MoE gate-tensor exception (Qwen3.6 layers 0–5 gates at 8-bit), the engine's Fisher bit-width bonus `teacher_entropy_bits >> 1` (:644) already boosts certainty → deeper refract; no separate parameter needed. This is a code mechanism, VERIFIED.
- `B` values (128/96/64) are PROPOSED (scaled ~2.5×L roughly? No: scaled to L and map capacity). [INFERRED] — require A39/A/B calibration; treat as starting point for ablation, not truth.

---

## 4. Trajectory-walk specific parameters (at depth)

To maximize _number of 1-cell walks_ that also qualify as knowledge (judge-gated), set:

| Walk control          | Setting                                     | Why (label)                                                                                                                                                                                                                                                                               |
| --------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Entropy sieve `Hmax`  | **keep 2.75 bits** (do not raise)           | PROPOSED optimum: lower (e.g. 2.0) raises walk _quality_ but cuts count; higher (e.g. 3.5) inflates walks with hallucination steps that fail novelty (T2). 2.75 is the engine default and the only value with measured roots (:95–104). Sweep is an open ablation (CONVERSION_MATH §5 δ). |
| Walk length           | `L` steps, one per layer, in layer order    | PROPOSED for max walks: chained continuous pass via `distill_live_step_novel_only` (:1997) preserves cross-layer order; repeats coalesce → idempotent.                                                                                                                                    |
| Minimum walk coverage | ≥1 step in **each** of early/mid/late bands | INFERRED from engine: closed role loop only reinforces when early+mid+late roles form a cycle with icosahedral neighbors; a single-band walk yields arcs but no salience boost.                                                                                                           |
| Chaining thread       | `previous_atom_and_role` passed each step   | VERIFIED mechanism (:1997–2085); makes pass single-read (T4).                                                                                                                                                                                                                             |
| MGS epsilon           | `0.08`                                      | From CONVERSION_MATH §1 (engine-independent; projection).                                                                                                                                                                                                                                 |
| Novelty threshold     | keep default cosh `1.0000001`               | VERIFIED constant (silo.rs:1226); raising merges too much, lowering mints swamp atoms (T2).                                                                                                                                                                                               |

Expected walk count per model per continuous pass (PROPOSED upper bound, assuming no entropy rejections):
`walks_max = L − 1` (one arc between consecutive kept steps). For 52-layer teachers: 51 arcs; 40-layer: 39; 36-layer: 35. With sieve rejections, actual = kept − 1 ≤ L − 1.

Volume floor comparison (REPORTED): v1 map has 8,644 atoms / 104,312 arcs (PROTOCOL §7). To exceed: a single Glimmer 52-layer continuous pass under v2 accounting must beat 104,312 arcs — i.e. ≥ 2,046 distinct 51-arc trajectories worth of walks, or equivalent accumulated chains. PROPOSED, not certified.

---

## 5. Proof sketches (per-model claims)

**P1 (Glimmer, L=52, dense).** T1 gives 9.90× retained-energy advantage of greedy over stride on N=1000/k=64 (Zipf s=1); T3 gives 1.17× for mid-heavy allocation vs uniform sweep at L=52. Combined expected retained energy ≈ 9.90 × 1.17 ≈ 11.6× the v1 baseline on the same budget, B=128. [INFERRED product of independent inequalities — multipliers are NOT measured jointly; this is a PROPOSED upper bound, not a measurement.]

**P2 (Nemotron, L=52, MoE, 4-bit).** Same depth math as Glimmer (L identical, bands identical). Additionally, only the active-expert path is captured per token (6/128), so the effective walk count is per-token, not per-expert — avoiding duplicate (swamp) atoms across co-activated experts. MGS orthogonalization (ε=0.08) keeps expert-parallel directions from colliding in the same well. [PROPOSED policy; code's MGS is listed as AUDIT finding 3 UNVERIFIED on Pillar 1.]

**P3 (Qwen3.6, L=40, MoE).** Bands at L=40: 13/14/13 (vs 52's 17/18/17). B scaled down to 96 so per-layer k stays ≥2 in the early/late bands. 8-bit gates receive Fisher bonus → early-layer certainty raised automatically; do not manually bump early budget above 0.20 (would violate the depth prior T3 assumes). [INFERRED.]

**P4 (Embedders, L=36).** Trajectory walks require `next_op_probs` and a teacher entropy; embedding models expose none. Therefore `distill_reasoning_chain` cannot run on them — VERIFIED by the requirement in `TeacherTapRecord` (projector needs next-op distribution, :334 context). Only Pillar 1 (`distill_static_weight_modes`) applies: B=64, top-k only. [VERIFIED capability gap.]

**P5 (No fine-tuning, all models).** T4: `dW = 0` identically because weights enter as `&FrozenBaseWeights` with zero `&mut` (grep verified). Therefore a continuous chained pass across 52 layers cannot become fine-tuning by accumulation — the magnitude of capability is bounded by coverage, not by parameter change. [VERIFIED.]

---

## 6. What was NOT checked (do not claim beyond this)

1. No distillation executed on any model; all B/k figures PROPOSED starting points (A39 open).

- Nemotron routed-expert count verified this pass: `n_routed_experts=128`, `n_shared_experts=1`, `num_experts_per_tok=6` (so the act/tot = 6/128 + 1 shared stated in §3 is VERIFIED).

3. VL-Embedding schema unverified.
4. Twin-fingerprint invariance test (T5) unrunned — teacher-invariance conditional.
5. Joint measurement of T1×T3 multipliers never performed (P1 numbers are products of separate inequalities).
6. MGS on Pillar 1 unverified in code beyond grep (AUDIT finding 3 carried).
7. Map capacity `B_map`/`mem(atom)` (C05.11/C21) not calibrated this pass; B numbers assume headroom.

---

## 7. Maximization checklist (bind before every pass — depth is SUPERNECESSARY)

**Depth is not optional.** Theorem 3 proves that for any teacher whose band energies differ (the normal case: mid/late layers carry reasoning, early carry sensory/symbolic per the `:160–200` prior), salience-stratified allocation retains strictly more energy per budget than a uniform `--layers` sweep (1.17× in the modeled 52-layer case). Skipping depth (flat budget) is strictly worse whenever bands differ; skipping it when they differ is a protocol violation.

Bind all of these or the pass does not count as “maximized”:

1. **Frozen read only.** Weights/hidden states entered via `&`; zero `&mut` (grep gate, T4).
2. **Greedy, not stride.** Analytic sieve `E_est(r)` + descending sort + top-k (T1). A single `top_k` huge is wrong: budget must follow salience.
3. **Entropy sieve at 2.75.** Do not raise to inflate walk counts without measuring the energy cost (ablation open item).
4. **Depth allocation.** Bands at `0.34 / 0.68`, budget `0.20 / 0.50 / 0.30`. Never flat across layers.
5. **Walks span all three bands.** Early→mid→late steps required so the role loop closes and the 2-cell holonomy +2 fires; otherwise arcs are minted but not reinforced.
6. **Chained pass.** Thread `(prev_atom, prev_role)` via `distill_live_step_novel_only` for O(h), idempotent re-reads.
7. **Novelty metering.** Use the novelty bools (`LiveNoveltyDistillationReceipt` / `FullModelDistillationReceipt`); refuse `minted += 1` unconditionally (T2).
8. **Rank preservation.** Keep ≥ 8 independent directions through MGS (ε=0.08) so `active_rank` can reach 8; collapse-detect via the engine's low-rank flag.
9. **Judge gate before shared insert.** Structural (AUDIT §7) + eventual A39 capability gate; no judge-passed packet, no shared-map entry.
10. **Invariance.** Teacher fingerprint in lineage/audit/receipts only; coordinates/retrieval/rank never consult it (§8 + twin-fingerprint test).
11. **Capacity headroom.** Check `N_max = B_map / mem(atom)` (C05.11/C21) so the minted floor fits; otherwise prune leaf-only (anchors/locks never overridden).
12. **Receipt chain.** `Σ receipt_ok == atoms_minted` (NN-G3 analog) or the pass is invalid.
13. **Post-pass verification.** A fresh run with pinned context + tail takes the same next action (PROTOCOL §2.10).

## 8. Correction log

- 2026-10-28 — appended maximization checklist. Prior claim set stands.

- 2026-10-28 — initial entry; no prior claims in this file superseded. (AUDIT corrections C-AUD-0/2/5 and Theorems T1–T5 stand and are incorporated, not duplicated.)
