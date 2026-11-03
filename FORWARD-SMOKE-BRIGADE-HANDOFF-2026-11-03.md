# GlobalMind forward smoke and carved swarm handoff

Date: 2026-11-03. Canonical implementation repository: this repository only, branch `codex/consolidate-n3-globalmind-20261102`. Keep other worktrees, live `hypermind/core`, and frozen model shards untouched. This is an evidence register and execution queue, not a certification.

## Target system

One SurrealDB GlobalMind holds model-neutral knowledge atoms, typed edges, trajectories and nested exact-model tensor K/V. Qdrant holds the fact library; HyperDX holds telemetry. Glimmer is the shared model weight owner for the carved swarm: L0 decomposes tasks, scoped workers read permitted graph neighborhoods, and L0 judges and adversarially merges outputs. The deterministic Meta Automaton sits at the MCP to harness boundary and audits/adjudicates proposals; it does not command workers. Keep Glimmer tokenization at the model adapter, TPPE identity in K/V contracts, and Q32.31 control state. Further MLIR lowering is paused. No Python or JSON belongs inside the model I/O boundary; MCP JSON-RPC framing is transport only.

## How the mind and nested hyperbolic K/V are meant to work

The model supplies a query location on H3×T2, workspace, provenance and bounded `k`. `mind.recall` ranks nearby atom addresses and returns an ordered bundle with atom IDs, content digests and ranking digest. The automaton at the harness/MCP boundary admits or rejects that retrieval proposal using policy, measured confidence and dependency checks; a carved worker receives only its permitted neighborhood. The model then uses the retrieved atom content and typed relations as evidence for the current task, records the selected trajectory and outcome, and only a judged replay may crystallize new shared atoms. A failed retrieval changes no atom truth.

The nested K/V is a separate, **model-bound continuation cache**, not shared semantic knowledge. `mind.kv_exact` must match model ID/revision/digest, request and prefix digests, TPPE version/configuration, address and tensor contracts, geometry space, dtype/layout/shape, layer/head/token range, phase state, revision, and key/value digests before payload reuse. Any mismatch is a miss and requires a fresh forward computation. The H3×T2 address selects context; it does not make two models' K/V tensors interchangeable. This is the coherent-use path to verify: recall relevant shared atoms, admit a proposal, hydrate only exact-compatible K/V, run the model, judge the answer against source evidence, receipt the trajectory, and update retrieval confidence from the verified outcome.

**Current proof boundary:** exact K/V storage/fetch and a Qwen Layer 3 parity test exist; an isolated MCP child test exists; no receipt yet shows a full Glimmer answer improving from recalled atoms through this exact path. `globalmind-mcp` currently carries recall/K/V results inside MCP JSON text; native typed model-facing hydration remains a gate. Do not call this coherent model use until a real task with a no-recall control, answer judge, and trajectory receipt passes.

2026-11-03 boundary correction: the backend requires an explicit non-core mind URL/namespace/database, compares the proposal map identity to that authority, spawns `globalmind-mcp` with the same target, and checks the child's live `mind.db_status` contract/authority before the automaton dispatches a read. A transport-valid response does not update confidence without an independent outcome judge. `globalmind-mcp` checks the immutable atom/address/K/V read contract; full distillation tables remain a separate write gate. The isolated child attestation test and backend component suite pass. **Still open:** typed model-facing hydration, native Glimmer forward, and task-level benchmark.

The child now has `mind.recall_scoped` over the existing persisted cognitive-well membership seam. An isolated MCP round-trip shows two visible atoms and one internal edge while excluding a third atom and a cross-scope edge; missing and duplicate wells are refused. The backend's handle path below binds worker requests to this tool.

The backend now implements that local handle seam: configure `ARES_L0_SCOPE_ISSUER_TOKEN` (at least 32 bytes) in the trusted harness process, plus explicit `ARES_MCP_MIND_URL/NS/DB` and `ARES_MCP_MIND_BIN`. L0 posts to `/api/harness/swarm/scope` with the token in `x-ares-l0-scope-token` and body `role`, `map_identity`, `prefix_identity`, `well_ids`, `expires_ms`. The returned opaque handle is process-local and expires within 10 minutes. A worker posts `role`, the same map/prefix identities, `scope_handle`, and a `mind.recall` query to `/api/harness/mcp/execute`. The backend resolves the handle and rewrites the MCP call to `mind.recall_scoped` with the registered wells, ignoring any supplied `well_ids`. Wrong role, map, prefix, missing handle, expiry, or worker exact-K/V fail before read dispatch. Restart revokes handles. This is an enforceable local carve but still lacks a live Glimmer worker task and a model-attested prefix binding. `mind.kv_exact` remains L0-only until exact tensor entitlements are designed.

## Evidence now

