---
name: ares-agent-cicd
description: "Agent CI/CD and regression for Ares: build/test/smoke gates, bench regression, compliance smoke, no fake metrics. Use when setting up or auditing agent CI."
---

# Ares Agent CI/CD & Regression

## Scope
- Build: `cargo build` (dev + release).
- Tests: compliance_smoke.sh, eval_atoma.sh, formal_bench_receipt.sh.
- Gates: latency/recall must not regress vs baseline receipt.
- Metrics: counts only; no args/results in telemetry.

## Rules
- No claim without verification.
- Receipts are append-only and versioned.
- Every change passes build + smoke + compliance before merge.
