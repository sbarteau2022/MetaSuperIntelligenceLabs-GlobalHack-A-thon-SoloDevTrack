# C-RUN-04 Report (Daemon DAG: long-lived child + live sink + multi-packet fan-out/fan-in)

**Date.** 2026-10-30. **Manifest.** `C-RUN-04-Manifest.md` (frozen; SHA-256 filed below before execution per C09.1/C11.3).
**Binary.** `hypermind-mcp` debug (F1/F2/F3, 46 bin tests green), `rgah` test profile `engine_n3`.
**Hardware.** HW-M3MAX primary (this Mac). Replica pending per C09.4.
**Commit.** Filed at commit time (this report + manifest + test committed together).

## Manifest hash

filed 2026-10-30 BEFORE execution: `sha256:c7e21298376694b6fa04a3ae094db0b7c135b272091390dbd4e263750f3806ab` (recompute: `sha256sum C-RUN-04-Manifest.md`).

## Results

| Check | Result | Evidence |
|---|---|---|
| (3) DAG 5/5 pass | **PASS** | `seed`, `qa`, `qb`, `gate`, `collect` all Passed through `run_parallel_n3`; topological levels 0→1(fan-out)→2→3(join). One `McpChild` spawned once, closed after assertions — ≥7 tool calls through one stdin/stdout pair |
| (1) daemon | **PASS** | Single child lifetime ⊃ whole run; state accumulated across packets (seed ingests read back by qa/qb/collect); all receipts `outcome: PASS`, all packets `status: Passed` |
| (2) live sink | **PASS** | `verify().is_ok()`, `len() = 10` = 5 distinct call-rows + 5 verdict rows, chain ok. Test stderr: `C-RUN-04: 5/5 DAG pass, sink events=10 chain=ok` |
| Judges | all per-packet contracts met | seed→receipt_sha256; qa→`bravo seven`; qb→`charlie nine`; gate→`retrieved_ids`; collect→`daemon` |

## Instrument history (corrections are entries — v1→v4 preserved)

- **v1** (`ca17cd5c…`): join = attend-only judged on text. Failed: attend returns gate metadata (IDs, no text) — judge-contract error, not substrate.
- **v2** (`0ba205d5…`): join = attend+query two-call packet. Failed: `execute_packet_n3` judges per-call conjunctively (`engine.rs:123`) — attend output can never satisfy a text needle. Engine semantic confirmed correct (no engine change); instrument wrong.
- **v3** (`dcd41c5b…`): join stage = `gate`+`collect` two packets. 5/5 PASS, sink verifies — but event count 10, not the filed 11. Manifest-authoring error: `record_call` idempotency keys are (packet,attempt,tool), so the seed's two ingests collapse to one row (tested sink behavior).
- **v4** (`c7e21298…`): sink math corrected to 10 rows + idempotency note. **PASS.**

## Follow-up gaps filed (NOT in this run)

- **Sink idempotency granularity.** (packet,attempt,tool) keys lose per-call rows for multi-same-tool packets. Finer keys (packet:attempt:tool:index) prescribed — sink change, new tests, no run impact until implemented.
- **Per-call vs joined judging.** `execute_packet_n3`'s conjunctive per-call judging is CORRECT for single-contract packets but makes multi-contract packets unpassable without splitting. If multi-contract packets are ever needed, the engine needs joined-output judging as an explicit, tested semantic — design change, not a patch.

## What this run licenses (and does not)

- Licenses: the three FULL-SMOKE NOT-VERIFIEDs retire to VERIFIED on primary (single-machine, debug): daemon lifetime, live write-through sink, multi-packet DAG with fan-out/fan-in. Replica per C09.4 before any wider claim.
- Does not license: cross-hardware claims; prompt-driven PPC separation (Track D); full-system claims beyond the daemon+DAG+sink path.
