---
name: architecture
description: Decide the structure, write down why. Requirements, constraints, >=2 options plus do-nothing, numeric comparison, one-sentence decision, named boundaries, riskiest slice proven first. Output is a decision record, not diagrams.
---

# architecture (interface)

## Purpose
Structure decisions as checkable records: requirements with numbers, constraints,
options with cost/ease/failure each, comparison on requirements (no taste), revisit
triggers, pure-logic core with I/O at edges, one owner per state, explicit
interfaces, contained failures.

## When to use
Choosing between designs, adding a component, drawing a boundary. Prove the
riskiest part with the smallest slice before committing. Record context,
alternatives, consequences next to the code.

## Out of scope
Taste-based comparison. Single-option railroads (minimum two real options plus
"do nothing"/"smallest change"). Unknowns without a cheap way to resolve them.

## Type signature
Compiled gate: `tool.rs` (`arch-decision-gate`, schema: `tool.json`; review
verdict helper shared with code-review).
- `options_enough(options_count: number) -> bool` — >= 2
- `arch_gate(cfg: ArchDecision) -> bool` — requirements AND options>=2 AND decided AND risky-sliced
- `review_verdict(blockers: number, should_fix: number) -> string` — "do-not-merge" | "approve-with-changes" | "approve"
- `make_arch(requirements_set, options_count, decided, risky_sliced) -> ArchDecision`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/architecture.md`
(`440e376b47cfa282372453a5c1f370f833b8f32291`). Shrunk to this interface
2026-10-31 (Agent A2). No tool/server claims in source; nothing removed.
See `RECEIPT.md`.
