# Microkernel Build & Certification Spec — Kuramoto/TPE Evolution (PATCH-03 line)

Date: 2026-10-27. Machine: M5 Pro, macOS 27.0.1, Rust 1.99.0, Metal 32023.921.
Steward directive: build on the laptop (not Zed sandbox), verify each kernel against the Rust oracle, iterate until the math stops giving gains.

## 1. What was built (vault paths, monster-hybrid-server repo)

| Artifact | Path in vault | Status 2026-10-27 |
|---|---|---|
| GPU kernel (mean-field, fused K) | `core_lm/src/kuramoto_meanfield.metal` | CERTIFIED (gate green 6/6) |
| CPU oracle (all-pairs, CORDIC) | `core_lm/src/lock.rs::kuramoto_step_cpu` | reference, unchanged |
| CPU mean-field (Euler reduction) | `core_lm/src/geometry/torus.rs::kuramoto_resultant_step` | PASS (<=1 LSB vs all-pairs, n=4..1024) |
| Gate test | `core_lm/tests/patch03_metal_verify.rs` | GREEN: circular <= 4 LSB + sectors bit-exact @ n=16/64/256 |
| CPU bench | `core_lm/benches/kuramoto_opt.rs` | mean-field 14.9x–6627x vs all-pairs (n=16–2560) |
| GPU bench | `core_lm/benches/kuramoto_gpu.rs` | 0.0x @n=16 (dispatch-bound) → 243x @n=2560 vs all-pairs |
| Retired reference | `core_lm/src/sin_lut_4096.inc` | retained for reference only; kernel takes no LUT dependency |

## 2. Measured numbers (VERIFIED this pass, no projection)

### 2a. CPU oracle chain (release bench, `kuramoto_opt`)

| n | all-pairs | mean-field | speedup |
|---|---|---|---|
| 16 | 6.0 us | 0.4 us | 14.9x |
| 64 | 100.8 us | 0.6 us | 167.9x |
| 256 | 1565.2 us | 2.4 us | 666.0x |
| 1024 | 25093.9 us | 9.2 us | 2712.9x |
| 2560 | 154416.2 us | 23.3 us | 6627.3x |

### 2b. GPU kernel (release bench, `kuramoto_gpu`, 200 dispatches, incl. dispatch)

| n | gpu/step | cpu mean-field | cpu all-pairs | gpu vs all-pairs |
|---|---|---|---|---|
| 16 | 213.1 us | 0.2 us | 5.8 us | 0.0x (dispatch-bound) |
| 64 | 248.8 us | 0.7 us | 99.5 us | 0.4x |
| 256 | 264.8 us | 2.5 us | 1572.6 us | 5.9x |
| 1024 | 396.6 us | 9.8 us | 24882.7 us | 62.7x |
| 2560 | 653.8 us | 23.2 us | 159113.3 us | 243.4x |

Reading: the GPU kernel loses below n=256 (dispatch ~200+ us dominates a sub-us CPU step) and wins above it, reaching 243x at n=2560. The crossover is dispatch, not math — batch K steps per launch (fused path, k_steps) to amortize.

### 2c. Gate (debug test, `patch03_metal_verify`, 6 consecutive greens 2026-10-27)

- Circular divergence <= 4 LSB @ n=16/64/256 (hists: [5,7,3,1] / [18,32,12,1,1] / [113,94,43,5,1]).
- 12-sector lock invariants bit-exact 16/16, 64/64, 256/256.

## 3. Proven irreducibles (the math stopped giving gains HERE — 5 rounds)

1. **Mantissa theorem**: fp32 has a 24-bit mantissa; phase is 31 bits. ANY float trig path has a ~64–128 LSB floor vs the integer oracle. Retired the `float(theta)` cast path.
2. **Compiler contraction**: Kahan/Neumaier/Knuth-2Sum all compiled to bit-identical naive (V1==V2==V3) WITH contraction. `#pragma clang fp contract(off)` (file scope) restores them.
3. **LUT quantization**: 12-bit LUT = ~12,000 LSB residual. Retired; cospi/sinpi path (terms <= 0.5ulp) dominates it.
4. **fp32-term floor**: per-term trig <= 0.5ulp is the irreducible residual (Metal has no double). Sum over n terms + chain maps to <= 4 LSB circular at n<=256. No accumulation scheme beats it (fixed-point accumulation PROVEN exact — the floor is the terms).
5. **Metric artifact**: linear wrapping_sub reported 4294967299 @n=256 on wrapped state; circular truth is 4. Gate uses circular distance + sector agreement.

## 4. Technique ledger (each VERIFIED by probe before adoption)

