# C06 — Successes

**Status.** OPEN ledger. **Date:** 2026-10-26.

[PROPOSED] A success is recorded only with a checker output or an inspectable receipt. Design completion is not recorded as an experimental performance success.

## 2026-10-26 — C06 contribution design set authored

- **Finding.** [VERIFIED by the final file-presence check] The C06 workstream contains a named minimum-frontier design (Continuity Capsule), a pre-run A38 registration, a claim register, a success ledger, and a failure ledger.
- **Evidence.** [VERIFIED by the final checker] Required-file result: `missing_files=[]`; design-reference result: `missing_design_refs=[]`; canonical A38-v1.1 SHA-256: `0078d3995c920329f67b89b04b3ae8951763402a7eea59f0a7ae3d9ee8920004`; preserved A38-v1 SHA-256: `28716ac077059c248a0720565dde032cc4341212a0ec82ab8543b4155a89eea8` (both filed in `Claims_Register.md`).
- **Consequence.** [INFERRED from the artifacts] The design is ready for implementation planning under the filed A38 registration. It does not establish that the mechanism works or performs better than baseline.

## Experimental Successes

[VERIFIED by workspace inspection] No A1–A38 ablation execution result, hardware replication result, or judge receipt exists in this workspace for C06. Therefore there are no experimental C06 successes recorded here.

## 2026-10-27 — A40 protocol pressure-tested and filed (design success, not experimental)

- **Finding.** [VERIFIED by execution] The Euclidean → Hyperbolic Retraining Protocol PDF (7 pp., SHA-256 `4bcca0fa…d89472`, filed byte-identical in workstream) passed math verification (Lorentz consistency, distance ordering 0.1106 vs 0.7584 on probes, Poincaré validity incl. near-boundary, preservation 0.80×0.98=0.784, 68 h reserve arithmetic) and compatibility checks (C05 claim boundary, C06.13 ladder, C09 proof-artifact rules satisfied on anti-gaming clauses).
- **Evidence.** [VERIFIED by file presence + recomputation log] `Euclidean-Hyperbolic-Retraining-Protocol.pdf`, `Pre-Registration-A40-v1.md`, `A40-PressureTest-2026-10-27.md`, claims CC-015…CC-020 in `Claims_Register.md`.
- **Consequence.** [INFERRED] The design is ready for condition-closure (C-A40-01…C-A40-08) and hash filing before any A40 run. It does not establish that hyperbolic retraining works or beats any baseline.

## 2026-10-27 — A40 v1.1 closes all eight conditions (design success, not experimental)

- **Finding.** [VERIFIED by file presence] `Pre-Registration-A40-v1.1.md` (canonical hash `ac991e6e…a125c`) supersedes v1 per ADR-19 with thresholds FROZEN and all conditions closed by construction: disjoint/blinded G1 split + overlap audit (C-01), exact quad construction ≥120/≥4 depths (C-02), M rubric + 0.50 floor + void-if-M_A<0.05 (C-03), N≥150/domain + power plan + Wilson reporting (C-04), B0 arm + B−B0 decision rule (C-05), 4 named domains + hardware pins + `hardware` log field + macro-average (C-06), two-process repeatability (C-07), replication + removal ablation (C-08).
- **Evidence.** [VERIFIED] v1.1 hash filed as FILED/CANONICAL in `Claims_Register.md`; v1 marked SUPERSEDED, PRESERVED; condition-closure map in v1.1.
- **Consequence.** [INFERRED] A40 is RUN-READY on filing — next step is the frozen run manifest, not more prereg text. F-A40-01…05 retired as blockers; they persist as history. No experimental claim is established.
