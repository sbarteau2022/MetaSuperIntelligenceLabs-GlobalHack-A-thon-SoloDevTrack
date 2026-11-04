# Ares Backend — Phase 0 Core Governance

## 2026-11-04 source recovery and live-pass boundary

This backend source was recovered from Git checkpoint f735d45e after the Ares
site rebuild removed it from HEAD. The running :4810 backend remains the
older binary and was not replaced. Source inspection and a binary string
check found that its mission executor forged generated token IDs from an
MCP receipt, called a hash of MCP text a logits digest, and supplied
placeholder model artifact fields. The recovered source now makes
MindRecallExecutor MCP-only: tool calls can carry genuine MCP receipts,
while bound model calls fail closed. The mission response identifies that
executor as mind-mcp-only. A real Glimmer model-harness-MCP/KV pass,
automaton MTP/speculative balance, and 30–50-step quality audit remain
unverified. Do not treat the still-running :4810 binary or its prior PASS
responses as model-forward certification. See MODEL-SEAT-STATUS-DESIGN.md.
The dormant NativeMtpExecutor now also refuses bound-output calls: its
FlatBuffers stream provides generated text and seat identity, but not real
logits, artifact digests, or exact K/V. Ordinary generation still yields a
backend-computed text binding digest; it is not a model-forward receipt.
The Glimmer HTTP seat no longer contains an unused placeholder model
artifact; its bound call remains unavailable pending real attestation.
Its component forward parser now recomputes the server's receipt over the
submitted input IDs, returned output IDs, and logits digest, rejecting a
tampered or mismatched response. This does not verify the logits themselves.
The live runner now marks every unsupported tool call as a packet failure;
declared calls can no longer disappear from a mission's audit trail.
It also rejects `ToolCall.declared=false` before any wire call.
The bound-output receipt now carries the exact native K/V digest. The live
runner rejects missing tool receipts, malformed model/K/V digests, changed
input bytes, and a mismatched receipt chain before accepting a packet.

The isolated `hypermind-toy` crate rehearses the forward-pass scheduler. Its
`backend_receipt_parity` integration test feeds receipts minted by this backend
into the toy's independent chain verifier; scheduler tests remain rehearsal
evidence, not a live model forward-pass qualification. Run it with an external
target directory because this checkout path contains a colon:
`CARGO_TARGET_DIR=/tmp/hypermind-toy-target cargo test --manifest-path 'hypermind-toy/Cargo.toml'`.

Qwen VL dense research retrieval is specified in
`QWEN-VL-RETRIEVAL-DESIGN.md` with a pinned binary MLX oracle fixture.
The Rust Qwen VL tokenizer adapter has been restored for the hackathon pass.
Its pinned digest and oracle token sequence are a model-specific boundary
check, not shared atom identity or full encoder parity. Further MLIR lowering
is paused; see `../../HACKATHON-SWARM-OVERWATCH-DESIGN-2026-11-03.md`.
The shared carved-swarm seat uses the pinned **Glimmer** tokenizer and config
from the WD Drive (`harness::glimmer_tokenizer`); Qwen remains the separate
research-library intake adapter. Glimmer's 6656-wide hidden state is not an
admitted semantic embedding. The live tokenizer gate is
`cargo test --test glimmer_tokenizer_live -- --ignored`.
The live text route remains BM25 until a Rust-native 28-layer VL executor
passes the fixture, stored-point, and Qdrant self-retrieval gates.

The native MTP adapter now pins the server-reported 64-hex model identity on
its first successful generation and rejects an identity change within the
same executor/mission. An intentional model swap creates a fresh executor and
must invalidate incompatible prefix and tensor K/V pages. This protects a
mission from mixed-model receipts; it does not supply a native Glimmer forward
executor. The current adapter still sends textual tool/args input, and
`/api/models` now reads the native server's binary `/v1/info` metadata frame
and reports a loaded identity/profile only when that probe succeeds. Its oMLX
and gateway entries remain separately probed. The AMST transport parser no
longer imposes Qwen's vocabulary size on another adapter. See
`../../FORWARD-SMOKE-BRIGADE-HANDOFF-2026-11-03.md` for the remaining typed
model-seat and carved-swarm gates.

