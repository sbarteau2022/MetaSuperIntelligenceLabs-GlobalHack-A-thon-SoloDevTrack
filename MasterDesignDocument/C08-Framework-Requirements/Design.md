# C08 — Framework Requirements (Contract Before Code)

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Functional and non-functional contract that must be satisfied before feature code.

## Subcategories (broad topics)

- C08.1 Functional F1–F11 (incl. prompt-injection hygiene)
- C08.2 Non-functional NF1–NF11 (incl. energy/SLO/peak-memory ceiling)
- C08.3 Requirement → test mapping (each F/NF → ablation/gate ID)
- C08.4 Traceability matrix (requirement → component → receipt)
- C08.5 Change control (how requirements change post-freeze)

## Open questions / gaps

- Numbers for NF targets TBD until hardware benchmarks land (C21).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 11. Framework Requirements (PROPOSED — contract before code)

### 11.1 Functional

- F1 Lane roster is data-driven (N agents), not 3 fixed slots; per-agent tools, RAG collection, approval policy.
- F2 Memory state auto-injection (§10) at session/file-tab start, bounded by context budget.
- F3 Mandatory structured session summary on every session end (§10.2) or pre-compaction.
- F4 RAG retrieval + ingest reachable by every lane and by tools; ingest role-gated.
- F5 Tool loop in lanes (plan → tool → observe → continue) behind approval.
- F6 Workflows with depends_on, judges, retry, escalation, preregistration; auto-runner admission.
- F7 Hole isolation with Merkle source/hole roots, diff, approve, merge.
- F8 Telemetry/receipts for every tool call, lane turn, judge, and memory event (counts/latency/ids only).
- F9 Compaction fold + post-compaction restart with pinned system/original user messages.
- F10 Streaming transport with measured TTFT (not unmeasured placeholders).

### 11.2 Non-functional

- NF1 No OOM on the 10-day sprint workload: idle + peak Metal/CPU memory below guard; backpressure (permits).
- NF2 Append-only data integrity: events/workflows/ledgers reject update/delete.
- NF3 No secret leakage: grep/scan checks; secrets by slot; telemetry never args/results/text.
- NF4 Portability: all paths resolved via workspace root; no baked user paths.
- NF5 Determinism: resolve v2 deterministic hashes; receipts reproducible.
- NF6 Graceful degradation: missing embed endpoint → BM25 honestly, no silent vector claim.
- NF7 Recoverability: restart persistence (interrupted packets marked failed), checkpoint restore.
- NF8 Security first: threat model + role matrix + audit log before feature code (per §3.7).

## Requirement traceability matrix (frozen for design)

| ID     | Component (category)           | Ablation (C09) | Gate (C15) | Day (C32) | Required?                     |
| ------ | ------------------------------ | -------------- | ---------- | --------- | ----------------------------- |
| F1     | Lane roster (C30)              | A5             | C15.1-2    | D3        | required after necessity pass |
| F2     | Memory auto-injection (C07)    | A9             | A9         | D4-5      | PROPOSED until A9             |
| F3     | Session summary (C07)          | A9             | C15.2      | D4-5      | required after enforcement    |
| F4     | RAG ingest (C26)               | A8             | A8         | D4-5      | required                      |
| F5     | Tool loop in lanes (C14.3/C29) | C16.1*         | C15.2      | D6-7      | required                      |
| F6     | Workflows (C27)                | A7 (hooks)     | C15.1-2    | D4-5      | required                      |
| F7     | Holes (C25)                    | property tests | C15.1      | D3        | required                      |
| F8     | Telemetry/receipts (C31)       | A4 (holonomy)  | C15.5      | D10       | required                      |
| F9     | Compaction restart (C24)       | A2             | C15.4      | D3        | required                      |
| F10    | Streaming/TTFT (C18)           | TTFT stress    | C15.1-2    | D6-7      | required                      |
| F11    | Prompt-injection hygiene (C16) | A10            | C15.6      | D10       | required                      |
| NF1    | OOM/peak memory (C21)          | A1             | C15.1      | D8-9      | required                      |
| NF2    | Append-only (C17)              | property tests | C15.1      | D3        | required                      |
| NF3    | No secret leakage (C16)        | scan check     | C15.6      | D1        | required                      |
| NF4    | Portability (C17/C14.10)       | smoke          | C15.2      | D3        | required                      |
| NF5    | Determinism (C09)              | replay         | C15.1      | D10       | required                      |
| NF6    | Graceful degradation (C26)     | A3             | C15.1      | D4-5      | required                      |
| NF7    | Recoverability (C22/C24)       | restart tests  | C15.4      | D6-7      | required                      |
| NF8    | Security first (C16)           | checklist      | C15.6      | D1        | required                      |
| NF9-11 | Energy/SLO (C21)               | perf profile   | C15.7      | D8-9      | required for claims           |

