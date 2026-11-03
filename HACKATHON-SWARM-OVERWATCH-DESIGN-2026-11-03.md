# Hackathon finish: carved swarm and one Meta Automaton overwatch

Date: 2026-11-03. Status: design and preregistration. Canonical repository only.
User decision: pause further MLIR lowering; restore the tokenizer as the
checkpoint-specific model adapter. Do not delete prior MLIR research.

## Source register and scope

- `N3/design/CARVED_SWARM_SPEC.md` and `MasterDesignDocument/C30-Multi-Agent-Lanes-Autorunner/BrigadeSwarmPluginBinding.md` state the one-weight, carved-view and brigade gates. Current backend `src/store.rs` has one `Arc<FrozenBaseWeights>` and 12 in-memory `SiloView`s. That is component evidence, not a live multi-agent run.
- `N3/design/superpositionholding.md` specifies branch holding on H3×T2. The user's *paper* `Ai Research and Preprint Vault/Deep Research/Own/Elle/SUPERPOSITION_HOLDING_PREPRINT.md` specifies an across-turn leaky tension reservoir and drift ledger. These are complementary mechanisms, not identical state machines. The paper's boundedness proof assumes bounded per-turn coherence drift; its suggested leak calibration is conditional on a local-level noise model.
- Rust server `src/cache/meta_automaton.rs` owns paged KV/prefill/rewind state; Spec 15 admission material is under its `docs/`. This is prior code to inspect and reuse, not a reason to create three new controllers in SoloDevTrack.
- Current backend `harness/runtime.rs` can judge packets and chain receipts; it has no proven live carved-swarm executor. Generated N3 `carve-specialist` remains canned data, not the capability.

## Requirement 1: one weight owner, many constrained workers

Meta-orchestrator (the harness model seat) decomposes a task into packets and proposes role, scope, atoms/edges, tools, budget and acceptance. The deterministic governor validates the proposal, pins its digest and issues each worker a **read-only graph handle** to exactly its carved atom/edge set. The same loaded weight owner serves L0 and every worker; assert pointer/model digest identity and count resident model buffers. Worker access must be enforced in lookup and tool dispatch, not only in a prompt. No role can query the global map or another role's view. Inter-role handoff requires a governor permit and a receipted edge. A worker submits an output/trajectory candidate; L0 merges only after the quality gate, independent adversarial pressure test, and receipt reconciliation. On failure, no merge and no silent scope widening.

Preregistered tests: planted out-of-scope atom gives zero hits in every worker query; forged/expired handle fails; attempts to traverse an edge crossing view boundary fail; same weights pointer/digest across workers; conflicting outputs are held for L0 judge; failed judge cannot merge. Run one real multi-packet model task and record task quality, per-worker graph visibility, weight-resident bytes, latency and receipts. A component test is not a swarm certification.

## Requirement 2: one deterministic Meta Automaton overwatch

One automaton accepts four interchangeable proposer types: MTP head, drafter, predictive prefill, and librarian/retrieval. A proposal is immutable until verified. The automaton's **single transition** records (a) validation audit, (b) confidence re-ranking from outcome receipts, and (c) the next prediction/prefill transition. It admits/commits or rejects/rolls back a proposal atomically; if retrieval confidence is below the preregistered floor it requests clarification. The model fills proposal and audit-observation fields; it never decides admission or changes thresholds. The automaton watches the loop and updates confidence; it does not execute packet steps or become a second planner.

State per proposal family: frozen policy version, stable strategy ID, Q32.31 confidence, calibration count, outcome history digest, cost/quality bounds, prefix/map/adapter identity and expiry. A receipt must bind prior state, proposal, measured audit signals, verdict, updated state and rollback/commit. Only verified outcomes update confidence; failures count as observations. Prefix prefetch must be keyed by exact model/prefix/TPPE/KV contract, and wrong-context prediction must invalidate rather than poison cache. Trajectory re-ranking changes **selection weights only**, never atom truth, provenance or the map's immutable origins.

The paper's holding law supplies across-turn memory: input-gated tension `T` and leaky drift `D`, with measured coherence `κ`, bounded inputs, and a versioned leak `ρ`. Keep it separate from the map branch hold: on ambiguous/conflicting proposals, hold 2–12 alternatives with zero external emission, apply the **same audit fields and thresholds** to every branch (isotropic scrutiny), and require each branch to satisfy local role, cross-invariant and whole-system consistency (homogeneity). “Same pressure” means equal policy and evidence opportunity, not forced equal scores. Suppress a branch only on a measured violation; collapse only after compliance, positive progress, replayability and quality floor. No direct map write on collapse; packet → judge → replay → crystallize remains mandatory.

The “Gödel point” is a design analogy about snapshot insufficiency, not an application of the incompleteness theorem. Cross-turn provenance allows the automaton to assess context sufficiency, but it cannot certify its own global optimality. Free-energy reduction, Landauer savings and compounding intelligence require measured controls; no metric is promoted from a single throughput/acceptance snapshot.

## Implementation order and stop gates

1. Bind the shared swarm model seat to the Glimmer checkpoint on the WD Drive:
   config SHA256 `3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031`,
   tokenizer SHA256 `c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d`.
   The config declares 52 layers, hidden width 6656 and vocabulary 202048.
   The tokenizer is model-bound and never enters shared atom identity. The
   separate Qwen tokenizer adapter is only for the previously specified
   Qdrant intake model. Pause new MLIR work. A 6656-wide hidden state is not
   automatically a calibrated semantic retrieval embedding; qualify that
   separately before indexing or comparing it to Qwen vectors.
2. Implement the carved graph handle against the one GlobalMind, enforced at read/traverse/tool boundaries; connect governor permits, real model seat and L0 judge. Do not port N3 canned `carve-specialist` percentages.
3. Extract/reuse the existing Meta Automaton transition/rollback semantics; add one typed proposer envelope for retrieval and prefill rather than separate controller loops. Persist confidence history and receipts in the governed store. Keep telemetry observation in HyperDX.
4. Add across-turn paper-style holding telemetry and branch-hold arbitration with same-policy scrutiny. Use Q32.31 for admission/confidence state; verify numeric bounds separately. Guard against circular self-evidence: model proposals cannot serve as independent outcome judges.
5. Run a real end-to-end task with two carved workers on one model buffer, retrieval proposal, MTP/prefill proposal, a forced failure and recovery, L0 adversarial merge, exact K/V identity checks and restart. Compare to same model without carving/overwatch, measuring task outcome, wrong-context prefetch, acceptance, latency, memory, and receipt completeness. Negative results stay in the report.

Stop if only schemas/defaults/canned responses run, if the same model weights are copied, if a worker can fetch outside its view, if a proposed audit self-certifies its outcome, or if a confidence change lacks an observed receipt. No live certification until the complete run passes.
