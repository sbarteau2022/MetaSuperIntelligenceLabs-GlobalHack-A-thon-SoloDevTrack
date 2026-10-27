# Pre-Registration — A1–A10

**Rule.** Every ablation requires this preregistration hash to be written before the run.
An ablation run without a pre-filed hash is automatically disqualified (C11.3).
Null/negative results are recorded as first-class entries.

## A1 — KV cache (hyperbolic vs standard)
- Hypothesis: nested hyperbolic cache improves tok/s and reduces peak memory vs O(N²) transformer.
- Baseline: standard transformer on same hardware/workload.
- Treatment: Atlas/ACFT path (C05/C12).
- Judge: throughput + peak memory on fixed prompt corpus.
- Status: TBD.

## A2 — Memory recall after compaction
- Hypothesis: attractor-basin compaction retains original objective after 2 compactions.
- Baseline: flat context dump.
- Judge: recall of original objective = 100%.
- Status: TBD.

## A3 — Retrieval (BM25 vs dense)
- Hypothesis: BM25+dense is no worse than BM25 on labeled queries.
- Judge: top-1 doc identical or better; no false-dense claims.
- Status: TBD.

## A4 — Model-size agent (single model vs specialist fibers)
- Hypothesis: live weight-copy count stays 1 under N=8 swarm.
- Judge: weight-copy count + per-task latency.
- Status: TBD.

## A5 — Agent count
- Hypothesis: N lanes improve judge-gated task pass rate without OOM.
- Judge: pass rate + throughput-per-watt.
- Status: TBD.

## A6 — Context budget
- Hypothesis: larger budgets improve accuracy/token-cost curve.
- Judge: accuracy vs token cost at 200-line / 8k / 131k.
- Status: TBD.

## A7 — Hooks
- Hypothesis: deterministic pre/post gating is correct.
- Judge: unauthorized merge denied, authorized merge allowed.
- Status: TBD.

## A8 — RAG ingest round-trip
- Hypothesis: ingest→query returns text with receipt.
- Judge: round-trip + receipt present.
- Status: TBD.

## A9 — Memory auto-injection
- Hypothesis: agent can state a prior decision from another lane without re-prompt.
- Judge: cross-lane recall test.
- Status: TBD.

## A10 — Prompt-injection hygiene
- Hypothesis: poisoned memory does not change lane tool calls without judge flag.
- Judge: red-team injection must be flagged/denied.
- Status: TBD.
EOF
echo done
