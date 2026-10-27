# Microkernel Build & Certification — Cross-Platform (macOS / Linux / Windows) + Cloudflare

**Date.** 2026-10-27. **Author.** Muse. **Status.** PROPOSED build + cert spec. **Parent.** HyperbolicMicrokernelPlugin-Spec (Level-1 fused / Level-2 fanned), TorodialPhasePositionalEncoder-Spec, TwisterPhPE MATH FREEZE, VaultMapDreamstateFormalization.
**Scope.** How every microkernel builds, proves itself, and ships on four targets: Apple Silicon macOS (Metal + AMX), Linux (CUDA-capable + CPU), Windows (CPU + DirectML-optional), Cloudflare Workers (WASM, CPU-only, no GPU). The steward's rule: **no platform dependence to function; at runtime, probe the system and use only kernels that work there.**
**Standards.** Documentation-standards (labels); research-evidence; C03.3 (no vault file modified; this spec constrains future code).

---

## 0. Decision (math, not preference — same rule as the level split)

Two candidate strategies: (A) single portable implementation everywhere (pure Rust, no accelerators); (B) per-platform accelerated kernels behind a runtime capability probe with automatic fallback. Decided by the same dependency analysis used for the level split:

- The vault ALREADY implements (B) in miniature: `ops.rs::matvec` → AMX on macOS (`#[cfg(target_os)]` + runtime row/col guard) → 8-way unrolled SIMD fallback; `lock.rs::kuramoto_step_cpu` labeled explicitly "(for testing; Metal does the real work)"; Metal kernels carry CPU oracles (`metal_quant.rs` test compares Metal output vs dequantized CPU reference, PASS < 1e-4). The codebase's revealed preference — measured, not asserted — is oracle + accelerator + harness.
- (A) discards the measured 9361×-class advantage (toroidal O(D) state vs O(N·D) KV) that the benchmark report documents, and the AMX dispatch the tree already paid for. Uniformity would be preference for symmetry, not math: the workload is data-parallel reductions + fixed-point trig, which is exactly what accelerators exist for.
- **Frozen: (B).** One Rust oracle per kernel (ALWAYS compiled, every platform — the determinism anchor); zero or more accelerator backends per kernel (Metal / CUDA / DirectML / WASM-SIMD / AMX / NEON / AVX2 / scalar); a runtime probe selects at load; the harness certifies every (kernel × backend) pair independently.

## 1. Capability probe (the steward's "gauge what system they are on")

At startup (and on backend failure — see §4), exactly one probe runs per accelerator class. Probe ≠ benchmark: it answers "does this work here," not "how fast."

| Class | Probe | Pass criterion | On fail |
|---|---|---|---|
| Metal (macOS Apple Silicon) | Compile embedded MSL → run `q4_gemv` 16×64 golden → compare vs oracle | max abs err < 1e-4 (existing gate) | Disable Metal backends; log receipt |
| AMX (macOS) | `accelerate_matvec_f32` on 64×64 golden vs scalar | bit-exact (same instruction path, deterministic) | Scalar fallback (already in `ops.rs`) |
| CUDA (Linux/Windows+NVIDIA) | `cudaDriverGetVersion` + PTX JIT of kernel → golden vs oracle | abs/rel err < backend tolerance (preregistered per kernel) | Disable CUDA backends |
| DirectML (Windows, optional) | DML device create + dispatch golden | Same tolerance gate | CPU backends only |
| WASM-SIMD (Cloudflare) | `wasm-feature-detect-simd` at load | Feature-present | Scalar WASM (always present) |
| CPU SIMD (all) | `std::is_x86_feature_detected!` / arm `neon` / `std::arch` | Dispatch widest available | Scalar unrolled (always present) |

Probe results are receipted (`backend_probe_receipt`: platform, backend, golden hash, pass/fail, timestamp) and pinned into the build receipt (ADR-23). A backend that was never probed PASS on THIS machine never dispatches — no hopeful dispatch, ever.

## 2. Backend matrix (what builds where)

| Kernel (Level) | Rust oracle (all) | Metal (macOS) | CUDA (Linux/Win) | CPU-SIMD (all) | WASM (CF) |
|---|---|---|---|---|---|
| `tpe.evolve` (Fused L1) | `torus.rs::evolve_vector` — REQUIRED | `kuramoto_tile.metal` EXTENDED (unwrapped accounting + radius/rapidity; current kernel covers the step only) | NEW PTX (from same MSL logic via shared codegen — §6) | NEON/AVX2 fixed-point Kuramoto loop | Scalar (CF path; bounded n) |
| `tpe.coherence` (Fanned L2) | `map_phase_coherence` + radial variant — REQUIRED | NEW MSL (pairwise, embarrassingly parallel) | NEW PTX (same codegen) | SIMD cos kernel | Scalar |
| `fterm.geo` (L2) | `hyp_distance` / inner product — REQUIRED | NEW MSL (arcosh + cosh-dist) | NEW PTX | SIMD | Scalar |
| `fterm.wigner` (L2) | `transport_frame_hop` math — REQUIRED | CPU-first (SO(3) small-matrix; Metal iff profiled hot) | CPU-first | CPU | CPU |
| `fterm.epistemic/token` (L2) | scalar formulas — REQUIRED | CPU (never GPU — control-flow-free but launch-dominated) | CPU | CPU | CPU |
| `fterm.compliance` (L2) | `OverlapPolicyMatrix` — REQUIRED | **CPU ONLY, all platforms** (policy is branchy audit logic; GPU placement forbidden by design) | CPU | CPU | CPU |
| `attn.linear` (L2) | Katharopoulos recurrent state — REQUIRED | NEW MSL (recurrent scan) | NEW PTX | SIMD scan | Scalar scan |
| `gemv.q{4,5,6}` (L2) | `metal_quant.rs` CPU ref — REQUIRED | EXISTING (`quant_gemv*.metal`, verified < 1e-4) | NEW PTX (quantized GEMV is the highest-value CUDA port) | AMX/NEON/AVX2 | Scalar (quantized) |

