# C22 — Failure Modes, Runbooks & Recovery

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Known failure modes, detection, mitigation, and runbooks.

## Subcategories (broad topics)

- C22.1 OOM / memory pressure
- C22.2 Circuit breaker open (recall collapse, repeated failure)
- C22.3 Hole merge conflicts / source-hole root mismatch
- C22.4 Ingest/receipt/packet failures and retry signatures
- C22.5 Compaction restart and checkpoint restore
- C22.6 Degraded retrieval (embed endpoint missing → BM25)
- C22.7 Recovery procedures and smoke verification

## Open questions / gaps

- Tie to existing pressure-test + load-smoke harness.

## Contract catalog (frozen for design)

| Failure                   | Detection                          | Runbook / mitigation                                      |
| ------------------------- | ---------------------------------- | --------------------------------------------------------- |
| OOM / memory pressure     | peak allocation + permits          | backpressure, reduce in-flight, fallback to standard path |
| Circuit breaker open      | repeated failure / recall collapse | inspect reason; do not disable guard without receipt      |
| Hole merge mismatch       | Merkle root mismatch               | reject merge; diff and re-approve                         |
| Packet failure on restart | interrupted packets marked failed  | replay/retry via signatures; idempotency                  |
| Missing embed endpoint    | config absent                      | degrade to BM25 honestly (NF6)                            |
| Compaction restart        | fold threshold + checkpoint        | restore pinned system/original user messages              |
| Toroidal retry spin       | retry signature repetition         | circuit breaker + idempotency (C13.2)                     |

## Fault Tree Analysis (FTA)

### Top-Level Event: System Unavailable

```
                    SYSTEM UNAVAILABLE
                          |
              +-----------+-----------+
              |                       |
        MODEL SERVING DOWN      MEMORY SYSTEM DOWN
              |                       |
      +-------+-------+       +-------+-------+
      |               |       |               |
  ACFT CRASH    STANDARD      ATLAS CORRUPT   WAL TAIL
  (no fallback) TRANSFORMER   (snapshot)      CORRUPT
      |               |       |               |
      v               v       v               v
  Kill switch    Health      Atomic write   Auto-repair
  failed         check       failed         failed
```

### Minimal Cut Sets

| Cut Set | Components                                      | Probability | Mitigation                      |
| ------- | ----------------------------------------------- | ----------- | ------------------------------- |
| CS1     | Kill switch fail + ACFT crash                   | 10⁻⁶        | Dual health checks              |
| CS2     | Atlas snapshot corrupt + WAL repair fail        | 10⁻⁷        | SHA-256 verification            |
| CS3     | OOM + backpressure fail + no fallback           | 10⁻⁵        | Hard memory limit + kill switch |
| CS4     | Circuit breaker stuck open + no manual override | 10⁻⁴        | Admin API to force close        |

### Formal Runbooks as Executable Procedures

```markdown
# RUNBOOK: OOM / Memory Pressure (C22.1)

## Detection

- Alert: `memory_pressure > 0.85` for > 30s
- Metric: `peak_allocation_gb > 30`
- Log: `admission_rejected_total` increasing

## Automated Mitigation (runs in < 5s)

1. Admission controller reduces `max_inflight` by 50%
2. If `acft_enabled`: flip kill switch → standard transformer
3. Evict non-pinned hyperbolic atoms (LRU by radial coordinate)
4. Emit `oom_mitigation` telemetry event

## Manual Escalation (if automated fails)

1. `kubectl exec -it atoma-backend -- atoma-cli memory status`
2. `kubectl exec -it atoma-backend -- atoma-cli memory evict --non-pinned --count 1000`
3. If still failing: restart backend with `--reduced-context`

## Verification

- `memory_pressure < 0.7` within 60s
- `decode_tok_s > 0` (service responsive)
- No data loss (check receipt chain integrity)

---

# RUNBOOK: Circuit Breaker Open (C22.2)

## Detection

- Alert: `circuit_breaker_state{lane="*", state="open"} == 1`
- Metric: `recall_failure_rate > 0.5` for > 60s
- Log: `circuit_breaker_opened_total` increment

## Automated Mitigation

1. Lane isolated: no new packets admitted
2. Existing packets: retry with exponential backoff (max 3)
3. Health check: probe lane with synthetic request every 30s
4. Auto-close: if 3 consecutive probes succeed → half-open → closed

## Manual Escalation

1. Inspect reason: `atoma-cli circuit-breaker inspect <lane>`
2. If recall collapse: check Atlas integrity (`atoma-cli atlas verify`)
3. If repeated failure: check upstream (Rapid-MLX health)
4. Force close (admin only): `atoma-cli circuit-breaker force-close <lane> --receipt`

## Verification

- `circuit_breaker_state{lane="*", state="closed"} == 1`
- `recall_failure_rate < 0.05`
- No packets stuck in `running` > 5min

---

# RUNBOOK: Hole Merge Mismatch (C22.3)

## Detection

- Alert: `hole_merge_failed_total` increment
- Log: `merkle_root_mismatch` with source/hole roots

## Automated Mitigation

1. Reject merge automatically
2. Preserve both source and hole states
3. Emit `hole_merge_conflict` event with diff
4. Notify assigned lane operator

## Manual Resolution

1. `atoma-cli hole diff <hole_id>` — view unified diff
2. `atoma-cli hole approve <hole_id> --force` — if confident
3. Or: `atoma-cli hole reject <hole_id> --reason "..."`
4. Re-run merge after resolution

## Verification

- `hole_status{hole_id="<id>", status="merged"} == 1`
- Merkle root matches expected
- No data loss (receipt chain intact)

---

# RUNBOOK: Packet Failure on Restart (C22.4)

## Detection

- On startup: packets with `status=running` and `updated_at > restart_time - 5min`
- Metric: `packet_restart_failed_total` increment

## Automated Mitigation

1. Mark interrupted packets `failed` with `restart_interrupted` reason
2. Generate retry signatures for idempotent replay
3. Re-admit packets with `depends_on` satisfied
4. Emit `packet_replayed` events

## Manual Escalation

1. `atoma-cli packet list --status failed --reason restart_interrupted`
2. Inspect each: `atoma-cli packet inspect <id>`
3. Replay manually: `atoma-cli packet replay <id> --idempotency-key <key>`
4. If non-idempotent: escalate to human for decision

## Verification

- No packets in `running` state from before restart
- All `restart_interrupted` packets either replayed or escalated
- Receipt chain continuity verified

---

# RUNBOOK: Compaction Restart (C22.5)

## Detection

- Context budget exceeded (fold threshold 0.6)
- `compaction_triggered_total` increment

## Automated Procedure

1. Agent deposits session summary (C07.2) — mandatory
2. `BrainContinuityCheckpoint` created from context
3. Context folded: keep pinned (system + original user) + checkpoint
4. Agent restarts with folded context + injected memory
5. Verify objective recall = 100% (C06.1)

## Failure Path

- If summary deposit fails: `summary_missing` receipt, agent flagged
- If checkpoint corrupt: restore from previous `.atoma_map.wal`
- If recall < 100%: alert, manual investigation

## Verification

- `compaction_completed_total` increment
- `objective_recall_after_compaction == 1.0`
- Agent continues without manual intervention
```

