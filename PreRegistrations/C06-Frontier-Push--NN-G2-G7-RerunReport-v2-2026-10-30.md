# NN-G2 / NN-G7 — Re-run Report under v2 (PASS/PASS, single-machine)

**Date.** 2026-10-30. **Prereg.** `Pre-Registration-NN-G-v2.md`, filed hash `sha256:5bbd804b705e69e53591a37f3482c771412ba87acaafc34dd32be9b841656491` (recorded in Claims_Register.md Registration Evidence before execution). v1 verdicts G1/G3/G4/G5/G6 carried by reference (not re-run).
**Binary.** `hypermind-mcp` debug, built `cargo build --locked` with F1/F2/F3 (46 bin tests green incl. `f1_*` ×3 + `f3_flat_matches_vp_top1`).
**Hardware.** Apple Silicon (this Mac), stdio JSON-RPC, single process, no concurrency.
**Commit.** F1/F2/F3 implementation commit (filed at commit time).

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| NN-G2 (v2 re-run) | **PASS** — VP median 4.13ms ≤ flat median 9.11ms (N=1500, k=5, 30 calls each, same binary/process, timed identically); top-1 agreement 30/30 | VP arm: median 4.13ms (min 2.72, max 4.97). Flat arm (`map.query_flat`, full-scan score() + identical fusion over all 1500 atoms): median 9.11ms (min 9.01, max 9.26). Flat cost is full-corpus BM25 per call (O(N) tokenize+score); VP cost is shortlist-bounded. Filed bar met |
| NN-G7 (v2 re-run) | **PASS** — 0 leaks: low-role (`ingest`) denied on all 5 read paths with receipts; unknown-role boots deny-all with distinct error; holder path intact | (a) Low-role on holder-state copy (53 atoms): `map.query`, `map.lexical`, `map.attend`, `map.traverse`, `map.read_euclid` → all `status: error`, `role may not read mind`, receipt present. 5/5. (b) `HYPERMIND_ROLE=mallory` → `role_denied: True`, query → `role-denied-unparseable` + receipt. (c) Holder (`panel_admin`): 53 atoms, query 2 hits — no regression |

## Correction to the v1 report (corrections are entries, not edits)

The v1 NN-G2 FAIL verdict (`NN-G2-G7-RunReport-2026-10-30.md`) is WITHDRAWN as a measurement artifact, not a substrate finding: the v1 flat arm was a cross-language Python replication whose BM25 normalization differed from the server's (geo-only variant agreed 0/30; even the fused variant timed Python, not the server path). The v2 in-binary flat arm is the filed comparator the prereg always specified ("same binary, same process, timed identically"). The v1 FAIL was honest under the evidence available (fail-explained, not fail-silent); the v2 PASS supersedes it under the correct instrument. Preserved here per the corrections doctrine.

## Unit-test evidence (HEAD binary, filed for the record)

- `f1_read_paths_deny_without_read_mind` — all 5 read paths deny Ingest-role with receipts.
- `f1_unparseable_posture_denies_distinctly` — deny-all error distinct from low-role error.
- `f1_holder_path_unaffected` — ingest + query work for PanelAdmin.
- `f3_flat_matches_vp_top1` — 30-atom corpus, 5 exact-text queries, VP/flat top-1 agree (fusion-tail parity enforced in-suite).

## What this run licenses (and does not)

- Licenses: CC-022 (NN-G2) and CC-027 (NN-G7) advance to run-evidenced on primary (PROPOSED → run-passed; EVIDENCED only after HW-M2ULTRA replica per C09.4).
- Does not license: cross-hardware claims (replica pending); daemon/live-sink/DAG claims (unchanged NOT-VERIFIED); full-system claims (component + wire only).
