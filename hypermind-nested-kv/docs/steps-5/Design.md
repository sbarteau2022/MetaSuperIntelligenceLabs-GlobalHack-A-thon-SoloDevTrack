# Design — Step 5 canonical text embedding (2026-11-01)

Spec (Stewart): BGE-M3, 1024D, HNSW cosine, table atom_embedding_text_v1,
exact Rust H3xT2 rerank authoritative. Image/audio admitted later, separately.

Decisions:
- Encoder: BAAI/bge-m3 @ 5617a9f61b028005a4858fdac845db406aefb181 (local-only,
  ~/models/bge-m3-5617a9f). Dense CLS head only, L2-normalized. Sparse/colbert
  heads ignored. Teacher identity never enters the vector (input = atom text).
- Schema 0002: encoder/encoder_revision/dimensions ASSERT-locked; vector
  array<float,1024>; per-(workspace,atom) UNIQUE; HNSW DIMENSION 1024 COSINE.
- Retrieval: SurrealDB cosine candidates -> hydrate from memory (canonical)
  -> rerank. map.query_vec gains optional qtext: WITH qtext the query address
  is derived (same hash-idf-rank math) and rank_candidates is a TRUE geometric
  rerank; WITHOUT qtext results are cosine-ordered with geo scores attached
  (a query-blind pseudo-atom rerank was tried, caught by battery, removed).
- vector_bench.preserved_at_k measures cosine-vs-geometric agreement.
