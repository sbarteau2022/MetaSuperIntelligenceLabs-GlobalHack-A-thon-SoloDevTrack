---
name: security-auth
description: Enforce deny-by-default auth with server-side authorization, safe password hashing, secure sessions, and OWASP pre-release review.
---

# Security & Auth (Interface)

Purpose: the deny-by-default gate — server-side authorization on every request, memory-hard password hashing, rotated sessions, flagged cookies, OWASP pass before release.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `277ea252…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when designing or reviewing authentication, authorization, session handling, cookie flags, redirect validation, HTML sanitization, or the pre-release OWASP pass.

## 2. Out of Scope

- Endpoint shape and envelopes → `../api-integrations/SKILL.md`.
- Pipeline guard wiring → `../middleware/SKILL.md`.
- UI hiding decisions (never authorization) → `../frontend-dev/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "security-auth" }
```

- Inputs: ban-hit count over the 12 compiled rules (B1 client-only auth … B12 missing OWASP review); allowlist/session-policy/OWASP record on disk as applicable; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 36, 38, 39, 44, 53–57. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md).
- NOT checked: penetration testing and dependency-CVE review (carried over from source).

## 6. Corrections

- None yet.
