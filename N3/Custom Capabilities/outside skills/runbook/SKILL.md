---
name: runbook
description: Execute multi-step work under the Agentic Runbook contract: six phases (intake, plan, scope, execute, verify, file), step packets with judges, bounded repair loops, named honest stops. Never grade your own work.
---

# runbook (interface)

## Purpose
The operational form of the Agentic Runbook (`~/Desktop/Special Ops/Meta/07-agentic-runbook.md`
v0.1): restate → plan with judges → scope → execute → verify → file. Output is a CLAIM
until a deterministic judge verifies it. Lane split: PLAN lane (think HIGH, packets +
judges, no code) before CODE lane (execute packets, no re-planning — escalate).

## When to use
Any multi-step run: decompose into step packets (id, intent, inputs, outputs, tools
permitted, judge + pass condition, budget, dependencies, rollback), one tool call per
packet, observe vs expected, Loop C repair (one variable, ≤5 passes/packet, ≤40/run),
named stops filed with evidence, run report with labeled claims.

## Out of scope
Steps without a conceivable verification (redesign or escalate, never run on faith).
Grading your own work. Parallel servers to "help" (queue or escalate). Guessing a
missing binding (loop back and fetch it; two no-progress spins = stop). Irreversible
actions without a filled rollback field. `read_file`/`run_python`/`run_shell` etc.
are host-runtime tools (belt), not compiled claims.

## Type signature
Compiled gate: `tool.rs` (`runbook-packet-gate`, schema: `tool.json`).
- `packet_complete(cfg: RunbookPacket) -> bool` — judge AND pass AND reversible AND budgeted
- `stop_valid(stop: String) -> bool` — unparseable_input|spin_detected|cap_exhausted|judge_unavailable|scope_boundary|irreversible_needs_commander
- `repair_budget_valid(passes: number) -> bool` — 0..5
- `make_packet(judge_set, pass_set, reversible, budget_set) -> RunbookPacket`

## Source
Trash source: `skills-desktop-plugins-20261030/agentic-runbook/skills/runbook.md`
(`8b84fa7522af13ba8a7f1951d36ab209fda45448385a7f1969b8a7317`).
Shrunk to this interface 2026-10-31 (Agent A2); merges into `N3/skills/runbook/`
(no collision: existing `runbook-generator` is a different skill). No callable
schema named in source beyond the host belt; nothing removed. See `RECEIPT.md`.
