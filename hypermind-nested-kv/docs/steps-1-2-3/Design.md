# Design — Steps 1-2-3 (2026-11-01)

Objective: wire HyperMind runtime to SurrealDB, create real graph edges,
replace fake traversal with atom_edge adjacency.

Decision (contract-first, approved by Stewart 2026-11-01):
- Dual-write, not cutover. Memory + JSONL stay canonical; SurrealDB shadows.
  `HYPERMIND_SURREAL=off|shadow|on` (default shadow). Cutover flag exists,
  canonical switch only after parity passes (not yet claimed).
- Zero new dependencies. `src/surreal.rs` is sync std-only HTTP/1.1 POST to
  `/sql` (TcpStream, no tokio/reqwest/surrealdb crate).
- Explicit edges only. `map.link` admits next|references|part_of|derived_from
  (derived_from requires evidence_atoms). causal|temporal|supports|contradicts
  are FORBIDDEN to fabricate (schema-error); semantic only via the gated
  `map.link_semantic` (similarity AND evidence gates, else fail-explained).
- Traverse v2 is additive: adjacency-first via (1-hop common neighbor, then
  direct neighbor), bounded BFS path, else the legacy midpoint scan explicitly
  labeled `geometric-fallback-*`. No behavior removed.

Files: `src/surreal.rs` (new, transport), `src/main.rs` (Store on State,
6 new tools, dual-write hook, traverse via_mode), `src/main.rs` tests (+9).
Schema `0001_hypermind_graph.surql` and `migrate_snapshot.rs` untouched.
