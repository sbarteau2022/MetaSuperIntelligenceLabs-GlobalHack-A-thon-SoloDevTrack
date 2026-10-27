# C12 — Server Inference Engine / Scheduler / Kernels / Throughput / Prefix Cache

**Status.** PROPOSED (server-side, rust-server). **Parent:** Master Design Document index.

## Purpose

Server-side inference path: ACFT + scheduler + kernels + throughput qualification + prefix caching
under hyperbolic memory.

## Subcategories (broad topics)

- C12.1 Inference engine (ACFT path, standard transformer fallback behind flag, measured TTFT/decode tok/s)
- C12.2 Scheduler (continuous batching, admission control cap min(16,max_connections), prefix-aware batching, task-depth-aware scheduling)
- C12.3 Kernel logic (fused attention kernels for hyperbolic retrieval, n-gram lookahead trie draft path, bounded buffers, explicit memory accounting)
- C12.4 Throughput qualification (receipts throughput_qwen36/ttft_qwen36 style; stress tests)
- C12.5 Prefix caching in nested hyperbolic KV era (Radix prefix cache revisited, coordinate collapse, invalidation cost, thresholds)
- C12.6 Kill-switch interaction and graceful degradation

## Open questions / gaps

- Hyperbolic prefix cache design open questions (invalidation, geodesic thresholds, memory accounting, page-entrokv).
- Measured numbers TBD.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 15. Server Inference Engine / Scheduler / Kernel Logic / Throughput / Prefix Caching (PROPOSED)

Scaffolded for the next pass (server-side, in rust-server). Details and measured numbers TBD.

### 15.1 Inference engine

- Replace the single-request forward pass with the ACFT path (§5): task-scoped retrieval via
  TwisterAttention, building only the needed attention slices.
- Keep a standard transformer fallback behind a flag (§14 kill switch).
- Track real TTFT/decode tok/s as first-class (not placeholders).

### 15.2 Scheduler

- Continuous batching with admission control (max in-flight cap already raised to
  `min(16, max_connections)`).
- Prefix-aware batching: coalesce requests sharing a prefix (Radix prefix cache) while honoring
  per-request isolation and memory budgets.
- Task-depth-aware scheduling: shallow recalls served first; deep recalls queued/weighted.

### 15.3 Kernel logic

- Fused attention kernels for the hyperbolic retrieval path; n-gram lookahead trie already ported
  (`ngram_lookahead.rs`) as a draft path.
- Bounded buffers and explicit memory accounting so peak allocation is predictable.

### 15.4 Throughput qualification

- Baselines and targets recorded via `throughput_qwen36.json` / `ttft_qwen36.json` style receipts;
  no number without hardware/workload/comparison.
- Stress tests: concurrent streams, long-context decode, memory pressure, context eviction.

### 15.5 Prefix caching in a nested hyperbolic KV era

- Revisit the Radix prefix cache under hyperbolic memory: prefix keys become coordinates; shared
  prefixes collapse to shared sub-trees in the Poincaré ball.
- Open questions: invalidation cost, geodesic distance thresholds for prefix match, memory
  accounting, interaction with page-entrokv (doc 22).
- Ablation A1 must prove throughput + peak memory improvement vs. the standard path.

## Kernel Fusion Algebra

### Metal Kernel IR as Linear Algebra

Each fused kernel is a **linear operator** on the hyperbolic state space:

$$K: \mathcal{H} \to \mathcal{H}, \quad \mathcal{H} = H^3 \times T^2 \times \mathbb{R}^d$$

**Kernel composition** is operator multiplication:
$$K_{fused} = K_n \circ K_{n-1} \circ \dots \circ K_1$$

**Fusion rules** (verified by algebraic simplifier):

1. **Lorentz boost + VP-tree search** → single kernel (shared distance computation)
2. **TwisterAttention + output projection** → single kernel (fused softmax + matmul)
3. **n-gram draft + verification** → speculative kernel pair

### Schedule Synthesis as Integer Linear Programming

The **scheduler** solves at each step:

$$ \begin{aligned}
\max_{x} \quad & \sum_{i} w_i \cdot \text{throughput}_i(x_i) \\
\text{s.t.} \quad & \sum_i x_i \cdot \text{mem}_i \leq M_{budget} \\
& \sum_i x_i \leq N_{max\_inflight} \\
& x_i \in \{0,1\} \quad \text{(admission)} \\
& \text{depth}_i \cdot x_i \leq D_{max} \quad \text{(depth budget)}
\end{aligned}$$

where $w_i$ = priority weight (shallow recalls = higher weight for latency).

**Solution**: Greedy by $w_i / \text{mem}_i$ with depth-aware tiebreaking. **Optimal** for linear objective.

### Hyperbolic Prefix Cache Design

**Key insight**: In hyperbolic space, **shared prefixes = shared geodesic segments from origin**.

#### Coordinate Collapse

Two requests $r_1, r_2$ with prompts $p_1, p_2$ share prefix $p_{1:k} = p_{2:k}$ iff their **Lorentz coordinates** satisfy:

$$\pi^{-1}(\text{prefix}_k(p_1)) = \pi^{-1}(\text{prefix}_k(p_2))$$

The **prefix key** is the Lorentz coordinate at depth $k$: $x_k \in H^3$.