The live `/api/harness/mcp/execute` route now requires an explicit
`ARES_MCP_MIND_URL`, `ARES_MCP_MIND_NS`, and `ARES_MCP_MIND_DB` outside the
quarantined `core` database. It spawns `globalmind-mcp` (override with
`ARES_MCP_MIND_BIN`), supplies the existing governance client's credentials
to that child, and checks the child's `mind.db_status` authority and contract
before automaton-gated dispatch. Governance receipts still persist in the
backend database. A well-formed MCP response alone is not an independently
verified task outcome, so this route does not update confidence from that
response. The request must carry an explicit valid role and exact
`URL|namespace|database` map identity. Until scoped capability handles are
bound end to end, L0 can invoke unscoped recall or exact K/V through this
route. `/api/harness/swarm/scope` now accepts a configured L0 issuer token,
mints an ephemeral 256-bit handle bound to role, well IDs, map identity,
prefix identity and expiry, and records a coordinator permit. A worker's
`mind.recall` request is rewritten to `mind.recall_scoped` with the handle's
wells; caller-supplied wells are replaced. Worker exact K/V remains denied
until a separate tensor entitlement is defined. The full shared-atom recall plus exact model-bound K/V use loop is
specified in the handoff and still needs a real model task gate.

Status: LANDED (core skeleton), 2026-11-02. One Rust binary (axum + Tokio),
loopback `127.0.0.1:4810`. Governance is baked in, not added on: **no
dispatch, no memory write, no model call, no erasure, and no run ships
without a compliance permit, an audit-chain entry, and a Merkle anchor** —
from the first skeleton build onward.

Design source (read from Mac Trash, never imported, server not live):
`monster-hybrid-server/immune_system/implementation/src/compliance.rs` +
`first_principles.rs`, translated to backend-native equivalents per the
2026-11-02 extraction + core governance spec. The workflow it serves is
`N3/system-workflow.pdf` §6 (Mission → Packet → Run → Verdict → Receipt →
Atom) over three stores by job: SurrealDB (mind graph, :8000), Qdrant (RAG
with provenance, :6333), HyperDX (telemetry, :8080).

## How the system works (native systems posture)

- **Governor cannot move without a permit.** `governor::dispatch_*` takes
  the `ImmuneSystem` lock, runs the gate, and only then works.
  `CompliancePermit.sequence`/`chain_digest` are private to
  `compliance.rs` — the unauthorized path does not exist in code; there is
  no "compliance off" flag and no debug bypass.
