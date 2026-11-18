# N3 Runbooks — Index

Date: 2026-10-30. Status: PROPOSED.

All 13 entries in `N3/runbooks/`. New runbooks (07–12) reuse the judge vocabulary of `01`, the escalation vocabulary of `02`, and the isolation vocabulary of `04` — never redefine them.

| # | File | Purpose (one line) | When to use |
|---|------|--------------------|-------------|
| — | `README.md` | Canonical home note for skill definitions and the skill-tree model | Orienting: where skills live and how they load |
| 01 | `01-runbook-plan-and-judges.md` | DAG packets with pre-registered deterministic judges; no self-graded completion | Starting any multi-step run — plan lane before code lane |
| 02 | `02-two-strike-tactical-escalation.md` | Strike 1 materially-different retry, Strike 2 mandatory escalation, T0–T4 progression | Any step judge fails — the only legal failure path |
| 03 | `03-kv-cache-and-continuity.md` | Radix prefix stability and 60% continuity checkpointing | Long-horizon runs where context and cache pressure grow |
| 04 | `04-hole-isolation-and-verification.md` | Seatbelt-isolated hole cavities with Merkle merge gates | Any untrusted edit or test execution before merge |
| 05 | `05-rust-tokio-systems-engineering.md` | Bounded-concurrency Rust/Tokio with append-only SQLite WAL ledger | Backend orchestration, runbook state, serving primitives |
| 06 | `06-agent-experience-and-tool-design.md` | Progressive disclosure, bounded tool outputs, PPC evolution | Designing skills, bundles, or harness overlays |
| 07 | `07-delivery-runbook.md` | Scoped change end-to-end: calibrate → retrieve → slice → implement → checkers → review → docs | The general delivery loop for any single change |
| 08 | `08-research-runbook.md` | Evidence handling with per-source provenance and labeled synthesis | Answering a question where claims must be traceable |
| 09 | `09-intake-runbook.md` | New-domain intake under the adapter contract through gates to registry | How Adapter #2+ earns the mandate |
| 10 | `10-pressure-test-runbook.md` | Boundary/edge assault with hostile inputs and per-item fix-or-waive | Before release: prove the slice survives EDGE-CASES.md |
| 11 | `11-release-runbook.md` | Mechanical done-gate: checkers run, outputs pasted, stop-gate, sign-off | Claiming anything is done |
| 12 | `12-incident-runbook.md` | Failure response: freeze → reproduce → isolate → fix → guard → post-mortem | Something broke — contain it and learn from it |