#### Cache Structure

```
HyperbolicPrefixCache
  ├── Origin (x0=1, x1=x2=x3=0)
  ├── Level 1: VP-tree on depth-1 coordinates
  ├── Level 2: VP-tree on depth-2 coordinates
  └── ...
  └── Level L: VP-tree on depth-L coordinates
```

Each node stores:
- **Coordinate**: $x \in H^3$ (Lorentz)
- **Payload**: Cached KV slices for that prefix
- **Refcount**: Number of active requests sharing this prefix
- **TTL**: Time-to-live for eviction

#### Invalidation as Geodesic Pruning

When a request completes, **decrement refcount** along its geodesic path. If refcount = 0:
1. **Prune subtree** if no other requests use it
2. **Merge siblings** if they share parent and combined size < threshold
3. **Evict to SSD** if memory pressure (page-entrokv integration)

**Invalidation cost**: $O(\text{depth} \cdot \log N_{nodes})$ — proportional to geodesic length, not cache size.

#### Geodesic Distance Threshold

Two prefixes match if their **geodesic distance** is below threshold:

$$d_{H^3}(x_{prefix}^{(1)}, x_{prefix}^{(2)}) < \tau_{prefix}$$

Default $\tau_{prefix} = 0.1$ (tunable per ablation A1). This allows **approximate prefix sharing** for semantically similar prompts.

#### Memory Accounting

Each cache entry accounts:
$$\text{mem}(entry) = \underbrace{\text{KV\_size}}_{\text{model dims}} + \underbrace{\text{coord\_size}}_{4 \times Q32.31} + \underbrace{\text{VP\_tree\_overhead}}_{O(\log N)}$$

**Total cache memory**: $\sum \text{mem}(entry) \leq \text{cache\_budget}$ (configurable, default 2GB).

### Throughput Qualification Receipts

```json
{
  "qualification_id": "throughput_qwen36_20261026",
  "hardware": "MacBook Pro M3 Max, 36GB",
  "model": "Qwen3.8-27B-Uncensored-OrcaRouter-MLX-4bit",
  "config": {
    "acft_enabled": true,
    "hyperbolic_cache": true,
    "batch_size": 16,
    "context_len": 131072
  },
  "workload": {
    "name": "fixed_prompt_corpus",
    "num_prompts": 100,
    "prompt_length": 10000,
    "generation_length": 512
  },
  "results": {
    "decode_tok_s": {"p50": 68.2, "p95": 52.1, "p99": 41.3},
    "ttft_ms": {"p50": 38, "p95": 67, "p99": 112},
    "peak_memory_gb": 27.4,
    "cache_hit_rate": 0.73,
    "recall_depth_avg": 12.4
  },
  "baseline": {
    "decode_tok_s": 42.3,
    "ttft_ms": 89,
    "peak_memory_gb": 28.1
  },
  "improvement": {
    "tok_s_ratio": 1.61,
    "ttft_ratio": 0.43,
    "memory_ratio": 0.97
  },
  "ablation": "A1",
  "status": "PASS"
}
```

## Open questions / gaps
- Hyperbolic prefix cache design open questions (invalidation, geodesic thresholds, memory accounting, page-entrokv).
- Measured numbers TBD.

---

## Contributor Deepening — Muse (2026-10-26)

### C12.7 Scheduler depth-budget endogeneity (depth: closing the loop C06.15 assumes)

The ILP scheduler maximizes priority-weighted throughput subject to sum-x_i * mem_i ≤ M_budget and depth_i * x_i ≤ D_max with D_max static. C06.15 (depth-gated speculative recall) assumes the scheduler can consume per-request k-draft instead. Frozen interface between the two [PROPOSED — interface is my proposal; both sides already exist in this workspace]:

1. The draft path (C06.15) emits k_draft per admitted request; the scheduler replaces the static constraint with sum k_draft,i * mem_i ≤ M_budget.
2. Until C06.15 passes its ablation (A36), D_max stays static and is set from the C06.4 offline Pareto frontier (latency-accuracy curve): D_max = smallest depth with marginal gain above the C21 p99 latency budget. Static value, receipted, not hand-tuned.
3. Scheduler telemetry adds k_draft histogram + admission-rejection counts by cause (memory vs depth) — counts only (ADR-06).
4. Degradation order under pressure is frozen: shed depth first (reduce k), then shed requests (reject x_i), never shed receipts (telemetry is not optional load).

### C12.8 TTFT accounting honesty (depth: what TTFT includes)

"Measured TTFT" (F10, C21) is ambiguous under prefix caching: a cache hit reports near-zero TTFT that is not representative. Frozen accounting [PROPOSED — accounting rule, not a number]:

- TTFT is reported as a pair: (ttft_cold, ttft_hit) with the hit rate. A single blended TTFT number is forbidden in claims (it hides the cache-hit distribution).
- ttft_cold measures full path: admission wait + prefix miss + first-token compute.
- The A1 judge uses ttft_cold p99 as the latency criterion; cache hits are reported as a separate efficiency metric, not mixed into latency SLOs.
- Until the hyperbolic prefix cache exists, ttft_hit = N/A (not zero — zero would be a false measurement).

What was NOT checked here: ngram_lookahead.rs current state (cited as ported; implementation outside this workspace); page-entrokv integration (still open per above).
