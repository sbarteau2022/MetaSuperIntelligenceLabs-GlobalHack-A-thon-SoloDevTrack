---
name: design-spec
description: Write the contract before the code. Purpose + non-goals, typed data, named state, numbered rules, edge table, failure behavior, invariants, >=3 hand-computed examples. Spec is what code AND tests are both checked against.
---

# design-spec (interface)

## Purpose
The contract all code and tests are checked against. Unambiguous before
implementation: every "few/recent/many" replaced by a number and one config knob
with one meaning; every rule maps to an observable output; every knob is used.

## When to use
Before any implementation. Then `design/preregister` (sealed claims) and
`design/test-suites` (the suite) before code. Checks: oracle, gaming, baseline,
two-reader, orphan.

## Out of scope
Vague rules (a rule no test could decide is unspecified, not done). Scores that
reward the wrong thing (close the loophole in the spec). Baselines that score
zero by construction. Burying a guess as a rule (ask or list under Open questions).

## Type signature
Compiled gate: `tool.rs` (`spec-gate`, schema: `tool.json`).
- `examples_enough(examples_count: number) -> bool` — >= 3
- `spec_gate(cfg: DesignSpec) -> bool` — purpose AND non-goals AND numbered rules AND edge table AND examples>=3
- `claim_sealed(sealed: boolean) -> bool`
- `make_spec(purpose_set, non_goals_set, rules_numbered, edge_table_done, examples_count) -> DesignSpec`

## Source
Trash source: `skills-desktop-plugins-20261030/design/skills/design-spec.md`
(`337551a9c8d3c8d9a3cb098adb1e3f6397cccd8b01b72a537a1`). Shrunk to this interface
2026-10-31 (Agent A2). No tool/server claims in source; nothing removed.
See `RECEIPT.md`.