### Recovery Smoke Verification

```bash
#!/bin/bash
# recovery_smoke.sh — runs after any failure mitigation

set -euo pipefail

echo "=== Recovery Smoke Verification ==="

# 1. Health endpoints
curl -f http://localhost:8088/health || exit 1
curl -f http://localhost:8088/api/map-state || exit 1

# 2. Memory injection works
agent_id=$(atoma-cli agent list --json | jq -r '.[0].id')
atoma-cli agent inject-memory $agent_id --verify || exit 1

# 3. Tool loop works
atoma-cli agent run $agent_id --prompt "test" --verify || exit 1

# 4. Hole lifecycle works
hole_id=$(atoma-cli hole create --agent $agent_id --verify)
atoma-cli hole run $hole_id --verify
atoma-cli hole approve $hole_id --verify
atoma-cli hole merge $hole_id --verify

# 5. Workflow admission works
packet_id=$(atoma-cli workflow submit --spec '{"name":"smoke"}' --verify)
atoma-cli workflow wait $packet_id --timeout 30 --verify

# 6. Metrics flowing
curl -s http://localhost:8088/metrics | grep -q "atoma_" || exit 1

echo "=== All smoke checks PASSED ==="
```

---

## Contributor Deepening — Muse (2026-10-26)

### C22.8 Runbook test status (depth: unexecuted runbooks are prose)

Every runbook above is currently PROPOSED text — none has been executed, including against a scratch environment. Frozen execution rule [PROPOSED — rule is my proposal; Gate 6 chaos/recovery is the authority]:

1. Each runbook gains a `last_drilled` field (date + environment + result). A runbook with no `last_drilled` before P5 is PROPOSED, and any Gate 6 claim citing it fails.
2. Drill order follows the C04.11 failure-isolation matrix: drill contained failures first (ingest fallback, hook veto, hole isolation), cascade failures second (lane breaker, Atlas kill-switch), full-system last (C12 serving loss). Do not drill cascade paths before contained paths pass — a failing contained drill during a cascade drill masks the root cause.
3. The OOM runbook's step 3 ("Evict non-pinned hyperbolic atoms (LRU by radial coordinate)") contradicts C05 eviction (victim = max-x0, which IS radial order — consistent) but the parenthetical "LRU" is wrong: hyperbolic eviction is NOT least-recently-used, it is least-salient by geometry. Correct the runbook step to "Evict max-x0 unpinned atoms per C05" via C36 correction; LRU language invites an implementation that violates the geometric invariant.

### C22.9 Cut-set probability provenance (depth: numbers without sources are wishes)

The Minimal Cut Sets table states probabilities (1e-4 to 1e-7) with no source. Frozen labeling [PROPOSED — labeling is my proposal; C03.2 is the authority]:

- All four probabilities are PROPOSED placeholders until measured or derived. They MUST NOT appear in any safety argument, slide, or claim until each carries either (a) a fault-injection measurement count ("0 dual-failures in N injections"), or (b) a derivation from component probabilities with the independence assumption stated.
- CS4 (breaker stuck open + no manual override) at 1e-4 is the weakest link by four orders of magnitude: if any cut set gets a real measurement first, it is CS4. The admin force-close API it cites is itself an elevation vector (C16) — its use emits a receipt and requires Admin role; document that here, not just in C16.

What was NOT checked here: atoma-cli command existence (no CLI tree in workspace — confirm before P1); pressure-test harness cited in the open question (confirm location).

## Open questions / gaps

- Tie to existing pressure-test + load-smoke harness.
