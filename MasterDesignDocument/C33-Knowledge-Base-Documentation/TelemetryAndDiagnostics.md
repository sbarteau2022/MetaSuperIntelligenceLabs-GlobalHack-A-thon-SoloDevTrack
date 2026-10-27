# C33 — Telemetry & Diagnostics

**Category:** C33-Knowledge-Base-Documentation  
**Purpose:** Observability contract for documentation agents and governance.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `ClaimLintCompleted` | `agent`, `files_checked`, `claims_found`, `violations`, `duration_ms` | counts/latency only |
| `MigrationAuditCompleted` | `agent`, `refs_scanned`, `pending`, `completed`, `dropped`, `duration_ms` | counts/latency only |
| `FiveFileCheckCompleted` | `agent`, `categories_total`, `categories_complete`, `missing`, `duration_ms` | counts/latency only |
| `RollupGenerated` | `agent`, `date`, `events_processed`, `file_size_bytes`, `duration_ms` | counts/size/latency only |
| `CrosslinkCheckCompleted` | `agent`, `refs_checked`, `broken`, `duration_ms` | counts/latency only |
| `ClaimRegistered` | `claim_id`, `label`, `workstream`, `has_evidence_hash` | ids/label/bool only |
| `CorrectionAppended` | `correction_id`, `corrects_claim_id`, `workstream` | ids only |

**Never logged**: Claim texts, evidence content, migration diff content, file contents.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `doc_claim_lint_total` | Counter | `result` | Lint runs |
| `doc_claim_violations_total` | Counter | `workstream`, `violation_type` | Label/evidence violations |
| `doc_migration_audit_total` | Counter | `result` | Migration audits |
| `doc_migration_status` | Gauge | `original`, `status` | Migration status (0/1) |
| `doc_five_file_complete` | Gauge | `workstream` | 1 if complete, 0 if missing |
| `doc_rollup_generation_total` | Counter | `result` | Rollup generations |
| `doc_rollup_events_processed` | Counter | — | Events in rollup |
| `doc_crosslink_check_total` | Counter | `result` | Crosslink checks |
| `doc_crosslink_broken_total` | Counter | `source_file`, `target_ref` | Broken references |
| `doc_claims_registered_total` | Counter | `workstream`, `label` | Claims added |
| `doc_corrections_total` | Counter | `workstream` | Corrections appended |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `doc.claim_lint` | ClaimLintCompleted | `files_checked`, `claims_found`, `violations` |
| `doc.migration_audit` | MigrationAuditCompleted | `pending`, `completed`, `dropped` |
| `doc.five_file` | FiveFileCheckCompleted | `complete`, `missing` |
| `doc.rollup` | RollupGenerated | `date`, `events_processed` |
| `doc.crosslink` | CrosslinkCheckCompleted | `refs_checked`, `broken` |
| `doc.claim_registered` | ClaimRegistered | `workstream`, `label`, `has_hash` |
| `doc.correction` | CorrectionAppended | `workstream` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Claim lint | `DocAgentReceipt` | `agent + files_hash + violations_hash + timestamp` |
| Migration audit | `DocAgentReceipt` | `agent + refs_hash + status_hash + timestamp` |
| Five-file check | `DocAgentReceipt` | `agent + categories_hash + missing_hash + timestamp` |
| Rollup generation | `RollupReceipt` | `date + events_hash + output_hash + timestamp` |

---

## Neural Map Roll-Up (C31.4)

Documentation meta-roll-up (weekly):
- Claim labeling compliance rate by workstream
- Migration completion rate
- Five-file set completeness
- Crosslink health
- Correction frequency

---

## Diagnostics Commands

```bash
# Documentation health
curl localhost:8088/api/docs/health

# Claim lint status
curl localhost:8088/api/docs/claims/lint

# Migration status
curl localhost:8088/api/docs/migrations/status

# Five-file set status
curl localhost:8088/api/docs/five-file/status

# Rollup history
curl localhost:8088/api/docs/rollups/list?limit=30

# Crosslink health
curl localhost:8088/api/docs/crosslinks/health

# Claims register query
curl localhost:8088/api/docs/claims/query?workstream=C28&label=VERIFIED
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — documentation agents observe only metadata.
- No claim texts, evidence content, file contents, or migration diffs in any telemetry.
- Claim IDs and workstream IDs are safe (no sensitive data).
- Violations caught by `scripts/compliance_smoke.sh` (C15).
