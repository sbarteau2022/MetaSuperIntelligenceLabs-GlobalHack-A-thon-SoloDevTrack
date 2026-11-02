---
name: kimi-frontend-orchestrator
description: Master Kimi K2.5 / OK Computer frontend orchestrator skill. Executes the 5-stage Coding-Driven Design loop, 3-Dial aesthetic calibration, Subagent Swarm delegation, Double-Bezel surface physics, and Vision-in-the-Loop Chrome DevTools QA.
---

# kimi-frontend-orchestrator (interface)

## Purpose
Master orchestration for high-craft builds: 4-line calibration block before code,
skill-chain matrix routing, subagent swarm with pinned `domain.ts` + Zustand store
contracts, Zero-Stub quality floor, live-browser verification to close.

## When to use
Complex multi-surface frontend builds (3D/WebGL, workbenches, knowledge graphs,
DAG builders) where subagents code against shared types. Phase 0 calibration, then
delegated slices into `src/App.tsx`.

## Out of scope
Jumping straight into `.tsx` without the calibration block. Dead clicks (every
control must mutate live Zustand state with visible feedback). Primitives missing
any of the 5 interaction states (default/hover/active/focus-visible/disabled).
Declaring done without live-browser verification (`chrome-devtools-mcp` is an
environmental QA step, not a compiled claim — host runtime performs the browsing).

## Type signature
Compiled gate: `tool.rs` (`orchestrator-qa-gate`, schema: `tool.json`).
- `qa_gate(cfg: OrchestratorQA) -> bool` — calibration AND types AND zero-stub AND browser-verified
- `dead_click_free(clicks_live: boolean) -> bool`
- `five_states_present(count: number) -> bool` — count >= 5
- `make_qa(calibration_done, types_pinned, zero_stub, browser_verified) -> OrchestratorQA`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/kimi-frontend-orchestrator/SKILL.md`
(`da6717413f36a2d3a85a6d3d6e7de7a0de90caac`). Shrunk to this interface 2026-10-31
(Agent A2). `npx tsgo` is a shell check; `chrome-devtools-mcp` QA kept as procedure,
no callable schema existed in source (grep VERIFIED across N3 MCP configs — absent);
nothing removed. See `RECEIPT.md`.
