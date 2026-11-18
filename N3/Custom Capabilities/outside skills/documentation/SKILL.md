---
name: documentation
description: Write what a stranger needs to use and change it. Three-sentence what/is-not, tested clean-machine quickstart, typed API with examples, model + change guide, decisions/limits with evidence. Same-commit doc updates; wrong docs deleted.
---

# documentation (interface)

## Purpose
Stranger-in-a-hurry docs: defined terms, checkable claims, exact commands with
expected output, one idea per paragraph, tables for comparison. Code change =
doc change in the same commit; a wrong doc is worse than none (delete it).

## When to use
Any user- or changer-facing documentation: what it is/is-not, quickstart,
usage (every public fn/command/endpoint typed + error + example), how it works
(state + invariants), how to change it (layout, tests, mutation, conventions,
release), decisions + limits + known bugs.

## Out of scope
Untested quickstarts. Untyped parameters. Examples that do not run as shown.
Undated version/hardware claims. Verified-vs-inferred blending.

## Type signature
Compiled gate: `tool.rs` (`doc-gate`, schema: `tool.json`).
- `doc_gate(cfg: DocPlan) -> bool` — quickstart-tested AND api-typed AND examples-runnable AND decisions-logged
- `make_doc(quickstart_tested, api_typed, examples_runnable, decisions_logged) -> DocPlan`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/documentation.md`
(`d2217cec64a834d853357b69b8f322b572d7`). Shrunk to this interface 2026-10-31
(Agent A2). No tool/server claims in source; nothing removed. See `RECEIPT.md`.
