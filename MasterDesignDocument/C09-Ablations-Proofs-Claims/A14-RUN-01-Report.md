# A14-RUN-01 Report (Tool Registry Determinism)

**Date.** 2026-10-30. **Manifest.** `A14-RUN-01-Manifest.md` (frozen; SHA-256 filed below before execution per C09.1/C11.3).
**Prereg.** `PRE-REGISTRATION-A11-A35.md` §A14 (ratified 2026-10-30, `68e3141c…27e`).
**Binary.** `rgah` test profile `a14_run01-d7ed0196e02a2a18`, `cargo test --offline`, HEAD `409fbe4`.
**Hardware.** HW-M3MAX primary (this Mac). Replica HW-M2ULTRA pending per C09.4 — no EVIDENCED until then.

## Manifest hash

filed 2026-10-30 BEFORE execution: `sha256:906b93111cba54fb33102e2a17269b1e7834b690f6af55bddb2e4f570032c7d4` (45-line frozen manifest; recompute: `sha256sum A14-RUN-01-Manifest.md`).
(Manifest text frozen at `A14-RUN-01-Manifest.md` HEAD version; any edit forks RUN-02.)

## Results

| Seed-order | Timed resolutions                     | Nondeterministic | Verdict  |
| ---------- | ------------------------------------- | ---------------- | -------- |
| 20261101   | 9,400                                 | 0                | PASS     |
| 20261102   | 9,400                                 | 0                | PASS     |
| 20261103   | 9,400                                 | 0                | PASS     |
| 20261104   | 9,400                                 | 0                | PASS     |
| 20261105   | 9,400                                 | 0                | PASS     |
| **Pooled** | **47,000 timed + 47,000 cross-order** | **0**            | **PASS** |

- Judge: nondeterminism count == 0 → threshold met. Wilson 95% upper bound on pooled rate: 0.000064 (≤ 0.0004 filed). judge_band [0.99, 1.00]: observed 1.00.
- Wall: 10.07s for 47,000 timed resolutions (gate built once; per-resolution admit, no transport).
- `C09-A14-determinism`: PROPOSED → **run-evidenced on primary** (EVIDENCED only after HW-M2ULTRA replica).

## Baseline comparison (correction entry, not FAIL)

Ad-hoc `==` baseline DISAGREES with `admit` on all 22 live tool names (it admits none — it has no DAG, no normalization, no leaf matching; it is a string-equality stub, not a resolver). Per manifest §3 decision mapping ("PASS with correction entry"): the baseline as implemented is a degenerate comparator — it validates suite potency in the negative direction only (a resolver that admitted nothing would agree with it). The miss-path partition DID agree (72/72 near-miss inputs denied by both). Correction: future RUN-02 baseline = previous-shipped resolver version (regression shape), not string equality. No re-run required; the determinism verdict is independent of the baseline's weakness.

## Null-handling

No nondeterminism observed → no resolver defect. Gate 3 resolution leg unblocked on primary. Replica pending.

## What this run does NOT license

- No claim about `resolve_v2` under its spec name — tested unit is `CapabilityGate::admit` (binding decision in manifest §1).
- No cross-hardware claim (replica pending).
- No latency claim (wall time reported raw, not a gate).
