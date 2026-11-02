# rust / hooks

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/rust/`.

- Gate: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | Judge: E | This machine 2026-10-30: INSTALLED 1.99.0
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `rs-unwrap` (deny, scope **/*.rs): unwrap() in non-test Rust panics on the unhappy path. Fix: Return Result with ? or map the error; expect("why") only for proven invariants.
- `rs-expect` (warn, scope **/*.rs): expect() needs a proven invariant behind it. Fix: State the invariant in the message or restructure to Result.
- `rs-panic-expected` (warn, scope **/*.rs): panic! for an expected failure crashes the process. Fix: Return Result; reserve panic! for unrecoverable bugs.
