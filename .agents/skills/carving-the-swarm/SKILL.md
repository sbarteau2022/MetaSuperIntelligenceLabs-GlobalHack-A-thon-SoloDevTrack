---
name: carving-the-swarm
description: Carve task-scoped specialist views from the global HyperMind map via map.carve, verify necessity and fingerprint, dissolve via map.dissolve. Use when isolating work for a subagent without copying weights or duplicating the map.
---

# CarvingTheSwarm

Carve a task-scoped fiber view from the global map `M_G`. Basins are views
over atom ids — never copies. Weights are never duplicated (`weight_copies == 1` always).

## When to use

A task needs a subset of the map (one domain, one incident, one feature).
Carve it, hand the fiber to the worker, dissolve the deltas back. Never hand
the whole `M_G` to a subagent.

## Procedure

1. **Classify the task.** `task_class(task)` maps text to one of:
   `C_code | C_memory | C_workflow | C_research | C_security | C_general`.
   Non-general classes restrict to their required domains; `C_general` spans all.
2. **Carve.** Call `map.carve` with `task`, `context_budget` (default 4096),
   `token_pressure` (default 0.2). The server:
   - scores candidate basins by `relevance × salience × geo_gate × necessity`,
   - selects with the data-dependent depth gate (same variational form as
     `map.attend`: `cost = ln(k) + k − mass×8`),
   - keeps `necessity ≥ θ_N = 0.50` plus pinned basins in scope (I4),
   - truncates to atom budget, pinned-first (pinned are never dropped).
3. **Verify the carve receipt.** Check: `weight_copies == 1`, no raw atom text
   in any digest, `M_G` atom count unchanged, views heterogeneous across tasks.
   The swarm certification test (`swarm_20_views_certification`, 20 views)
   is the bar — run it if you changed carve logic.
4. **Work inside the fiber.** The fiber id scopes all reads. Salience deltas
   (`intended`) accumulate against fiber atoms only.
5. **Dissolve.** Call `map.dissolve` with `fiber_id` + `deltas`. The server:
   - validates deltas, merges by atom id, recomputes the atlas fingerprint,
   - requires `post == pre + intended` (XOR check) — mismatch voids the
     merge (quarantine + rollback, never partial apply),
   - re-folds basins, frees the view, returns the write-set hash.
6. **Confirm.** Dissolve receipt must show `"dissolved": true`,
   `applied_ops` count, and matching pre/post fingerprints.

## Thresholds (preregistered — retune by sweep, never by feel)

- Necessity gate `θ_N = 0.50`. Basin temperature `τ_b = 1.0`.
- Fold radius for basin membership: `cosh(d) ≤ 1.5` shell-equivalent.
- Depth candidates `[1,2,3,4,5,6,8]`, capped by class depth.

## Failure doctrine

- Carve returns empty (no necessary basins): the task is underspecified.
  Decompose further (see `task-decomp`) — do NOT widen the gate.
- Dissolve voids (fingerprint mismatch): quarantine the fiber, re-read the
  deltas, re-dissolve. Never force-merge.
- Reference: `ORCHESTRATION-PROTOCOL.md` §1 (pipeline), live code
  `carve_swarm_map` / `dissolve_fiber` in `hypermind-nested-kv/src/main.rs`.

## Native-stack note

Carve/dissolve are pure Rust (`BTreeMap` ids, `f64` scores, sha256
fingerprints). No JSON on the hot path — the MCP envelope is JSON-RPC at
the boundary only; the fiber math is native. Python harnesses
(`distill_ceiling.py`, `a39_heldout.py`) drive the server, they never
implement carving. TS7.1-beta and C++ ports consume the same receipts;
they do not re-derive them.
