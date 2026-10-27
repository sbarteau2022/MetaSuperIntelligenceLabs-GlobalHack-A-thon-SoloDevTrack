# C31 — Observability, Telemetry & Metrics

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Event ledger (SQLite, append-only, monotonic), telemetry process + persisted session, Prometheus metrics, taps, receipts, fingerprints, Neural Map end-of-day roll-up. Privacy rule: counts/ids/latency only; never args/results/text. TTFT + throughput metrics propagation. Memory event telemetry (ids/scopes only).

## Subcategories (broad topics)

- C31.1 Event ledger (SQLite, append-only, monotonic)
- C31.2 Telemetry process + persisted session, honest availability
- C31.3 Metrics (Prometheus text), taps, receipts, fingerprints
- C31.4 Neural Map end-of-day roll-up
- C31.5 Privacy rule (counts/ids/latency only; never args/results/text)
- C31.6 TTFT + throughput metrics propagation
- C31.7 Memory event telemetry (ids/scopes only)

---

## Canonical Decisions (frozen for design)

| Decision                                                                              | Rationale                                                   |
| ------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| **Event ledger = SQLite** append-only; no UPDATE/DELETE; monotonic clock              | Tamper-evident; reproducible replay; forensic audit         |
| **Telemetry process** = separate OS process; honest availability (reports own health) | Isolation from agent crashes; self-monitoring               |
| **Prometheus text format** for metrics export; /metrics endpoint                      | Standard; scraper-agnostic; no push gateway dependency      |
| **Taps** = structured event stream for real-time dashboards                           | Low-latency observability; separate from ledger             |
| **Receipts/fingerprints** = Merkle paths for all change-affecting actions             | Verifiable execution; non-repudiation                       |
| **Neural Map roll-up** = end-of-day aggregation only; no raw data                     | Privacy; storage bound; trend analysis                      |
| **Privacy rule** = counts/ids/latency ONLY; never args/results/text (ADR-06)          | Legal compliance; user trust; enforced by schema validation |
| **TTFT/throughput** propagated via receipts, not separate channel                     | Single source of truth; receipt binds metrics to action     |

---

## Mathematical Specification

### 1. Event Ledger as Append-Only Log

**Ledger** `L` = sequence of entries `e_i` with:

```
e_i = (seq: ℕ, timestamp: TAI64, event_type: String, payload_hash: SHA256, payload: JSON)
```

**Invariants**:

1. `seq` strictly increasing: `seq(e_{i+1}) = seq(e_i) + 1`
2. `timestamp` monotonic: `timestamp(e_{i+1}) ≥ timestamp(e_i)`
3. `payload_hash = SHA256(payload)`
4. No `UPDATE` or `DELETE` SQL executed on ledger table
5. `PRAGMA journal_mode = WAL`; `PRAGMA synchronous = FULL`

**Schema**:

```sql
CREATE TABLE event_ledger (
  seq         INTEGER PRIMARY KEY,
  timestamp   TEXT NOT NULL,           -- TAI64N format
  event_type  TEXT NOT NULL,
  payload_hash TEXT NOT NULL,          -- SHA256 hex
  payload     TEXT NOT NULL            -- JSON
);
CREATE INDEX idx_ledger_type ON event_ledger(event_type);
CREATE INDEX idx_ledger_time ON event_ledger(timestamp);
```

**Merkle tree over ledger**: Root `R_n = MerkleRoot([payload_hash(e_1), ..., payload_hash(e_n)])` stored in `ledger_roots` table every 1000 entries.

**Verification**: `verify_ledger(db) → bool` checks all 5 invariants + Merkle root consistency.

### 2. Telemetry Process as Independent Monitor

**TelemetryProcess** `T` runs as separate PID with:

- Own SQLite connection to ledger (read-only)
- Own metrics registry (Prometheus)
- Health endpoint `/healthz` (liveness) and `/readyz` (readiness)
- **Honest availability**: Reports `healthy = true` iff:
  - Can write to ledger
  - Can scrape own metrics
  - No OOM in last 60s
  - Ledger Merkle root verifies

**Session persistence**: Telemetry session `S` survives agent restarts:

```
S = (session_id: UUID, start_time: TAI64, ledger_path: Path, config_hash: SHA256)
```

Stored in `telemetry_sessions` table; resumed on restart.

### 3. Metrics as Typed Prometheus Export

**Metric** `m` = `(name: String, type: {Counter, Gauge, Histogram, Summary}, labels: Map<String, String>, value: ℝ)`

**Naming convention**: `<subsystem>_<action>_<unit>` (e.g., `tool_execution_duration_ms`)

**Label constraints**:

- Max 10 labels per metric
- Label values: lowercase, alphanumeric + underscore, max 64 chars
- No high-cardinality labels (no raw IDs, hashes, paths)

**Export**: `/metrics` endpoint returns Prometheus text format. Scraped by external Prometheus or internal `/api/metrics` for UI.

**Cardinality bound**: Total active series < 10,000 (enforced at registration).

