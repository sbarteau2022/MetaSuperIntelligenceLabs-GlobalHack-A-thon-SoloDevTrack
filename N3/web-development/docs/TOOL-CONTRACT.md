# TOOL-CONTRACT.md — Frozen Interface for Tool Work

Date: 2026-10-30. Status: PROPOSED.

## Purpose

This file is the ACCEPTANCE TEST for PARTITION F (tools 2026). F writes the catalog; a PARTITION F deliverable passes if and only if every tool entry carries the fields below and every status matches machine truth. Anything F proposes beyond this schema is advisory, not law.

## 1. Mandatory tool-entry schema (MUST)

Every tool aggregated or proposed needs all seven fields. An entry missing any field FAILS the contract gate.

| Field | Meaning |
| ----- | ------- |
| `name` | Tool name as invoked on the command line. |
| `target` | What it checks/enforces (language, artifact, or concern). |
| `install-or-build status` | How it arrives: preinstalled, `brew install …`, `pip install …`, repo build, or N/A. |
| `checker command` | The single best 2026 checker command (exact invocation, with the flags that make it a gate). |
| `gate it feeds` | Which enforcement gate consumes its output: GATE (pre-completion block), PROVE (typecheck/test/build evidence), or RECORD (register/ledger entry) — plus WHICH file records it. |
| `machine status` | INSTALLED / MISSING / NEEDS-VERIFY on this machine, per §2. |
| `evidence` | The command output or path that settles the machine status. Paste the output, don't summarise. |

## 2. Machine-truth table (VERIFIED 2026-10-30, commands run on this machine)

F's catalog MUST agree with this table. Disagreement = F re-runs the evidence command, not the other way round.

| Tool | Version observed | Status |
| ---- | ---------------- | ------ |
| rustc | 1.99.0 (Homebrew) | INSTALLED |
| python3 | 3.14.7 | INSTALLED |
| node | v26.9.0 | INSTALLED |
| go | go1.27.3 darwin/arm64 | INSTALLED |
| clang | Apple clang 21.0.0 | INSTALLED |
| sqlite3 | 3.54.0 | INSTALLED |
| psql (client) | 17.11 (Homebrew) | INSTALLED |
| swiftc | Apple Swift 6.4.2 (driver 1.168.6) | INSTALLED |
| ruby | 4.0.7 +PRISM [arm64-darwin27] | INSTALLED |
| ruff | /Users/stewartbarteau/.local/bin/ruff | INSTALLED |
| tsc | not on PATH | MISSING |
| java | shim only — no runtime (`Unable to locate a Java Runtime`) | MISSING |
| php | not on PATH | MISSING |
| eslint | not on PATH | MISSING |
| prettier | not on PATH | MISSING |
| biome | not on PATH | MISSING |
| mypy | not on PATH | MISSING |
| shellcheck | not on PATH | MISSING |

Statuses are point-in-time facts about this machine, not recommendations. NEEDS-VERIFY is for tools F could not run here (remote-only, uninstalled-but-proposed); it requires naming the machine or environment where verification will happen.

## 3. Gate mapping (MUST)

Each entry's `gate it feeds` is exactly one of:

- `GATE → <file>` — blocks completion; output lands in the named gate file (e.g. `hooks/discipline.py`, `hooks/audit.py`).
- `PROVE → <file>` — typecheck/test/build evidence; pasted output lands in the named release or checklist file.
- `RECORD → <file>` — observation only; entry lands in the named register (`CLAIMS_REGISTER.md`, `Successes.md`, `Failures.md`).

PROPOSED-but-uninstalled tools MUST feed RECORD (never GATE): a gate nobody can run is a veto nobody can clear.

## 4. Acceptance checklist (F passes iff all hold)

- [ ] Every tool entry has all seven schema fields (§1).
- [ ] Every entry's machine status matches the §2 table or carries fresh evidence superseding it (dated correction entry, never a silent edit).
- [ ] Every entry maps to exactly one of GATE/PROVE/RECORD plus the recording file (§3).
- [ ] No MISSING tool feeds GATE.
- [ ] Evidence is pasted output, not a summary (AGENTS.md §7.2).

## NOT checked

Install decisions (brew/pip approve-list); CI wiring for any checker; performance benchmarks of the tools themselves.
