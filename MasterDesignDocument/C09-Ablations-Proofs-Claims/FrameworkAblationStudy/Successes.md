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
