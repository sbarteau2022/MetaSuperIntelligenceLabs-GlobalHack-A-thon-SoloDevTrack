# Distillation — Audit (Varying Models → Single Memory Space)

**Date.** 2026-10-27 (revised same-day after adversarial pressure test — corrections are entries, original findings preserved below). **Scope.** Design docs: `MasterDesignDocument/C06-Frontier-Push/ReasoningDistillationMapCarving.md`, `HowWeTurnModelDistillationToRawCapability.md`, `MethodAndMathSideBySide.md`, `OptimalDistillationDensity.md`. Code: `monster-hybrid-server/immune_system/implementation/src/distill_engine.rs` (read-only audit, C03.3: no vault file modified). **Status.** AUDITED, PRESSURE-TESTED, corrected findings below.

## What was found

- **Two-pillar engine.** `OpenWeightsDistillationEngine` exposes Pillar 1 `distill_static_weight_modes` (weight mode spectra → carved atoms) and Pillar 2 `distill_reasoning_chain` (multi-step trajectories → 0/1/2-cells). REPORTED (read from source).
- **Frozen weights invariant.** Teacher weights are `FrozenBaseWeights`; weights are never mutated. VERIFIED by doc (`OptimalDistillationDensity.md`: model never disassembled).
- **Knowledge minted.** Each static mode yields one atom via `keep_and_carve_with_origin` with a `WeightDistillationReceipt` (modes_scanned, atoms_minted, role_distribution, mean_retained_energy). VERIFIED shape.
- **Compile break (SUPERSEDED 2026-10-27 — see correction C-AUD-0 below).** ~~`atoma_immune` does not currently build (`distill_engine.rs:290,659` and `map_native.rs:1019,1359,1411,1655`); signature drift between `well()`/`classify_icosahedral_well`. REPORTED by prior ruling. Therefore **no current test receipt** can be admitted for the distillation suite (stale last-green only).~~

## Findings / corrections (proposed)

1. **Stride sampling wastes budget.** Pillar 1 iterates `mode_idx in 0..max_modes` with `chunk_idx = (mode_idx * stride) % total_chunks` (uniform stride). Under a fixed atom budget, picking modes by **descending spectral retained energy** (greedy top-k) yields higher mean retained energy per minted atom. Estimated gain: [PROPOSED — unmeasured; must be ablation-tested].
2. **Novelty gate not visible in the read path (REFUTED AS WRITTEN 2026-10-27 — see C-AUD-2).** ~~`distill_live_step_novel_only` exists, but `distill_static_weight_modes` does not call a novelty check before minting. Add `projector` novelty filter (repeated modes collapse into existing wells) to avoid swamp. PROPOSED.~~
3. **MGS orthogonality.** `C05.12`/carving methodology: tangent slots must be MGS-orthogonalized (ε=0.08). Verify the projector applies MGS; if not, add. PROPOSED (verify against code).
4. **Receipted judge gate.** Every minted atom must carry a judge pass; currently the receipt is emitted but not gated on a judge. PROPOSED invariant to enforce.
5. **Determinism (MOOT 2026-10-27 — see C-AUD-5).** ~~Mode ordering must be deterministic for reproducibility; stride sampling is deterministic but top-k must be stable-tied. PROPOSED.~~

## Knowledge-instilled improvement (proposal)

Increase "knowledge instilled per minted atom" by (a) greedy top-k by retained energy, (b) novelty gate, (c) MGS orthogonalization, (d) judge gate. Each is an independent ablation arm; none changes frozen weights.

## What was NOT checked

- Full read of `projector` and `keep_and_carve_with_origin` (not audited this pass).
- Build of `atoma_immune` (known broken); no tests run.
- No measurement of retained-energy distributions on a real teacher checkpoint.

## Pressure-test corrections (2026-10-27, VERIFIED by independent probe — read-only, scratch in `/tmp/distill_probe`)

