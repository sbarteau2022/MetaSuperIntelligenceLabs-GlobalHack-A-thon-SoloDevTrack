# Hyperbolic Microkernel Plugin — Formalized Dyptech Capstone (Kernel Logic)

**Date.** 2026-10-27. **Author.** Muse (contributor lane), on steward's directive.
**Status.** PROPOSED specification. **Parent.** `VaultMapDreamstateFormalization.md` (geometry/dream/attention) + `TorodialPhasePositionalEncoder-Spec.md` (TPE) + frozen TwisterPhPE MATH FREEZE.
**Scope.** The kernel substrate as a **custom plugin**: model-agnostic, formally specified, deterministically validated — Rust oracle + Metal kernel + verify harness per microkernel. This is the "Formalized dyptech" the steward advances every agent's work with.
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no vault file modified).

---

## 0. The decomposition decision (math, not preference)

Steward offered two splits: **(A)** one microkernel per positional channel (θ / r / amp / coherence — "4D w/respect to time") vs **(B)** one microkernel per free-energy term. Adjudicated by data-dependency analysis [VERIFIED — `torus.rs::evolve_vector`, `toroidal_attention.rs` coherence fns, `superpositionholding.md` §2 read]:

- **Channel-split (A) loses on the dependency graph.** Radius needs rotations, rotations need phase steps, amplitude needs arc steps, coherence needs all three triples. The graph is a CHAIN: `evolve → (θ̃, unwrapped) → (r, amp, η) → coherence`. Splitting a chain into kernels forces 3 global-memory round-trips per step and breaks the single-pass fusion the vault deliberately built (unwrapped arc + shell radius + helical tangent computed in ONE pair loop). The freeze's telescoping identity (`A_{i,j} = θ̃_f(i) − θ̃_f(j)`) additionally REQUIRES one shared unwrapped trajectory — channel-split kernels would each need the same buffer, i.e. the evolution must happen once, upstream. **A is rejected as a kernel boundary (not as a concept — channels remain the score's factors inside one kernel).**
- **Gate-split (B) wins on independence.** The five F-terms (geo-surprise, Wigner-holonomy, epistemic, compliance, token-cost) are independent given the candidate edge: each reads `(q, u, v, frames)` and writes one scalar; the sum is order-free. True fan-out: 5 parallel reductions. The steward's "how they process the order of operations internally" maps EXACTLY here — per-term reduction trees, precision points, and accumulation orders are free variants, each validated against the Rust oracle.

**Frozen decomposition (two levels — what the math wants):**
- **Level 1 — temporal, FUSED.** ONE phase-evolution microkernel (Kuramoto step + unwrapped accounting + radius/amplitude/rapidity derivation). Must stay fused: its outputs are data-dependent projections of one trajectory.
- **Level 2 — spatial, FANNED.** One microkernel per F-term (5) + one per coherence kernel (raised-cosine triple, map-phase, radial-pair) + the existing GEMV family. Each independently schedulable, each oracle-validated.

---

## 1. Substrate inventory (VERIFIED — read this pass)

| Layer | Files | Role in plugin |
|---|---|---|
| Rust oracle | `metal_quant.rs` (626 lines; `MetalQuantGemm`, CPU-ref test), `metal_kernels/metal_quant.rs` (228), `ops.rs` (AMX `matvec` + SIMD fallback), `attention_backend.rs` (validated `AttentionInput` envelope), geometry `torus/bridge/lorentz/picard/fixed` | Deterministic reference: every kernel output must match its oracle within tolerance or the kernel does not ship. |
| Metal kernels | `quant_gemv{,_opt}.metal` (q4/q5/q6/generic GEMV), `kuramoto_tile.metal` (`kuramoto_step`, all-pairs CORDIC sin, pi-unit Q32.31 torus), `metal/glimmer_kernels.metal` (group-sums, qmatvec, swiglu), `metal_kernels/quant_gemv.{metal,air,metallib}` | Fast path. Precompiled `.metallib` (+ `.air` IR) is the Apple-Silicon analogue of the steward's "Cubin" — deterministic binary artifact, not hopeful JIT. |
| Verify harness | `metal_kernels/verify_metal.rs` (CPU-ref vs Metal, max-abs/rel error, PASS `< 1e-4`, deterministic golden inputs) + in-tree test `metal_q4_q6_and_q5_simdgroup_kernels_match_cpu_reference` | The existing oracle-vs-kernel gate. Plugin spec generalizes this triple to every microkernel. |
| Bench | `bin/bench_geometric.rs` (map write smoke), `bin/bench_glimmer_lm_head.rs` | Perf arm (timing, not correctness). |

---

## 2. Plugin interface (frozen)

```rust
pub struct Microkernel {
    pub name: &'static str,            // e.g. "tpe.evolve", "fterm.geo_surprise"
    pub level: KernelLevel,            // FusedTemporal | FannedSpatial
    pub oracle: OracleFn,              // Rust reference: (inputs) -> outputs, deterministic
    pub metal_source: &'static str,    // MSL text; compiled once, pinned as .metallib
    pub tolerance: Tolerance,          // per-output (abs, rel); from the prereg, not tuned post-hoc
    pub receipt: KernelReceipt,        // msl_hash + metallib_hash + oracle_hash + verify_log_hash
}
pub enum KernelLevel { FusedTemporal, FannedSpatial }
```