`* C16.1 means necessity ablation registered before promotion.`

## Formal Specification (TLA+ Style)

### System State

```tla
VARIABLES
  \* Agent lanes
  lanes \in [AgentId -> LaneConfig],

  \* Memory state
  memory \in [MemoryId -> MemoryEntry],
  mind_log \in Seq(MindEntry),

  \* Workflows
  packets \in [PacketId -> Packet],
  judges \in [JudgeId -> Judge],

  \* Holes
  holes \in [HoleId -> HoleState],

  \* Telemetry
  telemetry \in Seq(TelemetryEvent),

  \* Configuration
  config \in Config,

  \* Vector clocks for causal consistency
  vc \in [AgentId -> [AgentId -> Nat]]
```

### Functional Requirements as Invariants

**F1 (Lane Roster)**:

```tla
LaneRosterInvariant ==
  \A a \in DOMAIN lanes:
    /\ lanes[a].tools \subseteq config.allowed_tools
    /\ lanes[a].rag_collection \in config.rag_collections
    /\ lanes[a].approval_policy \in config.approval_policies
```

**F2 (Memory Auto-Injection)**:

```tla
MemoryInjectionInvariant ==
  \A a \in AgentId:
    LET injected == GetInjectedMemory(a, config.context_budget) IN
    /\ injected.provenance \subseteq {lane, run_id, receipt_hash}
    /\ Len(injected) <= config.context_budget
```

**F3 (Session Summary)**:

```tla
SessionSummaryInvariant ==
  \A m \in mind_log:
    /\ m.sections = <<"brainstorm", "attempts", "plan", "executed", "conclusions", "coming_plan">>
    /\ m.receipt_hash = Hash(m)
    /\ m.lane \in DOMAIN lanes
```

**F4 (RAG Ingest)**:

```tla
RAGIngestInvariant ==
  \A r \in RAGCollections:
    \A d \in r.documents:
      d.receipt_hash = Hash(d)
      /\ d.ingest_role = "Ingest"
```

**F5 (Tool Loop)**:

```tla
ToolLoopInvariant ==
  \A a \in AgentId:
    \A t \in ToolCalls(a):
      t.status \in {"planned", "approved", "executed", "observed", "continued"}
      /\ t.approval_receipt # NULL => t.risk_level \in {"high", "critical"}
```

**F6 (Workflows)**:

```tla
WorkflowInvariant ==
  \A p \in packets:
    /\ p.depends_on \subseteq DOMAIN packets
    /\ p.judge \in DOMAIN judges
    /\ p.retry_policy \in RetryPolicies
    /\ p.preregistered_plan # NULL => p.problem_statement # NULL
```

**F7 (Holes)**:

```tla
HoleInvariant ==
  \A h \in holes:
    /\ h.source_root = MerkleRoot(h.source_state)
    /\ h.hole_root = MerkleRoot(h.hole_state)
    /\ h.status \in {"created", "running", "diffed", "approved", "merged", "rejected"}
    /\ h.status = "merged" => h.approval_receipt # NULL
```

**F8 (Telemetry)**:

