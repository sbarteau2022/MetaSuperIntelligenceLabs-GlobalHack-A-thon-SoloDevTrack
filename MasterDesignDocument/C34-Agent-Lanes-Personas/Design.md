# C34 — Agent Lanes, Personas & Responsibilities

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Define lane roster roles beyond generic agents. Lane catalog with capabilities, handoff protocols as session types, approval policies, lane telemetry identity, and receipt provenance.

## Subcategories (broad topics)

- C34.1 Lane catalog (builder, contributor, external/OpenAI-compatible)
- C34.2 Role responsibilities and hand-off protocols
- C34.3 Lane-specific tool/MCP access and RAG collection
- C34.4 Approval policy per lane and escalation
- C34.5 Lane telemetry identity and receipt provenance

---

## Canonical Decisions (frozen for design)

| Decision                                                                          | Rationale                              |
| --------------------------------------------------------------------------------- | -------------------------------------- |
| **Lane catalog** = data-driven (N lanes) in `configurations.json` (ADR-03, C30.1) | Extensible; no fixed slots             |
| **Handoff protocols** = session types (linear, typed)                             | Verifiable; no protocol mismatch       |
| **Per-lane tool/MCP/RAG** = allow-lists in lane config                            | Least privilege; isolation             |
| **Approval policy** = per-lane, per-effect (C29.4)                                | Context-appropriate gating             |
| **Lane telemetry identity** = lane_id in all events/receipts                      | Traceability; interference measurement |

---

## Mathematical Specification

### 1. Lane Persona as Typed Record

A **lane persona** `P` extends the lane configuration `L` (C30.1) with behavioral contracts:

```
P = (lane_id: LaneId,
     config: LaneConfig,              -- from C30.1
     responsibilities: Set<Responsibility>,
     handoff_protocols: Map<LaneId, SessionType>,
     escalation_path: List<LaneId>,
     rag_queries: RagQueryProfile,
     telemetry_identity: TelemetryIdentity)
```

**Responsibility** ∈ {
`CodeGeneration`, `CodeReview`, `Testing`, `Documentation`,
`Research`, `Planning`, `Deployment`, `Monitoring`,
`SecurityAudit`, `Compliance`, `ExternalAPI`
}

**SessionType** = Linear session type for handoff:

```
Handoff(from: LaneId, to: LaneId) =
  !TaskContext(from). ?Ack(to). !Artifacts(from). ?Confirm(to). End
```

- `!` = send, `?` = receive
- Linear: each message consumed exactly once
- **Verification**: Session type checker at compile (C14.8)

### 2. Lane Catalog (Default Seed)

```json
{
  "config_version": 3,
  "lanes": [
    {
      "id": "builder",
      "name": "Builder",
      "description": "Code generation, modification, and refactoring",
      "responsibilities": ["CodeGeneration", "Testing", "Refactoring"],
      "tools_allowlist": [
        "fs.read",
        "fs.write",
        "fs.list",
        "git.status",
        "git.diff",
        "git.commit",
        "code.parse",
        "code.query",
        "code.hover",
        "code.definition",
        "process.spawn"
      ],
      "rag_collection": "codebase",
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "auto",
        "WriteRemote": "explicit",
        "Destructive": "human"
      },
      "brigade_radius": 0.5,
      "submanifold_anchor": { "h3": [1.0, 0.0, 0.0, 0.0], "t2": [0.0, 0.0] },
      "max_concurrent": 4,
      "handoff_protocols": {
        "contributor": "!TaskContext. ?Ack. !Artifacts. ?Confirm. End",
        "reviewer": "!TaskContext. ?Ack. !Artifacts. ?Confirm. End"
      },
      "escalation_path": ["architect", "security"],
      "rag_queries": {
        "default_k": 10,
        "bm25_weight": 1.0,
        "dense_weight": 0.0
      },
      "telemetry_identity": "lane.builder",
      "enabled": true
    },
    {
      "id": "contributor",
      "name": "Contributor",
      "description": "Code review, testing, documentation, and research",
      "responsibilities": [
        "CodeReview",
        "Testing",
        "Documentation",
        "Research"
      ],
      "tools_allowlist": [
        "fs.read",
        "fs.list",
        "git.status",
        "git.diff",
        "code.hover",
        "code.definition",
        "code.references",
        "web.search",
        "web.fetch"
      ],
      "rag_collection": "docs",
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "explicit",
        "WriteRemote": "human",
        "Destructive": "human"
      },
      "brigade_radius": 0.3,
      "submanifold_anchor": {
        "h3": [1.0, 0.0, 0.0, 0.0],
        "t2": [3.14159, 0.0]
      },
      "max_concurrent": 2,
      "handoff_protocols": {
        "builder": "!TaskContext. ?Ack. !Artifacts. ?Confirm. End",
        "reviewer": "!ReviewRequest. ?Ack. !ReviewResult. ?Confirm. End"
      },
      "escalation_path": ["builder", "architect"],
      "rag_queries": {
        "default_k": 15,
        "bm25_weight": 1.0,
        "dense_weight": 0.5
      },
      "telemetry_identity": "lane.contributor",
      "enabled": true
    },
    {
      "id": "reviewer",
      "name": "Reviewer",
      "description": "Security audit, compliance, and quality gate",
      "responsibilities": ["SecurityAudit", "Compliance", "QualityGate"],
      "tools_allowlist": [
        "fs.read",
        "fs.list",
        "git.status",
        "git.diff",
        "code.hover",
        "code.definition",
        "web.search"
      ],
      "rag_collection": "policies",
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "human",
        "WriteRemote": "human",
        "Destructive": "human"
      },
      "brigade_radius": 0.2,
      "submanifold_anchor": { "h3": [1.0, 0.0, 0.0, 0.0], "t2": [1.5708, 0.0] },
      "max_concurrent": 1,
      "handoff_protocols": {
        "builder": "!ReviewRequest. ?Ack. !ReviewResult. ?Confirm. End",
        "contributor": "!ReviewRequest. ?Ack. !ReviewResult. ?Confirm. End"
      },
      "escalation_path": ["architect", "security"],
      "rag_queries": {
        "default_k": 20,
        "bm25_weight": 1.0,
        "dense_weight": 0.0
      },
      "telemetry_identity": "lane.reviewer",
      "enabled": true
    },
    {
      "id": "external",
      "name": "External (OpenAI-Compatible)",
      "description": "OpenAI-compatible endpoint for external model integration",
      "responsibilities": ["ExternalAPI"],
      "tools_allowlist": [],
      "rag_collection": null,
      "approval_policy": {
        "Pure": "auto",
        "ReadOnly": "auto",
        "WriteLocal": "human",
        "WriteRemote": "human",
        "Destructive": "human"
      },
      "brigade_radius": 0.0,
      "submanifold_anchor": null,
      "max_concurrent": 8,
      "handoff_protocols": {},
      "escalation_path": ["architect"],
      "rag_queries": null,
      "telemetry_identity": "lane.external",
      "enabled": true,
      "external_endpoint": {
        "base_url": "${OPENAI_BASE_URL}",
        "model": "${OPENAI_MODEL}",
        "api_key_slot": "openai_api_key"
      }
    }
  ],
  "default_lane": "builder"
}
```

