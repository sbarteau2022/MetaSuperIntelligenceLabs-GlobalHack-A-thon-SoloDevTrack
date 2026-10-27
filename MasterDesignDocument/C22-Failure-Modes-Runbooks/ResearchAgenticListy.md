# ResearchAgenticListy.md — Master Research Source List + Ingest Runbook

Status: PROPOSED runbook (needs Stewart's nod before it governs any pull).
Splits VERIFIED as Stewart's words, 2026-10-27. Infrastructure facts VERIFIED
against the live Mac paths, 2026-10-27. Entry points marked VERIFIED come
from tool-returned pages; anything marked CONFIRM AT PULL TIME was not
verified this pass and must be confirmed from the source's own pages before
any pull from it.

Placed: MasterDesignDocument/C22-Failure-Modes-Runbooks/ResearchAgenticListy.md
Staged locally only (git add, NOT committed, NOT pushed).

## 1. Master document list

### 1A. Already in the library — 26-source pull, complete Oct 24–26 2026

Full manifest: corpora/library/manifest.jsonl (122,990 lines).
DB state at last check: next_chunk_id 257,002, 23 segments, 7 collections
(papers, research, knowledge, models, platforms, build, engineering).

1. OpenAI
2. Anthropic
3. xAI / Grok — from your search lists, confirmed covered
4. Qwen — from your search lists, confirmed covered
5. Databricks
6. Safe Superintelligence (SSI)
7. LawZero / Bengio — from your search lists, confirmed covered
8. Mistral AI
9. Stanford AI Lab (SAIL) — from your search lists, confirmed covered.
   SAIL ONLY. Stanford HAI was explicitly out of that pull's scope (see 1B).
10. MIT CSAIL — from your search lists, confirmed covered
11. UC Berkeley BAIR — from your search lists, confirmed covered
12. Reflection AI
13. Cognition
14. Thinking Machines Lab
15. World Labs
16. Cohere
17. Cursor
18. Fireworks AI
19. SambaNova
20. NVIDIA
21. Alan Turing Institute
22. Alibaba
23. IBM Research — from your search lists, confirmed covered
24. UT Austin AI Lab
25. USC ISI
26. Oxford ML Group — from your search lists, confirmed covered

### 1B. New additions — 19 sources, not in the library

Grouped by where they came from. The first two groups are Stewart's lists;
the third group is carried forward from the 19-source scope set 2026-10-27
11:24–11:44 CDT, which still stands and was not withdrawn.

From Stewart's message list (7):
1. Caltech
2. Columbia
3. Princeton
4. HF Papers
5. Apple
6. Microsoft
7. WashU St. Louis

From Stewart's screenshots list (8):
8. CMU AI
9. NYU Global AI Frontier Lab
10. Tsinghua / Zhipu
11. HKU
12. Peking University
13. Allen Institute for AI (Ai2)
14. Alberta Machine Intelligence Institute (Amii)
15. Stanford HAI — NEW. SAIL is covered (1A); HAI was explicitly out of the
    SAIL pull's scope, so HAI counts as new. Do not merge HAI into SAIL.

Carried from the 19-source scope (4):
16. DeepSeek
17. Baseten
18. Modal
19. Meta Research / Meta Superintelligence Labs

Cautions that apply to this list (Stewart, 2026-10-27):
- HF Papers mostly mirrors arXiv. Pull HF paper pages, then take the arXiv
  full text. Do not double-count an HF mirror as a second paper.
- Microsoft Research is huge. AI-only filter, always. No unfiltered pull.
- Baseten and Modal mostly publish engineering blogs, not preprints.
  They go to research, not papers.
- Universities lack a single SAIL-style index. There is no second arXiv.
  Everything lands on arXiv first, then org pages + GitHub. Other homes:
  OpenReview, HF Papers.
- Google search blurbs jumble attributions across entries. Every pull
  enumerates from the source's own pages, never from search summaries.

## 2. Ingest rules (hard rules, in order)

1. Full text is the standard. Never partial.
   Abstract-only only when full text is genuinely unavailable, and then the
   chunk text carries ABSTRACT ONLY.
   BULK abstract-only ingest is CLOSED (Stewart ruling, 2026-10-27, on the
   Aug–Oct 2026 arXiv TOC slice). No repeat of that pattern.
2. Title dedupe BEFORE any ingest (Stewart ruling, 2026-10-27).
   Compare the batch's titles against DB segments (title = ordinal-0 chunk
   first heading) and exclude matches. The Aug–Oct 2026 check found 512
   already-have papers this way; that exclusion list is the model.
   Plus the standing dedupe: sha256 + source.
3. Enumerate from the source's own index pages, never from search summaries.
4. One source at a time for the manifest apply (serialized).
   Corpus + manifest only. NO DB writes during a pull.
5. Manifest backup before each source's writes:
   corpora/library/manifest.jsonl.bak-<source>-<stamp>.
