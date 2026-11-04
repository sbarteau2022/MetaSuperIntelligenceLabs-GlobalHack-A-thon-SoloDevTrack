# Harness status — 2026-11-04

Session model: Glimmer is served separately on local port 8011; the recovered
backend mission source currently uses an MCP-only GlobalMind executor. A
separate native MTP server on port 8010 is not a Glimmer forward proof.

Tools used this pass: Rust source review, Cargo check/test/clippy/fmt, Git
status, and the source-command-harness-status runbook. No live route was
changed; the older backend binary on port 4810 remains uncertified.

Verified source correction: both MCP-only and native text-stream executors
reject `call_bound` because they lack real model token/logit/artifact/K/V
evidence. Native ordinary `call` retains actual generated text and a clearly
backend-computed binding digest. `cargo check --tests`, focused refusal test,
strict Clippy, and format check passed. This is a fail-closed boundary, not a
model-forward pass.
The Glimmer HTTP seat's unused placeholder artifact field was removed.
Its component forward parser now checks the documented receipt formula;
changed input or output IDs are rejected by a focused test.
The live runner's former silent skip for non-map/non-mind tools now fails the
packet explicitly. This closes an audit hole before the long mission gate.
Undeclared tool calls likewise fail before wire contact.
The recovered backend's `cargo test --all-targets` passed after the
unsupported-tool change; live integration tests in that suite remain ignored.
The declaration-bit regression passed separately after the final edit.

Open gates: one typed native Glimmer invocation that emits real token IDs,
logits digest, pinned weights/config/tokenizer identity, exact input, and K/V
receipt; tool results bound into that same run; GlobalMind recall and exact
nested-KV reuse through MCP; automaton MTP/drafter admission and confidence
updates from audited outcomes; 30–50-step live run with quality and cost audit.

The native component gate now exists: GFX2/GFO2 exact forward returned real
token IDs, logits digest, complete K/V digest, and streamed artifact hashes
twice identically on frozen Glimmer. See
`native-exact-binary-smoke-2026-11-04.md`. The harness/MCP/automaton join
remains open.

The real isolated MCP/SurrealKV restart gate was rerun after adding ranked
atom text hydration. It passed with the exact prefix text, 104 K/V pages,
52-layer continuation, and argmax 13796. The witness is still a manually
created prefix atom, not admitted cross-model distillation. The recovered
backend bound receipt now requires a native K/V digest and rejects missing
tool receipts or corrupted input/chain hashes; no live backend was replaced.
The same real gate passed an adversarial negative control: a second atom with
wrong inline text for its content digest was refused by `mind.recall`.

Adversarial finding: the current replay builder writes `content_text` as
`replayed atom <digest-prefix>` while assigning a different teacher-neutral
input digest. The new read-side hydration rejects that mismatch. These replay
atoms cannot certify coherent or model-agnostic recall; the distillation
pipeline needs a real, digest-bound semantic content carrier before promotion.

Governing runbook: source-command-harness-status plus
`Ares WebSite : App/backend/MODEL-SEAT-STATUS-DESIGN.md`. The recovered backend
directory is untracked in the current tree; do not interpret the running
port-4810 binary as this corrected source.
