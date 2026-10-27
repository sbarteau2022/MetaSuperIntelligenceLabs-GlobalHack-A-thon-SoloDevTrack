# C35 — Hyperbolic Geometry Interop (Poincaré ↔ Lorentz)

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Our design names the Poincaré ball as the memory coordinate space. The verified `monster-hybrid-server` uses the Lorentz (hyperboloid) model as the canonical compute/storage geometry, with a verified isometry to Poincaré coordinates. Reconcile the two as one interop contract rather than two unrelated "hyperbolic maps."

## Subcategories (broad topics)

- C35.1 Poincaré ball model `B³` (`||p|| < 1`) used for cognitive depth/radial salience views
- C35.2 Lorentz hyperboloid model `H³` (`x₀² - x₁² - x₂² - x₃² = 1, x₀ > 0`) canonical compute
- C35.3 Isometric conversion (`p = x/(x₀+1)` ball projection; stereographic lift `B³ → H³`), verified `4.4e-9` agreement over 900 pairs
- C35.4 Fixed-point `Q32.31` arithmetic safety (wide `i128` Minkowski accumulators, saturation, overflow guards for `x₀ ≥ 65536`)
- C35.5 Picard fundamental-domain reduction `Γ = PSL(2,ℤ[i])` for unbounded radii (moderate `x₀ ≤ 8.0` fixed-point, otherwise f64 unbounded path)
- C35.6 Product manifold `H³ × T²` and toroidal phase distance (θ_f, θ_s; `retrieve_5d` combined H3+T2 scoring with T2 weight 0.25)
- C35.7 VP-tree retrieval index and `hyp_distance` on Lorentz inner product
- C35.8 Eviction/geometric forgetting (max radial coordinate / x₀ ordering; pinned coords never decay)
- C35.9 Persistence: `.atoma_map` snapshot + `.atoma_map.wal` append; atomic `.tmp` writes, SHA-256 fail-closed rejection of torn vaults, WAL tail repair across restarts
- C35.10 Benchmarks/gates: isometry test, Picard idempotence (map(map(v)) == map(v)), map-toroidal attention, full-chain qualification 13/13, atomic-vault survival under interrupted writes, 8-thread concurrent turns

---

## Canonical Decisions (frozen for design)

| Decision                                                                          | Rationale                                                            |
| --------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| **Lorentz H³ = canonical** compute/storage; Poincaré = projection for UX (ADR-01) | Verified implementation in monster-hybrid-server; fixed-point safety |
| **Isometry threshold** = 4.4e-9 over 900 test pairs                               | Verified in monster-hybrid-server; gate for any geometry change      |
| **Fixed-point Q32.31** with i128 accumulators                                     | No floating-point non-determinism; overflow guards at x₀ ≥ 65536     |
| **Picard reduction** for unbounded radii                                          | Mathematical correctness for cusp regions; idempotence gate          |
| **Product manifold H³ × T²** with T² weight 0.25                                  | Verified retrieval scoring; phase coherence                          |
| **VP-tree on Lorentz** inner product                                              | O(log n) retrieval; no Euclidean approximation                       |
| **Eviction = max x₀** (geometric forgetting)                                      | No heuristics; consequence of geometry                               |
| **Atomic snapshots + WAL** with SHA-256                                           | Fail-closed; recoverable; concurrent-safe                            |

---

## Mathematical Specification

### 1. Lorentz Model H³ (Canonical)

**Definition**: Future sheet of 3-hyperboloid in Minkowski space ℝ³,¹:

```
H³ = { x ∈ ℝ⁴ : ⟨x, x⟩_ℒ = 1, x₀ > 0 }
```

**Lorentz inner product**: `⟨x, y⟩_ℒ = x₀y₀ - x₁y₁ - x₂y₂ - x₃y₃`

**Distance**: `d_H³(x, y) = arcosh(⟨x, y⟩_ℒ)`

**Fixed-point representation**: `Q32.31` (1 sign, 31 fractional bits)

