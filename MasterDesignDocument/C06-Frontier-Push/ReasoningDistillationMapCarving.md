# C06.21 — Reasoning Distillation: From the Global Atlas Map to the Carved Swarm Map

**Status.** PROPOSED methodology specification (derived design, no implementation evidence). **Parent:** C06 Frontier Push, C05 Atlas, C06.12 Holonomy Brigade. **Ablation owner:** A39 (filed below).

---

## 1. Purpose

This document formalizes three things the sprint otherwise leaves implicit:

1. **Reasoning distillation** — the lawful operation that takes a region of the *global* hyperbolic memory map (the Atlas, `H^3 × T^2`) and produces a *task-scoped* "carved swarm map" that one subagent can reason inside without seeing, or paying for, the whole Atlas.
2. **Task-to-basin necessity** — a formal rule for deciding, for an incoming query or assigned task, *which* silos/basins of knowledge must be carved and handed to the agent, and which may be omitted.
3. **Map weight** — a formal scalar `λ_map` that states how much of the agent's context/token budget the carved map is allowed to occupy, as a function of task pressure and retrieval uncertainty.

The final claim this methodology is written to earn, through **A39**, is: *a non-tuned subagent given a lawfully carved swarm map completes the same task with fewer tool calls, fewer retries, fewer errors, and lower resource spend than an otherwise identical subagent given no map, while preserving (or improving) output quality.*

This is an **active-inference swarm at scale** claim: there is still **one weight set** (`W`, `weight_copies = 1`), and carving is **projection, not duplication** (C06.12, C04.5). The efficiency gain comes from reducing the *information surface* the agent must traverse, not from cloning model weights.

---

## 2. Terms and invariants

| Term | Definition |
|---|---|
| **Global map `M_G`** | The Atlas: a set of atoms `A ⊂ H^3 × T^2`, each an `Atom = (id, x, φ, salience, pinned, receipt, domain)` plus the VP-tree index and the pinned attractor basins. One per process. Never duplicated. |
| **Basin `B`** | An attractor region of `M_G` (C06.1). Carries an objective trajectory folded into a geodesic envelope; may be pinned (governance) or free. |
| **Silo `S_d`** | A named domain partition of `M_G`: `S_code`, `S_memory`, `S_workflow`, `S_research`, `S_security`, etc. Silos are *subsets tagged by domain*, not separate stores (C07.3 canonical home rule: no per-agent silo, no duplicate ledger). |
| **Anchor `μ_T`** | The hyperbolic coordinate that represents the task/query in `H^3` (phase-state encoding of the objective). |
| **Carved swarm map `M_T`** | The task-scoped projection (fiber) of `M_G`: `M_T = Φ(M_G, T)` (§3). A *view*, not a copy. |
| **Fiber `F_T`** | The active specialist's *transient* working view = `P_{γ_T}(M_G|_{U_T})` where `U_T` is the geodesic neighborhood. Dissolves back. |
| **Digest `D_T`** | The token-serialized form of `M_T` handed to the subagent under budget `B_map` (§6). |
| **Map weight `λ_map`** | Fraction of the agent's context budget devoted to `D_T`. Scalar in `[0, λ_max]`. |

**Invariants (non-negotiable):**

- `I1` One weight set: `weight_copies(M_T execution) = 1`. Carving never clones `W`.
- `I2` Canonical home: all summaries land in `M_G`; `M_T` and `D_T` are projections. No second ledger.
- `I3` No silent precision loss for exact state: exact critical state (objective, invariants, acceptance, commitments, pending, next action) travels through the Continuity Capsule (C06.17), *not* through `M_T`. `M_T` is supporting recall only.
- `I4` Pinned basins are never evicted from `M_T` by geometry: if a pinned basin is in the necessity set, it ships (subject to budget; if budget forbids, the run fails the judge rather than silently dropping).
- `I5` Privacy: `M_T` carries atom ids, receipt hashes, coordinates, and salience — never raw objective/invariant/commitment text from other lanes.