6. Papers collection guard: the ingest script refuses collection `papers`
   unless ALLOW_PAPERS=1 is set explicitly. That guard is deliberate.
   A papers ingest needs Stewart's explicit go.
7. Verify per source before the next source starts: chunk-count check
   plus a retrieval smoke. A source is not "done" until both pass.
8. Per-source staging: ~/workspace/<source>-staging/ on the VM, plus a
   receipt.jsonl per source per run (enumerated, new, embedded, ingested,
   manifest backup name, verify result).
9. There is NO single pull script and NO single API call. Each source is
   its own lane. Swarm pattern (proven on the Fireworks re-pull): lanes
   stage separately, manifest apply stays serialized.
10. Collection is decided by ITEM TYPE, not by source:
    full-text papers go to papers; blog posts, docs, and engineering
    write-ups go to research. A mixed source splits by item.

## 3. DB workflow — what scripts to run, when, what to point to

All Mac paths are under ~/Desktop/AtomaDB-RAG-PAMI unless noted.
All VM paths are under ~/workspace unless noted.

| Stage | Run this | Where | When |
|---|---|---|---|
| Enumerate | Per-source lane (agent run, no single script). Enumerate from the source's own index (see §5 for each source's entry point). Write corpus .md files + receipt.jsonl. | VM: ~/workspace/<source>-staging/ | On tier cadence (§4), or on Stewart's order |
| Title dedupe | Title comparison vs DB segments (check_title_overlap pattern: slice titles vs ordinal-0 chunk headings). Exclude matches from the batch. | VM catalog + Mac segments | BEFORE any embed or ingest, every batch, no exceptions |
| Transfer | sshmac cat > Mac path, then verify with a SEPARATE sshmac call (sha256 or line count). | VM file -> Mac data/ dir | After staging verified |
| Embed | cd ~/Desktop/AtomaDB-RAG-PAMI, then: ~/venvs/mlx-embeddings/bin/python scripts/embed_corpus.py <model> <corpus_dir> <vectors.jsonl>. Model: ~/models/Qwen3-VL-Embedding-2B-6bit-RUN. The script's built-in related-vs-unrelated sanity check must pass (FATAL exit otherwise). Crash-safe resume: re-running the same command resumes. | Mac | After transfer, only when the corpus dir is COMPLETE (resume misaligns otherwise) |
| Ingest | cd ~/Desktop/AtomaDB-RAG-PAMI, then: scripts/run_library_ingest.sh <vectors.jsonl> <collection>. Script is fixed (2026-10-27): cd to AtomaDB-RAG-PAMI, set -euo pipefail, scoped vectors+collection args, manifest backup first, papers guard, count check at the end. It never embeds; embedding stays a separate step. | Mac | After embed completes. Papers needs ALLOW_PAPERS=1 |
| Verify | Confirm added chunks == vectors lines, next_chunk_id advanced, segment count grew. Then a retrieval smoke: query terms from the new batch, top hits must carry the right labels. | Mac | Immediately after ingest, before the next source |
| Manifest apply | Serialized apply to corpora/library/manifest.jsonl, backup first (rule 5). One source at a time. | Mac | Per source, after verify passes |

DB binary (for reference, not for hand-rolled ingest):
engine/target/release/atomadb, usage: ingest-vec <db> <collection>
<vectors.jsonl>. Vectors JSONL schema: {doc_id, section, ordinal, text,
vector(2048)}. Embedder: qwen3-vl-embedding-2b-6bit, dim 2048.

## 4. Updating scheduler — PROPOSED, not installed

Mechanism (PROPOSED): Mac-local scheduler (launchd plist or Mac cron),
running a driver on the Mac that works the tiers in order, one source at a
time for the manifest apply. The driver does NOT exist yet; it is TO-BUILD
at scheduler-install time, following §2 and §3 exactly.

Why Mac-local, and why not a VM cron: background/proactivity sessions run
under an egress restriction and cannot reach the Mac, even when main
sessions can. A VM cron therefore cannot embed or ingest. It can only
enumerate and queue. Unattended updates must run ON the Mac, where the DB
and the embedder live. The scheduler also starts NO model servers: embedding
runs the mlx-embeddings venv directly, no server needed, so it does not
conflict with Stewart's one-server-at-a-time doctrine.

Tiers (PROPOSED cadences):

Tier A — weekly check. Fast-moving org labs, aggregators, and blogs.
DeepSeek, Meta Research / MSL, Apple, Microsoft (AI-filtered), HF Papers,
Baseten, Modal, Ai2.

Tier B — monthly check. Universities and institutes, which publish slower
and often in conference roundups rather than a steady stream.
Caltech, Columbia, Princeton, WashU St. Louis, CMU AI, NYU Global AI
Frontier Lab, Tsinghua / Zhipu, HKU, Peking University, Amii, Stanford HAI.

