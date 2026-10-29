---
name: ares-self-critique
description: "Self-critique and reflection loops for Ares: journaling, verdict checks, adversarial reflection, repair suggestions. Use when auditing agent output quality or improving runbook reasoning."
---

# Ares Self-Critique & Reflection Loops

## Scope
- Verdict module: `backend/src/verdict.rs`.
- Runbook packets: judge sets, attempt tracking, escalation.
- Checkpointing: `Handoff.md` at 60% context.
- Correction protocol: dated append-only entries.

## Rules
- A loop's done signal is a claim, not a verdict; judges are mechanical.
- Corrections are entries, not edits.
- Every proposal includes expected_metric and metric_window.
