---
name: adversarial-pressure
description: Try to break it, then grade it honestly. Fresh-thread black-box attack from spec + interface only, three attacks per rule, mutation survivors, differential oracle, hygiene sweep. Findings count only if they reproduce. Graded A–F with caps.
---

# adversarial-pressure (interface)

## Purpose
The author is the wrong grader (Elle: 13/13 self-checks, seven real bugs). Separate
pass, ideally fresh thread, spec + interface only, no implementation first: plan 3
attacks per rule (boundary / interleaving / violated assumption), black-box
runnables with spec-derived expectations, seeded fuzz 1000+, spec-vs-code orphan
audit both directions, loophole hunt on anything scoring, mutation survivors,
naive-oracle differential, resource/hygiene sweep.

## When to use
After a suite is green. Fixes re-attacked including the fix itself; stop only at
two consecutive clean rounds at wrong-output/spec-violation level or budget end.

## Out of scope
Unreproduced suspicions (list under "unconfirmed", never as findings). Praise
without specifics. Grades above what was attacked. Grading own work.

## Type signature
Compiled gate: `tool.rs` (`pressure-gate`, schema: `tool.json`).
- `severity_valid(severity: String) -> bool` — wrong-output|spec-violation|test-gap|hygiene
- `finding_counts(finding: AttackFinding) -> bool` — reproduced AND has-input AND severity-valid
- `pressure_gate(findings_rounds: number, kill_rate: number) -> bool` — rounds>=2 AND kill>=70
- `make_finding(severity, reproduced, has_input) -> AttackFinding`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/adversarial-pressure.md`
(`ae5a320c462f8b79431a140b0c006e75819df3a91`). Shrunk to this interface 2026-10-31
(Agent A2). `run_python`/`vm_exec` are host belt tools; nothing removed.
See `RECEIPT.md`.
