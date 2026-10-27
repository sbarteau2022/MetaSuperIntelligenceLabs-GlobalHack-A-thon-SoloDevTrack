# C33 — Knowledge Base, Documentation & Evidence Standards

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Documentation system, governance file set, and provenance standards. Five-file set per workstream, claim labeling automation, corrections as entries, dependency inventory, missing-document migration plan, naming/versioning/date conventions.

## Subcategories (broad topics)

- C33.1 Five-file set per workstream (Design, Pre-Registration, Claims Register, Successes, Failures)
- C33.2 Claim labeling (VERIFIED/REPORTED/INFERRED/PROPOSED) with evidence
- C33.3 Corrections as entries, not edits
- C33.4 Dependency inventory (AtomaApp docs 27/30/31, tools-design, mcp-servers design, unification audits)
- C33.5 Missing-document migration plan (external references not present in repo)
- C33.6 Naming/versioning/date conventions

---

## Canonical Decisions (frozen for design)

| Decision                                                | Rationale                                         |
| ------------------------------------------------------- | ------------------------------------------------- |
| **Five-file set** mandatory per workstream              | Complete evidence trail; no "undocumented" work   |
| **Claim labeling** required on every assertion          | Truthfulness; auditability; no unlabeled claims   |
| **Corrections as entries** (append-only)                | History preserved; no silent rewrites             |
| **Missing external docs** → migrate or delete reference | No dangling references in ship artifacts (ADR-07) |
| **Naming convention**: `CXX-Category-Name/Design.md`    | Consistent navigation; sortable                   |
| **Date format**: ISO 8601 (YYYY-MM-DD) in all docs      | Unambiguous; sortable; standard                   |

---

## Mathematical Specification

### 1. Five-File Set as Formal Requirement

For each workstream `W` (category `Cxx`), the following files **must exist**:

```
W.files = {
  Design.md,              -- What and why; formal spec
  Pre-Registration.md,    -- Predictions and judges, written BEFORE run
  Claims_Register.md,     -- Claims with labels + status + evidence pointers
  Successes.md,           -- Verified successes with receipts
  Failures.md             -- Failed attempts with root cause + receipts
}
```

**Invariant**: `∀W: |W.files| = 5 ∧ ∀f ∈ W.files: exists(f)`

**Verification gate** (C15): `check_five_file_set() → bool` scans all `Cxx/` folders.

### 2. Claim Labeling as Type System

**Claim** `c` = `(id: String, text: String, label: Label, evidence: EvidencePointer, date: Date, status: Status)`

**Label** ∈ { `VERIFIED`, `REPORTED`, `INFERRED`, `PROPOSED` }

| Label      | Definition                                      | Evidence Required           |
| ---------- | ----------------------------------------------- | --------------------------- |
| `VERIFIED` | Measured or read from source itself             | Link to measurement/run/log |
| `REPORTED` | A source says so; unchecked                     | Citation to source          |
| `INFERRED` | Reasoned from evidence; say where evidence ends | Premises + inference steps  |
| `PROPOSED` | A plan, not a fact                              | Design doc reference        |

**EvidencePointer** = `(source: URI, location: String, hash: SHA256 ⊕ None)`

**Status** ∈ { `OPEN`, `CONFIRMED`, `REFUTED`, `SUPERSEDED` }

**Claims Register** = append-only JSONL file:

```jsonl
{
  "id": "C05.1",
  "text": "Lorentz H^3 is canonical geometry",
  "label": "VERIFIED",
  "evidence": {
    "source": "monster-hybrid-server/core_lm/src/geometry/lorentz.rs",
    "location": "line 1",
    "hash": "sha256:..."
  },
  "date": "2026-10-26",
  "status": "CONFIRMED"
}
```

**Automation**: `scripts/claim_linter.py` validates:

