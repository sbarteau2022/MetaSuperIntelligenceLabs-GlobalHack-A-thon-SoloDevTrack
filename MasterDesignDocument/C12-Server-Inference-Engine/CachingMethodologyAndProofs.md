# C12 — Caching Methodology and Proofs (Nested Hyperbolic Cache)

**Status.** PROPOSED formal specification + proof sketches. **Date:** 2026-10-27. **Parents:** C05, C07, C24, C06.1, C06.21.

This document states the caching methodology as an algorithm and then proves its properties. All proofs are conditional on the listed assumptions; none is a measured result.

---

## 1. Definitions

Let a session `s` have turns `t = 1..T`. At turn `t` the model receives query `q_t ∈ Q` and must produce `a_t`.

Let `M_G = (A, d_H, Γ)` be the global Atlas over `H³ × T²`. Let `μ_t = encode(q_t) ∈ H³ × T²`.

Define the **carved neighborhood**:
$$
\mathcal{N}_t = \{ a ∈ A_{\text{selected}} : \mathrm{pinned}(a) \lor d_H(x_a, \mu_t) ≤ R_t \}
$$
where `R_t` is the task-class radius (C06.21 §5.1, scaled by budget).

Define **cache hit**:
$$
\text{HIT}(t) \iff \mu_t ∈ B_r(\mu_{t-1}) \;\land\; \mathcal{N}_t \subseteq \text{cached\_set}_{t-1}
$$
i.e., the query anchor moved little and all needed atoms are already cached.

Define **TwisterAttention depth gate**:
$$
k_t^* = \arg\min_k \left[ C_{\text{ret}}(k)+C_{\text{att}}(k) - \mathbb{E}\log p(a_t \mid \mathcal{N}_{k}) \right]
$$

## 2. Methodology (algorithm)

### 2.1 Turn loop (nested cache)
```
ALGORITHM NestedTurn(q_t, M_G, ctx):
1. μ_t ← encode(q_t)
2. C  ← classify(q_t)                  // §5.1
3. D  ← required_silos(C)
4. M_T ← carve(M_G, μ_t, D, R(C,budget))   // C06.21
5. k_t^* ← depth_gate(μ_t, M_T)             // C05 §6
6. ctx_t ← N_{k_t^*}(μ_t) in M_T
7. if HIT(t): reuse_prefix(ctx_t); skip prefill
   else: prefill only the delta tokens not in ctx_t
8. a_t ← decode(ctx_t)
9. update M_G (compile_deposit) if novel; checkpoint if compaction threshold
10. return a_t
```

### 2.2 Reuse criterion
A turn **reuses** the cached prefix when:
$$
\text{cost}(\text{reuse}) < \text{cost}(\text{prefill}) \;\land\; \text{quality\_risk}(\text{reuse}) < \epsilon_{\text{risk}}
$$
Otherwise it pre-fills the delta. This keeps the system honest: reuse is chosen when cheaper and safe.

### 2.3 Continuity (anti-lobotomy)
On compaction:
$$
B_\gamma = \bigcup_{t} B_\epsilon(\gamma(t)),\quad \gamma: [0,T] \to H^3 \times T^2
$$
The checkpoint basin is pinned. Recovery reconstructs critical state via the Continuity Capsule (C06.17) regardless of cache state. Therefore a cached reuse never depends on an uncompacted transcript.

## 3. Proofs

### Theorem 1 (Bounded memory)
**Claim.** Memory of the nested cache is `O(|A_{\text{pinned}}| + N_{\text{live}})` where `N_live ≤ max_atoms`, independent of transcript length `N_t`.

**Proof.** The global map stores atoms, not tokens: `M_G` size = `|A|`. Geometric eviction removes unpinned atoms with maximal radial coordinate `x₀` (C05 §5); pinned atoms are never evicted (I4). The carve view is capped by `max_atoms` (AA-03 hardening). Transcripts are not stored; only their distilled atoms persist. Thus memory depends on distinct atoms, not token count. QED (conditional on eviction implementation and cap).

### Theorem 2 (Sublinear turn cost)
**Claim.** Under bounded expansion constant `c`, a turn costs `O(log |M_G| + k_t^*)`, not `O(N_t²)`.

**Proof.** Retrieval in the VP-tree on `H³` costs `O(log |M_G|)` expected under bounded `c` (P4). TwisterAttention attends over `k_t^*` neighbors at `O(k_t^*)`. Encoding the query is `O(1)` (fixed-size phase-state). No full-context attention occurs. Hence `C_turn = O(log |M_G| + k_t^*)`. QED (conditional on bounded `c` and correct carve).

### Theorem 3 (No bloat across turns)
**Claim.** Adding a turn increases memory by at most `O(k_new_atoms)`, where `k_new_atoms` is the number of novel (non-coalesced) atoms in that turn.