"Fused" note: Level-1 `tpe.evolve` stays ONE kernel per platform (the math forbids splitting — spec §0). Cross-platform parity means each backend implements the WHOLE evolution, verified against the same oracle.

## 3. Single-source codegen (how "same microkernel" stays same across Metal/CUDA)

New MSL is authored once in a restricted subset (no Metal-only intrinsics in shared paths; `thread_position_in_grid` ↔ `blockIdx*blockDim+threadIdx` mapping table frozen in the spec appendix). CUDA PTX is generated from the same logical source, not hand-rewritten: shared algorithm text + per-target emitter. The emitter itself is tested (emit-then-verify: PTX output for golden kernels must match checked-in reference PTX byte-for-byte, else the emitter changed behavior). This is what makes "different implementations of the same microkernel" certifiable rather than hopeful: N implementations, ONE oracle, pairwise receipted.

## 4. Runtime selection + degradation (no platform dependence to function)

```
select_backend(kernel):
  for b in [preferred ordered by probe]:
    if probe_receipt[b].pass and b.enabled_in_manifest: return b
  return scalar_oracle  // ALWAYS exists, ALWAYS certified (it IS the spec)
on_backend_failure(b, kernel, error):
  quarantine (b, kernel) for session; emit receipt; fall back to next backend
  if all backends quarantined: scalar_oracle (function preserved, speed degraded)
```

Degradation is a RECEIPTED event, not a silent slowdown: every fallback emits `backend_degraded(kernel, from, to, reason)`. The agent-facing contract never changes (same inputs, same outputs within tolerance); only latency changes. Kill-switch analogue: `--force-backend=scalar` reproduces any run deterministically on any machine (C03.5 replay across heterogeneous hardware — the cross-hardware replication story for C09.4).

## 5. Cloudflare deployment (CF = fourth target, most constrained)

- CF Workers runs WASM: scalar + WASM-SIMD backends ONLY. No Metal/CUDA/AMX. The oracle IS the production path there (with SIMD where probed).
- Model implication: CF serves the orchestration + retrieval + carve path (stateless, cacheable); heavy kernels (GEMV, evolution at scale) stay on Mac/Linux/Windows endpoints. The plugin manifest marks each backend `edge_eligible: bool` — ineligible backends are compile-excluded from the WASM build, not runtime-disabled (smaller binary, no dead code).
- Download story (Mac/Linux/Windows): one installer per OS ships ALL backends for that OS compiled in (Metal+AMX macOS; CUDA+AVX2 Linux/Windows; scalar everywhere). First launch probes, pins the working set, receipt goes to the local ledger. No network needed post-install (local-only core loop, C01 constraint honored).

## 6. Certification ladder per (kernel × backend)

1. Oracle exists, deterministic, seeded (REQUIRED for all).
2. Golden test: backend output vs oracle within preregistered tolerance (1e-4 class for GEMV; per-kernel tables for TPE/attention — TPE spec §5 gates 11–15).
3. Fuzz: 10k randomized inputs, zero violations (NaN, non-finite, OOB write).
4. Cross-backend agreement: every enabled backend pair agrees within 2× tolerance on goldens (catches emitter divergence).
5. Perf gate: backend must beat scalar-oracle on latency at equal tolerance on the fixed workload, else ships DISABLED for that backend (the steward's falsifier, per-backend).
6. Receipt: `(kernel, backend, oracle_hash, msl/ptx_hash, tolerance, golden_hash, probe_receipt, date)` → build receipt (ADR-23) + claims register.

## 7. Register entries (append — owning agent; none applied here)

1. Preregister per-kernel tolerance tables + golden corpora (C09.9) before any new MSL ships.
2. Extend `kuramoto_tile.metal` to full `tpe.evolve` (unwrapped + radius/rapidity) OR split kernel with receipted boundary — decide by profiling, record the decision.
3. Shared MSL→PTX emitter + checked-in reference PTX for golden kernels.
4. `edge_eligible` manifest flags + WASM build that excludes ineligible backends at compile time.
5. First-launch probe + pin flow for Mac/Linux/Windows installers; `--force-backend=scalar` replay flag.
6. Per-(kernel×backend) receipts in the build receipt; degraded-dispatch telemetry (counts only).

## What was NOT checked

Full `metal/mod.rs` (869 lines) audit; `glimmer_kernels.metal` numerics; `optiq.rs` dequant paths; WASM build existence; DirectML API surface; NVIDIA driver-version matrix; installer/packaging work (out of scope — C10/C14 own it).
