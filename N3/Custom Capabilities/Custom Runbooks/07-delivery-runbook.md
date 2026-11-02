---
name: delivery-runbook
description: Execute one scoped change end-to-end from calibration through adversarial review to docs.
depends_on: [runbook-plan-and-judges, two-strike-tactical-escalation, hole-isolation-and-verification]
load_mode: eager
---

# Delivery Runbook — Scoped Change End-to-End

Date: 2026-10-30. Status: PROPOSED.

Objective: take one scoped change from brief to merged, judged at every step, with no self-graded completion.


## Condensed doctrine applied

**Plan-and-judges (condensed):** Work is DAG packets, never an unbounded loop. PLAN lane decomposes into atomic packets with contracts, depends_on edges, declared context/tools, and deterministic judges. CODE lane claims one ready packet at a time. No self-graded completion — a packet passes only when all declared judges pass.

**Two-strike (condensed):** Never retry identically. Strike 1: pause, read the receipt, change approach so retry_signature changes, or the retry is rejected. Strike 2: mandatory pause and escalation, packet locks, dependents skip. A 3rd attempt needs orchestrator guidance + a preregistered plan.

**Hole isolation (condensed):** Never mutate the primary root directly on untrusted work. Execute in a seatbelt hole cavity, record source fingerprint + Merkle root, merge only through the Merkle gate with approval.

## Procedure

1. **Calibrate.** Write the one-line calibration (BRIEF + RISK + BLAST-RADIUS + CONTRACT-FIRST) before any other work.
   - Judge: `file_exists` — calibration block present on disk; `nonempty` — all four fields filled.
2. **Retrieve.** Load only the contract sources named in CONTRACT-FIRST; record what was read.
   - Judge: `contains` — retrieval log names each source file and its status (read/deferred).
3. **Smallest slice.** Cut the smallest independently shippable slice; declare its packet contract (deliverable, `depends_on`, judges).
   - Judge: `file_exists` — packet contract; `contains` — at least one deterministic judge from the `01` vocabulary.
4. **Implement inside a hole.** All edits run in a hole cavity (`POST /holes`, `hole_id`); never mutate the primary root directly.
   - Judge: `command` — hole diff command exit `0` with nonempty delta; `source_merkle_root` unchanged at merge-gate check.
5. **Checkers.** Run the applicable checkers and paste output (never a summary).
   - Judge: `command` — each checker exit code `0`; pasted output present verbatim.
6. **Adversarial review.** A reviewer who did not write the code dispositions every checklist item pass/fail; coders never self-approve.
   - Judge: `file_exists` — review record with per-item disposition; zero un-dispositioned items.
7. **Docs.** Update the affected docs (Design/CLAIMS_REGISTER/Successes/Failures entries, dated) in the same slice.
   - Judge: `contains` — doc delta references the packet id and date.

## Exit criteria

Merge-gate passes (`tested_pass`, `approved: true`, no merkle drift) AND every step judge above is recorded passed. "Done" requires the checkers' output, not a summary of it.

## Failure path

Any step judge fails → Strike 1: pause, read the judge receipt, re-approach with a materially different `retry_signature` (identical retries are protocol violations). Second failure on the same step → Strike 2 per `02-two-strike-tactical-escalation.md`: `escalation_state: pending`, elevate T-level, micro pre-register before any third attempt.
