# Hyperbolic KV Benchmark - C++ Baseline (FW-A1-CUDA)

## Overview

C++20 baseline implementation with hand-rolled SIMD (NEON/AVX2) and inline PTX for Tensor Cores.
This provides the comparison baseline for the FW-A1-CUDA ablation study.

## Workload Specification

Same as CUDA Rust version:
- Vectors: 1,000,000
- Dimensions: H³(3) + T²(2)
- Fixed-point: Q32.31 with i128 accumulator
- Operations: insert, k-NN search, evict, Picard reduce

## Building

### Prerequisites
- CMake 3.28+
- C++20 compiler (clang-17 / gcc-13)
- CUDA 12.x / 13.3 toolkit (for Picard reduction kernels)
- NVIDIA GPU: H100 (sm_90) / B200 (sm_100) / RTX 5090 (sm_120)
- Ninja build system

### Build Commands

```bash
cd ablation_benchmarks/metal_cpp
mkdir build && cd build
cmake -G Ninja -DCMAKE_BUILD_TYPE=Release ..
ninja
```

### Build with Specific Architecture

```bash
# For H100 (sm_90)
cmake -DCMAKE_CUDA_ARCHITECTURES=90 ..

# For B200 (sm_100)
cmake -DCMAKE_CUDA_ARCHITECTURES=100 ..

# For RTX 5090 (sm_120)
cmake -DCMAKE_CUDA_ARCHITECTURES=120 ..
```

## Running Benchmarks

```bash
# Run full workload benchmark
./bin/hyperbolic_kv_bench_cpp

# Run with NCU profiling
ninja ncu_profile
```

## Output Artifacts

- `ablation_results/FW-A1-CUDA/cpp_raw_data.bin` - Raw measurements (binary format per FW-1)
- `ablation_results/FW-A1-CUDA/build_log.txt` - Build log
- `ablation_results/FW-A1-CUDA/ncu_log.txt` - NCU profiling log

## SIMD Implementation Notes

### Hand-rolled NEON (Apple Silicon)
- Uses ARM NEON intrinsics via `<arm_neon.h>`
- Vectorized Minkowski dot product (4 lanes)
- Hyperbolic distance with scalar fallback for `arcosh`

### Hand-rolled AVX2 (x86_64)
- Uses AVX2 intrinsics via `<immintrin.h>`
- 4x64-bit integer operations for Q32.31
- i128 accumulator emulated with 64-bit parts

### Inline PTX for Picard Reduction
- Warp-level reductions using `__shfl_down_sync`
- Custom matrix operations for PSL(2, Z[i]) action
- Cancellation-free x0 + x3 computation

## Kernel Fusion Rules (per FW-A1-CUDA.yaml)

1. **Lorentz boost + VP-tree search** → single kernel
2. **TwisterAttention + output projection** → single kernel  
3. **n-gram draft + verification** → speculative kernel pair

Verification: Manual PTX inspection required.

## Judge Criteria (FW-A1-CUDA)

| Metric | C++ Baseline Target |
|--------|---------------------|
| Throughput | Denominator for ratio (≥ 0.95 parity) |
| Peak Memory | Denominator for ratio (≤ 1.05) |
| Build Time | ≤ 120 seconds |
| Binary Size | ≤ 10 MB |

## Cross-Architecture Replication

Required replication across NVIDIA architectures:
- sm_90 (H100)
- sm_100 (B200)
- sm_120 (RTX 5090)

## Statistical Design

Same as CUDA Rust:
- α = 0.05, power = 0.80
- Lan-DeMets alpha spending (ρ=2)
- 4 interim analyses
- Sample size: 30 per group

## References

- FW-A1-CUDA Pre-Registration
- C05 Atlas/Hyperbolic KV (kernels)
- C12 Server Inference Engine (kernel logic)
