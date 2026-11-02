---
name: deploy-checklist
description: Ship without surprises. Before (green clean checkout, risky-change gates, reversible migrations, pinned deps, tried rollback), during (smallest unit, named signals, stop-and-roll-back), after (measured confirmation, recorded undo, deleted flags).
---

# deploy-checklist (interface)

## Purpose
No-surprise shipping: preconditions before, smallest-unit rollout with watched
health signals during, measured confirmation + recorded undo after. Build and
test in the microVM (`vm_exec` is host belt, not a compiled claim); check what
leaves (`._*`, caches, secrets never ship).

## When to use
Every ship. Risky changes additionally carry the mutation check + adversarial
pass and copy-of-real-data migration trials first.

## Out of scope
Unreversible migrations. Config/secrets in the repo. Unpinned dependencies.
Rollouts without a tried rollback step. Recovery "confirmed" by feeling.

## Type signature
Compiled gate: `tool.rs` (`deploy-gate`, schema: `tool.json`).
- `deploy_gate(cfg: DeployPlan) -> bool` — suite-green AND rollback-ready AND secrets-ok AND watch-signals
- `make_deploy(suite_green, rollback_ready, secrets_ok, watch_signals) -> DeployPlan`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/deploy-checklist.md`
(`81a2d989c08e8a358074d20378bd2a09c75b28b2e97`). Shrunk to this interface
2026-10-31 (Agent A2); `vm_exec` noted as host procedure, nothing removed.
See `RECEIPT.md`.
