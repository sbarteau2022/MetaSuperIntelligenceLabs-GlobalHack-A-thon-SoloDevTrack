# C06.x — From Distilled Trajectories/Charts to Raw Capability

**Status.** PROPOSED formalization. **Date:** 2026-10-27. **Parents:** C05, C06.12, C06.21, C07, C13.

---

## 1. What the hyperbolic map actually stores

The map does **not** store “answers” and does **not** store model weights. It stores *structure*:

- **0-cells (`GlobalAtom`):** distilled knowledge coordinates `(x, φ)` plus payload (a compacted fact/reasoning step) and provenance.
- **1-cells (`DistilledReasoningArc`):** geodesic connections between atoms (`from → to`) encoding reasoning steps/transitions.
- **2-cells (holonomy):** closed loops whose enclosed curvature encodes consistency (Gauss-Bonnet); used for salience reinforcement.
- **Charts (rank-8 MGS subspace):** local tangent bases `{u_1..u_k}` attached to atoms, allowing incremental accretion of reasoning directions without full retraining.

The map is therefore a **compressed reasoning graph**, not a parameter set. `FrozenBaseWeights` remains the single weight buffer (I1).

## 2. Definition: “raw capability”

Define capability informally as *the agent's ability to solve a task correctly, efficiently, and with provenance*. Formally, for a task distribution `𝒯` and agent policy `π`:

$$
\text{Cap}(\pi) = \mathbb{E}_{T \sim \mathcal{T}} \left[ \mathbb{1}[\text{JudgePass}(T,\pi)] \cdot \text{Efficiency}(T,\pi) \right]
$$

where efficiency combines tool calls, retries, tokens, and time (lower is better). A system has “more raw capability” if `Cap` increases **without changing model weights** and **without more training**.

The nested map increases `Cap` by providing a *structured prior*: the agent does not re-derive connections that the map already encodes.

## 3. Conversion pipeline: trajectories/charts → capability

The conversion is not magic; it is **use**: the base model consults the map as an external, receipt-anchored working memory. The pipeline:

```
DISTILL (once / periodically):
  teacher/agent traces → project → classify well → refract → GlobalAtom (0-cell)
  consecutive steps   → parallel transport → DistilledReasoningArc (1-cell)
  closed role loops   → holonomy (2-cell) → salience reinforcement

INVOKE (per task):
  task q → anchor μ_q → carve M_q (C06.21)
  retrieve k-nearest atoms + forward-close arcs (≤64 hops)
  TwisterAttention attends over N_k (not full context)

EXECUTE:
  agent follows recalled arcs / charts to produce answer
  dissolution merges validated deltas back into M_G
```

Thus “raw capability” emerges from **(a) distilled structural memory** + **(b) lawful retrieval** + **(c) unchanged base weights**. No fine-tuning occurs; the model uses its existing weights more effectively because the map narrows the search.

## 4. Mathematical formalization

Let `M` be the map and `π_M` the agent policy that may consult `M` (via carve/retrieve), with `π_0` the same agent without `M` (A39 baseline).

Define the agent's effective policy:
$$
\pi_M(T) = \pi_0(T \mid \text{retrieve}(M,T))
$$

The capability lift is:
$$
\Delta\text{Cap} = \mathbb{E}[\text{JudgePass}(T,\pi_M) - \text{JudgePass}(T,\pi_0)] + \eta \cdot \text{EffGain}
$$
where `EffGain` captures J2–J4 (tool calls, retries, errors). The hypothesis (CC-013) is `ΔCap > 0` on paired tasks.

### 4.1 Why trajectories become capability (information view)

A reasoning trajectory `γ = (a_1,…,a_m)` encodes how the agent moved from problem to answer in `H³ × T²`. Stored as arcs, it provides:

- **Connectivity prior:** P(next reasoning step | current) is higher along stored arcs than random.
- **Landmark prior:** atoms with high salience have high posterior probability of relevance.
- **Chart (tangent) prior:** rank-8 MGS bases give locally linear directions for continuation.

By conditional probability:
$$
P(\text{correct answer} \mid q, M) \ge P(\text{correct answer} \mid q)
$$
if `M` contains trajectories correlated with correct solutions. The map increases mutual information `I(answer; M | q)`. That is the formal sense in which distillation becomes capability: **the map is an external latent variable that the agent conditions on**, not a change to weights.

### 4.2 No-weight guarantee (invariant)

`π_M` uses the same `FrozenBaseWeights W` as `π_0` (I1, A4/A21). Therefore any `ΔCap` is attributable to **information scoping**, not parameter change. A39 (both arms non-tuned) isolates this cleanly.

### 4.3 Boundedness

The map is capped (`max_atoms`, carve radius `R`, `k ≤ 8`). Capability gain is bounded above by the information content of `M_T`. If `M_T` is empty or poisoned, `ΔCap ≤ 0`. This is why necessity precision ≥ 0.60 (A39 secondary) is required: carving must select atoms that are actually used, not just “more context.”

## 5. What “raw capability” is NOT

To avoid overclaiming (see `GeometryClaimBoundary.md`):

- It is **not** a new trained model. No gradients, no fine-tuning.
- It is **not** a proof of `H³×T²` validity. The geometry enables scoping; efficacy is measured by A39.
- It is **not** weight transport (C06.12 explicit correction: `HolonomyBinding` binds receipts; no weights move).
- It is **not** Penrose/twistor sheaf cohomology capability. The normative space is finite-feature toroidal phase (§10 roadmap only).

## 6. Harness definition (A39) — how to measure this

The conversion from trajectories to capability is only as good as the measurement. A39 provides it:

- **Arms:** Arm A (no map) vs Arm B (carved map `D_T`), same non-tuned weights, same capsule, paired tasks.
- **Judges:** J1 pass rate, J2 tool calls ≤0.8×, J3 retries ≤0.7×, J4 errors ≤0.5×, J5 tokens ≤1.1×, J6 no quality regression; secondary necessity precision ≥0.60.
- **Key diagnostic:** if J2/J3 pass and necessity precision ≥0.60, the gain is attributable to **correct scoping** (trajectories converted to capability), not to “more context.”

Until A39 runs and replicates, `ΔCap` remains PROPOSED (CC-013).

## 7. Summary formula

$$
\text{Raw capability gain} = I(\text{answer}; M_T \mid q) \;\times\; \text{scoping precision} \;\times\; \text{base-weight unchanged}
$$

The distillation makes capability possible; the carve makes it scoped; A39 proves whether it happened.

---

_Last updated: 2026-10-27 (trajectory-to-capability formalization pass)._
