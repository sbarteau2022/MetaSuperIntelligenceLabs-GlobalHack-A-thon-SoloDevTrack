# C06 — Pre-Registration A40 v1.1 (CANONICAL — supersedes v1)

**Status.** [PROPOSED] Pre-registered; no A40 execution has occurred. **Date:** 2026-10-27.
**Supersedes.** v1 (`Pre-Registration-A40-v1.md`, SHA-256 `af8892ce034a02a182df790f5b0e951f9ccab73467a55df1c4bbc9b4614d7e3a`): v1 preserved verbatim per ADR-19; results under the v1 hash are invalid for v1.1 and vice versa.
**What changed vs v1 (thresholds FROZEN).** Ordering ≥0.95, Spearman ≥0.90, validity 100%, repeatability exact, preservation ≥0.98, M 10× ratio — all UNCHANGED. v1.1 adds only the missing machinery: held-out/blinded G1 (C-A40-01), exact sibling construction + triple count (C-A40-02), M absolute floor + void rule + rubric (C-A40-03), minimum-N + power (C-A40-04), B0 arm (C-A40-05), frozen workload/hardware + aggregation (C-A40-06), cross-process repeatability (C-A40-07), replication + necessity (C-A40-08).
**Rule.** [PROPOSED] The SHA-256 of THIS file must be recorded in `Claims_Register.md` before any A40 run. Any later edit creates v1.2 + new hash (ADR-19, ADR-22). Protocol PDF is byte-frozen; v1.2+ may add an appendix, never rewrite the PDF.

## A40 hypothesis + arms (unchanged from v1, B0 ADDED)

**Hypothesis.** [PROPOSED] Unchanged from v1 (Δ_H passes G1, preserves within 2%/domain, arm C hits 10× graded reuse holding preservation; any other outcome reported as that outcome).

**Arms (FROZEN, v1.1).** [PROPOSED] A = Glimmer Q4 Euclidean, no map. **B0 = Euclidean Glimmer Q4 + map plugin, NO Δ_H (NEW — isolates map-context effect).** B = Δ_H retrain + same map as plugin. C = hypo-native retrain + multimodel map native to harness forward. D DROPPED as arm; ungated diagnostic only. Identical data, identical order, identical seed, greedy decoding, all arms. Δ_H effect = B − B0: if B ≈ B0 within noise while both preserve, the retrain contribution is UNPROVEN and B's reading downgrades to "system preserves knowledge; muscle-memory break unproven" (no gate earned by B).

## G1 triple independence + sibling construction (closes C-A40-01, C-A40-02)

- **Disjoint split.** [PROPOSED] Map tree T is split at the top level into T_place (placement fitting only) and T_test (G1 triples only), disjoint below the root. Placement operator receives node features with ancestry labels STRIPPED; triple sampler is a separate script with no access to embeddings. Overlap audit (zero shared node IDs between placement set and triple set) is logged per run; any overlap VOIDS G1.
- **Quad construction (exact).** [PROPOSED] For parent P at depth d with parent G: C = child of P (depth d+1); U = child of G with U ≠ P, i.e. uncle/aunt of C at depth d sharing grandparent G; X = node from a different top-level branch than P (no shared ancestor below root). Each quad (P,C,U,X) yields: ordering d(P,C) < d(P,X); sibling d(P,C) < d(P,U); radius monotonicity points (radius(P) < radius(C), depth d < d+1).
- **Count.** [PROPOSED] ≥120 quads, ≥4 depths, ≥25 quads per depth, frozen in the run manifest before any run. Spearman computed over all G1 nodes.

## M rubric + floors (closes C-A40-03)

- **M definition (deterministic judge).** [PROPOSED] Needed context spans are pre-registered in the workload manifest with expected content. Per span: 1.0 if served via traversal with ZERO re-prefill tokens (exact token match); 0.5 (near-miss) if served with ≤10% of span tokens re-prefilled; 0.0 otherwise. M = total credit / total spans. Judge is token comparison, greedy, no LLM grading.
- **Floors.** [PROPOSED] G3 passes iff M_C ≥ 10×M_A AND M_C ≥ 10×M_B AND M_C ≥ 0.50 absolute. If M_A < 0.05 the ratio gates are VOID and the absolute target (M_C ≥ 0.50) alone decides. Ratio-alone never earns G3.