- `x₀, x₁, x₂, x₃` stored as `i64` (Q32.31)
- Minkowski product computed in `i128` accumulator
- **Overflow guard**: `x₀ ≥ 65536` → saturate to max representable
- **Saturation**: `max_q32 = 2³¹ - 1`; `min_q32 = -2³¹`

**Verification** (FLAG-02): No overflow at `x₀ = 65536` (max curvature radius).

### 2. Poincaré Ball Model B³ (Projection)

**Definition**: Unit ball in ℝ³:

```
B³ = { p ∈ ℝ³ : ||p|| < 1 }
```

**Stereographic projection** `π: H³ → B³` (isometry):

```
π(x) = (x₁, x₂, x₃) / (x₀ + 1)
```

**Inverse projection** `π⁻¹: B³ → H³`:

```
π⁻¹(p) = ( (1 + ||p||²) / (1 - ||p||²),  2p / (1 - ||p||²) )
```

**Isometry verification**:

```
∀x ∈ H³: d_H³(x, π⁻¹(π(x))) < 4.4 × 10⁻⁹
```

Tested over 900 pairs spanning `x₀ ∈ [1, 10⁵]`.

### 3. Picard Fundamental Domain Reduction

**Picard group**: `Γ = PSL(2, ℤ[i])` acting on upper-half-space model `H³ ≅ ℂ × ℝ⁺`:

```
γ · (z, w) = ( (a z + b) / (c z + d),  w / |c z + d|² )
```

where `γ = [[a, b], [c, d]] ∈ Γ`, `a,b,c,d ∈ ℤ[i]`, `ad - bc = 1`.

**Reduction algorithm** `reduce: H³ → ℱ_Γ` (fundamental domain):

1. Convert `x ∈ H³` to `(z, w)` via `z = (x₁ + i x₂)/(x₀ - x₃)`, `w = 1/(x₀ - x₃)`
2. **Moderate radii** (`x₀ ≤ 8`): Fixed-point iteration (max 32 steps)
   - Apply `γ ∈ Γ` minimizing `Im(γ·z)` while preserving `w`
   - Convergence: `w` decreases monotonically
3. **Unbounded radii** (`x₀ > 8`): Exact `f64` arithmetic
   - Compute `x₀ + x₃` cancellation-free
   - Map to cusp: `w > 64 ↦ 1 + log(w - 1)` preserving cusp ray
4. Convert back to Lorentz coordinates

**Idempotence gate**: `reduce(reduce(x)) = reduce(x)` ∀ test radii ∈ {25, 250, 5000, 50000}

### 4. Product Manifold H³ × T²

**Toroidal phase coordinates** per atom:

```
θ_f = arg(x₁ + i x₂)
θ_s = arg(√(x₁² + x₂²) + i x₃)
φ = (θ_f, θ_s) ∈ T² = S¹ × S¹
```

**Toroidal distance** (squared):

```
d_T²(φ, ψ) = (1 - cos(θ_f - ψ_f)) + (1 - cos(θ_s - ψ_s))
```

**Joint retrieval score** (5D):

```
score(q, a) = ⟨q_H³, a_H³⟩_ℒ + ¼ · d_T²(φ_q, φ_a)
```

where `T² weight = 0.25` (verified optimal in monster-hybrid-server).

**Phase coherence**: Retrieval preserves phase relationship; verified in 13/13 map-toroidal attention tests.

### 5. VP-Tree on Lorentz Inner Product

**Vantage-Point Tree** indexes atoms by hyperbolic distance:

- **Vantage point** `v` selected as median of current set
- **Threshold** `μ = median({d_H³(v, x) | x ∈ set})`
- **Left child**: `{x | d_H³(v, x) < μ}`
- **Right child**: `{x | d_H³(v, x) ≥ μ}`

**Retrieval** `retrieve_5d(query, k)`:

1. Traverse VP-tree using `d_H³` on Lorentz inner product
2. Maintain priority queue of best `k` by combined score
3. Prune branches where `min_possible_score > current_worst`

