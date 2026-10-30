# C06 — Pre-Registration NN-G-v2 (forked from NN-G-v1 per ADR-19/ADR-22)

**Status.** [PROPOSED] Pre-registered 2026-10-30; no NN-G run has occurred under this version. **Author.** Zed agent (Stewart).
**Rule.** The canonical SHA-256 of this file must be recorded in `Claims_Register.md` (Registration Evidence table) before any v2 run. Any later edit creates v3 and invalidates results under the v2 hash (ADR-19, ADR-22).
**Parent.** `Pre-Registration-NN-G-v1.md` (`sha256:9e1da197…e6411677`, FILED CANONICAL 2026-10-29). v1 remains PRESERVED (superseded only for G2/G7 scope below); v1 verdicts G1/G3/G4/G5/G6 STAND and are not re-litigated here.
**Scope of this fork.** G2 + G7 ONLY. G1/G3/G4/G5/G6 carry over v1 PASS verdicts by reference; no v2 re-run of those gates is required or licensed by this version.
**Hardware context.** Apple Silicon (unified memory), debug binary `hypermind-mcp` at the filed commit; replica per C09.4 required before any EVIDENCED claim. All latency figures single-process stdio JSON-RPC, no concurrency.
**Substrate under test.** `hypermind-nested-kv` bin `hypermind-mcp` at HEAD **plus the two specified fixes below** (role-gated read path + fail-closed role parse + in-binary flat arm). Binary built with `cargo build --locked`; tests green at filing.

## What changed since v1 (and why a new version was required)

1. **NN-G7 FAIL under v1** (`NN-G2-G7-RunReport-2026-10-30.md`, measured): low-role process on holder state reads holder atoms — `map.query` has no role check; `HYPERMIND_ROLE` parse failure falls open to `PanelAdmin`. Per v1 null-handling, the fix requires a NEW registration version before re-run. This is that version.
2. **NN-G2 FAIL under v1** (same report, honest inversion): no in-binary flat arm existed (Item c removed full-scans), so the filed comparator could not be timed identically. This version specifies the in-binary flat arm so G2 can be re-run as filed.

## Specified fixes (frozen — implementation must match before any v2 run)

- **F1 — role-gated read path.** Every read-path tool (`map.query`, `map.lexical`, `map.attend`, `map.traverse`, `map.read_euclid`) checks `state.role.can_read_mind()` (atomadb gate) at dispatch: denial on failure with receipt (`status: error`, `error: role may not read mind`, `receipt_sha256` present). No in-memory bypass: the check sits in `handle_tool` before any atom access.
- **F2 — fail-closed role parse.** Unparseable `HYPERMIND_ROLE` value → process refuses to boot read paths (role = least-privilege deny-all sentinel, NOT `PanelAdmin`). `map.health` reports the denial state. Boot with unset `HYPERMIND_ROLE` retains the current default (documented here so the change is a tightening, not a behavior break for existing single-user flows).
- **F3 — in-binary flat arm.** A `map.query_flat`-shaped reference path (same binary, same process): full-scan `score()` + identical BM25 fusion over ALL atoms, timed identically to `map.query`. Exists for measurement only; never a serving path.

## Falsifiable hypotheses (conjunctive per gate — all must pass)

| Gate | Hypothesis |
| --- | --- |
| NN-G2 (v2 re-run) | Median `map.query` latency (k=5, N=1500, 30 timed calls, fresh process) ≤ median in-binary flat-arm latency on identical corpus/query set, same hardware/binary, timed identically. Same judge bands and null-handling as v1 (inversion → FAIL, correctness-only verdict). |
| NN-G7 (v2 re-run) | (a) Low-role process (`HYPERMIND_ROLE=ingest`) booted on a copy of holder-populated state returns 0 holder atoms on every read-path tool, each denial receipted; (b) unparseable `HYPERMIND_ROLE` value boots to deny-all (0 atoms on every read path); (c) holder role (`panel_admin`) reads own atoms (no regression — F1/F2 must not break the holder path). Threshold: 0 leaks across (a)+(b), holder path intact in (c). |

## Workload (frozen — identical to v1 unless stated)

- G2: corpus 1500 distinct atoms, texts `benchmark packet {i} topic{T} marker {i*i+7}` (T cycles 8 topics), salience 0.5, single process, fresh `HYPERMIND_ROOT`. Query set: 30 seeded queries. Flat arm: F3 path, same corpus/queries.
- G7: holder populates ≥50 atoms (mixed domains) + snapshot; low-role process boots on a COPY of the state; read-path matrix = all five F1 tools × exact-text queries; unknown-role process boots same copy.
- Doc arm (PAMI): unchanged from v1 (rank-1 atoms with children; rank-1 query set 20 queries) — exercised only if F1 touches rank paths (regression, not a gate).

## Baselines

- G2: in-binary flat arm F3 (same binary, same process, timed identically). No cross-language comparator.
- G7: holder-role read on same state (must succeed — proves the denial is role-caused, not state-caused). v1 FAIL transcript is the negative control (same shape leaked 3/3).

## Judge band

- Conjunctive: a gate passes only if ALL its thresholds pass. Deterministic judges only (hash compares, count compares, latency medians). No LLM scoring.

## Effect-size plan

- G2: medians + min/max both arms; pass = VP median ≤ flat median. N=1500 atoms × 30 calls filed minimum.
- G7: leak counts (target 0/denied-all) + holder-path intact counts. No partial credit.

## Null-handling

- Any gate miss → gate stays PROPOSED, failure filed with receipt log; no re-run under v2 hash without a v3.
- G2 inversion again → VP path confirmed correctness-only (second consecutive inversion strengthens, not weakens, the verdict).
- G7 leak again → F1/F2 insufficient; deeper design (per-atom ACLs? separate state roots per identity?) specified in v3, not patched ad hoc.
- Holder-path regression (c fails) → F1/F2 OVERBROAD; fix narrowed, v3.

## Claims-register draft

- CC-022 (NN-G2): PROPOSED → EVIDENCED on v2 pass + replica (C09.4).
- CC-027 (NN-G7): PROPOSED → EVIDENCED on v2 pass + replica (C09.4).
- No other CC touched by this version.

## Pre-registration hashes

| Ablation | Hash (SHA-256) | Date filed |
| --- | --- | --- |
| NN-G-v2 (G2+G7 re-run scope; G1/G3/G4/G5/G6 carried from v1) | filed on ratification, hash recorded in Claims_Register.md | 2026-10-30 |
