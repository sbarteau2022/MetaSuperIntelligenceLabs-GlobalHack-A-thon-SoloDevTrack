# Server-path exact K/V persistence + restart continuation — 2026-11-04

Test: `globalmind-runtime/tests/glimmer_server_kv_restart.rs` (ignored) against an
isolated Metal `serve_glimmer` on 18111 (frozen Muse Glimmer) and a disposable
SurrealKV (restarted mid-test). New binary routes: `/v1/prefill-exact` (GPF2/GPO2,
every layer/head page of a prefix) and `/v1/continue-exact` (GCX2 -> GFO2); wire in
`core_lm/src/native_pages_wire.rs`. Tokens came from `/v1/tokenize-native`.

Flow: tokenize "The capital of France is" (6 tokens) -> uninterrupted GFX2 run ->
prefill 5 tokens, 104 pages (52x2x128) -> persist via store identities -> restart
Surreal -> hydrate by exact identity (equal to exported pages) -> continue on server.

PASS: resumed output ids [13796, 26, 1573, 373], logits digest
64c290aa76a9a556430218e5f9799054ac5c8d5bb7be7be50ac6e5655f7222e7 and K/V digest
5bf0f22776cc6c6db8c999ec0d0b8452dd9302d70dbac445bc1dbda166d3ae12 all equal the
uninterrupted run; artifact composite dbe78c78…1a7e.

Rejected before bytes reach the server: changed model digest, changed shape, changed
TPPE contract, changed address contract, changed page digest, missing head.
Server controls: a perturbed value changes the continuation digests; pages of the
wrong shape are refused (no prefix recompute).

Limits: digest equality of the final step, not an independent oracle (the MLX oracle
still shows Rust logit divergence); the NativeGlimmerExecutor in the backend is NOT
yet wired to prefill/continue; MCP `mind.kv_exact` was not exercised in this test;
5-token prefix, one run, hand-made witness atom (not distilled knowledge).
