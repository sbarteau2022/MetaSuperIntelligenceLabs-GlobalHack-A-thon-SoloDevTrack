---
name: frontend-design-language
description: Consume and extend the canonical token, type, color, motion, theme, and component-spec system for all UI work.
---

# Frontend Design Language (Interface)

Purpose: the canonical token/type/color/motion/theme/component-spec gate — semantic consumption only, single-source tokens, fixed layer order.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `2e72200e…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when defining or consuming design tokens, type scale, color ramps, motion curves, themes, or component specs; and whenever any prompt asks what color / spacing / duration / font a value should be.

## 2. Out of Scope

- User flows and wireframes → `../ui-ux-flow/SKILL.md` (this skill receives token gaps flagged as `TOKEN-GAP`; it does not write flows).
- Component implementation → `../frontend-dev/SKILL.md`.
- Contrast/keyboard acceptance failures → `../testing-qa/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "frontend-design-language" }
```

- Inputs: ban-hit count over the 8 compiled rules (B1 tier-1 literal in components … B8 physical property); token file is the single source on disk; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 7, 20, 21, 24–35. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: rendered contrast ratios and keyboard behavior in a browser (carried over from source).

## 6. Corrections

- None yet.
