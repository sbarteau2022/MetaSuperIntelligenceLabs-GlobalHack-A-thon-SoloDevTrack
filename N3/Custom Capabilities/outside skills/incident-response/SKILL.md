---
name: incident-response
description: Stop the damage, find the cause, learn once. Impact sentence, reversible mitigation with timestamps, facts over theories, rhythmic comms, measured recovery, blameless follow-up with owned dated action items.
---

# incident-response (interface)

## Purpose
Damage-first incident handling with a learning close: state impact (what/for
whom/since when), stop the bleeding reversibly before understanding (rollback,
disable, shed, fallback — every action timestamped), gather facts, communicate
on rhythm even with no news, confirm recovery by measurement, then blameless
timeline + impact + root/contributing causes + owned dated action items that each
carry the test/alert that would have caught it.

## When to use
During any incident (the five during-steps) and within days after (the follow-up).

## Out of scope
Theory before mitigation. Irreversible "fixes" under pressure. Untimestamped
actions. Recovery by feeling. Action items without owner + date + catcher.

## Type signature
Compiled gate: `tool.rs` (`incident-gate`, schema: `tool.json`).
- `incident_gate(cfg: IncidentReport) -> bool` — impact AND bleeding-stopped AND recovery-measured AND actions-owned
- `make_incident(impact_set, bleeding_stopped, recovery_measured, actions_owned) -> IncidentReport`

## Source
Trash source: `skills-desktop-plugins-20261030/engineering/skills/incident-response.md`
(`17e601ca10eac86e96a67f941c8b18eafded24`). Shrunk to this interface 2026-10-31
(Agent A2). No tool/server claims in source; nothing removed. See `RECEIPT.md`.
