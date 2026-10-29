# FW-A1-CUDA — Result Status (2026-10-29)

**Status: PROPOSED / DRAFT. No result filed. This directory is intentionally
empty of results — the file you are reading is the disposition, not a result.**

## Why empty (VERIFIED 2026-10-29)

- Pre-registration `MasterDesignDocument/C09-Ablations-Proofs-Claims/
  FrameworkAblationStudy/PreRegistrations/FW-A1-CUDA.yaml` is **DRAFT**;
  no `FW-A1-CUDA.hash` filed. Per C09.1 / C11.3 (no filed hash, no run —
  no exceptions), no `claim.json` may be filed from this tree yet.
- The judge requires Linux + NVIDIA GPU (H100 sm_90 / B200 sm_100 /
  RTX 5090 sm_120, CUDA 12.x/13.3). CUDA does not run on Apple Silicon,
  so this Mac can only ever produce CPU-fallback smoke, never the judge.
- Toolchain probe this Mac, 2026-10-29 (VERIFIED): `nvcc` ABSENT,
  `nvidia-smi` ABSENT, `tsc`/`bun`/`deno` ABSENT; `node` v26.9.0,
  `cmake` 4.4.3, `cargo` 1.99.0 present.

## What exists instead (VERIFIED 2026-10-29, not results)

- `ablation_benchmarks/cuda_rust/` — CPU-fallback stub; prior ledger
  S-FW-A1-CUDA-01 records 16/16 tests + scaled smoke numbers as a
  semantics check ONLY. Not a CUDA result; fallback numbers must never
  be presented as CUDA numbers.
- `ablation_benchmarks/metal_cpp/` — C++ baseline scaffold, PROPOSED
  only, unbuilt on this machine (needs Linux + CUDA + CMake build).
- Scaffold trees `cf_workers/`, `go_wasm/`, `metal_rust/`,
  `ts_frontend/`, `zig/` — `src/` empty (PROPOSED placeholders, no code).

## Required artifacts per pre-reg (all PENDING)

`raw_data.json`, `analysis.ipynb`, `claim.json`, `failure.json`
(if FAIL), `build_log.txt`, `ncu_log.txt`, `receipt.json`.

## Blockers, in order

1. Finalize + file the FW-A1-CUDA pre-registration hash.
2. Linux + NVIDIA hardware (sm_90/100/120) with CUDA toolkit.
3. Build both binaries (CUDA Rust cutile-rs/cuda-oxide + C++ baseline).
4. Run the full 1M-vector workload per the pre-reg statistical design.
5. Compute throughput_ratio / peak_memory_ratio vs the filed judges,
   then file `claim.json` (or `failure.json`) with evidence links.

## What this marker is NOT

Not a claim, not a pre-registration, not a run. Do not cite this file,
the CPU-fallback smoke, or the scaffold trees as FW-A1-CUDA evidence.
