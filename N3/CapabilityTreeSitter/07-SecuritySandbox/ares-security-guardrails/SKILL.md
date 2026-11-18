---
name: ares-security-guardrails
description: "Security guardrails for Ares: seatbelt, sandboxing, Merkle receipts, circuit breakers, base-weight fingerprint, overlap policy. Use when hardening execution or auditing security."
---

# Ares Security & Guardrails

## Scope
- Seatbelt/fallback: `backend/src/terminal.rs`, `backend/src/holes.rs`.
- Merkle receipts: `backend/src/holes.rs` (SHA-256).
- Circuit breaker: `backend/src/server.rs` (per-lane, 3 failures, 429).
- Overlap policy: doc 07 / immune system (optional integration).

## Rules
- Network denied under seatbelt.
- Writes only to canonical cwd + tmp.
- No secrets in env/telemetry/logs.
- Approval required for write/high-risk tools.
- Base-weight fingerprint enforced before run.
