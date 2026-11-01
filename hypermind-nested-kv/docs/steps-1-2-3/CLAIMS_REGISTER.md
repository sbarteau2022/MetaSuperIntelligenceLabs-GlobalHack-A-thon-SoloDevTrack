# Claims Register (2026-11-01)

- VERIFIED: runtime is stdio JSON-RPC MCP binary, main.rs 5,197 lines, 42 tests pre-change; persistence = map.snapshot.bin + map.atoms.jsonl (ledger.rs). (read from source)
- VERIFIED: DB hypermind/core held 2,079 atoms / 2,079 addresses / 0 edges before work; all provenance atoma-mind; rank all 0. (queried 2026-11-01)
- VERIFIED: traverse was whole-map O(N) midpoint scan (main.rs:2786 pre-change). (read from source)
- VERIFIED: rank-1 docs require children ids (main.rs:2164-2170) — only explicit inter-atom structure in runtime. (read from source)
- VERIFIED: HTTP transport works (BEGIN/CREATE/CANCEL round-trip, graph traversal `->atom_edge->atom` returns []). (probed 2026-11-01)
- VERIFIED: relation tables require RELATE, not CREATE (got `expected a RELATION` error, switched). (probed 2026-11-01)
- VERIFIED: neighbors row shape `{"id":"atom:101","kind":...}` — BFS parser correct. (probed 2026-11-01)
- INFERRED: rank-1 children post-ingest persistence unknown — backfill covers only docs ingested after this ships unless caller replays children. Stated in approval, not yet resolved.
- PROPOSED: canonical cutover (HYPERMIND_SURREAL=on as default / JSONL read-only) after parity passes. Not done.