```tla
TelemetryInvariant ==
  \A e \in telemetry:
    /\ e.type \in {"tool_call", "lane_turn", "judge", "memory_event"}
    /\ e.latency_ms >= 0
    /\ e.ids_only  \* No args/results/text
    /\ e.receipt_hash = Hash(e)
```

**F9 (Compaction)**:

```tla
CompactionInvariant ==
  \A a \in AgentId:
    LET ctx == GetContext(a) IN
    /\ ctx.fold_threshold = 0.60
    /\ ctx.pinned_messages = {"system", "original_user"}
    /\ ctx.checkpoint = BrainContinuityCheckpoint(ctx)
```

**F10 (Streaming)**:

```tla
StreamingInvariant ==
  \A req \in Requests:
    req.streaming = TRUE
    /\ req.ttft_ms = MeasureTTFT(req)
    /\ req.ttft_ms > 0
```

**F11 (Prompt-Injection Hygiene)**:

```tla
PromptInjectionInvariant ==
  \A m \in InjectedMemory:
    m.risk_score <= config.inject_threshold
    \/ m.quarantined = TRUE
```

### Non-Functional Requirements as Temporal Properties

**NF1 (No OOM)**:

```tla
NoOOM ==
  [](peak_memory <= config.vram_guard_gb)
```

**NF2 (Append-Only)**:

```tla
AppendOnly ==
  \A log \in {mind_log, telemetry, events_log}:
    [](Len(log) >= Len(log)@prev)
```

**NF3 (No Secret Leakage)**:

```tla
NoSecretLeakage ==
  [](\A e \in telemetry: \A s \in Secrets: s \notin e)
```

**NF4 (Portability)**:

```tla
Portability ==
  \A p \in Paths: p = ResolveWorkspaceRoot(p)
```

**NF5 (Determinism)**:

```tla
Determinism ==
  \A r \in Receipts: Hash(r) = RecomputeHash(r)
```

**NF6 (Graceful Degradation)**:

```tla
GracefulDegradation ==
  [](config.embed_endpoint = NULL => RAGMode = "BM25")
```

**NF7 (Recoverability)**:

```tla
Recoverability ==
  [](RestartPossible /\ CheckpointValid)
```

**NF8 (Security First)**:

```tla
SecurityFirst ==
  ThreatModelSigned @ Day1
  /\ RoleMatrixSigned @ Day1
  /\ AuditLogEnabled @ Day1
```

**NF9-11 (Energy/SLO)**:

```tla
EnergySLO ==
  [](tok_per_sec_p99 >= config.tok_s_p99_min)
  /\ [](joules_per_token <= config.joules_per_token_max)
  /\ [](peak_memory_gb <= config.peak_memory_gb_max)
```

### Change Control as State Machine

```tla
ChangeControl ==
  \A req \in RequirementChanges:
    req.type \in {"add", "modify", "deprecate"}
    /\ req.pre_reg_hash # NULL  \* C11.3
    /\ req.impact_analysis # NULL
    /\ req.approval_receipt # NULL
    /\ req.migration_plan # NULL  \* forward-only
```

### Requirements as Executable Tests

Each requirement maps to an **executable property test**:

```rust
// Example: F7 Hole isolation property test
#[quickcheck]
fn hole_merge_preserves_merkle_roots(
    source: HoleState,
    hole: HoleState,
    approval: ApprovalReceipt
) -> TestResult {
    let merged = merge_holes(source, hole, approval)?;
    TestResult::from_bool(
        MerkleRoot(merged) == ComputeMerkleRoot(source, hole, approval)
    )
}
```

### Verification Checklist

