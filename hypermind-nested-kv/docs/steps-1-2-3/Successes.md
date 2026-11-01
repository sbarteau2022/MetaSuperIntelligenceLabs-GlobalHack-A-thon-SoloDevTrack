# Successes (2026-11-01)

1. `src/surreal.rs` (new): sync std-only store — upsert_atom/address, create_edge (RELATE), neighbors, counts, max_external_id. Zero new deps.
2. Dual-write on map.ingest: memory+JSONL first, shadow SurrealDB, note in receipt (`shadow-ok` / `shadow-skipped-*` / `shadow-off`).
3. Six tools: map.link, map.link_semantic, map.chain_next, map.neighbors, map.path (client BFS ≤6), map.db_status.
4. Traverse v2 via_mode: adjacency-1hop | adjacency-direct | geometric-fallback-* (labeled in every receipt).
5. Collision guard: db_id_floor fetched once per process; ids ≤ floor skip shadow UPSERT.
6. Full e2e in scratch namespace: links, neighbors, 2-hop path, semantic edge (sim 0.82), chain_next 3 edges, parity true. Scratch namespace removed after.
7. Validation: 68 tests pass (57+3+8), clippy clean (no errors).
8. Core DB untouched: 2,079 atoms / 0 edges after all work.
