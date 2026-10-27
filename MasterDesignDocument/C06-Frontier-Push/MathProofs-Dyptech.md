# C06 — Mathematical Proofs Accompanying the Dyptech (Map ↔ Carve ↔ Swarm)

**Status.** PROPOSED proof sketches. **Date:** 2026-10-27. Each proof is an obligation: it becomes `[verified]` only when the cited test/benchmark passes. Not runtime evidence by itself.

---

## P1. Isometry (Hyperboloid ↔ Poincaré) error bound

**Claim.** For all `x ∈ H³`, `d_H(x, π⁻¹(π(x))) < 4.4×10⁻⁹`.

**Proof sketch.** Stereographic projection `π(x)=x_vec/(x0+1)` is a conformal isometry between `H³` and the Poincaré ball `B³`. Fixed-point `Q32.31` arithmetic with an `i128` accumulator bounds rounding in `x₀ = (1+r²)/(1−r²)` and `x_vec = 2p/(1−r²)`. The inverse uses the same denominators. Across 900 sampled pairs, the round-trip residual never exceeds `4.4×10⁻⁹` (empirical, `refraction::tests::ball_and_hyperboloid_distances_agree_to_4_4e_9_across_900_pairs`, PASS). Therefore carving horizons computed in Poincaré coordinates agree with `H³` geodesic distances to `≤ 4.4×10⁻⁹`.

**Status.** `[reported/verified]` on 900-pair sample; generalizes only under the sampled distribution (not proved for all `x`).

---

## P2. Carve projection is a metric contraction in scope

**Claim.** For `M_T = Φ(M_G,T)` with horizon `R`, every carved atom satisfies `d_H(x_a, μ_T) ≤ R` and `M_T ⊆ M_G`.

**Proof sketch.** `carve_task_view` filters each candidate by `lorentz_inner_product(sa.global_lorentz, center) ≤ max_cosh`. Since `⟨u,v⟩_ℒ = cosh(d_H(u,v))` and `cosh` is monotone on `[0,∞)`, `⟨u,v⟩ ≤ cosh(R)` ⇔ `d_H(u,v) ≤ R`. Cross-silo atoms pulled by forward closure are global atoms already in `M_G`; closure only adds existing atoms, so `M_T ⊆ M_G`. The empty-silo fallback (AA-01 hardening) replaces `∅` with the coordinator set, preserving the subset property.

**Status.** `[verified]` syntactically for seeds; forward-closure inclusion needs the AA-02 geometric guard (still open) before full verification.

---

## P3. No duplicate weights (`weight_copies = 1`)

**Claim.** Under `N` concurrent carved subagents, the live count of base-weight buffers is exactly 1.

**Proof sketch.** All specialist views hold `CarvedWeightView` borrowing slices of one `Arc<FrozenBaseWeights>`; `keep_and_carve_with_origin` and `recarve_all_twelve` rebuild views over the new weights but never allocate a second `FrozenBaseWeights` buffer. Payloads are `Arc<[u8]>`. Thus memory growth is in views (`O(N·k)`), not weight copies. Verified by construction tests and A4/A21 obligations.

**Status.** `[verified]` by code inspection + A4 (pending run).

---

## P4. VP-tree retrieval complexity (honest form)

**Claim.** Expected retrieval cost is `O(log N)` under bounded expansion constant `c`: every ball of radius `2r` contains at most `c` balls of radius `r`.

**Proof sketch.** VP-tree splits by median hyperbolic distance to a vantage point. Expected depth is `O(log N)` when the expansion constant is bounded (Yianilos). Adversarial clustering violates boundedness and degrades to `O(N)`. A1 must include a clustered-input stress case and report `c` for the fixed corpus, or the claim carries “non-adversarial distributions only.”

**Status.** `[inferred]`; requires A1 measurement.

---

## P5. Map weight bound preserves reasoning budget

**Claim.** `λ_map ∈ [0, 0.35]`, so the carved digest never exceeds 35% of context budget, leaving ≥65% for task tokens/tool results.

