# SpecOps Research → Qdrant Audit — 2026-11-01

VERIFIED live this session. No inference on counts.

## Disk inventory (VERIFIED via sha256 walk)

- Path: `/Users/stewartbarteau/Desktop/Special Ops/Research`
- Total files: 162 (155 PDFs + 7 md/NOTES + .DS_Store excluded from ingest)
- Total size: 270M (waiver said 169 PDFs / 291M — stale; live is 155 PDFs)
- Filename dups: NONE (only NOTES.md x6, expected per-folder notes)
- Byte dups: NONE (0 groups)
- Per-folder PDFs: kernel-compute 24, kv-batching-mechanics 54, prefix-kv-disaggregation 17, schedulers 14, spec-decoding 27, vault-annex 19

## Qdrant comparison (VERIFIED via scroll + source_path exact match)

- Collection `research_library_v1`: 39,744 points at audit time
- silo=specops points: 3,945 across 155 doc_ids
- doc_id scheme: `<folder>__<stem>` (e.g. `kernel-compute__2024-FlashAttention-3`)
- Vectors per point: text_rank0 1024d + text_rank1 1024d (Qwen3-VL-Embedding-2B-6bit-RUN, rank-rotation-v1, instruction embed-v1, native 2048 → admitted 1024)
- Lexical sparse `lexical` indexed per collection config
- DISK=155, RAG_DOCS=155, MISSING=0, EXTRA=0

## Disposition

- Missing to ingest: 0 — prior `/tmp/ingest_spec.py` run already landed all 155 (3,945 chunks, receipt `/tmp/spec_ingest.log`, DONE in 759s)
- Duplicates to trash: 0 — no intra-folder dups, nothing in RAG but off disk
- Waiver `SPECOPS_WAIVER_2026-11-01.md` findings (169 / 11 dups / 3 present / 166 missing) are STALE — superseded by this audit

## Note on earlier confusion

An early stem-substring match showed 75 present / 80 missing because it compared against non-specops doc_ids and counted generic SPEC buckets as hits. Exact source_path + doc_id reconciliation proves 155/155 present. No ingest ran this session because none was needed.