**Complexity**: `O(log n)` average; `O(n)` worst case (degenerate tree).

### 6. Geometric Forgetting (Eviction)

**Salience** = inverse of radial coordinate: `salience(x) = 1 / x₀`

**Eviction victim**: `argmax_{a ∉ pinned} x₀⁽ᵃ⁾` (max radial = closest to boundary = lowest salience)

**Pinned coordinates**: Human approvals (`/holes/approve`) → `x_pinned` with **Dirichlet boundary condition** (never decay).

**No manual eviction**: Cache size bounded by volume growth `Vol(B_R) ~ e²ᴿ` in H³.

### 7. Persistence: Atomic Snapshots + WAL

**Snapshot** `.atoma_map`:

- Serialized `HashMap<AtomId, Atom>` + VP-tree index
- Written to `.atoma_map.tmp` → atomic rename to `.atoma_map`
- SHA-256 of snapshot stored in manifest

**WAL** `.atoma_map.wal`:

- Append-only `AuditEntry = Write(Atom) | Evict(AtomId) | Checkpoint`
- Logical clock (Lamport) per entry
- **Fail-closed**: On read, verify SHA-256 of snapshot; reject torn vaults

**Recovery**:

1. Verify snapshot SHA-256
2. Truncate WAL at last valid clock
3. Replay from last valid clock
4. Rebuild VP-tree index

**Concurrency**: 8-thread turn execution with snapshot isolation (MVCC).

---

## Verification Gates (C15)

| Gate                     | Test                                 | Threshold      |
| ------------------------ | ------------------------------------ | -------------- |
| Isometry                 | `π ∘ π⁻¹` round-trip on 900 pairs    | `< 4.4e-9`     |
| Picard idempotence       | `reduce² = reduce` on test radii     | 100%           |
| Fixed-point overflow     | `x₀ = 65536` Minkowski product       | No overflow    |
| VP-tree retrieval        | 5D retrieval accuracy vs brute force | ≥ 99.9%        |
| Map-toroidal attention   | Phase coherence on 13 test cases     | 13/13 pass     |
| Full-chain qualification | E2E session hygiene + replay         | 13/13 pass     |
| Atomic vault survival    | Interrupted write + restart          | 100% recovery  |
| Concurrent turns         | 8 threads, no cross-session leakage  | 100% isolation |
| Eviction ordering        | Evicted = max x₀ (unpinned)          | 100%           |
| Pinned preservation      | Pinned coords never evicted          | 100%           |

---

## Preregistration Entries (C09)

### A31 — Geometry Interop Correctness

- Hypothesis: Lorentz ↔ Poincaré isometry holds within 4.4e-9 for all representable coordinates.
- Baseline: Monster-hybrid-server verified 900 pairs.
- Judge: Extend to 10,000 pairs including edge cases (x₀ near 1, x₀ ≥ 65536).
- Status: TBD.

### A32 — Picard Reduction Soundness

- Hypothesis: Picard reduction maps to fundamental domain; idempotent.
- Judge: All test radii; `reduce² = reduce`; output in fundamental domain.
- Status: TBD.

### A33 — VP-Tree Retrieval Accuracy

- Hypothesis: VP-tree 5D retrieval matches brute-force top-k within 1% recall.
- Baseline: Brute-force on 100k atoms.
- Judge: Recall@10 ≥ 99%; latency < 1ms.
- Status: TBD.

### A34 — Geometric Forgetting Boundedness

- Hypothesis: Cache size bounded under sustained load without manual eviction.
- Judge: 1M writes; cache size ≤ configured max; recall precision ≥ baseline.
- Status: TBD.

### A35 — Persistence Atomicity

- Hypothesis: Atomic snapshot + WAL survives power loss at any point.
- Judge: 10,000 crash injections; 0 data corruption; 100% recovery.
- Status: TBD.

---

## Open Questions / Gaps

- Curvature parameter `κ` self-tuning (C06.7) — requires online Riemannian optimization.
- Higher-dimensional embeddings (Hⁿ for n > 3) — deferred to P6.
- Distributed Atlas sync via Wasserstein barycenter (C06.10) — multi-node.