---

## 3. Method of reasoning distillation: `Global Map → Carved Swarm Map`

Given a task `T` with objective `o_T`, the distillation is the operator `Φ : M_G × T → M_T`.

```
Φ(M_G, T) =
   1. μ   = encode(o_T) ∈ H^3 × T^2                  // anchor
   2. C   = classify(T)                                 // task class
   3. D   = required_domains(C)                         // silo subset
   4. S   = { b ∈ basins(M_G) : domain(b) ∈ D }        // candidate basins
   5. score(b) = relevance(μ, b) · salience(b) · geo_gate(μ, b)
   6. B_sel = top-k basins by score, k = depth_gate(T)  // TwisterAttention depth
   7. R    = radius(C, budget)                          // geodesic radius
   8. A_sel = { a ∈ atoms(M_G) :
                   domain(a) ∈ D
                   ∧ d_H(x_a, centroid(B_sel)) ≤ R
                   ∧ phase_dist(φ_a, μ) ≤ δ_phase
                   ∧ pinned(a) ∨ affinity(a, T) ≥ α }    // atom selection
   9. M_T  = fold(A_sel into basins B_sel)              // compaction into basins
  10. M_T  = pin(M_T, B_sel ∩ pinned)                  // governance pins survive
  11. D_T  = serialize(M_T, B_map = floor(λ_map · B_ctx)) // budgeted digest
  12. return (M_T, D_T, μ, receipts(M_T))
```

### 3.1 Step notes

- **Step 1 (anchor)** uses the same phase-state encoder as the Atlas (C05 §4), so `μ` lives in the same coordinate system as atoms. No translation layer.
- **Step 2 (classify)** uses the task class table in §5. If the classifier confidence `< τ_class`, the task is treated as `C_general` and `D = all silos` (safe default, no scoping claim earned for that run).
- **Step 3–4 (domains)** select silos, not individual atoms. This is the unit of necessity; atoms are only filtered inside selected silos.
- **Step 5 (score)**:  
  `relevance(μ,b) = exp(-d_H(μ, centroid(b)) / τ_b)`  
  `geo_gate(μ,b) = 1` if `d_H(μ, centroid(b)) ≤ ρ_cutoff`, else `0` (hard geodesic cutoff, not softmax; prevents long-tail contamination).
- **Step 6 (depth gate)** reuses the TwisterAttention variational gate (C05 §6): `k* = argmin_k [C_retrieve(k) + C_attend(k) − E log p(answer | context_k)]`. The same gate, but now it also bounds *basin* count, so retrieval cost scales with answer hardness.
- **Step 7 (radius)** `R(C, budget)` is given in §5 per class, scaled by `sqrt(B_map / B_map_default)` so a larger map budget widens the neighborhood proportionally.
- **Step 8 (atom selection)** includes *pinned* atoms in selected domains even if their affinity is below `α`. Pinned = governance-approved; they may not silently fall out of scope.
- **Step 9 (fold)** compacts the selected atoms into the selected basins rather than listing them. This is what makes the *carved* map a map and not a dump: trajectory → basin (C06.1), preserving radial salience.
- **Step 10 (pin)** applies the governance pins (C06.8) so approved coordinates never decay inside the swarm map either.
- **Step 11 (serialize)** emits `D_T` as an ordered list of basin digests `{ basin_id, centroid, radius, top-m atom ids + receipt hashes, pinned flag, salience band }`. No raw context text (C24.8 privacy).
- **Step 12 (receipts)** attaches a Merkle path per basin to the atom receipts, terminating in the ledger root (CORR-C06-02). `M_T` is then auditable: "why did you remember X?" terminates in a receipt root.

### 3.2 Dissolution (inverse operator, required)

After the subagent returns `ΔM_T` (coordinate deltas, new basin candidates, salience updates):

