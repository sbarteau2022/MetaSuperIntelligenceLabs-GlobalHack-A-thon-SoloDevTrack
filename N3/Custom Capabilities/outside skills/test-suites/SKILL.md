---
name: test-suites
description: Build a suite that can actually fail. Worked examples, boundaries t-1/t/t+1, sequences, properties, metamorphic, oracle, errors, regression. Mutation-checked to 90%+ killed. Green means something only if red is reachable.
---

# test-suites (interface)

## Purpose
Suites built from spec + claims, never from the code under test. Expected values
from spec/hand arithmetic, exact-output asserts, every knob varied, every branch
reached, determinism via separate-process `json.dumps(sort_keys=True)` comparison.
Run in the microVM with `PYTHONDONTWRITEBYTECODE=1`.

## When to use
After `design/design-spec` + `design/preregister`, before implementation code.
Layers in order: worked → boundaries → sequences → properties (seeded 500+) →
metamorphic → oracle → errors → regression. Then the mutation gate
(`mutate.py`, target 90%+ killed; every survivor gets a killing test or a written
equivalence reason).

## Out of scope
Vacuous tests (no-crash-only, last-element-only, zero-by-design baselines).
Circular expectations (computed by the code under test). `try/except: pass`,
skipped tests, loosened tolerances. Work at import time. Shared mutable defaults.
Float equality without a stated tolerance.

## Type signature
Compiled gate: `tool.rs` (`suite-gate`, schema: `tool.json`).
- `kill_ok(kill_rate: number) -> bool` — >= 90
- `grade_cap(has_wrong_output: boolean, kill_rate: number, spec_violated: boolean) -> string` — "C-cap" | "B-cap" | "pass" | "C-plus-cap"
- `suite_gate(cfg: SuitePlan) -> bool` — worked AND boundary AND oracle AND kill>=90
- `make_suite(worked_done, boundary_done, oracle_done, kill_rate) -> SuitePlan`

## Source
Trash source: `skills-desktop-plugins-20261030/design/skills/test-suites.md`
(`e19c07380328cfb7a50c9d1d398114cc1dc3fb884`). Shrunk to this interface 2026-10-31
(Agent A2). `vm_exec` is a host belt tool, not a compiled claim; nothing removed.
See `RECEIPT.md`.
