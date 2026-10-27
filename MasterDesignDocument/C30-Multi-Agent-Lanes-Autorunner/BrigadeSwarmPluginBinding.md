# C30.7 — BrigadeSwarmPlugin Binding: The Only Legal Agent Invocation Path

**Status.** PROPOSED contract. **Date:** 2026-10-27. **Parent:** C30 Multi-Agent Lanes, C06.12 Holonomy, C06.21 Reasoning Distillation.

---

## 1. Core rule (hard gate)

An agent **may not** be spawned, messaged, or granted a tool surface unless the Meta-orchestrator has invoked the **`BrigadeSwarmPlugin`** (registered as a `CustomTooling` tool in the skill/tool registry). There is **no fallback path**: absent the plugin, the invocation is rejected with a receipt, not silently degraded.

```
MetaOrchestrator::handle_task(task)
  ├─ invoke skill "task-decomposition-and-assign.md"
  ├─ scope → TaskCarvedView (C06.21, §3)
  ├─ pin result to top of worktree as Agents.md
  └─ iff all gates pass: call BrigadeSwarmPlugin::spawn(spec)
       else: emit refusal receipt, no agent
```

## 2. What the plugin bundles (all required, all active)

`BrigadeSwarmPlugin` is a **single composite tool** that bundles three sub-capabilities; partial activation is forbidden:

| Sub-capability | Responsibility |
|---|---|
| `HyperSpecializedSpecOpsBrigade` | Lane roster, specialization, fiber projection/dissolution (C06.12), weight-copy invariant |
| `TelemetryAndObservabilityMCP` | Process + persisted telemetry, taps, receipts, metrics (C31) |
| `SpecOpsTooling` | Typed tool registry + effect system + hooks for this brigade only (C29/C28) |

If any sub-capability fails its boot check, the whole plugin fails closed (no partial swarm).

## 3. SpecOpsAgent assignment (each agent gets)

1. **Workspace tree** — a dedicated worktree/branch root (C25 hole substrate; branch isolation).
2. **NestedHyperbolicKVCache + mini Hyperbolic Global Map** — a task-scoped carved map `M_T` (C06.21) acting as the agent's only addressable memory; global map remains single and shared.
3. **Agents.md pinned at top of the worktree** — the decomposed, scoped task output (see §4). This is the agent's contract of engagement; it is immutable during the run except via dissolution.

Without (1)–(3), an "agent" is not a SpecOpsAgent and must not execute agentic turns; it collapses to a stateless model call with no trajectory.

## 4. Task → scope → Agents.md pipeline

Meta-orchestrator executes the skill **`task-decomposition-and-assign.md`**:

1. Classify task → class `C` (C06.21 §5.1).
2. Select silos, basins, atoms; compute `λ_map`; carve `M_T`.
3. Build the **minimal necessary payload** `I_nec` (exact critical state via Continuity Capsule + carved map digest).
4. Render `Agents.md` with: objective, invariants, acceptance, scope class, allowed silos, atom digest ids, `λ_map`, budget, tool surfaces, refusal conditions.
5. Pin `Agents.md` to the top of the assigned worktree (revision-gated; immutable mid-run).
6. Hand the pinned artifact + `M_T` handle to `BrigadeSwarmPlugin::spawn`.

The agent reads `Agents.md` first on every turn; if `Agents.md` is missing or hash-mismatched, the agent turn is void (refusal receipt).

## 5. Invariants (non-negotiable)

- **I-BSP-1:** `agent_spawn ⇒ brigade_plugin_invoked ∧ brigade_plugin_boot_ok`. No exceptions.
- **I-BSP-2:** One global map `M_G`; each agent sees only its carved `M_T` (fiber/view), with `weight_copies = 1` for the base model.
- **I-BSP-3:** `Agents.md` is the single source of scoped truth for the agent; the agent may not widen scope beyond it without dissolution + replan.
- **I-BSP-4:** Every invocation emits a receipt terminating in a ledger root (C06.3/C06.11); "agent did work with no receipt" is a ship-blocker.
- **I-BSP-5:** On any plugin failure (boot, interference `I < 0.1`, weight-copy >1), the breaker opens and the agent is quarantined, not retried blindly.

## 6. Failure model without the binding

Absent the plugin or any of (workspace, nested cache, mini-map, pinned Agents.md), the "agent" devolves into an untuned, unscoped model call. Expected consequences: more tool calls, more retries, more errors, higher token spend, higher hallucination rate, no provenance, no dissolution. This is precisely what A39 measures on the map axis; the plugin binding is the *organizational* counterpart to that measurement.

## 7. Ablation ownership

A dedicated registration is required to earn the binding claim (per CORR-C06-03, one ablation per mechanism). Proposed: **A44 — BrigadeSwarmPlugin binding A/B** (renumbered 2026-10-27: A40 is dream gain per certification ruling §8; one ablation per mechanism per CORR-C06-03):

- Arm A: orchestrator spawns agents directly (no plugin, no carved map, no pinned Agents.md).
- Arm B: orchestrator uses `BrigadeSwarmPlugin` with all §3 artifacts.

Judges: spawn-receipt coverage, tool-call count, retry count, error rate, token spend, output quality, dissolution success, and `weight_copies = 1`.

**Status:** proposed; not yet filed. Filing required before any run (C11.3).

---

_Last updated: 2026-10-27 (plugin-binding pass)._
EOF
