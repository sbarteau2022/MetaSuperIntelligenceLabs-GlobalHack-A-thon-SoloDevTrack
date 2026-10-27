# C33 — Modelfiles.md

**Category:** C33-Knowledge-Base-Documentation  
**Purpose:** Model files (schemas, templates, configs) used by documentation agents.

---

## Claim Schema (JSON Schema Draft 2020-12)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Claim",
  "type": "object",
  "required": ["id", "text", "label", "evidence", "date", "status"],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^C\\d+(\\.\\d+)*$|^CORR-\\d+$|^A\\d+$",
      "description": "Claim ID: Cxx.y for design claims, CORR-n for corrections, An for ablation claims"
    },
    "text": {
      "type": "string",
      "minLength": 1,
      "maxLength": 5000,
      "description": "Claim text; must be labeled with [VERIFIED|REPORTED|INFERRED|PROPOSED] inline"
    },
    "label": {
      "type": "string",
      "enum": ["VERIFIED", "REPORTED", "INFERRED", "PROPOSED"],
      "description": "Epistemic status of the claim"
    },
    "evidence": {
      "type": "object",
      "required": ["source"],
      "properties": {
        "source": {
          "type": "string",
          "format": "uri-reference",
          "description": "Path to source file or URI (e.g., C05/Design.md, monster-hybrid-server/...)"
        },
        "location": {
          "type": "string",
          "description": "Line number, section, or function name in source"
        },
        "hash": {
          "type": "string",
          "pattern": "^sha256:[a-f0-9]{64}$",
          "description": "SHA-256 of source content at time of claim (optional but recommended for VERIFIED)"
        }
      }
    },
    "date": {
      "type": "string",
      "format": "date",
      "description": "ISO 8601 date of claim entry"
    },
    "status": {
      "type": "string",
      "enum": ["OPEN", "CONFIRMED", "REFUTED", "SUPERSEDED"],
      "description": "Current validation status"
    }
  },
  "additionalProperties": false
}
```

---

## Migration Status Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "MigrationStatus",
  "type": "array",
  "items": {
    "type": "object",
    "required": ["original", "target", "date", "status"],
    "properties": {
      "original": {
        "type": "string",
        "description": "Original external reference path"
      },
      "target": {
        "type": "string",
        "description": "Target category/file in MasterDesignDocument"
      },
      "sections_migrated": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Subcategory IDs migrated (e.g., C29.3)"
      },
      "date": {
        "type": "string",
        "format": "date",
        "description": "Migration completion date"
      },
      "status": {
        "type": "string",
        "enum": ["pending", "in_progress", "completed", "dropped"],
        "description": "Migration status"
      },
      "notes": {
        "type": "string",
        "description": "Rationale for dropped or partial migration"
      }
    },
    "additionalProperties": false
  }
}
```

---

