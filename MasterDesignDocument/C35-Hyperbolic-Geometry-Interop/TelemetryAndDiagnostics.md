# C35 — Telemetry & Diagnostics

**Category:** C35-Hyperbolic-Geometry-Interop  
**Purpose:** Observability contract for hyperbolic geometry operations.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `IsometryCheck` | `test_pairs`, `max_error`, `passed` | counts/result only |
| `PicardReduction` | `input_x0`, `output_x0`, `iterations`, `idempotent` | counts/bool only |
| `VPTreeRetrieval` | `query_hash`, `k`, `candidates_examined`, `latency_ms`, `recall_at_k` | counts/latency only |
| `Eviction` | `victim_x0`, `cache_size_before`, `cache_size_after`, `pinned_count` | counts only |
| `SnapshotWrite` | `atom_count`, `vp_tree_nodes`, `size_bytes`, `sha256`, `duration_ms` | counts/size/latency only |
| `WALReplay` | `entries_replayed`, `truncated`, `duration_ms`, `recovery_ok` | counts/latency/bool only |
| `ConcurrentTurn` | `thread_id`, `turn_id`, `isolation_ok`, `latency_ms` | ids/latency/bool only |

**Never logged**: Coordinate values, atom payloads, query vectors, snapshot content.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `geometry_isometry_error` | Gauge | — | Max isometry error (updated per check) |
| `geometry_picard_iterations` | Histogram | — | Picard reduction iterations |
| `geometry_picard_idempotent_total` | Counter | `result` | Idempotence checks |
| `geometry_vptree_retrieval_duration_ms` | Histogram | — | VP-tree query latency |
| `geometry_vptree_candidates_examined` | Histogram | — | Candidates examined per query |
| `geometry_vptree_recall_at_k` | Gauge | `k` | Recall@k vs brute force |
| `geometry_eviction_total` | Counter | — | Eviction count |
| `geometry_eviction_victim_x0` | Histogram | — | Radial coordinate of evicted |
| `geometry_cache_size` | Gauge | — | Current atom count |
| `geometry_pinned_count` | Gauge | — | Pinned atom count |
| `geometry_snapshot_write_duration_ms` | Histogram | — | Snapshot write latency |
| `geometry_snapshot_size_bytes` | Gauge | — | Snapshot file size |
| `geometry_wal_replay_duration_ms` | Histogram | — | WAL replay latency |
| `geometry_wal_entries_replayed` | Counter | — | WAL entries replayed |
| `geometry_concurrent_turns_total` | Counter | `isolation_ok` | Concurrent turn results |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `geometry.isometry` | IsometryCheck | `test_pairs`, `max_error`, `passed` |
| `geometry.picard` | PicardReduction | `input_x0`, `output_x0`, `iterations`, `idempotent` |
| `geometry.vptree` | VPTreeRetrieval | `k`, `candidates`, `latency`, `recall` |
| `geometry.eviction` | Eviction | `victim_x0`, `cache_size`, `pinned` |
| `geometry.snapshot` | SnapshotWrite | `atoms`, `size`, `duration` |
| `geometry.wal_replay` | WALReplay | `entries`, `truncated`, `duration`, `ok` |
| `geometry.concurrent` | ConcurrentTurn | `thread_id`, `isolation_ok`, `latency` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Snapshot write | `SnapshotReceipt` | `sha256 + atom_count + vp_tree_hash + timestamp` |
| WAL replay | `WALReceipt` | `last_clock + entries_hash + snapshot_hash + timestamp` |
| Eviction | `EvictionReceipt` | `victim_id + victim_x0_hash + cache_state_hash + timestamp` |

---

## Neural Map Roll-Up (C31.4)

End-of-day aggregation:
- Isometry error trend (must stay < 4.4e-9)
- Picard reduction performance (iterations, idempotence rate)
- VP-tree retrieval latency and recall@k
- Eviction rate and victim radial distribution
- Snapshot/WAL write/replay latency
- Concurrent turn isolation success rate

**No coordinates, atom data, or query vectors** in roll-up.

---

## Diagnostics Commands

```bash
# Geometry health
curl localhost:8088/api/geometry/health

# Isometry verification
curl localhost:8088/api/geometry/isometry/verify

# Picard reduction test
curl localhost:8088/api/geometry/picard/test

# VP-tree stats
curl localhost:8088/api/geometry/vptree/stats

# Cache status
curl localhost:8088/api/geometry/cache/status

# Persistence audit
curl localhost:8088/api/geometry/persistence/audit

# Concurrent turn test
curl localhost:8088/api/geometry/concurrent/test?threads=8
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — enforced by telemetry process schema validation.
- No coordinate values, atom payloads, query vectors, or snapshot content in any metric, tap, receipt, or log.
- Hashes (SHA-256) of coordinates/snapshots are safe (preimage resistant).
- Violations caught by `scripts/compliance_smoke.sh` (C15).
