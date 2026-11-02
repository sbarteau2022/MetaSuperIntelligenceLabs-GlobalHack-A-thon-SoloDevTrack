# TODO — Task Ledger (Commandment I)

Columns: ID | Task | Status(OPEN/DONE/DEFERRED) | Evidence(pointer) | Owner | Notes
A task is DONE only with an evidence pointer. Deferred requires reason+owner.

| ID | Task | Status | Evidence | Owner | Notes |
|----|------|--------|----------|-------|-------|
| T-001 | Write backend roadmap to repo | DONE | ROADMAP.md | agent | 2026-11-02 |
| T-002 | Scaffold Ten Commandments enforcement | DONE | COMMANDMENTS.md, scripts/verify/gate.sh | agent | 2026-11-02 |
| T-003 | Phase 1: Axum backend :4810 + /api/status | OPEN | | agent | blocked: rust/cargo check |
| T-004 | Phase 1: Zustand client /api/status in UI | OPEN | | agent | |
| T-005 | Define SurrealDB schema (RG-AH vocab) | OPEN | | agent | Phase 2 |
| T-006 | SSE streaming to oMLX end-to-end | OPEN | | agent | Phase 2 |
| T-007 | Qdrant rag/query wiring | OPEN | | agent | Phase 3 |
| T-008 | React Flow run DAG real nodes | OPEN | | agent | Phase 3 |
| T-009 | Cloud seats one at a time | OPEN | | agent | Phase 4 |
| T-010 | HyperDX telemetry last | OPEN | | agent | Phase 5 |
