# MCP to harness automaton boundary receipt — 2026-11-03

Source changes: backend `harness/mcp.rs`, `governor.rs`, `http.rs`, `harness/automaton.rs`; GlobalMind `store.rs`, `bin/globalmind_mcp.rs`. All source lives in the canonical SoloDevTrack repository. No persistent `hypermind/core` write was made.

The previous live route spawned a backend-owned `ares-mcp-mind` child and passed governance authority (`ares/ares`) as both governance and mind authority. It therefore could not prove that retrieval reached the one GlobalMind. The route now requires explicit `ARES_MCP_MIND_URL/NS/DB`, refuses `core`, pins request `map_identity` to `URL|NS|DB`, spawns `globalmind-mcp`, and checks the child's live `mind.db_status` (mode, contract, URL, NS, DB, receipt) before automaton-gated dispatch. A missing governance store also refuses. The role must be explicit. Only L0 can invoke recall/exact K/V until worker scoped handles exist.

The response receipt is a transport binding, not an independent answer judge. This route now passes `verified=false` to the automaton, so a well-formed response cannot raise confidence. The governor persists the transition receipt in governance SurrealDB and sends observation to HyperDX. Atom and tensor K/V reads remain in the separate mind database.

Observed gates:

- Backend `CARGO_TARGET_DIR=/tmp/ares-backend-target cargo test --all-targets --quiet`: PASS, including 27 library tests; live tests remained ignored.
- Backend `CARGO_TARGET_DIR=/tmp/ares-backend-target cargo clippy --all-targets -- -D warnings`: PASS.
- Backend `cargo fmt --check`: PASS after formatting the existing carved swarm test.
- GlobalMind `cargo test --locked --test surreal_integration isolated_mcp_child_attests_recall_and_exact_kv_contract -- --nocapture`: PASS against a disposable SurrealKV process and the real stdio child; child reported `On`, correct authority and `contract=verified`.
- GlobalMind `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: PASS after local lint/format corrections.

The older persistent `hypermind/globalmind_mcp_test` fixture failed a stronger status assertion because its atom-address schema is stale; it was not migrated. The new disposable fixture applies the current schema chain and proves the current read contract. There is no claim of a live `hypermind/core` child, a full Glimmer forward, scoped worker retrieval, coherent answer improvement, or benchmark readiness.