- Every `.md` file claim has label (regex: `\[(VERIFIED|REPORTED|INFERRED|PROPOSED)\]`)
- Evidence pointer resolves (file exists, hash matches if provided)
- No duplicate claim IDs
- Status transitions valid (OPEN → CONFIRMED/REFUTED/SUPERSEDED)

### 3. Corrections as Entries

**Correction** `c` = `(id: String, corrects: ClaimID, text: String, date: Date, author: String)`

**Rule**: Original claim **never deleted or modified**. Correction appended to `Claims_Register.md` with `status: SUPERSEDED` on original.

**Format**:

```jsonl
{
  "id": "CORR-001",
  "corrects": "C05.1",
  "text": "Curvature is -1, not -0.5",
  "date": "2026-10-27",
  "author": "architect"
}
```

### 4. Dependency Inventory as Directed Graph

**Dependency** `d` = `(from: DocID, to: DocID, type: {hard, soft}, status: {present, migrated, deleted})`

**Inventory** (frozen from audit):

| Referenced Doc                                                  | From    | Type | Status  | Disposition            |
| --------------------------------------------------------------- | ------- | ---- | ------- | ---------------------- |
| `AtomaApp/docs/27-tree-sitter-master-document.md`               | C29     | hard | missing | Migrate into C29       |
| `AtomaApp/AtomaDB/30-rules-and-governance.md`                   | C16/C23 | hard | missing | Migrate into C16/C23   |
| `AtomaApp/AtomaDB/31-constraint-grammar-decomposition.md`       | C29     | soft | missing | Migrate or drop        |
| `AtomaApp/docs/tools-design.md`                                 | C29     | hard | missing | Migrate into C29       |
| `AtomaApp/mcp-servers/design.md`                                | C18     | hard | missing | Migrate into C18       |
| `AtomaApp/Harnesses-2026-10-23/audits/THREE-WAY-UNIFICATION.md` | C04/C33 | soft | missing | Migrate into C04/C33   |
| User AGENTS.md (claim labeling)                                 | C33     | hard | present | Adopted in C33.2/C33.6 |

**Migration status tracking**: `migration_status.json` updated per migration.

### 5. Missing-Document Migration Plan

For each missing doc `D`:

1. **Locate** original if possible (search workspace, ask owner)
2. **Extract** relevant content (not wholesale copy)
3. **Integrate** into target category `Design.md` with proper claim labels
4. **Delete** "Read next" reference from `README.md` or source doc
5. **Record** migration in `migration_status.json`:

```json
{
  "original": "AtomaApp/docs/27-tree-sitter-master-document.md",
  "target": "C29-Tool-Registry-Code-Intel/Design.md",
  "sections_migrated": ["C29.3"],
  "date": "2026-10-26",
  "status": "completed"
}
```

### 6. Naming/Versioning/Date Conventions

| Artifact          | Convention                           | Example                          |
| ----------------- | ------------------------------------ | -------------------------------- |
| Category folder   | `CXX-Category-Name` (XX zero-padded) | `C28-Skill-Registry-Hooks`       |
| Design doc        | `Design.md`                          | `C28/Design.md`                  |
| Rules doc         | `Rules.md`                           | `C28/Rules.md`                   |
| Telemetry doc     | `TelemetryAndDiagnostics.md`         | `C28/TelemetryAndDiagnostics.md` |
| Agents doc        | `Agents.md`                          | `C33/Agents.md`                  |
| Modelfiles doc    | `Modelfiles.md`                      | `C33/Modelfiles.md`              |
| Pre-registration  | `PRE-REGISTRATION-A{X}-A{Y}.md`      | `PRE-REGISTRATION-A11-A25.md`    |
| Claims register   | `Claims_Register.md`                 | `C28/Claims_Register.md`         |
| Date in text      | ISO 8601                             | `2026-10-26`                     |
| Version in config | SemVer + config_version              | `1.2.0`, `config_version: 3`     |
| Hash references   | `sha256:<hex>`                       | `sha256:a1b2c3...`               |

