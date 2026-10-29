soft j# C30.8 — Meta Soft-Budget, Profitability, and Hidden-Variational Control

**Status.** PROPOSED economic/active-inference contract. **Date:** 2026-10-27. **Parent:** C21 Performance Budgets, C30 Lanes, C31 Telemetry, C06.21 Carving.

---

## 1. Distinction: hard budget vs soft budget

- **Hard budget (C21):** physical, enforced limits — memory ceiling, KV cap, wall-clock caps, OOM guards. Visible-ish through circuit breakers.
- **Soft budget (this doc):** a **Meta-only** preallocation used as a baseline and decision reference: `(B_tok, B_tools, B_quality, B_security, B_adapt, B_time)`. The Meta sets it per task/agent from the task scope, but **the agent never sees it**.

The soft budget is not an allowance to spend; it is a **reference point for data-driven Meta control and profitability**.

## 2. Formal objects

For an assigned task `T` and agent/branch `a`:

- `B_a^soft = (b_tok, b_tools, b_quality, b_security, b_adapt, b_time)` — Meta preallocation.
- `x_a(t) = (tok_used, tools_used, quality_now, security_now, adapt_now, time_used)` — observed execution state.
- `F_a(t)` — agent’s variational free energy (local active inference over its carved map): `F = complexity − accuracy` (ACFT/C05).
- `π_a(t) = b_tok·unit_tok + b_tools·unit_tool + ...`? — profit function.

**Profit (Meta perspective):**

$$
\Pi_a = \text{value}(B_a^{\text{soft}}) - \text{cost}(x_a^{\text{final}})
$$

where value is a function of quality, security, and adaptability targets (higher is better), and cost is token/tool/time spend (lower is better). A positive `Π_a` is “profitable”; zero/negative is not.

**Agent objective (local, unaware of `B`):**

$$
\min_{\pi_{\text{agent}}} \mathbb{E}[F_a(t)] + \lambda_{\text{cost}} \cdot \text{resource\_cost}(x)
$$

The agent minimizes its own free energy plus an intrinsic resource cost. It does **not** target the Meta’s soft budget, and must not be able to infer it.

## 3. Information asymmetry (anti-gaming)

- `B_a^soft` is **never** present in `Agents.md`, `TaskCarvedView`, tool responses, telemetry visible to the agent, or prompts.
- The agent’s reward signal is limited to task-level feedback (pass/fail, quality delta) and its own free-energy gradient. It never receives “budget remaining.”
- The Meta computes `Π_a` in a separate control plane from persisted telemetry (C31) and posts updates only to the Meta map / reliability ledger, never to the agent.

This prevents Goodharting: if the agent knew the budget, it could spend exactly to the cap or optimize the proxy.

## 4. Meta control loop (swarm in motion)

```
while swarm active:
  observe telemetry x_a(t) (counts, ids, ratios — never content, C24.8/C31)
  compute F_proxy_a(t) (free-energy trend) and spend trend
  compute Π_a (final or running estimate)
  if Π_a trend < 0 for k ticks:
     tighten scope (smaller M_T, lower λ_map, fewer tools)
     or quarantine branch (AA-02/AA-06 guardrails)
  if high profit & high quality:
     increase agent reliability/affinity weight in Meta map
     relax scope slightly (exploration) bounded by hard budget
  update Meta soft-budget priors (Bayesian) for future tasks
```

Meta adjustments affect **future** tasks/allocations and scope, never the agent’s current hidden objective.

## 5. Invariants

- **I-SB-1:** Agent never observes `B_a^soft` in any channel (prompt, map, tool result, telemetry echo).
- **I-SB-2:** Profit `Π_a` is computed only by the Meta control plane; the agent’s reward is task utility only.
- **I-SB-3:** Soft budget never overrides hard budgets; if `B` and C21 conflict, C21 (physical) wins.
- **I-SB-4:** Meta may tighten scope mid-run only through dissolution (§3.2 of ReasoningDistillationMapCarving.md); never through hidden prompt injection.
- **I-SB-5:** Privacy non-interference (C06.17 M7) applies: no raw objective/invariant/commitment text appears in any telemetry used for `Π`.

## 6. Telemetry fields (counts/ids/ratios only)

Meta needs at least:

- `tokens_used`, `tool_calls`, `retries`, `elapsed_ms`
- `pass_rate`, `quality_score`, `security_violations`, `adaptation_events`
- `free_energy_proxy` (scalar trend)
- `scope_size_atoms`, `lambda_map_effective`
- `profit_estimate` (Meta-only; never echoed to agent)

## 7. Reward allocation

- A profitable agent (high quality, secure, adaptive, low spend) increases its **reliability weight** in the Meta map; it gets preferred for similar future tasks.
- An unprofitable agent is not “punished” by prompt scolding (that would leak the budget); instead, the Meta reduces its future scope/allocation, and the breaker may quarantine.
- The global model/weights are unchanged (one weight set); only routing/allocation priors move. This keeps `weight_copies = 1`.

## 8. Ablation ownership

Proposed **A41 — Soft-budget profitability A/B**:

- Arm A: no Meta soft budget / no profitability routing (baseline).
- Arm B: Meta soft budget + hidden control loop + reliability-weight routing.

Judges: pass rate, quality, security, tool calls, retries, tokens, time, and `Π` improvement over a task corpus; confound control for token totals as in A39 §8.6.

**Status:** proposed; not filed. One ablation per claim (CORR-C06-03).

---

_Last updated: 2026-10-27 (soft-budget/profitability pass)._
EOF
