# CALIBRATION — hypermind-toy (T-013)

Date: 2026-11-02. Status: PROPOSED → IN PROGRESS.

- **BRIEF.** Build a solo toy that mirrors the backend's governed RAG+memory
  path byte-for-byte, so the scheduler/framework work in
  HYPERBOLIC-SCHEDULER-FRAMEWORK-2026-11-02.md can be exercised without the
  live backend. When the backend is ready, port by diffing, not rewriting.
- **RISK.** Medium-low. New crate, no shared state with the backend. Big risk
  is drift: toy diverges from backend semantics and the "rehearsal" teaches
  wrong lessons. Mitigated by MIRROR.md (§5: same struct names, same chain
  formulas, port checklist) and byte-level wire-shape tests.
- **BLAST-RADIUS.** `backend/hypermind-toy/` only. No edits to `src/`,
  `tests/`, `Cargo.toml`, docs, or TODO outside T-013. Runs on its own port
  nothing (library-only toy; no server, no ports, no stores).
- **CONTRACT-FIRST.** Sources read before code: `src/harness/runtime.rs`
  (L1–175: CapabilityGate, Executor, CallOutcome, Engine::judge_output),
  `src/harness/mission.rs` (chain_hash L295–321, ReceiptLog::append/verify
  L328–406, EnergyBudget L415–441, Block/WorkingSetCache L686–809,
  research_gate L639–680, geometry_mission L847–895),
  `tests/harness_core.rs` (acceptance bar), `Cargo.toml` (dep versions),
  `hypermind-nested-kv` VP-tree + shell projection + depth-gate cost form
  (read-only reference, re-implemented small). No other sources admitted.

Judges for this slice (delivery-runbook step 7 = checkers here):
`cargo test` (all pass), `cargo clippy --all-targets -- -D warnings` (clean),
`cargo fmt --check` (clean). Output pasted in §7 of the design doc.