```
Φ^{-1}(M_G, M_T, ΔM_T) =
   1. Validate ΔM_T against effect system (C29) + receipt chain (C06.3/C06.11)
   2. Transport deltas back along γ_T^{-1} (parallel transport, C06.12)
   3. Merge into M_G by atom id, last-writer-wins only inside the same basin id
   4. Recompute Atlas Merkle fingerprint; it must equal
         fingerprint_pre  XOR  intended_write_set_hash
   5. On mismatch → quarantine ΔM_T, emit receipt, open circuit breaker (C06.9)
   6. Dissolve fiber F_T; free the view; weight_copies check = 1
```

If `Φ^{-1}` fails at step 4 or 6, the entire subagent result is void, not "partially applied." This is the "dissolution leaves the map consistent" judge from C06.12(d).

---

## 4. Formalization: what the map *is* as a reasoning substrate

A carved swarm map is not a summary. It is a **coordinate-indexed, receipt-anchored sub-manifold**:

$$
M_T = (\mathcal{A}_T,\ \mathcal{B}_T,\ \mu_T,\ R,\ d_{H^3},\ \Gamma_{\text{rec}})
$$

where

- `\mathcal{A}_T ⊂ M_G` atoms in the selected domains inside radius `R`,
- `\mathcal{B}_T` selected basins (with pinned flags),
- `\mu_T` anchor,
- `d_{H^3}` restricted distance,
- `\Gamma_{\text{rec}}` receipt fiber (C06.11).

**Reasoning inside `M_T`**: the subagent does not attend over raw tokens from `M_G`; TwisterAttention is restricted to `\mathcal{N}_k^{(T)}(q)` for `q` inside `M_T`. Thus the *trajectory* of the agent is confined to the fiber, which is the source of the efficiency claim: the search space is `O(log |M_T|)` instead of `O(log |M_G|)` with `|M_T| ≪ |M_G|` in practice, and `M_T` already excludes irrelevant domains by construction.

This is the active-inference content: instead of minimizing free energy over the *whole* Atlas, the agent minimizes free energy over `M_T`, and `M_T` is itself chosen to minimize a (task, cost, risk) objective. Nested minimization.

---

## 5. Formalized methodology: which queries/tasks require which silos/basins

### 5.1 Task classes and required silos (the "necessity table")

| Class `C` | Example | Required silos `D(C)` | Default radius `R` (normalized `x_0` depth) | Depth gate `k` |
|---|---|---|---|---|
| `C_code_change` | multi-file edit, refactor | `S_code, S_workflow, S_memory` | `0.35` | A3 (adaptive) |
| `C_debug` | incident, failure triage | `S_code, S_memory, S_security` | `0.45` | 5 |
| `C_research` | synthesis, compare | `S_research, S_memory` | `0.50` | 4 |
| `C_review` | PR review, audit | `S_code, S_security, S_workflow` | `0.40` | 3 |
| `C_plan` | plan, design | `S_workflow, S_memory, S_research` | `0.55` | 6 |
| `C_general` | unclassified / low confidence | `all silos` | `0.60` | 8 |

`S_memory` is always considered but may be empty after carving (then it simply adds no atoms).

### 5.2 Necessity scoring for basins within selected silos

For a selected silo `S_d` and basin `b ∈ S_d`, necessity:

$$
\mathcal{N}(T, b) =
0.35 \cdot \mathrm{aff}(T, b) +
0.25 \cdot \mathrm{dep}(T, b) +
0.20 \cdot \mathrm{rec}(b) +
0.10 \cdot \mathrm{pin}(b) +
0.10 \cdot \mathrm{interf\_safety}(T, b)
$$

where

