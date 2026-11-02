# sqlite / hooks

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/middleware/languages/sqlite/`.

- Gate: `sqlite3` + `PRAGMA integrity_check` | Judge: E | This machine 2026-10-30: INSTALLED 3.54
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
(none — no `rules-languages.json` rule scopes this language's extensions)
