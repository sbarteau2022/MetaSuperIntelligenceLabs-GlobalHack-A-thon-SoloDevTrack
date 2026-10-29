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

| Component | Primary | Fallback | Guarantee | What is recorded |
|---|---|---|---|---|
| RAG embedder (PAMI) | `atomadb::tf_embed` + `rotate_rank` | tf_embed-only (no phase) + `degraded=pami` flag | retrieval still returns hits; claim restricted to same-salience | receipt + health flag |
| Retrieval index | VP-tree exact-prune kNN | brute-force full scan | same top-1 at tested scale; higher latency | `exact_attention_measured=false` |
| Memory compaction | `map.compact` sha-chained checkpoint | keep prior checkpoint, defer compaction | no silent overwrite; pinned invariants preserved | escalation receipt (TTL) |
| Capability hook | deny / append-context | skip hook, continue with base context | tool call still evaluated by role gate | hook-failure receipt |
| Runbook packet | judge pass required | escalate with receipts + TTL + prereg | no identical retry; no unjudged packet enters mind | escalation record |
| Telemetry SQLite | append to DB | in-band receipt + WAL queue for replay | no lost receipt from the caller's view | deferred-write marker |
| Snapshot/recover | hash-verified snapshot | rebuild from AtomaDB mind lines | epoch preserved | recovery receipt |
| Capability registration | tool in fabric | deny invocation, keep registry intact | no unauthorized execution | denial receipt |

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

- [ ] All four parts reachable in one `tools/call` path with receipts on every response.
- [ ] PAMI method (`rotate_rank`, `participation_ratio`) on the RAG path; tf_embed-only fallback flagged.
- [ ] Auto-compaction trigger + rollback verified end-to-end (not just primitives).
- [ ] Hook/runbook failure paths exercised (deny/append-context, escalation).
- [ ] Telemetry receipts replay into the SQLite sink when available; in-band receipts always.
- [ ] `cargo test` green (resolve the 5 pre-existing compile errors or scope them explicitly).
- [ ] Pressure harness extended to cover degraded paths (fallback assertions) and re-run.
- [ ] Full-system benchmark captured after wiring; per-component numbers retired in favor of end-to-end numbers.

## 7. What was NOT verified

- No integration of these four parts exists yet; this file is the wiring spec. Any claim of wiring is PROPOSED until the acceptance checklist is green and pressure-tested.
