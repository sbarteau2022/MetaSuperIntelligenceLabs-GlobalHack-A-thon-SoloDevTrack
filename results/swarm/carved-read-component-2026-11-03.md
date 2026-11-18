# Carved graph read component receipt

Scope: backend in-memory carved-map API only. No MCP server, no SurrealDB
worker handle, no model call, and no quality merge are claimed.

The existing planted out-of-view probe now exercises `ScopedGraphRead`:
L1 reads its homed atom payload; non-adjacent L12 reads none; a transition
with an L12 endpoint is refused in L1; both handles borrow the same frozen
weight `Arc`; L0 cannot construct a worker handle.

Verification: `cargo check --tests`, `cargo test --test governance_core`
(12 passed), `cargo test --all-targets` (exit 0),
`cargo clippy --all-targets -- -D warnings` (exit 0), and `rustfmt --check`
on changed Rust files (exit 0). The wider `cargo fmt --check` remains affected
by another agent's untracked `tests/gate12_route.rs` and is not credited here.

Next gate: the MCP adapter must enforce the same membership against the one
SurrealDB GlobalMind, bind map revision/permit to every worker read, and
refuse stale handles after recarve. A real Glimmer worker must then produce
an output with independent judge and adversarial receipts before L0 merge.