- **C-AUD-0 (original finding WITHDRAWN).** `atoma_immune --lib` builds clean TODAY (`cargo check --lib` zero errors) and the full suite is green: **66 lib + 114 orchestrator passed, 0 failed** (hardening pass B, `ErrAndImprovements-AA.md` §6). The compile-break claim was stale at audit time. Consequence: distillation-suite test receipts are admissible NOW; staleness caveat lifted.
- **C-AUD-2 (finding 2 REFUTED as written, REPLACED).** Pillar 1 mints via `keep_and_carve_with_origin` (`distill_engine.rs:300-306`), which delegates to `keep_and_carve_novel_only` (`silo.rs:1308-1316`) — spatial-bucket + Lorentz-inner-product coalescence IS in the Pillar 1 path. Surviving sub-finding: `with_origin` discards the novelty boolean (`.0` at `silo.rs:1315`) and `minted += 1` is unconditional (`distill_engine.rs:312`) — coalesced repeats inflate `knowledge_atoms_minted` and are indistinguishable from novel atoms in the receipt. The concern (no novelty _accounting_) stands; the mechanism claim does not.
- **C-AUD-5 (finding 5 MOOT).** Pillar 1 is pure index arithmetic (`:242,248,250`), no RNG in file, carve path uses `BTreeMap` throughout, existing top-k sorts are stable-tied (`:575-579` + stable `sort_by`). Keep as invariant, not a fix.
- **CONFIRMED with numbers.** Finding 1 direction: stride mean 0.0204 vs greedy top-k 0.0741 (+0.0537, 3.63×) on Zipf-1000/chunks-64 simulation (magnitude PROPOSED — real spectral skew unmeasured). Findings 3 (zero MGS mentions in `refraction.rs`; only engine call site `:1736` is the safetensors path, not Pillar 1) and 4 (zero `judge` references in `distill_engine.rs`/`refraction.rs`; Pillar 1 has no entropy sieve either — Pillar-2-only) CONFIRMED.
- **New doc-side defects found and FIXED this pass.** (i) Template JSON had a 4-byte `and ` trailer — validator crashed on its own template (FIXED, now valid). (ii) Validator accepted empty roster, contradicting §4 hold point — now rejects `models == []` (VERIFIED: exit 1). (iii) Validator traceback on malformed JSON — now clean `invalid JSON` error (VERIFIED). (iv) `CONVERSION_MATH.md §3` validity condition (`Σ receipt_ok == atoms_minted`) references nonexistent receipt fields — see §7 below. (v) α units undeclared; `p_useful` unmeasurable without the missing judge gate — see §7.

## 7. Open calibration (revised — was §5)

- α (conversion coefficient) — measure via A/B (held-out). **Units must be declared first:** dimensionless iff Cap is a normalized score, else capability-points per (coverage × energy). BLOCKS calibration, not docs.
- p_useful — measure via judge coverage report. **BLOCKED on finding 4** (no judge in mint path → currently unmeasurable).
- δ (novelty threshold) — sweep in ablation.
- B_map budget — from C21.
- **Validity condition REWRITTEN (was §3 `Σ receipt_ok == atoms_minted`).** `WeightDistillationReceipt` (`distill_engine.rs:66-73`) has no `receipt_ok`, no per-atom receipts, no sha chain — and Pillar 1 cannot reject (`minted += 1` unconditional), so the old condition was vacuous. Interim gate until per-atom receipting lands: `knowledge_atoms_minted == modes_scanned AND role_distribution sums to minted` (structural sanity, both computable TODAY) — receipt fields exist, check is falsifiable. Full `receipt_ok` gating returns when finding 4 (judge) is fixed.

## 8. In-repo corrections 2026-10-29 (Zed agent; corrections are entries, originals preserved above)

- **C-AUD-6 (interim-gate fan-out correction).** The §7 interim gate (`minted == scanned`) FAILS on the filed 2026-10-28 run (1183 scanned → 4468 cells minted): one weight-mode fans out to many atoms, so strict 1:1 is the wrong check for Pillar 1. CORRECTED in-repo: `research-grade-harness/src/distill_judge.rs::DistillationJudge` now takes `atoms_per_mode: Option<f64>` (None = trajectory strict 1:1; Some(f) = weight-mode expected `scanned*f` ± tolerance) and gate 2 balances `novel + coalesced == MINTED` (not scanned). Evidence: 20/20 lib tests green incl. 2 new fan-out tests (`weight_mode_fanout_balances` on the filed 1183/4468 shape; `weight_mode_without_fanout_fails_loudly` proving unfiled fan-out FAILS, not passes). Filed-run recheck: G1/G2 PASS under filed fan-out for all 5 models; G3 (role_counts) + G4 (Rbar) UNFILED in `distillation_run_2026-10-28.json` — the run receipt does not carry them, so the run is NOT judge-certified. Next: file per-role counts + mean retained energy per run, or re-run with receipting.
- **C-AUD-7 (α units DECLARED).** `CONVERSION_MATH.md §5`: `[α] = [Cap]` — capability-points per unit of (coverage × retained-energy); dimensionless iff Cap is a normalized score. The A/B MUST state the rubric scale at prereg.
- **C-AUD-8 (§3 validity condition CORRECTED to match C-AUD-6).** `CONVERSION_MATH.md §3` now states the four-gate interim form with the fan-out-corrected gate 2, enforced by `DistillationJudge`. The old `Σ receipt_ok` line is superseded with reason (vacuous — nonexistent fields), not deleted.
- **Standing gaps (unchanged):** finding 3 (MGS in Pillar 1), finding 4 (no judge in engine mint path — the in-repo `DistillationJudge` gates RUNS, not the engine), C-AUD-2 surviving sub-finding (`minted += 1` unconditional in the vault engine, out of this repo).

