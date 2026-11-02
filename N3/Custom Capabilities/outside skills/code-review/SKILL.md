---
name: code-review
description: Review the diff, not the author. Correctness against spec first, then edges, state, failures, security, tests, maintainability last. Findings carry file:line, severity, input, fix. Ends with one verdict line.
---

# code-review (interface)

## Purpose
Two-pass review (intent, then attacker) in strict attention order: spec
correctness → edge cases → state/ordering → failure paths → security → tests →
maintainability (never mixed into correctness). Blockers reproduced with runnable
snippets; what was not reviewed is stated.

## When to use
Any change review with the spec/task read first. Severities: blocks (wrong
output, data loss, security) > should fix > nit. Praise only what is specific.

## Out of scope
Author-based review. Defaults hidden in failure paths, swallowed errors,
retry-amplified side effects, untrusted input at shell/path/query/template,
secrets in code/logs, over-wide permissions — all findings, none waived.
Vacuous or circular tests pass no review.

## Type signature
Compiled gate: `tool.rs` (`review-gate`, schema: `tool.json`).
- `verdict_valid(verdict: String) -> bool` — "approve" | "approve-with-changes" | "do-not-merge"
- `review_gate(cfg: ReviewInput) -> bool` — spec-traced AND edges AND repro AND verdict-valid
- `make_review(spec_traced, edges_done, repro_present, verdict) -> ReviewInput`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/code-review.md`
(`694a694d8cab6f182ac6c3a79ab4c5553e2026`). Shrunk to this interface 2026-10-31
(Agent A2). No tool/server claims in source; nothing removed. See `RECEIPT.md`.
