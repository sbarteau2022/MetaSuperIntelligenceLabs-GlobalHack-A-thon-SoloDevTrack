# Scoped Surreal recall receipt, 2026-11-03

Implemented read-only cognitive-well membership filtering and internal edge projection in `globalmind-runtime/src/retrieval/scoped.rs`. The isolated SurrealKV fixture uses the repository's schema 0001 and 0007, persists two well members plus one internal and one cross-scope edge, and proves the cross-scope edge and non-member atoms are absent from returned data. It proves deterministic scope digest and fail-closed empty scope/budget behavior. No live GlobalMind database was mutated.

This is a component integration check, not forward-pass certification. The MCP bridge target check, governor permit authorization, Glimmer weight owner, and live worker path are not yet wired into this function. Existing retrieval still scans the workspace up to candidate budget; this seam does not establish production latency or large-map scale.

Gates: `cargo check --tests` PASS; `cargo test --lib` 80 passed, 3 ignored; focused isolated-Surreal test PASS; `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` run after final edit.
