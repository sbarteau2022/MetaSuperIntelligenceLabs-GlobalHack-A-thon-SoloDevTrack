# lua / rules

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/lua/`.

- Normative rules live in `skills.md` (`## Rules` section, carried from `N3/languages/lua.md`). This file is the index, not a second copy (RETRIEVE: one canonical source per decision).
- Banned patterns for this language are enforced by `web-development/hooks/rules-languages.json` (merged union with `rules-table.json`, D8-verified) — see `hooks.md` for the rows scoping `lua`.
- Adding a rule: give it an exact value/pattern, a severity, and a judge (executable gate, pasted-output requirement, or named sign-off). A rule with no judge is cut, not kept.
