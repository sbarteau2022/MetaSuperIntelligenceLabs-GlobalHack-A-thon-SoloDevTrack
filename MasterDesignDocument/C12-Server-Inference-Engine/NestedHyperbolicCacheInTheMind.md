# C12.x — Benefits of Nesting the Hyperbolic Cache Inside the Mind

**Status.** PROPOSED formalization. **Date:** 2026-10-27. **Parents:** C05 Atlas/KV, C07 Memory, C24 Compaction, C06.1 Anti-Lobotomy, C06.21 Carving.

---

## 1. What changes about the model's job

In the standard path, a model must **re-derive its entire world** on every turn:

- Prefill over the full context `O(N²)` attention.
- Grow the KV cache to `O(N)` memory.
- Evict or compact when limits are hit.
- Forget (literally, via context truncation) on compaction.

In the nested path, the Mind (Atlas) **persists** memory as coordinates. The model's job changes from:

> "read everything, compute everything, emit answer"

to:

> "retrieve the smallest sufficient neighborhood, attend over it, emit answer, and **reuse** what was already paid for."

The forward pass becomes **optional and marginal**, invoked only when retrieval + reuse cannot reduce free energy below threshold.

## 2. Cost model

Let `N_t` = total tokens seen by turn `t` in a session. Let `k_t` = retrieved neighbor count at turn `t` (TwisterAttention depth gate). Let `c_retrieve = O(log N)` and `c_attend = O(k)`.

**Standard:**
$$
C_{\text{std}}(t) = \Theta(N_t^2) \text{ compute},\quad M_{\text{std}}(t) = \Theta(N_t) \text{ KV memory}
$$
each turn re-pays in full; bloat is structural.

**Nested:**
$$
C_{\text{nested}}(t) = C_{\text{retrieve}}(k_t) + C_{\text{attend}}(k_t) = O(\log N_{\text{map}} + k_t)
$$
$$
M_{\text{nested}} = \Theta(|M_G|) \text{ (one map)} + \Theta(k_{\text{transient}})
$$
Memory does not grow with transcript length; it grows with **distinct atoms**, which geometric eviction bounds by salience radius.

## 3. Multi-turn amortization (the profitability core)

After turn 1, the tokens that matter are stored as atoms/basins at coordinates `x_a ∈ M_G`. On turns 2..T, the model does not re-prefill them; it **recalls**:

$$
\text{context}_t = \mathcal{N}_{k_t}(\mu_t) \subset M_G,\quad k_t \ll N_t
$$

Define cumulative cost:

$$
S_{\text{std}}(T) = \sum_{t=1}^T \Theta(N_t^2)
$$

$$
S_{\text{nested}}(T) = C_{\text{first}} + \sum_{t=2}^T O(\log N_{\text{map}} + k_t)
$$

For any multi-turn task with `N_t ≥ O(k)` and `T ≥ 2`, `S_nested ≪ S_std`. The gap widens with `T` because standard re-pays quadratically while nested pays retrieval each turn.

**Profitability (Meta view, cf. C30.8):**
$$
\Pi = \text{value}(quality) - \text{cost}(tokens, time) \implies \Pi_{\text{nested}} > \Pi_{\text{std}}
$$
once `T ≥ 2` and retrieval cost is bounded.

## 4. TTFT becomes recall speed

Time-to-first-token in the standard path is dominated by prefill:
$$
\text{TTFT}_{\text{std}} \approx \frac{N_t^2}{\text{throughput}}
$$

In the nested path, prefill is replaced by retrieval + a bounded attend:
$$
\text{TTFT}_{\text{nested}} \approx \frac{\log N_{\text{map}} + k_t}{\text{throughput}_{\text{retrieve}}}
$$

Thus, **after a warm map, TTFT is a function of neighborhood size, not transcript size.** The reward for reusing cached computation is that latency decouples from session length. On cache-hit turns, “response time” tends toward recall speed, not forward-pass speed.

## 5. Eviction becomes a non-event; compaction becomes edge, not policy

- **Eviction:** geometric forgetting (`argmax unpinned x₀`) handles staleness automatically. Pinned basins never decay. There is no hand-written LRU/heuristic; the boundary handles it.
- **Compaction:** the 60% fold rule (C24) still exists for the **context window**, but the underlying map does not compact away—the BrainContinuityCheckpoint is folded into an Atlas basin (C06.1) and remains recoverable. Compaction of context is no longer loss of memory; it is folding a trajectory into a basin.

Result: in a steady session, neither eviction nor compaction is on the hot path; both are background consequences of geometry and only matter at extremes.

## 6. Dynamic looping rule: compute only when necessary

The system should loop (call the model) only when retrieval cannot resolve the next step. Formally, at a turn boundary:

$$
\text{recompute}(t) \iff
\Delta F_{\text{reuse}}(t) > \tau_{\text{reuse}} \;\lor\; \text{novelty}(t) > \tau_{\text{novelty}}
$$

where `ΔF_reuse` is the variational free-energy gain from using cached context vs. a fresh forward pass, and `novelty(t)` measures whether the task has departed the cached neighborhood (anchor moved beyond radius `R`).

Otherwise: reuse `M_T`, regenerate only the output tail, and spend zero (or marginal) tokens on re-prefill. This is the “refuse the forward pass” reward: the meta earns profitability by not paying for computation whose result is already in the map.

## 7. Anti-lobotomy is what makes this safe

Without continuity, caching would be lobotomy-risky: compacting context would destroy the trajectory. With the checkpoint folded as an attractor basin (C06.1), and recovery via gradient flow / critical-state capsule (C06.17), the cached map is **reconstructable** even after compaction or restart. Therefore reuse is safe: the model is not hallucinating from a truncated prompt; it is recalling from a pinned, receipt-anchored basin.

## 8. What the model “does” differently (operational)

1. **First turn:** full (or large) pass; populate atoms/basins; store trajectory.
2. **Subsequent turns:** classify task → carve `M_T` → retrieve k neighbors → attend over k → emit. If the answer is already in the basin (low free-energy residual), emit with minimal generation.
3. **Only on novelty/departure:** expand the carve radius or trigger a fresh pass; log the reason.
4. **Never:** re-prefill the full transcript when the map already contains it; evict by hand; compact in a way that loses pinned basins.

## 9. Boundaries (not claimed)

- No hardware tok/s, latency, or memory numbers are claimed here; they are earned by A1/A5/A6/A39 on real hardware.
- “TTFT tends toward recall speed” assumes a warm map and a successful carve; cold start pays full cost.
- The finite-feature toroidal-phase construction remains normative (see `GeometryClaimBoundary.md`); this doc describes the caching benefit, not a proof of `H³×T²`.
- If the map is poisoned or carved incorrectly, reuse amplifies error; AA-02 guard and necessity precision (≥0.60) are prerequisites.

---

_Last updated: 2026-10-27 (nested-cache-benefit formalization pass)._
EOF
