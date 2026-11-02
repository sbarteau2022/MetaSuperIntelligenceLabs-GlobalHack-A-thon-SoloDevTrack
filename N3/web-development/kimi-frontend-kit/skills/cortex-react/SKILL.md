---
name: cortex-react
description: "Apply these opinionated React conventions when writing React 18+ or 19 components: hooks patterns, Server Components, Suspense boundaries, state management, performance memoization, React 19 features (use, form actions)."
---

# cortex-react (interface)

## Purpose
Opinionated React 18/19 conventions: Server Component by default, `'use client'`
pushed to the deepest leaf, serializable boundary props, state at its natural
level, effects only for external sync, stable keys, role-based test queries.

## When to use
Writing or reviewing React 18+/19 components: hooks patterns, Server Components,
Suspense boundaries, state management, memoization, React 19 `use` / actions.

## Out of scope
Non-React UI work. Class components. Effects for derived data (belongs in render).
Index-as-key. `getByTestId` where a role query reaches the element. Test IDs as
primary query strategy.

## Type signature
Compiled gate: `tool.rs` (`react-conventions-gate`, schema: `tool.json`).
- `boundary_valid(boundary: String) -> bool` — "deep-leaf-client"
- `state_valid(state_level: String) -> bool` — "local" | "lifted-shared" | "zustand-global" | "tanstack-server"
- `async_valid(async_model: String) -> bool` — "suspense-action"
- `react_gate(cfg: ReactConventions) -> bool` — server_by_default AND the above
- `make_react(server_by_default, boundary, state_level, async_model) -> ReactConventions`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/cortex-react/SKILL.md`
(`7ed463b4c6f4e70231267c0997ed956d81e158e9b64f24cf69f6524ea47b12d6`).
Full prose shrunk to this interface 2026-10-31 (Agent A2). No tool/server claims in
source; nothing removed. See `RECEIPT.md`.
