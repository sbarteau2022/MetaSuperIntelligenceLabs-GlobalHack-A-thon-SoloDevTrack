# Design.md — Web-Development Scaffold Workstream

Date: 2026-10-30. Status: PROPOSED.

## What

A durable, stack-agnostic web-development folder under `skills/web-development/` that encodes an operating discipline, not just a directory tree: skill contracts, executable hooks, token system, swarm protocol, templates, and governance docs.

## Why (INFERRED from the request thread)

Flat folder trees decay: placement conventions live in chat history, rules are re-litigated per task, and quality gates depend on memory. The kit audit showed the fix is structural — retrieve-don't-invent (queryable data), negative contracts (exact ban-lists), defaults-over-choices (pinned templates), and mechanical gates (deny fast, veto dirty completion).

## Non-goals

- Not a runnable app. No framework, no lockfile, no CI wiring. Templates are skeletons.
- Not a fork of the kit. `kimi-frontend-kit/` is vendored read-only reference; our logic is stack-agnostic where the kit is React-19-locked.
- Not a second brain. Claims discipline, five-file set, and Mermaid rules come from AGENTS.md §7 and stay there as authority.

## Structure (VERIFIED on disk 2026-10-30)

`GLOBAL-LOGIC.md` (pinned first) → `RULES.md` → 10 area `SKILL.md` → `hooks/` (schema + rule table + 5 scripts + batch audit) → `frontend-design-language/` (tokens.css + README) → `templates/` (12 starters) → `docs/` (this set) → `kimi-frontend-kit/` (vendored reference).

## Decisions

1. Python stdlib-only hooks (no tsc/eslint dependency) so gates run anywhere. PROPOSED.
2. Append-only issue store + archive-on-veto (fixes kit's last-write-wins log). PROPOSED.
3. Kit excluded from our audit scope (reference, not owned). PROPOSED.
4. Two skill tiers: full (generative areas) vs flat (convention areas). From kit's cortex pattern. PROPOSED.
5. Risk dials (`RISK/BLAST-RADIUS/CONTRACT-FIRST`) for non-generative areas instead of forcing visual dials. PROPOSED.

## Decisions (added 2026-10-30, discipline enforcement)

6. Discipline is REQUIRED via three mechanisms, not one document: MUST-level rules (RULES.md §11) + executable gates (`discipline.py` D1–D7, `audit.py` 18 rules) + completion lock (stop-gate veto / release-checklist sign-off). Any one layer alone is advisory; all three together are required.
7. `discipline.py` checks STRUCTURE (files exist, skeletons in order, rule table valid, no floating pins); `audit.py` checks CONTENT (no secrets, no banned patterns). Both MUST exit 0. Neither replaces typecheck, tests, or human review.
8. Agent-reported results are REPORTED until independently re-checked (rule 74). Structural claims get re-run, not trusted.

## NOT checked

Live harness wiring of `hooks.json` (no harness here to fire events against); PyYAML parse of `openapi.yaml` (PyYAML absent — eye-checked only); `tsc --noEmit` over templates (no TS toolchain wired).
