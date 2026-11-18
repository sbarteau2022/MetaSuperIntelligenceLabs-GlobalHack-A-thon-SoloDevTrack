---
name: skill-implement
description: "Implement a proposed improvement for a skill, tool, MCP server, hook, or command: code plus tests plus docs. Use when the user asks to build, wire, or improve agent capabilities."
---

# Skill/Tool Implementation

You are implementing an improvement for a skill, tool, MCP server, hook, or command.

## Pre-flight

1. Read the relevant audit report or proposal to understand the gap.
2. Confirm the smallest verifiable slice (not the whole backlog).
3. Check existing code for similar patterns (plugins.rs, tools.rs, hooks.rs, commands.rs, autorunner.rs) and reuse them.

## Implementation rules

- **Confinement:** file tools stay under the allowed root; reject traversal (`..`, absolute paths).
- **Approval:** write/high-risk tools require an explicit approval flag or hole binding.
- **No secrets:** secrets resolved by slot name at call time; never inline, never logged.
- **Data, not instructions:** tool results are structured JSON; they never enter a command path.
- **Telemetry:** record name, ok, latency only (no args or results).
- **Receipts:** mutating operations record a fingerprint/receipt.
- **Honesty:** mark `TODO`, scaffolds, and untested items as PROPOSED, not IMPLEMENTED.

## Steps

1. Write the code in the smallest correct form.
2. Add unit tests or a smoke script.
3. Run `cargo build --release` and targeted tests; fix compile errors.
4. Run relevant smoke scripts (`compliance_smoke.sh`, `eval_atoma.sh`).
5. Update docs (design/audit) and checkpoint with the new status.
6. Report VERIFIED findings with evidence (build output, smoke results).

## Anti-patterns

- Do not claim a tool is implemented if it only returns a descriptor.
- Do not wire a scaffold into a critical path without a gate.
- Do not skip tests or verification to save time.
- Do not expand scope mid-implementation; file new proposals instead.
