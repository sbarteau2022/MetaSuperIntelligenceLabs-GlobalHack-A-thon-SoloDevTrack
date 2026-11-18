# rust / rules — compiled predicates

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/rust/`.

Canonical prose source: `skills.md` (`## Rules`, carried from `N3/languages/rust.md`). This file is the compiled predicate form (one row per rule: id, scope, severity, pattern, consequence, judge). RETRIEVE: one canonical source per decision — prose lives in `skills.md`, enforceable patterns live here and in `web-development/hooks/rules-languages.json`.

Severity: `deny` = binding (write blocked, merge blocked). `warn` = advisory (review flag, stop-gate note).

## Predicates

| id | scope | severity | pattern | consequence on violation | judge (executable gate) |
|----|-------|----------|---------|--------------------------|-------------------------|
| rs-unwrap | `**/*.rs` | deny | `\.unwrap\(\)` | write blocked; merge blocked until `Result`+`?` or proven-invariant `expect("why")` | `pre_tool_guard.py` deny gate + `cargo clippy --all-targets -- -D warnings` |
| rs-expect | `**/*.rs` | warn | `\.expect\(` | review flag; stop-gate note until invariant stated in message or restructured to `Result` | `post_edit_verify.py` warn gate + `cargo clippy --all-targets -- -D warnings` |
| rs-panic-expected | `**/*.rs` | warn | `\bpanic!\s*\(` | review flag; stop-gate note until `Result` | `post_edit_verify.py` warn gate + `cargo test` |
| rs-unsafe-safety | `**/*.rs` | deny | `\bunsafe\b` | merge blocked without `// SAFETY:` invariant comment and test | `grep SAFETY` + `cargo miri test` |
| rs-borrow-arg | `**/*.rs` | warn | `&String\|&Vec<` | review flag until borrow-in-own-out (`&str`, `&[T]`, `impl AsRef<Path>`) | `cargo clippy --all-targets -- -D warnings` (`ptr_arg`) |
| rs-narrow-cast | `**/*.rs` | warn | `\bas\b` | review flag until `try_from` or `checked_*`/`saturating_*` | `cargo clippy --all-targets -- -D warnings` (cast lints) |

Notes:
- `rs-unwrap` / `rs-expect` / `rs-panic-expected` mirror `rules-languages.json` rows exactly (same id, scope, severity, pattern). Tests (`#[cfg(test)]`, `tests/`) exempt `rs-unwrap` by convention; judge enforces.
- `rs-unsafe-safety` compiles `skills.md` Rule 7 (smallest scope + `SAFETY:` + test/miri). Deny because unsound `unsafe` is a soundness hole, not style.
- `rs-borrow-arg` compiles `skills.md` Rule 1. `rs-narrow-cast` compiles `skills.md` Rule 5. Both warn: clippy flags them, human confirms intent.
- `skills.md` Rules 2, 4, 6, 8, 9, 10, 11 have no single-regex predicate; their judges are `cargo clippy -D warnings` / `cargo test` / `cargo fmt --check` / `cargo audit` directly (see `hooks.md` gate). Per spec they are not listed here as patterns — no judge-able pattern, no row (cut, recorded below).

## Contradiction check (2026-10-30)

Merged union = `rules-languages.json` + `rules-table.json`. Sibling = `skills.md` Rules 1–11.

| predicate × existing | conflict? | reason |
|----------------------|-----------|--------|
| rs-unwrap × rs-unwrap (json) | no | identical id/scope/severity/pattern; this row is the compiled mirror |
| rs-unwrap × rs-expect (json) | no | disjoint patterns (`unwrap()` vs `expect(`); different severity by design (deny vs warn) |
| rs-unwrap × rules-table.json rows | no | no `**/*.rs` scope in rules-table except `**` globals (`no-secrets-aws-key`, `no-private-key`); complementary |
| rs-expect × rs-expect (json) | no | identical mirror |
| rs-expect × skills.md Rule 3 | no | same norm ("proven invariant + why message"); predicate is the enforceable subset |
| rs-panic-expected × rs-panic-expected (json) | no | identical mirror |
| rs-panic-expected × skills.md Rule 3 | no | same norm ("no panic! for expected failures"); predicate is the enforceable subset |
| rs-unsafe-safety × any json row | no | no existing `unsafe` row in either json; complementary, fills Rule 7 gap |
| rs-borrow-arg × any json row | no | no existing borrow-arg row; complementary, fills Rule 1 gap |
| rs-narrow-cast × any json row | no | no existing cast row; complementary, fills Rule 5 gap |
| rs-borrow-arg × skills.md Rule 1 | no | same norm; predicate narrows to the two greppable anti-patterns |
| rs-narrow-cast × skills.md Rule 5 | no | same norm; `as` flagged warn-only because widening `as` is legitimate — human confirms |

Resolutions: none required (zero conflicts, 2026-10-30). Rule on future conflict: binding (deny) wins over advisory (warn); recorded here when triggered.

## Cuts (no judge = cut)

2026-10-30: 0 cuts among listed predicates — all six carry an executable judge. skills.md Rules 2/4/6/8/9/10/11 deliberately have no pattern row (enforced by the hooks.md gate commands directly, not by a regex predicate); recorded, not silently dropped.

## Chain receipt (tool-write)

- `tsc --noEmit --strict` (TS 7.1.0-beta, local toolchain): exit 0 on the 6-predicate TS subset source (class `RulePredicate`, typed factories, `auditRust`). No object literals.
- `rustc --crate-type lib -D warnings` (rustc 1.99.0): exit 0 on the ported Rust (`enum Severity`, `struct RulePredicate`, `audit_rust`).
- `ts_check` / `ts_to_rust` MCP tools: could not execute — configured compiler path missing in this environment (`.../Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc` does not exist; even a trivial `add(a,b)` probe returned `ok:false` with empty diagnostics). Chain PROVEN via the real local toolchain above; harness failure recorded, not hidden.
- Versions: tsc 7.1.0-beta, tsrs 7.1.0-beta, rustc 1.99.0, cargo 1.99.0, clippy 0.1.99, rustfmt 1.10.0 (this machine, 2026-10-30).
- sent: true (all three booleans true on the local toolchain: ts_strict=true, rust_port_supported=true, rust_compile=true).
