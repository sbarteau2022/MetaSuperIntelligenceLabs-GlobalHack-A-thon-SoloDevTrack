# Pre-Registration — Closure Items P0-1..P0-5 + Improvements (2026-11-01)

**Status.** PROPOSED (this file). Predictions recorded BEFORE the run. Judges below.
**Scope.** `research-grade-harness` only. The plugin server
(`hypermind-nested-kv/src/main.rs`, separate crate) is out of scope — read, never
modified. Model server on :8000/:8001 is DOWN (VERIFIED this session: curl exit 7 /
000); PPC re-runs are therefore predicted VOID, not FAIL.

## 0. Verified ground truth (measured this session, before any edit).

**CORRECTION 2026-11-01 (replaces the live-gate env in all commands below):**
the plugin server's working tree changed mid-session: unset
`HYPERMIND_SURREAL` now resolves to `Shadow` → hard startup rejection
(`shadow startup rejected`, read from `hypermind-nested-kv/src/main.rs:5351`).
All live invocations MUST export `HYPERMIND_SURREAL=off` (isolated-test mode).
This is a server-crate behavior change, out of scope — not modified here.

- `cargo test --all-targets`: 35 lib + 1 a14 + 10 build_345 + 5 engine_n3 (3 ignored)
  - 8 implementation + 10 pressure = 69 passed, 0 failed.
- Live N3 gate: 8/8 with `hypermind-nested-kv/target/debug/hypermind-mcp`.
- Live plugin gate: 1/1 with the same binary (`map.kv_fetch` exists there).
- `cargo clippy --all-targets`: clean. `cargo fmt --check`: 1 trailing blank line
  in `src/lib.rs:99`.
- `rgah run` exit code: 1 (fail-closed VERIFIED — earlier `EXIT:0` was `head`
  pipe-masking; corrected 2026-11-01, replaces that reading).
- `rgah resume` with 2 fabricated PASS lines: accepted=2, exit 0.
  **Forgery acceptance VERIFIED — P0-1 reproduces.**
- Standalone `hyperbolic-map-nested-kv-plugin/` repo: ABSENT (was listed earlier
  in-session, now gone; `find` confirms only `Dynanic-Hyperbolic-Neural-Graph`
  matches). README's plugin path is therefore stale — P1-7.
- Server concurrency: `run_stdio` = single-threaded `for line in stdin` loop over
  one `Arc<Mutex<State>>`; `run_http` = serial accept loop. One child instance
  is inherently serial on both transports (read from source, lines 5108+, 7192+).
- `map.kv_store` writes to in-memory BTreeMap, snapshotted via `map.snapshot`;
  separate `HYPERMIND_ROOT`s = separate namespaces (read from source).
- Docker: `HYPERMIND_SURREAL` paths exist but `map.db_status` reports surreal
  disabled by default; NOT-APPLICABLE prediction below stands regardless.

## 1. Closure predictions (falsifiable)

| #      | Change                                                                                                                                                                                                                                                | Prediction (judge)                                                                                                                                                                                                                            |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P0-1/2 | Receipt chain-hash + exact N3 digest binding; `resume` verifies chain, rejects forgeries, exit≠0 on any reject                                                                                                                                        | Forged-prior test: accepted=0, exit≠0. Genuine-prior flow: `rgah run --live` writes chained receipts; `rgah resume` accepts them. `timestamp_ms` ≠ 0 on all new receipts.                                                                     |
| P0-3   | `rgah run/resume` gain `--live` path: plugin binary + root + gate manifest → `run_sequential_n3`/`run_parallel_n3`. No flag = current fail-closed behavior, unchanged                                                                                 | `--live` without binary: exit≠0 naming the missing env/binary. `--live` with binary: geometry mission executes against live plugin; receipts carry exact digests. Default (no flag): exit 1, zero PASS — existing tests unchanged.            |
| P0-4a  | Bounded thread pool over ready packets; one child + one sink per worker, own `HYPERMIND_ROOT`; merge in packet order; budget charged under a mutex                                                                                                    | Daemon-DAG live test still 5/5 (shared-state test keeps the one-child path). New hermetic test: N independent packets, K workers, completion = all packets, order-stable merge. Wall-time gate: NOT claimed (CI variance) — correctness only. |
| P0-4b  | Material-change retry: transport-failure retry respawns the child (fresh execution conditions) exactly once; judge-fail retry escalates (identical re-send forbidden)                                                                                 | Hermetic test: transport retry uses a fresh child (spawn counter = 2, no third attempt). Judge-fail path: escalate after first attempt, no identical retry — existing `gate_denial` test shape preserved.                                     |
| P0-5   | Typed `json` module: strict parser (no `Value`, no float coercion, depth cap, exact escapes incl. `\/` + `\uXXXX`), `Writer` with key-order control; `mcp_client` uses it; child stderr captured to a bounded buffer and surfaced on transport errors | Fuzz fixture: malformed envelopes (truncated, bad escapes, depth overflow, duplicate keys rejected) all `Err`, zero panics. Existing live tests still 8/8. `has_receipt_sha256` keeps its 64-hex validation.                                  |
| P1-7a  | LICENSE (Apache-2.0 text, author MetaSuperIntelligenceLabs) added                                                                                                                                                                                     | `cargo package --list`-equivalent: file present at crate root.                                                                                                                                                                                |
| P1-7b  | README: plugin path corrected to `hypermind-nested-kv`, stale `live-n3` wording updated, fmt fixed                                                                                                                                                    | `cargo fmt --check` exit 0.                                                                                                                                                                                                                   |
| P1-6   | PPC re-run                                                                                                                                                                                                                                            | VOID (no model server). Receipt files untouched. Staleness recorded, not repaired.                                                                                                                                                            |

## 2. Explicit NON-predictions (out of scope, will NOT happen this pass)

- No server modifications. No model-server restoration. No energy recalibration
  (surrogate stays labeled surrogate). No provenance rewrite beyond accurate
  path/naming fixes (clean-room rewrite of the design lineage is a docs-track
  decision for Stewart, not a code patch).
- No wall-clock speedup claims. No new benchmark numbers of any kind.
- No serde_json dependency (doctrine upheld; offline crates.io 403 VERIFIED).

## 3. Improvement pass (after gates green — structural, no behavior change)

1. `engine.rs` split: `execute_packet_n3` (~140 lines) extracted to
   `src/execute.rs` with its helpers; `run_*` orchestrators stay. No signature
   changes.
2. `README.md` evidence boundary rewritten to match post-closure behavior.
3. Adversarial fixtures: forged-chain resume tests (3: bad prev_hash, edited
   outcome, truncated file) — all must exit≠0 with zero accepted.

## 2026-11-01 swarm-gated results

(i) Registry audit: 16/23 live:true FAIL, flipped to live:false (7 verified live remain).
(ii) Receipt review: 4 CLEAR + 1 HOLE fixed (tampered-chain test now flips chain= digit, helper asserts flip happened).
(iii) JSON review: 4 CLEAR + 1 BUG fixed (1e999/giant-int now Err(BadNumber), added to rejection list).
(iv) receipt_adversarial.rs verified EXISTS, 10/10 format fields match, TRACE-CLEAR.
(v) End-to-end binary proof: forged resume exit 1 accepted=0, tamper exit 1 chain_broken=1.
(vi) SurrealDB deferred per operator (map untouched until deterministic core certifies).