| Requirement | Formal Spec | Property Test | Ablation  | CI Gate |
| ----------- | ----------- | ------------- | --------- | ------- |
| F1          | ✓ TLA+      | ✓ quickcheck  | A5        | C15.1-2 |
| F2          | ✓ TLA+      | ✓ quickcheck  | A9        | A9      |
| F3          | ✓ TLA+      | ✓ quickcheck  | A9        | C15.2   |
| F4          | ✓ TLA+      | ✓ quickcheck  | A8        | A8      |
| F5          | ✓ TLA+      | ✓ quickcheck  | C16.1     | C15.2   |
| F6          | ✓ TLA+      | ✓ quickcheck  | A7        | C15.1-2 |
| F7          | ✓ TLA+      | ✓ quickcheck  | property  | C15.1   |
| F8          | ✓ TLA+      | ✓ quickcheck  | A4        | C15.5   |
| F9          | ✓ TLA+      | ✓ quickcheck  | A2        | C15.4   |
| F10         | ✓ TLA+      | ✓ quickcheck  | TTFT      | C15.1-2 |
| F11         | ✓ TLA+      | ✓ quickcheck  | A10       | C15.6   |
| NF1         | ✓ TLA+      | ✓ stress      | A1        | C15.1   |
| NF2         | ✓ TLA+      | ✓ quickcheck  | property  | C15.1   |
| NF3         | ✓ TLA+      | ✓ scan        | scan      | C15.6   |
| NF4         | ✓ TLA+      | ✓ smoke       | smoke     | C15.2   |
| NF5         | ✓ TLA+      | ✓ replay      | replay    | C15.1   |
| NF6         | ✓ TLA+      | ✓ smoke       | A3        | C15.1   |
| NF7         | ✓ TLA+      | ✓ restart     | restart   | C15.4   |
| NF8         | ✓ TLA+      | ✓ checklist   | checklist | C15.6   |
| NF9-11      | ✓ TLA+      | ✓ perf        | perf      | C15.7   |

---

## Contributor Deepening — Muse (2026-10-26)

### C08.6 Requirement lifecycle states (depth: what PROPOSED → required actually means)

The traceability matrix marks rows PROPOSED vs required but the transition rule is scattered across C13.1, ADR-22, and rule 8. Frozen lifecycle [PROPOSED — consolidates existing rules, adds no new gate]:

```
DRAFT → PROPOSED (in matrix) → PREREGISTERED (hash filed, C09)
  → EVIDENCED (ablation PASS + replication) → REQUIRED (row updated, enforcement on)
  → (on FAIL at any stage) OPTIONAL/OFF-BY-DEFAULT with measured cost, or WITHDRAWN
```

- DRAFT → PROPOSED requires: statement + owning category + judge sketch. (All current F/NF rows satisfy this.)
- PROPOSED → PREREGISTERED requires: filed hash per C11.3. No hash, no run, no exceptions.
- PREREGISTERED → EVIDENCED requires: PASS on primary + replication on second hardware (C09.4). Single-hardware PASS stays PREREGISTERED.
- EVIDENCED → REQUIRED requires: necessity ablation (remove → regress, C13.1) + C36 correction entry updating the matrix row.
- Every transition is recorded as a dated entry; transitions never rewrite the row's history (C03.3).

### C08.7 Orphan-requirement rule (depth: matrix completeness)

[INFERRED — audit of the matrix against categories read so far; ends at C08/Design.md + C01–C07 files:] every requirement ID must appear in at least one verification gate (C15) AND one sprint day (C32). Rows failing this are orphans and block Gate 1:

- F2 maps to "A9" in the Gate column — but A9 is an ablation, not a C15 gate. Orphan-by-reference: rewrite as the C15 gate number covering integration tests, or record the deviation.
- F4, F6 map to "A8"/"A7" with gate entries that duplicate the ablation ID — same orphan class.
- F5 cites "C16.1" — a necessity ablation, not a gate; the row needs its C15 gate number alongside.

Frozen rule: the Gate column contains C15 gate IDs only; ablation IDs live in the Ablation column (which F2/F4/F6 already populate). Fix at C15 pass; recorded here so it is not lost.

What was NOT checked here: C15 gate numbering vs these rows (verify at C15); C32 day assignments (verify at C32).

## Open questions / gaps

- Numbers for NF targets TBD until hardware benchmarks land (C21).
