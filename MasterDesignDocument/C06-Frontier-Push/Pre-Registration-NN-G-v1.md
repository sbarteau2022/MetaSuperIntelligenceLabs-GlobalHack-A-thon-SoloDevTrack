# C06 — Pre-Registration NN-G1..NN-G7

**Status.** [PROPOSED] Pre-registered 2026-10-29; no NN-G run has occurred under this file. **Author.** Zed agent (Stewart).
**Rule.** The canonical SHA-256 of this file must be recorded in `Claims_Register.md` (Registration Evidence table) before any NN-G run. Any later edit creates a new registration version and invalidates results under the old hash (ADR-19, ADR-22).
**Hardware context.** Apple Silicon (unified memory), debug binary `hypermind-mcp` at the filed commit; replica per C09.4 required before any EVIDENCED claim. All latency figures are single-process stdio JSON-RPC, no concurrency.
**Substrate under test.** `hypermind-nested-kv` bin `hypermind-mcp` at HEAD (PAMI rank tags rank 0/1, rotate-then-fold, `map.pami_stats`, VP exact-prune kNN + BM25 fusion). Binary built with `cargo build --locked`; tests green at filing (42 bin + 3 lib + 8 transducer).

## Falsifiable hypotheses (conjunctive per gate — all must pass)

| Gate                                       | Hypothesis                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| NN-G1 Nested retrieval parity              | VP-tree path (`map.query` k=5 shortlist+fusion) returns the same top-1 as the ingested atom for 50 seeded EXACT-TEXT queries on a fixed 500-atom corpus (identical texts, salience 0.5); threshold ≥ 49/50. Partial-token probes are diagnostic only (under-specified queries legitimately tie — geo flattens to ~-1.0, BM25 to noise; see RUN report). |
| NN-G2 Nested retrieval speed               | Median `map.query` latency (k=5, N=1500, 30 timed calls, fresh process) ≤ median flat brute-force latency on identical corpus/query set on the same hardware/binary; measurement: wall ms per call inside the harness (stdio round-trip included both arms).                                                                                            |
| NN-G3 Zero lost judge receipts             | 500 ingests → snapshot → recover → all 500 atoms present, epoch preserved, every tool result carries `receipt.receipt_sha256`, snapshot hash round-trip verified. Threshold: 100% receipts, 0 drops.                                                                                                                                                    |
| NN-G4 Prune receipt reproducibility        | Session snapshot → export passport → recover into fresh root → atom set, epochs, and passport digest identical to pre-recovery state. Threshold: byte-identical digests.                                                                                                                                                                                |
| NN-G5 No model self-summary in record path | `grep` audit: no model-generated summary text enters `map.ingest` without a judge pass in the run log; every ingested atom carries `judge: pass` with structural reason. Threshold: 0 unjudged atoms committed.                                                                                                                                         |
| NN-G6 Multimodal parity                    | 5-level corpus (elementary → postdoc-multilingual) round-trips byte-exact ingress→expand per `transducer_pressure.rs`; exact-text `map.query` top-1 per level. Threshold: 8/8 transducer tests + 5/5 level top-1.                                                                                                                                       |
| NN-G7 Capability gate                      | Cross-identity retrieval attempt (Role parse other than holder) denied with receipt; cross-silo query without carve returns no out-of-domain atoms unless pinned. Threshold: 0 leaks.                                                                                                                                                                   |

## Workload (frozen)

- Corpus: 1500 distinct atoms, texts `benchmark packet {i} topic{T} marker {i*i+7}` (T cycles 8 topics), salience 0.5, single process, fresh `HYPERMIND_ROOT`.
- Query set: 50 seeded queries (`benchmark packet {seed} {topic} marker`, seeds fixed at run-manifest filing).
- Doc arm (PAMI): 50 rank-1 atoms with `children` over the chunk set; rank-1 query set 20 queries.
- Transducer corpus: `tests/transducer_pressure.rs` (5 levels + multilingual + 5k run).

## Baselines

- Flat brute-force: full-scan `score()` sort over all atoms (the `vp_tests` reference implementation), same binary, same corpus, timed identically.
- No-receipt run: not a baseline — receipts are mandatory; absence is a failure, not a comparison.

## Judge band

- Conjunctive: a gate passes only if ALL its thresholds pass. No averaging across gates.
- Deterministic judges only (hash compares, count compares, latency medians). No LLM scoring of any verdict.

## Effect-size plan

- NN-G2: report medians + min/max both arms; pass = VP median ≤ flat median. No small-N significance theater: N=1500 atoms × 30 calls is the filed minimum; larger N strengthens, never weakens.
- NN-G6: exact-match counts, no partial credit.

## Null-handling

- Any gate miss → gate stays PROPOSED, failure filed with the receipt log; no re-run under a new hash without a new registration version.
- Latency inversion (flat faster) → NN-G2 FAIL; the VP path ships as correctness infrastructure, not a speed claim.
- Receipt drop → NN-G3 FAIL; substrate not shippable until 100%.

## Claims-register draft

- NN-G1..G7 PROPOSED → EVIDENCED on pass + replica (C09.4) + necessity ablation where applicable.
- `map.health` parity fields may cite NN-G1/G2 only after pass; until then `exact_attention_measured=false` stands.

## Pre-registration hashes

| Ablation                                                    | Hash (SHA-256)                                        | Date filed |
| ----------------------------------------------------------- | ----------------------------------------------------- | ---------- |
| NN-G1..G7 (see Registration Evidence in Claims_Register.md) | filed 2026-10-29, hash recorded in Claims_Register.md | 2026-10-29 |
