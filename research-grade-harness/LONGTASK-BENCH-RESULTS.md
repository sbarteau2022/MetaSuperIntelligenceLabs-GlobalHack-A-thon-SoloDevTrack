# Long-Task Head-to-Head — Results (filed AFTER pre-reg, before claims)

**Date.** 2026-10-28.
**Pre-reg.** `LONGTASK-BENCH-PRE-REG.md` (filed before runs; predictions P-01..P-08).
**Hardware context (REPORTED, this host).** macOS, Apple silicon (Metal unified memory); Rust tests in-process; Harness² Python 3.14.7 subprocess executor. No watt-meter calibration — all energy figures PROPOSED surrogates (see design §8.1).
**Containment.** Harness² demo ran with read-only clone + fresh `target/harness2-k5/` output dir; no network; no benchmark creds. RG-AH in-process synthetic judges. Zero kernel violations / bypasses / egress anomalies observed on both sides.

## 1. Harness² (on-disc, FIRST per order) — VERIFIED from run artifacts

- Command: `PYTHONPATH=<harness2> python3 -m harness2.cli.demo --output target/harness2-k5 --k 5 --mode parallel`
- `demo.json`: done=2, failed=0, execution_count=**40**, editor_calls=**34** (P-02 ✓).
- Ledger per task (`summary.json` agg mode): R=**20** (=5 base + 10 probes + 5 finals), expected_R=20; P=**15**, expected_P=15; stages len **15** (P-01 ✓).
- Step count per task: 20 rollouts + 15 sessions + 1 selection + 1 reflection = **37 steps** — in 30–50 band (P-01 ✓, both tasks identical).
- Harnessing-level retention: ledger-2 `library_entries_before={agg:1}` vs ledger-1 `{agg:0}` — experience carried forward (P-03 ✓).
- `kept_base=[]`, `fell_back=[]` on both tasks: no abstention in demo (matches known finals-only-pool property; NOT a defect here, noted per audit §"What not to copy" #2).

## 2. RG-AH (this crate) — VERIFIED from `cargo test --test longtask_bench`

- 40-packet DAG validates (`validate()` Ok); sequential engine run:
  - **passed=39, escalated=1** (`p10-redteam` leaf), skipped=0 — no cascade (P-04 ✓).
  - receipts=**40** (39 PASS + 1 ESCALATE), every packet receipted (bar ✓).
  - packet executions = **41** (39×1 + 1 redteam retry) — in 30–50 band (P-04 ✓).
  - energy spent **0.6150 J-surrogate**, erased **1640 bits**, `remaining ≥ 0` — gate holds (P-05 ✓).
  - coverage_exact **40/40** after 3 decay epochs (P-07 ✓ — reported as number, no bar asserted).
  - PPC k=2: `kept_base=false, abstained=false, gain=+0.050, energy=2.0000, adopted_v=1`; R=8, P=6 ledgers exact (P-06 ✓ — outcome direction not pre-constrained).
  - AIF ranker smoke: EIOR-ranked candidate selected over uniform baseline.
- Full suite: **16/16 green** (8 implementation + 1 longtask + 7 pressure), clippy 0 warnings, fmt clean.

## 3. Head-to-head (labels per §7 — read carefully)

| Dimension (30–50 step band) | Harness² k=5 (per task)                                                | RG-AH 40-packet                                                    | Label                                                                 |
| --------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------- |
| Steps                       | 37 (20 rollouts + 15 sessions + sel + refl)                            | 41 packet executions                                               | VERIFIED (both in band)                                               |
| Task completion             | 2/2 ok, 0 failed                                                       | 39 pass / 1 escalated (engineered fail)                            | VERIFIED (different fault models — see §4)                            |
| Receipt/ledger completeness | R/P ledgers exact; per-stage notes; selection+reflection adoption docs | 40/40 receipts; energy+erased-bits per receipt                     | VERIFIED                                                              |
| Energy accounting           | usd=0.0 recorded (local demo, no priced lane); ledger shape present    | 0.6150 J-surrogate spent of 50.0 budget; Landauer min tracked      | VERIFIED as recorded; **PROPOSED as physics**                         |
| Abstention                  | none available (finals-only pool)                                      | KEEP_BASE first-class (not triggered here — pool beat base +0.050) | VERIFIED behavior; INFERRED consequence                               |
| Wall clock                  | Python-subprocess minutes-scale                                        | in-process sub-second                                              | **REPORTED only — NOT a speed claim** (unlike substrates, P-08 holds) |

## 4. Non-comparabilities (INFERRED — where the evidence ends)

1. **Different fault models.** Harness²'s demo task is pass-only (scripted fix always lands); RG-AH's long task deliberately injects one unmeetable judge to exercise escalation. Pass-rate comparison is therefore meaningless; doctrine-behavior comparison (escalate cleanly vs finish cleanly) is the valid readout — both behaved per spec.
2. **Different executors.** Harness² shells Python per rollout; RG-AH synthesizes output in-process. Latency/cost columns are substrate artifacts, not harness rankings.
3. **Synthetic quality on both sides.** Scripted proposer (Harness²) and marker-scoring PPC (RG-AH `score()`) — no quality-superiority claim licensed. A real-model PPC run (open item PRESSURE-AUDIT §"Open items") is the gate for H5/H2 claims.

## 5. Pre-reg verdict

P-01 ✓ | P-02 ✓ | P-03 ✓ | P-04 ✓ | P-05 ✓ | P-06 ✓ | P-07 ✓ | P-08 ✓ (held — no speed claim made). All 8 predictions confirmed; no correction needed.

## 6. Dated corrections

- 2026-10-28 (Stewart standing doctrine — NO SYNTHETIC BENCHMARKS, see `NO-SYNTHETIC-BENCHMARKS.md`): all comparative readings in §3 are DOWNGRADED to throughput/ledger-shape observations. Both sides ran synthetic quality signals (scripted DemoProposer; marker-heuristic `score()`); no quality/waste/energy superiority is licensed. Gates for citing bench numbers: real-model runs + watt-meter calibration. _(none other at filing)_
