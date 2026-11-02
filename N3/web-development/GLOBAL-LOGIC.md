# GLOBAL-LOGIC.md — The Operating Logic of This Folder

Status: PROPOSED (synthesised 2026-10-30, ratified by owner on use).
Date: 2026-10-30.
Source: audit of `kimi-frontend-kit/` (14 skills, 6 hook scripts, 6 subagent roles, 2 rule files, 3 templates, 1 workflow loop), vendored verbatim at `kimi-frontend-kit/`. Claim labels per AGENTS.md §7. **Read this file first. Everything else in this folder implements it.**

---

## 0. The One-Sentence Version

> **Retrieve, don't invent** (skills query data, not vibes) → **declare context, then forbid the past** (hooks inject the stack, deny known-bad writes) → **contracts before parallel work** (architect writes shared types first, coders own disjoint trees) → **refuse to finish dirty** (warnings accumulate, the stop-gate converts them into a hard veto).

---

## 1. Where This Came From (provenance)

- VERIFIED: `kimi-frontend-kit/` contains 14 `SKILL.md` files (206,856 bytes total), `hooks/hooks.json` + 5 executable scripts, `subagents/subagent_definitions.json` (6 roles) + `SUBAGENT_SWARM_LOGIC.md`, `workflows/KIMI_OK_COMPUTER_LOOP.md` (5 phases), `rules/rules.md` + `rules/AGENTS.md`, and 3 build templates. Audited 2026-10-30 by 4 parallel read-only subagent audits.
- VERIFIED: 3 of 14 skills shell out to code (`ui-ux-pro-max/scripts/search.py` BM25-over-CSV retrieval, `design-system` token generators/validators, `ui-styling` component generators). The other 11 are prompt-only: exact tokens, ban-lists, and checklists the agent reproduces.
- INFERRED: the kit's optimisation is **retrieval + constraint satisfaction** — exact values eliminate taste-decisions, ban-lists prune the model's highest-probability generic outputs, and scripts move ranking out of the context window.

## 2. The Five Layers (what the kit actually is)

Every durable agent kit in this folder follows the same five layers, in this order:

| #   | Layer                    | Kit evidence                                                                                                              | This folder's implementation                                                                      |
| --- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| 1   | **Skill contract**       | 14 `SKILL.md`: frontmatter routing + scope gate + banned list + positive spec + execution protocol + pre-output checklist | Each subsystem ships a `SKILL.md` in this exact shape (see §5)                                    |
| 2   | **Data over prose**      | 15 CSV catalogs + `catalog-summary.json` + `data-provenance.json`; `reasoning_contract.py` decision grammar               | `frontend-design-language/tokens/`, per-area `data/` catalogs; scripts query, prompts don't paste |
| 3   | **Executable retrieval** | `search.py --domain --stack --design-system`; `--persist` writes `MASTER.md`                                              | Area scripts under `tools/` invoked by absolute path, CWD stays at project root                   |
| 4   | **Hook lifecycle**       | `SessionStart → PreToolUse → PostToolUse → Stop` with a single-issue store between post and stop                          | `hooks/hooks.json` + `hooks/scripts/` (see §4)                                                    |
| 5   | **Swarm protocol**       | Explorer → Architect (contracts gate) → parallel coders (disjoint ownership) → Critic (sole verifier with live tools)     | Routing rule + contract gate + loop exit condition (see §6)                                       |

## 3. The Universal Skill-File Pattern (mandatory shape)

INFERRED from all 14 kit skills. Every `SKILL.md` in this folder MUST follow this skeleton, in order:

```markdown
---
name: <kebab-case>                    # routing: matched by loader
description: <one line: what + when> # routing: trigger conditions
---

# <Discipline Title> (<persona>)

## 1. When to Use / Out of Scope # scope gate: triggers + explicit non-triggers

## 2. Banned Patterns (fail = reject) # negative contract: EXACT names/values, not vibes

## 3. Calibration # 1-line brief + dials; declared BEFORE generating

## 4. System Spec # positive contract: tokens, types, rules + code

## 5. Reference Patterns # copy-pasteable canonical skeletons per use-case

## 6. Scripts & Data (if any) # invocation paths, query contract, retry/fallback rule

## 7. Execution Protocol # numbered pipeline: analyze → generate → review → build

## 8. Pre-Output Checklist # checkbox QA gate; nothing ships with an unchecked box

## 9. Chaining & References # sibling skills, references/*.md, canonical sources
```

Why this order (INFERRED): §§1–2 bound the behavior space before any generation happens; §3 forces a declared plan so repeated runs diverge per brief instead of collapsing to the default; §§4–5 supply exact values to **copy instead of invent**; §6 offloads search to code; §§7–8 make review mandatory, not advisory; §9 keeps skills composable instead of monolithic.