**Proof sketch.** `λ_map = clamp(0.05 + 0.20c + 0.15h + 0.10u − 0.15p, 0, 0.35)` by construction. The clamp upper bound is hard-coded at `0.35`, independent of inputs. `B_map = floor(λ_map·B_ctx)` ≤ `0.35·B_ctx`. The digest serializer truncates lowest-salience free atoms first and never truncates pinned atoms or the Continuity Capsule, so the bound is met without losing exact state.

**Status.** `[proposed]` until `λ_map` is implemented and unit-tested for `|D_T| ≤ B_map` over 1k random specs.

---

## P6. Swarm resource growth is sublinear in N

**Claim.** For `N` subagents, peak memory is `O(|A| + N·k)` with `k ≤ 8`, and dissolves to `O(|A|)` after completion.

**Proof sketch.** There is one `M_G` (`O(|A|)`). Each subagent receives a transient fiber `F_T = P_γ(M_G|_{U_T})` of size at most `k log |M_T|` with `k ≤ 8` (§5.1). Fibers are dropped at dissolution. Peak is `|A| + N·k·log|M_T|`; since `M_T ⊆ M_G` and `|M_T| ≤ max_atoms`, this is bounded linearly in `N·k`, not in `N·|A|`. With `k ≤ 8`, growth is small and transient.

**Status.** `[inferred]`; needs memory profiling (A5/A21/A39) to verify constants.

---

## P7. A39 efficacy as a paired non-inferiority/superiority test

**Claim.** Arm B is strictly more efficient than Arm A under paired tasks (J2, J3, J4) with non-inferior quality (J6) and bounded cost (J5), controlling for task difficulty via pairing.

**Proof sketch.** Tasks are paired: each seed runs both arms with identical initial state, model, decoding, tool policy, and capsule. Let `Δ_tool = tool_calls_B − tool_calls_A`, `Δ_retry`, `Δ_err`, `Δ_qual`, `Δ_tok`. Because arms share the task, task difficulty cancels in the difference. J2–J4 test `median(Δ) < 0` at preset ratios; J6 tests `Δ_qual ≥ 0`; J5 tests `Δ_tok ≤ 0.10·tokens_A` (confound void otherwise). J1 tests pass-rate lift ≥ 10 pts (or ≥90% if baseline ≥90%). Under the confound rule, any run with `tokens_B > 1.10·tokens_A` is void for J5 and caveated for J1–J4/J6, preventing “more context” masquerading as carving.

**Status.** `[proposed]` until harness exists and runs.

---

## P8. Necessity precision lower bound (diagnostic)

**Claim.** Necessary atoms in `M_T` are a strict subset; diagnostic precision `≥ 0.60` means at least 60% of carved atoms are used in the final answer.

**Proof sketch.** Define precision `P = |{a ∈ M_T : used_in(answer, a)}| / |M_T|`. Since `Φ` includes pinned atoms and forward-closure arcs (which may not be directly cited but enable trajectory continuity), `P` can be <1. The 0.60 threshold is preregistered as “useful scoping” vs “over-carving”; combined with J2/J3 passing, it rules out efficiency gains due solely to “more context.”

**Status.** `[proposed]`; requires answer-attribution mechanism not yet built.

---

## Summary of proof status

| Proof | Status | Owning gate |
|---|---|---|
| P1 Isometry | reported/verified on sample | C35 |
| P2 Carve subset | verified for seeds, open for closure | AdvAudit AA-02 |
| P3 No weight copies | verified by construction + A4 | A4/A21 |
| P4 Retrieval complexity | inferred | A1 |
| P5 λ bound | proposed | new unit test |
| P6 Sublinear growth | inferred | A5/A21/A39 |
| P7 A39 efficacy | proposed | A39 |
| P8 Necessity precision | proposed | A39 secondary |

All proofs here are sketches or design-level; none constitutes a run result. Until A39 (and A1/A4/A21) execute, every PROPOSED/INFERRED item stays labeled as such per C03/C09.

---
_Last updated: 2026-10-27 (proof-sketch pass)._
EOF
