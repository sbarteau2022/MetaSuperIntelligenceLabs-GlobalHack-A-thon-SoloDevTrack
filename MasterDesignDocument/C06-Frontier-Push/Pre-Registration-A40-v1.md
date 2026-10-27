# C06 — Pre-Registration A40 v1 (Euclidean → Hyperbolic Retraining Protocol)

**Status.** [PROPOSED] Pre-registered; no A40 execution has occurred. **Date:** 2026-10-27.
**Canonical source.** `Euclidean-Hyperbolic-Retraining-Protocol.pdf` (7 pages, filed alongside as `Euclidean-Hyperbolic-Retraining-Protocol.pdf`, SHA-256 `4bcca0fa5d2e4a4e2ff2ef0e9093067f63fd637fa3c0a42328a306be63d89472`).
**Rule.** [PROPOSED] The SHA-256 of THIS file must be recorded in `Claims_Register.md` before any A40 run. Any later edit creates v1.1 + new hash (ADR-19, ADR-22). Protocol PDF is byte-frozen; v1.1+ may add an appendix, never rewrite the PDF.

## A40 — Hyperbolic-native retrain (A/B/C) with geometry acceptance + preservation + reuse gates

**Hypothesis.** [PROPOSED] A hyperbolic-native delta (Δ_H, Riemannian updates on H) passes deterministic geometry acceptance (G1), preserves general knowledge within 2% per domain (G2), and the native harness arm (C) achieves ≥10× graded reuse rate over both Euclidean (A) and plugin (B) arms while holding preservation (G3). Any other outcome is reported as that outcome.

**Arms (frozen).** [PROPOSED] A = Glimmer Q4 Euclidean, no map. B = Δ_H retrain + pre-distilled hypo map as plugin. C = hypo-native retrain + multimodel map native to harness forward. D DROPPED as arm (reader choice, not geometry); kept as diagnostic dual-rank overlap only. Identical data, identical order, identical seed, greedy decoding, all arms.

**Judges (frozen thresholds).** [PROPOSED] G0 provenance logged. G1: ordering accuracy ≥0.95, radius–depth Spearman ≥0.90, manifold validity 100%, repeatability exact match. G2: preservation ≥0.98 per domain AND aggregated. G3: M_C ≥ 10×M_A AND ≥ 10×M_B, with preservation still ≥0.98×A. G4: same seed/data/order, greedy; repeats within stated tolerance. Gate order fixed G0→G1→G2→G3.

**C09.9 checklist mapping.** [PROPOSED] Hypothesis ✓ (above). Workload PARTIAL — general task sets unnamed (see condition C-A40-06). Hardware PARTIAL — Apple Silicon implied, unpinned (C-A40-06). Baseline ✓ (Arm A). Judge band ✓ thresholds frozen; near-miss rubric for M open (C-A40-03). Effect-size plan ✓ (10× / 2%) but no power/N (C-A40-04). Null-handling ✓ (protocol §6 interpretation rules). Claims-register draft ✓ (CC-015…CC-020, filed herewith).

## Certification conditions (ALL must close before first run; harness/docs enforce, not intent)

- **C-A40-01 Triple independence.** G1 triples must come from a taxonomy HELD OUT from map placement, or placement must be blinded to ancestry. A placement algorithm that sets radius from depth passes G1 trivially (see `A40-PressureTest-2026-10-27.md`, F-A40-01).
- **C-A40-02 Sibling-test disambiguation.** "Sibling-of-parent" triple construction must be defined exactly (uncle/aunt node at same depth as parent, shared grandparent), with minimum triple count (≥100 triples, ≥3 depths).
- **C-A40-03 M-gate floors.** M_C ≥ 10×M_A AND ≥ 10×M_B AND M_C ≥ 0.50 absolute. If M_A < 0.05 the ratio gate is VOID and the absolute target (M_C ≥ 0.50) decides. Near-miss grading rubric for M must be filed (what counts as graded reuse vs re-prefill).
- **C-A40-04 Minimum-N + power.** ≥50 base-correct items per domain for G2 (or report Wilson 95% CI and gate on lower bound ≥0.98). File power plan (80%, α=0.05) with N before run; N grows before, never after.
- **C-A40-05 B-confound.** Add arm B0 (Euclidean + map plugin, NO Δ_H), OR downgrade B's interpretation to "system preserves knowledge" — B passing with map context cannot by itself prove muscle memory was broken by retraining (F-A40-02).
- **C-A40-06 Freeze the workload/hardware.** Name held-out general task sets, pin hardware + MLX version, add hardware field to logging schema (§8 currently lacks it per C09 proof-artifact rule). Aggregation rule for "per domain, aggregated" must be fixed (macro-average recommended).
- **C-A40-07 Repeatability across processes.** G1 test 5 + G4 must run as two separate processes (same seed) with MLX/GPU-determinism pins logged; in-process re-run is trivially exact and does not count.
- **C-A40-08 Replication + necessity before REQUIRED.** Cross-hardware replication (dev Mac + one other, C09.4) and a removal ablation (Δ_H disabled → M collapses, G1 fails) are required before any A40 claim advances past PROPOSED (C06.13 ladder).

## Dependencies / non-goals

- [PROPOSED] A40 is a PARALLEL submission track (Glimmer 30B MLX Q4 base). It does NOT change the Ares core-loop model (Qwen3.8-27B, README) and does NOT overwrite W in place (Δ_H trained separately; satisfies C06.14 interference statement in principle).
- [PROPOSED] A40 does not re-test A1/A4/A21/A38/A39. Tensor count "1,568" is REPORTED from the protocol, verified at pull time per §2 step 1.
- [PROPOSED] Oct 30 is a target, not a gate. Calendar never lowers a gate; 68 h reserve is VERIFIED arithmetic (12+12+24+12+8), not a measured result.
