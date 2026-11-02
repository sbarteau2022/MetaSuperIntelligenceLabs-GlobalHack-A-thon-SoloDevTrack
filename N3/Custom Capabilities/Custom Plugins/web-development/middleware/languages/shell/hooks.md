# shell / hooks

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/middleware/languages/shell/`.

- Gate: `shellcheck -S error` + `sh -n` | Judge: E/D | This machine 2026-10-30: shellcheck/shfmt MISSING; `sh-unquoted-var` = D-warn
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `sh-unquoted-var` (warn, scope **/*.sh): Unquoted variable expansion splits on spaces and globs. Fix: Quote: echo "$VAR". Prefer shellcheck -S error in CI.
