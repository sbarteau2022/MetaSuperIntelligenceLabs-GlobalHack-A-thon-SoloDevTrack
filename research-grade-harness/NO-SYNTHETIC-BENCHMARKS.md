# STANDING DOCTRINE — No Synthetic Benchmarks (Stewart, 2026-10-28)

**Rule.** No synthetic benchmarks, ever. Scripted proposers, marker-heuristic
scoring, arithmetic-surrogate baselines, and deterministic-synthetic judges
produce data that means nearly nothing and force the work to be redone.

**Consequence for this workstream (RG-AH).**

- `src/bench.rs` synthetic baselines (MadMind/Harness2/Cordis/Rgah arithmetic)
  and `tests/longtask_bench.rs` synthetic judges are **scaffolding only**.
- They license NO quality, waste, or energy claim. The 2026-10-28 long-task
  head-to-head numbers (Harness² 37 steps/task; RG-AH 41 execs, 39/1/0) stand
  as **throughput/ledger-shape observations only**, downgraded from any
  comparative reading.
- Gates before any bench number is cited: real-model runs (Harness² paper
  baseline class: GLM-5, Gemini 3.5 Flash — REPORTED, not reproduced) +
  watt-meter calibration of the Landauer surrogate.
- Any future bench file that ships synthetic scores as results is a FAIL on
  audit, not a caveat. Pre-register real-model runs first.
