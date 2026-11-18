---
name: ares-agents
description: Design and operate Ares agent lanes (Qwen builder, Muse contributor, external/OpenAI-compatible) as endpoint-backed configurations with lane assignment, circuit breakers, and telemetry. Use when configuring, debugging, or extending agent lanes.
---

# Ares Agents

## Scope
- Lane definitions: `backend/src/lanes.rs` (QwenLane, NvidiaLane, MuseLane, chat_openai_compatible, embed_openai_compatible).
- Agent roster: `backend/src/agent_config.rs` (3 slots: Builder, Researcher, Verifier).
- Endpoint registry: `backend/src/endpoints.rs`.
- Circuit breaker: `backend/src/server.rs` (per-lane open after 3 failures).

## When to use
- Configuring a new agent lane or endpoint.
- Debugging lane failures, circuit breaker trips, or throughput.
- Enforcing agent roster constraints and lane validity.

## Rules
- A lane's default model is adopted at startup; failures are recorded, not hidden.
- Agent lanes remain plain completions; MCP tools are never auto-injected into lanes.
- Secrets resolve by slot name at call time; never inline.

## Outputs
- Lane probe results (`/lanes`), telemetry (`/metrics`), circuit breaker state.