- **Fail-closed startup.** Schema → seed 12 controls → load audit chain →
  verify genesis→head. A broken chain refuses startup with a why-receipt
  (proven live: tampering one stored `action` → exit 1, "entry 2 digest
  mismatch (log tampered)"). The backend never runs ungoverned.
- **Audit chain.** Entry digest =
  `SHA-256(seq LE ‖ approved u8 ‖ mark tag ‖ prev_digest ‖ action)`.
  Rejections are entries too (`approved: false`). Verified independently
  (Python hashlib recomputation from SurrealDB rows, not the backend's own
  arithmetic).
- **Merkle vault.** RFC 6962 (`0x00` leaf / `0x01` node prefixes), odd-node
  carry-forward without self-duplication (CVE-2012-2459 fix), constant-time
  digest comparison. `GET /api/compliance/vault/:root` recomputes from leaves.
- **Carved views.** 12 lanes (RG-AH vocabulary: mission_intake …
  gate_keeper) + L0 coordinator. A view that does not contain a row cannot
  return it; Ω = 0 pairs have no handoff path (not a runtime denial).
  Default Ω = 0.08 between workflow-adjacent lanes; expansion needs a
  written justification or `UnjustifiedOverlapExpansion`; negative Ω fails.
  `ScopedGraphRead` now lends a worker only its carved atom reads and checks
  both endpoints of a proposed transition. It borrows the same frozen-weight
  `Arc` without copying it. See `CARVED-GRAPH-READ-DESIGN.md`; SurrealDB/MCP
  enforcement, real Glimmer execution, and L0 quality merge remain open.
- **The 12 gates** (G1 weights, G2 spawn, G3 handoff, G4 loss accounting,
  G5 kill criteria `0x94ce_0283_17e0_ecac`, G6 forward pass, G7 supervision,
  G8 deposit, G9 hot swap, G10 cross-epoch, G11 MLIR SSA, **G12 human
  oversight** — backend addition: core-scope changes need a single-use
  human permit; the system can propose, never self-approve).
- **Statutory registry.** The 12 rules (GDPR/EU AI Act/SOC2/NIST CSF 2.0/
  ISO 42001, verbatim ids) evaluated from **live state**: each invariant is
  proven against the running system (planted-probe carve test, live
  erasure with 0-residual + root change, Arc-identity across views…).
  A rule with an unproven invariant is NOT credited — with an empty audit
  chain, AIA-ART-12 correctly reports unsatisfied.
- **Surgical erasure.** Pre-root → excise → re-carve 12 views → residual
  scan across 13 maps → post-root. The receipt carries the residual count
  either way.

### Disclosed deviations from the source (both fail-closed)

1. **Gate 7 enforces the restart cap.** Source records the params and
   discards them; spec forbids silently copying the gap → over-cap
   restarts are rejected (`RestartCapExceeded`) and audited.
2. **Gate 12 (human oversight) is a backend addition** implementing the
   core spec's "human oversight is a gate, not a notification."
3. **Lane adjacency replaces icosahedral geometry** for the default Ω
   matrix (workflow-stage adjacency, stated as data — not a geometric claim).

## Build & run (this Mac)

The repo path contains `:` and a space, which breaks Cargo's linker-path
joining (`DYLD_FALLBACK_LIBRARY_PATH`). Build with an external target dir:

```sh
CARGO_TARGET_DIR=/tmp/ares-backend-target cargo build --offline
CARGO_TARGET_DIR=/tmp/ares-backend-target cargo test --offline   # 12 acceptance tests
/tmp/ares-backend-target/debug/ares-backend                      # binds 127.0.0.1:4810
```

SurrealDB credentials: `~/.config/ares-backend/secrets.env` (0600) if
present, else discovered from the live `surreal start` process args.
Nothing secret is in this crate or in git.

## Verification receipts (2026-11-02, M5 Pro arm64)

- `cargo test` — 12/12 pass (`tests/governance_core.rs`, one per acceptance line).
- `cargo clippy --all-targets -- -D warnings` — clean. `cargo fmt --check` — clean.
- Live boot against SurrealDB 3.3.5 (`ares`/`ares`): schema + 12 controls
  seeded; spawn dispatch minted permit seq 2 anchored in SurrealDB;
  independently recomputed chain `6142ddb1…` matched stored head.
- Startup tamper test: forged stored action → startup refused, exit 1.
- `/api/status` reports real up/down (oMLX :8002 down — process not running;
  shown down, never faked) and `governance.ok` is false until the chain
  has entries and all 12 controls have evidence.

## RG-AH harness — native runtime (landed 2026-11-02; store layer recut 2026-11-02)

The research-grade harness is implemented as native backend modules in
`src/harness/` (mission.rs, runtime.rs, telemetry.rs, qdrant_memory.rs) —
backend-owned, never linked against the `rgah` crate. Framework: the Rust
& Tokio systems-engineering doctrine (bounded concurrency via semaphore +
`spawn_blocking`, explicit timeouts/byte caps, secrets at call time from
0600 `secrets.env`).

- Sources honored as design: runbook (Plan/Packet DAG, judges mandatory),
  judges (ExitCodeZero/GoldenHash/Contains/NotContains — coders never
  self-approve), chain-authenticated receipts (GENESIS mint; a forged
  prior skips nothing on resume), mission fail-closed escalation,
  capability gate (no judged atom + receipt → `map.*` call denied),
  research gate, session chain, working-set cache, frozen policy kernel,
  energy budget as _receipt fields only, never a permit_.
- Store ownership, corrected (three-store lock, not collapsed):
  **HyperDX (localhost:8080 UI; OTLP collector 4317/4318) is the
  telemetry/tracking store** — OTLP logs/traces land in ClickHouse
  `default.otel_logs` / `otel_traces`, append-only. The prior SQLite
  spine (`store_sqlite.rs`, `ares_events.sqlite3`) is retired and removed
  from the backend. **Qdrant is the RAG + memory store** — the existing
  `research_library_v1` corpus plus a separate `session_memory_hash384_v2`
  collection for append-only session summaries + queryable memories; the
  newest summaries/memories auto-tail into session start via
  `load_session_context` (`GET /api/memory/context`). SurrealDB remains
  the governance authority (permits, audit chain, controls, runs) on
  :8000; it is not an MCP server, and `hypermind-mcp` remains an MCP
  server *backed by* SurrealDB.
- Memory encoder disclosure: the active backend selects the TPPE candidate
  collection `session_memory_tppe_v1`, with a 384D vector built from lexical
  content and a TPPE positional state. It is neither a neural embedding nor
  transformer K/V, and its retrieval quality is not yet certified. The old
  TPE hybrid was removed; the interim lexical `session_memory_hash384_v2`
  collection remains separate and empty. The research library keeps its
  existing 1024D dual-rank contract. See `STORE-SUBSTRATE-DESIGN.md`.
- 2026-11-02 cutover: deleted the two old `session_memory_v1` points at
  user request; `session_memory_hash384_v2` starts empty. Startup checks
  both Qdrant vector contracts. The 48,422-point research library was
  preserved. Its recorded ingest used Qwen3-VL-Embedding-2B-6bit-RUN
  (the first 1024 final-hidden components, normalized, plus a rotated rank);
  see `N3/Quadrant Brain Library RAG/KB_INGEST_RECEIPT_2026-11-02.md`.
  The backend's current RAG endpoint scrolls payloads; a Qwen query
  encoder and vector search have not yet been wired into this backend.
- Typed research-vector retrieval is now available in Rust as
  `QdrantMemory::query_research_rank0`: it validates a normalized 1024D
  rank0 vector, derives rank1 with the recorded rotation, searches both
  Qdrant vector names, and fuses hits by reciprocal rank. The live test
  `tests/qdrant_research_live.rs` retrieved a stored SpecOps point as its
  own top hit and checked rank1 parity (2026-11-02). Text-to-vector Qwen
  inference remains the separate promotion gate for `/api/rag/query`.
- Live Qdrant check (2026-11-02): `session_memory_tppe_v1` has the named
  384D cosine vector and six payload indexes. `tests/qdrant_store_live.rs`
  verified its startup contract alongside the research library.
  `tests/qdrant_tppe_memory_live.rs` appended one true cutover event and
  retrieved it by recency tail and TPPE vector query from a new client;
  the collection has one point. This is an operational smoke, not a
  retrieval-quality certification.
- GlobalMind forward-pass status: the exact-model Layer-3 tensor K/V
  hardware qualification was rerun on the current TPPE contract and
  passed all seven gates. Full-model parity and the harness connection
  remain open. The research harness's old text/string `MapPlugin` is
  separate from typed GlobalMind graph and tensor K/V; see
  `STORE-SUBSTRATE-DESIGN.md`.
- HyperDX key handling: the ingestion key lives only in
  `~/.config/ares-backend/secrets.env` as `HYPERDX_API_KEY` (mode 0600,
  verified) and is sent only in the OTLP `Authorization` header — never
  in payloads, telemetry attributes, logs, or the UI bundle.
- Inference boundary: `harness::runtime::Executor` — the engine never
  calls a provider directly. Governed dispatch = compliance permit
  (Gate 2) + bounded run slot + harness execution. Live targets (oMLX
  :8002, native `rust-server` AMST, hypermind MCP) plug in behind it.
  Today no executor is wired, so missions ESCALATE with receipts — the
  honest state, proven end-to-end (permit + Surreal run row + audit
  chain, restart-verified).
- Routes (this layer): `GET /api/harness/status`,
  `POST /api/harness/mission/geometry`, `POST /api/harness/dry-run`,
  `GET /api/memory/context`, `POST /api/memory`,
  `POST /api/memory/query`, `POST /api/telemetry/event`,
  `POST /api/rag/query`.
- Tests: `tests/harness_core.rs` 10/10 + governance 12/12. Source-crate
  baseline recorded: `rgah` 45 unit + 36 integration pass;
  `a14_run01_determinism_50k` fails pre-existing on a missing external
  CapabilityTreeSitter manifest (environmental, not a port defect).
- Live verification (2026-11-02): boot created `session_memory_v1`
  (restart: `created_new=false`); appended 2 memories across a restart
  and tailed them into session context; `/api/memory/query` scored the
  store-answer memory 0.562 > context 0.299; `/api/rag/query` returned
  research hits with provenance; `/api/telemetry/event` was accepted
  (HTTP 200) and read back from ClickHouse
  `default.otel_logs WHERE ServiceName='ares-backend'` (2 rows incl.
  a restart-proof event). A clean re-boot resumed all three stores.
- Architecture rulings recorded: the live forward pass is `RUST SERVER`
  MLX with **regular fixed-cap 512-token KV** (36 tok/s set); hyperbolic
  KV is bench (`ablation_benchmarks/`) + retrieval geometry
  (`hypermind-nested-kv`) + PROPOSED docs — **zero files in any live
  forward path**. The nested KV in the mind map is a string provenance
  cache + atom map, not a tensor KV. Any future hyperbolic-KV-in-forward
  -path work enters as a kernel-register candidate: necessity ablation
  vs the fixed-cap incumbent, same machine/inputs, one variable changed.

## Not yet (deferred, per roadmap phases)

- Phase 1–2: threads/message SSE streaming to oMLX, seat recording per call.
- `/api/compliance/core-scope` route (Gate 12) is implemented in `http.rs`
  as `core_scope_change` but not yet mounted in the router.
- Phase 3: neural/semantic RAG scoring (today `/api/rag/query` returns
  scroll+payload-filtered `research_library_v1` hits with provenance;
  embedder/embedder wiring is explicitly not faked) + the run DAG view.
- SurrealDB / hyperKV / MCP server integration is being ironed out
  separately (owner, 2026-11-02): this backend keeps SurrealDB as the
  governed persistence authority until that lands; memory in Qdrant and
  telemetry in HyperDX are unaffected by its outcome.
- UI wiring (status pill in `AtomaCloudApp` footer) — UI untouched so far.

Research vector route: `POST /api/rag/query` accepts a normalized 1024D `rank0` array from the pinned Qwen encoder, searches both named Qdrant ranks, and fuses hits by reciprocal rank. Text-only queries use the separate Qdrant BM25 lane; dense text queries still require a Rust-native Qwen encoder matching the ingest recipe. Omit both `rank0` and `query` for payload scroll.

Qdrant `research_library_v1` now has a scored full-text payload index on `text`, built from the 48,422 stored points without changing their IDs or two Qwen vector ranks. `/api/rag/query` routes a nonempty `query` to Qdrant BM25 and labels it `qdrant-bm25-text`; a supplied normalized `rank0` still takes the dual-rank Qwen vector path. BM25 is a lexical retrieval lane, not the Rust-native Qwen VL query encoder or full PAMI. Startup verifies the scored index as well as the vector schema. The official index/query contract is documented at https://qdrant.tech/documentation/search/text-search/full-text-search/.
