# C30 — Multi-Agent Lanes & Autorunner

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Configurable lane roster (N data-driven lanes) with per-lane tool allow-lists, RAG collections, and approval policies. Autorunner with ready-packet admission control. Subagent loops in isolated holes with depth/budget caps. Cross-lane comms via geodesic bridges on H³×T² with interference measurement.

## Subcategories (broad topics)

- C30.1 N data-driven lanes (not fixed 3)
- C30.2 Per-lane tools, RAG collection, approval policy
- C30.3 Autorunner (N workers, ready-packet admission)
- C30.4 Subagent loops (holes, depth/budget caps)
- C30.5 Branch isolation (holes/worktrees)
- C30.6 Cross-lane comms and Holonomy Brigade (submanifolds, geodesic bridges)
- C30.7 Multi-agent submanifolds and interference measurement

---

## Canonical Decisions (frozen for design)

| Decision                                                                               | Rationale                                                      |
| -------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| **Lane roster = data-driven** (N lanes) in `configurations.json`                       | No fixed 3-slot assumption; extensible; forward-only migration |
| **Per-lane config**: tools allow-list, RAG collection, approval policy, brigade radius | Isolation; least privilege; measurable interference            |
| **Autorunner** = work-stealing scheduler with ready-packet admission                   | Throughput; fairness; backpressure                             |
| **Subagent loops** in holes with depth cap (default 3) and budget cap (tokens/time)    | Containment; no runaway recursion                              |
| **Branch isolation** via git worktrees + hole sandbox                                  | Reproducible state; no cross-branch contamination              |
| **Cross-lane comms** = geodesic bridges on H³×T² with logged distance                  | Interference as measurable quantity; circuit breaker trigger   |
| **Holonomy Brigade** (C06.12): single weight set, N views via parallel transport       | Weight copy invariant = 1                                      |

---

## Mathematical Specification

### 1. Lane Configuration as Typed Record

A **lane** `L` is a tuple:

```
L = (id: LaneId,
     name: String,
     description: String,
     tools_allowlist: Set<ToolId>,
     rag_collection: CollectionId ⊕ None,
     approval_policy: ApprovalPolicy,
     brigade_radius: ℝ⁺,
     submanifold_anchor: Point(H³ × T²),
     max_concurrent: ℕ,
     enabled: Bool)
```

**ApprovalPolicy** = `Map<EffectLevel, ApprovalMode>`
where `ApprovalMode ∈ { Auto, Explicit, Human }`

**Configurations.json** schema (forward-only migration via `config_version`):

```json
{
  "config_version": 3,
  "lanes": [
    {
      "id": "builder",
      "name": "Builder",
      "description": "Code generation and modification",
      "tools_allowlist": ["fs.read", "fs.write", "git.commit", "code.parse", "code.query"],
      "rag_collection": "codebase",
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "auto",
        "WriteRemote": "explicit",
        "Destructive": "human"
      },
      "brigade_radius": 0.5,
      "submanifold_anchor": { "h3": [1.0, 0.0, 0.0, 0.0], "t2": [0.0, 0.0] },
      "max_concurrent": 4,
      "enabled": true
    },
    {
      "id": "contributor",
      "name": "Contributor",
      "description": "Review, test, and documentation",
      "tools_allowlist": ["fs.read", "git.diff", "git.status", "web.search", "code.hover"],
      "rag_collection": "docs",
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "explicit",
        "WriteRemote": "human",
        "Destructive": "human"
      },
      "brigade_radius": 0.3,
      "submanifold_anchor": { "h3": [1.0, 0.0, 0.0, 0.0], "t2": [π, 0.0] },
      "max_concurrent": 2,
      "enabled": true
    }
  ],
  "default_lane": "builder"
}
```

**Migration**: `config_version` increments; migration function `migrate_vN_to_vN+1` pure; applied at boot (C14.6).

### 2. Autorunner as Work-Stealing Scheduler

**Autorunner** state:

```
A = (workers: N, queue: ReadyQueue, running: Map<WorkerId, Task>, backpressure: BackpressureState)
```

**ReadyPacket** = `(task_id: TaskId, lane_id: LaneId, priority: Priority, estimated_cost: CostEstimate, dependencies: Set<TaskId>)`

**Admission control**: Packet admitted iff:

1. `lane.enabled = true`
2. `running[lane_id].count < lane.max_concurrent`
3. `backpressure.allows(lane_id, estimated_cost) = true`
4. All `dependencies` resolved (completed or failed)

**Work-stealing**: Idle workers steal from other lanes' queues if `lane.enabled` and `backpressure.allows`.

**Backpressure** (C21.4): PI controller on `tokens_per_second` and `memory_pressure`:

```
error = target_throughput - current_throughput
integral += error * dt
adjustment = Kp * error + Ki * integral
backpressure.throttle = clamp(adjustment, 0.1, 2.0)
```

