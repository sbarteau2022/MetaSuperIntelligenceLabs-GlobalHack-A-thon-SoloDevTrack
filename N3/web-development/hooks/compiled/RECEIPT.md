# RECEIPT — Agent 3 (Hooks-A), Wave 1

Date: 2026-10-31. Slice (disjoint): `pre_tool_guard.py`, `session_init.py` only.
Agent 4 files (`post_edit_verify.py`, `stop_completion_gate.py`) NOT touched.
Read-only refs (NOT emitted): `scripts/_engine.py`, `hooks.json`, `HOOK_EVENT_SCHEMA.md`, `rules-table.json`.

## 1. Hashes

| Artifact | SHA-256 |
|---|---|
| Original `pre_tool_guard.py` (pre-migration) | `cbbc7f68395a2948a0fc5b3c99cc904832952594fb5b3b17b13104eb64faacd1` |
| Original `session_init.py` (pre-migration) | `4bd70854ccf20eb1eeee345086ac2d6911087b925bac242c1a08614d2706a7cd` |
| Ported `pre_tool_guard.rs` | `a8b37cae82101d5f5a2e35a927a18bbe29dbac41b38d1268f520f302deeed21f` |
| Ported `session_init.rs` | `c9c46a993b97598e830c13bfe192259e824a17b7edfa950c584d5e5283f313ff` |

## 2. Tool-write chain booleans (all must be true — all ARE true)

| Gate | pre_tool_guard | session_init | Evidence |
|---|---|---|---|
| `ts_check` strict (TS 7.1beta subset shim) | true | true | `ts_check ok:true`, exit 0, zero diagnostics, 2026-10-31 |
| `ts_to_rust` supported | true* | true* | `ok:true` both; *one disclosed unsupported item each: "object literal (no target struct known in expression)" — the two struct-return expressions were completed by hand in Rust (explicit, not silent) |
| Rust compile exit 0 | true | true | `rustc --edition 2021 --crate-type lib --deny warnings` exit 0 both; `cargo check --all-targets` exit 0; `cargo clippy --all-targets -- -D warnings` exit 0 |
| rust-analyzer zero diagnostics | true | true | `rust-analyzer diagnostics` scan complete, exit 0; zero code diagnostics (only expected `inactive-code` WeakWarnings on `#[cfg(test)]` modules, which are active under `cargo test`: 7/7 pass) |

`sent`: true (all three per-hook booleans true).

## 3. Dry-run log (compiled Rust vs Python oracle, same events)

| Case | Python oracle | Compiled Rust | Match |
|---|---|---|---|
| allow clean `src/clean.ts` | `allow`, rule null | `allow`, rule null | YES |
| deny `eval(` in `src/app.ts` | `deny no-eval` + JSON.parse fix | `deny no-eval` + JSON.parse fix | YES |
| deny `AKIAIOSFODNN7EXAMPLE` in `src/keys.ts` | `deny no-secrets-aws-key` + rotate fix | `deny no-secrets-aws-key` + rotate fix | YES |
| fail-closed, no content | `deny schema-error` | `deny schema-error` | YES |
| fail-closed, no path | `deny schema-error` | `deny schema-error` | YES |
| session with package.json (react/vitest, 5 gates) | system_message + audit hint, deps `[react vitest]`, gates `[build test lint typecheck audit]` | identical | YES |
| session empty workspace | system_message, `detected {}` | system_message, `package_json:false, deps:[], gates:[]` (typed zero-value of `{}`) | YES (semantic) |

Note: `report_reason` quote style differs only (Rust `{:?}` double quotes vs Python `repr` single quotes); matched text, rule_id, fix identical. Format-only, INFERRED immaterial.
Unit tests in-port: 7/7 pass (`cargo test` exit 0).

## 4. Proofs

- (a) No out-of-root writes: `pre_tool_guard` is pure (no fs calls at all); `session_init` only calls `fs::read_to_string(<root>/package.json)`. Tested against read-only rooted temp dir (`chmod 555`): full detection worked, `find` showed only the pre-existing `package.json`, zero `.agent-state` dirs created. TRUE.
- (b) No unlisted network: `grep -nEi socket|TcpStream|fetch\(|reqwest|hyper|std::net|Command::` over both ports → only doc-comment hits; no socket/fetch/import-net in code. `strings` over built rlib → no URLs/sockets. TRUE.
- (c) Unload restores old version: pre-migration `.py` hashes recorded above; byte-identical stash at `/tmp/hooks_a3/stash/` (`sha256sum` matches). Restore path = copy stash back to `skills/web-development/hooks/scripts/`; compiled N3 files coexist without overwriting originals. TRUE.
- (d) Hook lists callable schemas: `hook.json` event map — `pre_write → pre_tool_guard(PreWriteEvent)→PreVerdict`, `session_start → session_init(SessionEvent)→SessionVerdict`. All listed functions non-empty and unit-tested. No listed-but-empty entries in this slice; nothing REMOVED (post_write/stop belong to Agent 4). TRUE.

## 5. Destinations

- N3: `/Users/stewartbarteau/Desktop/N3/web-development/hooks/compiled/pre_tool_guard.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/hooks/compiled/session_init.rs`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/hooks/compiled/hook.json`
- N3: `/Users/stewartbarteau/Desktop/N3/web-development/hooks/compiled/RECEIPT.md` (this file)
- Originals: `skills/web-development/hooks/scripts/pre_tool_guard.py`, `skills/web-development/hooks/scripts/session_init.py` — DELETED post-receipt (see §6).

## 6. Migration

All four gates true per hook → MIGRATED (replace, no duplicates).
`hooks.json` command entries still reference the `.py` paths: harness rewiring to the compiled functions is OUT OF SCOPE for this slice (no harness contract for compiled-Rust invocation exists on disk; inventing one would violate RULES.md rule 75). Recorded as follow-up, NOT as done.
NOT checked: `discipline.py`/`audit.sh` re-run after deletion (owner/critic gate); live-harness invocation of the `.rs` functions; Agent 4 slice status.
