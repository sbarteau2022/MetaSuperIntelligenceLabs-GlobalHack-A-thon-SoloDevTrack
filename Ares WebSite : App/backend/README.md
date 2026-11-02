# Ares Backend — Phase 0 Core Governance

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
  `research_library_v1` corpus plus a new governed `session_memory_v1`
  collection for append-only session summaries + queryable memories; the
  newest summaries/memories auto-tail into session start via
  `load_session_context` (`GET /api/memory/context`). SurrealDB remains
  the governance authority (permits, audit chain, controls, runs) on
  :8000; it is not an MCP server, and `hypermind-mcp` remains an MCP
  server *backed by* SurrealDB.
- Memory encoder disclosure: `session_memory_v1` uses a deterministic
  hashed 3-gram embedding (384-d, L2-normalized, cosine) — real vector
  search for recall + exact payload filters, **not** a neural encoder and
  not a hyperbolic KV. A replacement embedding model can be swapped
  behind `embed_text` without changing append-only semantics.
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
