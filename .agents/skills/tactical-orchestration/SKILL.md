---
name: tactical-orchestration
description: Map decomposed leaves to executors with budgets, run them in dependency order with receipts, re-plan on fail-explained. Use when executing a task tree from task-decomp.
---

# TacticalOrchestration

Assign every leaf to exactly one executor with a budget, run in dependency
order, receipt everything. Unassigned leaves fail at plan time, never mid-run.

## When to use

After `task-decomp` emits a tree. This is the tactics + orchestration stage
of `ORCHESTRATION-PROTOCOL.md` §1.

## Procedure

1. **Tactical mapping.** For each leaf, assign ONE of:
   - a map tool (`map.query` = lookup, `map.lexical` = doc-find,
     `map.attend` = budgeted read, `map.traverse` = governed move,
     `map.carve` = isolate work, `map.hunger` = check ignorance),
   - a carved fiber (from `carving-the-swarm`) for multi-step work,
   - a traversal arc (from→to with `mlir_op`) for chained reasoning.
   Record per-leaf: executor + energy budget + token budget + depth budget.
2. **Order by dependency.** Leaves with no prerequisites run first (parallel
   where independent — disjoint write scopes only). A leaf runs when its
   inputs' receipts exist.
3. **Execute with receipts.** Every call returns a receipt (`receipt_sha256`).
   No receipt = the step didn't happen. Ledger the receipts in order.
4. **Handle fail-explained — the core move.** A refusal names cost, budget,
   and the missing piece. Do NOT retry blindly (identical retry is a
   violation — N³ runbook failure doctrine). Instead:
   - over-budget → raise the budget (if the leaf's value justifies it) or
     split the leaf (cheaper hops),
   - form violated → the path doesn't exist; carve the missing arc first
     (distill priority from `map.hunger`), then re-traverse,
   - frame shear → the move deforms; reduce hop size (intermediate atoms).
   Re-plan around the named gap, then continue.
5. **Sundowning watch.** Past ~40 steps, re-check every traversal AT DISPATCH
   (contracts re-verified live — this is automatic in `map.traverse`).
   A mid-plan fail-explained IS the sundowning detector: re-ground (fresh
   query + fresh traverse), never push through a failed check.
6. **Close out.** All leaves green → consolidate (`map.consolidate` with
   traversed ids: reinforce used, decay untouched, sate covered gaps).
   Dissolve fibers. Snapshot.

## Budgets (defaults — override per leaf, record the override)

- Traverse energy budget: 16.0 (default), 100.0 (generous), 0.0 (probe:
  always refuses — use to test the fail path, never in production).
- Frame eps: 1e-2 (default dispatch), 1e-3 (strict audit). Form tol: 1e-9
  (proven non-binding — leave it).
- Attend depth: data-dependent (server picks); explicit `k` only to bound cost.

## Failure doctrine

- Missing receipt mid-plan: halt the branch, re-run the leaf. Never
  continue a plan on an unreceipted step.
- Hunger top-1 matches the current failure: stop and distill first
  (the ledger already told you what's missing — reading it is cheaper
  than failing again).
- Reference: `ORCHESTRATION-PROTOCOL.md` §§1,4,5; `TAP-N-TRACE.md` §5
  (the mlir_op#26 worked example).

## Native-stack note

Orchestration state is native Rust (`BTreeMap` atoms, `Vec` receipts,
`u64` ids). The MCP boundary speaks JSON-RPC; inside, everything is typed
structs with serde only at snapshot. The symplectic envelope
(`TraversalContract`: form + energy + frame) is checked in `f64` before
any dispatch — the check that makes "fail-explained, never fail-silent"
a mechanical property, not a prompt wish. Python drives, Rust decides.
