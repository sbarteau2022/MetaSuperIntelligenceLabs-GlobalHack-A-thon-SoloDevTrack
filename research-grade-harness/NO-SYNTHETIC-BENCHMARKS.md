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

**Enforcement update (2026-10-31).**

- `Engine::run_sequential` and `Engine::run_parallel` now fail closed and can
  never mint PASS receipts. Live acceptance uses the N3 executor methods.
- Packets with no admitted `map.*` output fail instead of receiving a marker.
- The synthetic CLI benchmark is disabled unless the explicit
  `synthetic-scaffold` feature is compiled, and still labels itself scaffold.
- The historical long-task test is both feature-gated and ignored.
- `cargo test --features live-n3 --test engine_n3` promotes the three N3 tests
  from ignored checks to mandatory live gates; it requires `HYPERMIND_MCP_BIN`.
