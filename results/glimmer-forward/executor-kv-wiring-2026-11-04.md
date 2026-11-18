# NativeGlimmerExecutor exact-K/V wiring — 2026-11-04

`native_glimmer_executor.rs` gains `KvPageStore` + `with_kv_store`. For a prefix-eligible
call (2..=17 tokens) it either hydrates stored pages (hit) or prefills, persists and
continues (miss); all other calls keep the plain forward-exact path. Client wire:
`native_pages_client.rs` (mirrors core_lm `native_pages_wire.rs`).

Gate: `Ares WebSite : App/backend/tests/native_glimmer_kv_live.rs` (ignored). Isolated
Metal serve_glimmer on 18111, real globalmind-mcp child, disposable SurrealKV restarted
between calls, real globalmind store adapter (dev-dependency only).

PASS: plain forward-exact, kv-miss and post-restart kv-hit calls all gave
ids=[13796, 26, 1573, 373, 290, 2167, 26899, 1212]
logits=149ea41c029a3061780e760c3ffc1cbdc2078f68a20123b8a98c1e89ff7a6ead
kv=55a03fddcf8bd197a4c5e94f769a2d04e40475e145658aacc48ae327845ba9c9; receipts validate.
miss tools: mind.recall, native.tokenize, native.kv-prefill, native.continue-exact
hit tools:  mind.recall, native.tokenize, native.kv-hit,     native.continue-exact
A changed TPPE contract made the call fail closed (no recompute fallback).

Limits: the page-digest roster (needed to build an ExactKvIdentity) is carried by the
test caller across the restart, not recovered from the DB; production has no store
implementation or roster source yet and the backend does not depend on the runtime;
dispatch_mission_live does not attach a store; MCP kv_exact not exercised; digest
equality on the server's own forward, not an independent oracle; hand-made witness atom.
