# The Postgres Agent RAG — Worktree Overview

**PINNED to the RAG hardcoded skill.** This document lives in the
harness's skills directory (`~/Desktop/skills/`) so the agent reads
it as standing knowledge. Its purpose: **targeted retrieval.** The
agent should never wander a flat sea of chunks with random search —
it works from this map: resolve the document first, then the place
inside it. Governing register: the Solo Agentic Lab Certification
Register, §5 (DB-1, EMB-1, ORG-1, IDX-1, CHK-1, DIA-1, MEM-1,
PANEL-1, SES-1..3).

---

## 1. State of the territory (2026-10-22)

The old RAG database was **removed on Stewart's order**: it was
embedded with an old model's hidden-state-derived vectors
(model2vec potion-code-16M, 256-dim) and built on fixed-size
chunking. The Postgres 17 instance on **127.0.0.1:5433** (launchd
`com.home.postgres`, data at `~/Library/Application Support/
ModelHome/pg`) remains, empty. The new store is a fresh build whose
design center is the exploration Stewart directed: **context-aware
dynamic chunking (CHK-1)** with **document-index-aware protocols
(DIA-1)**.

Preservation, taken before the removal and kept until the new
build is certified:

- `~/Desktop/Model Home/backups/pre-qwen3-full-2026-10-22.dump`
  — the entire old database (64.4 MB).
- `~/Desktop/Model Home/backups/pre-qwen3-mind-2026-10-22.dump`
  — memories + session tables only (856 KB). The new build
  restores its `mind` schema from this; `rag` is re-derived from
  source files, never restored from old vectors.

## 2. The embedder

**Qwen3-VL-Embedding-2B, 6-bit MLX** — in house at
`~/models/Qwen3-VL-Embedding-2B-6bit/` (a single MLX safetensors
blob under a hash name; config and tokenizer files are assembled
at rebuild step 0). Verified from its tensors:

| Property | Value |
|---|---|
| Embedding dimension | **2,048** |
| Language layers | 28 |
| Vision tower | Present (315 tensors) — images embed in the same space as text |
| Vocabulary | 151,936 |
| Stored form | float32 vectors, cosine similarity |

Also in house, not chosen: Qwen3-Embedding-8B-4bit-DWQ (text-only).
The rules that govern any embedder here (EMB-1): the model is part
of the index's identity; vectors from two models never share an
index; a model change is a full re-ingest event.

## 3. The fresh database shape

One instance, two schemas, roles enforcing the boundary (DB-1):

- **`rag`** — sources, the **document index**, and chunks. Derived
  data: rebuildable from source files at any time.
- **`mind`** — memories (deposit-and-retrieval only, MEM-1) and
  the session tables (messages, journal, scratch, tails, crons).
  Precious: restored from the mind backup, backed up first and
  always, never droppable.
- **Roles:** ingest writes `rag` only; the agent runtime reads
  `rag` and may INSERT + SELECT in `mind`; only the panel/admin
  role mutates memory or dispositions flagged RAG items (PANEL-1).

## 4. The ingest worktree (DIA-1 + CHK-1)

```
source files ──► parse ──► document record ──► dynamic chunking ──► embed ──► rag
                              │                        │                          │
                        structure map            parent/child chunks        2,048-dim
                        + doc summary            boundary-typed             vectors +
                        embedded as an           overlap                    full-text
                        index entry                                        vectors
```

- **A document is ingested as a document**, not as text confetti:
  its index record carries identity, collection, the structure
  map (sections with chunk ranges), provenance, and a
  document-level summary embedded in the same space.
- **Chunking is context-aware and dynamic (CHK-1):** boundaries
  from structure (headings, sections, code fences, tables,
  semantic breaks); sizes dynamic by content type under caps and
  floors; small retrieval chunks carry their parent unit for
  reading in context; overlap only across soft boundaries.
- **Ingest stays idempotent** (ORG-1): unchanged sources skip;
  changed sources replace exactly their own chunks and their
  document record.
- **Images are first-class:** figures and diagrams in a document
  embed through the vision tower into the same space (an
  exploration axis the old embedder could not offer at all).

Collections to rebuild from their source files (chunk counts from
the removed store, for scale): **papers** (26,114), **aie**
(9,203), **uploads** (1,642) — 1,145 registered sources in total.

## 5. The retrieval worktree — targeted, in two stages

1. **Resolve the document index.** Which documents, which
   sections. Topic and "what do we have on X" questions stop here
   if the index answers them.
2. **Search chunks inside the resolution.** Fact probes run hybrid
   search (dense cosine over HNSW + full-text over GIN, fused by
   reciprocal rank; pool = max(5k, 30) candidates for top-k) scoped
   to the resolved documents or sections.

Rules of the road:

- Every hit **cites its place**: document, section path, chunk
  ordinal. A result without a location is malformed.
- If stage one resolves nothing, the fallback to a full-collection
  search is **visible in the result**, never silent.
- **Memory is a different shelf than the RAG.** Personal facts,
  preferences, and session continuity come from `mind` (memory
  deposits, session summaries with their autotail, SES-3) — not
  from chunk search. Scratch is working space, not retrieval.
- Retrieval trouble (rank disagreement, vanished sources,
  conflicting deposits, staleness) goes to the **audit panel with
  its reasoning attached** (PANEL-1) — nowhere else, and the agent
  does not improvise around a flagged item.

## 6. Constants and code pointers

| Constant | Value |
|---|---|
| Instance | Postgres 17 + pgvector, 127.0.0.1:5433, launchd `com.home.postgres` |
| Embedder | Qwen3-VL-Embedding-2B-6bit, 2,048-dim |
| Old chunking (retired) | 1,200 chars / 150 overlap, fixed |
| Hybrid fusion | Reciprocal rank, top-k default 6, pool max(5k, 30) |
| Provision | ~20 GB (≈1.5M chunks at 2,048-dim; sizing in register §5.6) |

Code: `home/store.py` (schema + HybridRetriever), `home/ingest.py`
(parsing + chunking — the CHK-1 work surface), `home/embed.py`
(the embedder binding — the EMB-1 work surface), `home/memory.py`;
Rust side: `crates/home-core/src/tools.rs` (`kb_search`),
`extra.rs` (memory/journal/scratch tools), `store.rs` (session
checkpoints), `session.rs` (consolidation + autotail seeding).

*This overview is a map, and maps go stale: when the rebuild
changes the territory, this document changes in the same commit.*