## Minimum-N + power (closes C-A40-04)

- **N.** [PROPOSED] ≥150 base-correct items per domain. General pools ≥250 items/domain; base (Arm A) runs first; gate computed on the base-correct subset (must reach 150, else domain UNDERPOWERED → INCONCLUSIVE, not pass).
- **Power plan (PROPOSED, computed from assumed rates — not measured).** One-sided non-inferiority, margin δ = 0.02, assumed true preservation 0.995, 80% power, α = 0.05: n = (1.645+0.842)² × 0.995×0.005 / 0.015² ≈ 137 → round UP to 150/domain. Wilson 95% CI reported per domain; lower bound <0.90 with point ≥0.98 → flag UNDERPOWERED (INCONCLUSIVE). N grows BEFORE the run (preregistered amendment + new hash), never after.

## Frozen workload / hardware / aggregation (closes C-A40-06)

- **Domains (4, held-out general).** [PROPOSED] factual-QA, summarization-faithfulness, instruction-following, code-repair. General and novel tasks reported SEPARATELY, never averaged (protocol §5). Aggregation for "per domain, aggregated": MACRO-AVERAGE over domains (frozen).
- **Hardware (pinned).** [PROPOSED] Apple Silicon, ≥36 GB unified memory, Metal 3.0+. Run manifest records exact SoC model, RAM, macOS build, MLX version, model revision, tensor count (1,568 REPORTED — verified at pull per protocol §2 step 1). **Logging schema adds `hardware` field** (protocol §8 lacked it; required by C09 proof-artifact rule).

## Cross-process repeatability (closes C-A40-07)

- [PROPOSED] G1 test 5 + G4 run as TWO SEPARATE OS PROCESSES, same seed, same input order; ordering outputs byte-compared, must match exactly. Determinism pins logged: seed, greedy flag, batch order hash, thread count, MLX version. In-process re-run does not count. Any mismatch FAILS G1/G4.

## Replication + necessity (closes C-A40-08)

- [PROPOSED] Cross-hardware replication on a second DISTINCT machine (different SoC or RAM class) with frozen v1.1 + manifest before any claim leaves PROPOSED (C09.4, C06.13). Removal ablation: Δ_H disabled (zero delta / bypass) must collapse M and fail G1; if removal does not regress, necessity FAILS and no A40 claim advances to REQUIRED.

## Condition closure map

| Condition | Closed by (v1.1 section) | Finding retired |
|---|---|---|
| C-A40-01 | Disjoint T_place/T_test split + blinding + overlap audit | F-A40-01 (CRITICAL) |
| C-A40-02 | Exact quad construction + ≥120 quads / ≥4 depths | F-A40-04 (part) |
| C-A40-03 | M rubric + 0.50 floor + void-if-M_A<0.05 | F-A40-03 (MAJOR) |
| C-A40-04 | N≥150/domain + power plan + Wilson reporting | F-A40-04 (part) |
| C-A40-05 | B0 arm + B−B0 decision rule | F-A40-02 (MAJOR) |
| C-A40-06 | 4 named domains + hardware pins + `hardware` log field + macro-average | F-A40-04 (part) |
| C-A40-07 | Two-process repeatability + determinism pins | F-A40-05 (MINOR) |
| C-A40-08 | Replication + removal ablation | Ladder requirement |

## Dependencies / non-goals (reaffirmed from v1)

- [PROPOSED] A40 is a PARALLEL submission track (Glimmer 30B MLX Q4 base); does NOT change the Ares core-loop model and does NOT overwrite W in place. Run manifests state eviction/weight-copy/privacy effects per run (C06.14).
- [PROPOSED] A40 does not re-test A1/A4/A21/A38/A39. Oct 30 is a target, not a gate; Nov 4 9 PM ET deadline stands.
