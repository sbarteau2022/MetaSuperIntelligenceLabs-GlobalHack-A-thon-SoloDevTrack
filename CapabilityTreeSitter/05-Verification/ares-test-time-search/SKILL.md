---
name: ares-test-time-search
description: "Test-time search and tree reasoning for Ares: DAG traversal, document graph, PAMI phase-augmented search, branch-and-verify. Use when exploring search strategies or reasoning trees."
---

# Ares Test-Time Search & Tree Reasoning

## Scope
- Workflow DAG: `backend/src/workflows.rs` (packets, dependencies, judges).
- DIA-1 recall: doc shortlist → dense + BM25 → RRF → whole-doc assembly.
- PAMI: phase rotation (rank 0 chunks, rank 1 docs), participation ratio.
- Graph: contains/parent/related edges, bounded 1-hop expansion.

## Rules
- No full-scan traversal without an index/key.
- Graph paths are recorded for explainability.
- Top-1 accuracy must not regress on measured benchmarks.
