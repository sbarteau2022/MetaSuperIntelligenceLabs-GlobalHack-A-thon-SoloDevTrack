# MCP single-mind read-only preflight receipt

The harness bridge rejects an MCP declaration that differs in SurrealDB URL,
namespace, or database, uses shadow mode, or names quarantined `core`.
Passing config then connects to the same isolated SurrealKV authority.

`cargo test --test globalmind_bridge_live -- --ignored --nocapture`: 1 passed
on 2026-11-03. The test applied the existing schema to a disposable local
SurrealKV, inserted one real atom/address and one exact K/V page, read GMRC
binary recall and exact K/V, and rejected three authority mutations first.
No `hypermind/core` access or `.hypermind` creation occurred. This proves
declared-target parity and bridge operation, not live MCP process wiring or
Glimmer model execution.
