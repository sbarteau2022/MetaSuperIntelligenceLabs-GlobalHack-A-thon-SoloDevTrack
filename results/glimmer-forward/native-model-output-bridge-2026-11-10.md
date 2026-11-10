# Native model output bridge — 2026-11-10

## Change and identity

The backend now calls `/v1/tokenize-native` with GTK2/GTO2, runs exact GFX2/GFO2 forward, calls `/v1/detokenize-native` with the generated IDs, verifies the GDR2 request-bound receipt, and judges the decoded model text. The recall JSON remains MCP control evidence and is no longer returned as the model's answer. The detokenizer request, output digest, and receipt are included in the bound call receipt. The live mission response reports the selected executor kind.

The server and harness now use `model-artifact-v1\0` for the weights/tokenizer/config composite. This deliberately changes the old Glimmer composite identity and invalidates model-bound caches keyed by it. The isolated server reported composite `b2e63fa50bf2d82cf58ac62f5af86f88e041762043b0be14ac5326f0931fab8b` for the frozen local checkpoint.

## Gates run

- `core_lm`: `cargo check --tests --offline` passed with two existing unused-variable warnings; `cargo test --lib --offline -q` passed 46 tests. A release server was built and run on isolated port 18111. `smoke_exact_wire` independently hashed the frozen checkpoint, called exact forward twice, and passed token/logit/K/V/artifact equality.
- Backend: `cargo check --tests --offline`, `cargo test --all-targets --offline -q`, and strict all-target Clippy passed. The binary detokenizer parser test covers changed IDs, changed text, truncation, and empty requests.
- The ignored real-hardware `executor_kv_miss_then_restart_hit_equals_plain_forward` test passed against that isolated server, real MCP child, and disposable SurrealKV restart. The miss receipt contained `native.kv-prefill` and `native.detokenize`; the hit receipt contained `native.kv-hit` and `native.detokenize`. The test now rejects returning recall text or control JSON as the model output. Both paths matched the plain exact-forward IDs, logits, and K/V digest.
- The isolated server was stopped after the run; port 18111 cleared. The live backend on 4810 and persistent SurrealDB on 8000 were untouched.

## Limits

The K/V store adapter used by this test keeps its identity roster in process memory; the production mission route still does not attach a persistent K/V store. The test proves an isolated component join across a Surreal restart, not a production restart. The real model output is now judged, but no independent answer-quality pass, Qwen/Nemotron swap, session checkpoint hydration, or single automaton proposal loop was established here. The old `core_lm` tree has broader pre-existing `cargo fmt --check` drift; this pass did not reformat it wholesale.
