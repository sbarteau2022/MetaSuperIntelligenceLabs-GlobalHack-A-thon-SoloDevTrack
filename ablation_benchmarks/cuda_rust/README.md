# Hyperbolic KV Benchmark - CUDA Rust (FW-A1-CUDA)

## Overview

This benchmark implements the FW-A1-CUDA ablation workload for comparing CUDA Rust (cutile-rs Tile + cuda-oxide SIMT) against hand-rolled C++ SIMD for hyperbolic KV kernels on H³×T² manifold with Whitehead 12-vertex structure (Picard fundamental domain Γ₁₂).

## Workload Specification (from FW-A1-CUDA.yaml)

| Parameter | Value |
|-----------|-------|
| Vectors | 1,000,000 |
| Dimensions | H³(3) + T²(2) |
| Fixed-point | Q32.31 with i128 accumulator |
| Insert operations | 1,000,000 |
| k-NN search | 100,000 queries, k=32 |
| Evict operations | 100,000 |
| Picard reduce | 10,000 ops, 32 iterations |
| Benchmark iterations | 1000 |
| Warmup iterations | 100 |

## Architecture

### Two-Track CUDA Rust Implementation

1. **cutile-rs (Tile Track — REACH FOR FIRST)**
   - Stable Rust 1.89+, crates.io
   - JIT: Rust AST → Tile IR → cubin per NVIDIA arch (sm_90/sm_100/sm_120)
   - Partition ownership safety (exclusive output sub-tensor per tile)
   - Used in: Grout (Qwen3 engine), mistral.rs, Candle MoE kernels
   - Kernels: VP-tree distance, insert, evict, Minkowski dot, arcosh

2. **cuda-oxide (SIMT Track — ESCAPE HATCH)**
   - Custom rustc backend, nightly
   - AOT: Rust MIR → Pliron IR → LLVM IR → PTX
   - DisjointSlice + typed thread index + #[launch_contract]
   - Warp/cluster ops for Picard reduction (Matrix units)
   - Shared memory currently REQUIRES unsafe (NVIDIA: "active work")
   - Toolchain: `cargo oxide` (pinned stable 1.99.1 or nightly-2026-04-03)

### Shared Runtime
Both tracks use the same runtime stack:
- `cuda-bindings` + `cuda-core` + `cuda-async`
- Tile ↔ SIMT interop on same CUDA stream
- CUDA IPC + atomic refcount for mmap-equivalent persistence

## Building

### Prerequisites
- Rust 1.89+ (stable for cutile-rs)
- CUDA 12.x / 13.3 toolkit
- NVIDIA GPU: H100 (sm_90) / B200 (sm_100) / RTX 5090 (sm_120)
- Linux (CUDA does NOT run on Apple Silicon)

### Build Commands

```bash
# Tile track only (default, stable Rust)
cargo build --release --features cutile-track

# SIMT track (requires nightly + cargo-oxide)
cargo build --release --features simt-track

# Full (both tracks)
cargo build --release --features full
```

### Using cargo-oxide for SIMT kernels
```bash
# Install cargo-oxide
cargo install cargo-oxide --git https://github.com/NVIDIA/cuda-oxide

# Build with pinned toolchain
cargo oxide build --release --features simt-track
```

## Running Benchmarks

```bash
# Run full workload benchmark
cargo run --release --features cutile-track

# Run microbenchmarks with criterion
cargo bench --features cutile-track
```

## Output Artifacts

Per FW-1 (No JSON), results are written in binary format:
- `ablation_results/FW-A1-CUDA/raw_data.bin` - Raw measurements
- `ablation_results/FW-A1-CUDA/build_log.txt` - Build log
- `ablation_results/FW-A1-CUDA/ncu_log.txt` - NCU profiling log
- `ablation_results/FW-A1-CUDA/receipt.json` - Merkle receipt (only JSON allowed)

## Judge Criteria (from FW-A1-CUDA.yaml)

| Metric | Threshold | Description |
|--------|-----------|-------------|
| throughput_ratio | ≥ 0.95 | CUDA Rust / C++ throughput ≥ 0.95 (parity) |
| peak_memory_ratio | ≤ 1.05 | CUDA Rust / C++ peak memory ≤ 1.05 |
| compile_time_ratio | ≤ 1.5 | Build time ratio |
| tile_ir_autotune_correctness | PASS | JIT autotune correct across arches |
| partition_safety_verification | PASS | Compile-time aliasing rejection |
| simt_escape_hatch_functional | PASS | cuda-oxide Picard reduction works |
| shared_runtime_interop | PASS | Tile + SIMT on same stream |

## Hardware Requirements

| Role | Specification |
|------|---------------|
| Primary | Linux + NVIDIA H100 (sm_90) / B200 (sm_100) / RTX 5090 (sm_120) |
| Replica | Different NVIDIA arch for cross-arch replication |
| Mac Validation | M3 Max (Metal only - design/verify semantics, cannot execute PTX) |

## Cross-Architecture Replication

The ablation requires replication across NVIDIA architectures:
- sm_90 (H100)
- sm_100 (B200) 
- sm_120 (RTX 5090)

Tile IR JIT autotuning must produce correct results on all three.

## Statistical Design

- α = 0.05, power = 0.80
- Effect size assumed: Cohen's d = 0.3
- Lan-DeMets alpha spending (ρ=2)
- 4 interim analyses (25%, 50%, 75%, 100%)
- Futility threshold: conditional power < 0.2
- Sample size: 30 per group

## Effect Size Reporting

| Metric | Type | Method | Meaningful Threshold |
|--------|------|--------|---------------------|
| Throughput | Cohen's d | Bootstrap (10k) | d > 0.3 |
| Peak Memory | % change | Percentile bootstrap | > 5% |
| Autotune Portability | Ratio | Log-normal | > 1.2 |

## References

- CUDA Rust Research (27 Oct 2026): cutile-rs Tile track, cuda-oxide SIMT track
- C05 Atlas/Hyperbolic KV (kernels)
- C12 Server Inference Engine (kernel logic)
- C08 Framework Requirements (NF1, NF4, NF5)
- FW-A1 (Metal kernels baseline)
