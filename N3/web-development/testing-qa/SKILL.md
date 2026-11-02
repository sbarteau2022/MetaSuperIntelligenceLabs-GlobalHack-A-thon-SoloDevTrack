---
name: testing-qa
description: Write hermetic unit, integration, and e2e tests with error/empty/loading coverage, regression-first fixes, and verifiable pass claims.
---

# Testing & QA (Interface)

Purpose: the verifiable-quality gate — regression-test-first, hermetic units, happy+error+empty(+loading) coverage, pass claims carry verbatim checker output.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `72ee11680…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when writing unit, integration, or e2e tests; building fixtures; authoring QA checklists; reproducing bugs; or verifying a fix.

## 2. Out of Scope

- What the UI should look like → `../ui-ux-flow/SKILL.md`, `../frontend-design-language/SKILL.md`.
- API contract shape under test → `../api-integrations/SKILL.md`.
- CI pipeline wiring → `../devops-deployment/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "testing-qa" }
```

- Inputs: ban-hit count over the 6 compiled rules (B1 happy-path-only … B6 unlabeled claims); reproduction test + verbatim checker output on disk; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 4, 49–52, 58, 59. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: flakiness under CI load and real-browser rendering (carried over from source).

## 6. Corrections

- None yet.
