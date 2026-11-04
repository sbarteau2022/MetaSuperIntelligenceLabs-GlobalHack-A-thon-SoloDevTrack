# Glimmer / GlobalMind / harness handoff — 2026-11-04

## Scope and location

All implementation and receipts are in the canonical SoloDevTrack Git repo:
`/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack`.
This is a component-qualified build, **not** certification of the requested
model–harness–HyperMind/KV-MCP automaton loop. Do not use a past mission PASS
as proof of a Glimmer forward. The source tree is dirty; `core_lm/`, the
recovered `Ares WebSite : App/backend/`, and several receipts are untracked.
No broad clean/reset/commit is safe because other agents' site and runtime
changes are present.

Live listeners at last inspection: persistent SurrealDB 8000 (PID 1709), old
Glimmer server 8011 (PID 1865), old backend 4810 (PID 2709). Leave all three
untouched. The isolated test server on 18111 was stopped and the port cleared.
The old backend binary still carries the previously found fake model receipt.

## What is real now

1. `core_lm/src/bin/serve_glimmer.rs` adds `POST /v1/forward-exact` with a
   bounded **binary GFX2/GFO2** token protocol. It executes real frozen
   Glimmer weights with `forward_with_backend(ExactKvCache)`, returns actual
   generated token IDs, final f32 logits digest, exact K/V digest covering
   every layer/head page, and streamed hashes of the index, weight shards,
   tokenizer, and config. Artifact hashes are checked before and after model
   load. `core_lm/src/native_exact_wire.rs` verifies frame bounds, receipt,
   and artifact composite. `core_lm/src/artifact_digest.rs` defines the hash
   contract; `core_lm/src/bin/smoke_exact_wire.rs` is the Rust hardware probe.
   The two identical real requests returned token 13796, identical logits
   and K/V digests, and a client-side independent artifact hash match. Full
   values and limitations: `native-exact-binary-smoke-2026-11-04.md`.
2. `globalmind-runtime/src/store.rs` now hydrates ranked inline atom text
   only if workspace, atom ID, stored digest, actual SHA-256, and 16 KiB bound
   match. Both `mind.recall` and `mind.recall_scoped` in
   `src/bin/globalmind_mcp.rs` include that verified text in MCP **control**
   output; URI-only atoms remain metadata-only. MCP JSON does not enter model
   I/O. The real isolated SurrealKV restart test
   `tests/glimmer_native_full_pass.rs` passed with the exact prefix text,
   104 K/V pages, 52-layer bitwise continuation, argmax 13796, and a
   digest-mismatched adversarial atom refused by MCP. This witness was
   manually created for the component test, not admitted cross-model knowledge.
3. Recovered backend source in `Ares WebSite : App/backend/` rejects fake
   bound calls from MCP-only and native text-stream executors. Bound receipts
   now require an exact K/V digest, valid input-byte and chain hashes, and at
   least one tool receipt. Unsupported and undeclared tool calls fail packets.
   The live 4810 process was **not** rebuilt or replaced.
4. `core_lm/src/native_token_wire.rs` defines bounded, receipted binary
   GTK2/GTO2 semantic-text tokenization frames. This is **source-only**:
   no server route, backend consumer, or hardware run yet. The backend must
   not link Glimmer's tokenizer or send MCP JSON across the model boundary.

## Verification and honest limits

- `core_lm`: `cargo check --tests` passed after the token wire edit;
  `cargo test --lib` passed 38 tests before that final module. The new token
  frame regression passed separately; it has not yet received a hardware run.
  Strict all-target Clippy fails
  on dozens of existing candidate warnings, not a release gate pass.
- `globalmind-runtime`: `cargo check --tests` passed; `cargo test --lib`
  passed 104 with 3 ignored; the explicit ignored **real hardware**
  Glimmer/MCP/SurrealKV test passed after the hydration negative control.
- Recovered backend: `cargo test --all-targets`, strict Clippy, and format
  check passed in this pass. Ignored live tests do not count as certification.
- Independent MLX oracle still finds Rust full-logit numerical divergence;
  matching argmax does not prove checkpoint parity. See
  `independent-mlx-oracle-2026-11-04.md`.
- The new binary server returns a K/V **digest**, not persisted pages. The
  separate isolated component test proves persistence/restart, but the two
  are not yet one server/harness invocation.

## Critical blockers to full certification