- `331195d`: release-profile Qwen Layer 3 exact K/V forward smoke passed 1/1 in 48.39 s with isolated SurrealKV and a declared MCP authority preflight. This is not a full Glimmer forward or an MCP child round-trip. Receipt: `results/tensor-kv/tensor-kv-harness-bridge-report.txt`.
- `0584b2e`: Glimmer frozen checkpoint bytes rehashed, then native forward preflight exited 1 as designed: manifest still binds retired `tppe-reference-v1` and no native Glimmer executor is pinned. Receipt: `results/glimmer-forward/preflight-2026-11-03.md`.
- `5baa147`: Glimmer tokenizer adapter is bound to WD Drive artifact and has a live tokenizer test. It is not a full model forward.
- `f033dff`: bridge checks declared MCP URL/namespace/database against one mind. Declared parity is weaker than child-attested authority.
- `5e8bb16`: isolated Surreal scoped read component passes; it filters members and edges with both endpoints visible and fails closed on invalid scope. It remains a trusted-caller API and uses a workspace-wide scan. Governor permit and MCP binding are open.
- `6b94bf13`: two workers share one `Arc` test weight buffer, isolate in-memory views, fail an L0 test gate on tamper and replay the same merge root. It uses 64 test floats, not Glimmer; the merge gate is test-local, not a production API; no real model task ran. The test labels weight buffer equality as K/V identity, but tensor K/V identity is a separate contract and still needs proof.
- The backend native MTP executor now pins the first attested model identity for each executor/mission and rejects a subsequent identity change. A fresh executor can bind a different model. This is a mission continuity guard, not a second native model adapter.
- `f13356db` and `e27cc7fa`: GlobalMind MCP tool path and a child transport route exist. Inspect their receipts before extending them; do not infer production readiness from commit titles.

## New Glimmer Layer 0 weight carve receipt supplied by Stewart

`carve_proof.py` selected `model.layers.0.*` only as a mechanism proof, not the intended carve policy. Two runs over the same 17 GB frozen Glimmer base produced 30 tensors, 323758374 bytes, and the same output SHA-256 `523c8ff571c3680bf4f1b67c980ca77596dcb73364d6c8da843239e05a695728`; `cmp` was byte identical. Four base shard hashes remained unchanged. This is a user-supplied receipt; the script and artifacts were not found in this checkout at handoff time, so it has not been independently reproduced here. The result supports local deterministic extraction for this one selection rule. It does not prove the selection is useful, that workers share one loaded Glimmer buffer, cross-machine determinism, or runtime specialization. Do not use layer 0 as a default worker carve without a task-level ablation and selection specification.

## Critical path to an actual carved swarm

Native model note: the frozen Glimmer checkpoint's own architecture uses RoPE on sliding layers and NoPE on full-attention layers. Existing TPPE Q/K tests are partial-model and promoted no replacement. The first full-model baseline must preserve that trained attention path; TPPE can address GlobalMind memory separately. Installing TPPE into Glimmer Q/K is a separate same-weights, same-tape full-model ablation with held-out NLL and task-quality gates. If it loses the preregistered frontier, bench that replacement. This is required by the user's math-first rule, not a retreat from TPPE memory geometry.

1. Freeze a Glimmer model contract: checkpoint shard/index digests, tokenizer digest, architecture and native RoPE/NoPE layout, separate TPPE memory-address contract, K/V tensor layout, backend ABI, exact serving executable digest and path. Fail closed on mismatch. Fix the preflight stop; then execute one genuine frozen Glimmer forward with token/logit and K/V receipts. A Glimmer 6656 hidden width is not automatically a semantic embedding dimension.
2. Make the *serving* API model agnostic through a typed `ModelSeat`/`ForwardExecutor` contract: `load/attest`, `tokenize`, `forward`, `kv_page`, `generate`, `unload`. Adapter identity must be returned by the serving process and pinned in every receipt; tokenizer and TPPE must be adapter-owned. Checkpoint switching must invalidate incompatible prefix/KV pages. Test with two actual model adapters or explicitly report one-adapter-only. The existing backend `NativeMtpExecutor` is only transport generic: it sends `tool: ...\nargs: ...` as text to `/v1/generate`, so it is not a native typed model I/O path and must not be called complete model agnosticism. `/api/models` also hardcodes Qwen.
3. Bind the production worker dispatch to one Glimmer process/buffer. A worker receives a signed/expiring role+scope+budget handle and only a scoped read interface; every atom, edge, traversal, retrieval and tool call checks that handle. Attest the live Surreal MCP child target before dispatch. A worker cannot receive the unrestricted map client. Verify same process/model digest and resident bytes across L0 and at least two simultaneous workers. Disk extraction into a separate safetensors file is not evidence of shared memory.
4. Implement a production L0 merge gate. It must require each packet judge, independent adversarial result, permit, scope proof, model identity, exact K/V contract, receipt chain and replayable output digest. Failure must not publish an atom or widen scope. Run one real multi-packet model task with forced failure/recovery and restart, then compare against the same model without carving. Measure quality, latency, token count, memory, leakage attempts, and receipt completeness.
5. Only after the above, test the automaton overwatch on the same route. Compare with and without its retrieval/prefill proposals. Require measured outcomes before confidence updates; do not let model-provided audit text self-certify.

