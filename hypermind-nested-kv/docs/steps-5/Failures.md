# Failures — Step 5 (2026-11-01)

1. BUG (fixed same session): vector_candidates used `SELECT value {id,sim}
   ORDER BY sim` — silently returned STORAGE ORDER (identical top-k for all
   queries). Root cause: ORDER BY cannot see inside VALUE projections.
   Fix: top-level `SELECT meta::id(id) AS ext, ... AS sim ORDER BY sim DESC`.
   Regression test live_vector_candidates_vary_by_query pins it.
2. BUG (fixed same session): map.query_vec ranked with a fixed pseudo-atom,
   making the "exact rerank" query-blind (same top-3 for all queries). Fix:
   optional qtext -> derived query address (true rerank); without qtext ->
   cosine-ordered with geo attached. Pseudo-atom path removed.
3. BUG (fixed same session): map.embed_status misused one() for a 2-statement
   batch (always 0/0). Fix: counts_embed() with two tolerant singles.
4. HARNESS (fixed same session): loader edit changed range step 50->20 but
   not the slice — overlapping 50-row chunks, garbage counters (loaded_err
   5100 > total). Fix: slice matches step. Lesson: rerun with --limit and
   read the counters before full apply.
5. HARNESS (fixed same session): probe rows with non-numeric backtick ids
   squatted the UNIQUE index, blocking 60 real rows. Fix: deleted 40 probe
   rows, retried 60/60 clean. Lesson: probe ids must not share atom links.
6. ANOMALY: ~1MB server request ceiling (bisected 808KB ok / 1.15MB broken
   pipe). Loader pinned to 20/chunk. Cause unknown (server-side close);
   recorded for step 8 (may interact with write-concurrency tests).
7. TEST INTERFERENCE (fixed same session): two live tests used identical
   one-hot vectors and ran in parallel — cross-talk failure. Fix: orthogonal
   probe positions (7,8 vs 0). Lesson: live-test vectors must be unique per test.
