# CLAIMS_REGISTER.md

Date: 2026-10-30. Status: PROPOSED.
Every substantive claim about this workstream, labeled per AGENTS.md §7. Append; never edit in place.

| #   | Claim                                                                                          | Label    | Evidence                                                                           | Status                                        |
| --- | ---------------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------- | --------------------------------------------- |
| 1   | kimi-frontend-kit contains 14 SKILL.md, 225 files, 9.8MB                                       | VERIFIED | `find … \| wc -l` = 225; `du -sh` = 9.8M; `wc -c skills/*/SKILL.md`                | current                                       |
| 2   | 3/14 kit skills shell out to scripts; 11 are prompt-only                                       | VERIFIED | Skill-audit subagent, file-by-file read 2026-10-30                                 | current                                       |
| 3   | Kit hook lifecycle is SessionStart→PreToolUse→PostToolUse→Stop with a single-issue file store  | VERIFIED | Hook-audit subagent read hooks.json + all 5 scripts                                | current                                       |
| 4   | Kit post-hook overwrites (not appends) its audit log                                           | VERIFIED | Hook-audit: `post-edit-verify.sh` opens log with `"w"`                             | current, fixed in ours (append)               |
| 5   | Kit live hooks and batch audit have drifting rule sets                                         | VERIFIED | Hook-audit: comment advertises a check the script lacks; `linear` vs `ease-linear` | current, fixed in ours (one table)            |
| 6   | Kit pre-guard is coupled to one harness's arg schema                                           | VERIFIED | Hook-audit: reads `toolCall.args.Path/Content/Replacement`                         | current, fixed in ours (HOOK_EVENT_SCHEMA.md) |
| 7   | Kit swarm = Explorer→Architect→parallel Coders→Critic, contracts-gated, critic sole MCP holder | VERIFIED | Swarm-audit read subagent_definitions.json + SWARM_LOGIC + loop doc                | current                                       |
| 8   | Kit optimisation = retrieval + constraint satisfaction                                         | INFERRED | Skill-audit synthesis from ban-lists, exact values, BM25 scripts                   | current                                       |
| 9   | Our audit.py exits 0 on our tree (96 files, kimi-frontend-kit excluded)                        | VERIFIED | Ran 2026-10-30: `audit: 96 files, 0 deny, 0 warnings`, exit 0                      | current                                       |
| 10  | Our hook scripts compile under python 3.14                                                     | VERIFIED | `py_compile` all 6 files, COMPILE-OK, 2026-10-30                                   | current                                       |
| 11  | templates batch: braces balanced, no `latest`, no hex in base.css                              | REPORTED | Templates subagent ran brace-count + greps 2026-10-30                              | needs independent re-check                    |
| 12  | openapi.yaml is structurally valid YAML                                                        | INFERRED | Eye-checked (no tabs/trailing space); PyYAML absent                                | UNVERIFIED — see Pre-Registration #5          |
| 13  | 10/10 area SKILL.md follow the §3 skeleton                                                     | REPORTED | Both authoring subagents confirmed §§1–10 order + Corrections section              | needs independent re-check                    |

| 14 | discipline.py exits 0 on our tree (D1–D7) | VERIFIED | Ran 2026-10-30: `discipline: 0 fail, 0 warnings`, exit 0 | current |
| 15 | audit.py exits 0 on our tree (122 files, kit excluded) | VERIFIED | Ran 2026-10-30: `audit: 122 files, 0 deny, 0 warnings`, exit 0 | current |
| 16 | Agent output is REPORTED until independently re-checked | PROPOSED | RULES.md rule 74, added 2026-10-30 | current |
| 17 | PARTITION replaces SEPARATE as core law 5 | PROPOSED | Owner directive 2026-10-30: small whole-slice ownership, siloed swarms required; propagated to ENGINEERING-GOVERNANCE.md, RULES.md §§9/11, GLOBAL-LOGIC.md §6, README, §9 core list | current |

