---
name: backend-systems
description: Build server-side routes, controllers, services, models, jobs, config, and validators behind thin controllers and a single error envelope.
---

# Backend Systems (Interface)

Purpose: the server-side structure gate — thin controllers (parse → authorize → service → serialize), aggregate-owned services, one error envelope.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `cbf0f4f2…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when the task touches `routes/`, `controllers/`, `services/`, `models/`, `jobs/`, `config/`, `utils/`, `validators/`, API shapes, domain logic, or background work.

## 2. Out of Scope

- Request-pipeline code (guards, validators-as-middleware, error handler) → `../middleware/SKILL.md`.
- Client code → `../frontend-dev/SKILL.md`.
- Schema migrations and data scripts → `../database-data/SKILL.md`.
- Auth primitives (hashing, sessions, cookies) → `../security-auth/SKILL.md` (this skill *calls* those primitives, never reimplements them).

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "backend-systems" }
```

- Inputs: ban-hit count over the 10 compiled rules (B1 fat controller … B10 secrets in VCS); shared types/API shapes exist on disk before code; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 36–42 (plus 7, 9, 44). No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: none beyond source scope.

## 6. Corrections

- None yet.
