---
name: runbook-plan-and-judges
description: Decompose 200+ step systems engineering goals into DAG runbooks with pre-registered context, tools, and deterministic judges.
user-invocable: true
depends_on: []
load_mode: eager
---

# Runbook Plan-as-Data & Judge Pre-Registration

Long-horizon systems engineering runs execute as a directed acyclic graph (DAG) of packets, never as an unbounded conversational loop.

## Core Rules

1. **Two-Lane Separation**:
   - **PLAN Lane (`think_mode: high`)**: Decomposes the goal into atomic packets (`max_packets: 512`, hard cap `1024`). Specifies deliverable contracts, `depends_on` edges, `declared_context`, `declared_tools`, and deterministic `judges`. Never over-specify brittle line-by-line implementation details that cascade errors downstream.
   - **CODE Lane (`think_mode: off`)**: Claims one dependency-ready packet (`ready_packets`) at a time inside an isolated hole cavity (`hole_id`).
2. **No Self-Graded Completion**:
   - Every packet must declare at least one deterministic judge (`nonempty`, `contains`, `file_exists`, `command` exit code, or `GoldenOutput` SHA-256).
   - A packet transitions to `passed` only when all declared judges pass.
3. **Orientation Protocol Before Every Session**:
   - Read `mind.tail` and `GET /events` to reconstruct the exact state of prior packets.
   - Verify the workspace/hole passes the baseline build or smoke judge before admitting new work.
