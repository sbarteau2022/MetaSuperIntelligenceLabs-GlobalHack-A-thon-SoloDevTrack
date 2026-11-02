---
name: codebase-pressure-test
description: "Pressure-test a codebase implementation for correctness, robustness, and honesty of claims before declaring it done."
---

# CodeBasePressureTest

Run a full pressure audit against the named implementation. State every verdict as VERIFIED (measured here), INFERRED (reasoned from evidence — say where the evidence ends), or PROPOSED (a plan, not a fact).

1. **Build and tests.** Run the project's build and its full test suite. Record exact commands and outputs. Any failure is a FAIL, not a caveat.
2. **Adversarial pressure.** Apply at minimum:
   - malformed / oversized / replayed inputs,
   - concurrent writers under snapshot isolation,
   - interrupted writes / hash-mismatch recovery,
   - path traversal and schema violations,
   - role/permission boundary violations,
   - resource exhaustion (k caps, rate limits, budgets).
3. **Claim honesty.** For every user-facing claim (correctness, complexity, safety, security), find the measurement that earns it. Hardcoded flags, heuristic-only selections, heuristic evictions, or unmeasured throughput numbers become FAIL findings until measured or relabeled.
4. **Receipt discipline.** Every mutating operation must return a receipt (hash + timestamp). Missing receipts are FAILs.
5. **Storage integrity.** Verify atomic writes (tmp + rename), WAL append shape, content-addressed snapshots (hash verify), and idempotent recovery.
6. **Output.** Produce a dated table: finding ID, surface, pressure, expected vs observed, verdict (VERIFIED PASS / VERIFIED FAIL / INFERRED / PROPOSED), fix or gap. Append corrections as new dated entries; never silently rewrite history.

Fail-closed rule: if the pressure path is missing, the gate is OPEN — not passed.
