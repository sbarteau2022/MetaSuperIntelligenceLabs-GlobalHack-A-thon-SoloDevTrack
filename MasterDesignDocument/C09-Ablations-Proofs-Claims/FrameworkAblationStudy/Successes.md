# C09-FW — Framework Ablation Study Successes

**Status.** LIVE — append-only log of passing ablations.  
**Parent:** C09-FW FrameworkAblationStudy/Design.md  
**Standard:** Per AGENTS.md §7 — date everything, name files/paths/versions, state what was NOT checked.

---

## Success Format

Each entry records a **PASS** result with full context:

```
## YYYY-MM-DD — Ablation FW-A{X} — [Component] — PASS

**Claim**: [Claim ID] → VERIFIED
**Hardware**: [Primary + Replica]
**Workload**: [Name + parameters]
**Baseline**: [Language + config + metrics]
**Treatment**: [Language + config + metrics]
**Effect Size**: [Metric + value + CI]
**Bayes Factor**: [BF₀₁]
**Replication**: [Replica result]
**Artifacts**: [Paths]
**Not Checked**: [Explicit gaps]
```

---

## Successes Log

*No successes recorded yet. Phase 1 ablations (FW-A1, FW-A6, FW-A10, FW-A12) pending execution.*

---

## Template for First Success (FW-A1 Example)

```
## 2026-10-28 — Ablation FW-A1 — Hyperbolic KV Kernels — PASS

**Claim**: C09-FW-A1-throughput → VERIFIED
**Hardware**: 
  Primary: MacBook Pro M3 Max, 36GB (clang-17, rustc-1.82, mlx-0.21)
  Replica: Mac Studio M2 Ultra, 96GB (same toolchain)
**Workload**: hyperbolic_kv_bench v1.0 (1M vectors, H³×T², Q32.31, seed=42)
**Baseline**: C++20, clang-17 -O3 -march=native, hand-rolled NEON, jemalloc, mmap
  - throughput: 68.1 M vec/s (p50)
  - peak_memory: 27.4 GB
  - instructions: 1.2B per 1M ops
  - build_time: 98s
**Treatment**: Rust 1.82, mlx-0.21, portable_simd, jemallocator, memmap2
  - throughput: 42.3 M vec/s (p50)
  - peak_memory: 28.1 GB
  - instructions: 1.8B per 1M ops
  - build_time: 142s
**Effect Size**: 
  - throughput: Cohen's d = 1.42 [1.1, 1.8] (bootstrap 10k)
  - memory: -2.5% [-5.1%, 0.2%] (percentile bootstrap)
  - instructions: ratio = 0.67 [0.58, 0.78] (log-normal)
**Bayes Factor**: BF₀₁ = 47.2 (strong evidence C++ necessary)
**Replication**: Mac Studio M2 Ultra: 1.52x throughput ratio (within 20% of primary)
**Artifacts**: 
  - ablation_results/FW-A1/raw_data.json
  - ablation_results/FW-A1/analysis.ipynb
  - ablation_results/FW-A1/claim.json
  - ablation_results/FW-A1/perf_stat.txt
  - ablation_results/FW-A1/receipt.json (merkle: sha256:...)
**Not Checked**: 
  - Linux/H100 portability (optional replica)
  - Long-running memory fragmentation (>24h)
  - Kernel fusion algebraic verification (separate gate C12.3)
```

## S-FW-A1-CUDA-01 — CUDA Rust CPU-fallback harness compiles + 16/16 tests pass (2026-10-27)

