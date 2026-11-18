# Knowledge-Base RAG → Qdrant Ingest Receipt — 2026-11-02

## Source
- Directory: `~/Desktop/N3/AI Research For<ingested in Qdrant>/knowledge-base-rag/pdfs/`
- PDFs ingested: 233 (stems are arXiv IDs, e.g. 2610.11324v1)
- Ignored: 2 txt files in `pdfs/.trash/` (not PDFs, not ingested)
- Manifest: `knowledge-base-rag/manifest.json` (252 entries; 19 had no PDF on disk)

## Method (adapted from proven /tmp/ingest_spec.py SpecOps run)
- Embedding model: Qwen3-VL-Embedding-2B-6bit-RUN, instruction embed-v1, model config digest 1f16082110c786f38704d43a7a5443ca0a94c2178bca3e1d7fd4370f14cfce45
- Extractor: pypdf (mlx-embeddings venv; pdfminer unavailable in that venv)
- Chunking: CAP 3200 chars, same markdown/window chunking as spec ingest
- Vectors: text_rank0 + text_rank1, 1024d Cosine, rank-rotation-v1
- Qdrant: Docker hypermind-qdrant, REST http://127.0.0.1:6333, collection research_library_v1
- Payload: silo="knowledge-base", document_id="knowledge-base__<stem>", source_type pdf, plus arxiv_id (version stripped) and published_year/section from manifest where available
- Upserts: batch 50, wait=true
- Runtime: 1,444s (~24 min), job pid 49857, log /tmp/kb_ingest.log

## Verification (all passed before any deletion)
- Chunks upserted: 8,678
- Qdrant count, filter silo=knowledge-base, exact: 8,678
- Distinct document_ids for silo knowledge-base (full scroll): 233
- Spot checks by arxiv_id: 2610.11324 → knowledge-base__2610.11324v1; 2610.40586 → knowledge-base__2610.40586v1; 2412.15444 → knowledge-base__2412.15444v1
- Total points before: 39,744; after: 48,422 (39,744 + 8,678)
- Pre-ingest state was 0 matches for these PDFs in Qdrant, so no duplicates were created by this run

## Deletion (after verification only)
- Moved to Trash (recoverable, NOT permanent delete): `~/.Trash/AI Research For<ingested in Qdrant>/`
- Verified: gone from `~/Desktop/N3/`, present in Trash with all 233 PDFs intact
- Note: source_path fields in the new Qdrant points still point at the old N3 path; that path is now in Trash, so source_path is a historical record, not a live file location