Quarterly audit — every 3 months, regardless of tier: full manifest audit,
a fresh title-dedupe pass over recent batches, and a missed-item sweep
(re-check each source's index for items the tier runs skipped).

Rate discipline (PROPOSED, grounded in the TOC pull precedent):
- arXiv API: 4 s between calls (established), 30,000-result cap, month
  paging, retries with backoff, resumable state.
- Full-text PDF fetches: single-threaded, at least 3 s between fetches.
- Respect each site's robots and rate limits. Never hammer a source.
  A 429 or block stops that source's run; it does not get retried
  through another route in the same run.

Scheduler status: PROPOSED ONLY. Nothing is installed. Install needs
Stewart's nod plus Mac-local setup (launchd plist or cron on the Mac),
which is a main-session job, not a background job.

## 5. What to point to — entry point per new source

VERIFIED = the URL came back from a tool result this pass.
CONFIRM AT PULL TIME = no single index was verified this pass; confirm
from the source's own pages before pulling. Never pull from search
summaries, and never guess a URL.

| Source | Point to | Type | Tier | Default collection |
|---|---|---|---|---|
| Caltech | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| Columbia | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| Princeton | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| HF Papers | https://huggingface.co/papers — VERIFIED. Daily Papers feed. Mostly mirrors arXiv: take the arXiv full text, do not double-count. | Aggregator | A | papers |
| Apple | https://machinelearning.apple.com — VERIFIED. Publications list on the ML Research overview. | Papers | A | papers |
| Microsoft | microsoft.com/en-us/research, AI-only filter, always. No single verified index URL this pass. CONFIRM the publications index AT PULL TIME. | Papers | A | papers |
| WashU St. Louis | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| CMU AI | No single index found (SCS/CS/lab pages + scholars.cmu.edu + arXiv). CONFIRM AT PULL TIME. | Papers | B | papers |
| NYU Global AI Frontier Lab | Lab page https://kyunghyuncho.me/global-ai-frontier-lab-at-new-york-university/ — VERIFIED. Seminar series page; no papers index found. Enumerate via the lab page + arXiv (Cho/LeCun groups). | Papers | B | papers |
| Tsinghua / Zhipu | No single verified index. arXiv (Zhipu/Tsinghua KEG GLM papers). CONFIRM any zhipu.ai research page AT PULL TIME. | Papers | B | papers |
| HKU | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| Peking University | No single index (university rule). Dept/lab pages + arXiv affiliation search. CONFIRM AT PULL TIME. | Papers | B | papers |
| Ai2 | https://allenai.org/papers — VERIFIED. Filterable papers index (year, tags, awards). | Papers | A | papers |
| Amii | No papers index found. amii.ca updates + per-conference roundup posts (ICLR/ICML/NeurIPS) + arXiv. CONFIRM AT PULL TIME. | Papers | B | papers |
| Stanford HAI | https://hai.stanford.edu/research — research index (cited in search results this pass). SAIL is separate and already covered; do not merge. | Papers | B | papers |
| DeepSeek | No single official index verified. arXiv DeepSeek-AI technical reports + deepseek.com. CONFIRM AT PULL TIME. | Papers | A | papers |
| Baseten | https://www.baseten.co/blog/category/ai-engineering/ — VERIFIED, plus blog categories Infrastructure, Model performance, Research. Engineering blogs, not preprints. | Blogs | A | research |
| Modal | No verified blog index this pass. CONFIRM modal.com blog AT PULL TIME. Engineering blogs, not preprints. | Blogs | A | research |
| Meta Research / MSL | ai.meta.com (AI Research -> Publications; researcher pages under ai.meta.com/people/). No single index URL verified this pass. CONFIRM AT PULL TIME. Public pages only; no Meta Model API is involved in this pull. | Papers + posts | A | papers for papers, research for posts |

## 6. What this document does NOT do

- It does not launch any pull. The 19 new sources are listed, not pulled.
- It does not install the scheduler. §4 is a proposal.
- It does not verify the CONFIRM AT PULL TIME entry points. That check is
  part of each source's pull, per rule 3.
- It does not change the already-covered 26. Those stand as pulled.

## 7. Sources for this document

- Stewart's splits, 2026-10-27 (this message): already-covered names,
  new names, HAI-out-of-SAIL-scope note.
- 19-source scope, 2026-10-27 11:24–11:44 CDT (DeepSeek, Baseten, Modal,
  Meta Research carried forward).
- 26-source pull manifest, corpora/library/manifest.jsonl (122,990 lines).
- Ingest script fix, 2026-10-27 (run_library_ingest.sh, embed_corpus.py,
  atomadb usage, vectors schema).
- Title-dedupe ruling + abstract-only ruling, Stewart, 2026-10-27 14:02 CDT.
- Entry-point verification pass, 2026-10-27 (search results for HAI, Ai2,
  Amii, Apple, Microsoft, HF Papers, DeepSeek, Baseten, Modal, Meta
  Research/MSL, CMU AI, NYU GAFL, Zhipu).
