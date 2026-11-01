# Successes — Step 5 (2026-11-01)

1. Pinned BGE-M3 pull (rev 5617a9f, 4.3G, local-only) + frozen embedder
   (tools/embed_bgem3.py) + bulk loader (tools/load_text_embeddings.py).
2. Schema 0002 applied: table + ASSERTs + UNIQUE + HNSW all OK.
3. 2,079 text vectors, coverage 1.0 (map.embed_status), migration recorded.
4. Transport: upsert_text_embedding, vector_candidates (fixed projection).
5. Tools: map.embed_status (fixed counts_embed), map.query_vec (+qtext),
   map.vector_bench. 76 tests pass (65+3+8), clippy clean.
6. Live triple-probe tests (roundtrip + vary-by-query) green, no residue.
7. Core: 2,079 / 2,079 / 2,079 vectors / 0 edges. Scratch namespaces dropped.
