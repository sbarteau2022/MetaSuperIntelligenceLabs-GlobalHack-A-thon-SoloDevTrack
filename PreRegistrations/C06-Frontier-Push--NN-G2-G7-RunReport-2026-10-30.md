# NN-G2 / NN-G7 — Run Report 2026-10-30 (measured verdicts under NN-G-v1 prereg)

**Prereg.** `Pre-Registration-NN-G-v1.md`, filed hash `sha256:9e1da197f5239a4ec6579cfa2082c743e62034429a527d6725336b28e6411677`
(recorded in `Claims_Register.md` Registration Evidence). No prereg text touched in this pass — verdicts only.
**Binary.** `hypermind-mcp` debug, built `cargo build --locked` (PAMI rank tags, VP exact-prune kNN + BM25 fusion).
**Hardware.** Apple Silicon (this Mac), stdio JSON-RPC, single process, no concurrency.
**Harness.** Python stdio drivers (fresh `HYPERMIND_ROOT` per run unless stated); wall-ms per call inside harness, stdio round-trip included both arms.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| NN-G2 speed | **FAIL (honest inversion)** — VP arm median ~4.2–4.6ms vs in-harness full-fusion flat median ~5.2ms on COMPUTE terms, but on the filed end-to-end measure (wall ms per call, both arms) the VP path does not beat flat: flat-faster observed on the geo-only shape (2.1ms), and the VP path carries fixed costs (8k-capped shortlist, per-call re-score, receipt + serialization) that keep its end-to-end median above flat at N=1500 | VP arm: N=1500, k=5, 30 timed calls, median 4.25ms (min 2.77, max 5.20), exact top-1 30/30; transport floor (`map.health` round-trip) median 0.18ms — VP cost is compute, not transport. Flat arms: geo-only full `score()` sort median 2.10ms (agreement 0/30 vs server — WRONG comparator, fusion missing); full-fusion flat (same 0.5*geo+0.5*BM25 over all 1500 atoms) median 5.19ms with 10/10 top-1 agreement vs VP arm. Per prereg null-handling: inversion → NN-G2 FAIL; VP path ships as correctness infrastructure, not a speed claim |
| NN-G7 leaks | **FAIL (genuine substrate gap, measured)** — low-role process (`HYPERMIND_ROLE=ingest`) booted on a copy of real persisted state (53 atoms) reports `atoms at boot: 53` and `map.query` returns holder atoms (3/3 hits) with `status: ok` + receipt | Mechanism (VERIFIED from source): `state.role` (set from `HYPERMIND_ROLE`, default `PanelAdmin`) gates only atomadb mind paths (`mind_deposit_with_id`, `can_read_mind` on `mind_list`); `map.query` serves the in-memory `atoms` map with NO role check; boot loads `mind.jsonl` via raw fs read, bypassing `can_read_mind`. Fail-open default: unparseable `HYPERMIND_ROLE` string → falls back to `PanelAdmin` (measured: `HYPERMIND_ROLE=mallory` → `server_role: PanelAdmin`). Correct-scoping notes: `map.carve` fibers DO isolate (fiber-scoped views, measured ok); `map.create` on a fresh subpath yields empty state (measured) — the leak is specifically same-state-root cross-identity read, which has NO gate. Per prereg null-handling: gate stays PROPOSED; fix (role check on `map.query` read path + fail-closed default) requires a NEW registration version before re-run (ADR-19/ADR-22) — NOT implemented in this pass |

## Method notes (what the numbers mean and don't)

- NN-G2: the 0/30 geo-only "agreement" run is RETAINED as a diagnostic of comparator choice, not a gate result — geo-only ranking is not the server's fused ranking, so disagreement was expected and proves only that the comparator must be fusion-identical (which the 10/10 fused-flat run is). The filed bar is end-to-end wall-ms; the verdict stands on the VP arm's measured median vs like-for-like flat work.
- NN-G7: prior attempts measured `atoms: 0` for the low-role process because (a) `map.create` on a fresh subpath yields empty state and (b) snapshot files live under the workspace root, not the state root (`open_or_create(root.join(".hypermind"))` + `run_stdio`: `open_or_create(root/.hypermind)`). The decisive run boots the low-role process on a COPY of real persisted state — the only shape that tests cross-identity read. `map.recover` on the copy reports `parse-error` (snapshot bincode/schema drift, pre-existing) — NOT load-bearing for G7: boot-loaded atoms (53) sufficed to demonstrate the read.
- No EVIDENCED promotion (replica per C09.4 still required where gates pass; G2/G7 did not pass).
- Corrections are entries: the 2026-10-29 report's "NOT RUN" rows stand as the prior record; this report advances both to measured FAIL verdicts with evidence.

## What this run licenses

- NN-G2: VP path = correctness infrastructure (exact-prune + receipts), NOT a speed claim. Any future speed claim needs a same-binary flat arm or N where tree pruning demonstrably amortizes (filed minimum N=1500 held; larger N strengthens).
- NN-G7: a specified fix direction (role-gated `map.query` + fail-closed role parse) under a NEW prereg version. No leak claim may be made until re-run passes.
