# AGENTS.md — N3 (the engineering home) + Atoma Global Engineering Law

Date: 2026-11-01. Status: MANDATED — married from org (9 laws) + prior AGENTS.md (8-law ref) + ENGINEERING-GOVERNANCE.md. Single canonical doc. Sources archived to Trash.

## Marriage note (2026-11-01)
Prior AGENTS.md (PENDING) pointed to ENGINEERING-GOVERNANCE.md for 8-law core. Org file held 9 global laws. Both now married here — one doc. Retired files trashed.

---

## Part I — Atoma Global Engineering Law (9 laws)

ATOMA GLOBAL ENGINEERING LAW — governs every tool, skill, runbook, and agent decision.

Origin: discipline from the Kimi audit, broken down into broader logic discipline, then applied as law. It decides tools, skills, and runbooks — not fashion, speed, or vendor defaults.

1. Quality over speed. Slow and done once, with gates locked, beats doing it ten times because each speed run broke one different thing. No rework loops.

2. Gates before ship. Every tool, hook, loop, skill, rule, subagent, and swarm node passes a boolean type/judge check before it runs or ships. No score, no warning-tolerated pass. A failed gate stops that lane, with a why-receipt: what ran, command, exit, last output, expected result, evidenced cause, unblocker.

3. Bounded, receipted tools. Tools are code, not prompts. A skill is a prompt; a tool is invoked code with route-first, bounded output, truncation, versioning, and a receipt per run. No silent runs, no unbounded output, no tool without a judge.

4. One pass, single shape. Confirm scope and expected output shape in one line before a long run. Check the first artifact against the literal order before completing the batch. Execute a whole list in a single pass; only blockers only the owner can clear stop that item, never the rest.

5. Names native to the harness. No vendor, host app, file path, or IDE names in skill, hook, tool, MCP, plugin, telemetry, runbook, or policy names. Use RG-AH vocabulary: mission, packet, lane, judge, receipt, atom, arc, basin, cache, tap, gate.

6. Auto-load, auto-route, no per-turn invoking. Plugins, skills, and MCP load once at startup, stay live, and route by task. An agent may identify a working-but-improvable hook, rewrite it, test it, stage it with the old version recoverable, set it, then actually use it.

7. Facts bind, interpretation wanders. Facts are binding constraints, never softened for comfort. Interpretation may traverse for cross-modal connections, except when truth-language is used to step around facts. N3 is the persistent anchor outside the session.

8. No synthetic ships. No simulated run is ship evidence. Hardware runs before anything ships. Deletions go to trash, recoverable, never permanent on inference. Re-acquiring a deleted artifact is a decision surfaced first, never a silent sub-step.

9. Landed means live. A file on disk is not landed. Landed means live in the running app, verified against its process, routes, and UI.


---

## Part II — N3 Invariant Core + Canonical Tree + Per-task + Judge table (from prior AGENTS.md)

# AGENTS.md — N3 (the engineering home)

Date: 2026-10-30. Status: PENDING GLOBAL LAW (see ratification block in ENGINEERING-GOVERNANCE.md).
Scope: everything under `~/Desktop/N3/`. This directory is the single home for Atoma engineering: governance, languages, skills, hooks, runbooks, policies.

## 1. The mandate (never skipped or reasoned around in scope)

Production, prototype, and shippable work under N3 follows the invariant core in `ENGINEERING-GOVERNANCE.md` (8 laws: DECLARE, RETRIEVE, FORBID, GATE, PARTITION, PROVE, RECORD, WAIVE-OR-OBEY) plus the adapter that owns the work's domain (Adapter #1: `web-development/`; languages: `languages/`). Core is mandatory everywhere once ratified; an adapter is mandatory within its scope and forbidden outside it. The ONLY legal exception is a written waiver from Stewart — dated, rule-numbered, scoped, expiring, recorded BEFORE the work.

## 2. Canonical tree (one home — nothing canonical lives outside N3)

- `ENGINEERING-GOVERNANCE.md` — the 8-law invariant core + adapter contract + registry (moved from `~/Desktop/skills/`).
- `web-development/` — Adapter #1: 10 area SKILL.md, hooks (schema, rules table, 5 gate scripts, audit, discipline), design language, templates, docs, vendored kimi kit (read-only reference).
- `languages/` — 41+ language skills (Done-means + Rules + Pitfalls + Versions) + `plugin.json` routing.
- `skills/` — harness-meta judiciary (`skill-author/-audit/-implement/-verify`), runbooks, policies.
- `runbooks/` — the effective engineering runbooks (delivery, research, planning, escalation, isolation, verification).
- `plugins/` — pre-existing domain plugins (n3, webdev, frontend-systems, backend-systems, middleware, graphs); canonical, untouched by the move.

Sources outside N3 (e.g. `~/Desktop/skills/`, `~/Desktop/AtomaApp/`, `~/Desktop/N3-OpenDesign/`) are legacy locations. After the move they hold nothing canonical; anything still referenced is marked there as MOVED with its N3 path.

## 3. Per-task minimum (same law as N3-OpenDesign)

1. Calibration line: `BRIEF:` + dials, or `RISK/BLAST-RADIUS/CONTRACT-FIRST`.
2. Load `ENGINEERING-GOVERNANCE.md` first, then the adapter + area SKILL.md the task triggers.
3. Parallel work ⇒ PARTITION: Explorer → Architect (contracts on disk, reviewed) → Coders (each owns ONE small partition front-to-back, never horizontal layers) → gates/Critic pass or fail.
4. Done = `discipline.py` exit 0 AND `audit.py` exit 0 AND language checkers (per `languages/plugin.json`) exit 0, all pasted. No summary-substitutes.
5. Stop-gate veto final; failures recorded; corrections appended, never rewritten.

## 4. Judge table (every law has a judge)

| Law | Judge |
|---|---|
| DECLARE | Reviewer checks the calibration line exists before work starts |
| RETRIEVE | `discipline.py` D2 (skeletons reference canonical paths); no-duplicate-schema review |
| FORBID | `audit.py` deny-rules (`rules-table.json`), fail-closed pre-guard |
| GATE | Architect-output review before fan-out; checklist contract-gate item |
| PARTITION | Delegation messages state owned partitions; reviewer rejects horizontal slices and shared-write scopes |
| PROVE | Pasted checker output (gates, language checkers, tests); coders never self-approve |
| RECORD | `docs/` five-file set; claim labels per §7 |
| WAIVE-OR-OBEY | Waiver exists dated+scoped in Failures.md, or the work didn't happen |

## 5. Corrections

- None yet. Append dated entries; never rewrite history.


---

## Corrections (append-only)

- 2026-11-01 — Married org (9 laws) + AGENTS.md (PENDING) + ENGINEERING-GOVERNANCE.md (8 laws, reconstructed) into single canonical doc. Org file trashed, governance retired (now MANDATED). Complementary — no duplication.