---

## Agents.md (Sidecar for Documentation Category)

**Purpose**: Documents the agent personas that operate on the documentation system.

```markdown
# C33 — Agents.md

## Documentation Agents

### claim-linter

- **Role**: Validates claim labels, evidence pointers, status transitions
- **Trigger**: Pre-commit hook; CI gate
- **Capabilities**: Read all `.md` files; write `Claims_Register.md` corrections
- **Approval**: Auto (pure validation)

### migration-auditor

- **Role**: Scans for missing external references; proposes migrations
- **Trigger**: Weekly cron; manual
- **Capabilities**: Read repo + external refs; write `migration_status.json`
- **Approval**: Explicit (proposes changes)

### five-file-enforcer

- **Role**: Ensures every Cxx has 5-file set
- **Trigger**: CI gate
- **Capabilities**: List directories; report missing
- **Approval**: Auto (fail build if missing)

### rollup-generator

- **Role**: Generates daily Neural Map roll-up (C31.4)
- **Trigger**: Daily 02:00 UTC
- **Capabilities**: Read event ledger; write `neural_map_YYYYMMDD.json`
- **Approval**: Auto (read-only ledger)
```

---

## Modelfiles.md (Sidecar for Documentation Category)

**Purpose**: Documents the model files (schemas, templates, configs) used by documentation agents.

````markdown
# C33 — Modelfiles.md

## Claim Schema (JSON Schema Draft 2020-12)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["id", "text", "label", "evidence", "date", "status"],
  "properties": {
    "id": { "type": "string", "pattern": "^C\\d+(\\.\\d+)*$|^CORR-\\d+$" },
    "text": { "type": "string", "minLength": 1 },
    "label": {
      "type": "string",
      "enum": ["VERIFIED", "REPORTED", "INFERRED", "PROPOSED"]
    },
    "evidence": {
      "type": "object",
      "required": ["source"],
      "properties": {
        "source": { "type": "string", "format": "uri" },
        "location": { "type": "string" },
        "hash": { "type": "string", "pattern": "^sha256:[a-f0-9]{64}$" }
      }
    },
    "date": { "type": "string", "format": "date" },
    "status": {
      "type": "string",
      "enum": ["OPEN", "CONFIRMED", "REFUTED", "SUPERSEDED"]
    }
  }
}
```
````

## Migration Status Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "array",
  "items": {
    "type": "object",
    "required": ["original", "target", "date", "status"],
    "properties": {
      "original": { "type": "string" },
      "target": { "type": "string" },
      "sections_migrated": { "type": "array", "items": { "type": "string" } },
      "date": { "type": "string", "format": "date" },
      "status": {
        "type": "string",
        "enum": ["pending", "in_progress", "completed", "dropped"]
      }
    }
  }
}
```

## Five-File Set Manifest

```json
{
  "workstream": "C28",
  "files": {
    "Design.md": { "present": true, "hash": "sha256:..." },
    "Pre-Registration.md": { "present": true, "hash": "sha256:..." },
    "Claims_Register.md": { "present": true, "hash": "sha256:..." },
    "Successes.md": { "present": true, "hash": "sha256:..." },
    "Failures.md": { "present": true, "hash": "sha256:..." }
  }
}
```

## Claim Linter Config

```toml
[claim_linter]
claim_pattern = '\\[(VERIFIED|REPORTED|INFERRED|PROPOSED)\\]'
evidence_required_labels = ["VERIFIED", "REPORTED", "INFERRED"]
max_claims_per_file = 100
```

---

## Verification Gates (C15)

| Gate                   | Test                                                 | Threshold                     |
| ---------------------- | ---------------------------------------------------- | ----------------------------- |
| Five-file set          | All Cxx folders have 5 files                         | C01–C35 + scaffold appendices |
| Claim labeling         | All claims in .md files have label                   | 100%                          |
| Evidence resolution    | All evidence pointers resolve                        | 100%                          |
| Correction append-only | No claim deleted/modified; only corrections appended | 100%                          |
| Migration completeness | All missing docs migrated or dropped                 | 100%                          |
| Naming convention      | All files match convention                           | 100%                          |
| Claim linter           | `scripts/claim_linter.py` passes                     | 0 errors                      |

