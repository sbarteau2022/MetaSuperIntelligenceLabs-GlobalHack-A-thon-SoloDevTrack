---
title: System Workflow Lock
document_type: Architecture Decision Record
version: 2026-11-02
date: 2026-11-02
implementation_date: 2026-11-02
author: Stewart Barteau
reviewers: []
status: LOCKED
related_documents:
  - HOLOMORPHIC_FUNCTION_SPACE.md
  - ../COMMANDMENTS.md
tags: [stores, workflow, evidence, non-synthetic]
---

# System Workflow Lock (from N3 system-workflow.pdf)

LOCKED — do not re-litigate.

## Stores (three, by job; never collapse)
- SurrealDB (127.0.0.1:8000 ONLY): multi mind graph — nodes, edges, atoms, receipts, runs; tensor KV nested inside. Nothing else binds 8000.
- Qdrant (6333): RAG — memories and facts with provenance. Collection research_library_v1.
- HyperDX (8080): telemetry — run traces, tool calls, logs, append-only, per-run tied to DAG nodes.

No single store does every job. No cloud relay. No Postgres in v1.

## Typed workflow (mission → packet → lane → judge → receipt → atom)
Every run follows the typed path; a step that cannot produce its output type does not pass forward. N3 loads in order: TreeSitter, Top MCP, Plugins, Skills.md, Hooks/Tooling (once at startup, kept live). Hooks/tails/port run before chat completion.

## Evidence rules
- No synthetic/simulated runs ship — ever. Proof runs on hardware.
- One atom per passed run through the judge gate.
- Landed means live in the running app (process, routes, UI), not a file on disk.
- Every claim labelled: VERIFIED / REPORTED / INFERRED / PROPOSED.
- Deletions go to trash (recoverable) unless a permanent deletion is explicitly ordered for a named list.
- A run without a receipt did not happen.

## Build order
Stores first (SurrealDB, Qdrant, HyperDX) → load redone map → ingest library with provenance → wire typed workflow → capability tree from DAG (gated) → one proof run on hardware → spend heat where it proves.

Done means: three stores live on their jobs, workflow runs end-to-end on hardware, capability tree contains only proven nodes, every claim in the README points to a section, receipt, or run.
