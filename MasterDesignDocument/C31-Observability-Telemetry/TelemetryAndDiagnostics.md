# C31 — Telemetry & Diagnostics (Self-Observability)

**Category:** C31-Observability-Telemetry  
**Purpose:** Observability contract for the telemetry system itself.

---

## Event Ledger Entries (C31.1)

| Event                       | Fields                                                 | Privacy          |
| --------------------------- | ------------------------------------------------------ | ---------------- |
| `TelemetryProcessStarted`   | `session_id`, `config_hash`, `ledger_path`             | ids only         |
| `TelemetryProcessStopped`   | `session_id`, `reason`                                 | ids only         |
| `TelemetryHealthCheck`      | `healthy`, `checks: {ledger, metrics, memory, merkle}` | bools only       |
| `TelemetryPrivacyViolation` | `event_type`, `field`, `violation_type`                | ids/type only    |
| `LedgerMerkleRootUpdated`   | `seq`, `merkle_root`                                   | ids/hash only    |
| `MetricsCardinalityWarning` | `active_series`, `threshold`                           | counts only      |
| `RollUpCompleted`           | `date`, `event_count`, `file_size_bytes`               | counts/size only |

**Never logged**: Ledger payloads, metric label values, tap event data, roll-up raw data.

---

## Prometheus Metrics (C31.3)

| Metric                              | Type      | Labels               | Description              |
| ----------------------------------- | --------- | -------------------- | ------------------------ |
| `telemetry_process_uptime_seconds`  | Gauge     | —                    | Process uptime           |
| `telemetry_health_check_total`      | Counter   | `result`             | Health checks            |
| `telemetry_privacy_violation_total` | Counter   | `violation_type`     | Privacy violations       |
| `ledger_entries_total`              | Counter   | —                    | Total ledger entries     |
| `ledger_merkle_root_updates_total`  | Counter   | —                    | Merkle root updates      |
| `ledger_verification_duration_ms`   | Histogram | —                    | Ledger verify latency    |
| `metrics_active_series`             | Gauge     | —                    | Active Prometheus series |
| `metrics_cardinality_warning_total` | Counter   | —                    | Cardinality warnings     |
| `tap_emit_total`                    | Counter   | `tap_name`, `result` | Tap emits                |
| `tap_ringbuffer_drops_total`        | Counter   | `tap_name`           | Ring buffer overflow     |
| `receipt_verification_total`        | Counter   | `result`             | Receipt verifications    |
| `rollup_duration_ms`                | Histogram | —                    | Roll-up computation time |
| `rollup_events_processed`           | Counter   | —                    | Events in roll-up        |

---

## Taps (C31.3)

| Tap                           | Trigger                   | Emits                          |
| ----------------------------- | ------------------------- | ------------------------------ |
| `telemetry.health`            | TelemetryHealthCheck      | `healthy`, `checks`            |
| `telemetry.privacy_violation` | TelemetryPrivacyViolation | `event_type`, `violation_type` |
| `ledger.merkle_update`        | LedgerMerkleRootUpdated   | `seq`, `merkle_root`           |
| `metrics.cardinality`         | MetricsCardinalityWarning | `active_series`                |
| `rollup.complete`             | RollUpCompleted           | `date`, `event_count`          |

---

## Receipts & Fingerprints (C31.3)

| Action                    | Receipt Type        | Fingerprint Input                              |
| ------------------------- | ------------------- | ---------------------------------------------- |
| Ledger Merkle root update | `MerkleRootReceipt` | `seq + merkle_root + timestamp`                |
| Roll-up generation        | `RollUpReceipt`     | `date + event_count + output_hash + timestamp` |

---

## Neural Map Roll-Up (C31.4)

Telemetry self-roll-up (meta-roll-up):

- Telemetry process uptime/availability
- Privacy violation rate (must be 0)
- Ledger verification success rate
- Metrics cardinality trend
- Tap drop rate

---

## Diagnostics Commands

```bash
# Telemetry process health
curl localhost:8088/api/telemetry/health

# Ledger verification
curl localhost:8088/api/telemetry/ledger/verify

# Metrics cardinality
curl localhost:8088/api/telemetry/metrics/cardinality

# Privacy violation audit
curl localhost:8088/api/telemetry/privacy/audit?since=24h

# Roll-up status
curl localhost:8088/api/telemetry/rollup/status

# Tap ringbuffer inspection
curl localhost:8088/api/telemetry/taps/{name}/buffer?limit=100
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — telemetry system observes itself under same rules.
- No ledger payloads, metric label values, or tap data in any self-telemetry output.
- Privacy violations on telemetry process itself are logged (meta-privacy) and alerted.
- Violations caught by `scripts/compliance_smoke.sh` (C15).
