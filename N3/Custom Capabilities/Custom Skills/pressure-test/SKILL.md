---
name: pressure-test-runbook
description: Assault boundaries and edges with hostile inputs and per-item disposition.
user-invocable: true
depends_on: [runbook-plan-and-judges, two-strike-tactical-escalation, hole-isolation-and-verification]
load_mode: eager
---

# Pressure-Test Runbook — Boundary/Edge Assault

Date: 2026-10-30. Status: PROPOSED.

Objective: prove the slice survives hostile inputs — every edge dispositioned fix-or-waive, none silently dropped.

## Procedure

1. **Enumerate edges.** List the applicable edges using EDGE-CASES.md vocabulary (Input & Validation, Auth & Sessions, Data & Consistency, API & Network, Frontend & UI) plus domain-specific boundaries (timeouts, byte caps, concurrency limits).
   - Judge: `contains` — edge list cites an EDGE-CASES.md section per item; `nonempty` — at least the sections touching the slice's blast radius.
2. **Craft hostile inputs.** For each edge, write a concrete hostile input or sequence (empty/`null`/oversize/unicode, expired token, concurrent write, double-submit, upstream timeout, disconnect mid-upload).
   - Judge: `file_exists` — hostile-input set; each edge maps to ≥1 input.
3. **Execute in a hole.** Run every hostile input inside the hole cavity under seatbelt confinement; record per-item pass/fail with the judge receipt.
   - Judge: `command` — execution log exit-recorded per item; `contains` — every edge has a disposition.
4. **Fix-or-waive each.** Every failing item gets a fix OR a dated waiver naming the exact risk accepted; no bulk waivers, no silent drops.
   - Judge: `contains` — per-item disposition (fixed with packet ref, or waived with date + risk); zero undispositioned items.

## Exit criteria

100% of enumerated edges dispositioned; fixes re-run through step 3 with passing receipts; waivers dated and recorded in Failures.md.

## Failure path

Hostile input exposes a defect → freeze it as a packet (per `04-hole-isolation-and-verification.md` hole lifecycle for the repro), fix with a materially different approach on Strike 1; repeated failure on the same edge → Strike 2 per `02-two-strike-tactical-escalation.md` with T-level elevation and pre-registered plan before the third attempt.
