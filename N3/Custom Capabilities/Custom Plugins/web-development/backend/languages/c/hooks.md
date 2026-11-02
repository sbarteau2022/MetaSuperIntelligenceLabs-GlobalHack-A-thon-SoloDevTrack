# c / hooks

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/c/`.

- Gate: `clang -std=c17 -Wall` + `clang-tidy` | Judge: E/D | This machine 2026-10-30: clang 21 INSTALLED; `c-malloc-check` = D-warn
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `c-malloc-check` (warn, scope **/*.c, **/*.h): malloc without a visible NULL check. Fix: Check for NULL immediately, or use a wrapper that aborts cleanly.