- `aff(T,b) = cos_sim(e_T, e_b)` semantic affinity of the task embedding to the basin's stored topic vector.
- `dep(T,b) = 1` if `b` is a dependency basin (e.g., the objective/invariant basin of the active run), else `0`.
- `rec(b)` recency band in `{0, 0.5, 1}` (recent/neutral/stale).
- `pin(b) = 1` if pinned, else `0`.
- `interf_safety(T,b) = 1 − I(T_lane, b_lane)` estimated cross-lane interference (C06.5), so high-interference basins in *other* lanes are deprioritized unless pinned.

A basin is **necessary** iff `\mathcal{N}(T,b) ≥ θ_N` with `θ_N = 0.50`. Necessary basins are carved. Basins with `\mathcal{N} < θ_N` are omitted unless pinned (I4).

### 5.3 Necessity scoring for atoms within carved basins

Inside a carved basin, atoms are filtered by

$$
a \in M_T \iff \mathrm{pinned}(a) \lor \mathrm{aff}(T, a) \ge \alpha_T
$$

with `\alpha_T = 0.40` for code tasks and `0.50` otherwise. This is a *policy constant*; changing it requires a preregistration amendment (C11.3 spirit).

### 5.4 What counts as "information necessary"

The **minimal necessary payload** handed to a subagent is therefore

$$
I_{\text{nec}}(T) = \{ \text{objective, invariants, acceptance, commitments, pending, next action} \}
\;\cup\;
\{ \text{atom digests with } \mathcal{N} \ge \theta_N \}
\;\cup\;
\{ \text{pinned basins in selected domains} \}
$$

where the first set travels via the Continuity Capsule (exact, C06.17) and the rest travel via `D_T` (approximate supporting recall). `I_nec` is the *entirety* of what the agent is permitted to assume; anything not in `I_nec` must be re-derived or re-fetched, not hallucinated.

This is the answer to "what information is necessary to give the agent based on the tasks I'm assigning it": **the capsule gives exact critical state; the carved map gives the smallest receipt-anchored supporting neighborhood; nothing else.**

---

## 6. Map weight assignment: `λ_map`

### 6.1 Definition

For an agent with context budget `B_ctx` tokens, the carved digest budget is

$$
B_{\text{map}} = \lfloor \lambda_{\text{map}} \cdot B_{\text{ctx}} \rfloor
$$

and the digest is serialized to *exactly* fit `B_map` (truncate lowest-salience free atoms first; never truncate pinned or exact-state capsules).

### 6.2 Assignment rule

Let

