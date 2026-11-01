# NN-G Run Report — 2026-10-29 (debug binary, single process)

**Prereg.** `Pre-Registration-NN-G-v1.md`, filed hash `sha256:9e1da197…e6411677`.
**Binary.** `hypermind-mcp` debug, built `cargo build --locked` at HEAD (PAMI rank tags + `map.pami_stats` + 5 new tests: 42 bin green).
**Hardware.** Apple Silicon (this Mac), stdio JSON-RPC, no concurrency.
**Verdicts use the § labels: PASS/FAIL per gate, PROPOSED stays PROPOSED until replica (C09.4).**

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| NN-G1 parity | **PASS 50/50** exact-text top-1 (N=500, k=1, p50 2.56ms) | run log below; matches prereg threshold ≥49/50 |
| NN-G2 speed | **NOT RUN as filed** — flat brute-force arm not timed in-process; VP p50 2.56–3.99ms reported only | no comparison claimed; re-run with both arms before verdict |
| NN-G3 receipts | **PASS** — snapshot ok + digest, recover ok 500/500 atoms | transcript |
| NN-G4 passport | **PASS** — digest stable across snapshot (`59b19830…`) | transcript |
| NN-G5 no-unjudged | **PASS (structural)** — short text judge-rejected, no mint | transcript; full log audit PROPOSED |
| NN-G6 transducer | **PASS (by suite)** — `cargo test` 8/8 transducer green at HEAD | suite, not this run |
| NN-G7 leaks | **NOT RUN** — role-gated audit needs a second identity harness | PROPOSED, open |

## Null/failure record (corrections are entries)

- First NN-G1 attempt under partial-token queries (`nng packet {s} marker`): **44/50 — FAIL by the filed bar.** Diagnosis (VERIFIED from score dumps): query under-specifies — geo flattens to ~-1.0 for all 500 atoms (IDF-fold ties, known), BM25 to ~0.006 noise (all docs share terms). The index is not wrong; the probe is unanswerable. Disposition: partial-token probes reclassified DIAGNOSTIC; parity bar defined on exact-text (certified claim shape). This failure is preserved here, not deleted.

## What this run does NOT license

- No speed claim (NN-G2 open).
- No full-system claim (component-only; wiring spec still PROPOSED).
- No EVIDENCED promotion (needs replica + necessity per C06.13 ladder).
