# GlobalMind bridge contract

The research harness should use the single `globalmind-runtime` SurrealDB substrate for graph recall and model-bound tensor K/V. Qdrant remains the fact library and HyperDX remains telemetry. The old `MapPlugin` checks a text `map.ingest` and string `map.kv_store` MCP; it does not satisfy the GlobalMind contract and must not be promoted as tensor K/V.

The bridge boundary is read-only, typed, and binary at model I/O. It verifies the isolated GlobalMind Surreal schema, retrieves an H3×T2 neighborhood using the existing runtime, and returns the runtime's `GMRC` binary bundle. It rejects quarantined `hypermind/core` by default, malformed coordinates, empty provenance, and candidate-budget overflow. It never mints atoms or creates a second map. Model-bound K/V requires an exact identity query over the session and page before bytes are exposed; page ID alone is insufficient. That exact lookup is implemented and qualified for the Layer-3 candidate.

The exact K/V query binds workspace, model ID/revision/digest, request and prefix digests, TPPE version/configuration digest, address and tensor contract digests, geometry space, session dimensions and dtype, plus page layer/head/token interval/shape/layout/address/phase digest/revision and expected key/value digests. The store rejects zero or multiple matches, then rehashes returned bytes. The query is read-only and does not update hits or access time. The harness exposes this typed call after the isolated candidate-v2 qualification passed a real baseline and 20 identity misses.

The current candidate-v2 Layer-3 hardware receipt proves exact-model tensor K/V after restart and logit parity. It does not prove full-model decode or a live harness forward pass. Promotion requires a real harness call through this bridge and a matched K/V identity receipt, then a forward-pass smoke.

The capability DAG manifest was consolidated under `N3/CapabilityTreeSitter`; the harness's A14 gate must read that canonical path so the full suite exercises the real consolidated manifest.

Bridge acceptance uses an isolated SurrealKV process: apply the canonical schema, seed one actual atom and H3×T2 address, connect through `GlobalMindBridge`, retrieve one binary GMRC bundle, decode it, and verify the returned atom identity. A dead endpoint and `hypermind/core` must fail before a model call. This proves the live bridge read path, not model inference.

The same isolated bridge test should create a typed tensor session and one small f32 K/V page through `GlobalMindStore`, fetch it through the harness bridge with the full exact identity, and prove that mutating a model digest returns a miss. This proves the harness-to-store K/V route; the real Qwen Layer-3 hardware receipt remains the separate payload and logit-parity proof.

The explicit `tests/globalmind_bridge_live.rs` acceptance passed against a disposable SurrealKV instance on 2026-11-02. It applied the canonical schema, seeded one atom/address, connected the actual bridge, decoded the expected single-atom GMRC bundle, created one typed tensor session and K/V page, fetched exact bytes through the bridge, and rejected a changed model digest. The process and its temporary store were removed after the test.

## Forward recall binding gate (preregistered 2026-11-02)

For a recalled neighborhood `N` and exact K/V identity `I`, preparation may return tensor bytes only if `I.address` is an address of a member of `N`. Exact model identity alone proves that the page belongs to its session; it does not prove that the page belongs to the selected recall. The oracle for a valid join is set membership, `I.address ∈ {m.address | m ∈ N}`. A two-atom counterexample is the falsifier: retrieve only atom B while asking for a valid exact K/V page attached to atom A; both independent reads succeed, but preparation must fail. The rule changes no retrieval metric, payload, model weights, or cache key. Test order: one-atom positive, two-atom mismatch, changed-model-digest mismatch. This is a boundary safety proof, not inference or full-model quality evidence.
