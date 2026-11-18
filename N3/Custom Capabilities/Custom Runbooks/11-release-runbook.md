---
name: release-runbook
description: Execute the mechanical done-gate — checkers, pasted outputs, stop-gate, sign-off.
depends_on: [runbook-plan-and-judges]
load_mode: eager
---

# Release Runbook — The Done-Gate

Date: 2026-10-30. Status: PROPOSED.

Objective: give "done" a mechanical definition — checkers run, outputs pasted, stop-gate cleared, sign-off recorded.


## Condensed doctrine applied

**Plan-and-judges (condensed):** Work is DAG packets, never an unbounded loop. PLAN lane decomposes into atomic packets with contracts, depends_on edges, declared context/tools, and deterministic judges. CODE lane claims one ready packet at a time. No self-graded completion — a packet passes only when all declared judges pass.

## Procedure

1. **Run discipline + audit.** Execute `discipline.py` and `audit.py` (or the domain equivalents) and capture full output.
   - Judge: `command` — `discipline.py` exit `0` AND `audit.py` exit `0`; full outputs captured.
2. **Run language checkers.** Execute the applicable language typecheck/test/build for the slice and capture full output.
   - Judge: `command` — checker exit `0`; output captured verbatim.
3. **Paste into the release checklist.** Paste all outputs (not summaries) into the release checklist entry for this slice.
   - Judge: `contains` — checklist entry holds verbatim outputs; `nonempty` — zero "results summarized" placeholders.
4. **Stop-gate status.** Confirm no pending issues, or list each with a dated resolution/waiver; the stop-gate veto is final.
   - Judge: `contains` — stop-gate line reads PASS, or every pending item has a dated disposition.
5. **Sign-off line.** Record the dated sign-off (who, what slice, which checklist entry).
   - Judge: `contains` — sign-off line with date + identity + slice reference.

## Exit criteria

Release checklist entry complete with pasted checker outputs, stop-gate PASS, and sign-off line. A summary of results is not a result — missing paste = gate failed.

## Failure path

Any checker nonzero or stop-gate item unresolved → release blocked; Strike 1 fix-and-rerun with a materially different approach. Repeated gate failure → Strike 2 per `02-two-strike-tactical-escalation.md`: escalate, dependents `skipped`, no sign-off until judges pass.
