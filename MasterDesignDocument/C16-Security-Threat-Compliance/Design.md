# C16 — Security, Threat Model & Compliance

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

First-class security/compliance before build: threat model, role matrix, audit logs, erasure,
injection hygiene.

## Subcategories (broad topics)

- C16.1 Threat model (assets, actors, entry points, trust boundaries)
- C16.2 Role matrix (roles → permissions → tools → approval policy)
- C16.3 Secret-by-slot discipline (no literal secrets; slot resolution)
- C16.4 Merkle receipts and audit log (append-only, tamper-evident)
- C16.5 Prompt-injection hygiene for injected memory (sanitization/tagging, red-team A10)
- C16.6 Right-to-erasure plan
- C16.7 Regulatory mapping (AI Act/GDPR)
- C16.8 Compliance smoke checks and evidence

## Open questions / gaps

- Detail threat model and role matrix with owners.

## Contract catalog (frozen for design)

| Asset               | Threat                               | Control                                                        | Evidence              |
| ------------------- | ------------------------------------ | -------------------------------------------------------------- | --------------------- |
| Secrets             | Leakage in logs/telemetry            | Slot resolution, no literal secrets, grep/scan checks          | NF3, receipt IDs only |
| Memory injection    | Prompt injection via stored memory   | Sanitization/tagging pass before injection (F11); red-team A10 | A10                   |
| Holes/merge         | Unauthorized write                   | Approval gate + Merkle source/hole root check                  | A7                    |
| Telemetry           | Privacy leak                         | Counts/ids/latency only; never args/results/text               | F8, §10.4             |
| Identity/provenance | Untraceable action                   | Receipt fingerprint on every mutating action                   | C03.1                 |
| Erasure             | Retained data after deletion request | Right-to-erasure plan per store (C17)                          | NF8                   |

### Role matrix (minimum)

- Reader: read tools, RAG query, memory tail.
- Contributor: + hole create/run/diff (no merge without approval).
- Operator: + ingest, workflow admission, approvals.
- Admin: + config/feature flags, audit export, erasure execution.

## Formal Threat Model (STRIDE)

### Assets

| Asset ID | Asset                         | Classification   | Location                      |
| -------- | ----------------------------- | ---------------- | ----------------------------- |
| A1       | User prompts/context          | PII/Confidential | Memory (RAM), `mind.jsonl`    |
| A2       | Agent memories/summaries      | Confidential     | `mind.jsonl`, RAG collections |
| A3       | Model weights                 | IP/Confidential  | Rapid-MLX (:8001)             |
| A4       | Atlas geometry (`.atoma_map`) | IP/Confidential  | Data root                     |
| A5       | Secrets (API keys, tokens)    | Secret           | Secret slots (Keychain)       |
| A6       | Audit logs                    | Confidential     | `events.sqlite`               |
| A7       | Configuration                 | Internal         | `configurations.json`         |
| A8       | Telemetry                     | Internal         | Prometheus metrics            |

### STRIDE Analysis

| Threat              | Vectors                                                    | Mitigation                                                    | Verification              |
| ------------------- | ---------------------------------------------------------- | ------------------------------------------------------------- | ------------------------- |
| **Spoofing**        | Fake agent registration, forged receipts                   | Mutual TLS (internal), receipt signatures, Merkle roots       | C15.1, C25 property tests |
| **Tampering**       | Memory injection, hole merge manipulation, config drift    | Sanitization (C07.7), Merkle verification, compile-time hooks | A10, C14.8, C25           |
| **Repudiation**     | Deny action, forge audit log                               | Append-only ledger, Merkle receipts, signed build receipts    | C17.3, C15.7              |
| **Info Disclosure** | Telemetry leakage, memory exposure, side-channels          | Telemetry privacy rule (C31), secret slots, constant-time ops | C15.6, NF3                |
| **DoS**             | OOM via malicious prompts, recursive holes, toroidal retry | Memory permits, circuit breakers, depth/budget caps           | C22.1, C25.5              |
| **Elevation**       | Privilege escalation via tool/hole misuse                  | Role matrix (C16.2), approval gates, seatbelt                 | A7, C16.8                 |

### Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                     USER SPACE                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Frontend   │  │   Backend    │  │   Rapid-MLX      │  │
│  │  (Browser)   │──│  (Rust)      │──│  (:8001)         │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                     │           │
│         ▼                 ▼                     ▼           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              TRUST BOUNDARY: SEATBELT                 │  │
│  │  ┌────────────┐ ┌────────────┐ ┌──────────────────┐  │  │
│  │  │ Hole Sub-  │ │  Secrets   │ │  File System     │  │  │
│  │  │ strate     │ │  Manager   │ │  (Sandboxed)     │  │  │
│  │  └────────────┘ └────────────┘ └──────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Formal Role-Based Access Control (RBAC)

```rust
// RBAC as typed capabilities
#[derive(Clone, PartialEq, Eq, Hash)]
enum Permission {
    ReadTool(ToolId),
    WriteTool(ToolId),
    ReadRAG(CollectionId),
    WriteRAG(CollectionId),
    ReadMemory,
    WriteMemory,
    CreateHole,
    RunHole,
    DiffHole,
    ApproveHole,
    MergeHole,
    IngestDocument,
    AdmitWorkflow,
    ApproveWorkflow,
    ConfigureSystem,
    ExportAudit,
    ExecuteErasure,
}

// Role definitions
const ROLES: &[(Role, &[Permission])] = &[
    (Role::Reader, &[
        Permission::ReadTool(_),
        Permission::ReadRAG(_),
        Permission::ReadMemory,
    ]),
    (Role::Contributor, &[
        Permission::ReadTool(_),
        Permission::ReadRAG(_),
        Permission::ReadMemory,
        Permission::CreateHole,
        Permission::RunHole,
        Permission::DiffHole,
    ]),
    (Role::Operator, &[
        Permission::ReadTool(_),
        Permission::WriteTool(_),
        Permission::ReadRAG(_),
        Permission::WriteRAG(_),
        Permission::ReadMemory,
        Permission::WriteMemory,
        Permission::CreateHole,
        Permission::RunHole,
        Permission::DiffHole,
        Permission::ApproveHole,
        Permission::MergeHole,
        Permission::IngestDocument,
        Permission::AdmitWorkflow,
        Permission::ApproveWorkflow,
    ]),
    (Role::Admin, &[
        // All permissions
        Permission::ConfigureSystem,
        Permission::ExportAudit,
        Permission::ExecuteErasure,
    ]),
];

// Authorization check
fn authorize(role: Role, perm: Permission) -> bool {
    ROLES.iter()
        .find(|(r, _)| *r == role)
        .map(|(_, perms)| perms.iter().any(|p| matches_perm(p, &perm)))
        .unwrap_or(false)
}
```

### Secret Management

```rust
// Secret slots — never literal secrets in code/config
struct SecretManager {
    keychain: Keychain,  // macOS Keychain / libsecret
    slots: HashMap<SlotName, SecretRef>,
}

impl SecretManager {
    fn resolve(&self, slot: SlotName) -> Result<Secret> {
        self.keychain.get(slot)
    }

    fn scan_for_literals(&self, code: &str) -> Vec<Violation> {
        // Regex patterns for common secret formats
        // Runs in CI (C15.6)
    }
}
```

### Right-to-Erasure Protocol

```rust
fn execute_erasure(request: ErasureRequest) -> Result<ErasureReceipt> {
    // 1. Verify request authenticity
    verify_signature(&request)?;

    // 2. Identify all stores containing subject data
    let stores = identify_stores(&request.subject_id)?;

    // 3. For each store, execute erasure
    let mut receipts = Vec::new();
    for store in stores {
        let receipt = store.erase(&request.subject_id)?;
        receipts.push(receipt);
    }

    // 4. Verify erasure completeness
    verify_erasure_complete(&request.subject_id, &stores)?;

    // 5. Emit erasure receipt (append-only)
    let erasure_receipt = ErasureReceipt {
        subject: request.subject_id,
        stores_affected: stores,
        receipts,
        timestamp: now(),
        hash: Hash::new(),
    };
    audit_log.append(erasure_receipt)?;

    Ok(erasure_receipt)
}
```