- hi/lo integer-exact phase split (`>>12` arithmetic + `lo = t-(hi<<12)`; the `(int)(t<<20)>>20` formulation was WRONG for 9/16 thetas — sign bug, caught by probe).
- cospi/sinpi with EXACT argument (hp = hi/524288, |hi|<=2^20) + ang_lo first-order correction.
- Fixed-point Q32 accumulation + split-K (re-quant at O(1) scale) + Newton-refined reciprocal.
- Dekker-2Prod tail (e_hi == 0 PROVEN — tail exact, residual is term trig).
- rint (half-even) final conversion.
- sincos argument-order trap documented (returns sin, writes cos — burned twice).

## 5. Open (NOT certified — next in goal order)

- `tpe.evolve` full (unwrapped+radius/rapidity) and `tpe.coherence` GPU kernels.
- Cross-platform backend probing (Metal/CUDA/W shoreshim + CPU fallback) per cross-platform design.
- Fused-K amortization bench (k_steps >> 1) to move the GPU crossover below n=256.
- A39 harness (agent-level capability claim) — still preregistered, untouched by this pass.

## 6. Corrections appended (not silently rewritten)

- 2026-10-27: gate tolerance history 2 -> 3 -> 4 LSB. Each change followed a PROVEN floor (never relaxed to pass — relaxed to DOCUMENT, with the sector-agreement assertion added as the load-bearing check).
- 2026-10-27: emulator (Python fp32 replica) DIVERGED from GPU by 53M LSB on one run — replica bug, not kernel bug. Debug-the-kernel-directly rule adopted (dump intermediates via probe kernels, never trust replicas).
- 2026-10-27: `>>19` vs `>>20` factor-2 phase error; `(int)(t<<20)>>20` sign bug (9/16 thetas); sincos arg-order (2x); `double` in Metal (unsupported); `long long` in Metal (unsupported). All caught by compile or probe, none shipped.

## 7. Next-kernel analysis (2026-10-27, steward goal order: tpe.evolve full, tpe.coherence)

Per-pair op census of `TorPhaseEncoder::evolve` (torus.rs:102): ~15 Q32_31 fixed-point ops
(integer-exact on Metal `long`, including the 256-entry fixed LUT-interp sin — portable
as a tiny Metal constant array) + ~10 f64 ops (2x sqrt, div, from_f64 for the amplitude
channel). The amplitude channel is the ONLY non-bit-exact part (Metal has no double;
f32 sqrt/div ~1ulp each, random-walk over pairs — softmax-irrelevant but not bitwise).

Decision (dependency analysis, no preference): build the TPE phase kernel FIRST with
bit-exact phases (fixed-point path, LUT ported) + f32 amplitude, certified phase-bitwise
+ amplitude-tolerance — BEFORE the coherence weights (25 f64 transcendentals/pair,
certified relatively 1e-6, attention-irrelevant precision). Rationale: phases are the
chain everything else hangs off (psi -> phi -> shell -> coherence); an inexact phase
kernel poisons all downstream certs, while coherence can NEVER be bitwise (f64 ln_1p /
powf) so it must be last. Cross-platform shim + fused-K amortization ride along.

## 8. TPE phase kernel (2026-10-27 — CERTIFIED, gate green 5/5 + compile-clean)

Kernel: `core_lm/src/tpe_phase.metal::tpe_evolve_fused` — GPU port of
`TorPhaseEncoder::evolve` (torus.rs:102). Gate: `core_lm/tests/tpe_phase_verify.rs`.

Measured (VERIFIED, M5 Pro, this pass):

| config | dtf | dts | dpsi | damp |
|---|---|---|---|---|
| n=16 k=1 | 0 | 0 | 0 | 7.5e-9 |
| n=16 k=8 | 0 | 0 | 0 | 6.0e-8 |
| n=64 k=4 | 0 | 0 | 0 | 3.0e-8 |
| n=256 k=2 | 0 | 0 | 0 | 1.5e-8 |

- Phases BITWISE (all-integer Q32.31 path incl. 256-entry fixed LUT-interp port).
- Amplitude within 6e-8 (f64 CPU vs f32 GPU fold; gate tolerance 1e-5, >150x headroom).
- Range proof: every product fits i64 (k*sin <= 2^60, frac*delta <= 2^53, psi via
  shift); plain arithmetic == saturating for runs < 1e6 steps (14 orders headroom).

Correction appended (not silently rewritten): first bring-up diverged 114762320 LSB
because the port used TPE_LUT_STEP = 52666253 (a hand-division error; correct
TWO_PI/256 = 13492913325/256 = 52706692). The error ratio dtf/dts = 2.0 exactly
pointed at the sin path; a Metal-vs-Rust idx probe isolated it to the STEP constant
(GPU idx 0 vs CPU idx 255 on negative diffs). Fixed, then phases went bitwise first try.
Lesson: port EVERY constant by evaluating the CPU expression, never by hand arithmetic.

## 9. Fusion round (2026-10-27 — TPE CERTIFIED fused; Kuramoto frozen-resultant DOCUMENTED)

TPE step-major fusion (`tpe_phase.metal`, drives step-major `df[s*n+id]`):
single-launch k-step trajectories, gate green with phases BITWISE at
n=16/k=8, n=64/k=4, n=256/k=2 (damp <= 6e-8 throughout). TRUE fusion was free
here — no cross-thread dep in the step (dependency analysis, not preference).

