# Ares App — Backend Roadmap and UI Integration

Status: PROPOSED, 2026-11-02. UI is kept as-is. Backend is built from scratch.
Neither old backend is reused (Tauri home-core, AtomaApp/backend — both rejected 2026-11-02).

## Bottom line
One local Rust API server on loopback connects the existing Vite UI to the
live stores and model seats. The UI never talks to a model provider or a store
directly. Everything the UI shows about a run must come from a real run, with
a why-receipt on every failure. No synthetic runs, ever.

## The app, as it stands
Keeper: MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/Ares WebSite : App/
Stack: Vite + React 19 + R3F/WebGPU + React Flow (@xyflow/react) + Zustand.
  Design: obsidian black glass only.
framer-ui/ inside the app is the extracted Framer UI copy (30 files, 2026-11-02).
  It is reference, not a second app. Edit the live src/ once; do not maintain
  two copies of a surface.
wrangler.toml (name atoma-labs-web) is the Cloudflare marketing deploy.
  The local backend is separate from that deploy and is not served through it.

## Ports (do not change without updating the port key)
- 8000 — SurrealDB ONLY, the persistent mind. Nothing else binds here.
- 8002 — oMLX, Qwen3.6-MoE-Combined (the one confirmed local agent).
- 4001 — gateway-3p (cloud seats: Beam, Spark Contributor, Nemotron Ultra).
- 6333 — Qdrant REST (Docker hypermind-qdrant, collection research_library_v1). 6331 is stale, nothing listens there.
- 8080 — HyperDX telemetry (Docker clickstack).
- 4810 — NEW backend API (proposed). Check lsof before binding.
- 5173 — Vite dev server (existing).

## Backend shape
Rust, axum, tokio. One binary, one process. No framework beyond axum + serde + tokio.
Secrets on the Mac only, ~/.config/ares-backend/secrets.env (0600). No key,
  token, or password in git, in the UI bundle, or anywhere under the app folder.
Gateway keys stay in ~/.config/gateway-3p/keys.env. oMLX key stays in
  ~/.omlx/settings.json.
Plugins, skills, and MCP load once at startup and stay live. No per-turn invoking.
SurrealDB tables use RG-AH vocabulary only: thread, message, run, run_step,
  tool_call, receipt, atom, packet. No vendor, host, or IDE names in table names.
Model routing is decided by the backend, not the UI. Default seat: local oMLX.
  Cloud seats via gateway only when the user picks them. Every call records
  which seat served it; a result with no seat recorded is not trusted.
Wonder is a separate seat: GUI with prompt caching required, never via Cline.

## Backend API routes
- GET /api/status — backend, Surreal, Qdrant, oMLX, gateway, each real up/down, no fake ok
- POST /api/threads — create a thread
- GET /api/threads/:id — thread + messages + tool calls + receipts
- POST /api/threads/:id/message — send a message, stream the run (SSE)
- GET /api/runs — list runs, newest first
- GET /api/runs/:id — one run: steps, DAG nodes, status per node, why-receipt on failure
- POST /api/runs/:id/cancel — cancel a run
- POST /api/rag/query — { query, silo?, limit } → Qdrant hits with source_path + document_id
- GET /api/models — local (oMLX) + cloud (gateway) seats, with which seat served the last call
- POST /api/telemetry/event — append-only telemetry event → HyperDX

Every failure returns a why-receipt: what ran, command/route, exit/status,
last output, expected result, evidenced cause, unblocker.

## UI integration map
UI surface (in src/) → what it shows → backend route it uses → phase

