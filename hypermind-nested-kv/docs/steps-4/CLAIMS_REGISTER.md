# Claims Register — Step 4 (2026-11-01)

- VERIFIED: SurrealDB 3.3.5 here rejects SEARCH and MTREE index syntax
  (parse errors captured 2026-11-01); CONTAINS/CONTAINSANY work (full-scan).
- VERIFIED: neighbors row shape {"id":"atom:101",...}; map.path BFS parses it.
- VERIFIED: live prefilter on core returns atom 1 for "distillation"
  (HYPERMIND_E2E_LIVE=1 test).
- VERIFIED: battery N=300 — VP recall 1.000 (24/24), SDB recall 0.917 mean
  (22/24 at 1.000; 2 misses = deliberate no-match query, 0 candidates,
  correct behavior); oracle 1.9ms / VP 0.59ms / prefilter 2.2ms means.
- VERIFIED: map.query rewire ranking-neutral (mirror test green, 72 total pass).
- VERIFIED: db_status on table-less DB now reports zeros (was error).
- INFERRED: 10K→1M scale behavior unknown — step 8's job. Step-4 numbers are
  N=300 shape only, not a scale claim.
