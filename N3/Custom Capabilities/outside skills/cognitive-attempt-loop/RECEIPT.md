# RECEIPT — cognitive-attempt-loop (Agent C2 final round, 2026-10-31)

Finishes Agent 5's unfinished attempt_loop migration as a compiled state machine
that REFERENCES (path-depends on) the Slice 2 source. No TS shim: the machine is
authored directly in Rust, so the tool-write TS chain is N/A (recorded below).

## Gate booleans

| Gate | Boolean | Verdict source |
|---|---|---|
| `ts_strict` | N/A (no TS shim authored; pure-Rust crate) | no .ts files in crate — VERIFIED |
| `rust_port_supported` | N/A (no TS->Rust port; direct Rust authorship) | same |
| `rust_compile` (`cargo check`, `-D warnings`) | **true** | exit 0 paste §1 |
| `cargo clippy --all-targets` (`-D warnings`) | **true** | exit 0 paste §2 (after 1 fix, dated below) |
| `cargo test` (`-D warnings`) | **true** | 8/8 pass paste §3 |
| rust-analyzer zero errors | **true** | scan exit 0 paste §5 |
| dummy packet TERMINATED w/ iter proof | **true** | 4/4 TERMINATED paste §4 |
| `sent` | **true** | all applicable gates true |

## Correction entry (2026-10-31, per claims discipline: entries, not edits)

- Clippy first run FAILED with `clippy::assertions_on_constants` on a unit test
  (`assert!(MAX_ITERS <= 8)`). Fix18798 attempted: `const { assert!(...) }` —
  REJECTED by rustc 1.99.0 (E0658, `const_block_items` experimental). Final fix:
  kept the runtime test with `#[allow(clippy::assertions_on_constants)]`.
  Clippy re-run: exit 0. The bound itself (`MAX_ITERS: u8 = 8 <= 8`) was never in
  doubt; only the assertion style changed.

## Machine reference

- States: `MachineState::{Plan, Run, Judge, Receipt, Escalated, Terminated}`
  (`src/lib.rs:25`); `TerminalOutcome::{AssessedClean, Caught, Missed,
  FalsePositive, Unresolved, Disputed, Escalated}` (`src/lib.rs:42`).
- Bound: `MAX_ITERS: u8 = 8` (`src/lib.rs:17`); over-budget `plan` returns
  `MachineError::IterBudgetExhausted`. No path loops without `bump_iter`.
- Escalation (in code, `src/lib.rs:343`): each `Unresolved` increments `strikes`;
  `strikes >= STRIKE_LIMIT (2)` transitions Judge -> Escalated and returns
  `TerminalOutcome::Escalated`. First strike returns to Plan for bounded retry.
- Receipt per transition: `TransitionReceipt::post_run` (`src/lib.rs:110`)
  REQUIRES `job_id / env_digest / input_hash / test_bundle_hash / exit_status`
  plus `grade: Option<GradeSummary>` (integer v/j/q + FN/FP/fully_clean flags).
  Every post-run transition in `judge` uses `post_run`; TERMINATED count is
  asserted per packet in §4.
- Preserved Slice 2 semantics (delegated, not reimplemented): commit_prediction
  hash-sealed pre-reveal -> reveal (event-before-content) -> submit immutable
  finalized artifact (I03 idempotent retry) -> grade_and_admit double-run
  flakiness gate -> dispute clears active evidence; `Runner` trait
  (`LocalProcessRunner`, `FlakySecondRun`) passes through untouched.
- Source untouched: sha256 of
  `cognitive-map/slice2/src/attempt_loop.rs` re-verified after build as
  `b2475cf131e7ce9defa1447c7bad2f066e5b3e5b57ec19d03292feb846bdfc23` (paste §6).
  Defer-deletion with dependent list: `DEFER-DELETION.md` (integration/src/
  tools.rs:18, cogmap_tools_v1.rs:19, integration + slice3 + slice4 Cargo.tomls).
  No B1/B2 files touched (no writes outside `skills/cognitive-attempt-loop/`).

## N3 paths (all VERIFIED present)

- `/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop/Cargo.toml`
- `/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop/src/lib.rs` (449 lines)
- `/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop/tests/dummy_packet.rs`
- `/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop/DEFER-DELETION.md`
- `/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop/RECEIPT.md` (this file)

## §1 PASTE — cargo check (RUSTFLAGS="-D warnings", --all-targets)

```text
Locking 2 packages to highest compatible versions
    Checking cogmap-slice1 v0.1.0 (/Users/stewartbarteau/Desktop/N3/cognitive-map/slice1)
    Checking cogmap-slice2 v0.1.0 (/Users/stewartbarteau/Desktop/N3/cognitive-map/slice2)
    Checking cognitive-attempt-loop v0.1.0 (/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s
```

## §2 PASTE — cargo clippy final (RUSTFLAGS="-D warnings", --all-targets)

```text
    Checking cognitive-attempt-loop v0.1.0 (/Users/stewartbarteau/Desktop/N3/skills/cognitive-attempt-loop)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
```

## §3 PASTE — cargo test (RUSTFLAGS="-D warnings")

```text
     Running unittests src/lib.rs (target/debug/deps/cognitive_attempt_loop-5b75dbffa4fe2ee3)
running 4 tests
test unit::max_iters_bound_holds ... ok
test unit::strike_limit_is_two ... ok
test unit::iter_budget_exhausts ... ok
test unit::bad_transition_rejected ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/dummy_packet.rs (target/debug/deps/dummy_packet-8e8916a020403ca5)
running 4 tests
test packet_dispute_clears_evidence_terminated ... ok
test packet_clean_terminates_assessed_clean ... ok
test packet_defective_terminates_caught ... ok
test packet_flaky_two_strikes_escalate_terminated ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
   Doc-tests cognitive_attempt_loop
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## §4 PASTE — termination log (cargo test -- --nocapture, eprintln packet lines)

```text
PACKET defective: TERMINATED outcome=Caught iter=1 receipts=4 strikes=0
PACKET clean: TERMINATED outcome=AssessedClean iter=1 receipts=4 strikes=0
PACKET dispute: TERMINATED outcome=Disputed iter=1 receipts=5
PACKET flaky: TERMINATED-ESCALATED first=Unresolved second=Escalated iter=2 receipts=6 strikes=2
```

Iter proof: every packet `iter <= MAX_ITERS (8)`; clean/defective/dispute
terminate in 1 iteration, flaky escalates after exactly 2 strikes / 2 iterations.
Post-run receipts carry full runner evidence (asserted: job_id, env_digest,
input_hash, test_bundle_hash, exit_status, grade v/j/q + flags).

## §5 PASTE — rust-analyzer diagnostics (binary 1.98.1, exit=0)

```text
diagnostic scan complete / exit=0; only WeakWarning Ra("inactive-code") on
#[cfg(test)] modules (lib.rs:402-453 unit tests; slice2 hash.rs / attempt_loop.rs
/ slice1 cogmap.rs test modules) — expected, not errors.
```

## §6 PASTE — source-untouched re-verification (post-build)

```text
b2475cf131e7ce9defa1447c7bad2f066e5b3e5b57ec19d03292feb846bdfc23  attempt_loop.rs
880 lines
```

Toolchain: rustc/cargo 1.99.0 (Homebrew), rust-analyzer 1.98.1, python3 3.15.2
(runner interpreter). No TS shim: TS gates N/A, honestly labeled.
