---
name: database-data
description: Evolve schemas with forward-only additive migrations, dual-compat rollouts, and indexed, timezone-explicit queries.
---

# Database & Data (Interface)

Purpose: the data-evolution gate — forward-only additive migrations, expand/contract dual-compat, indexed hot paths, UTC storage.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `6da13013…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when writing or reviewing a schema migration, data fix script, index change, query, seed, or cache-invalidation rule.

## 2. Out of Scope

- Endpoint shape and versioning → `../api-integrations/SKILL.md`.
- Server code consuming the data → `../backend-systems/SKILL.md`.
- Secret storage design → `../security-auth/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "database-data" }
```

- Inputs: ban-hit count over the 7 compiled rules (B1 in-place migration edit … B7 secret in migration/seed); migration file exists on disk with documented rollback; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 45–48 (plus 44 for secrets). No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md).
- NOT checked: migration behavior against a live database (carried over from source).

## 6. Corrections

- None yet.
