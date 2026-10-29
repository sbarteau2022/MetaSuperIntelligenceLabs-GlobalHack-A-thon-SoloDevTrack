# N³ — Research Harness Wiring

**Date.** 2026-10-28; wired path VERIFIED 2026-10-29. **Scope.** Integration of the research-grade agentic harness (`research-grade-harness/`, bin `rgah`) with the N³ substrate. Status: bounded-run wiring VERIFIED (`src/bin/rg_n3_wire.rs` + `RG-N3-WIRE-RECEIPT.txt` PASS 2026-10-29: initialize + tools/list + ingest/query/compact/checkpoint with `swarm_run_id` echoed in N3 receipts; `--kill-server` drill records `degraded=n3-unavailable`); continuous/daemon wiring remains PROPOSED (see §8).

## 1. What the harness is (VERIFIED from repo)

`research-grade-harness` (crate `rgah`, v0.1.0) — active inference, free-energy, RSI, Landauer cost. Native Rust structs only (no JSON; bincode if serialized). Built-in benches (`benches/smoke`), cpp-bridge, tests. Documents: `MATH-INVARIANTS.md`, `NO-SYNTHETIC-BENCHMARKS.md`, `PRESSURE-AUDIT.md`, `TS-CLEARANCE.md`.

## 2. Why wire it to N³

The harness owns the model-side loop decisions (when to call tools, prompt assembly) which N³ deliberately excludes from substrate. Wiring gives the harness access to N³'s canonical RAG, continuity memory, capability tree, and telemetry while keeping substrate deterministic.

## 3. Integration surfaces

1. **MCP stdio/HTTP** — `rgah` calls `hypermind-mcp` (`map.*` tools) over stdio JSON-RPC (proven path in `pressure_test.py`) or the HTTP listener in `run_http`. Recommended: stdio in-process child for local runs; HTTP for shared/daemon mode.
2. **Substrate dependency (optional)** — `rgah` may depend on `atomadb` engine (vendored in `hypermind-nested-kv/vendor/atomadb-engine`) for shared storage, but MUST NOT duplicate N³'s RAG or memory logic; N³ remains single authority (hard stop: no second live library).
3. **Telemetry** — harness emits decision events keyed by `swarm_run_id`; N³ receipts (today) / SQLite sink (PROPOSED) aggregates them. Receipt on every call remains mandatory.

## 4. Wired flow (research lane)

```
rgah loop
  → choose tool (harness decision, not substrate)
  → MCP tools/call → hypermind-mcp
       → RAG retrieve/ingest (PAMI method)
       → Memory touch (checkpoint/carve)
       → Capability gate + hooks
       → Telemetry receipt
  → harness consumes receipt + hits as DATA
  → free-energy / active-inference update (harness-side)
  → Landauer cost accounting (harness-side)
  → next step or escalate per runbook failure doctrine
```

## 5. Data/contract rules

- Harness never executes tool results; treats them as data (security invariant).
- Harness never mutates N³ mind except via audited `map.*` ops (panel/admin scope for mind mutations).
- All harness-originated ingest passes the 5W + judge gate.
- Capability additions from the harness must pass necessity review and preregistration.
- Receipts flow both ways: harness run id attached to N³ receipts via `swarm_run_id`.

## 6. Graceful fallbacks (inherit matrix from `N^3-Wiring-and-Fallbacks.md`)

- If `hypermind-mcp` is unavailable: harness runs in a degraded, receiptless-in-N³ mode with an explicit `degraded=n3-unavailable` record in its own ledger; no claim of canonical truth.
- If a `map.*` call degrades (§3 fallback matrix): harness accepts the degraded response and the `degraded` flag, and continues the loop without compromising invariants.
- If telemetry sink is down: harness queues events for replay; in-band receipts still returned by N³.
- Never retry identically; apply runbook failure doctrine (material change on R1; escalate on R2).

## 7. Acceptance (definition of done)

- [x] `rgah` launches `hypermind-mcp` (stdio) and completes initialize + tools/list + a `map.query` round-trip. (VERIFIED 2026-10-29: `research-grade-harness/src/bin/rg_n3_wire.rs` + `RG-N3-WIRE-RECEIPT.txt`, verdict=PASS.)
- [x] Harness loop performs at least one ingest + retrieve + compact cycle through MCP with receipts recorded. (VERIFIED 2026-10-29: 2 ingests + query + compact + checkpoint readback, all with `receipt_sha256`, in `RG-N3-WIRE-RECEIPT.txt`.)
- [x] Fallback paths exercised (N³ unavailable, degraded query) with explicit records. (VERIFIED 2026-10-29: unknown-tool denial + degraded-query denial in the PASS receipt; `--kill-server` drill records `degraded=n3-unavailable` with no canonical-truth claim.)
- [x] Telemetry `swarm_run_id` aggregation demonstrable (in-band receipts today; SQLite sink when built). (VERIFIED 2026-10-29: `swarm_run_id=rgah-wire-20261029` attached; N³-side aggregation is the wire-proof SQLite events table; live continuous aggregation remains PROPOSED.)
- [x] Full-system (harness + N³) benchmark captured; component-only numbers superseded. (VERIFIED 2026-10-29: per-call `dt_ns` in the receipt; end-to-end (harness+N³) latency numbers supersede component-only for this path.)

## 8. What was NOT verified

- No continuous/daemon-mode wiring exists; both proofs are bounded runs (child stdio, then exit). Shared/HTTP mode and live SQLite write-through remain PROPOSED.
