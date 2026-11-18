---
name: ares-subagents
description: Operate subagent loops inside isolated holes with context/return contracts, fan-in, depth/budget caps, and merge gates. Use when delegating work, debugging subagent failures, or managing hole lifecycle.
---

# Ares Subagents

## Scope
- Substrate: holes (`backend/src/holes.rs`) — isolated execution cavities.
- Fan-in: workflow packets (`backend/src/workflows.rs`).
- Contracts: declared_context + task + judges per packet.
- Depth/budget: max subagent depth 3, token budget per branch.

## When to use
- Delegating work to a subagent or hole.
- Debugging hole runs, merge refusals, or dependency skips.
- Enforcing confinement (seatbelt/fallback) and approval gates.

## Rules
- Subagent edits land in holes, never project root, until merge.
- Merge requires tested_pass + (optional) approval.
- Results are deposited as structured data, never executed commands.
EOF
