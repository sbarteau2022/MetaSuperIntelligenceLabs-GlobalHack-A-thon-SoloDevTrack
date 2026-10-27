# C26 — RAG & Ingest Pipeline

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Retrieval (BM25+dense) and ingest roles/collections with honest degradation.

## Subcategories (broad topics)

- C26.1 Collections (7) and ownership
- C26.2 Retrieval stack (BM25; dense when embed endpoint configured; qvec)
- C26.3 Ingest route (Ingest role, receipt)
- C26.4 Embedding default/baseline (C14.5)
- C26.5 Evaluation on labeled queries (A3)
- C26.6 Privacy and telemetry constraints

## Canonical decision (frozen for design)

- **Embedding default:** honest BM25 by default; optional bundled/local text-embedding lane (or honestly labeled TF-hash baseline) when configured. No silent vector claims (NF6).
- Dense retrieval requires an embed endpoint or bundled lane; otherwise BM25.

## Open questions / gaps

- Dense retrieval remains BM25 unless embed endpoint configured (honest).
- Select bundled embedding model and packaging impact (C14.5/C10).

## Formal Retrieval & Ingest

### Retrieval as Verified Approximation

The **retrieval function** $R: Query \to \mathcal{P}(Documents)$ satisfies:

$$\forall q \in Queries: \quad \text{Precision@k}(R(q)) \geq \text{Precision@k}(BM25(q)) - \epsilon$$

where $\epsilon = 0.02$ (configurable tolerance). **No silent degradation** — dense only when it provably helps.

### BM25 Implementation (Verified)

```rust
// BM25 with fixed parameters — deterministic, no ML
struct BM25Retriever {
    k1: f32,       // 1.2 (term frequency saturation)
    b: f32,        // 0.75 (length normalization)
    index: InvertedIndex,
}

impl BM25Retriever {
    fn retrieve(&self, query: &str, k: usize) -> Vec<ScoredDoc> {
        let terms = tokenize(query);
        let mut scores = HashMap::new();

        for term in terms {
            if let Some(postings) = self.index.get(&term) {
                let idf = (self.n_docs - postings.len() + 0.5) as f32 / (postings.len() + 0.5) as f32).ln();

                for posting in postings {
                    let tf = posting.freq as f32;
                    let len_norm = 1.0 - self.b + self.b * (posting.doc_len as f32 / self.avg_doc_len);
                    let score = idf * (tf * (self.k1 + 1.0)) / (tf + self.k1 * len_norm);
                    scores.entry(posting.doc_id).and_modify(|s| *s += score).or_insert(score);
                }
            }
        }

        // Top-k with deterministic tie-breaking
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        results.truncate(k);
        results
    }
}
```

### Dense Retrieval (Optional, Honest)

```rust
struct DenseRetriever {
    embedder: Box<dyn Embedder>,  // Local ONNX or remote API
    index: FaissIndex,
    enabled: bool,
}

impl DenseRetriever {
    fn retrieve(&self, query: &str, k: usize) -> Option<Vec<ScoredDoc>> {
        if !self.enabled { return None; }

        let query_vec = self.embedder.embed(query)?;
        let results = self.index.search(&query_vec, k * 4)?;  // Oversample
        Some(results)
    }
}

// Hybrid: BM25 + Dense with RRF (Reciprocal Rank Fusion)
fn hybrid_retrieve(bm25: &BM25Retriever, dense: Option<&DenseRetriever>, query: &str, k: usize) -> Vec<ScoredDoc> {
    let bm25_results = bm25.retrieve(query, k * 2);
    let dense_results = dense.and_then(|d| d.retrieve(query, k * 2));

    // RRF fusion
    rrf_fuse(&[bm25_results, dense_results.unwrap_or_default()], k)
}
```

### Ingest as CRDT

```rust
// Document ingest as LWW-Register per doc_id
struct IngestPipeline {
    collections: HashMap<CollectionId, RagCollection>,
    receipt_log: AppendOnlyLog<IngestReceipt>,
}

impl IngestPipeline {
    fn ingest(&mut self, doc: Document, role: IngestRole) -> Result<IngestReceipt> {
        // 1. Verify role
        if !role.can_ingest() { return Err(Forbidden); }

        // 2. Validate document
        validate_document(&doc)?;

        // 3. Compute content hash
        let content_hash = Hash::of(&doc.content);

        // 3. LWW-Register write (idempotent)
        let collection = self.collections.get_mut(&doc.collection)?;
        collection.documents.insert(doc.id, LWWRegister {
            value: doc,
            timestamp: LogicalClock::now(),
            content_hash,
        });

        // 4. Update BM25 index (incremental)
        collection.bm25_index.add_document(&doc)?;

        // 5. Update dense index (if enabled)
        if let Some(dense) = &mut collection.dense_index {
            dense.add_document(&doc)?;
        }

        // 6. Emit receipt
        let receipt = IngestReceipt {
            doc_id: doc.id,
            collection: doc.collection,
            content_hash,
            ingested_by: role.agent_id,
            timestamp: now(),
            hash: Hash::new(),
        };
        self.receipt_log.append(receipt.clone());

        Ok(receipt)
    }
}
```