---

## Preregistration Entries (C09)

### A26 — Documentation Completeness

- Hypothesis: Five-file set enforced; claim linter catches 100% unlabeled claims.
- Judge: Inject 50 unlabeled claims; all caught.
- Status: TBD.

### A27 — Migration Traceability

- Hypothesis: Every missing external doc traced to migration or deletion.
- Judge: Audit `migration_status.json`; 0 unresolved references.
- Status: TBD.

---

## Open Questions / Gaps

- Claim linter integration with IDE (VS Code/Zed) for real-time feedback.
- Automated evidence hash computation on file save.
- Cross-workstream claim dependency tracking.

---

## Contributor Deepening — Muse (2026-10-26)

### C33.7 Self-application audit (depth: the docs standard grades its own workspace)

C33 defines the five-file set, claim labeling, and migration tracking — and this workspace is its first subject. Applying the C33 gates to the current workspace state [VERIFIED — directory listings + file reads in this pass]:

1. Five-file set: every Cxx folder currently contains Design.md + Rules.md only (plus TelemetryAndDiagnostics/Agents/Modelfiles sidecars in deepened categories). NO category has Pre-Registration.md, Claims_Register.md, Successes.md, or Failures.md as separate files — preregs live in PreRegistrations/ (centralized, correct per C09) but Claims/Successes/Failures exist nowhere yet. Per the C33.1 invariant (all 5 files exist per workstream), the workspace is 2/5 per category. Frozen disposition: either (a) amend the invariant — preregs centralized in PreRegistrations/, claims/successes/failures centralized per sprint (not per category) — or (b) scaffold the missing files. Centralized claims with per-category sections is my PROPOSED recommendation (36 folders × 3 near-empty files = noise; one CLAIMS_REGISTER.md with category prefixes = auditable). Recommend a C36 correction entry; do not silently ignore the invariant.
2. Claim labeling: the claim-linter regex requires [BRACKETED] labels on every `.md` claim. My own contributions in this pass use [VERIFIED/PROPOSED/INFERRED — ...] bracket form — compliant by construction. But the PRE-EXISTING design text (migrated + deepened sections) carries almost no bracketed labels: the linter, run today, would fail every category. The linter rollout needs a grandfather rule: pre-existing text gets a dated labeling pass (scheduled task, Day 6–7 with C33), not a Day-1 build failure — else G1 fails on legacy content and blocks the sprint on day one.
3. Dependency inventory: the C33.4 table lists 6 missing external docs (AtomaApp paths). My C05.14 GAP-C05-01 (on-disk systems proof) is the same class: a referenced-but-absent source. Add it to the migration tracker: GAP-C05-01 → migration_status.json entry with status pending. The inventory is the single registry for absent sources — no side-channel gap lists.
4. Naming convention: the C33.6 table specifies `CXX-Category-Name/Design.md` — the workspace uses exactly this (VERIFIED via listings). The convention holds; cite as the one C33 gate currently passing.

What was NOT checked here: PreRegistrations/ content vs the C09.9 checklist (audit scheduled in C32.7 item 3 — still open); migration_status.json existence (not in workspace).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Documentation Standards (Original)

- **Five-file set** per workstream: Design, Pre-Registration, Claims Register, Successes, Failures
- **Claim labeling**: VERIFIED/REPORTED/INFERRED/PROPOSED with evidence
- **Corrections as entries**, not edits
- **Dependency inventory**: 7 external docs referenced; 6 missing
- **Missing-doc migration**: Migrate or delete; no dangling references
- **Naming/versioning/date conventions** standardized
