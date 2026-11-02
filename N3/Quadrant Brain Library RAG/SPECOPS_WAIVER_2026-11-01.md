# WAIVER — SPECOPS Research Inventory → RAG 2026-11-01

Date: 2026-11-01
Owner: Stewart Barteau
Waiver-ID: WAIVE-2026-11-01-SPECOPS-001
Scope: Inventory ~/Desktop/Special Ops/Research (169 PDFs, 177 files, 291M) vs Qdrant BrainLibrary research_library_v1 (35,799 points). Missing → ingest, duplicates → trash.

## Declaration
Inventory 6 subfolders: kernel-compute 23 PDFs (37M), kv-batching 52 PDFs (121M), prefix-kv 23 PDFs (47M), schedulers 19 PDFs (29M), spec-decoding 26 PDFs (29M), vault-annex 26 PDFs (28M). RAG has 1,987 doc_ids from Ai Research Vault + AIE (5,249). SpecOps is new, not in RAG (0 Special Ops source_path prior).

## Retrieve
- Source: ~/Desktop/Special Ops/Research (169 PDFs)
- RAG: Qdrant 127.0.0.1:6333 research_library_v1 (35,799 points, verified 2026-11-01)
- Inventory script: /tmp/inv2.py (filename + file hash vs RAG doc_id/source_path)

## Forbid
- No rm -rf. Duplicates → /usr/bin/trash to ~/.Trash (recoverable). Missing → ingest via Qwen3-VL-Embedding-2B-6bit-RUN (1024d text_rank0/1, same as library/AIE).
- No ingest of duplicates — dedup first.

## Gate
- Inventory pasted (169, 6 filename dups, 11 bytes dups, 3 present, 166 missing → 155 unique after dedup)
- Each trash exit 0 + sha256, each ingest batch receipt pasted
- Post: RAG count + duplicate Trash verification

## Partition
Single mind front-to-back. No swarm.

## Findings (VERIFIED)
- Found 169 PDFs, 6 filename dups (12 files), 11 exact-bytes dups (22 files, includes 6 above + 5: Orca, PagedAttention, Hydragen, SGLang, Sarathi-Serve)
- Present in RAG (by stem match): 3 (vault-annex 2023-12__2312.06635, MoE-SpeQ, Bole) — duplicates vs RAG
- Missing from RAG: 166 — after removing 11 intra-specops extra copies, 155 unique missing to ingest
- Total dup files to trash: 14 (11 intra extra + 3 RAG present)

## Disposition
| # | Action | Count | Paths |
|---|--------|-------|-------|
| 1 | Trash intra-specops extra copy | 11 | keep kv-batching-mechanics/, trash schedulers/ and prefix-kv-disaggregation/ second copy (and 5 cross-named) |
| 2 | Trash RAG present (already in BrainLibrary) | 3 | vault-annex 3 listed above |
| 3 | Ingest missing unique | 155 | 166 missing - 11 trashed = 155 (spec-decoding 26, kernel 23, kv-batching 41, prefix-kv 15, schedulers 12, vault-annex 23, etc) |

Recovery: Finder Put Back from ~/.Trash, or /tmp/specops_backup_2026-11-01/

## Waive-or-Obey
Dated waiver before work. Trash archives stay until owner reviews per-item.
