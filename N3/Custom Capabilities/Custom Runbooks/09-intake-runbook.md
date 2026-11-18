---
name: intake-runbook
description: Intake a new domain under the adapter contract so Adapter #2+ earns the mandate.
depends_on: [runbook-plan-and-judges, two-strike-tactical-escalation]
load_mode: lazy
---

# Intake Runbook — New-Domain Intake Under the Adapter Contract

Date: 2026-10-30. Status: PROPOSED.

Objective: take a new domain from survey to registered adapter with gates passed — the path by which Adapter #2+ earns the mandate.


## Condensed doctrine applied

**Plan-and-judges (condensed):** Work is DAG packets, never an unbounded loop. PLAN lane decomposes into atomic packets with contracts, depends_on edges, declared context/tools, and deterministic judges. CODE lane claims one ready packet at a time. No self-graded completion — a packet passes only when all declared judges pass.

**Two-strike (condensed):** Never retry identically. Strike 1: pause, read the receipt, change approach so retry_signature changes, or the retry is rejected. Strike 2: mandatory pause and escalation, packet locks, dependents skip. A 3rd attempt needs orchestrator guidance + a preregistered plan.

## Procedure

1. **Survey.** Map the domain: canonical sources, existing patterns, failure modes, and what the core law already covers vs what is genuinely domain-specific.
   - Judge: `file_exists` — survey note; `contains` — explicit core-vs-domain split.
2. **Draft rules table.** Write the candidate binding rules, each numbered, each citing the failure it prevents.
   - Judge: `nonempty` — rules table with ids; `contains` — every rule names its failure mode.
3. **Draft skeleton set.** Produce the domain skeleton (rules, edge cases, checkers, gates) in the adapter's canonical layout.
   - Judge: `file_exists` — every skeleton file listed in the intake plan exists on disk.
4. **Gates.** Run the domain gates (discipline/audit equivalents plus the domain checker) and paste outputs.
   - Judge: `command` — each gate exit code `0` with pasted verbatim output.
5. **Registry entry.** Record the adapter in the registry with scope, mandate status, and effective date ONLY after all gates pass.
   - Judge: `contains` — registry entry names scope + mandate status + date; no entry predates a passing gate run.

## Exit criteria

Registry entry exists, all gates green with pasted outputs, and the adapter's scope boundary is written (what it governs and what it must not govern).

## Failure path

Gate failure → Strike 1: materially different fix (rule change or skeleton change, never blind re-run). Second failure → Strike 2 per `02-two-strike-tactical-escalation.md`: escalate, mark intake blocked, no registry entry until judges pass — an ungated adapter never earns the mandate.
