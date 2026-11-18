# Successes.md

Date: 2026-10-30. Append-only.

## 2026-10-30 — Scaffold + kit ingestion complete

- VERIFIED: Brought `kimi-frontend-kit/` in verbatim (225 files, 9.8MB, byte-identical copy) under `web-development/` — provenance preserved, kit untouched.
- VERIFIED: 4 parallel read-only audits (hooks lifecycle, swarm protocol, skill-file pattern, rules+templates) returned structured findings with zero file writes.
- VERIFIED: `GLOBAL-LOGIC.md` pinned at folder top — the one-sentence version, five layers, skill skeleton, hook lifecycle, swarm law, dials, compliance map.
- REPORTED: 10/10 area `SKILL.md` written (6 full-tier, 4 flat-tier) in the §3 skeleton, each ending with a checkbox gate and `## 10. Corrections`.
- VERIFIED: `hooks/` executable — shared `rules-table.json` (18 rules), 5 stdlib-only scripts, batch `audit.py` + `audit.sh`; self-audit exit 0; full-tree audit exit 0 (96 files, kit excluded as reference).
- VERIFIED: Two live rule-engine fixes during self-audit — `exclude` support added for `tokens.css` literal palette; `kimi-frontend-kit/` excluded from audit scope (reference, not owned).
- REPORTED: 12 starter templates (middleware ×4, openapi.yaml, env.example, base.css, release-checklist, docker-compose, migration template, secrets policy); braces balanced, no `latest`, no hex in base.css.
- VERIFIED: `RULES.md` extended with §9 (swarm protocol + calibration) and a 3-item-longer enforcement checklist.

## 2026-10-30 — Discipline made required (not advisory)

- VERIFIED: `hooks/discipline.py` added (D1–D7 structural gate) and exits 0 on first run.
- VERIFIED: `RULES.md` §11 added (rules 69–75) — gate-before-done, audit-before-done, stop-gate lock, disjoint delegation, independent re-check of agent output, no invented scripts.
- VERIFIED: release checklist extended with discipline gate, calibration line, swarm contract gate, and stop-gate sign-off.
- VERIFIED: both gates re-run after the change — audit exit 0 (122 files), discipline exit 0.

## 2026-10-30 — PARTITION law adopted (owner directive)

- SEPARATE withdrawn and replaced: law 5 is now PARTITION — decompose into many small partitions, one mind owns each front-to-back, siloed swarms as REQUIREMENT.
- New rule 60a: slices are vertical (feature/surface/contract), never horizontal layers.
- Propagated to ENGINEERING-GOVERNANCE.md law 5, RULES.md §9 title + rules 60a/72/73, GLOBAL-LOGIC.md §6 (rewritten + re-titled) with dated correction entry, web-development README adapter note, and ~/.config/zed/AGENTS.md §9 core list.

## 2026-10-30 — N3 move measured + PARTITION D contracts written

- VERIFIED: N3 move measured on disk 2026-10-30 — 42 language `.md` files (+ `plugin.json` = 43 files) in `N3/languages/`; 354 files in `N3/web-development/`, all common files byte-identical to `skills/web-development/` (`diff -r -q` clean except the source-side marker and the two new PARTITION D contracts); 9 skill dirs in `N3/skills/`; 14 docs in `N3/runbooks/` (12 numbered + INDEX.md + README.md).
- NOTE (dated 2026-10-30, supersedes brief count): the PARTITION D brief stated "7 runbook docs" — true at brief time (01–06 + README.md, all timestamped 15:23); runbooks 07–12 + INDEX.md landed 15:26–15:27. Disk truth is now 14.
- VERIFIED: move markers placed — `skills/web-development/CANONICAL-HOME-IS-N3.md` and `skills/plugins/languages/CANONICAL-HOME-IS-N3.md` ("canonical home is now ~/Desktop/N3/").
- PROPOSED: `SUBAGENT-CONTRACT.md` + `TOOL-CONTRACT.md` written in `N3/web-development/docs/` as acceptance tests for PARTITIONS E/F (see CLAIMS_REGISTER #18–19).

## 2026-10-30 — PARTITION B engine fixes landed (synthetic suite green)

- REPORTED: Three engine bugs fixed per the move manifest + PARTITION B brief — `fnmatch` scope bug + `lstrip` bug + exclude-component bug, all fixed.
- REPORTED: Synthetic suite green; audit exit 0 at 126 files; `discipline.py` exit 0 with D8.

## 2026-10-30 — Languages scaffold reshaped to Stewart's five-doc spec

- VERIFIED: `plugins/mcp-tools/README.md` top node (7 core tools, route-first contract, staging pointers); `plugins/web-development/{frontend,backend,middleware}/languages/<lang>/{hooks,skills,tools,loops,rules}.md` for all 60 languages (15 frontend / 32 backend / 13 middleware).
- VERIFIED: per-doc content — `tools.md` from routing metadata (exts/checks/harness-tools/trigger-words), `hooks.md` from LANGUAGE-GATES.md rows + rules-languages.json scoping (8 langs hit, 52 honestly marked none), `loops.md` from runbooks, `rules.md` index-only (no duplicate normative copy), `skills.md` = carried body + footer.
- VERIFIED: gates — `audit.py` exit 0 (309 files); `discipline.py` fail set identical to pre-work baseline (12 pre-existing D9s only).
