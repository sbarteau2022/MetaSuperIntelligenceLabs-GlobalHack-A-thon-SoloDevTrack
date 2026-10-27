# C30 — Telemetry & Diagnostics

**Category:** C30-Multi-Agent-Lanes-Autorunner  
**Purpose:** Observability contract for lanes, autorunner, subagents, and cross-lane comms.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `LaneStarted` | `lane_id`, `config_version`, `submanifold_anchor_hash` | ids only |
| `LaneStopped` | `lane_id`, `reason` | ids only |
| `AutorunnerPacketAdmitted` | `task_id`, `lane_id`, `priority`, `estimated_cost`, `queue_depth` | ids/cost only |
| `AutorunnerPacketRejected` | `task_id`, `lane_id`, `reason` | ids/reason only |
| `SubagentSpawned` | `parent_task_id`, `subagent_id`, `depth`, `budget_tokens`, `budget_time_ms` | ids/budget only |
| `SubagentReturned` | `subagent_id`, `ok`, `cost_tokens`, `cost_time_ms`, `receipt_hash` | ids/cost/hash only |
| `SubagentDepthExceeded` | `subagent_id`, `depth` | ids only |
| `SubagentBudgetExhausted` | `subagent_id`, `budget_type` | ids only |
| `BranchCreated` | `branch_id`, `lane_id`, `base_commit_hash`, `worktree_hash` | ids/hashes only |
| `BranchMerged` | `source_branch_id`, `target_branch_id`, `merge_receipt_hash` | ids/hash only |
| `CrossLaneMessage` | `source_lane`, `target_lane`, `bridge_distance`, `interference_I`, `receipt_hash` | ids/distance/I/hash only |
| `InterferenceBreach` | `lane_i`, `lane_j`, `I_value`, `threshold` | ids/values only |
| `BrigadeFiberProjected` | `lane_id`, `skill_id`, `fiber_radius`, `projection_latency_ms` | ids/latency only |
| `BrigadeFiberDissolved` | `lane_id`, `skill_id`, `dissolution_latency_ms`, `weight_copies` | ids/latency/weight_copies only |

**Never logged**: task payloads, subagent context, branch diffs, message content, skill fiber weights.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `lane_active` | Gauge | `lane_id` | Lane running (1) / stopped (0) |
| `lane_concurrent_tasks` | Gauge | `lane_id` | Currently running tasks |
| `autorunner_queue_depth` | Gauge | `lane_id`, `priority` | Ready queue depth |
| `autorunner_admission_total` | Counter | `lane_id`, `result` | Admitted/rejected packets |
| `autorunner_worker_utilization` | Gauge | `worker_id` | Worker busy ratio |
| `subagent_spawn_total` | Counter | `lane_id`, `depth` | Subagent spawns |
| `subagent_return_total` | Counter | `lane_id`, `ok` | Subagent returns |
| `subagent_depth_exceeded_total` | Counter | `lane_id` | Depth cap hits |
| `subagent_budget_exhausted_total` | Counter | `lane_id`, `budget_type` | Budget cap hits |
| `branch_merge_total` | Counter | `lane_id`, `result` | Merge attempts |
| `cross_lane_message_total` | Counter | `source_lane`, `target_lane`, `ok` | Cross-lane messages |
| `interference_I` | Histogram | `lane_i`, `lane_j` | Measured interference |
| `interference_breach_total` | Counter | `lane_i`, `lane_j` | Circuit breaker triggers |
| `brigade_projection_duration_ms` | Histogram | `lane_id` | Fiber projection time |
| `brigade_dissolution_duration_ms` | Histogram | `lane_id` | Fiber dissolution time |
| `brigade_weight_copies` | Gauge | — | Live weight copies (must = 1) |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `lane.start` | LaneStarted | `lane_id` |
| `lane.stop` | LaneStopped | `lane_id` |
| `autorunner.admit` | AutorunnerPacketAdmitted | `lane_id`, `priority`, `queue_depth` |
| `autorunner.reject` | AutorunnerPacketRejected | `lane_id`, `reason` |
| `subagent.spawn` | SubagentSpawned | `lane_id`, `depth`, `budget_tokens` |
| `subagent.return` | SubagentReturned | `lane_id`, `ok`, `cost_tokens` |
| `branch.create` | BranchCreated | `lane_id` |
| `branch.merge` | BranchMerged | `source_lane`, `target_lane`, `ok` |
| `cross_lane.send` | CrossLaneMessage | `source`, `target`, `distance`, `I` |
| `interference.breach` | InterferenceBreach | `lane_i`, `lane_j`, `I_value` |
| `brigade.project` | BrigadeFiberProjected | `lane_id`, `latency` |
| `brigade.dissolve` | BrigadeFiberDissolved | `lane_id`, `latency`, `weight_copies` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Subagent execution | `SubagentReceipt` | `subagent_id + call_hash + result_hash + hole_log_hash + timestamp` |
| Branch merge | `MergeReceipt` | `source_hash + target_hash + diff_hash + approval_hash + timestamp` |
| Cross-lane transport | `TransportReceipt` | `source_lane + target_lane + bridge_hash + payload_hash + timestamp` |
| Brigade projection | `BrigadeReceipt` | `lane_id + skill_id + fiber_hash + transport_hash + timestamp` |

---

## Neural Map Roll-Up (C31.4)

End-of-day aggregation:
- Lane utilization and throughput
- Autorunner admission rate and queue latency
- Subagent spawn/return ratio; depth/budget exhaustion rates
- Branch merge success rate
- Cross-lane message volume and interference distribution
- Brigade projection/dissolution latency; weight copy invariant

**No task payloads, subagent context, or message content** in roll-up.

---

## Diagnostics Commands

```bash
# Lane health
curl localhost:8088/api/lanes/health

# Autorunner status
curl localhost:8088/api/autorunner/status

# Subagent trace
curl localhost:8088/api/subagents/trace?lane={id}&limit=100

# Branch status
curl localhost:8088/api/branches/status

# Interference matrix
curl localhost:8088/api/lanes/interference

# Brigade audit
curl localhost:8088/api/brigade/audit
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — enforced by telemetry process schema validation.
- No task arguments, subagent context, branch diffs, message content, or skill weights in any metric, tap, receipt, or log.
- Commit hashes, worktree hashes, and receipt hashes are SHA-256 digests only.
- Violations caught by `scripts/compliance_smoke.sh` (C15).
