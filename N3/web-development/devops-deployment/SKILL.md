---
name: devops-deployment
description: Keep environment parity with pinned builds, documented env matrix, secret-manager hygiene, and monitored releases.
---

# DevOps & Deployment (Interface)

Purpose: the release-integrity gate — same artifact dev→staging→prod, pinned versions, documented env matrix, secrets outside VCS, monitored rollout.

Status: MIGRATED 2026-10-31 (Agent A1, Wave 2). Full prose retired to Trash original (sha `06a039ad…`); executable core compiled to `compiled/`.
Claims labeled per §7: interface statements below are VERIFIED (they mirror the compiled gate); all how-to prose is PROPOSED-retired (see RECEIPT.md).

## 1. When to Use

Use when cutting a release, changing build config, editing the env matrix, pinning a dependency, or writing a runbook entry.

## 2. Out of Scope

- Endpoint and contract design → `../api-integrations/SKILL.md`.
- Test authoring → `../testing-qa/SKILL.md`.
- Secret primitive design (KDF, sessions) → `../security-auth/SKILL.md`.

## 3. Type Signature

```text
evaluate({ bannedHitCount: number, contractOnDisk: boolean, auditClean: boolean })
  -> { pass: boolean, skill: "devops-deployment" }
```

- Inputs: ban-hit count over the 7 compiled rules (B1 `latest`/unpinned … B7 audit missing from pipeline); env matrix + release record exist on disk; audit clean.
- Output: `pass` true only if zero ban hits AND contract on disk AND audit clean. Fail CLOSED otherwise.
- Executable contract: `compiled/tool.ts` (TS source) → `compiled/tool.rs` (`evaluate` + `BAN_RULES`) → `compiled/tool.json` (schema). See RECEIPT.md for gate evidence.

## 4. Rules Enforced

RULES 9, 53; GLOBAL-LOGIC §8 (pinning); contract rules for typecheck-in-build and release notes. No other rules claimed.

## 5. Removed Claims (VERIFIED — nothing compiled ever existed)

- No `audit.sh` invocation, no live-hook registration, no MCP server, no callable endpoint was ever registered for this area (VERIFIED absent 2026-10-31: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md).
- NOT checked: pipeline runs against real infrastructure (carried over from source).

## 6. Corrections

- None yet.