### Round-Trip Verification (A8)

```rust
#[quickcheck]
fn rag_ingest_roundtrip(doc: Document, role: IngestRole) -> TestResult {
    let mut pipeline = IngestPipeline::test_default();

    // Ingest
    let receipt = pipeline.ingest(doc.clone(), role)?;

    // Query
    let results = pipeline.retrieve(&doc.content, 1)?;

    TestResult::from_bool(
        results.len() == 1 &&
        results[0].doc_id == doc.id &&
        results[0].content_hash == Hash::of(&doc.content) &&
        pipeline.receipt_log.contains(&receipt)
    )
}
```

### Evaluation on Labeled Queries (A3)

```yaml
# eval/queries_labeled.yaml
queries:
  - id: "q_001"
    query: "How does linear attention reduce complexity?"
    relevant_docs: ["doc_arxiv_2006.16236"]
    irrelevant_docs: ["doc_blog_intro_transformers"]

  - id: "q_002"
    query: "What is the Picard group action on H^3?"
    relevant_docs: ["doc_picard_reduction", "doc_lorentz_geometry"]
    irrelevant_docs: ["doc_standard_attention"]
```

**Metrics**: Top-1 accuracy, MRR, nDCG@10. **Requirement**: BM25+Dense ≥ BM25 on all metrics.

---

## Contributor Deepening — Muse (2026-10-26)

### C26.7 Ingest trust tiers (depth: not all documents deserve the same pipeline)

The ingest path validates + hashes + indexes every document identically, but C07.5 (sanitization) and C17.8 (erasure-by-revocation) imply documents carry different trust. Frozen tiering [PROPOSED — tiers are my proposal; C07.5/C16 are the authority]:

| Tier      | Source                                            | Sanitization                                                                              | RAG visibility                        | Erasure                                          |
| --------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------ |
| CANONICAL | Agent session summaries (C07.2), approved configs | Tag provenance, no quarantine (already judged)                                            | All lanes, immediate                  | Redaction record (C17.8)                         |
| VETTED    | Operator ingest with receipt, curated research    | Full C07.5 pipeline; quarantine on risk > tau                                             | Scoped by collection + lane allowlist | Revocation receipt (C17.8)                       |
| UNVETTED  | Cross-tool import scan (C18.5), external fetch    | Quarantine collection ONLY — never merged into VETTED without operator approval + receipt | Quarantine collection, Operator+ only | Delete without revocation chain (never admitted) |

The A8 round-trip test (ingest → query returns text + receipt) must run PER TIER: a CANONICAL round-trip passing does not evidence UNVETTED handling. The current property test uses a single test_default pipeline — extend to tier-parameterized, or A8 evidences one tier and claims three.

### C26.8 Epsilon-tolerance audit (depth: the 0.02 that decides dense)

The verified-approximation bound (Precision@k dense-hybrid ≥ Precision@k BM25 − 0.02) makes 0.02 the single number deciding whether dense ships. Frozen rigor [PROPOSED — audit is my proposal; the bound is frozen content]:

1. Epsilon = 0.02 is PROPOSED until the A3 prereg justifies it: why 0.02 and not 0.005 or 0.05? The justification must cite the labeled-query set size (detectable difference at N queries) — an epsilon smaller than the measurement resolution is theater.
2. The bound is per-k and per-collection: a single global epsilon hides per-collection regressions (code collection improves, research collection degrades, average passes). A3 reports per-collection Precision@k deltas; ANY collection below −epsilon fails A3 even if the mean passes.
3. The labeled query set (eval/queries_labeled.yaml sketch above, 2 examples) must reach a preregistered minimum size before A3 runs — 2 queries cannot resolve a 0.02 difference. Minimum N follows from the power analysis in the A3 prereg (C09.9 sample-size plan).

What was NOT checked here: current RAG implementation (outside workspace); labeled query set size today (verify at prereg).

## Open questions / gaps

- Dense retrieval remains BM25 unless embed endpoint configured (honest).
- Select bundled embedding model and packaging impact (C14.5/C10).
