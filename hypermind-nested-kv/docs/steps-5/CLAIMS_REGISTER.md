# Claims Register — Step 5 (2026-11-01)

- VERIFIED: HNSW DEFINE INDEX parses on SurrealDB 3.3.5 (full params in INFO).
  Every query shape probed (function predicate, ORDER BY similarity) EXPLAINs
  to TableScan — index is forward cover, planner doesn't route to it yet.
- VERIFIED: `SELECT value {..} ORDER BY sim` silently no-ops (nested field);
  top-level projection orders correctly. Caught by battery, fixed, regression-tested.
- VERIFIED: BGE-M3 rev 5617a9f61b028005a4858fdac845db406aefb181, 1024D CLS,
  unit-norm, deterministic rerun byte-identical.
- VERIFIED: 2,079/2,079 vectors loaded; digest agreement 2000/2000 sampled
  (full check script ran on LIMIT 2000 sample — rest by count + loader gates).
- VERIFIED: battery N=2,079 — self-search top=self (cos 1.0); 6/6 distinct
  queries distinct tops; prefilter ~110ms, rerank ~45ms, oracle ~90ms.
- VERIFIED: vector_bench oracle cosine tops start with query atom (ordering fix proven).
- FINDING (measured, not a failure): preserved_at_k = 0.0 — cosine top-8 and
  geometric top-8 are disjoint on this corpus. The two spaces disagree; per
  the authority rule the H3xT2 order governs dispatch. Recorded, not hidden.
- VERIFIED: server request ceiling between 808KB and 1.15MB (bisected);
  loader uses 20/chunk (~460KB).
- INFERRED: image/audio encoders, DiskANN migration, HNSW planner routing —
  future work, explicitly out of scope.