## Safe commands and boundaries

The 2026-11-03 disposable SurrealKV live gate now passes through the active
`globalmind-mcp` child (`mind.db_status`) and the typed harness bridge for one
atom and exact K/V page. Receipt:
`results/forward-smoke/globalmind-mcp-child-bridge-2026-11-03.md`. The retired
`hypermind-mcp` fails on model-neutral string atom IDs and is not the forward
path. No model activation was executed by this gate.

The decoder must be a native operator-graph executor, not a hardcoded Glimmer
branch. `globalmind-runtime/UNIVERSAL-DECODER-DESIGN.md` defines the binary
harness ABI, admission, operator families, and second-model falsifier. A new
checkpoint revision can reuse the same engine when its graph operators remain
supported; exact model-bound K/V must miss after the revision changes.

## Attention policy gate from the three design references

`hypermind-nested-kv/ATTENTION-ECONOMICS-FEP.md` proposes I-RECALL, but explicitly says dependency edges, exact invalidation, and measured agreement between local depth cost and session cost are not built. `MasterDesignDocument/C06-Frontier-Push/AttentionEconomicsAndActiveInference-Spec.md` corrects its own reward claim: maximizing `Q/(H+cN)` is not equivalent to minimizing `F=C-Q`. `Ares WebSite : App/backend/HYPERBOLIC-SCHEDULER-FRAMEWORK-2026-11-02.md` gives CE1 worst-case scan, CE2 graph-distortion, CE3 stale-producer recall, and CE4 beta/SLO counterexamples. Keep these as design references, not certified runtime policy. For promotion, receipt actual retrieval quality, producer validity, elapsed time, measured energy or a clearly labeled proxy, and thermal state; preregister a quality floor, deadline, and budget. Fail closed on stale producer or missing dependency lineage. Landauer's bound is a physical lower bound, not evidence of actual per-call dissipation or an alignment guarantee.

From `globalmind-runtime/`: `cargo run --release --bin glimmer_forward_preflight -- manifests/muse-glimmer-candidate.manifest` currently exits 1 by design. From the same directory: `cargo test --release --features live-forward-model --test tensor_kv_qualification -- --ignored --nocapture` reruns the existing Qwen Layer 3 isolated SurrealKV hardware gate; it is not a Glimmer gate. Do not point these at live `hypermind/core`. Before any commit, inspect `git status --short` and stage only owned files: other agents have unrelated dirty edits, including backend HTTP, frontend, MCP, distillation, and transducer work.

## Immediate ownership split

- Glimmer serving lane: native frozen-weight executor, typed model seat, TPPE/K/V contract and real forward proof. Do not change the mind database.
- Carved swarm lane: governor-issued scoped handles, MCP child attestation, production L0 merge, one real two-worker task. Use the existing `ScopedGraphRead` and Surreal scoped retrieval; do not create a second map.
- Model-agnostic server lane: replace hardcoded `/api/models` identity with live adapter attestation; add second adapter conformance gate and cache invalidation. Do not relabel the Qwen transport wrapper as universal execution.
- Evaluation lane: preregister carved versus uncarved control and negative cases, retain real receipts. No synthetic percentages or pass markers.

Stop claims at the exact evidence boundary. The current tests prove useful components. The native Glimmer forward, one-buffer live swarm, and two-adapter server remain open.

## Session handoff addendum — 2026-11-03

**Read this first on return.** The canonical git checkout is the path at the
top of this document, branch `codex/consolidate-n3-globalmind-20261102`.
Commits `c94debfd` and `20b46190` landed after the earlier evidence list.
The former makes backend `/api/models` probe the native seat's binary
`/v1/info` identity instead of hardcoding Qwen and adds a `--native-baseline`
Glimmer preflight mode. The latter switches the harness live child gate from
retired numeric-ID `hypermind-mcp` to active `globalmind-mcp` and records the
universal decoder design. The statement above that `/api/models` still
hardcodes Qwen is superseded; **execution is still not universal**.

