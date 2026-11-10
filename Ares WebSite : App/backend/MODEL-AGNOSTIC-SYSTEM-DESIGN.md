# Model agnostic governed forward: implementation contract

Status: design and preregistration, 2026-11-10. No certification claim.

## Authorities and boundaries

- The Rust harness owns session identity, policy, tool permits, budgets, quality judges, and the single Meta Automaton. A model proposes tokens or actions; it never changes its own admission policy.
- A model adapter owns its tokenizer, weights, forward kernel, and exact tensor K/V format. The harness accepts only a versioned native binary request and receipted response with pinned artifact identity. Swapping models invalidates model-bound K/V; it does not erase semantic atoms or Qdrant facts.
- SurrealDB owns one GlobalMind: immutable, source-grounded semantic atoms, typed edges, trajectory outcomes, and nested model-bound exact K/V pages. Qdrant owns research RAG and session memory. HyperDX owns telemetry. N3 MCP is a read-only/control stream to the harness; it is never a second mind.
- JSON is permitted on MCP/Qdrant control planes, never across the model I/O boundary.

## One governed turn

1. Load the last independently verified compaction checkpoint and bounded recent session summaries from Qdrant. Keep their source IDs and content digests. Resolve research hits using the collection's declared encoder identity and dimension; a swapped model cannot reinterpret vectors from another embedding space.
2. Obtain scoped GlobalMind recall through MCP. Verify atom content digest, provenance, scope, and trajectory receipt. Merge only bounded, admitted content into the model adapter's input transducer.
3. The automaton admits retrieval, prefill, MTP, and drafter proposals through one Q32.31 policy. For each proposal record expected gain, measured latency/energy proxy, quality evidence, and prediction error. Apply the same audit fields to every family (isotropic scrutiny) and reject any family whose gain harms a preregistered system invariant (homogeneity). A policy change requires a version and replay comparison; it cannot silently retune the live loop.
4. Execute one native forward on the selected seat. Verify token IDs, logits and exact K/V digests, artifact pins, and receipt chain. Commit accepted speculative prefix atomically; rollback rejected tail. Meter actual resource use where available; time-derived joules must be labeled an estimate.
5. Only a judged successful trajectory may gain rank. New atoms require source-grounded content, stable model-neutral identity, immutable origins, and deduplication; model-specific tensor witnesses remain separate. Failed walks are retained as negative evidence. Writes are queued behind the independent judge, not performed by the model itself.
6. Persist the session checkpoint and compaction summary to Qdrant with content digest, session/turn, predecessor digest, and model identity. A new session must load the bounded tail before its first forward, fail closed on digest mismatch, and record what was admitted to context.

## Qualification gates

- The two swap targets are **Qwen 3.6 Cyber** and **Nemotron 3 Nano Omni 30B A3B MXFP4**. Admit each as a distinct artifact-pinned adapter. Run the same real task corpus through both: the same research documents and admitted semantic atom IDs must remain reachable, while a tensor K/V page from one seat is refused by the other. Assert the research encoder's own identity and vector dimension before Qdrant search; neither model's hidden width defines the RAG vector width.
- For each seat, record the tokenizer, execution contract, weights/config identity, output mode, and available forward/K/V evidence. A text-only API adapter may prove semantic recall and task quality, but cannot claim exact logits, exact tensor K/V, speculative verification, or native binary model-boundary compliance. Those gates require an actual native forward adapter, not a fabricated receipt.
- Restart the harness: checkpoint chain and summary tail hydrate before forward; corrupt or missing checkpoints produce an explicit refusal. Verify no duplicate point overwrites.
- Run one real N3 MCP stream and GlobalMind read through the governed port. A malformed, out-of-scope, or stale receipt must be refused.
- Compare uninterrupted and restarted exact K/V continuation against the same model. Compare speculation on/off output, rollback, quality, throughput, and resource receipts across a real task set. Do not promote an optimizer from throughput alone.
- Walk a real cross-model task trajectory, judge outcome independently, persist a new atom and typed edges, and verify retrieval after restart. A placeholder replay atom or model-bound tensor hash cannot satisfy this gate.

## Current code gap to close

`dispatch_mission_live` selects `NativeModelExecutor` but does not call `load_session_context` or run the Meta Automaton around its forward. `load_session_context` exists only as a memory API route. `globalmind-mcp` recall is read-only; it does not write judged atoms or trajectory outcomes. The N3 MCP stream is not joined into this mission route. These are separate integration gates, not a live system pass.

Local inspection on 2026-11-10 found Nemotron's checkpoint under `~/models/Nemotron-3-Nano-Omni-30B-A3B-MXFP4`, served at `127.0.0.1:8003` by `serve_omni.py`; `/v1/models` answered, but no native binary forward route was shown. Qwen Cyber has a DSpark configuration in `research-grade-harness/qwen-cyber.env` targeting reserved port 8001; that port had no listener, and no local Cyber checkpoint was found under `~/models`. Neither observation is a model-swap pass. Do not repoint port 8001 or print the environment file's credential values.
