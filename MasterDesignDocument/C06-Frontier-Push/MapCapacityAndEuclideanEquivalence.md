# Map Capacity & Euclidean Equivalence — Formalized Dyptech

**Date.** 2026-10-27. **Author.** Muse (contributor lane).
**Status.** PROPOSED formalization with MEASURED byte budgets. **Parent.** `VaultMapDreamstateFormalization.md` + `HyperbolicMicrokernelPlugin-Spec.md` + TwisterPhPE MATH FREEZE.
**Scope.** Two steward questions: (1) how much distilled knowledge fits in a fixed-size map; (2) what that costs in Euclidean parameters. Answers derived from vault bytes + `map_native.rs::encode_binary` + sampled payload census (n=297, seed 7).
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no vault file modified).

---

## 0. Measured byte budgets (VERIFIED — computed this pass from vault bytes + code)

Per-atom wire format (`encode_binary`, BF16 effectors, sketch 256-dim f32):

| Rank | Formula (bytes) | Value |
|---|---|---|
| Rank-1 spine | 36 + 256·4 + h·2·2 | h=512 → **3,108 B** (min observed 3,112 ✓) |
| Per extra slot (2..8) | 12 + 256·4 + h·2·2 | h=512 → **3,084 B** |
| Rank-8 total | 3,108 + 4 + 7·3,084 | **24,700 B ≈ 24.7 KB** (sample median EXACT ✓) |
| Rank-8 lossless f32 | 5,156 + 4 + 7·5,132 | **41,084 B ≈ 41.1 KB** |
| h=6656 outlier (1 sampled) | scaled by h | 221,308 B max observed |

Vault census: 603,865,908 B total; 17,006 `ATOMA_OP` payloads; sampled ranks 245×R8 / 52×R1, all v3-flagged, sketch always 256, h_dim 512 (296/297).

Composition (mixed label): 17,006 × ~24.7 KB ≈ **~420 MB (~70%)** charts [VERIFIED arithmetic on measured median]; ~216k 1-cell arcs × ~200 B struct ≈ **~43 MB (~7%)** [INFERRED — struct size × REPORTED audit count]; remainder ~140 MB KV/MLIR atoms + VP-tree index + ledger [INFERRED].

---

## 1. Fixed-size map capacity (the law)

**Capacity law.** For map budget S bytes and per-atom cost c (≈25 KB rank-8 BF16):
`N_charts(S) ≈ (S − O_index) / c`, with `O_index` = VP-tree nodes + HashMap overhead (≈100 B/entry in RAM, less on disk).

| Budget S | Rank-8 charts | Note |
|---|---|---|
| 604 MB (current vault) | ~24k capacity → 17k heavy + ~200k light residents | Consistent: lights average ~1 KB |
| 1 GB | ~40,000 charts | — |
| 8 GB (lane slice) | ~320,000 charts | Under C21 ceiling with headroom |
| 27.45 GB (peak cap) | ~1.1 M charts | Hard ceiling, never target |

**What "fits" means (honest semantics).** A fixed map is NOT an archive — eviction (max unpinned x₀) + `max_atoms` make it a **salience-weighted retention window** over the teacher manifold. Capacity in knowledge terms = N charts × neighborhood each, where each chart covers its basin radius. Coverage fraction (teacher behaviors within cosh-reach of an anchor) is MEASURABLE via Suite-1-style probe but UNMEASURED — proposed measurement M-CAP-1, not a claim. What is VERIFIED: the retention rule (evict max-x₀, pins survive) guarantees the window holds the highest-salience N, whatever N the budget buys.

**Temporal state is negligible beside knowledge.** Per-step trajectory state (θ_f, θ_s, r, amp + unwrapped accumulators) ≈ tens of bytes; a 1k-step trajectory ≈ tens of KB — three orders below one rank-8 chart. The map's bytes are knowledge, not history. (This is the 4D-wrt-time split made quantitative: evolution state is O(10¹) B/step; distilled operators are O(10⁴) B/chart.)

---

## 2. Euclidean equivalents (three honest conversions + one structural)

**(a) Byte parity.** 604 MB ≈ **302 M BF16 params** of raw bytes ≈ a 300 M dense model. Caveat stated: bytes ≠ capability — charts are local linearizations with provenance, not general weights. Parity is a budget anchor, not a capability claim.

**(b) Per-chart dense-Jacobian ratio (the tightest conversion).** Each rank-8 chart expresses a local linear map Δx = Σ_{r=1..8} σ_r⟨Π(x),u_r⟩ψ_r. Dense Euclidean equivalent: full h×h Jacobian at 2 B/entry = 2h² bytes.

| h_dim | Dense equiv | Chart (BF16) | Ratio |
|---|---|---|---|
| 512 (typical) | 524,288 B | 24,700 B | **~21× compression** |
| 6,656 (outlier) | 88,576,512 B | 221,308 B | **~400× compression** |

The rank constraint (≤8, MGS-orthogonalized, ε=0.08) is what buys the ratio — Euclidean pays full rank whether the local map needs it or not. [INFERRED from measured sizes + struct; ends at the byte census.]

**(c) Hierarchical addressing (structural, from literature).** Euclidean tree embedding needs unbounded dimension for arbitrary-low distortion (Nickel & Kiela §2, REPORTED); hyperbolic does it in 2–3 dims. The map's basin/silo addressing (radial salience + icosahedral sectors) is nearly free in H³ and exponentially expensive in E^d: a Euclidean equivalent needs explicit tree structures (pointer overhead, no metric continuity) or dimension blowup. This is the geometry-necessity argument from formalization §6, now with the byte denomination attached.

**(d) Serving amortization.** One 604 MB map + ONE frozen weight buffer serves all lanes/roles (base=1, knowledge-views=N — vault correction ledger, VERIFIED). Euclidean multi-specialist equivalent: N fine-tuned copies (N× weights) or N adapters + router. The map replaces N model copies with N projections — bytes scale as views, not models. At N=12 roles: Euclidean LoRA-class equivalent ≈ 12 × (adapters + router state); map cost = 12 × fiber views (transient, dissolved).

---

## 3. Falsifiers & proposed measurements (not claims)

- **M-CAP-1 (coverage probe).** Fraction of held-out teacher behaviors within cosh-reach R of a chart anchor, vs budget S. Lack of monotonic coverage-vs-S growth falsifies "more map = more knowledge" (would indicate redundancy/saturation instead).
- **M-CAP-2 (byte audit).** One command emitting per-type byte histogram (charts/arcs/KV/MLIR/index) so §0's INFERRED split becomes measured. Trivial to implement from `encode_binary` sizes + arc struct sizes.
- **M-CAP-3 (Euclidean bakeoff arm).** Same teacher behaviors stored as (i) dense local Jacobians, (ii) LoRA adapters, (iii) map charts — bytes-vs-recall curves on the fixed corpus. Decides whether (b)'s 21–400× is realized end-to-end or dissolved by overhead.
- **Falsifier.** If M-CAP-1 shows coverage saturating below the task's required recall at max budget, the fixed map is INSUFFICIENT for that workload — answer is "bigger budget or different scoping," not "geometry failed." If M-CAP-3 shows Euclidean adapters winning bytes-vs-recall, the compression claim is withdrawn (register entry).

## What was NOT checked

Coverage probe unrun; M-CAP-2/3 unimplemented; multi-machine replication; 998 MB trajectory vault contents; outcomes-ledger binary; full `metal/mod.rs` audit; CUDA portability (out of scope per C01/C09.4 staging).
