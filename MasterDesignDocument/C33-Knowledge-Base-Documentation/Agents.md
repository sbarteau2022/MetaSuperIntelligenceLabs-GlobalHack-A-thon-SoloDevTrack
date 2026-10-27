# C33 — Agents.md

**Category:** C33-Knowledge-Base-Documentation  
**Purpose:** Agent personas operating on the documentation system.

---

## Documentation Agents

### claim-linter

- **Role**: Validates claim labels, evidence pointers, status transitions in all `.md` files
- **Trigger**: Pre-commit hook; CI gate (Gate 2)
- **Capabilities**:
  - Read: All `Cxx/Design.md`, `Cxx/Rules.md`, `PreRegistrations/*.md`, `Claims_Register.md`
  - Write: `Cxx/Claims_Register.md` (corrections only, append-only)
- **Approval**: Auto (pure validation; no side effects)
- **Effect**: `Pure`
- **Config**: `tools/claim_linter.toml`

### migration-auditor

- **Role**: Scans for missing external references; proposes and executes migrations
- **Trigger**: Weekly cron (Mondays 03:00 UTC); manual via `cargo xtask doc-audit`
- **Capabilities**:
  - Read: All `.md` files; external reference targets (if accessible)
  - Write: `migration_status.json`; target `Cxx/Design.md` (migrated content)
- **Approval**: Explicit (proposes changes; human reviews diff)
- **Effect**: `WriteLocal`
- **Config**: `tools/migration_auditor.toml`

### five-file-enforcer

- **Role**: Ensures every `Cxx` category has complete five-file set
- **Trigger**: CI gate (Gate 1); pre-commit
- **Capabilities**:
  - Read: Directory listing `MasterDesignDocument/C*/`
  - Write: None (reports only)
- **Approval**: Auto (fail build if missing)
- **Effect**: `Pure`
- **Config**: None

### rollup-generator

- **Role**: Generates daily Neural Map roll-up (C31.4) from event ledger
- **Trigger**: Daily 02:00 UTC via cron
- **Capabilities**:
  - Read: Event ledger (SQLite, read-only)
  - Write: `neural_map_YYYYMMDD.json` in `rollups/`
- **Approval**: Auto (read-only ledger; append-only output)
- **Effect**: `WriteLocal`
- **Config**: `tools/rollup_generator.toml`

### crosslink-validator

- **Role**: Validates all `Cxx` references resolve to existing files
- **Trigger**: CI gate (Gate 1); pre-commit
- **Capabilities**:
  - Read: All `.md` files
  - Write: None (reports only)
- **Approval**: Auto (fail build if broken refs)
- **Effect**: `Pure`
- **Config**: None

---

## Agent Communication Protocol

Agents communicate via **structured events** in the event ledger (C31.1):

```json
{
  "event_type": "DocAgentAction",
  "payload": {
    "agent": "claim-linter",
    "action": "validate",
    "files_checked": 42,
    "claims_found": 156,
    "violations": 0,
    "duration_ms": 234
  }
}
```

**Privacy**: Only counts/ids/latency; no file contents or claim texts in telemetry.

---

## Agent Registry Entries (C28)

Each agent registered as skill with `entry = "native"` (built-in):

```json
{
  "id": "agent.claim-linter",
  "version": "1.0.0",
  "depends_on": [],
  "hooks": ["PreCommit", "PrePush"],
  "entry": "native",
  "caps": ["fs.read", "ledger.write"],
  "provenance": "sha256:...",
  "lazy": false
}
```

---

## Verification Gates (C15)

| Gate                | Test                                                 | Threshold |
| ------------------- | ---------------------------------------------------- | --------- |
| Claim linter        | Runs on all .md; 0 unlabeled claims                  | 100%      |
| Migration auditor   | Finds 0 unresolved external refs                     | 100%      |
| Five-file enforcer  | All C01–C35 + scaffold appendix folders have 5 files | 100%      |
| Rollup generator    | Daily rollup completes < 60s                         | 100%      |
| Crosslink validator | All Cxx refs resolve                                 | 100%      |
