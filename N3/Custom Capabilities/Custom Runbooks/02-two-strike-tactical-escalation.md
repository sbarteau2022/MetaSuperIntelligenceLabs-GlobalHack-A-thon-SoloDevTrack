---
name: two-strike-tactical-escalation
description: Enforce materially different retries on Strike 1, mandatory bus escalation on Strike 2, and T0-T4 tactical mode progression.
depends_on: [runbook-plan-and-judges]
load_mode: eager
---

# Two-Strike Failure Doctrine & Tactical Mode (`T0–T4`)

Never brute-force a failing systems engineering step with identical retries.

## Strike Progression

1. **Strike 1 — Pause, Inspect Receipts, Materially Different Re-Approach**:
   - Read the failed judge output and stderr receipt.
   - Change the `approach` or switch to `fallback_action` so `retry_signature` (`hash(task + lane + approach)`) changes. Identical retries are rejected by the backend (`HTTP 400 protocol violation: identical_retry`).
2. **Strike 2 — Mandatory Pause & Escalation (`escalation_state: pending`)**:
   - After 2 failed attempts (`max_attempts = 2`), the packet locks in `escalated` state (`HTTP 409 Conflict` if re-run without guidance).
   - Downstream dependent packets automatically transition to `skipped` (`reason: dependency_failed`) via `reconcile_dependencies`.
3. **Advised Retry — Micro Pre-Registration (`T3 Campaign`)**:
   - Elevate tactical level (`T0 March` → `T1 Scout` → `T2 Maneuver` → `T3 Campaign` → `T4 Grand Strategy`).
   - Record orchestrator `guidance` and a non-null `preregistered_plan` containing `steps[].predicted_tool_calls` and explicit verification criteria before executing the 3rd attempt.
