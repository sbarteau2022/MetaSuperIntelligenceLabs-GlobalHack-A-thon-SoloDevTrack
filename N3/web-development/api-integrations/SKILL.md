---
name: api-integrations
description: Ship versioned API contracts with generated types, standard envelopes, cursor pagination, and safe webhooks.
---

# API & Integrations (Interface)

Purpose: the versioned-contract gate for every public surface — one schema, generated types, one envelope, cursor pages, replay-safe webhooks.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `94d21f13…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when defining or consuming a versioned endpoint, contract schema, error envelope, pagination shape, or webhook handler.

## 2. Out of Scope

- Server internals (controllers, services, jobs) → `../backend-systems/SKILL.md`.
- Request-pipeline wiring (guards, rate-limit, error handler) → `../middleware/SKILL.md`.
- Login/session/secret design → `../security-auth/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "api-integrations" }
```

- Inputs: ban-hit count over the 9 compiled rules (B1 unversioned route … B9 hand-duplicated generated types); contract file exists on disk and is the single source; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 36–42, 44; contract rule for 415 / Retry-After / generated-types-single-source. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: generated clients against live providers (carried over from source).

## 6. Corrections

- None yet.