Kuramoto fused loop: MEASURED 48M LSB circular @k=8 (sectors 13/16). Root cause:
threads re-read step-0 global theta every step (no mid-dispatch visibility), so
k_steps>1 = K repeats of the frozen step-0 resultant (explicit Euler with frozen
coupling — a valid but DIFFERENT integrator, uncertified vs the oracle).
Documented on the kernel; k_steps=1 path stays CERTIFIED. TRUE fusion needs a
threadgroup-barrier v2 (resultant is global) — queued behind the coherence kernel
in dependency order (coherence weights are per-pair like TPE drives: no cross-thread
dep, so the coherence kernel fuses the same free way TPE did).

Correction appended: the TPE test initially looped k_steps=1 launches (which never
exercised the kernel's k-loop); the single-fused-launch form now covers it.

## 10. Backend probe (2026-10-27 — CERTIFIED, gate green + warm-dispatch bound)

Module: `core_lm/src/backend.rs` (wired as `pub mod backend`). Gate:
`core_lm/tests/backend_probe.rs`. `detect()` returns Metal on M-series macOS,
CudaShim on Linux, WasmShim on wasm32, Cpu otherwise — explicit, receipt-logged,
never silent (`assert_no_silent_fallback` fails the run on a quiet CPU fallback).

Measured (VERIFIED, M5 Pro): cold call 217–235ms (MSL JIT compile), warm call
447–660us (dispatch + readback). The 189ms compile-per-call was caught by the
receipt log itself and fixed with a `OnceLock<(Device, ComputePipelineState)>`
cache — same persistent-resource lesson as the metal_quant MTLBuffer cache.
Warm bound asserted in-gate (< 5000us).

Correction appended: the two-call cache test initially compared the oracle
(one step from theta0) against the TWICE-stepped probe state (failed at 4 LSB
bound by construction, not by kernel error). Isolated call #2 to a fresh clone.

## 11. Coherence kernel (2026-10-27 — CERTIFIED d<=128, gate green 3/3)

Kernel: `core_lm/src/tpe_coherence.metal::tpe_coherence_fused` — port of
`TorPhaseEncoder::evolve_vector` (torus.rs:168). Grid: one thread per token,
sequential pair fold (matches the consumer `step_and_readout`, which calls it
per token). Gate: `core_lm/tests/tpe_coherence_verify.rs`.

Measured (VERIFIED, M5 Pro):

| config | traj rel err (worst-lane) | bound |
|---|---|---|
| d=64 batch=4 | 2.33e-5 | 1e-4 |
| d=128 batch=8 | 7.35e-5 | 1e-4 |

- Rapidity/cumrot ~1e-7 (pure accumulates). h3/raw carry the trig floor.
- Certified scope d<=128: the f32-ha trig floor grows with d (ha O(5*d/2));
  d=256 measured 1.82e-4 — needs integer-pre-reduction as a separate cert.

Corrections appended (the honest log of a wrong turn):
1. h3 double-normalization: kernel normalized raw[0..2] by their own norm AFTER
   the CPU had already normalized them (rel ~1.0). Probe proved it: gpu raw[0..2]
   == cpu h3 to 4 decimals. Removed the second normalization.
2. Integer-pre-reduction + Cody-Waite detour: built a full Dekker/Cody-Waite ha
   path (reduction PROVEN dcos<=6e-6) while the live divergence never moved —
   proof the bug was downstream (#1), not trig. Reverted to naive f32 ha with
   the floor documented. Lesson: move only what the probe convicts.
3. Reciprocal typo (0.2575117758 vs 0.2575181127) + ab=5520 pathological frac
   (.49996, 1 ulp from .5): real issues found during the detour, fixed with
   remainder-guarded quotient — but moot after the revert; recorded for the
   d=256 cert when it comes.
4. s2 = sh/(2*ch2) division-by-near-zero: real hazard, fixed with direct
   sincos(ha*0.5) (ha O(1) post-reduction) — kept, since the naive path uses it.

Research ingest (same pass, steward-ordered RESEARCH FIRST):
- `papers/2511.17826` (TBIK): tree-invariant reductions across TP sizes; the
  threadgroup-barrier v2 for Kuramoto fusion IS a TBIK intra-device tree.
  Column-vs-row-parallel analysis dictates where determinism gates belong.
- `papers/2602.00182` (EigenAI): three deterministic-kernel invariants (§6.5)
  cited in all three gate headers; receipt schema (arch/seed/req/out hashes)
  implemented in `backend.rs`. Manifest lines appended (no duplicates).
- CUDA-Rust brief: tile = portable unit; Metal is the local verification of
  tile semantics (can't execute PTX/cubin on Mac); registry pattern
  (name→impl per substrate) is the next layer after the dispatcher; toolchain
  disagreement flagged, nothing pinned.
