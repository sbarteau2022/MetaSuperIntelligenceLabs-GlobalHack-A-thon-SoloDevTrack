---
title: "N³ observability implementation contract"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-29T20:54:29.675284+00:00"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files: []
related_issues: []
related_documents: ["README.md"]
tags: [n3, observability, sqlite]
---

# N³ system observability contract

Status: implementation contract, 2026-10-29. Runtime adapters remain PROPOSED.
Sources: C04 topology, C17 persistence, C21 resource budgets, C27 DAG,
C28 skills, C29 tools, C30 lanes, C31 observability, C33 knowledge,
MasterN^3DB-RAG, N^3-Wiring-and-Fallbacks, TAP-N-TRACE.

## Ownership and privacy

The outer shell owns writes. Models receive read-only, bounded projections.
No inference, network access, tool execution or raw arguments/results/text in
this database. IDs and SHA256 references point to separately governed content.
The SQLite file is not an authorization boundary against its filesystem owner.
SQL constraints protect ordinary writers, not an administrator rewriting files.

## Contract

All facts are immutable. Corrections append a superseding event. Each event
belongs to a project/task/run trace and optional span/actor, has a producer ID
and sequence for replay, occurrence UTC microseconds, boot-relative monotonic
nanoseconds, and a separate global ingestion order. Ingestion clock is clamped
monotonically across restarts; occurrence time may arrive late. Each committed
event binds its typed facts and previous event hash. Hashes are not signatures.
Merkle roots every 1000 events bind event hashes; external anchoring is future work.

A logical invocation has numbered attempts. Dispatch, execution and acceptance
are distinct. Unknown is never pass. Fallbacks link attempts and require a changed
strategy hash. Accepted outcomes require a passed evidence-backed check.
The graph stores versioned immutable snapshots; DAG edges must remain acyclic.
Knowledge graphs may contain cycles. Neither graph executes anything.

Brigade leases require a meta decision with an approved nonnegative cost cap,
expiry and max agents. The database enforces recorded admissions and cumulative
reservation caps; runtime must actually enforce stop/release, hard resource caps,
and expiry. Idle models are not justified by an unreleased row alone.

Measurements distinguish counters/deltas/gauges and measured/estimated values,
missing values, scope, allocation and interval. Shared physical allocations are
sampled once; do not add inclusive totals to their children. KV byte/token
occupancy min/max are sampled extrema, not proven continuous peaks. Application
KV and transformer KV are different cache kinds; identity sharing is forbidden.

## Coverage

Identity/workspaces/sessions; objectives and acceptance; meta routing/budgets;
swarms/agents/model instances; model decoding/quality; tools/skills/hooks/MCP/
plugins; retries/fallbacks; DAG scheduling/messages; auth/policy/seatbelt/holes/
patches/approvals; evidence/receipts/Merkle; resource accounting; KV; context/
compaction/checkpoints; atoms/ingest/retrieval/PAMI/basins/carve/dissolve; novelty/
reranking/trajectories; Obsidian graph revisions/sync; distillation/consolidation/
hunger; experiments/ablations/judges/claims; service health/degradation/recovery;
telemetry spool/replay/loss; releases/configuration/migrations/retention/erasure.

## Compatibility and operational gates

SQLite >=3.37 STRICT tables, JSON1, foreign_keys=ON on every connection,
WAL, synchronous=FULL, bounded busy timeout. SQL migration is versioned and
checksum checked by the admin utility. Single transaction per event and facts.
No Rust dependency or transport changes. Python is offline administration/testing,
not a model worker or inference path. Existing live-map is untouched.

Test malformed status, missing FK, false pass, cross-run references, mutation,
replay conflicts, migration drift/rollback, DAG cycles, budget/expiry limits,
concurrent writers, crash rollback, backups, hash tampering and pressure ingestion.
Persist evidence before advertising completion. No full-system performance claim
follows from a passing database suite.

## Changelog

1.0.0 — 2026-10-29: initial contract; runtime adapters remain proposed.