**Verified now:** `cargo test --test globalmind_bridge_live
bridge_reads_one_real_atom_as_binary_recall -- --ignored --nocapture` passed
1/1 with a disposable SurrealKV, live `globalmind-mcp` child attestation,
GMRC atom recall, and exact-model tensor K/V read. Full harness tests and
Clippy passed; formatting passed after formatting the touched harness files.
The Glimmer baseline preflight rehashed the frozen checkpoint but exited 1
because `GLIMMER_NATIVE_FORWARD_BIN` is unset. Receipts are
`results/forward-smoke/globalmind-mcp-child-bridge-2026-11-03.md` and
`results/glimmer-forward/native-baseline-preflight-2026-11-03.md`. There is
still **no full native Glimmer forward, no real worker task, no task-quality
benchmark, and no second decoder architecture conformance run**.

### Parallel work packets for separate agents

Give each packet to one agent with exclusive file ownership. Agents should
report changed paths, exact commands and exit codes, real receipts, and the
next unresolved gate. They should stage only their own paths. Do not edit the
live `hypermind/core`, frozen checkpoint shards, or another agent's dirty
files. Keep all implementation and receipts in this canonical repository.

1. **Universal native decoder (critical path):** own
   `globalmind-runtime/UNIVERSAL-DECODER-DESIGN.md`, a new native executor
   module/crate inside this repo, and its README. Implement bounded safetensors
   loading, immutable artifact attestation, typed operator graph compilation,
   a versioned binary forward ABI, and real CPU/Metal execution of the
   operators required by the frozen Glimmer graph. The harness must not switch
   on the name “Glimmer.” A revision of the same supported graph must load
   without editing Rust source; an unknown operator/layout must reject before
   execution. Preserve trained RoPE on sliding layers and NoPE on full layers
   for baseline; TPPE is the separate mind-address path. First deliver a
   real weight-backed layer receipt, then full-model logits/K/V and parity
   against an independent frozen reference. Do not copy the discarded,
   untracked `core_lm/` into production or claim a partial layer as a full
   decoder. The installed `libmlx.dylib`/`libmlxc.dylib` and older native
   tensor code may be studied as references; the implementation home is here.
2. **Model-neutral harness wire:** own the harness model boundary and binary
   ABI integration, plus README. Connect one attested decoder seat to the
   existing `GlobalMindBridge::prepare_forward_recall` and automaton, with
   checkpoint/tokenizer/operator-graph digest in every request and receipt.
   Prove model switch invalidates old prefix/K/V while shared atom recall
   remains usable. Do not make `tool:...` text through `/v1/generate` the
   claimed native model boundary. Gate on a real decoder executable; a mock
   transport is only a component test.
3. **MCP/mind and carved scope:** own `globalmind-mcp` and scoped retrieval,
   not the decoder. Keep one SurrealDB authority; require child
   `mind.db_status` attestation, non-core DB, governor-issued well scope, and
   no worker `mind.kv_exact` without explicit tensor entitlement. Replace
   workspace-wide scan only after preserving exact scoped-result parity and
   testing traversal time on a real graph. The disposable bridge pass above
   is the baseline. Do not revive legacy `hypermind-mcp` numeric hydration.
4. **Carved swarm execution:** own L0 dispatch/merge and its README. Use one
   resident model weight owner across L0 and two scoped workers, with separate
   role handles and graph visibility. Require independent judge and
   adversarial result before merge or atom mint. A carve file on disk or the
   64-float toy is insufficient. Run a real task with a forced scope escape,
   invalid receipt, retry, and no-carve control. Record quality, latency,
   memory, and receipt chain.
5. **Benchmark/evaluation:** own preregistration and receipts only. Freeze
   the prompts, reference logits/continuations, quality judge, source corpus,
   budgets and counters before seeing results. Measure native baseline,
   baseline+mind recall, exact K/V continuation, and carved swarm under the
   same checkpoint. Separate cold/warm cache and report failures. Keep Qdrant
   fact-library retrieval separate from SurrealDB mind recall and HyperDX
   telemetry; do not feed prior DNHG/ELLE work into competition claims.

### Return-session audit order

Check `git status --short` and each agent's commit/receipt first. Then inspect
the real decoder binary and its artifact hash, operator graph, native I/O,
full forward output, and independent parity. Next rerun the disposable
MCP/bridge gate, then one harness→automaton→MCP→mind→decoder task with exact
K/V and a no-recall control. Only then run benchmark sweeps. If a lane has
only a design, compiler pass, synthetic fixture, or component output, label
that lane incomplete. Do not promote the attention-economics ratio as policy:
its own correction refutes reward/free-energy equivalence; require measured
cost, producer validity, quality floor, and exact invalidation first.

**Existing dirty tree at handoff:** frontend files, backend
`tests/mcp_e2e_live.rs`, N3 tool ports, `hypermind-nested-kv/src/lib.rs`,
distillation/tensor-K/V reports, and untracked `core_lm/`, transducer and
preregistration files belong to other work. Inspect their actual diffs before
editing or committing them. No deletion or cleanup was performed here.