1. **Replay atoms are not actionable knowledge.** In
   `globalmind-runtime/src/replay/mod.rs::build_atoms`, rank-0 text is the
   placeholder `replayed atom <digest-prefix>` while `content_digest` is a
   different teacher-neutral input digest. The new recall hydration refuses
   this mismatch. `extract/mod.rs::atom_key` also includes kind, dtype,
   shape, layer, and head, so cross-model coalescence is not proven. Do not
   label those atoms model-agnostic or use the current replay PASS for that
   claim. Require a real source-grounded semantic content carrier, exact
   digest match at admission/write, immutable origins, and cross-model task
   evidence; rerun distillation after the fix.
2. **No single live mission execution.** `/api/harness/mission/live` in the
   recovered backend uses MCP-only `MindRecallExecutor`; its bound call
   correctly refuses. `GlimmerHttpSeat::call_bound` remains a stub. The old
   4810 binary is misleading and must not be used for certification.
3. **No automaton MTP/speculative loop in the mission path.** Retrieval,
   prefill, MTP, and drafter proposer types exist, but mission dispatch does
   not execute one proposal–verify–atomic commit/rollback loop or update
   Q32.31 confidence from independently judged prediction error.
4. **No 30–50-step governed run** with real Glimmer tool calls, output-quality
   audit, K/V reuse, rollback evidence, and cost measurements.

## Next work in order

1. Finish the binary tokenization server route using `native_token_wire.rs`;
   call the model-owned tokenizer, add BOS according to the request, bind the
   pinned tokenizer digest, and pressure-test malformed/truncated frames and
   a real checkpoint request. Update the coreLM README.
2. Build one backend executor that calls MCP recall, parses/validates the
   control response, sends **only verified content text** to binary tokenize,
   then sends token IDs to binary exact forward. Pin all four artifact hashes
   and bind MCP tool receipt, native token receipt, native forward receipt,
   K/V digest, and exact input bytes in one `BoundOutputReceipt`. Do not use
   the old separate generate-plus-forward path. Keep the old live processes
   untouched; smoke on an isolated port first.
3. Join exact K/V page persistence and MCP exact hydration to that executor.
   Compare uninterrupted versus restart continuation and reject changed
   model, shape, TPPE/address contract, or page digest. A digest-only response
   is insufficient to claim nested K/V reuse.
4. Repair rank-0 distillation content and model-neutral identity. Stop
   placeholder atom writes; use source-grounded content and immutable
   provenance; prove cross-model recall on a held-out task before promotion.
5. Connect the single Meta Automaton to the mission path. MTP head and
   speculative drafter are proposer plugins under one deterministic governor;
   verify each proposal against actual model outcomes, update Q32.31
   confidence/prediction error only from independent receipts, and compare
   with a no-speculation control.
6. Only then run 30–50 real steps through the rebuilt Rust backend with
   admitted map/mind tools. Audit every declared call, output correctness,
   atom provenance, exact K/V hit/miss, acceptance/rollback, latency, token
   cost, and receipt-chain integrity. Fail on missing evidence. Commit only
   reviewed source and receipts; never add `target/`, binaries, logs, or
   unrelated UI work.

## Reproduction commands

From `core_lm/`: `cargo check --tests`; `cargo test --lib`;
`cargo build --release --bin serve_glimmer --bin smoke_exact_wire`.
Check `lsof -nP -iTCP:18111 -sTCP:LISTEN` before an isolated launch.
Run `./target/release/serve_glimmer 127.0.0.1:18111 --model-dir
/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX`, then
`./target/release/smoke_exact_wire 127.0.0.1:18111
/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX`; stop only this
isolated process afterward. The current CLI still tokenizes locally until
step 1 is complete.

From `globalmind-runtime/`: `cargo build --bin globalmind-mcp`, then
`cargo test --release --test glimmer_native_full_pass
glimmer_native_mcp_kv_full_pass -- --ignored --nocapture`.
This test starts and stops its own disposable SurrealKV, never port 8000.

From recovered `Ares WebSite : App/backend/`, use
`CARGO_TARGET_DIR=<canonical-repo>/globalmind-runtime/target/ares-backend
cargo check --tests` because the checkout path contains a colon. Run
`cargo test --all-targets` with the same target dir. Do not restart 4810.

## Governing status

Session model: frozen local Muse Glimmer via `core_lm`; native MTP on 8010
is a separate seat. Tools used: Rust/Cargo, isolated SurrealKV, MCP child,
read-only process/port inspection, Git. Open tasks are the six items above.
Runbooks: root `AGENTS.md`, backend `MODEL-SEAT-STATUS-DESIGN.md`,
`globalmind-runtime/EXACT-GLIMMER-KV-DESIGN.md`, and the
`source-command-harness-status` skill. No goal-complete claim is justified.