Two tiers are allowed:

- **Full skill** (§§1–9): `frontend-dev`, `backend-systems`, `middleware`, `ui-ux-flow`, `frontend-design-language`, `testing-qa`.
- **Flat skill** (frontmatter + `## Conventions` list + worked examples, cf. kit's `cortex-*` pair): `database-data`, `api-integrations`, `devops-deployment`, `security-auth`.

## 4. The Hook Lifecycle (mandatory shape)

Distilled from the kit's 4 wired hooks + 1 batch script (VERIFIED behavior; kit gaps FIXED here):

```text
1. on session_start → detect stack (workspaces, engines, scripts) → inject constraints as system context. NEVER blocks.
2. on pre_write(path, content) → if matches deny-list → DENY + reason + fix. Fail CLOSED on parse error.
3. on post_write(path) → if matches warn-list → APPEND to pending-issues store. Never blocks, never judges.
4. on stop(reason==idle) → if pending-issues non-empty → CONTINUE + issues. Delete store only after read (no loops).
5. batch_audit(dir) → same deny+warn rules over a tree → exit 2 + machine-readable report. Same rules as live hooks or they WILL drift.
```

Kit gaps fixed here (VERIFIED in kit, corrected in `hooks/`):

- Kit's post-hook **overwrote** the issue store per event (last-write-wins, earlier warnings silently dropped). Ours **appends**.
- Kit's live hooks and batch audit had **drifting rule sets** (comment advertised a check the script lacked; `linear` vs `ease-linear`). Ours share ONE rule table sourced from a single file.
- Kit's pre-guard was coupled to one harness's arg schema. Ours reads a documented `HOOK_EVENT_SCHEMA.md` and fails closed with a schema error, not a silent allow.
- Kit wrote state to the workspace root. Ours writes to `.agent-state/` (gitignored).

## 5. Workflow Optimisation (how this folder makes agents faster)

The kit's optimisation, generalised (INFERRED, applied here):

1. **Retrieval over generation.** Decisions live in queryable files (tokens, catalogs, contracts). The agent looks up instead of inventing. Context stays small because data stays on disk.
2. **Negative contracts first.** Banned lists prune the model's most probable (most generic) outputs. A ban-list is cheaper than a taste lecture and enforces by literal match.
3. **Defaults over choices.** One import path, one state pattern, one easing curve, one error envelope — recorded in `RULES.md` and templates. Bikeshedding is a context-window tax.
4. **Fail fast, fail loud.** Deny at write-time (pre-hook), verify per-edit (post-hook), typecheck + build at the phase gate, refuse dirty completion (stop-hook). Each layer catches what the previous one can't.
5. **Amortise decisions.** `--persist`-style writes (`MASTER.md`, token files, contracts) mean each later task starts from recorded decisions, not a blank brief.
6. **Defer the subjective, deny the objective.** Objective violations (secrets, banned APIs) block immediately; subjective ones (motion curves, TODOs) accumulate as warnings and veto completion. Never block the loop on taste; never let taste ship unchecked.

## 6. Partition & Swarm Protocol (siloed swarms are a REQUIREMENT)

Distilled from `SUBAGENT_SWARM_LOGIC.md` + `subagent_definitions.json` + `KIMI_OK_COMPUTER_LOOP.md` (VERIFIED structure, stack-agnosticised, PARTITION reading 2026-10-30 — see Corrections):

```text
1. ROLES: Explorer (read-only, maps repo) → Architect (writes SHARED CONTRACTS) → N Coders (parallel, each owns ONE SMALL PARTITION front-to-back) → Critic (verifies live, holds sole external-tool access).
2. ROUTING: task ≤3 files and single engine → solo. Else explorer → architect → parallel coders.
3. PARTITION RULE: decompose into many small partitions; one mind engineers its partition end-to-end. Slices are VERTICAL (a feature, a surface, a contract) — never horizontal layers (all routes here, all styles there). Small and focused is how we win.
4. CONTRACT GATE: no coder starts until shared types/store/API contract exist ON DISK. This is the single serialization point. A bad contract poisons every slice — the architect's output is reviewed before fan-out.
5. OWNERSHIP RULE: each coder owns its partition exclusively, front-to-back; everything outside it is READ-ONLY. No merge protocol beyond partitioning — keep slices partitionable.
5. VERIFY LAYERS: (a) per-edit static check, (b) typecheck + build + batch-audit gate, (c) live-runtime critic (errors, layout math, screenshots, interactions).
6. VERIFICATION RULE: coders NEVER self-approve. Only deterministic gates and the critic may pass work. The critic may patch, but a critic-introduced change re-enters layer (b).
7. LOOP EXIT: done IFF typecheck ∧ build ∧ audit clean AND zero runtime errors AND critic checks pass; else patch and re-enter at the FAILING layer, not from the top.
8. COMPLETION LOCK: a stop-hook re-checks all gates; any failure vetoes "done".
```

Known kit gaps inherited as open risks (INFERRED): the critic holds both the pen and the stamp (verifier = patcher); visual QA has no a11y specialist beside it; directory-partitioning is the only concurrency control — cross-slice shared files break the model. See `docs/Failures.md`.

## 7. Calibration Dials (declared before generating)

From the kit's Phase-0 (VERIFIED): every generative task declares these UP FRONT, 1–10, in one line:

- `DESIGN_VARIANCE` — how far from the default aesthetic (1 = system-conformant, 10 = bespoke).
- `MOTION_INTENSITY` — animation energy (1 = static/fades only, 10 = choreographed).
- `VISUAL_DENSITY` — information density (1 = spacious, 10 = compact).

Plus `STACK` (what renders this) and `BRIEF` (one line: what + for whom). No generation starts without this line. It is the cheapest anti-slop device in the kit: declared variance beats default variance.

## 8. File-Level Rules Imported Into This Folder

- Skill files are **query interfaces over data**, not essays. If a section can be a table or a script, it MUST be.
- Ban-lists use **exact values** (`!important`, `* {}`, `<div role="button">`, `eval(`, `.env` writes) — never "avoid bad patterns".
- Every skill ends in a **checkbox gate**. Nothing ships with an unchecked box.
- Scripts are invoked by **absolute skill path**, CWD stays at project root. No CWD-relative `./scripts/` assumptions.
- Rules files come in two depths: `RULES.md` (enforcement stub, loaded always) → area `SKILL.md` (spec, loaded on trigger). Never invert this.
- Templates pin versions and bake the gates into `scripts` (`build` = typecheck + build; `audit` = batch script). `latest` tags are FORBIDDEN (kit gap, VERIFIED: two `latest` pins broke reproducibility).

---

## 9. Compliance Map (where each law lives)

| Law                                                                                    | Lives in                                                                      |
| -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Claim labels, dating, no-batch, correction-not-edit                                    | `RULES.md` §0 (from AGENTS.md §7)                                             |
| Placement, single-source contracts, no secrets, `.gitkeep`                             | `RULES.md` §1                                                                 |
| Frontend MUSTs (native HTML, names, focus, contrast, motion, tokens, themes)           | `RULES.md` §2 + `frontend-dev/SKILL.md` + `frontend-design-language/SKILL.md` |
| Token tiers, layers, logical props, no resets, fallbacks                               | `RULES.md` §3 + `frontend-design-language/tokens/tokens.css`                  |
| Edge validation, error envelope, idempotency, pagination, versioning, middleware order | `RULES.md` §4 + `middleware/SKILL.md` + `backend-systems/SKILL.md`            |
| Migrations, dual-compat, no hand-edits, indexing                                       | `RULES.md` §5 + `database-data/SKILL.md`                                      |
| Regression-first, error/empty coverage, checker output                                 | `RULES.md` §6 + `testing-qa/SKILL.md`                                         |
| Secrets, KDF, rotation, cookies, OWASP                                                 | `RULES.md` §7 + `security-auth/SKILL.md`                                      |
| Dated docs, failures recorded, five-file set, Mermaid                                  | `RULES.md` §8 + `docs/`                                                       |
| Skill-file skeleton (§§1–9), flat-tier exception                                       | This file §3 → each `SKILL.md`                                                |
| Hook lifecycle, append-store, shared rule table                                        | This file §4 → `hooks/`                                                       |
| Partition + swarm roles, routing, contract gate, loop exit                             | This file §6 → `RULES.md` §§9, 11                                             |
| Dials + brief line                                                                     | This file §7 → `RULES.md` §9                                                  |
| PARTITION law (small whole-slice ownership, siloed swarms required)                    | `../ENGINEERING-GOVERNANCE.md` law 5 → this file §6 → `RULES.md` §§9, 11      |

## 10. Corrections

- 2026-10-30 — PARTITION reading adopted (owner directive): §6 rewritten from disjoint-ownership (SEPARATE) to small whole-slice ownership with vertical-slice rule (§6.3) and re-titled; siloed swarms stated as REQUIREMENT. Prior SEPARATE wording withdrawn in the same breath: it described keeping state apart, not small focused ownership front-to-back. Supersedes: GLOBAL-LOGIC.md §6 (2026-10-30 original), ENGINEERING-GOVERNANCE.md law 5 (SEPARATE). Never rewrite history (RULES.md rule 4).
