# C06 — A40 Pressure Test (Euclidean → Hyperbolic Retraining Protocol)

**Status.** [VERIFIED by execution below] Pressure test executed 2026-10-27. **Date:** 2026-10-27.
**Target.** `Euclidean-Hyperbolic-Retraining-Protocol.pdf` (7 pages, SHA-256 `4bcca0fa5d2e4a4e2ff2ef0e9093067f63fd637fa3c0a42328a306be63d89472`).
**Method.** Full 7-page text extraction (10,505 chars), independent re-derivation of every math gate, 7 adversarial probes (P1–P7) + 5 edge tests (E1–E5), cross-check against repo rails (C05 claim boundary, C06.13 ladder, C09.9, GeometryClaimBoundary).

## VERIFIED math (recomputed, not quoted)

| Check | Recomputation | Result |
|---|---|---|
| Lorentz consistency `x0 = sqrt(1+\|s\|²)` | `s=[0.1,0.2]` → x0 = 1.0247; `s=[0.3,0.4]` → self-inner-product −1.0000000000000002 | VERIFIED — formula sound, fp floor ~2e-8 on self-distance |
| Hyperbolic distance ordering | `d(parent,[0.1,0])→(child,[0.2,0.05]) = 0.1106` vs `→(unrelated,[0.8,0.5]) = 0.7584` | VERIFIED — ordering discriminates on test vectors |
| Poincaré validity | `‖[0.1,0]‖ = 0.1 < 1`; near-boundary r=0.999 → d(origin,p)=0.8807, still valid | VERIFIED — §4 test 4 is computable; near-boundary growth confirmed shallow in low-dim probe |
| Preservation arithmetic | 0.80 × 0.98 = 0.784 — matches protocol example 78.4% | VERIFIED |
| Time reserve | 12+12+24+12+8 = 68 h | VERIFIED arithmetic; NOT a measured result (protocol itself states this) |

## Probe verdicts

- **P1 WHERE is Δ_H.** Protocol defines Δ_H as "hyperbolic-native delta parameterized by the hyperbolic graph" with Riemannian update/retraction/parallel-transport and a receipted separate versioning path — then trains with "full forward/backward pass through the stack" (steps 3–4, §7 "full forward"). [INFERRED] A full-stack forward/backward pass through Euclidean Glimmer weights with gradients into those weights is Euclidean SGD semantics; a point constrained to H that is read by/maximally compatible with Euclidean attention is a reader-compatible embedding, not a freed geometry. Fails the protocol's own §2 "not relabeling" standard UNLESS the run manifest proves: (a) which tensors are Δ_H vs P_gen-read, (b) Riemannian optimizer identity + retraction operator, (c) that no gradient step lands in W without a fuse receipt. Recorded as **F-A40-02** (B-confound) and condition C-A40-05 (add B0 arm or downgrade interpretation).
- **P2 Geometry labels laundered.** §4 inputs are "triples from the map tree" with ancestry/radius from the tree; the same tree family is placed in H (§2 steps 2–3). [INFERRED] If placement sets radius from depth, ordering + Spearman ≥0.90 pass by construction — the benchmark would verify the placement script, not a learned geometry. Fix is C-A40-01 (held-out taxonomy or blinded placement). **This is the highest-severity flag: an unblinded G1 is a vacuous gate.**
- **P3 10× gate feasibility.** M is a fraction (0–1). No absolute floor is specified. M_A = 0 → any M_C passes ("No log" analog: a zero denominator mints a pass). M_A = 0.01 → M_C ≥ 0.10 passes while serving 90% of context by re-prefill. Fix is C-A40-03 (absolute floor M_C ≥ 0.50; void-if-M_A<0.05). The bounded-accuracy disclaimer is VERIFIED present and correct.
- **P4 2% per-domain hiding.** "Per domain, aggregated" has no aggregation rule and no minimum-N. n=5 probe: 4/5 = 0.80 fails while one item flips the rate by 0.20 — small domains are volatility-dominated. Fix is C-A40-04 (≥50 base-correct/domain or Wilson lower-bound gating + power plan).
- **P5 Shard/batch contradiction.** [VERIFIED] Protocol text already contains the correction: per-shard steps are wrong-gradient; fuse-only is the legal per-shard operation (pp. 2, Steps). No new defect; certification checks run manifests for shard-shaped batches.
- **P6 Calendar.** [VERIFIED] Protocol target-vs-gate language (p. 6) satisfies C09/C06.13 — Oct 30 target, gates inviolable, Nov 4 9 PM ET deadline. 68 h reserve VERIFIED arithmetic; "not measured" disclaimer VERIFIED present.
- **P7 D-drop soundness.** [INFERRED] The D-drop reason (Poincaré point is a Euclidean-ball point; Euclidean ranking = reader choice) is geometrically correct. D as ungated diagnostic is the right call; no flag.

## Failure entries

- **F-A40-01 (CRITICAL).** G1 triple source and placement use the same map-tree family with no blinding statement. An unblinded run passing G1 proves the placement script, not a retrained geometry. Blocks G1 interpretation until C-A40-01 closes.
- **F-A40-02 (MAJOR).** Arm B bundles Δ_H + map context; preservation success cannot isolate the retrain's contribution. Blocks "muscle memory broken by retraining" reading of B until C-A40-05 closes.
- **F-A40-03 (MAJOR).** M 10× ratio with no floor and no void-if-zero rule is gameable from any weak baseline. Blocks G3 interpretation until C-A40-03 closes.
- **F-A40-04 (MINOR).** Sibling-of-parent construction, triple count, aggregation rule, workload names, hardware pins, near-miss rubric, power/N all underspecified. Blocks first run until C-A40-02/04/06 close.
- **F-A40-05 (MINOR).** Repeatability test as written passes trivially in-process; cross-process MLX determinism untested. Gated by C-A40-07.

## Compatibility cross-checks (repo rails)

- **C05 GeometryClaimBoundary.** [VERIFIED] Protocol makes no TwisterPhPE / sheaf-cohomology / H³×T² proof claim; its G1 is a relative-delta geometry check on Lorentz/Poincaré charts — the allowed class (roadmap/deformation deltas, not geometry proof). Compatible, no boundary edit needed.
- **C06.13 ladder / C09.9.** Protocol satisfies prereg-first, fixed gate order, no-skip, withdrawal-on-fail, null-reporting rules. C09.9 checklist is 5.5/8 (workload/hardware partial, M rubric + power open) — conditions C-A40-03/04/06 close the gap; hash-before-run enforced by `Pre-Registration-A40-v1.md`.
- **C06.14 interference.** Protocol never overwrites W in place (Δ_H separate, receipted) — satisfies the interference-statement requirement in principle; run manifests must still state eviction/weight-copy/privacy effects per run.

## Verdict

**CERTIFIED WITH CONDITIONS.** The protocol's math is sound, itsanti-gaming clauses (no API-token provenance, withdrawal-on-fail, fixed gate order, no calendar override, no hiding preservation loss in novel learning) are its strongest assets, and its known-unknown honesty (time unmeasured, per-shard correction already internalized, bounded-accuracy disclaimer) meets the documentation standard. It is accepted as ablation **A40** with claims **CC-015…CC-020** (all PROPOSED, all OPEN), runnable only after C-A40-01…C-A40-08 close, tracked in `Claims_Register.md`.

**What was NOT checked.** No Glimmer weights were pulled; 1,568-tensor count is REPORTED, verified at pull time. No training, ablation, or gate run occurred. MLX/GPU determinism across processes untested. External literature/novelty search not performed.