## Five-File Set Manifest Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FiveFileSetManifest",
  "type": "object",
  "required": ["workstream", "files"],
  "properties": {
    "workstream": {
      "type": "string",
      "pattern": "^C\\d{2}$",
      "description": "Category ID (e.g., C28)"
    },
    "files": {
      "type": "object",
      "required": [
        "Design.md",
        "Pre-Registration.md",
        "Claims_Register.md",
        "Successes.md",
        "Failures.md"
      ],
      "properties": {
        "Design.md": { "$ref": "#/definitions/FileEntry" },
        "Pre-Registration.md": { "$ref": "#/definitions/FileEntry" },
        "Claims_Register.md": { "$ref": "#/definitions/FileEntry" },
        "Successes.md": { "$ref": "#/definitions/FileEntry" },
        "Failures.md": { "$ref": "#/definitions/FileEntry" }
      },
      "additionalProperties": false
    },
    "validated_at": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp of last validation"
    }
  },
  "definitions": {
    "FileEntry": {
      "type": "object",
      "required": ["present", "hash"],
      "properties": {
        "present": { "type": "boolean" },
        "hash": { "type": "string", "pattern": "^sha256:[a-f0-9]{64}$" },
        "size_bytes": { "type": "integer", "minimum": 0 }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

---

## Claim Linter Config (TOML)

```toml
# tools/claim_linter.toml

[claim_linter]
# Regex to find claim labels in markdown
claim_pattern = '\\[(VERIFIED|REPORTED|INFERRED|PROPOSED)\\]'

# Labels that require evidence pointer
evidence_required_labels = ["VERIFIED", "REPORTED", "INFERRED"]

# Maximum claims per file (soft limit; warning only)
max_claims_per_file = 100

# Claim ID patterns by type
design_claim_pattern = "^C\\d+(\\.\\d+)*$"
correction_claim_pattern = "^CORR-\\d+$"
ablation_claim_pattern = "^A\\d+$"

# Evidence hash required for VERIFIED claims
require_hash_for_verified = true

# Status transition rules (from -> allowed to)
[claim_linter.status_transitions]
OPEN = ["CONFIRMED", "REFUTED", "SUPERSEDED"]
CONFIRMED = ["SUPERSEDED"]
REFUTED = ["SUPERSEDED"]
SUPERSEDED = []

# Output
output_format = "jsonl"  # Claims_Register.md format
output_path = "{workstream}/Claims_Register.md"

# Linting rules
[claim_linter.rules]
# Disallow unlabeled claims in Design.md
disallow_unlabeled_in_design = true
# Disallow VERIFIED without hash
disallow_verified_without_hash = true
# Disallow duplicate claim IDs
disallow_duplicate_ids = true
# Require date on all claims
require_date = true
```

---

## Migration Auditor Config (TOML)

```toml
# tools/migration_auditor.toml

[migration_auditor]
# External references to track (from README.md "Read next" sections)
external_refs = [
  "AtomaApp/docs/27-tree-sitter-master-document.md",
  "AtomaApp/AtomaDB/30-rules-and-governance.md",
  "AtomaApp/AtomaDB/31-constraint-grammar-decomposition.md",
  "AtomaApp/docs/tools-design.md",
  "AtomaApp/mcp-servers/design.md",
  "AtomaApp/Harnesses-2026-10-23/audits/THREE-WAY-UNIFICATION.md"
]

# Target categories for migration
target_mapping = {
  "27-tree-sitter-master-document" = "C29",
  "30-rules-and-governance" = "C16,C23",
  "31-constraint-grammar-decomposition" = "C29",
  "tools-design" = "C29",
  "mcp-servers-design" = "C18",
  "THREE-WAY-UNIFICATION" = "C04,C33"
}

# Migration status file
status_file = "migration_status.json"

# Dry-run by default
dry_run = true

# Require human approval for write
require_approval = true
```

---

## Rollup Generator Config (TOML)

```toml
# tools/rollup_generator.toml

[rollup_generator]
# Event ledger path
ledger_path = "data/event_ledger.db"

# Output directory
output_dir = "rollups/"

# Aggregation window
window_start = "00:00:00Z"
window_end = "23:59:59Z"

# Percentiles to compute
percentiles = [50, 95, 99]

# Minimum events for valid rollup
min_events = 100

# Output file naming
file_pattern = "neural_map_{date}.json"

# Retention
retention_days = 90
```

---

## Crosslink Validator Config (TOML)

```toml
# tools/crosslink_validator.toml

[crosslink_validator]
# Root directory to scan
root = "MasterDesignDocument/"

# Reference patterns to validate
patterns = [
  "C\\d{2}-[^/]+/Design\\.md",
  "C\\d{2}-[^/]+/Rules\\.md",
  "C\\d{2}-[^/]+/TelemetryAndDiagnostics\\.md",
  "C\\d{2}-[^/]+/Agents\\.md",
  "C\\d{2}-[^/]+/Modelfiles\\.md",
  "PreRegistrations/.*\\.md",
  "VerificationCommands\\.md"
]

# Allowed external references (none for ship artifacts)
allowed_external = []

# Fail on broken internal refs
fail_on_broken = true
```

---

## Verification Gates (C15)

| Gate               | Test                                    | Threshold |
| ------------------ | --------------------------------------- | --------- |
| Claim schema       | All Claims_Register.md entries validate | 100%      |
| Migration schema   | migration_status.json validates         | 100%      |
| Five-file manifest | All workstreams have valid manifest     | 100%      |
| Linter config      | claim_linter.toml validates             | 100%      |
| Migration config   | migration_auditor.toml validates        | 100%      |
| Rollup config      | rollup_generator.toml validates         | 100%      |
| Crosslink config   | crosslink_validator.toml validates      | 100%      |
