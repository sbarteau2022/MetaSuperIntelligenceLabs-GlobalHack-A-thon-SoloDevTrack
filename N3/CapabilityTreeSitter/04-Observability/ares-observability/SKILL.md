---
name: ares-observability
description: "Telemetry, metrics, and tracing for Ares: event spine, taps, counters, /metrics endpoint, receipt fingerprints, circuit breaker logging. Use when adding instrumentation, debugging metrics, or auditing observability gaps."
---

# Ares Observability

## Scope
- Telemetry spine: `backend/src/telemetry.rs` (events, counters, taps).
- Metrics endpoint: `GET /metrics` (Prometheus-ish text).
- Receipts: SHA-256 non-crypto fingerprints on hole/packet transitions.
- Circuit breaker logging: `circuit_open`, `lane_refusal`, `escalation`.

## When to use
- Adding new telemetry events or counters.
- Debugging missing metrics or false positives.
- Auditing what data is recorded (counts only, no secrets/text).

## Rules
- Telemetry carries counts, latencies, ids, and scopes — never prompt/output text or secrets.
- Every state transition records a receipt.
- Taps gate categories; a closed gate drops and counts the drop.
EOF
