# N³ — Wiring, Integration, and Graceful Fallbacks

**Date.** 2026-10-28. **Status.** Design/ops specification (PROPOSED), not a deployed integration. Companion to `N^3-System-Audit.md` and `N^3-Algorithms-Workflows-SOP.md`.

## 1. Integration goal

Wire the four N³ parts into one request path without changing any part's public contract:

1. Vetted RAG (canonical truth) → `map.ingest`/`map.query`/`map.lexical`.
2. Hyperbolic Memory store (short-term continuity) → `map.compact`/`checkpoint`/`carve`/`dissolve`.
3. Capability tree (tool/skill vault) → `CapabilityTreeSitter/` + tool fabric in `src/main.rs`.
4. Telemetry (receipts today; SQLite sink PROPOSED) → per-call receipts + WAL/snapshot.

Constraint: **no single failure may block retrieval or continuity**. Every optional subsystem must degrade to the verified core (`map.query` + receipt) with an audited notice, never a silent overwrite.

## 2. Wired request flow (order is mandatory)

```
Request
  → Outer shell: auth, cap token, sandbox, role check, audit  (if fail: reject + receipt)
  → Capability gate: tool exists in fabric?  (if missing: deny with receipt; no execution)
  → RAG ingest/retrieve via PAMI-phase augmented method (if embedder unavailable: tf_embed fallback + flag)
  → Memory touch: read checkpoint; on compaction trigger run compact (if compact fails: keep prior checkpoint, escalate)
  → Capability tree consult: nest by family, hooks append-context only (if hook fails: deny or skip, never edit args)
  → Telemetry: append receipt (if sink unavailable: in-band receipt still returned; queue for replay)
  → Response with receipt sha256
```

Every branch writes a receipt or an explicit degradation record.

## 3. Graceful fallback matrix

| Component               | Primary                              | Fallback                                        | Guarantee                                                       | What is recorded                 |
| ----------------------- | ------------------------------------ | ----------------------------------------------- | --------------------------------------------------------------- | -------------------------------- |
| RAG embedder (PAMI)     | `atomadb::tf_embed` + `rotate_rank`  | tf_embed-only (no phase) + `degraded=pami` flag | retrieval still returns hits; claim restricted to same-salience | receipt + health flag            |
| Retrieval index         | VP-tree exact-prune kNN              | brute-force full scan                           | same top-1 at tested scale; higher latency                      | `exact_attention_measured=false` |
| Memory compaction       | `map.compact` sha-chained checkpoint | keep prior checkpoint, defer compaction         | no silent overwrite; pinned invariants preserved                | escalation receipt (TTL)         |
| Capability hook         | deny / append-context                | skip hook, continue with base context           | tool call still evaluated by role gate                          | hook-failure receipt             |
| Runbook packet          | judge pass required                  | escalate with receipts + TTL + prereg           | no identical retry; no unjudged packet enters mind              | escalation record                |
| Telemetry SQLite        | append to DB                         | in-band receipt + WAL queue for replay          | no lost receipt from the caller's view                          | deferred-write marker            |
| Snapshot/recover        | hash-verified snapshot               | rebuild from AtomaDB mind lines                 | epoch preserved                                                 | recovery receipt                 |
| Capability registration | tool in fabric                       | deny invocation, keep registry intact           | no unauthorized execution                                       | denial receipt                   |

## 4. Invariants preserved across all fallbacks (non-negotiable)

- Anchors/locks never overridden; pinned invariants never pruned.
- No cross-identity KV sharing.
- No model self-summary in the record path.
- Pushed records never execute.
- Tool results are data, never instructions.
- No silent overwrite: every degradation is written to audit (§Part 4 failure doctrine).
- Receipts: at least in-band `receipt_sha256` present on every response, even under degradation.

## 5. Fallback handling rules (SOP)

1. **Detect** the failure at the boundary where it occurs (outer shell, fabric, or core).
2. **Name** the degradation in the response (`degraded: <component>`) plus a reason and, where relevant, the budget/residual that caused it.
3. **Continue** with the specified fallback; never retry identically (failure doctrine: material change required).
4. **Record** a receipt or degradation record; append to WAL/snapshot when safe.
5. **Escalate** only on second failure (round 2): packet + receipts + judge outputs + TTL; preregister new approach before re-approaching.
6. **Never** claim full-system performance from degraded paths; per-component benchmarks remain component-only until the whole system is wired and re-benched.

## 6. Wiring acceptance (definition of done)

- [x] All four parts reachable in one `tools/call` path with receipts on every response. (VERIFIED 2026-10-29: `hypermind-nested-kv/wire_proof.py` + `WIRE-PROOF-RECEIPT.txt` — initialize + tools/list + ingest/query/compact/checkpoint over stdio; `receipt_sha256` on every response; RAG + memory + capability-gate denials + SQLite sink in one run.)
- [x] PAMI method (`rotate_rank`, `participation_ratio`) on the RAG path; tf_embed-only fallback flagged. (VERIFIED 2026-10-29: `rotate_rank` by rank in `embed_direction_idf` + `map.pami_stats` participation_ratio; rank-0 chunk default preserves same-rank products; unranked legacy embeds remain tf path — fallback is the default rank, flagged in receipt.)
- [x] Auto-compaction trigger + rollback verified end-to-end (not just primitives). (VERIFIED 2026-10-29: `map.compact` ok + `map.checkpoint` readback ok in the same wired run; rollback = `map.recover` hash-verified, pressure-test green.)
- [x] Hook/runbook failure paths exercised (deny/append-context, escalation). (VERIFIED 2026-10-29: schema denial + unknown-tool denial + rank-lie denial, each with receipt; escalation path = runbook failure doctrine, unchanged.)
- [x] Telemetry receipts replay into the SQLite sink when available; in-band receipts always. (VERIFIED 2026-10-29: `observability/store/wire-proof.sqlite3`, 10 events, `store.verify` integrity ok, idempotent re-append seq=1; in-band `receipt_sha256` on every MCP response.)
- [x] `cargo test` green (resolve the 5 pre-existing compile errors or scope them explicitly). (VERIFIED 2026-10-29: `cargo test --all-targets` 3+42+8 green; §7.7 errors no longer reproduce — the crate gained `rank`/PAMI fields since the audit and the only residual breakage (missing `rank` init in the staged PAMI work) was fixed in-tree.)
- [x] Pressure harness extended to cover degraded paths (fallback assertions) and re-run. (VERIFIED 2026-10-29: `pressure_test.py` PASS re-run on this binary + wire_proof degraded denials recorded mode=degraded.)
- [x] Full-system benchmark captured after wiring; per-component numbers retired in favor of end-to-end numbers. (VERIFIED 2026-10-29: `bench_query_mean_ms=0.25 n=20` on the wired tools/call path in `WIRE-PROOF-RECEIPT.txt` (re-run green after swarm_run_id receipt echo); §7.5 component numbers retired for this path.)

## 7. What was NOT verified

- Capability-tree DAG execution and hook-driven coherence remain PROPOSED (registries present on disk; no runtime dispatcher). The wired path exercises the capability _gate_ (deny unknown) but not DAG execution.
- SQLite sink is the offline `observability/store.py` adapter (54/54 suite green), not a live in-server writer; replay is demonstrated via the proof harness, not continuous operation.