### 4. Taps as Structured Event Stream

**Tap** = `(name: String, trigger: EventType, schema: JSONSchema, sink: TapSink)`

**TapSink** ∈ { `Ledger`, `Stdout`, `Webhook`, `RingBuffer` }

**Built-in taps** (registered at boot):

| Tap                   | Trigger              | Schema                            | Sink                    |
| --------------------- | -------------------- | --------------------------------- | ----------------------- |
| `tool.resolve`        | `ToolResolved`       | `{tool_id, effect, latency}`      | Ledger + RingBuffer     |
| `tool.execute`        | `ToolExecuted`       | `{tool_id, effect, ok, latency}`  | Ledger + RingBuffer     |
| `hook.fire`           | `SkillHookFired`     | `{skill_id, moment, ok, latency}` | Ledger + RingBuffer     |
| `lane.message`        | `CrossLaneMessage`   | `{source, target, distance, I}`   | Ledger + RingBuffer     |
| `interference.breach` | `InterferenceBreach` | `{lane_i, lane_j, I_value}`       | Ledger + Stdout (alert) |

**RingBuffer**: In-memory circular buffer (10,000 events) for real-time UI; not persisted.

### 5. Receipts and Fingerprints as Merkle Paths

**Receipt** `r` for action `a`:

```
r = (action_id: UUID, action_type: String, merkle_root: SHA256, path: MerklePath, timestamp: TAI64)
```

**MerklePath** = `[ (leaf_hash, sibling_hash, direction) ]` where direction ∈ {L, R}

**Fingerprint** `f` = `SHA256(action_type || action_id || merkle_root || timestamp)`

**Verification**: `verify_receipt(r, manifest) → bool`:

1. Reconstruct root from path
2. Check root matches `r.merkle_root`
3. Check `action_id` exists in manifest with matching type
4. Check timestamp within clock skew tolerance (±5s)

### 6. Neural Map Roll-Up as Daily Aggregation

**RollUp** `R_d` for day `d`:

```
R_d = Aggregate( { e ∈ L | e.timestamp ∈ [d, d+1) } )
```

**Aggregation functions** (privacy-preserving):

- `count(event_type)` — total events
- `rate(event_type)` — events/second
- `p50(latency)`, `p95(latency)`, `p99(latency)` — latency percentiles
- `unique_count(label)` — distinct label values (e.g., active lanes)
- `sum(counter)` — counter increments

**Output**: `neural_map_YYYYMMDD.json` with schema:

```json
{
  "date": "2026-10-26",
  "event_counts": { "ToolExecuted": 12345, ... },
  "latency_p50_ms": { "tool_execution": 12.3, ... },
  "latency_p95_ms": { "tool_execution": 45.6, ... },
  "latency_p99_ms": { "tool_execution": 123.4, ... },
  "active_lanes": 3,
  "total_tool_calls": 12345,
  "total_subagent_spawns": 67,
  "brigade_weight_copies_max": 1,
  "interference_breaches": 0
}
```

**No raw payloads, hashes, or IDs** in roll-up — only aggregates.

### 7. Privacy Rule as Non-Interference Proof

**Privacy Policy** `P`: For all telemetry output `o` (metrics, taps, ledger payloads, roll-ups):

```
∀o: o contains no (args, results, text, source_code, file_content, message_body)
```

**Enforcement**: Telemetry process validates every emit against schema:

```json
{
  "type": "object",
  "properties": {
    "event_type": { "type": "string" },
    "payload": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "ids": { "type": "array", "items": { "type": "string" } },
        "latency_ms": { "type": "number" },
        "counts": { "type": "object" },
        "hashes": {
          "type": "array",
          "items": { "type": "string", "pattern": "^sha256:" }
        }
      }
    }
  },
  "required": ["event_type", "payload"]
}
```

**Violation** = schema validation failure → telemetry process emits `TelemetryPrivacyViolation` event (itself compliant) and drops offending entry.

### 8. TTFT/Throughput Propagation via Receipts

**Receipt** for model inference includes:

```
{
  "ttft_ms": 42,
  "throughput_tok_s": 123.4,
  "peak_memory_mb": 2048,
  "energy_joules": 15.2
}
```

**Propagation**: Every `ToolExecuted` receipt for inference tools carries these fields. Downstream consumers (autorunner, UI, roll-up) read from receipt — no separate metrics channel.

---

## Verification Gates (C15)

| Gate               | Test                                                | Threshold |
| ------------------ | --------------------------------------------------- | --------- |
| Ledger append-only | Attempt UPDATE/DELETE on ledger table               | SQL error |
| Ledger monotonic   | 1M entries; verify seq + timestamp monotonic        | 100%      |
| Merkle root        | Reconstruct root from path; match stored root       | 100%      |
| Telemetry health   | Process reports healthy iff all checks pass         | 100%      |
| Metrics format     | `/metrics` validates as Prometheus text             | 100%      |
| Cardinality bound  | Active series < 10,000 under load                   | 100%      |
| Tap schema         | All tap emits validate against declared schema      | 100%      |
| Receipt verify     | All receipts verify against manifest                | 100%      |
| Privacy schema     | All telemetry emits validate against privacy schema | 100%      |
| Roll-up privacy    | Roll-up JSON contains no raw payloads/IDs           | 100%      |
| TTFT propagation   | Inference receipts carry ttft/throughput            | 100%      |