### 3. Handoff Protocol as Session Type

**Session Type Grammar**:

```
S ::= End
    | !T.S          -- send value of type T, continue as S
    | ?T.S          -- receive value of type T, continue as S
    | &{l_i: S_i}   -- offer choice (external choice)
    | ⊕{l_i: S_i}   -- select choice (internal choice)
    | μt.S          -- recursive type
```

**Builder → Contributor Handoff**:

```
BuilderContributorHandoff =
  !TaskContext.          -- send task context
  ?Ack.                  -- receive acknowledgment
  !Artifacts.            -- send generated artifacts
  ?Confirm.              -- receive confirmation
  End
```

**Contributor → Reviewer Handoff**:

```
ContributorReviewerHandoff =
  !ReviewRequest.        -- send review request with artifacts
  ?Ack.                  -- receive acknowledgment
  !ReviewResult.         -- send review result (pass/fail/changes)
  ?Confirm.              -- receive confirmation
  End
```

**Type Safety**: Session type checker ensures:

- Dual types match (sender `!T` matches receiver `?T`)
- Linearity: each channel used exactly once
- Progress: no deadlocks in well-typed sessions

### 4. Approval Policy as Effect-Gated Matrix

**ApprovalPolicy** = `Map<EffectLevel, ApprovalMode>`

**ApprovalMode** = `Auto | Explicit | Human`

| Lane        | Pure | ReadOnly | WriteLocal | WriteRemote | Destructive |
| ----------- | ---- | -------- | ---------- | ----------- | ----------- |
| builder     | auto | auto     | auto       | explicit    | human       |
| contributor | auto | auto     | explicit   | human       | human       |
| reviewer    | auto | auto     | human      | human       | human       |
| external    | auto | auto     | human      | human       | human       |

**Escalation**: If `mode = human` and no human available within `timeout` (default 5 min), task queued with `EscalationPending` event.

### 5. RAG Query Profile per Lane

**RagQueryProfile** = `(default_k: ℕ, bm25_weight: ℝ, dense_weight: ℝ, filters: Map<String, String>)`

| Lane        | default_k | bm25_weight | dense_weight | filters              |
| ----------- | --------- | ----------- | ------------ | -------------------- |
| builder     | 10        | 1.0         | 0.0          | `{"type": "code"}`   |
| contributor | 15        | 1.0         | 0.5          | `{"type": "doc"}`    |
| reviewer    | 20        | 1.0         | 0.0          | `{"type": "policy"}` |
| external    | N/A       | N/A         | N/A          | N/A                  |

**Dense opt-in** (ADR-02): `dense_weight > 0` only if embed endpoint configured.

### 6. Telemetry Identity and Receipt Provenance

**TelemetryIdentity** = `lane.{id}` — prefixed on all events/receipts from lane.

