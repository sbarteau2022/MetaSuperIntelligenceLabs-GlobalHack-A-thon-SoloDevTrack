---
name: frontend-dev
description: Build client-side components, pages, routes, state, styles, and assets that pass the a11y, token, and audit gates.
---

# Frontend Dev (Interface)

Purpose: the client-implementation gate — native HTML first, semantic-token-only styles, fixed layer order, motion/density fallbacks.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `fadedfe1…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when the task touches `components/`, `pages/`, `hooks/`, `store/`, `styles/`, `assets/`, `lib/`, client routes, client state, or client build config.

## 2. Out of Scope

- Canonical token definitions or theme architecture → `../frontend-design-language/SKILL.md` (this skill only *consumes* tokens).
- Server code → `../backend-systems/SKILL.md`.
- Request-pipeline code → `../middleware/SKILL.md`.
- Flow maps, journey logic, screen inventory → `../ui-ux-flow/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "frontend-dev" }
```

- Inputs: ban-hit count over the 12 compiled rules (B1 div-role-button … B12 nontrivial inline style); per-component doc block on disk (anatomy, 5 states, a11y notes, token list); audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 11–35. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: none beyond source scope.

## 6. Corrections

- None yet.
