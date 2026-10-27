# C34 — Telemetry & Diagnostics

**Category:** C34-Agent-Lanes-Personas  
**Purpose:** Observability contract for lane personas, handoffs, and approvals.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `LaneTaskStarted` | `lane_id`, `task_id`, `responsibility`, `telemetry_identity` | ids only |
| `LaneTaskCompleted` | `lane_id`, `task_id`, `ok`, `duration_ms`, `receipt_hash` | ids/latency/hash only |
| `HandoffInitiated` | `from_lane`, `to_lane`, `task_id`, `protocol`, `session_id` | ids only |
| `HandoffCompleted` | `from_lane`, `to_lane`, `task_id`, `session_id`, `ok`, `duration_ms` | ids/latency/ok only |
| `HandoffFailed` | `from_lane`, `to_lane`, `task_id`, `session_id`, `error_code` | ids/error only |
| `ApprovalRequested` | `lane_id`, `task_id`, `effect`, `mode`, `timeout_ms` | ids/effect/mode only |
| `ApprovalGranted` | `lane_id`, `task_id`, `effect`, `mode`, `grantor`, `latency_ms` | ids/latency only |
| `ApprovalDenied` | `lane_id`, `task_id`, `effect`, `mode`, `reason_code` | ids/reason only |
| `EscalationTriggered` | `lane_id`, `task_id`, `escalation_path`, `level` | ids only |
| `RAGQueryExecuted` | `lane_id`, `collection`, `k`, `bm25_weight`, `dense_weight`, `latency_ms` | ids/latency only |

**Never logged**: Task context, artifacts, review content, approval reasoning, RAG query/results.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `lane_task_duration_ms` | Histogram | `lane_id`, `responsibility`, `ok` | Task latency |
| `lane_task_total` | Counter | `lane_id`, `responsibility`, `ok` | Task count |
| `handoff_duration_ms` | Histogram | `from_lane`, `to_lane`, `ok` | Handoff latency |
| `handoff_total` | Counter | `from_lane`, `to_lane`, `ok` | Handoff count |
| `approval_request_total` | Counter | `lane_id`, `effect`, `mode` | Approval requests |
| `approval_grant_total` | Counter | `lane_id`, `effect`, `mode` | Approval grants |
| `approval_deny_total` | Counter | `lane_id`, `effect`, `mode`, `reason` | Approval denials |
| `approval_latency_ms` | Histogram | `lane_id`, `effect`, `mode` | Approval latency |
| `escalation_total` | Counter | `lane_id`, `level` | Escalations |
| `rag_query_duration_ms` | Histogram | `lane_id`, `collection` | RAG query latency |
| `rag_query_total` | Counter | `lane_id`, `collection` | RAG queries |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `lane.task_start` | LaneTaskStarted | `lane_id`, `responsibility` |
| `lane.task_complete` | LaneTaskCompleted | `lane_id`, `ok`, `latency` |
| `lane.handoff` | HandoffCompleted | `from`, `to`, `ok`, `latency` |
| `lane.approval` | ApprovalGranted/Denied | `lane_id`, `effect`, `mode`, `granted` |
| `lane.escalation` | EscalationTriggered | `lane_id`, `level` |
| `lane.rag_query` | RAGQueryExecuted | `lane_id`, `collection`, `latency` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Lane task | `LaneTaskReceipt` | `lane_id + task_id + context_hash + result_hash + timestamp` |
| Handoff | `HandoffReceipt` | `from_lane + to_lane + task_id + session_id + artifacts_hash + timestamp` |
| Approval | `ApprovalReceipt` | `lane_id + task_id + effect + mode + grantor + timestamp` |

---

## Neural Map Roll-Up (C31.4)

End-of-day aggregation:
- Task throughput by lane and responsibility
- Handoff success rate and latency by lane pair
- Approval grant/deny rate by effect and mode
- Escalation frequency
- RAG query volume and latency by lane

**No task context, artifacts, or review content** in roll-up.

---

## Diagnostics Commands

```bash
# Lane persona health
curl localhost:8088/api/lanes/personas/health

# Handoff trace
curl localhost:8088/api/lanes/handoffs/trace?from=builder&to=contributor&limit=50

# Approval audit
curl localhost:8088/api/lanes/approvals/audit?lane=builder&since=24h

# RAG query stats
curl localhost:8088/api/lanes/rag/stats

# Escalation log
curl localhost:8088/api/lanes/escalations?since=24h
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — enforced by telemetry process schema validation.
- No task context, artifacts, review content, approval reasoning, or RAG queries/results in any metric, tap, receipt, or log.
- Session IDs and task IDs are opaque UUIDs.
- Violations caught by `scripts/compliance_smoke.sh` (C15).