### 3. Subagent Loops as Bounded Hole Executions

**SubagentLoop** = `(hole: Hole, depth: ℕ, budget: Budget, parent: TaskId ⊕ None)`

**Hole** (C25): Seatbelt sandbox with capability subset, resource limits, Merkle receipt on exit.

**Depth cap**: `depth ≤ MAX_DEPTH` (default 3). Incremented on each `PreSubagentSpawn` hook.

**Budget cap**: `budget = (max_tokens: ℕ, max_time_ms: ℕ, max_tool_calls: ℕ)`. Decremented on each tool call; exhausted → `BudgetExhausted` error.

**Invariant**: `∀ subagent: depth ≤ 3 ∧ budget.tokens ≤ 50,000 ∧ budget.time ≤ 300,000`

**Loop semantics**:

```
loop(parent_task):
  if depth > MAX_DEPTH: return DepthExceeded
  if budget.exhausted: return BudgetExhausted
  packet = autorunner.admit(next_ready_packet(parent_task.lane))
  if packet = None: return NoWork
  result = hole.execute(packet.task)
  emit SubagentReturn(parent_task, result)
  budget.consume(result.cost)
  depth += 1
```

### 4. Branch Isolation via Worktrees + Holes

**Branch** `B` = `(worktree_path: Path, hole: Hole, base_commit: CommitHash, lane_id: LaneId)`

**Isolation invariants**:

1. **FS isolation**: Worktree at `worktree_path` is separate git worktree; no shared working directory
2. **Hole isolation**: Each branch gets fresh hole with lane's capability subset
3. **State isolation**: Branch writes to hole-local DB; merge = explicit `PostMerge` hook with receipt
4. **No cross-branch reads**: Branch `B_i` cannot read `B_j`'s worktree or hole state

**Merge protocol** (C25.3):

```
merge(B_source, B_target):
  require approval_policy[Destructive] = human (or explicit with judge)
  receipt = hole.execute(merge_operation)
  if receipt.verified:
    target.worktree.apply(receipt.diff)
    emit MergeReceipt(source, target, receipt)
  else:
    emit MergeFailed(source, target, receipt.error)
```

### 5. Cross-Lane Comms as Geodesic Bridges

**Lane submanifold** `M_i ⊂ H³ × T²` anchored at `L_i.submanifold_anchor` with radius `L_i.brigade_radius`.

**Geodesic bridge** between lanes `i, j`:

```
bridge(i, j) = γ: [0, 1] → H³ × T²
  where γ(0) ∈ M_i, γ(1) ∈ M_j
  and γ is minimizing geodesic
```

**Bridge distance**: `d_bridge(i, j) = length(γ) = d_{H³×T²}(M_i, M_j)`

**Interference metric** (C06.5):

```
I(i, j) = inf_{x ∈ M_i, y ∈ M_j} d_{H³×T²}(x, y)
```

Logged in `/events` ledger (C31.1) on every cross-lane message.

**Circuit breaker** (C22.3): If `I(i, j) < τ_interfere` (default 0.1), lane `i` and `j` communication paused; `InterferenceBreach` event emitted.

**Message passing**:

```
send(i, j, msg):
  require I(i, j) ≥ τ_interfere
  bridge = compute_bridge(i, j)
  receipt = hole_i.execute(transport_along(bridge, msg))
  hole_j.receive(receipt.payload)
  emit CrossLaneMessage(i, j, bridge.length, receipt.hash)
```

### 6. Holonomy Brigade: Single Weight Set Invariant

**Global weight set** `W` (single copy in VRAM).

**Brigade specialist** `i` receives **fiber projection**:

```
W_i = P_{γ_i}(W|_{U_i})
```

where:

- `U_i ⊂ H³ × T²` is lane `i`'s submanifold
- `γ_i` is geodesic from origin to `U_i`
- `P_{γ_i}` is parallel transport along `γ_i`

**Dissolution**: On task completion,

```
W = ⊗_i P_{γ_i^{-1}}(W_i)
```

**Invariant**: `weight_copies = 1` at all times.

**Verification** (A4): Under N=8 swarm, `weight_copies` measured via VRAM allocation = 1.

---

## Verification Gates (C15)

| Gate                     | Test                                            | Threshold |
| ------------------------ | ----------------------------------------------- | --------- |
| Lane config migration    | `config_version` N → N+1 preserves all lanes    | 100%      |
| Autorunner admission     | Packets admitted only when all 4 conditions met | 100%      |
| Subagent depth cap       | Depth > 3 → DepthExceeded (1000 trials)         | 100%      |
| Subagent budget cap      | Budget exhausted → BudgetExhausted              | 100%      |
| Branch isolation         | Cross-branch FS/hole reads denied               | 0 leaks   |
| Interference logging     | Every cross-lane msg logs `I(i,j)`              | 100%      |
| Circuit breaker          | `I(i,j) < 0.1` → comms paused + event           | 100%      |
| Brigade weight invariant | VRAM weight copies = 1 under N=8 load           | 100%      |
| Merge receipt            | All merges produce verified receipt             | 100%      |