| 18  | SUBAGENT-CONTRACT.md written as acceptance test for PARTITION E (delegation template, rules, E's exact shapes) | PROPOSED | PARTITION D task 2026-10-30: authored in N3/web-development/docs/, not yet consumed by E | current |
| 19  | TOOL-CONTRACT.md written as acceptance test for PARTITION F (7-field schema, machine-truth table, gate mapping) | PROPOSED | PARTITION D task 2026-10-30: toolchain versions re-run on this machine 2026-10-30 (see Successes entry) | current |

## Corrections

- None yet.

| 20 | SUBAGENT-PROTOCOLS.md written (ROLES table, WHEN/THEN/VIA routing, PARTITION-first, rule 60a, 4-item CONTRACT-GATE, 3-item LOOP-EXIT) | PROPOSED | PARTITION E task 2026-10-30: authored in N3/web-development/docs/ against SUBAGENT-CONTRACT.md acceptance shapes; not yet consumed | current |
| 21 | SUBAGENT-LIFECYCLES.md written (spawn→work→verify→merge, 5-field template + worked example, ownership, critic-reenter-(b), no-self-approval, COMPLETION-LOCK) | PROPOSED | PARTITION E task 2026-10-30: authored in N3/web-development/docs/ against SUBAGENT-CONTRACT.md acceptance shapes; not yet consumed | current |

| 22 | TOOLS-2026.md written: 25 target rows with 7/7 contract fields; 6 NEEDS-VERIFY marks; installs PROPOSED-only, never run | PROPOSED | PARTITION F task 2026-10-30: authored in N3/web-development/docs/ against TOOL-CONTRACT.md acceptance schema; machine truth GIVEN, not probed | current |

| 23 | BACKEND-ATTACK-LIST.md written: 18-row backend queue (11 DONE / 4 NEEDS-EXPAND / 3 MISSING), ranked by OpenDesign need then blast radius, NEXT-UP docker > redis > queues | PROPOSED | PARTITION E (SWARM E) 2026-10-30: ground truth plugin.json 55 ids + TOOLS-2026 machine-truth + adapter-contract; no invented versions, NEEDS-VERIFY where unsure | current |

| 24 | Languages scaffold reshaped to Stewart's spec: plugins/ → mcp-tools/ (top) → web-development/{frontend,backend,middleware}/ → languages/ → <lang>/ → hooks.md, skills.md, tools.md, loops.md, rules.md (60 langs: 15/32/13) | VERIFIED | `find` cross-check 2026-10-30: 60 lang dirs, 5 docs each, zero placeholder dirs/README.md left; TREE.md + 3 index.md + languages-plugin.json (60 entries) consistent | current |
| 25 | Skill bodies carried losslessly from N3/languages/*.md into <lang>/skills.md (only Date/Status footer + provenance pointer appended) | VERIFIED | Sample re-reads (rust, typescript) + assert every skills.md pre-existed as skills/<lang>.md before dir removal; real-content assert blocked any dir with non-placeholder files | current |
| 26 | Routing restored as languages-plugin.json built from plugin-original.json (60-lang superset), skill paths repointed at scaffold | VERIFIED | `json.load` 2026-10-30: 60 entries; orig-vs-legacy diff (19 extra ids, 3 differing rows) recorded; every plugin skill path stat-checked on disk | current |
| 27 | audit.py exit 0 on the reshaped plugins tree (309 files, 0 deny, 0 warnings) | VERIFIED | Ran 2026-10-30: `audit: 309 files, 0 deny, 0 warnings`, exit 0 — pasted, not summarised | current |
| 28 | discipline.py D9 12 FAILs are pre-existing (flat N3/languages/ + plugin.json gone in concurrent reshuffle), not caused by this reshape | VERIFIED | Baseline run BEFORE any move 2026-10-30: identical 12 FAILs; fail set unchanged after reshape | current, needs D9 repoint decision (Stewart) |
