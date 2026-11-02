# RECEIPT — Agent B1 (Swarm), Wave 3-FINAL

Date: 2026-10-31. Slice (disjoint): swarm logic → compiled DAG ONLY.
B2's decomposition/prereg files NOT touched — never listed, never opened.

Read-only refs (NOT emitted, never modified): orchestrator `src/swarm/mod.rs`
(Governor: `RestartStrategy::OneForOne/OneForAll`, `RestartIntensity`,
`ProcessState::EscalatedIntensityExceeded`), `src/routing/mod.rs` (typed handoff
edges), `schema/carved_swarm.fbs` (FlatHandoffPacket wire shape),
`SUBAGENT_SWARM_LOGIC.md` (node order + disjoint ownership), `design/
CARVED_SWARM_SPEC.md`, harness `tier4-carved-swarm-immune.md`, `N3/...
subagents/compiled/*.rs` (A5/A6, called by `#[path]`, never duplicated),
`N3/.../workflows/compiled/ok_computer_loop.rs` (bounds mirrored: 3/2/4).

## 1. What was built (N3 only, never /skills)

| Artifact | Path | Size |
|---|---|---|
| DAG source | `N3/web-development/swarm/compiled/swarm.rs` | 1371 lines (after 2 fixes) |
| Crate manifest | `N3/web-development/swarm/compiled/Cargo.toml` | package `kimi_swarm`, bin `swarm` |
| Node/edge/judge registry | `N3/web-development/swarm/compiled/swarm.json` | 4250 bytes, JSON-valid |
| This receipt | `N3/web-development/swarm/compiled/RECEIPT.md` | — |

## 2. Nodes / edges / judges

Nodes (7, `NODE_ORDER` const, topological): Explorer → Architect →
SurfaceCoder → GraphCoder → WorkbenchCoder → Critic → Governor.
Coders run sequentially in NODE_ORDER with disjoint `OWNS` (no file claimed
twice — proven by `coders_disjoint_ownership_pass` test).

Edges (typed artifacts): `SwarmPacket` → `ExplorerBrief` → `ArchitectOutput`
(`contracts_ready` gate ×3) → `SurfaceCoderOutput` + `GraphCoderOutput` +
`WorkbenchCoderOutput` (files[] + `CriticInput` evidence) → `CriticOutput` →
round `NodeReceipt`s → `GovernorPermit` → `SwarmReceipt` terminal.

Judges (7, one per node, every judge returns `NodeReceipt{pass,detail,strikes,state}`):
`judge_explorer` (ReadOnlyGate+BriefBudget≤60+HandoffComplete),
`judge_architect` (ContractsGate: non-empty, all inside OWNS),
`judge_surface/graph/workbench` (OwnershipGate: every file passes that agent's
`may_write`), `judge_critic` (Pass verdict AND no `needs_layer_b_reentry`),
`judge_governor` (ImmuneGate: ALL upstream receipts pass).

Swarm API (reusable, no prompts): `Swarm::spawn(strategy, intensity)` →
`swarm.run(&packet, &mut evidence)` → `Swarm::collect(&receipt)`.
Bounds: `MAX_ROUNDS=3`, `MAX_STRIKES=2`, `MAX_NODE_ATTEMPTS_PER_ROUND=4`
(mirrored from `ok_computer_loop.rs`). Two-strike escalation in code:
2 consecutive failures at the SAME node → one re-entry (OneForOne) or
whole-round restart (OneForAll, live variant) → terminal `Escalated`;
`RestartIntensity{max_restarts:2, within_ticks:3}` caps restart history;
`ProcessState::EscalatedIntensityExceeded` is the Governor-judge fail state.

Reuse proof: `#[allow(dead_code)] #[path=...]` import of all six A5/A6 files
verified by pre-test (`rustc --edition 2021` on real file: fails without the
allow, exit 0 with it); all 48 inherited A5/A6 tests pass inside this crate.

## 3. Checker outputs (pasted, not summarized)

`cargo check --all-targets` (exit 0, `#![deny(warnings)]` in source):

    Checking kimi_swarm v0.1.0 (.../swarm/compiled)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s

`RUSTFLAGS="--deny warnings" cargo check --all-targets --message-format short`
(exit 0, zero diagnostics emitted).

`cargo clippy --all-targets -- -D warnings` (exit 0):

    Checking kimi_swarm v0.1.0 (.../swarm/compiled)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s

`cargo test` (56 passed, 0 failed — 8 swarm + 48 inherited A5/A6):

    test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

rust-analyzer MCP (`rust_analyzer_diagnostics` via bridge): FAILED TO EXECUTE
(bridge error `Missing file_path` on all param spellings — server-side schema
mismatch, retried 3×). Fallback: native `diagnostics` refreshed — covers only
the `skills/` workspace (0 entries under N3; pre-existing `skills/` warnings
NOT mine, NOT touched). File-level zero-warning proof stands on
`#![deny(warnings)]` + `RUSTFLAGS=--deny warnings` + `clippy -D warnings`, all exit 0.

## 4. Dummy packet runs (TERMINATED both directions, pasted)

Clean (`AlwaysClean`): all 7 nodes pass, governor permits=6,
`final receipt: Passed(rounds=1) passes=7 fails=0` → `TERMINATED: PASS after 1 round(s)`.

Dirty (`AlwaysDirty`): coders pass, critic fails attempt 1/4 and 2/4,
`two-strike rule: critic re-entered ONCE (OneForOne), still failing` →
`TERMINATED: ESCALATED node=Some(Critic) rounds=1 strikes=2
reason='two-strike re-entry failed at critic'`.
Bound honored: escalation at rounds=1 ≤ MAX_ROUNDS=3; `main` asserts both
terminals (Passed for clean, Escalated for dirty) — asserts held.

Full log in §3 run output above (`cargo run`, both packets + counters
`evidence_calls=1` each + `disposed clean: no leak` via `debug_assert!(leak_check())`
on every node path in dev/test profiles).

## 5. Fixes during validation (corrections, not silent edits)

- 2026-10-31 `unused_assignments` ×2 (critic-pass branch): removed dead
  `strikes = 0; failing = None;` — receipt field `ok.strikes = 0` kept (live).
- 2026-10-31 `unused_assignments` ×2 (governor-pass branch): removed dead
  `strikes = 0; failing = None;` before `return` (locals die at return;
  terminal state carried by `SwarmReceipt`).

## 6. Booleans

cargo_check_exit_0: true. clippy_D_warnings_exit_0: true.
cargo_test_56_pass: true. deny_warnings_rustflags_exit_0: true.
dummy_clean_TERMINATED_Pass: true. dummy_dirty_TERMINATED_Escalated: true.
bound_MAX_ROUNDS_3_honored: true. two_strike_escalation_in_code: true.
rust_analyzer_mcp_executed: FALSE (bridge schema error; fallback recorded).
ts_check/ts_to_rust: N/A (pure-Rust DAG, no TS surface — recorded, not skipped).
sent: true (all applicable gates true).

## 7. NOT-checked

rust-analyzer MCP diagnostics on `swarm.rs` (bridge refused; see §3).
Prod evidence wiring (DevTools MCP handle) — dummy `EvidenceSource` only.
`OneForAll` full-round-restart path beyond `escalation_bounded_by_max_rounds`
test (strategy discriminant live, restart logged, intensity-capped).
