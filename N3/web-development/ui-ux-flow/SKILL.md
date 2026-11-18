---
name: ui-ux-flow
description: Map user flows and produce wireframes, mockups, prototypes, and component specs with full state and accessibility coverage.
---

# UI/UX Flow (Interface)

Purpose: the flow-completeness gate — every flow ships happy+error+empty+loading, Mermaid diagrams, token-by-name specs with a11y notes.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `20837ddd…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when the task involves user flows, wireframes, mockups, prototypes, component specs, accessibility notes, or UX research synthesis.

## 2. Out of Scope

- Defining or changing design tokens → `../frontend-design-language/SKILL.md` (this skill links tokens, never redefines them).
- Implementing components in code → `../frontend-dev/SKILL.md`.
- API shapes behind a flow → `../api-integrations/SKILL.md`.
- Acceptance-test authoring → `../testing-qa/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "ui-ux-flow" }
```

- Inputs: ban-hit count over the 8 compiled rules (B1 placeholder copy … B8 sub-24px targets); flow doc + component spec on disk (Mermaid, state table, a11y notes, token list); audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 7, 11–24, 50, 58, 61. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: rendered output on real devices and assistive tech (carried over from source).

## 6. Corrections

- None yet.