## 9. Audit addendum 2026-10-31 (Zed agent; corrections are entries, originals above preserved)

Scope of this pass: `DistillationProtocol/` + `ablation_results/` ONLY (no docs/harness/KV/git writes beyond this addendum; no commits).

- **C-AUD-9 (roster/receipt drift, VERIFIED by re-reading both sides 2026-10-31).**
  (a) `muse_fingerprint_receipt.json` (model Muse-Glimmer, unprefixed combined `48a6c13c...`) has NO roster entry: the roster row `Muse-Glimmer` points at `muse_fingerprint_receipt.json` as its `fingerprint_source` and matches its bytes/shards exactly, but the sibling `muse_glimmer_fingerprint_receipt.json` duplicates the same model+bytes with the `sha256:`-prefixed form. Disposition: duplicate filename, not duplicate evidence — one model, two receipt filenames, identical shard lists. Recommend canonicalizing to one filename in a future pass (not done here).
  (b) Sweep model name `Nemotron-30B` (`ablation_sweep.py:31`, 3 of 9 rows) matches NO roster name: the roster carries `Nemotron-3.5-Lightning` (L=52, same layer count). The sweep consumed only `L` from the name, so numbers are unaffected, but the label is wrong. Disposition: rename to the roster name on the next sweep edit (not renamed here, to keep this pass evidence-only).
  (c) Fingerprint-string drift (prefix + content, VERIFIED full-compare 2026-10-31): only the Muse-Glimmer pair matches (modulo the `sha256:` prefix on one side). `Qwen3.6-35B-MoE`, `Qwen3.8-27B`, `Nemotron-3.5-Lightning`, and `QwQ-32B` roster strings match NO on-disk receipt `combined_sha256` (all four differ past the shared prefix). Roster fingerprints are therefore STALE for 4/5 models — re-fingerprint or re-pin before any run that verifies fingerprints.
- **C-AUD-10 (sweep evidence hardening, MEASURED 2026-10-31, Mac CPU-only sim).**
  `ablation_sweep.py` now carries three named PASS/FAIL threshold constants with 1-line rationales (`PROBE_GREEDY_STRIDE_RATIO_MIN=3.0`, `ROW_GREEDY_GE_STRIDE_REQUIRED`, `WALKS_MAX_EQ_L_MINUS_1_REQUIRED`); every output row carries `threshold_checks` + `verdict`. Re-ran: probe ratio 3.6260 >= 3.0 PASS; 9/9 rows PASS (`greedy>=stride`, `walks_max==L-1`). Per-row receipts filed `ablation_results/SWEEP-01..09/claim.json` + `SWEEP-PROBE/claim.json` (C09.9 `claim.json` shape; `simulation_only: true`). Numbers remain Zipf-simulation, NOT teacher-checkpoint measurements (pre-reg prediction 2, mid-heavy band 1.10x, still INFERRED/unmeasured).
- **C-AUD-11 (twin-fingerprint runner, MEASURED reference smoke 2026-10-31, Mac CPU-only).**
  New `DistillationProtocol/scripts/twin_fingerprint_run.py` (pure-Python reference projector, no learned weights, no GPU): 8-step fixed tape under fp_A/fp_B -> byte-identical coords (cosine 0.0), 8 distinct coords -> 8 atoms salience 2 each, identical ranking; in-repo identity grep over `DistillationProtocol/scripts` + `research-grade-harness/src` returns zero non-audit teacher-identity reads. Verdict PASS; receipt `ablation_results/TWIN-01/claim.json` + `scripts/twin_fingerprint_result.json`. EXCLUSION (recorded, not run): the vault-engine `distill_reasoning_chain` twin run is absent — engine source lives outside this repo (`monster-hybrid-server/` not present) — and anything needing `nvcc`/`nvidia-smi` (both ABSENT this Mac, verified 2026-10-31) was not attempted.