---

## Preregistration Entries (C09)

### A22 — Ledger Integrity Under Crash

- Hypothesis: Ledger invariants hold after power loss / process kill at any point.
- Judge: 1,000 crash injections; 0 invariant violations on recovery.
- Status: TBD.

### A23 — Telemetry Privacy Enforcement

- Hypothesis: No telemetry output contains args/results/text under any code path.
- Judge: Static analysis + 10,000 fuzzed emits; 0 privacy schema violations.
- Status: TBD.

### A24 — Metrics Cardinality Bound

- Hypothesis: Active series stays < 10,000 under maximum load (N=8 lanes, 1000 req/s).
- Judge: Load test; max series observed.
- Status: TBD.

### A25 — Roll-Up Completeness

- Hypothesis: Daily roll-up captures 100% of events in the day window.
- Judge: Compare roll-up counts to ledger counts; difference = 0.
- Status: TBD.

---

## Open Questions / Gaps

- Ledger compaction/archival policy for long-running deployments (deferred to P6).
- Distributed ledger for multi-node (tied to C06.10).
- Real-time alerting rules for interference breaches (tied to C22).

---

## Contributor Deepening — Muse (2026-10-26)

### C31.8 Ledger payload vs privacy schema (depth: the ledger stores what telemetry forbids)

The C31.1 ledger entry is e_i = (seq, timestamp, event_type, payload_hash, payload: JSON) — the ledger stores the FULL payload, while the privacy rule (C31.5/ADR-06) forbids args/results/text in telemetry output. Both can hold only if the ledger's payload field is NOT telemetry output — i.e., the ledger is a restricted store, not a broadcast channel. Frozen access rule [PROPOSED — rule is my proposal; ADR-06 + C17 schema are the authority]:

1. Ledger payloads are readable ONLY by (a) the telemetry process (read-only connection, emits schema-validated aggregates), (b) Admin role via audit export (C16.2, receipted), (c) crash-recovery replay (C22.4, hash-chain verification only — replay reads hashes, not content).
2. The /metrics endpoint, taps, roll-ups, and UI NEVER read payload — they read payload_hash + declared aggregate fields. Any tap schema containing a payload-content field fails the C15 privacy gate (A23).
3. This makes the ledger the ONE place raw content rests — which makes ledger encryption-at-rest mandatory, not optional. State it here: unencrypted ledger = PII in SQLite (A1/A2 assets) with only filesystem permissions as control. Record as residual risk in C16.1; mitigation is platform encryption (FileVault/APFS) + documented assumption, or column-level encryption with the key in a secret slot (C16.3).
4. Consequence for C16.9 erasure: ledger payloads containing erased-subject content are the reason erasure uses tombstones (hash chain intact) rather than DELETE — deleting payload rows would break Merkle roots. The tombstone protocol is downstream of THIS decision: ledger immutability is load-bearing for the entire audit story.

### C31.9 Taps vs ledger consistency (depth: two streams, one truth?)

Taps emit to Ledger + RingBuffer; the RingBuffer (10k events, unpersisted) feeds real-time UI. Under crash, RingBuffer contents are lost while ledger contents survive — the UI after restart shows a different recent-history than the ledger holds. Frozen rule [PROPOSED — rule is my proposal]:

1. RingBuffer is explicitly a LOSSY VIEW: the UI labels real-time panels with data-source ("live buffer — last N events, lossy on restart") vs ledger panels ("ledger — complete, append-only"). A UI that presents buffer contents as complete history is a misrepresentation (C19 honest-degradation rule).
2. Post-restart, the UI replays the last N ledger entries into the buffer (backfill on reconnect) and marks the seam with a `restart_seam` marker event — the seam itself is receipted (restart count, clock range backfilled).
3. Alerting taps (interference.breach → Stdout) MUST also write the ledger synchronously before alerting: an alert without a ledger entry is an un-auditable alarm. Ledger-first, alert-second — no exceptions.

What was NOT checked here: telemetry process implementation (outside workspace); FileVault/APFS assumption validity on the replica machine (verify at C09.4 replication).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Observability (Original)

- **Event ledger**: SQLite, append-only, monotonic.
- **Telemetry process**: Separate, honest availability, persisted session.
- **Metrics**: Prometheus text format; taps for real-time.
- **Receipts/fingerprints**: Merkle paths for change-affecting actions.
- **Neural Map roll-up**: End-of-day aggregation.
- **Privacy**: Counts/ids/latency only; never args/results/text.
- **TTFT/throughput**: Propagated via receipts.
- **Memory events**: IDs/scopes only.