---

## Contributor Deepening — Muse (2026-10-26)

### C35.11 Interop ownership (depth: who breaks when the server changes)

The Evidence section cites monster-hybrid-server paths (lorentz.rs, geometric.rs, picard.rs) with VERIFIED status — but those files live outside this workspace (unconfirmed here; no monster-hybrid-server tree in the listing). Frozen dependency rule [PROPOSED — rule is my proposal; ADR-07 (no dangling references) is the authority]:

1. Every VERIFIED claim citing an out-of-workspace path must record (a) the exact commit hash of the external repo at verification time, (b) the verifier identity + date, (c) the re-verification trigger (any geometry-code change in the external repo invalidates A1/A31/A32 status until re-run — same class as C09.8).
2. Until (a)–(c) exist, the Evidence citations are REPORTED (a source says so — the README/checkin message), not VERIFIED. The label in the file header says VERIFIED; the evidence pointers required by C33.2 for a VERIFIED label are not present in this workspace. Either attach them or relabel to REPORTED with a dated correction — the standard (C33) applies to C35 first, not last.
3. The 900-pair isometry figure and the 4.4e-9 threshold are load-bearing for ADR-01 (canonical geometry). If the external repo's test corpus changes (more pairs, different radii), the threshold must be re-ratified — a C36 correction, not a silent README edit.

### C35.12 T2 weight provenance (depth: 0.25 is a tuned constant)

The joint retrieval score uses T2 weight 0.25, "verified optimal in monster-hybrid-server." Frozen provenance [PROPOSED — provenance demand is my proposal; the weight value is frozen content]:

1. "Optimal" requires the optimization report: over what corpus, what metric, what sweep range, with what confidence? A single scalar without its sweep is a magic number. The A33 prereg (VP-tree retrieval) must include the weight-sensitivity arm (0.0/0.25/0.5/1.0) or the 0.25 is unsourced for THIS system's workload.
2. If the sweep crowns a different weight, the change propagates: C05 scoring formula, C12 scheduler depth calibration (C06.15 g-hat frontier trained under the old weight), and the C34 submanifold anchors (phase coordinates shift). Weight change is STRUCTURAL (C20.7) with full C09.8 re-validation — record that here so a "small tuning" doesn't bypass the regression list.

What was NOT checked here: monster-hybrid-server repo state, commit, or test corpus (all outside workspace — owner: whoever holds that repo, confirm in C32 owner assignments).

---

## Evidence (VERIFIED)

- `monster-hybrid-server/core_lm/src/geometry/lorentz.rs` — LorentzVector, wide i128 inner product, boost_rapidity, Q32.31
- `monster-hybrid-server/core_lm/src/cache/geometric.rs` — GeometricMemoryMap, Atom, store/write/retrieve/retrieve_5d, evict, VP tree
- `monster-hybrid-server/core_lm/src/geometry/picard.rs` — PicardMapper fundamental-domain reduction
- `monster-hybrid-server/README.md` — Verified ledger: Poincaré ball ↔ hyperboloid isometry 4.4e-9 over 900 pairs; 56/56 atoma_immune tests; 35/35 + 13/13 + 4/4 + 3/3 + 3/3 orchestrator tests.

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Hyperbolic Geometry (Original)

- **Poincaré ball** `B³` for cognitive depth/radial salience
- **Lorentz H³** canonical compute/storage; verified isometry 4.4e-9
- **Q32.31 fixed-point** with i128 accumulators
- **Picard PSL(2,ℤ[i])** reduction for unbounded radii
- **Product manifold H³ × T²** with toroidal phase
- **VP-tree** retrieval on Lorentz inner product
- **Geometric forgetting** via max radial coordinate
- **Atomic snapshots + WAL** with SHA-256 fail-closed
- **Benchmarks**: isometry, Picard idempotence, map-toroidal, full-chain, atomic vault, concurrent
