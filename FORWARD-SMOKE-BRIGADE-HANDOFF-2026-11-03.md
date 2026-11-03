# GlobalMind forward smoke and carved swarm handoff

Date: 2026-11-03. Canonical implementation repository: this repository only, branch `codex/consolidate-n3-globalmind-20261102`. Keep other worktrees, live `hypermind/core`, and frozen model shards untouched. This is an evidence register and execution queue, not a certification.

## Target system

One SurrealDB GlobalMind holds model-neutral knowledge atoms, typed edges, trajectories and nested exact-model tensor K/V. Qdrant holds the fact library; HyperDX holds telemetry. Glimmer is the shared model weight owner for the carved swarm: L0 decomposes tasks, scoped workers read permitted graph neighborhoods, and L0 judges and adversarially merges outputs. The deterministic Meta Automaton sits at the MCP to harness boundary and audits/adjudicates proposals; it does not command workers. Keep Glimmer tokenization at the model adapter, TPPE identity in K/V contracts, and Q32.31 control state. Further MLIR lowering is paused. No Python or JSON belongs inside the model I/O boundary; MCP JSON-RPC framing is transport only.

## Evidence now

- `331195d`: release-profile Qwen Layer 3 exact K/V forward smoke passed 1/1 in 48.39 s with isolated SurrealKV and a declared MCP authority preflight. This is not a full Glimmer forward or an MCP child round-trip. Receipt: `results/tensor-kv/tensor-kv-harness-bridge-report.txt`.
- `0584b2e`: Glimmer frozen checkpoint bytes rehashed, then native forward preflight exited 1 as designed: manifest still binds retired `tppe-reference-v1` and no native Glimmer executor is pinned. Receipt: `results/glimmer-forward/preflight-2026-11-03.md`.
- `5baa147`: Glimmer tokenizer adapter is bound to WD Drive artifact and has a live tokenizer test. It is not a full model forward.
- `f033dff`: bridge checks declared MCP URL/namespace/database against one mind. Declared parity is weaker than child-attested authority.
- `5e8bb16`: isolated Surreal scoped read component passes; it filters members and edges with both endpoints visible and fails closed on invalid scope. It remains a trusted-caller API and uses a workspace-wide scan. Governor permit and MCP binding are open.
- `6b94bf13`: two workers share one `Arc` test weight buffer, isolate in-memory views, fail an L0 test gate on tamper and replay the same merge root. It uses 64 test floats, not Glimmer; the merge gate is test-local, not a production API; no real model task ran. The test labels weight buffer equality as K/V identity, but tensor K/V identity is a separate contract and still needs proof.
- `f13356db` and `e27cc7fa`: GlobalMind MCP tool path and a child transport route exist. Inspect their receipts before extending them; do not infer production readiness from commit titles.

## New Glimmer Layer 0 weight carve receipt supplied by Stewart

`carve_proof.py` selected `model.layers.0.*` only as a mechanism proof, not the intended carve policy. Two runs over the same 17 GB frozen Glimmer base produced 30 tensors, 323758374 bytes, and the same output SHA-256 `523c8ff571c3680bf4f1b67c980ca77596dcb73364d6c8da843239e05a695728`; `cmp` was byte identical. Four base shard hashes remained unchanged. This is a user-supplied receipt; the script and artifacts were not found in this checkout at handoff time, so it has not been independently reproduced here. The result supports local deterministic extraction for this one selection rule. It does not prove the selection is useful, that workers share one loaded Glimmer buffer, cross-machine determinism, or runtime specialization. Do not use layer 0 as a default worker carve without a task-level ablation and selection specification.

## Critical path to an actual carved swarm

1. Freeze a Glimmer model contract: checkpoint shard/index digests, tokenizer digest, architecture, TPPE candidate/position contract, K/V tensor layout, backend ABI, exact serving executable digest and path. Fail closed on mismatch. Fix the preflight stop; then execute one genuine frozen Glimmer forward with token/logit and K/V receipts. A Glimmer 6656 hidden width is not automatically a semantic embedding dimension.
2. Make the *serving* API model agnostic through a typed `ModelSeat`/`ForwardExecutor` contract: `load/attest`, `tokenize`, `forward`, `kv_page`, `generate`, `unload`. Adapter identity must be returned by the serving process and pinned in every receipt; tokenizer and TPPE must be adapter-owned. Checkpoint switching must invalidate incompatible prefix/KV pages. Test with two actual model adapters or explicitly report one-adapter-only. The existing backend `NativeMtpExecutor` is only transport generic: it sends `tool: ...\nargs: ...` as text to `/v1/generate`, so it is not a native typed model I/O path and must not be called complete model agnosticism. `/api/models` also hardcodes Qwen.
3. Bind the production worker dispatch to one Glimmer process/buffer. A worker receives a signed/expiring role+scope+budget handle and only a scoped read interface; every atom, edge, traversal, retrieval and tool call checks that handle. Attest the live Surreal MCP child target before dispatch. A worker cannot receive the unrestricted map client. Verify same process/model digest and resident bytes across L0 and at least two simultaneous workers. Disk extraction into a separate safetensors file is not evidence of shared memory.
4. Implement a production L0 merge gate. It must require each packet judge, independent adversarial result, permit, scope proof, model identity, exact K/V contract, receipt chain and replayable output digest. Failure must not publish an atom or widen scope. Run one real multi-packet model task with forced failure/recovery and restart, then compare against the same model without carving. Measure quality, latency, token count, memory, leakage attempts, and receipt completeness.
5. Only after the above, test the automaton overwatch on the same route. Compare with and without its retrieval/prefill proposals. Require measured outcomes before confidence updates; do not let model-provided audit text self-certify.

## Safe commands and boundaries

From `globalmind-runtime/`: `cargo run --release --bin glimmer_forward_preflight -- manifests/muse-glimmer-candidate.manifest` currently exits 1 by design. From the same directory: `cargo test --release --features live-forward-model --test tensor_kv_qualification -- --ignored --nocapture` reruns the existing Qwen Layer 3 isolated SurrealKV hardware gate; it is not a Glimmer gate. Do not point these at live `hypermind/core`. Before any commit, inspect `git status --short` and stage only owned files: other agents have unrelated dirty edits, including backend HTTP, frontend, MCP, distillation, and transducer work.

## Immediate ownership split

- Glimmer serving lane: native frozen-weight executor, typed model seat, TPPE/K/V contract and real forward proof. Do not change the mind database.
- Carved swarm lane: governor-issued scoped handles, MCP child attestation, production L0 merge, one real two-worker task. Use the existing `ScopedGraphRead` and Surreal scoped retrieval; do not create a second map.
- Model-agnostic server lane: replace hardcoded `/api/models` identity with live adapter attestation; add second adapter conformance gate and cache invalidation. Do not relabel the Qwen transport wrapper as universal execution.
- Evaluation lane: preregister carved versus uncarved control and negative cases, retain real receipts. No synthetic percentages or pass markers.

Stop claims at the exact evidence boundary. The current tests prove useful components. The native Glimmer forward, one-buffer live swarm, and two-adapter server remain open.
