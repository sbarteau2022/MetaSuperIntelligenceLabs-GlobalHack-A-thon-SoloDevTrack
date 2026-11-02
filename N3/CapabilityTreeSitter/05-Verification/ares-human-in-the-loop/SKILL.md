---
name: ares-human-in-the-loop
description: "Human-in-the-loop controls for Ares: approval gates, merge approval, hole approve, circuit breaker, escalation queues. Use when configuring or auditing HITL flows."
---

# Ares Human-in-the-Loop

## Scope
- Holes: `request_merge`, `approve`, `merge` (approval gate).
- Runbook: escalation after 2 failed attempts; guidance + preregistration for 3rd.
- Circuit breaker: 3 transport failures open circuit (429).
- Queues: review queue, vulnerability, compliance, etc.

## Rules
- High-risk tools require approval.
- Merge without approval records a receipt.
- Escalation is not auto-resolved; human guidance required.