- **Status**: VERIFIED (measured on this machine, `cargo test --features stub --lib`: 16 passed, 0 failed) [VERIFIED — test output above; ends at test binary]
- **What**: `ablation_benchmarks/cuda_rust/` — hyperbolic.rs (H³/T² Q32.31), vptree.rs, picard.rs (CPU fallback), toroidal.rs, workload.rs, bench.rs — builds `release` on macOS/ARM64 without CUDA toolchain (stub feature skips `cudart`/`cuda` link; CUDA link gated to Linux only)
- **Smoke run** (scaled 5k/500/500/100, 3 iters, release, M-series Mac CPU fallback): insert 3.479 M vec/s, k-NN 0.008 M vec/s, evict 0.053 M vec/s, Picard 15700.570 K ops/s, peak 0.002 GB, 72.946 ms/iter [VERIFIED — stdout above; ends at smoke config, NOT full 1M workload]
- **Not proven**: parity vs C++ (judge needs both binaries on Linux+NVIDIA); Tile IR JIT autotune (cutile-rs git auth blocked here); SIMT warp kernels (cargo-oxide toolchain absent)
- **Evidence**: `cargo check --features stub` clean (48 warnings, 0 errors); smoke stdout; restored full-scale defaults (`workload.rs`)
- **Known gaps recorded in code**: BM-01 (scaffolding Picard reducer not a projection; `verify_idempotence` returns false by design, `verify_contract` asserts termination+x0 bound); self-distance quant tolerance 5e-4 documented
- **Next**: Linux+H100 full-scale run of CUDA Rust + C++ baseline → compute throughput_ratio / peak_memory_ratio → file FW-A1-CUDA claim

## S-FW-A1-CUDA-02 — C++ baseline scaffold written (2026-10-27)

- **Status**: PROPOSED (written, uncompiled on this machine — needs Linux+CUDA+CMake to build) [PROPOSED — files exist, no build log]
- **What**: `ablation_benchmarks/metal_cpp/` — header-only C++20 VP-tree/Lorentz/Q32.31 in `include/hyperbolic_kv_bench.h`, `main.cpp` workload runner, `picard.cu` warp+batch kernels with `__shfl_down_sync` reductions, `CMakeLists.txt` (sm_90/100/120)
- **Next**: build + run on Linux+H100 alongside CUDA Rust; NCU profile (`ninja ncu_profile`); write `cpp_raw_data.bin`

## S-A39-RUNNER-01 — Canonical judge patch, working-tree review (2026-10-27)

- **Status**: VERIFIED (reviewed in working tree `~/Desktop/monster-hybrid-server`, uncommitted; `cargo check --bin ablation_runner` 0 errors; `cargo test --bin ablation_runner` 9/9 pass incl. `sha256_known_vector` == `ba7816bf…20015ad`) [VERIFIED — check/test output above; ends at this machine]
- **What reviewed**: `orchestrator/src/bin/ablation_runner.rs` (+227/-~40) + new `orchestrator/src/bin/judge.rs` (484 lines, `deterministic-judge-1.1`, std-only SHA-256/tokenizer/canonicalize + v1.1 graded judges 0..1000 mP + `terms_hash` comparability guard)
- **Claims confirmed true**:
  - Success gate is now judge-canonical: `pass → !fallback_used`, `not_applicable → non-empty`, else false — "non-empty = success" is gone from both arms
  - Retries fixed: `retries = attempt_count - 1` (control 0, treatment 0/1 by fallback), `superposition_interventions` reported separately — the old 118-was-retries conflation is eliminated in schema
  - `--no-warmup` skips even the neutral warm-up (learn_enabled=true contamination path closed); `--vault-copy` overrides the hardcoded live vault path; every row carries `vault_path`
  - Full decoded text + `judge_version/verdict/checks` + `decoded_sha256` in every row → run file re-judges itself
  - `retries: 1` on `Err` arms changed to 0 with `judge_verdict: error` — honest (an engine error is not a retry)
  - `gate_rejection_count: None`, `backtrack_count: 0`, `vram_peak_mb: None` left as null/0, NOT faked — matches the "still honestly not done" report
  - `vram_peak_mb` deprecated in favor of sampled RSS (`/bin/ps`, macOS) with the unified-memory caveat stated in code
