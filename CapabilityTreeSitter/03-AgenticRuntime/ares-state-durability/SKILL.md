---
name: ares-state-durability
description: "State durability and idempotency for Ares: JSON stores, checkpoint/resume, packet attempt tracking, hole merge, Merkle receipts. Use when auditing persistence or crash recovery."
---

# Ares State Durability & Idempotency

## Scope
- Stores: workflows.json, holes.json, plugins.json, secrets.env.
- Checkpoint: `Handoff.md` at 60% context; pin system + original request.
- Idempotency: packet/hole idempotency keys; optimistic concurrency on merge.
- Crash recovery: auto-runner restarts on boot; workflow/hole state persists.

## Rules
- Atomic writes (temp + fsync + rename).
- Append-only corrections; no silent history rewrites.
- No bulk deletes.
