# Forward-pass smoke setup — 2026-11-03

Status: setup in progress. One SurrealDB GlobalMind, hyperbolic map with nested
exact-model tensor K/V; automaton at MCP-to-harness boundary. Qdrant is the
fact library, HyperDX telemetry. The live `hypermind/core` remains quarantined.
Further MLIR lowering is paused. Shared swarm model seat is Glimmer with its
pinned WD Drive tokenizer; Qwen is the existing Layer-3 qualification model
and separate Qdrant-intake encoder candidate. Do not mix their K/V or vector
spaces.

The **carved swarm is a Glimmer execution function**, not an automaton
function: one Glimmer weight owner serves L0 and all scoped worker turns.
The **Meta Automaton is a separate boundary overwatch** at MCP↔harness; it
validates proposals and observed outcomes, updates confidence and prediction,
and leaves Glimmer's task decomposition, worker execution and L0 merge to the
harness. No automaton branch may silently become a worker or commander.

## Real baseline re-run today

From `globalmind-runtime/`:

```sh
cargo test --release --features live-forward-model --test tensor_kv_qualification -- --ignored --nocapture
```

Exit 0 in 43.27 seconds. The test rehashed the frozen Qwen3.6 roster, started
an isolated SurrealKV instance on an ephemeral loopback port, wrote an atom
address and both real Layer-3 K/V heads, restarted SurrealDB, retrieved them
through `GlobalMindBridge::prepare_forward_recall`, and required bit-exact
hidden and 248,320-logit parity against the uninterrupted control. Identity
mutations and readonly-write attempts failed closed. The updated raw receipt
is `results/tensor-kv/tensor-kv-harness-bridge-report.txt`. No write to port
8000 or `hypermind/core` was made. This is a **real Layer-3 component gate**,
not Glimmer execution, a complete-model forward, a live MCP round-trip or a
carved-swarm qualification.

## Smoke chain to assemble

1. **Identity and source** — freeze one Glimmer weight/config/tokenizer
   manifest and exact adapter digest. Verify the WD Drive checkpoint; pin
   shared-weight owner, not one copy per worker.
2. **Single mind** — MCP child must target the same isolated SurrealDB
   namespace/database as `GlobalMindBridge`, never a separate `.hypermind`
   root. Assert target identity and schema before any call. Use read-only
   recall and exact-model K/V fetch; keep writes behind existing admission.
3. **Glimmer carved task** — L0 Glimmer proposes two packets and scoped
   atom/edge handles.
   Backend governor permits them, and every worker read/traverse is checked
   against its carved handle. A planted out-of-scope real atom is invisible.
4. **Separate overwatch boundary** — one automaton at MCP/harness seam receives
   retrieval/prefill/MTP proposals and observed receipts. Same policy audits
   every held branch; confidence changes only after verified outcomes.
   Rejected proposals roll back atomically. No second planner or mind store.
5. **Real forward** — one frozen Glimmer model buffer serves L0 and workers.
   Execute a task with recalled typed atoms and, only on exact identity hit,
   nested K/V; compare against the same model without recall. Record complete
   output/quality, model digest, source atom IDs, K/V identity, latency,
   resident weight bytes and receipt chain. A K/V miss must cause normal
   recomputation or explicit refusal, not cross-model reuse.
6. **Adversarial merge** — L0 validates outcomes, pressures conflicting
   claims, merges only judged/replayable results, and checks Surreal restart
   reconciliation. Failed retrieval is still a receipted confidence update.

## Current hard gaps

- No Glimmer full-forward executor is wired through this GlobalMind bridge.
  Glimmer tokenizer parity and 6656 hidden width do not prove inference or
  semantic embedding quality.
- `research-grade-harness/src/globalmind_bridge.rs` is typed/read-only and
  proven in isolation. `map_plugin.rs` still describes an independently
  installed map executable/root; it must not create another map for this run.
- Backend carved `SiloView` and one `Arc<FrozenBaseWeights>` exist in memory,
  but there is no live two-worker, one-model-buffer task receipt. N3
  `carve-specialist` constants are not carving.
- The automaton's existing KV paging/rewind code is in the separate Rust
  server. The one cross-turn confidence/holding transition is not yet wired
  at the MCP/harness seam.

Completion requires one end-to-end receipt covering all six steps above.
Component PASS labels cannot substitute for it. Keep every failed gate and
its observed reason in the final report.
