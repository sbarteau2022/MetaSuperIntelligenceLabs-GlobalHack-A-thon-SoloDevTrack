# RULES.md — Binding Rules for This Scaffold

Status: PROPOSED (ratified by owner on use). Date: 2026-10-30.
Scope: everything under `skills/web-development/`.

Rules are labeled **MUST**, **SHOULD**, or **MAY**. MUST violations block review.

---

## 0. Global Constraints (from AGENTS.md, inherited)

1. MUST label every claim VERIFIED / REPORTED / INFERRED / PROPOSED.
2. MUST date every document and name exact paths and versions.
3. MUST NOT report performance numbers without hardware, settings, workload, and baseline.
4. MUST NOT silently rewrite a wrong claim — append a dated correction entry.
5. MUST NOT bulk-delete, bulk-rewrite, or batch-claim. Per-item disposition only.

---

## 1. Structure & Placement

6. MUST place code in the matching subsystem folder; if it fits nowhere, create the folder and update `README.md` in the same change.
7. MUST keep shared contracts (API shapes, data models, design tokens) in ONE canonical location and link to it from consumers. Never duplicate a schema.
8. MUST NOT put source code in `docs/`, `templates/`, or `frontend-design-language/references/`. These are reference-only.
9. MUST NOT commit secrets, `.env` files, keys, or tokens. `.env.example` only.
10. MUST keep every directory non-empty in git via `.gitkeep` where no file exists yet.

## 2. Frontend

11. MUST use native HTML elements before ARIA. No `<div role="button">`; use `<button>`.
12. MUST NOT add redundant ARIA (`<ul role="list">`, `role="button"` on `<button>`).
13. MUST use one `<h1>` per page, sequential heading levels, no skipped levels.
14. MUST NOT use `<div>`/`<span>` styled to look like headings.
15. MUST use real `<table>` + `<th scope>` for tabular data; MUST NOT use tables for layout.
16. MUST provide a skip link to a focusable target (`tabindex="-1"`) on pages with repeated nav.
17. MUST label every interactive element with an accessible name (`<label for>`, `aria-label`, `aria-labelledby`).
18. MUST NOT apply `aria-hidden="true"` to focusable elements or their ancestors.
19. MUST respect `prefers-reduced-motion` for all non-essential animation.
20. MUST NOT style with inline nontrivial values — use design tokens from `frontend-design-language/tokens/`.
21. MUST keep `color` contrast ≥ 4.5:1 for body text and ≥ 3:1 for large text and UI boundaries.
22. MUST size interactive targets to at least 24×24 CSS px (44×44 preferred on touch).
23. MUST support both light and dark themes unless a product decision is recorded.
24. MUST NOT use `!important` outside the `utilities` cascade layer.

## 3. Design Language & Tokens

25. MUST use the 4-tier token model: literal → semantic → UI → component.
26. MUST reference semantic tokens in component code. Literal tokens only inside the token file.
27. MUST NOT hardcode colors, spacing, radii, or durations in component styles.
28. MUST define layer order explicitly: `@layer reset, base, theme, components, utilities;`
29. MUST prefer logical properties (`margin-inline-start`) over physical (`margin-left`).
30. MUST use `currentColor` over a color variable when it expresses the same intent.
31. MUST NOT introduce BEM-style naming to manage specificity — use cascade layers and `:where()`.
32. MUST NOT use global resets (`* { ... }`).
33. MUST use `:not()` to express exclusion instead of override-then-restore, where the intent is exclusion.
34. MUST prefer `@scope` over `:not()` for excluding deeply nested subtrees.
35. MUST feature-detect newly-available CSS before making it load-bearing; provide a fallback.

## 4. Backend, Middleware & API

36. MUST validate all input at the edge. Never trust a client.
37. MUST return a consistent error envelope across every endpoint.
38. MUST NOT leak stack traces, SQL, or internal paths in production error responses.
39. MUST authorize every request server-side. UI hiding is not authorization.
40. MUST make every state-changing endpoint idempotent or explicitly guarded against replay.
41. MUST paginate any list endpoint that can grow without bound.
42. MUST version the API contract (`/v1`) and record breaking changes in `api-integrations/docs/`.
43. MUST make middleware order explicit and documented in `middleware/README.md`.
44. MUST NOT log secrets, session tokens, or full card numbers.

## 5. Data

45. MUST version every schema change as a forward-only migration with a documented rollback.
46. MUST write migrations that are safe to run while the old code is still serving traffic.
47. MUST NOT hand-edit production rows; use a reviewed script under `database-data/`.
48. MUST index every field used for filtering, sorting, or joining on hot paths.

## 6. Testing & Quality

49. MUST have a failing test that reproduces a bug before the fix (regression guard).
50. MUST cover error and empty states, not only the happy path.
51. MUST NOT claim a test suite passed without showing the checker's output.
52. SHOULD keep unit tests free of network and filesystem access.