- `c = complexity(T) ∈ [0,1]` = normalized (#tool deps + #state fields + expected hops), clamped,
- `h = retrieval_entropy(T) = H(softmax(score(b)))` over candidate basins (nats),
- `p = token_pressure = tokens_used_so_far / B_ctx`,
- `u = uncertainty(T) ∈ [0,1]` = classifier confidence inverted (`1 − conf`).

Then

$$
\lambda_{\text{map}} = \mathrm{clamp}\bigl(
0.05 + 0.20\,c + 0.15\,h + 0.10\,u - 0.15\,p,\;
0,\; \lambda_{\max}=0.35
\bigr)
$$

### 6.3 Interpretation and guardrails

- **Low-value tasks** (trivial, single-shot, confident): `λ ≈ 0.05` → map is a small breadcrumb, not a load.
- **High-complexity / high-entropy tasks**: `λ` approaches `0.35` → map carries up to ~1/3 of context.
- **Token pressure** pulls `λ` *down*: when the transcript is already full, we do not make it worse by injecting a map; we rely on the Continuity Capsule instead (which is already exact and cheap).
- **Hard cap `0.35`**: the map never dominates the agent's reasoning budget; at least 65% of context stays available for the actual task tokens and tool results. This prevents "map poisoning via dominance."
- **Zero-map allowed**: `λ = 0` is valid and is exactly the A39 baseline arm. The formula must produce `0` only when `c, h, u` are all ~0 *and* pressure is high; otherwise the default `0.05` floor protects against "no map ever" (which would also starve the agent of continuity breadcrumbs).
- Changing the coefficients or the cap requires a preregistration amendment and re-running A39.

---

## 7. Active inference swarms at scale: resource accounting

For `N` subagents carved from one `M_G`:

| Resource | Cost | Scaling |
|---|---|---|
| Weight set `W` | 1 copy | `O(1)` regardless of N (I1) |
| Global map storage `M_G` | 1 copy | `O(|A|)` |
| Fiber views `F_T` | `N × O(k log |M_T|)` transient | freed at dissolution |
| Digest tokens per agent | `λ_map · B_ctx` | bounded by cap |
| Peak KV / metal | `baseline + Σ fiber` | must stay under C12 ceiling (36 GB / 27.45 GB peak) |
| Merge (dissolution) | `O(|ΔM_T|)` per agent | serialized on basin id; no global lock |

**Claim**: peak resource growth is **sublinear in N** because fibers are views and dissolved; the only N-scaling term is transient `k`-NN working sets, each bounded by `k ≤ 8` (§5.1). This is the formal version of "active inference swarms at scale": you scale the *number of trajectories through the map*, not the number of models or maps.

**Explicitly not claimed here**: tok/s, latency, or memory numbers. Those are earned by A1/A5/A39 on real hardware and remain PROPOSED until measured.

---

## 8. Ablation A39 — non-tuned subagent without map vs. map-assigned subagent

Filed as `Pre-Registration-A39-v1.md` (sidecar) per the C06 pattern. Canonical hash to be recorded in `Claims_Register.md` before any run (C11.3).

### 8.1 Hypothesis

A non-tuned subagent given a lawfully carved swarm map `D_T` (Arm B) will, on the same task set, achieve equal-or-better output quality with strictly fewer tool calls, fewer retries, fewer errors, and lower total token spend than an identical non-tuned subagent with no map (Arm A), because domain scoping and radial salience compress the reasoning trajectory space.

### 8.2 Arms

- **Arm A (baseline):** subagent receives the Continuity Capsule exact state + task prompt + (flat, top-k) BM25 context dump from the selected domains, **no** `D_T`, `λ_map = 0`. This isolates the map's *structural* contribution from the capsule's.
- **Arm B (treatment):** same subagent, same capsule, same model/decoding/tools/policy, but additionally receives `D_T` with `λ_map` chosen by §6.2.

Both arms use the **same** non-tuned weights (no fine-tuning, no prompt engineering beyond the map injection template, which is frozen in the A39 run manifest).

### 8.3 Paired workload

At least `N ≥ 40` seeded tasks across the classes in §5.1, 8 per primary class, each with a canonical expected outcome, decoy context, and at least one tool receipt. Seeds, order, and expected values frozen in the A39 run manifest before execution. Each task runs both arms (paired), eliminating task-difficulty confounds.

### 8.4 Primary judges (all must pass for the claim)

| ID | Metric | Frozen threshold | Check |
|---|---|---|---|
| J1 | Task pass rate (judge-gated) | Arm B ≥ Arm A + 10 pts (or Arm B ≥ 90% if Arm A ≥ 90%) | Deterministic judge rubric, no LLM scoring of the primary verdict |
| J2 | Median tool calls per task | Arm B ≤ 0.80 × Arm A | Counted from tool ledger |
| J3 | Median retries per task | Arm B ≤ 0.70 × Arm A | Retry-signature ledger |
| J4 | Error rate (failed tasks / total) | Arm B ≤ 0.50 × Arm A | Failed-task receipts |
| J5 | Total token spend per completed task | Arm B ≤ 1.10 × Arm A (map cost allowed, but not a blowup) | Tokenizer counted |
| J6 | Output quality | Arm B ≥ Arm A on the canonical rubric (no regression) | Rubric check |

### 8.5 Secondary judges

p50/p95 wall-clock per task; peak metal/VRAM; `weight_copies` invariant (must be 1 in both arms); fiber dissolution success rate (must be 100% in Arm B); map digest fit (`|D_T| ≤ B_map` for every Arm B run); necessity precision (fraction of carved atoms actually used in the final answer) as a diagnostic.

### 8.6 Decision rule

1. A39 **FAILS** if any of J1–J4 or J6 misses threshold. The map-carving claim cannot ship; record in `Failures.md`.
2. A39 **PASSES CORE** if J1–J4 and J6 pass. If J5 misses, the claim ships as "quality/efficiency of trajectory" only, not "cost reduction," and `λ_max` must be re-derived.
3. Necessity + scoping claim: additionally requires J2 and J3 to pass *and* necessity precision ≥ 0.60 as a secondary diagnostic. Otherwise the gain may be attributable to "more context" generally rather than *correct* carving.
4. Before the methodology becomes *required* (not optional), it must replicate on a second machine and pass a necessity ablation showing that removing the carved map regresses J2 or J3 (C06.13).
5. A failed arm remains in the record. A retry needs a new A39 version + new hash; prior results are never overwritten (ADR-19, ADR-22).
6. **Confound control**: any run where Arm B receives more *total* tokens than Arm A by > 10% is void for J5 but may still be evaluated for J1–J4/J6 with a recorded caveat.

### 8.7 What A39 deliberately does *not* test

- It does not test hyperbolic-geometry speed (A1 owns that).
- It does not test weight copying (A4/A21 own that).
- It does not test the Continuity Capsule exactness (A38 owns that).
- It does not test necessity of the map in a fine-tuned agent: both arms are non-tuned, so the claim is about *information scoping*, not training.
- It does not, by itself, prove "active inference swarms at scale" end-to-end; that broader claim needs A5 (lanes) + A21 (weights) + A39 together.

---

## 9. Claims seeded by this document (proposed, OPEN)

| Claim ID | Claim | Label | Evidence pointer | Status |
|---|---|---|---|---|
| CC-010 | Reasoning distillation from the global Atlas to a task-carved swarm map is a well-defined projection (fiber), not a copy, and preserves `weight_copies = 1`. | PROPOSED | this doc §3, §7; C06.12 | OPEN |
| CC-011 | The necessity table + scoring rule in §5 select a strict subset of basins/atoms without dropping pinned or exact-state content. | PROPOSED | this doc §5; A39 necessity precision | OPEN |
| CC-012 | `λ_map` assignment by §6 improves trajectory efficiency without allowing map dominance (`≤ 0.35`). | PROPOSED | this doc §6; A39 J5 | OPEN |
| CC-013 | A non-tuned subagent with a carved swarm map achieves equal-or-better quality with fewer tool calls, retries, and errors than without (A39). | PROPOSED | A39 J1–J6 | OPEN |
| CC-014 | Swarm resource growth is sublinear in `N` because fibers are views that dissolve (§7). | INFERRED | §7; A21 | OPEN |

None of these are VERIFIED. [VERIFIED by workspace inspection] No implementation, benchmark, judge output, or run exists for A39.

---

## 10. What was NOT checked here

- The classification table in §5.1 is a *proposal*, not an empirical result; class boundaries need a classifier and its confidence gate (§3.1 step 2).
- The coefficients in §5.2/§6.2 are *proposed* constants; they are not tuned and must be treated as preregistered constants subject to amendment rules.
- Interoperability of `Φ`/`Φ^{-1}` with C35 geometry (Lorentz ↔ Poincaré) is assumed, not proven here; if isometry (4.4e-9) does not hold, carving distances are meaningless. Verify at C35.
- No multi-machine replication, no chaos (interrupted dissolution) run, and no memory-ceiling proof exists yet.
- `Necessity precision` (J secondary) requires an answer-attribution mechanism that does not yet exist in the codebase.

---
_Last updated: 2026-10-27 (methodology formalization pass)._
EOF
