# C-RUN-01 Manifest (FROZEN — Daemon DAG: long-lived child + live sink + multi-packet fan-out/fan-in)

**Date.** 2026-10-30. **Status.** [PROPOSED] Manifest frozen; no RUN has occurred.
**Rule.** SHA-256 of THIS file recorded in the run report before execution (C09.1/C11.3). Any edit → C-RUN-02 + new hash.
**Parents.** `N^3-Research-Harness-Wiring.md` (child-stdio path); `N^3-Wiring-and-Fallbacks.md` (fallback drills); `FULL-SMOKE-RECEIPT` NOT-VERIFIED section (this run closes its three items).
**Model.** None — determinism + wiring probe, no model calls, no PPC. `Qwen3.6-MoE-Combined` reserved for Track D.

---

## 1. What this run proves (the three NOT-VERIFIEDs, one run)

| # | NOT-VERIFIED claim | This run's proof |
|---|---|---|
| 1 | Daemon mode (every run used short-lived stdio children) | ONE `McpChild` held open for the entire 4-packet DAG (≥8 tool calls through one stdin/stdout pair, no respawn). Child lifetime spans seed→fan→join; kill only at run end. Long-lived = many sequential calls, one process, state accumulates |
| 2 | Live write-through sink (no live sqlite writer path exercised end-to-end) | `execute_packet_n3` already records every call + verdict (`sink.record_call`/`record_verdict` — code-verified at `engine.rs:96,114,134,390`). This run asserts the sink chain AFTER the daemon run: `verify().is_ok()` + event count == expected (4 packets × calls + 4 verdicts) + spot-read of one event row |
| 3 | DAG execution beyond the 2-packet seed→chain | 4-packet DAG with fan-out AND fan-in through `run_parallel_n3`: `seed` → {`qa`, `qb`} (one topological level, `max_parallel`≥2) → `join` (depends on both). All four judged live on returned text |

## 2. Frozen parameters

| Parameter | Value | Provenance |
|---|---|---|
| DAG | `seed` (ingest 2 texts) → `qa` (query text-A), `qb` (query text-B), both `depends_on: [seed]` → `join` (`map.attend` over A-terms, `depends_on: [qa, qb]`) | Fan-out/fan-in minimum shape; `run_parallel_n3` topological levels exercised (level 0: seed; level 1: qa+qb; level 2: join) |
| Corpus | `seed` ingests `daemon packet alpha bravo seven` + `daemon packet alpha charlie nine`; `qa` queries `alpha bravo`; `qb` queries `alpha charlie`; `join` attends `daemon alpha` | Distinct texts per branch so fan-in is meaningful (each branch retrieves its own text) |
| Judges | Per-packet `Contains`: seed→`receipt_sha256`; qa→`bravo seven`; qb→`charlie nine`; join→`daemon` (attend returns fused hits containing the term) | Each stage judges its own contract (live-test pattern) |
| Child | Single `McpChild::spawn(bin, root)`, `HYPERMIND_ROOT` fresh temp dir; `mcp.close()` only after report assertions | Daemon = process lifetime ⊃ run lifetime |
| Sink | `TelemetrySink::open(temp sqlite3)` passed `&mut` through the whole run; post-run: `verify()`, `len()`, one `replay_from(1)` row read | Write-through proven by read-back, not by code inspection |
| Gate | Canned manifest admitting `map.ingest`, `map.query`, `map.attend` (+ `map.lexical` unused — proves admission is per-tool, not blanket) | Capability-gate stays in the loop on the daemon path |
| Timeout | 300s wall (expected < 60s) | — |
| Hardware | HW-M3MAX primary (this Mac) | File convention |

## 3. Decision mapping (binds the report before it exists)

| Outcome | Reading |
|---|---|
| 4/4 pass, sink verifies, event count == 8 calls + 4 verdicts + gate rows | C-RUN-01 PASS: daemon + live sink + multi-packet DAG proven on primary. The three NOT-VERIFIEDs retire to VERIFIED (single-machine); replica per C09.4 before any wider claim |
| Child dies mid-run (transport-error) | RUN VOID, not FAIL — infra defect; root-cause (OOM? pipe break? VP rebuild?) filed, re-run under same manifest + new run hash |
| Any packet FAIL on judge (not transport) | FAIL with receipt: which stage, returned text, judge needle. DAG semantics suspect (fan-in join contract?) — file and diagnose, no silent re-run |
| Sink verify fails | FAIL: write-through path broken under daemon lifetime (the exact gap this run hunts). Highest-value failure — file with chain state |

## 4. Run report requirements (acceptance)

Per C09.3: child lifetime (spawn→close timestamps, call count through one pair), per-packet verdicts, sink `verify()` + `len()` + sample row, gate admissions, wall time, hardware pins, manifest SHA-256, commit hash. Raw sink DB retained as artifact (temp dir path recorded), not committed.
