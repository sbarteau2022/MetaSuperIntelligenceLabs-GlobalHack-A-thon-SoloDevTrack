---
name: plugin-builder
description: Build your own skills, plugins and tools with rigor. Spec first, >=2 hand-computed examples, sandboxed connector, approval, one real use, then the when-to-use skill. Small and single-purpose.
---

# plugin-builder (interface)

## Purpose
Build instead of downloading when the job is specific: skill (procedure/body of
knowledge, `skill_write`, `plugin/skill`), plugin (bundle, `plugin_create`,
enforced sparingly), connector (exact-work Python fn, `connector_write`,
`ARGS`-dict in, print result, `SystemExit(1)` on bad input, 60s, no net/clock/
randomness unless specified).

## When to use
New skill/plugin/connector/tool builds. Spec first, examples before code (one an
edge case, `expect` hand-computed, never copied from output), `connector_write`
runs examples (fail = nothing registered), approval, one real use, then the skill
that tells future-you when to use it.

## Out of scope
Rebuilding what an existing tool already does (ask first if unsure). Enforce rules
by default (each match costs prompt tokens). Editing connector code after approval
(voids approval — re-approve). Network/clock/randomness in sandbox connectors.

## Type signature
Compiled gate: `tool.rs` (`plugin-build-gate`, schema: `tool.json`).
- `kind_valid(kind: String) -> bool` — "skill" | "plugin" | "connector"
- `examples_valid(examples_count: number) -> bool` — >= 2
- `plugin_gate(cfg: PluginBuild) -> bool` — spec AND examples>=2 AND sandbox AND approved AND kind
- `make_plugin(spec_done, examples_count, sandbox_ok, approved, kind) -> PluginBuild`

## Source
Trash source: `skills-desktop-plugins-20261030/design/skills/plugin-builder.md`
(`de3b995da0a6a36a39bad7737d8bab8dc180a34d`). Shrunk to this interface 2026-10-31
(Agent A2). `plugin_create`/`connector_write`/`skill_write` are host belt tools,
not compiled claims; nothing removed. See `RECEIPT.md`.
