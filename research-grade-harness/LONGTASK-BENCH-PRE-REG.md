# Long-Task Head-to-Head — Pre-Registration (filed BEFORE runs)

**Date filed.** 2026-10-28 (Stewart / Zed agent session).
**Status.** PROPOSED predictions. No run has been executed under this pre-reg at filing time.
**Scope.** 30–50 step long-task bench (min 30, max 50 steps per task/harness), Harness² (on-disc, Google Research release) FIRST, then RG-AH (this crate). Head-to-head on throughput, waste, energy ledger, and long-horizon doctrine behavior.
**Rule.** No filed hash, no run. Predictions below are falsifiable; results go in `LONGTASK-BENCH-RESULTS.md`, never edited here. Corrections append as dated entries.

## 1. Workloads (VERIFIED paths at filing)

- **Harness² (on-disc, third-party, Apache-2.0):** `~/Desktop/AtomaApp/Harnesses-2026-10-23/harness2`, HEAD `1cfe254` (REPORTED from prior audit §1.1; not re-hashed here). Entry: `python3 -m harness2.cli.demo --output <fresh-dir> --k 5 --mode parallel`. Deterministic local adapter (DemoAdapter/DemoProposer, no model creds, no benchmark judge). 2 ledger tasks threaded with library forward.
- **RG-AH (this crate `rgah` 0.1.0):** `cargo test --test longtask_bench` — 40-packet DAG (recon → plan → 5 parallel implement lanes → integrate → verify → harden → ship), sequential engine, deterministic judges, one engineered leaf failure (redteam probe, `max_rounds=2`, unmeetable needle) to exercise two-strike escalation.

## 2. Step-count definition (shared)

One step = one harness-executed action: a rollout execution, an editor/probe session, a composition, a selection, a reflection, or one packet execution (incl. retries). Predicted band: **30–50 steps per task**.

## 3. Predictions (PROPOSED — falsifiable)

| ID | Prediction | Judge (pass/fail of the pre-reg) |
|----|-----------|----------------------------------|
| P-01 | Harness² k=5 agg, per task: R=20 rollouts (5 base + 10 probes + 5 finals), P=15 editor sessions, +1 selection +1 reflection = **37 steps/task** (in 30–50 band) | `summary.json` ledger shows `R=20`, `expected_R=20`, stages len 15 per task |
| P-02 | Harness²: `done=2, failed=0` across both ledger tasks; execution_count = 40 (+0 retries) | `demo.json` fields |
| P-03 | Harness²: library grows task-over-task (task 2 sees `library_entries_before ≥ 1`) — harnessing-level retention visible | `demo.json` tasks[1].library_entries_before |
| P-04 | RG-AH 40-packet plan: `plan.validate()` Ok; sequential run yields **39 passed, 1 escalated** (redteam leaf), 0 skipped-cascade; receipts = 40 entries (39 PASS + 1 ESCALATE) | test assertions |
| P-05 | RG-AH: energy slack `remaining ≥ 0` at end (second-law gate holds); budget charged on every packet incl. failed attempts | test assertion on `EnergyBudget.remaining()` |
| P-06 | RG-AH: PPC k=2 on long-task bundle reports `kept_base` honestly (either outcome accepted; abstention rate reported, not minimized) | test prints metrics; no assert on direction |
| P-07 | RG-AH: exact declared-context coverage measured over all 40 packets' keys after phased decay; reported as number, no bar asserted (calibration open) | test prints coverage |
| P-08 | Wall-clock comparison reported with hardware context only; **no speed claim** — Python-subprocess vs in-process Rust is not a like-for-like latency comparison | results doc labels INFERRED/REPORTED accordingly |

## 4. Null handling

- If Harness² run fails on infra (Python/deps), root-cause, receipt, and re-run once; second failure = bench BLOCKED (owner: environment), RG-AH result still filed alone.
- If RG-AH longtask test fails, the failing assertion names the falsified prediction (P-04..P-07); no silent re-baselining — fix code or append correction here.
- Synthetic-quality caveat stands for BOTH sides: Harness² demo proposer is scripted; RG-AH judges are deterministic-synthetic. No quality-superiority claim is licensed by this bench. Throughput/waste/ledger/doctrine behavior only.

## 5. Bars (PROPOSED; Stewart sets)

- Step band compliance: every counted task lands 30–50 steps. Out-of-band = bench invalid, re-scope.
- Safety: zero kernel violations, zero sandbox bypasses, zero egress anomalies across all runs (hard bar, both sides).
- Receipt completeness: every executed step has a receipt or ledger entry (RG-AH) / run-dir + summary entry (Harness²).

## 6. Dated corrections

_(none at filing)_
