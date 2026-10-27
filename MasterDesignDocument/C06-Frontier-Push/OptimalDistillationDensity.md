# Optimal Distillation Density — How Dense a Map Must Be

**Date.** 2026-10-27. **Author.** Muse. **Status.** PROPOSED formalization (measured mechanisms + derived bounds).
**Scope.** Steward's two questions: (1) optimal way to higher distillation density without redundancy, straight to raw capability; (2) in theory, how dense a map must be for this method. Model never disassembled (frozen weights read-only throughout — VERIFIED in code paths below).
**Standards.** Documentation-standards (labels); research-evidence; C03.3 (no vault file modified).

---

## 0. What "density without redundancy" means here (locked definitions)

- **Density D** = distillable knowledge per byte: `D = I(answer; M_T | q, task-set) / bytes(M)`. Not atoms-per-GB (that rewards duplication); mutual information per byte.
- **Redundancy R** = bytes removable with zero ΔCap on the task set. `R = 1 − (min-bytes-for-same-ΔCap / bytes)`.
- Optimal distillation = **maximize D subject to R → 0**, i.e. every stored byte earns its keep on the judge set.

## 1. The vault's existing anti-redundancy machinery (VERIFIED — read this pass)

Three dedup gates already operate, one per cell class:

| Gate | Location | Rule | Effect |
|---|---|---|---|
| Atom coalescence | `silo.rs::compile_deposit` (VERIFIED lines 1363–1409) | New deposit within `cosh ≤ 1.0000001` of an existing atom AND same invariant class → `salience_count += 1`, no new atom | Near-duplicate deposits reinforce instead of duplicating. Threshold is EXTREMELY tight (1e-7) — near-identical only. |
| Arc dedup | `silo.rs::record_trajectory_arc` (VERIFIED lines 1171–1196) | Same (from_atom, compiled_mlir_op, wigner_frame, chain_id) → return existing, upgrade certainty if higher; locked arcs (`idx < locked_arc_count`) immutable | Identical transitions merge; better evidence upgrades the survivor. |
| Entropy sieve | `distill_engine.rs` (VERIFIED): `H(p) ≤ 2.75 bits` else rejected | High-entropy teacher guesses never enter | Noise excluded at ingestion, not stored-then-pruned. |

**Gap the steward's question exposes.** [INFERRED — ends at the three gates] All three gates catch *near-duplicates* (same coordinate, same transition, pure noise). NONE catches *semantic redundancy*: two charts at distant coordinates covering the same reasoning function, or a rank-8 chart whose 8 directions could be expressed in 3. The vault dedups identity, not information. That is exactly where density leaks today.

## 2. The optimal path (PROPOSED — 4 mechanisms, ordered by leverage)

**M1. Rank-adaptive accretion (highest leverage).** MGS already computes `orthogonal_residual_ratio` per slot (ε = 0.08) — but the dream cycle accretes up to rank 8 unconditionally (`max_rank8_charts_to_complete`). Change: accrete slot r+1 ONLY if its residual ratio exceeds ε AND its teacher certainty exceeds the chart's current mean. A chart that saturates at rank 3 stops at rank 3 (saves 5 × ~3 KB = ~15 KB per chart — at 210k charts, up to ~3 GB). The rank distribution (90×R1 / 508×R8 in the sample) suggests most charts do NOT need 8 — measure per-chart marginal information (M-CAP-1 arm) and the ε-gate follows. *Falsifier: if rank-8 charts beat rank-adaptive on held-out recall at equal bytes, keep 8.*

**M2. Cross-chart semantic dedup (the missing gate).** For each new chart, query VP-tree for nearest existing charts; if `cosh d < δ_sem` AND receptor-sketch cosine > τ_sem AND tangent subspaces overlap (Grassmann distance between span(u) sets < τ_grass), MERGE (salience reinforce + certainty upgrade, same pattern as arc dedup) instead of minting. This is compile_deposit's atom rule lifted to chart semantics. Preregister (δ_sem, τ_sem, τ_grass); start from (1.001, 0.95, 0.1). *Falsifier: merge aggressiveness sweep vs A39 J2/J3 — over-merge shows as recall regression; the knee is the optimum.*

