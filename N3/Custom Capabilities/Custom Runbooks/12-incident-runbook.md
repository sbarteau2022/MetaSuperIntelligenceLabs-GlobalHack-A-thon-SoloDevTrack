---
name: incident-runbook
description: Respond to failure — freeze scope, reproduce, isolate, fix, regression-guard, post-mortem.
depends_on: [runbook-plan-and-judges, two-strike-tactical-escalation, hole-isolation-and-verification]
load_mode: eager
---

# Incident Runbook — Failure Response

Date: 2026-10-30. Status: PROPOSED.

Objective: contain a failure, fix its root cause, and leave a regression guard plus a dated post-mortem — no fix without both.


## Condensed doctrine applied

**Plan-and-judges (condensed):** Work is DAG packets, never an unbounded loop. PLAN lane decomposes into atomic packets with contracts, depends_on edges, declared context/tools, and deterministic judges. CODE lane claims one ready packet at a time. No self-graded completion — a packet passes only when all declared judges pass.

**Two-strike (condensed):** Never retry identically. Strike 1: pause, read the receipt, change approach so retry_signature changes, or the retry is rejected. Strike 2: mandatory pause and escalation, packet locks, dependents skip. A 3rd attempt needs orchestrator guidance + a preregistered plan.

**Hole isolation (condensed):** Never mutate the primary root directly on untrusted work. Execute in a seatbelt hole cavity, record source fingerprint + Merkle root, merge only through the Merkle gate with approval.

## Procedure

1. **Freeze scope.** Stop new work in the blast radius; record what is frozen and why.
   - Judge: `contains` — freeze notice names the scope boundary and the incident id.
2. **Reproduce.** Write the minimal reproduction (inputs, commands, environment) that triggers the failure deterministically.
   - Judge: `command` — repro runs and fails the same way twice; repro script stored.
3. **Isolate.** Move the repro into a hole cavity (`POST /holes`, `hole_id`); use the `04-hole-isolation-and-verification.md` vocabulary — seatbelt execution, unified diff inspection, `source_merkle_root` drift check.
   - Judge: `file_exists` — hole diff of the isolated failure; merkle root verified undrifted.
4. **Fix.** Address the root cause (never a surface patch); re-run the repro to green inside the hole before requesting merge.
   - Judge: `command` — repro exit `0` post-fix; `contains` — fix note names the root cause, not the symptom.
5. **Regression-guard.** Add a deterministic judge (from the `01` vocabulary) that would have caught this failure; run the full gate.
   - Judge: `file_exists` — new regression judge; `command` — gate exit `0`.
6. **Post-mortem.** Append the dated entry to Failures.md (finding, evidence, consequence); corrections to prior claims appended, never rewritten.
   - Judge: `contains` — Failures.md entry dated 2026-10-30+ with finding/evidence/consequence.

## Exit criteria

Repro green, regression guard in the gate, merge-gate passed (`tested_pass`, `approved: true`), post-mortem entry recorded, scope unfrozen explicitly.

## Failure path

Fix attempt fails → Strike 1: pause, read receipts, materially different approach (`retry_signature` must change). Second failure → Strike 2 per `02-two-strike-tactical-escalation.md`: `escalation_state: pending`, T-level elevation, micro pre-registered plan with `steps[].predicted_tool_calls` before any third attempt.
