# tauri / hooks

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/middleware/languages/tauri/`.

- Gate: `cargo clippy -- -D warnings` + `cargo tauri build` | Judge: E | This machine 2026-10-30: cargo INSTALLED (1.99.0); tauri-cli NEEDS-VERIFY
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
(none — no `rules-languages.json` rule scopes this language's extensions)