### Regulatory Mapping (AI Act / GDPR)

| Requirement            | Article           | Implementation                                     | Evidence     |
| ---------------------- | ----------------- | -------------------------------------------------- | ------------ |
| Data minimization      | GDPR Art. 5(1)(c) | Context budget (C24), geometric forgetting (C06.2) | C21, C06     |
| Purpose limitation     | GDPR Art. 5(1)(b) | Per-lane RAG scopes, role matrix                   | C16.2, C30   |
| Right to erasure       | GDPR Art. 17      | Erasure protocol above                             | C16.6, C17.6 |
| Records of processing  | GDPR Art. 30      | Event ledger (C17.1)                               | C17          |
| Security of processing | GDPR Art. 32      | Seatbelt, Merkle, encryption                       | C16.3, C25   |
| DPIA                   | GDPR Art. 35      | Threat model (this doc)                            | C16.1        |
| Human oversight        | AI Act Art. 14    | Approval gates, human-in-loop                      | C07.4, C25.4 |
| Transparency           | AI Act Art. 13    | Telemetry, receipts, claims register               | C31, C09     |

---

## Contributor Deepening — Muse (2026-10-26)

### C16.9 Erasure vs append-only: the honest conflict (depth, not novelty)

C16.6 (right-to-erasure) and C03.3/NF2 (append-only truth) are in direct tension: GDPR Art. 17 requires deletion, the ledger forbids it. Frozen resolution [PROPOSED — resolution is my proposal; both requirements are frozen]:

1. Ledger entries are never deleted. Erasure is executed as a tombstone append: ErasureRecord(subject_id_hash, stores_affected, receipts, timestamp) in the ledger + per-store redaction where the store format allows it.
2. Payloads vs hashes. The ledger stores payload_hash, not payloads (C17 schema) — erasure of the payload store (mind.jsonl lines, RAG docs) satisfies Art. 17 for content while the hash chain stays intact for audit. The tombstone records that erasure happened and what was erased (store + clock range), never the erased content.
3. mind.jsonl redaction mechanics. Because mind.jsonl is append-only with O_APPEND, erasure of a line is implemented as a redaction record appended later (kind = redaction, redacts_clock = N) plus logical suppression at read time (MindClient::tail skips redacted clocks). Physical deletion happens only at roll-up compaction (C07.7), where redacted entries are dropped and the roll-up receipt records the drop. Until then, erased-but-present bytes are encrypted-at-rest scope — note as residual risk in the DPIA (C16.1).
4. Backups. Erasure propagates to backups by policy: backups older than the erasure request are destroyed on rotation (bounded retention, e.g. 30 days), not surgically edited. State the retention bound in the erasure plan — unbounded backup retention makes Art. 17 unimplementable.
5. Stale pointer recorded (not silently fixed): the STRIDE table above cites "Sanitization (C07.7)" — the pipeline lives at C07.5 (C11.7 same finding). Recommend a C36 correction entry.

### C16.10 Threat-model freshness (depth: a Day-1 signature that rots)

NF8 requires the threat model signed before Day 1 feature code — but the system changes for 9 more days. Frozen freshness rule [PROPOSED — rule is my proposal; NF8 is the authority]:

- Any C09.8 trigger event (lane change, Atlas change, new skill/tool, embed change) re-opens the threat model for the affected rows. Re-opened rows must be re-signed before the next phase exit.
- A10 (red-team) is re-run at P5, not just once: the Day-1 sanitization pipeline faces a Day-10 attack surface (new lanes, new tools, grown Atlas). A10 PASS at Day 1 + no re-run = PROPOSED hygiene at Day 10, not EVIDENCED.
- The role matrix (C16.2) gains a row per new lane at creation time; a lane operating without a matrix row fails Gate 1 (unregistered capability = elevation vector).

What was NOT checked here: C17.6 erasure mechanics vs the tombstone protocol above (verify at C17); DPIA document existence (not in workspace).

## Open questions / gaps

- Detail threat model and role matrix with owners.
