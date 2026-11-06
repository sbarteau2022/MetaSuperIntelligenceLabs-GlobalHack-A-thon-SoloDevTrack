# Handoff to Claude Code: automaton loop closure (2026-11-05)

Read `AGENTS.md` first. Its rules apply: no JSON or Python across the model I/O boundary, no synthetic gate passes, cargo gates on every edit, `git status` proof before "done".

Branch: `claude/automaton-loop-closure`, based on `codex/consolidate-n3-globalmind-20261102` (26ce312).
Crate: `Ares WebSite : App/backend`.

## Why

The Meta-Superposition paper (`~/Desktop/Atoma-Papers/meta-superposition-v2.tex`) claims one automaton holds weighted proposals, re-ranks them by earned confidence, and closes a loop. An audit of the code found the loop was not closed:

1. `rerank_strategies` (paper Proof 3) did not exist in this branch.
2. A strategy below its floor was never run, so it was never judged again and could not recover (absorbing state).
3. `StrategyState::digest()` omitted `confidence_floor`, so floor changes were invisible in receipts.
4. Nothing ever held more than one proposal. The native MTP policy has `max proposers 1`.
5. The 0.9649 acceptance in the paper is from the native MTP seat (:8010), which is not wired into the automaton-gated loop. The gated prompt-lookup drafter accepted 2 of 32 on Glimmer.

## What changed (3 files, +628/-26)

- `src/harness/automaton.rs`: `rerank_strategies` (confidence desc, `strategy_id` asc, duplicate id fails closed), `StrategyState::digest_v2()` (binds the floor; the transition and both update functions now use it), `probe_admit` (automaton-authorized bounded probe past the floor only; never past open dependencies, ambiguity, or a Reject).
- `src/harness/speculative.rs`: `Drafter` trait, `HeldProposer`, `run_speculative_held` (hold, re-rank, run top, update only the strategy that ran). `run_speculative` is now a one-proposer wrapper. `SpecConfig` gained `probe_every` and `probe_width`. `RoundReceipt` gained `strategy_id`, `held` and `probe`. `SpecRun` gained `confidence_sql`.
- `tests/speculative_live.rs`: config updated, plus the new ignored hardware gate `held_loop_reranks_two_real_strategies_and_equals_control`.

## Verified (cloud container, no hardware)

`cargo check --tests`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`: clean.
`cargo test --all-targets`: 89 passed, 0 failed, 18 ignored (hardware).
The unit tests use a table stand-in model, so they prove control flow only and are not a hardware pass. Examples: a liar at 0.9 loses rank after 4 rounds and the honest 0.6 strategy takes over; a strategy at 0.05 recovers in 2 probes; one success from 0.6 gives raw `1374389534`.

## NOT verified (needs this Mac)

Start the isolated Glimmer server with `/v1/verify-exact` (NOT :8011 or :8000), then:

```
export ARES_GLIMMER_NATIVE_URL=http://127.0.0.1:<port>
export ARES_GLIMMER_WEIGHTS_SHA256=<pinned weights sha256>
cd "Ares WebSite : App/backend"
cargo test --release --test speculative_live -- --ignored --nocapture
```

Pass criteria: output equals the control arm's greedy tokens; an off-topic strategy started at 0.9 loses rank from verified rejections; the on-topic strategy runs. Paste the per-round trace.

## Next, in order

1. Run the hardware gate above and record the trace.
2. Wire the native MTP seat (:8010) as a second `Drafter`. This needs a verify path for MTP proposals, because its accept loop currently runs server-side. Then re-measure acceptance together with proposals per target call and tokens per target call (3.05 and 2.12 in the 1965/645 and 1368/645 bench).
3. Call `run_speculative_held` from `dispatch_mission_live` and persist `SpecRun.confidence_sql` through the governed client. Create the row first, because UPDATE on a missing Surreal row is a no-op.
4. Bump `policy_version` for the `digest_v2` change. Receipts written before it are not comparable.
5. The paper cites `ares-backend-loop/src/harness/automaton.rs` and `math_gate.rs`, which are not in this repo. Reconcile that version against this one on the Mac.
6. Add a Limits section to the paper (v2) with the five findings above, each tied to a receipt.

## Still open from the audit

- The MCP route passes `verified=false`, so confidence never moves there. That is by design, but it means the cycle is not closed on that route.
- The governed mind call failed attestation in the paper's measured run.
- The 2000-token closed-loop run versus 1965 committed is not reconciled (paper v2, Proof 5 Step 10). Step 9 is resolved: an exact replay and a Rust test confirm the starting confidence was 0.6.
