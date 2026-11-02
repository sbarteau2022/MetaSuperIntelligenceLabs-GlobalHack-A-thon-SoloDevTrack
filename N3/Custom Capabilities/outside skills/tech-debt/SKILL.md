---
name: tech-debt
description: Find it, price it, pay the right part. Evidenced inventory, leaving-vs-fix pricing, payoff ranking, safe payment (tests first, one change per commit), prevention checks so it never returns.
---

# tech-debt (interface)

## Purpose
Priced debt ledger: inventory with file-path evidence (dead/duplicated/orphaned
code, never-failing or flaky tests, slow builds, undocumented steps, stale deps),
leaving-cost vs fix-cost + fix-risk per item, ranking by leaving/fix payoff, cheap
high-value first, dated deferrals with reasons, safe payment, prevention checks.

## When to use
Debt passes and after-incident hardening. Small safe payments continuously; never
mix refactor with behavior change.

## Out of scope
Unpriced lists. Expensive low-value work without a written deferral reason.
Fix-risk ignored. Payment without pinned behavior (tests first, full suite each
commit). Debt paid without the check that stops its return.

## Type signature
Compiled gate: `tool.rs` (`techdebt-gate`, schema: `tool.json`).
- `techdebt_gate(cfg: TechDebtItem) -> bool` — evidenced AND priced AND ranked AND pinned
- `make_techdebt(evidenced, priced, ranked, pinned) -> TechDebtItem`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/tech-debt.md`
(`f853f2032c15345d1c13baaa37b9805ee50fb93f237eccb28b2e97a01b58050e5`).
Shrunk to this interface 2026-10-31 (Agent A2). No tool/server claims in source;
nothing removed. See `RECEIPT.md`.
