---
name: polyglot-cwe-entropy-auditor
description: Zero-dependency Python CWE-78/89/798 static vulnerability and Shannon secret-entropy scanner.
---

# polyglot-cwe-entropy-auditor (interface)

## Purpose
Zero-dependency static scanner: CWE-78/89/798 patterns across Rust, Python,
TypeScript, C++ plus Shannon-entropy secret detection with deterministic risk
tiers. PredictiveSimulation well, MlirOpId 202564.

## When to use
Verifying/invoking the auditor: `python::synthesize::polyglot_cwe_and_entropy_auditor`
trigger or macro-opcode `MlirOpId(202564)`. Standalone verification:
`python3 -m py_compile` + `python3 polyglot_cwe_and_entropy_auditor.py` self-test
(VERIFIED green 2026-10-31, [PASS] self-test — see RECEIPT).

## Out of scope
CWE classes beyond 78/89/798. Probabilistic risk tiers (deterministic thresholds
only). Findings without a compiled-clean scanner run.

## Type signature
Compiled gate: `tool.rs` (`polyglot-audit-gate`, schema: `tool.json`).
- `cwe_valid(cwe: String) -> bool` — "CWE-78" | "CWE-89" | "CWE-798"
- `polyglot_gate(cfg: PolyglotFinding) -> bool` — cwe AND entropy-high AND compiled
- `make_polyglot(cwe, entropy_high, compiled) -> PolyglotFinding`
Native artifact stays: `polyglot_cwe_and_entropy_auditor.py` + `skill.atoma_skill`.

## Source
Already in N3 (`N3/design/autonomous_skills/polyglot-cwe-entropy-auditor/`);
interface shrunk 2026-10-31 (Agent A2); `tool.ts`/`tool.rs`/`tool.json` added.
No tool/server claims beyond the `python3` verifier (re-ran VERIFIED).
See `RECEIPT.md`.
