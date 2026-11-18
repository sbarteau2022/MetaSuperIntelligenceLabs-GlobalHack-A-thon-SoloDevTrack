---
name: debug
description: Find the cause, not a patch that hides it. Reproduce smallest, failing test first, trace to first wrong value, rank hypotheses, bisect, fix the class, prove it, record one line.
---

# debug (interface)

## Purpose
Root-cause discipline: smallest always-failing repro, failing test as bug proof +
fix proof, quoted error + code-path trace, cheapest-hypothesis-first with
predicted outcomes (surprise = update the model), bisect, class-level fix with
sibling grep, one-line record (symptom, cause, fix, guarding test).

## When to use
Any bug. If it cannot be reproduced, say so — that is the finding, not a guess.

## Out of scope
Catching and ignoring the error, adding a sleep, special-casing the failing
input. Changes that proceed on a guess about intent (list the guess and ask).

## Type signature
Compiled gate: `tool.rs` (`debug-gate`, schema: `tool.json`; `deploy_ready`
helper shared with deploy-checklist).
- `debug_gate(cfg: DebugCase) -> bool` — reproduced AND failing-test AND cause-fixed AND suite-green
- `deploy_ready(suite_green: boolean, rollback_ready: boolean, secrets_ok: boolean) -> bool`
- `make_debug(reproduced, failing_test, cause_fixed, suite_green) -> DebugCase`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/debug.md`
(`cae4a7b052b466104dcffeae941e1349c4a4d`). Shrunk to this interface 2026-10-31
(Agent A2). No tool/server claims in source; nothing removed. See `RECEIPT.md`.