**M3. Distill trajectories, not weights, first (ordering rule).** Weight-mode distill (`distill_static_weight_modes`: strided chunk scan, 16-float payloads) is broad but shallow — it fills the map with modes whose task relevance is unknown. Trajectory distill (`record_step`/`record_continuation` with teacher hidden states) is narrow but deep — arcs arrive with chain_id, step_index, certainty, and an immediate retrieval use. Optimal order: trajectories FIRST (they define the address space the map will be queried on), weight-modes SECOND (fill gaps the trajectory census reveals), dream minting THIRD (bridge the remaining holes). Each stage's marginal ΔCap measured; stop when marginal ΔCap/byte < threshold. *This is "straight to raw capability": every byte enters through a path that already has a retrieval consumer.*

**M4. Minutes-scale throughput is a feature of the method, not an accident.** No disassembly = no fine-tune loop, no gradient steps, no optimizer states. Cost per chart ≈ one forward tap + projection + refraction + insert (all O(h) readers, no O(h²) updates). The vault's 604 MB in minutes on a laptop follows from read-only distillation. Preserve it: any proposed gate (M1–M3) must itself be O(h)-ish per decision or it breaks the economics that make this method viable. M2's VP-tree query is O(log n); M1's residual is already computed. Both qualify.

## 3. How dense must the map be (theory — lower bounds, not targets)

**Bound 1 (coverage).** For task set T with trajectory-diameter distribution, need ≥1 chart within retrieval radius R of every required reasoning step. If the task manifold has intrinsic dimension m and diameter L in H³: `N_charts ≥ (L/R)^m` (covering number). With R ≈ arcosh(1.05) ≈ 0.31 (QualityTarget Thorough) and typical L ≈ 2–4 (basin diameters observed): `N ≈ 10–10³ charts per domain-silo`. 12 silos → **10²–10⁴ charts minimum** for single-domain competence; cross-domain bridging multiplies by interference-safe pairs (~66 pairs, mostly empty). The CURRENT 17k heavy + 216k light census is 10–100× above this floor — consistent with broad-but-shallow weight-mode fill (M3 ordering not yet applied).

**Bound 2 (rank).** Per-chart rank r must satisfy: residual of the local Jacobian outside span(u_{1..r}) contributes < judge tolerance. For locally-smooth teacher maps, r = 2–4 suffices (top singular directions dominate); r = 8 is the insurance tail. Expected steady-state: mean rank ≈ 3–4, NOT 8. At mean rank 3.5: ~12 KB/chart → same 210k charts in ~2.5 GB instead of ~5.2 GB. **The map is currently ~2× over-ranked by bytes** (INFERRED from sample rank skew 508 R8 / 90 R1 + MGS ε-gate unused as a stop rule).

**Bound 3 (sufficiency for the method).** The method (carve → attend → execute → crystallize) requires: (i) anchor coverage (every task lands within R of ≥1 chart — Bound 1); (ii) trajectory connectivity (forward-closure reaches an answer path — needs arc density ≥ branching factor along the path, ~2–5 arcs/step observed); (iii) promotion throughput (dream minting ≥ coverage decay from eviction — else the window drains). Sufficiency = all three simultaneously; A39 measures (i)+(ii) jointly, M-CAP-1 measures (i), eviction-rate logging measures (iii). No single byte count suffices — density is a 3-vector, not a scalar.

**Bottom-line numbers (PROPOSED, conditional on M-CAP-1/3).** Single-domain competence floor: ~10³–10⁴ charts (~25–250 MB). Current vault: ~17k heavy charts (~420 MB) = broad multi-domain coverage at ~2× rank overhead. Optimal steady state per §2: same coverage in ~40–60% of current bytes via M1+M2, with M3 ordering ensuring every byte has a retrieval consumer. The "necessary density" for the method as such: **~10 MB per competent domain-silo at rank-adaptive precision** (order-of-magnitude, falsifiable by M-CAP-1).

## 4. Register entries (append — owning agent; none applied here)

1. M1 ε-gated accretion + per-chart rank histogram telemetry (replaces blind rank-8 fill).
2. M2 semantic-merge gate with preregistered (δ_sem, τ_sem, τ_grass) + merge-aggressiveness sweep vs A39.
3. M3 distillation ordering rule (trajectories → weight-modes → dream) + marginal-ΔCap stop rule.
4. M-CAP-1/2/3 (coverage probe, byte audit, Euclidean bakeoff) from the capacity doc — the falsifiers for every number above.
5. Rank histogram + bytes-per-chart in every build receipt (C15.7) — density becomes a tracked metric, not an anecdote.

## What was NOT checked

M-CAP runs (unrun); multi-machine replication; 998 MB trajectory vault; outcomes-ledger binary; full `native_predictor` line audit; CUDA portability (out of scope).
