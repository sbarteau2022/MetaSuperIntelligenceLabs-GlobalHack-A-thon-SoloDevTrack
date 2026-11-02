---
name: middleware
description: Build the ordered request/response pipeline — auth, validation, logging, security, performance — fail-closed with one error envelope.
---

# Middleware (Interface)

Purpose: the pipeline-order gate — stages 1–9 in fixed order (request-id → logging → security headers → CORS → rate-limit → auth → validation → handler → error envelope), fail-closed guards.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `38b46565…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).
NOTE (INFERRED — dest diverged pre-migration): N3 dest carried an extra Framework Profiles §10 (Express / Meteor-style / ASP.NET Core mappings, frozen-order constraint) absent from the Trash source. Profiles are prose guidance, not compiled gates; they are NOT carried into this interface. If profiles are wanted, they belong in the area README or docs, re-added by owner decision.

## 1. When to Use

Use when the task touches the request/response pipeline: request IDs, logging, security headers, CORS, rate-limiting, auth guards, input validation middleware, compression/caching, timeouts, or the central error handler.

## 2. Out of Scope

- Domain logic, controllers, services, jobs → `../backend-systems/SKILL.md` (middleware never calls services; it guards and forwards).
- Client code → `../frontend-dev/SKILL.md`.
- Auth primitive design (KDF choice, session rotation, cookie flags) → `../security-auth/SKILL.md` (this skill *wires* the guards).

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "middleware" }
```

- Inputs: ban-hit count over the 9 compiled rules (B1 duplicated auth … B9 middleware importing services); per-middleware contract on disk (stage number, fail-closed behavior); pipeline order matches §4 order exactly; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 36–44 (plus 38, 39). No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md). The §6 audit-table references in the retired prose pointed at a PROPOSED rollout and are NOT carried forward.
- NOT checked: none beyond source scope.

## 6. Corrections

- None yet.