**Registration rule.** `register(mk)` compiles MSL → `.metallib`, runs the verify harness (oracle vs kernel on golden + fuzz inputs), and records the receipt. A kernel WITHOUT a passing receipt cannot be dispatched — dispatch checks `receipt.verify == PASS` (fail-closed, same class as C03.7 security-before-build).

**Dispatch rule.** Level-1 fused kernel runs once per step (single launch, trajectory buffer resident). Level-2 fanned kernels launch per candidate edge; order-free — the scheduler may reorder, batch, or split reductions. Any internal order-of-operations variant (different reduction tree, different precision staging) is a NEW kernel version requiring a NEW receipt (ADR-19, no silent substitution).

## 3. Microkernel catalog (initial, mapped to verified code)

| ID | Level | Computes | Oracle source (VERIFIED read) | Metal home (PROPOSED unless noted) |
|---|---|---|---|---|
| `tpe.evolve` | FusedTemporal | Kuramoto 2:1 step + unwrapped arc + `r(w)` + `η` + amplitude + helical tangent | `torus.rs::evolve_vector` | NEW MSL (`kuramoto_tile.metal::kuramoto_step` covers the all-pairs step; extend to unwrapped accounting) |
| `tpe.coherence` | FannedSpatial | `cos(θ_q−θ_a)·κ_r·κ_a` triple score (§4.4 TPE spec) | `map_phase_coherence` + radial variant (existing fns) | NEW MSL (pairwise, embarrassingly parallel) |
| `fterm.geo` | FannedSpatial | `ΔS_geo(q,u,v)` geodesic surprise | `hyp_distance` / `lorentz_inner_product_wide` | NEW MSL (arcosh + cosh-dist) |
| `fterm.wigner` | FannedSpatial | `ΔH_Wigner` normalized holonomy divergence | `holonomy.rs::transport_frame_hop` frame math | CPU-first (SO(3) small-matrix; Metal iff profiled hot) |
| `fterm.epistemic` | FannedSpatial | `ΔE_epistemic` teacher inaccuracy (`1/(1+H)`) | `distill_engine` certainty path | CPU (scalar per edge; launch overhead dominates) |
| `fterm.compliance` | FannedSpatial | `Φ_compliance/safety` barrier (+∞ veto) | `OverlapPolicyMatrix` + gates | CPU (branchy policy logic; NOT a GPU kernel — policy stays on CPU by design) |
| `fterm.token` | FannedSpatial | `C_token` spend + redundancy | tokenizer counts | CPU (scalar) |
| `attn.linear` | FannedSpatial | Katharopoulos `φ(Q)(φ(K)ᵀV)` recurrent state | `toroidal_attention.rs` state machine | NEW MSL (recurrent scan; the Hypformer-linear precedent, REPORTED title-only) |
| `gemv.q{4,5,6}` | FannedSpatial | quantized matvec | `metal_quant.rs` CPU ref | EXISTING (`quant_gemv*.metal`, verified `< 1e-4`) |

**Deliberate CPU placements** (`fterm.compliance/epistemic/token`, Wigner pending profile): a microkernel is a *validation boundary*, not a GPU mandate. Placing branchy policy logic on Metal to chase uniformity would be preference, not math — the dependency analysis puts irregular control flow on CPU and regular data flow on Metal.

## 4. Deterministic-not-hopeful (the steward's core demand, formalized)

1. **Oracle primacy.** The Rust oracle is the SPECIFICATION executable: kernel output `==` oracle output within preregistered tolerance on golden + seeded-fuzz corpora, or the kernel version is rejected. Hope ends at the harness.
2. **Binary pinning.** Compiled `.metallib` hash recorded in the receipt; dispatch loads the pinned binary, never recompiles at serve time (the "Cubin" property: deterministic bits, auditable supply chain).
3. **Fixed-point discipline.** All H³×T² arithmetic in Q32_31 with i128 accumulators (existing convention, VERIFIED in `lorentz.rs`/`fixed.rs`); float64 allowed ONLY in named, receipted reduction stages (energy RMS, rapidity log) with error bounds stated.
4. **Order-of-operations registry.** Each validated internal ordering (reduction tree, precision staging, pair-loop order) is a distinct receipted version. Two kernels computing "the same part" differently are DIFFERENT plugins until the harness proves them equivalent — then they share a receipt class. This is the steward's microkernel-validation idea, made mechanical.
5. **Falsifier.** [PROPOSED] If the Metal kernel cannot beat the Rust/AMX oracle path on latency at equal tolerance on the fixed workload, it ships OFF (C13.1) — acceleration is earned per kernel, not assumed per platform.

## 5. Register entries (append — owning agent; none applied here)

1. Preregister the per-kernel tolerance table + golden/fuzz corpora (C09.9) before any new MSL ships.
2. Extend `verify_metal.rs` pattern to `tpe.evolve` + `tpe.coherence` first (RoPE-replacement path is the critical path).
3. Pin current `.metallib` hashes into receipts; record the toolchain (`xcrun metal` version) per ADR-23.
4. Calibrated AA-02 guard (§2 of ruling) becomes `fterm.compliance`'s first policy unit test.
5. `kuramoto_tile.metal` CORDIC gain/table audit against `fixed.rs` LUT (two sin implementations must agree — property test).

## What was NOT checked

Full `metal/mod.rs` (869 lines) line audit; `glimmer_kernels.metal` numerics; `optiq.rs` dequant paths; `qwen_forward`/`glimmer_forward` layer code; multi-device (H100/CUDA) portability — Apple-Silicon-first per C01, CUDA mapping explicitly out of scope until cross-hardware replication (C09.4) demands it.