## 7. Security

53. MUST store secrets in a secret manager or `.env` outside version control.
54. MUST hash passwords with a memory-hard KDF (Argon2id / bcrypt) and a per-user salt.
55. MUST rotate session identifiers on privilege change (login, logout, role change).
56. MUST set `HttpOnly`, `Secure`, and `SameSite` on session cookies.
57. MUST run an OWASP Top 10 review before any public release.

## 9. Partition & Swarm Protocol (siloed swarms are a REQUIREMENT)

58. MUST follow the swarm routing law in `GLOBAL-LOGIC.md` §6 for any task spanning >3 files or >1 engine: Explorer (read-only) → Architect (shared contracts on disk) → parallel Coders (disjoint ownership) → Critic (sole verifier with live tools).
59. MUST NOT start parallel coders until shared types/store/API contracts exist on disk and are reviewed.
60. MUST NOT let coders self-approve. Only deterministic gates and the critic pass work.
    60a. MUST partition work into small whole slices, each owned front-to-back by exactly one mind. MUST NOT assign one layer across many minds (one writes all routes, another all styles) — slices are vertical (a feature, a surface, a contract), never horizontal. Siloed swarms are a REQUIREMENT, not a suggestion.
61. MUST declare the one-line calibration before generating: `BRIEF:` + `DESIGN_VARIANCE/MOTION_INTENSITY/VISUAL_DENSITY` (generative work) or `RISK/BLAST-RADIUS/CONTRACT-FIRST` (backend/middleware/data work). See `GLOBAL-LOGIC.md` §7.
62. MUST treat the loop as condition-driven, not sequence-driven: on failure, re-enter at the FAILING layer, never from the top.
63. MUST NOT claim completion while the stop-gate reports pending issues. The gate vetoes "done".

## 10. Documentation

64. MUST date every document and state what was NOT checked.
65. MUST record failures — a recorded failure beats an asserted success.
66. MUST keep the five-file set per workstream: `Design.md`, `Pre-Registration.md`, `CLAIMS_REGISTER.md`, `Successes.md`, `Failures.md`.
67. MUST use Mermaid for flow diagrams in Markdown; no hardcoded hex in diagrams.
68. MUST append dated correction entries; never rewrite a wrong claim in place.

## 11. Discipline Enforcement (how this level of discipline is REQUIRED)

Discipline written down is advisory. These three mechanisms make it required:

69. MUST run the conformance gate before claiming done on any change to skills, tools, hooks, rules, templates, or swarm tasking: `python3 skills/web-development/hooks/discipline.py` MUST exit 0, pasted into the release checklist (RULES.md rule 51). A FAIL blocks completion; a warning MUST be fixed or recorded with a dated reason.
70. MUST run the batch audit in the same change: `sh skills/web-development/hooks/audit.sh skills/web-development` MUST exit 0. Same rules as the live hooks by construction (one table).
71. MUST NOT claim completion while the stop-gate reports pending issues. Wire `hooks/stop_completion_gate.py` to the harness Stop event per `hooks/HOOK_EVENT_SCHEMA.md`; where no harness exists, the release checklist + pasted checker output is the manual equivalent, and skipping it is a RULES violation, not a shortcut.
72. MUST task subagents as siloed whole-slice owners with a single contract gate: Explorer (read-only) → Architect (contracts on disk, reviewed) → parallel Coders (each owns one small partition front-to-back, exclusive subtrees) → Critic/gates pass or fail. Coders MUST NOT self-approve. See `GLOBAL-LOGIC.md` §6.
73. MUST scope every delegated subtask to its partition in the delegation message — what it owns end-to-end, what it MUST NOT touch, and the contract it builds against; MUST NOT issue duplicate delegations for the same unresolved subproblem.
74. MUST verify agent-reported results independently for structural claims (re-run the gate, re-check the skeleton order) before recording them as VERIFIED. Agent output is REPORTED until independently checked.
75. MUST NOT invent scripts, hooks, or gates that do not exist on disk. §6 of every SKILL.md references only `skills/web-development/hooks/audit.sh`, `discipline.py`, and `rules-table.json`.

---

## Enforcement Checklist (copy into PR description)

- [ ] No secrets or `.env` committed
- [ ] Inputs validated at the edge; errors use the standard envelope
- [ ] Every new UI surface checked for a11y (name, focus, contrast, motion)
- [ ] All styling values come from tokens; no new literals in components
- [ ] Tests cover happy path + error + empty states
- [ ] Calibration line declared (dials or risk triple)
- [ ] Swarm contract gate satisfied (if parallel work)
- [ ] Batch audit exit 0; stop-gate has no veto
- [ ] Docs updated and dated; claims labeled