- **Corrections to the report** (minor, none load-bearing):
  1. "Judge unit tests: 5/5" → **9/9** on rerun (5 original + 4 graded-judge tests: empty/loop/diverse-text/terms-hash). The 5/5 was presumably the pre-v1.1 count.
  2. `results/control.jsonl` + `treatment.jsonl` in `results/` are **stale pre-patch artifacts** (00:10, old schema, `retries: 1` on fail-closed errors) — NOT the smoke output. The reported smoke run (judge pass, 256 tokens, RSS ~211 MB) has **no run file found** in the tree; re-run to produce the first canonical-schema artifact before analysis.
  3. `err_receipt` + `JudgeReceipt.version/expected_sha256/canonical_sha256/first_diff/error` fields + `strip_loc`/`judge_normalized` full-receipt path are **dead code in the runner path** (runner stores flattened `judge_checks: Vec<String>`, not `CheckResult`s; `judge_task` maps normalized/exact to pass/fail + detail string). Tested but unwired — fine, but `analyze_a39.py` must parse the flat schema, not the `JudgeReceipt` struct.
  4. `CorpusTask.seed` field is read from JSON but never used for per-task seeding (task_seed = manifest seed + idx). Harmless but the name misleads.
- **Certified as**: canonical-judge infrastructure + metric-hygiene patch, ready for a real A39 run. **Not certified as**: an A39 result — no post-patch run file exists yet. The three remaining steps named in the report (frozen corpus, signed fallback, second machine) are agreed as the true blockers.

## S-A39-CORPUS-01 — Frozen 200-task judged corpus + domain_terms judge (2026-10-27)

- **Status**: VERIFIED (built + tested on this machine; `cargo test --bin ablation_runner` **12/12 pass** incl. 3 new `domain_terms_*` tests; `cargo check` 0 errors; release binary rebuilt) [VERIFIED — test/check output above; ends at this machine]
- **What**: `corpus_manifest.json` now carries per-task `judge: {check: "domain_terms", targets: [20 frozen domain stems], threshold: 1, case_sensitive: false}` on all 200 tasks (70 code / 70 sec / 60 math). SHA-256 `5e4b408b…` (was `5f46cdf6…` unjudged). Frozen copy at `/tmp/corpus_manifest_FROZEN_200_judged_20261027.json`.
- **Why this design** (audited against the N400 run files, not assumed):
  - Synth-prompt token echo rate = **0/5** sampled (engine never emits prompt content tokens) → `token_contains` on prompt vocab = permanent fail, REJECTED as judge.
  - Definiendum-head presence = perfect vocab-boundary split (6 heads 100%, 14 heads 0%) → zero cross-arm variance, REJECTED as judge.
  - Domain-stem hits = the live separator (control domain-mass 40 vs treatment 14) → adopted. Uniform threshold 1, no per-domain tuning, no quantile-picking from the same N400 it will be validated on (threshold logic documented as PROPOSED pending second-machine confirmation).
- **Offline re-judge of N400 through new logic** (same files, new verdicts): control **75/200 (37.5%)**, treatment **46/200 (23.0%)**; per-domain control 54.3/21.4/36.7 vs treatment 22.9/11.4/36.7. Math identical 22/60 both arms (null-domain control holds — PT-J-002 consistent). Direction is control-favorable: treatment must beat a judge tilted against it, which strengthens any future H1 claim rather than weakening it.
- **Runner wiring**: `CorpusJudge` gains optional `targets`/`threshold` (backward-compatible — old `target`-only blocks still parse); `judge_task` dispatches `"domain_terms"`; `judge::domain_terms_hits` is std-only, deterministic, re-judgeable from run file alone.
- **Not done**: corpus NOT pre-registered (hash `5e4b408b…` must be filed in PRE-REG before the real run); duplicate `corpus_manifest_200.json` NOT yet deleted; git disposition (track/ignore) NOT yet executed — all deferred per instruction.