**Receipt Provenance**: Every receipt from lane `L` includes:

```
receipt.lane_id = L.id
receipt.telemetry_identity = L.telemetry_identity
receipt.provenance_fiber = F_L  -- brigade fiber (C06.12)
```

**Cross-lane trace**: Given receipt `r`, can reconstruct:

1. Origin lane: `r.lane_id`
2. Brigade fiber: `r.provenance_fiber`
3. Handoff chain: via `CrossLaneMessage` events with matching `receipt_hash`

---

## Verification Gates (C15)

| Gate                        | Test                                     | Threshold    |
| --------------------------- | ---------------------------------------- | ------------ |
| Session type checking       | All handoff protocols type-check         | 100%         |
| Lane config validation      | All lanes validate against schema        | 100%         |
| Tool allowlist enforcement  | Lane cannot call tool not in allowlist   | 0 violations |
| Approval policy enforcement | Effect → mode mapping respected          | 100%         |
| Handoff linearity           | Each handoff channel used exactly once   | 100%         |
| Telemetry identity          | All lane events carry `lane_id`          | 100%         |
| Receipt provenance          | All receipts carry lane_id + fiber       | 100%         |
| RAG profile                 | Dense weight = 0 unless embed configured | 100%         |

---

## Preregistration Entries (C09)

### A28 — Handoff Protocol Correctness

- Hypothesis: Session-typed handoffs prevent protocol mismatches.
- Judge: 1,000 random handoffs; 0 type errors; 0 deadlocks.
- Status: TBD.

### A29 — Lane Isolation

- Hypothesis: Lane cannot access tools/RAG outside allowlist.
- Judge: 10,000 cross-lane tool calls; 0 succeed.
- Status: TBD.

### A30 — Approval Policy Enforcement

- Hypothesis: Destructive actions require human approval in all lanes.
- Judge: 100 destructive attempts without human; 0 execute.
- Status: TBD.

---

## Open Questions / Gaps

- Dynamic lane creation at runtime (requires config hot-reload).
- Lane-specific model parameters (temperature, top-p) in config.
- External lane: MCP server integration vs OpenAI-compatible.
- Telemetry identity rotation for privacy (deferred to P6).

---

## Contributor Deepening — Muse (2026-10-26)

### C34.7 Lane-count ceiling (depth: N lanes is bounded by physics, not config)

The roster is data-driven (N lanes, no fixed slots) — but N is bounded by the C01 feasibility theorem and the C21 capacity table (ACFT topology: 8 lanes; standard: 4). Frozen admission rule [PROPOSED — rule is my proposal; C01/C21 are the authority]:

1. Lane creation is an admission-controlled operation: a new lane is admitted iff projected VRAM (model weights + per-lane Atlas fiber + per-lane context budget share) stays under the C21 topology ceiling with 10% headroom. The headroom is for compaction transients (fold doubles transient allocation — checkpoint + folded context coexist briefly).
2. The feasibility inequality (C01) is evaluated at admission time with N+1, not N: if it fails, the lane is created DISABLED (config present, enabled=false) with a `capacity_blocked` receipt — visible, auditable, non-fatal. Silent lane rejection is a repudiation vector (operator believes capacity exists that does not).
3. The C34 seed catalog (builder/contributor/reviewer/external) consumes 4 lanes at max_concurrent 4/2/1/8 — the external lane's max_concurrent=8 is connection concurrency (no local compute), not VRAM lanes. Document the distinction: connection lanes vs compute lanes have different admission math. A reviewer treating external max_concurrent=8 as 8 VRAM lanes will mis-audit capacity.

### C34.8 Handoff vs escalation (depth: two paths, one trigger condition)

Handoff protocols (session-typed, linear) move work; escalation paths (ordered lane lists) move authority. The trigger separating them is unstated. Frozen trigger [PROPOSED — trigger is my proposal]:

- HANDOFF when: the receiving lane's allowlist + approval policy can complete the task without privilege elevation. Routine, receipted, no human involved.
- ESCALATION when: the task requires an effect level above the current lane's approval mode (e.g. contributor needs WriteRemote → human; builder needs Destructive → human), OR the C30 breaker isolated the lane, OR the C27.8 AlternativeLane already failed. Escalation always emits an EscalationPending-class event and, at Human mode, blocks the packet (no silent auto-escalation to a more-privileged lane — that is elevation, C16).
- Misrouted handoff (sent to a lane whose allowlist cannot execute it) is a scheduler fault, not a protocol fault: the receiving lane REJECTS with `capability_mismatch` receipt, and the rejection counts against the sender's routing calibration (feeds C06.16-style monitoring if built for routing judges).

What was NOT checked here: lane runtime implementation (outside workspace); whether escalation_path lists are enforced or advisory today (verify at implementation).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Agent Lanes (Original)

- **Lane catalog**: builder, contributor, external/OpenAI-compatible
- **Per-lane**: tools allow-list, RAG collection, approval policy
- **Handoff protocols** between lanes
- **Approval policy** per lane and escalation
- **Lane telemetry identity** and receipt provenance
