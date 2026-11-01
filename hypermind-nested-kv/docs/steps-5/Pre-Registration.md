# Pre-Registration — Step 5 (written BEFORE bulk load, 2026-11-01)

Predictions:
1. Embedder smoke: 1024D, unit-norm, identical texts cosine 1.0, byte-identical rerun.
2. Bulk: 2,079 vectors, digest(source) == atom content_digest for all rows.
3. ORDER BY sim works (top-level projection) — regression test enforces it.
4. Battery N=2,079: self-search returns self first; distinct queries give
   distinct tops; vector_bench oracle tops start with the query atom.
5. Core after step 5: 2,079 atoms / 2,079 addresses / 2,079 text vectors / 0 edges.
6. 65+ unit tests pass; clippy clean. No BGE download leaves this machine.

Judges: embedder stdout, /sql counts, digest-agreement script, battery output,
cargo test, cargo clippy.
