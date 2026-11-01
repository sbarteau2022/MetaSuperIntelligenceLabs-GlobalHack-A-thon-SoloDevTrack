# Claims Register — Step 6 (2026-11-01)

- VERIFIED: batched `WHERE in IN [...]` adjacency works (2 rows, scratch probe).
- VERIFIED: seeded e2e — snapshot 9 nodes/10 links/5 kinds; references filter
  2 links; path 1->10 = 3 hops [1,8,9,10]; 1->7 resolves via 4-hop chain
  (valid, not an error); export HTML written, sha returned.
- VERIFIED: export geometry — 9 titled nodes + ball, 10 kind-colored edges,
  all nodes inside r225 disk, 104px spread (cluster expected: near-identical
  test texts project adjacently; positions distinct, no collapse).
- VERIFIED: 38 tools listed; 78 tests pass (67+3+8); clippy Finished, no errors.
- VERIFIED: scratch namespaces removed; core 2,079/2,079/2,079/0.
- INFERRED: snapshot BFS fans out per-node neighbors() (N+1 round-trips);
  subgraph_edges() exists for batching — optimization follow-up, not a defect.