---

## Preregistration Entries (C09)

### A17 — Lane Configuration Migration

- Hypothesis: Forward-only migration preserves lane semantics across versions.
- Judge: Migrate v1→v2→v3; all lanes functional; no data loss.
- Status: TBD.

### A18 — Autorunner Throughput Fairness

- Hypothesis: Work-stealing achieves ≥90% lane utilization fairness under load.
- Baseline: Round-robin scheduler.
- Judge: Jain's fairness index ≥ 0.9 across lanes.
- Status: TBD.

### A19 — Subagent Containment

- Hypothesis: Depth/budget caps prevent runaway subagent recursion.
- Judge: 10,000 malicious subagent spawns; 0 exceed caps.
- Status: TBD.

### A20 — Interference Measurement Accuracy

- Hypothesis: `I(i,j)` computed from geodesic distance matches empirical recall interference.
- Judge: Correlation ≥ 0.8 between `I(i,j)` and cross-lane recall error.
- Status: TBD.

### A21 — Brigade Weight Invariant

- Hypothesis: Single weight set maintained under N=8 concurrent brigade tasks.
- Judge: VRAM allocation shows 1 copy; per-task latency < standard spawn.
- Status: TBD.

---

## Open Questions / Gaps

- Lane hot-reload without session restart (requires config hot-swap proof).
- Dynamic brigade radius adjustment based on task complexity (C06.7).
- Cross-lane RAG sharing with provenance (tied to C26/C28).

---

## Contributor Deepening — Muse (2026-10-26)

### C30.8 Work-stealing vs lane isolation (depth: the scheduler contradicts the breaker)

Canonical decisions state BOTH "work-stealing: idle workers steal from other lanes' queues" AND "branch isolation: no cross-branch reads; lane allowlists; breaker isolation." Unrestricted work-stealing violates all three: a thief worker from lane j executing lane i's packet inherits lane i's caps — or worse, executes with lane j's caps on lane i's data [INFERRED — composes the C30.2 work-stealing rule with the C30.1 allowlist + C30.4 isolation invariants; ends at those sections]. Frozen stealing rule [PROPOSED — rule is my proposal]:

1. A stolen packet executes under the ORIGIN lane's allowlist + approval policy + RAG scope, never the thief's. The worker is compute; the packet is authority. Stealing without authority transfer is privilege escalation (C16).
2. No stealing FROM isolated lanes (breaker-open, C22.2) or TO isolated lanes: isolation means no new packets admitted, including stolen ones (C27.8 same principle).
3. No stealing across RAG scopes unless cross-lane RAG sharing (open question above) is resolved: a builder-lane worker stealing a reviewer packet must not gain reviewer-collection reads via the packet. Until resolved, stealable = same-RAG-scope lanes only.
4. Every steal emits a CrossLaneMessage-class receipt (thief lane, origin lane, packet hash, authority-provenance) — the interference metric I(i,j) (C06.5) counts steals as contact. Unlogged stealing is unmeasured interference.

### C30.9 max_concurrent vs admission control (depth: two throttles, one queue)

Admission requires running[lane].count < lane.max_concurrent AND backpressure.allows(...) AND deps-resolved. Two independent throttles (static cap + PI controller) fight: the PI controller (C21) throttles globally on memory pressure while max_concurrent throttles per lane — under pressure the controller says stop and the cap says room, or vice versa. Frozen precedence [PROPOSED — precedence is my proposal]:

1. backpressure.allows is NECESSARY (hard gate): no admission when memory pressure exceeds target, regardless of per-lane room. OOM risk (R2, score 20) outranks lane utilization.
2. max_concurrent is the FAIRNESS bound within the backpressure envelope: it prevents one lane starving others, not OOM. Its tuning metric is Jain's fairness (A18), not memory.
3. The A18 fairness judge must run UNDER memory pressure (backpressure active), not on an idle machine — fairness at zero load is trivially 1.0 and evidences nothing.

What was NOT checked here: autorunner implementation (outside workspace); whether backpressure.allows is currently consulted before or after the cap check (verify at implementation).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Multi-Agent Lanes (Original)

- **N data-driven lanes** in `configurations.json` (not fixed 3).
- **Per-lane**: tools allow-list, RAG collection, approval policy, brigade radius.
- **Autorunner**: N workers, ready-packet admission, work-stealing.
- **Subagent loops** in holes with depth/budget caps.
- **Branch isolation** via holes/worktrees.
- **Cross-lane comms** via geodesic bridges; interference measured.
- **Holonomy Brigade**: single weight set, N views via parallel transport.