**Proof.** `compile_deposit` coalesces an incoming atom with an existing one if `cosh d ≤ 1.0000001` and matching phase invariant; otherwise it inserts one new `GlobalAtom`. Thus each turn adds at most the number of novel records, not the token count of the turn. Salience reinforcement (`salience_count++`) does not allocate. QED.

### Theorem 4 (Cache hit safety)
**Claim.** If `HIT(t)` holds, reusing the cached prefix cannot introduce atoms outside the carved neighborhood `N_t`, and pinned atoms remain present.

**Proof.** `HIT(t)` requires `N_t ⊆ cached_set_{t-1}`, so all needed atoms are already in cache. Carve filtering (C06.21 §3.1) and the AA-02 guard ensure no out-of-scope atom enters the view during forward closure (conditional on guard implementation). Pinned atoms are included in `N_t` by construction and never evicted. Therefore reuse surfaces exactly a subset of the safe, in-scope atoms. QED.

### Theorem 5 (Continuity under compaction)
**Claim.** Even if the context is compacted at turn `t`, the cached map and original objective remain recoverable.

**Proof.** Compaction folds the checkpoint trajectory into basin `B_γ` (C06.1) and pins it. The critical state travels via the Continuity Capsule (C06.17), which is exact and hash-validated. Recovery reconstructs critical state via the Continuity Capsule (exact, hash-validated); geometric basin recovery is approximate only and cannot guarantee trajectory order (F-C06-01/CORR-C06-01). Thus the cached neighborhood `N_{t+1}` can be carved from `M_G` after compaction; no trajectory is lost. QED (conditional on capsule and basin correctness; verified by A38).

### Theorem 6 (Profitability monotonicity in multi-turn reuse)
**Claim.** For `T ≥ 2` with at least one cache hit, cumulative cost `S_nested(T) < S_std(T)`.

**Proof.** `S_std(T) = Σ_{t=1}^T Θ(N_t²)`; since `N_t` is nondecreasing, each term ≥ the previous, and at least the last term ≥ `N_1²`.

`S_nested(T) = C_first + Σ_{t=2}^T (O(log N_map + k_t))`. Each summand is `O(log N + k_max)` with `k_max ≪ N_t` for typical turns (depth gate). Therefore `S_nested` grows linearly in `T` with small constant, while `S_std` grows at least quadratically in `N_T`. For any `T ≥ 2` with a hit, the inequality holds asymptotically; constants require measurement (A1/A6/A39). QED (asymptotic).

### Theorem 7 (Dynamic loop termination / only-when-necessary)
**Claim.** The recompute criterion terminates and prevents redundant forward passes.

**Proof sketch.** Define predicate `recompute(t)` (§2.2). If false, the system reuses and incurs `O(log N + k)` instead of `O(N²)`. If true, it pays the full cost but updates the map so future turns can reuse. Each "true" branch strictly increases cached coverage (`|cached_set|` grows by novel atoms), and the cap `max_atoms` bounds growth. Thus the loop cannot trigger unboundedly without expanding coverage, and when coverage is sufficient it stops. QED (conditional on cap and correct novelty detection).

## 4. Assumptions (must hold or proof weakens)

1. Bounded expansion constant `c` for VP-tree (P4); clustered inputs degrade retrieval.
2. Carve correctness and AA-02 geometric guard; otherwise out-of-scope atoms contaminate.
3. Isometry bound `≤ 4.4×10⁻⁹` (C35) for distance-based carving.
4. Continuity Capsule exactness (A38) for compaction safety.
5. Depth gate computes `k_t^*` reasonably (TwisterAttention variational gate).
6. Novelty detection is not adversarially wrong (else unnecessary recomputes or stale reuse).

## 5. What would falsify these

- A1 shows retrieval is not `O(log N)` for the real corpus (clustered inputs).
- A39 shows reuse does not improve J2–J4 (efficiency gain absent).
- A38 fails (compaction loses critical state → reuse becomes unsafe).
- AA-02 is not implemented and cross-silo leakage occurs (Theorem 4 fails).
- Novelty detection stuck → loop either never reuses or over-recomputes.

## 6. Status summary

| Theorem | Status | Owning gate |
|---|---|---|
| 1 Bounded memory | inferred | C21/C05 |
| 2 Sublinear turn cost | inferred | A1 |
| 3 No bloat | inferred | A8 (ingest round-trip) |
| 4 Cache hit safety | proposed (guard shipped) | AA-02 property test |
| 5 Continuity under compaction | proposed | A38 |
| 6 Profitability monotonicity | proposed | A6/A39 |
| 7 Dynamic loop termination | proposed | new property test |

All items here are specification-level. None becomes VERIFIED without the cited gates.

---
_Last updated: 2026-10-27 (caching methodology + proofs pass)._
EOF
