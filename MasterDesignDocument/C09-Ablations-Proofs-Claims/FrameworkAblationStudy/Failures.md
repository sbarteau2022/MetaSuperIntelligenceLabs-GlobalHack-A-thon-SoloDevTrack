# C09-FW — Framework Ablation Study Failures

**Status.** LIVE — append-only log of failing ablations.  
**Parent:** C09-FW FrameworkAblationStudy/Design.md  
**Standard:** Per AGENTS.md §7 / C09.7 — **failures recorded as entries, not hidden**. A failure recorded is worth more than a success asserted.

---

## Failure Format

Each entry records a **FAIL** result with full root cause and correction:

```
## YYYY-MM-DD — Ablation FW-A{X} — [Component] — FAIL

**Failure ID**: C09-FW-A{X}-[short-name]
**Hypothesis**: [What we tested]
**Result**: [Measured outcome vs threshold]
**Root Cause**: [Why it failed — technical, not procedural]
**Correction**: [Specific fix — not "try harder"]
**Follow-Up Ablation**: [FW-A{X}-retry or new ID]
**Status**: OPEN | CORRECTED | WITHDRAWN
**Artifacts**: [Paths to raw data, analysis]
**Not Checked**: [Explicit gaps in failure analysis]
```

---

## Failures Log

*No failures recorded yet. Phase 1 ablations pending execution.*

---

## Template for First Failure (FW-A1 Example)

```
## 2026-10-28 — Ablation FW-A1 — Hyperbolic KV Kernels — FAIL

**Failure ID**: C09-FW-A1-kernel-throughput-fail
**Hypothesis**: C++ kernels achieve ≥1.5x throughput at ≤1.05x memory vs Rust MLX
**Result**: 
  - Throughput ratio: 1.12x (threshold 1.5x) — FAIL
  - Memory ratio: 1.08x (threshold 1.05x) — FAIL
  - C++: 47.2 M vec/s, 29.1 GB peak
  - Rust MLX: 42.1 M vec/s, 26.9 GB peak
**Root Cause**: 
  MLX graph capture + lazy evaluation (mx.eval) fused VP-tree search + Lorentz boost
  automatically, achieving kernel fusion parity with hand-rolled C++. 
  C++ manual fusion had register pressure on M3 Max (40 GPU cores) causing spill.
  MLX's unified memory + graph optimizer outperformed explicit SIMD for this workload.
**Correction**: 
  1. Redesign C++ kernels using MLX-style lazy graph capture (via metal-cpp + MTLComputePipeline)
  2. Focus C++ effort on Picard reduction (where MLX has no equivalent)
  3. Re-run FW-A1 with hybrid: MLX for VP-tree/Lorentz, C++ for Picard
**Follow-Up Ablation**: FW-A1-retry (hybrid MLX+C++)
**Status**: OPEN
**Artifacts**: 
  - ablation_results/FW-A1/raw_data.json
  - ablation_results/FW-A1/analysis.ipynb
  - ablation_results/FW-A1/failure.json
  - ablation_results/FW-A1/perf_stat.txt
  - ablation_results/FW-A1/metal_gpu_trace.gputrace
**Not Checked**: 
  - Whether C++ can match MLX with better kernel scheduling (MTLCommandBuffer optimization)
  - Energy efficiency (joules/token) — only throughput/memory measured
  - Compile-time kernel specialization via Zig comptime (FW-A2) as alternative
```

## F-FW-A1-CUDA-01 — Audit corrections to S-FW-A1-CUDA-01/02 claims (2026-10-27)

**Auditor**: independent rerun on same Mac (not taken on trust). **Verdict**: certified as scaffold + CPU-fallback semantics check only. NOT certified as a CUDA ablation claim.
**Disposition**: both ledger entries STAND with the corrections below; nothing withdrawn that was true, nothing retained that was overstated.

### Correction 1 — "Compiles clean" overstated [replaces S-FW-A1-CUDA-01 "compiles clean" phrase]
- **Wrong**: "compiles clean" implied warning-free.
- **Right**: 0 errors, **48 bin warnings** (unused imports, dead fields, `unused_mut`) — honest signal of scaffolding, not a clean tree. Gate 3 build must be warning-gated (`-D warnings`) before claim filing.
- **Status**: CORRECTED.

### Correction 2 — Self-distance tolerance misattributed [replaces S-FW-A1-CUDA-01 "self-distance quant tolerance 5e-4 documented" + my "asserts 0 for its exact case" summary]
- **Wrong**: implied the unit tests assert exact 0 while only the integration test carries tolerance.
- **Right**: `test_hyperbolic_distance` uses a true-H³ point and DOES assert exact `dist == 0` (passes — correct, that path is exact). The ~7e-5 figure is the *quantized-random-atom* self-distance seen in `test_full_pipeline` (`results[0].1 == 146543 Q32.31 ≈ 6.8e-5`), which is bounded by the integration tolerance `|dist| < 1e6 Q32.31 (≈5e-4)`. Both facts coexist: exact on-locus inputs give 0; quantized random atoms give ~1e-4-scale residue. Neither is an order-of-magnitude claim about all inputs.
- **Status**: CORRECTED.

### Correction 3 — "No user files touched" false as written [replaces S-FW-A1-CUDA-01/02 "No user files touched"]
- **Wrong**: stated absolutely.
- **Right**: `Successes.md` was appended **by design** (the ledger is the work product), plus new `ablation_benchmarks/` trees and `target/` build artifacts. No evidence of unrelated personal files touched, but that is not proof across the whole Mac — scope the claim to: no tracked files modified except `FrameworkAblationStudy/Successes.md` (append-only), new files confined to `ablation_benchmarks/` + `ablation_results/`.
- **Status**: CORRECTED.

### Correction 4 — "Judge ratio computable now" not verified [replaces S-FW-A1-CUDA-01 "judge ratio is computable the moment both binaries run"]
- **Wrong**: implied readiness.
- **Right**: `metal_rust/src/` is **empty** in this tree (verified); neither CUDA Rust nor C++ binary is built; no Linux+H100 run exists; `HKVB01` magic shared in source only. Computability is PROPOSED infrastructure, not a VERIFIED capability.
- **Status**: CORRECTED — downgraded to PROPOSED.

### Auditor-confirmed (no correction needed)
- Rust files exist; 16/16 tests rerun-pass (auditor numbers: insert 3.486, k-NN 0.008, evict 0.047, Picard 15094 K ops/s, 74 ms/iter — within noise of reported 3.479/0.008/0.053/15700/72.946 ms); C++ scaffold PROPOSED-only; BM-01 gap honestly recorded; ledger entries present at Successes.md lines 74/84.
- **What this does not prove** (agreed): no CUDA executed, no PTX, no Tile IR, no cuda-oxide toolchain, no parity vs C++, no FW-A1-CUDA claim. Linux+H100 full-scale run still required — exactly as the ledger says.