- components/cloud/AtomaCloudApp — App shell, cloud surfaces — GET /api/status — Phase 1
- components/cloud/CloudResearchHubView — Research hub, RAG search — POST /api/rag/query, GET /api/models — Phase 3
- components/cloud/CloudSwarmAndMindView — Swarm + mind view — GET /api/runs/:id, GET /api/threads/:id — Phase 3
- components/proof/SwarmDAG — Run DAG, queued/running/VERIFIED/BLOCKED nodes, click node → evidence — GET /api/runs/:id — Phase 3
- components/webgl/ForceGraph3D — 3D mind-map view — GET /api/runs/:id, GET /api/threads/:id — Phase 3
- components/cloud/JudgeWalkthroughAndReceipts — Judge walkthrough + receipts — GET /api/runs/:id, GET /api/threads/:id — Phase 2–3
- components/proof/TelemetryTable, components/ui/TelemetryRow — Telemetry tables, VERIFIED/BLOCKED pills — POST /api/telemetry/event, GET /api/runs — Phase 5
- components/simulations/FlareCodeGraphIDE — IDE chat + project DAG — POST /api/threads/:id/message (SSE), GET /api/runs/:id — Phase 2
- components/simulations/AresRunbookLab — Runbook lab runs — POST /api/threads, POST /api/threads/:id/message, GET /api/runs — Phase 2–3
- components/simulations/OrbitalAgentOS — Agent OS simulation surface — GET /api/status, GET /api/models — Phase 1–2
- components/hero, architecture, proof, brand, pages, layout — Marketing / static surfaces — None (status pill only, GET /api/status) — —

Integration rule: a UI surface that today renders static or simulated data
must not be wired to a fake backend. It stays static until its phase lands,
then it reads the real route. A surface showing a run that did not execute on
hardware is not a run view, it is a mock, and it ships as neither.

## Roadmap

### Phase 1 — Skeleton and status (gate: real up/down, no fake ok)
Axum server on 4810, GET /api/status hitting Surreal, Qdrant, oMLX, gateway.
One Zustand client in the UI hitting /api/status only.
Gate: every dependency reports its true state. A down dependency shows down.
UI touched: AtomaCloudApp shell, status pill, OrbitalAgentOS surface.

### Phase 2 — Threads, streaming, one seat (gate: end-to-end local run)
Threads + messages + SSE streaming to oMLX only. One model seat, end to end.
Receipt on every failure, recorded in SurrealDB, shown in the UI.
Gate: a message sent in the UI produces a real oMLX run, streamed live,
  with steps and a receipt if it fails.
UI touched: FlareCodeGraphIDE, AresRunbookLab, JudgeWalkthroughAndReceipts.

### Phase 3 — RAG and the DAG view (gate: real nodes, real evidence)
POST /api/rag/query to Qdrant (research_library_v1, silos: research, papers,
  aie, specops, knowledge-base).
Run DAG view: React Flow nodes are real run steps, queued/running/VERIFIED/
  BLOCKED with named reasons, click a node → its evidence and receipt.
3D mind-map view reads the same run/thread data as the DAG view.
Gate: every node in the DAG maps to a run_step row; no node without evidence.
UI touched: CloudResearchHubView, CloudSwarmAndMindView, SwarmDAG, ForceGraph3D.

### Phase 4 — Cloud seats, one at a time (gate: seat receipt per seat)
Add gateway seats one at a time: Beam, then Spark Contributor, then Nemotron Ultra.
Each seat gets a seat receipt before the next seat is added.
Wonder is not added here; it is a separate GUI session with prompt caching,
  and it attaches by opening the repo and pointing at its instructions, not
  through this backend's chat route.
Gate: a run through each seat records model_seat on the thread and run.
UI touched: GET /api/models list, seat picker in the cloud surfaces.

### Phase 5 — Telemetry last (gate: events only from real runs)
POST /api/telemetry/event to HyperDX, append-only.
Telemetry tables and pills read real run data, not simulated rows.
Gate: no telemetry event exists for a run that did not execute.
UI touched: TelemetryTable, TelemetryRow.

## What is NOT in this backend (v1)
- No old Tauri home-core, no AtomaApp/backend.
- No Postgres in v1. The store lock is Surreal + Qdrant + HyperDX. Postgres is
  a separate, unreconciled migration plan, not a v1 dependency.
- No per-turn plugin/skill invoking.
- No website rebuild. The marketing surfaces in this app stay as-is; the
  Atoma Labs site redo is Stewart's own work, not this backend's job.
- No cloud relay. The backend on the Mac calls providers directly, keys on the Mac.
