# C17 — Data Model, Persistence & Migrations

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Canonical data stores, schemas, migrations, and append-only integrity rules.

## Subcategories (broad topics)

- C17.1 Stores: SQLite event ledger, mind.jsonl, RAG collections, workflows/packets, skills, configs
- C17.2 Schemas for each store (fields, indexes, constraints)
- C17.3 Append-only integrity (reject update/delete; corrections appended)
- C17.4 Migration strategy (forward-only serde defaults, e.g. rag_collection)
- C17.5 Data-root resolution and portability (C14.10)
- C17.6 Backup/restore and right-to-erasure mechanics
- C17.7 Idempotency keys and restart persistence (interrupted packets → failed)

## Open questions / gaps

- No schema catalog exists; extract from current Rust structs.

## Canonical decision (frozen for design)

- **Data root:** resolve via workspace discovery plus `ATOMA_WORKSPACE_ROOT` override; never bake user-specific paths (NF4, C14.10).
- All stores are rooted at the resolved data root; migrations are forward-only with serde defaults.

## Remaining gaps after frozen decisions

- Extract current Rust struct schemas into a store catalog (append).
- Add backup/restore and erasure mechanics tests.

## Contract catalog (frozen for design)

| Store                  | Content                      | Integrity                         | Notes                      |
| ---------------------- | ---------------------------- | --------------------------------- | -------------------------- |
| `events.sqlite`        | append-only event ledger     | reject update/delete              | monotonic clock            |
| `mind.jsonl`           | structured session summaries | append-only                       | canonical home             |
| RAG collections (7)    | docs/text                    | ingest receipt                    | BM25+dense when configured |
| `workflows.db`/packets | task graph                   | append/correction records         | judges, retry/escalation   |
| `skills/` registry     | skill tree                   | idempotent scan                   | depends_on topo sort       |
| `configurations.json`  | lane/config state            | forward-only migration            | serde defaults             |
| `.atoma_map` + `.wal`  | geometry/memory map          | atomic snapshot + WAL tail repair | C35.9                      |

## Formal Schema Definitions

### events.sqlite (Append-Only Event Ledger)

```sql
-- Schema version: 1
CREATE TABLE events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    clock        INTEGER NOT NULL,           -- Monotonic logical clock
    event_type   TEXT NOT NULL,              -- Enum: tool_call, lane_turn, judge, memory_event, ...
    agent_id     TEXT NOT NULL,              -- Agent identifier
    lane_id      TEXT NOT NULL,              -- Lane identifier
    run_id       TEXT NOT NULL,              -- Run identifier
    project_id   TEXT NOT NULL DEFAULT '',   -- Project scope
    payload_hash TEXT NOT NULL,              -- SHA-256 of payload (privacy)
    receipt_hash TEXT NOT NULL,              -- Merkle receipt
    prev_hash    TEXT NOT NULL,              -- Hash chain: hash(prev_event)
    timestamp    INTEGER NOT NULL,           -- Unix ms
    CONSTRAINT monotonic_clock CHECK (clock > (SELECT MAX(clock) FROM events))
);

CREATE INDEX idx_events_lane_clock ON events(lane_id, clock);
CREATE INDEX idx_events_agent_clock ON events(agent_id, clock);
CREATE INDEX idx_events_project_clock ON events(project_id, clock);
CREATE INDEX idx_events_type_clock ON events(event_type, clock);
```

**Integrity**: `clock` is globally monotonic; `prev_hash` forms hash chain; no UPDATE/DELETE allowed.

### mind.jsonl (Session Summaries)

```json
{
  "schema_version": 1,
  "entry": {
    "id": "mind_abc123",
    "clock": 42,
    "lane": "researcher",
    "run_id": "run_20261026_001",
    "project": "default",
    "sections": {
      "brainstorm": "Considered X, Y, Z...",
      "attempts": "Tried A (failed: timeout), B (success)...",
      "plan": "Predicted outcome: 90% pass rate. Falsifier: <80%.",
      "executed": "Ran A, B. Receipts: rec_1, rec_2. Verdicts: pass, pass.",
      "conclusions": "Invariant: tool loop requires approval. Decision: enforce.",
      "coming_plan": "Next: integrate C. Handoff to coder lane."
    },
    "receipt_hash": "sha256:...",
    "kind": "session_summary"
  }
}
```

**Append-only**: File opened with `O_APPEND`; each line is valid JSON; no modification after write.

### RAG Collections (7 Collections)

```rust
enum RagCollection {
    Memory,          // Agent summaries, memories
    Research,        // Papers, technical docs
    Knowledge,       // Curated knowledge base
    Code,            // Code snippets, patterns
    Config,          // Configuration examples
    Logs,            // Historical logs, debug info
    External,        // Imported from other tools
}
```

Each collection: `documents/` (text files) + `index.bm25` + optional `index.dense.faiss`

**Document schema**:

```json
{
  "id": "doc_xyz789",
  "collection": "research",
  "title": "Transformer are RNNs",
  "content": "...",
  "metadata": {
    "source": "arxiv:2006.16236",
    "ingested_by": "operator",
    "ingest_receipt": "rec_ingest_456",
    "tags": ["attention", "linear", "rnn"]
  },
  "embedding": null // or [float; 384] when dense enabled
}
```

### workflows.db (Packets + Judges)

```sql
CREATE TABLE packets (
    id           TEXT PRIMARY KEY,
    clock        INTEGER NOT NULL,
    name         TEXT NOT NULL,
    spec         TEXT NOT NULL,              -- JSON: depends_on, retry, escalation, prereg
    status       TEXT NOT NULL,              -- pending, running, done, failed, escalated
    assigned_lane TEXT,
    judge_id     TEXT REFERENCES judges(id),
    result       TEXT,                       -- JSON result
    receipt_hash TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE judges (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    spec         TEXT NOT NULL,              -- JSON: criteria, threshold, held_out
    type         TEXT NOT NULL               -- deterministic, llm, human
);
```

### .atoma_map + .wal (Geometry/Memory Map)

**Snapshot format** (binary, versioned):

```
MAGIC: "ATOM\0" (4 bytes)
VERSION: u32 (1)
ATOM_COUNT: u64
VP_TREE_SIZE: u64
ATOMS: [Atom; ATOM_COUNT]
VP_TREE: [VpNode; VP_TREE_SIZE]
CHECKSUM: SHA-256 of above
```

**Atom struct** (Q32.31 fixed-point):

```rust
struct Atom {
    id: u64,
    clock: u64,
    phase: Phase { theta_f: i64, theta_s: i64 },  // Q32.31
    lorentz: LorentzVector { x: [i64; 4] },       // Q32.31 (x0, x1, x2, x3)
    payload_hash: [u8; 32],                        // SHA-256
    pinned: bool,
}
```

**WAL format** (append-only):

```
ENTRY: [clock: u64] [op: u8] [atom_id: u64] [timestamp: u64] [payload_hash: [u8; 32]]
op: 0=Write, 1=Evict
```

### Migration Strategy (Forward-Only)

```rust
trait Migration {
    fn version(&self) -> u32;
    fn up(&self, store: &mut dyn Store) -> Result<()>;
    // NO down() — forward only
}

// Example: Add rag_collection field to AgentConfig
struct AddRagCollectionMigration;
impl Migration for AddRagCollectionMigration {
    fn version(&self) -> u32 { 2 }
    fn up(&self, store: &mut dyn Store) -> Result<()> {
        let mut configs = store.load_configs()?;
        for config in &mut configs {
            if config.rag_collection.is_none() {
                config.rag_collection = Some("default".to_string());
            }
        }
        store.save_configs(configs)
    }
}
```

**Migration runner** (idempotent, atomic):

```rust
fn run_migrations(store: &mut dyn Store) -> Result<()> {
    let current = store.get_schema_version()?;
    let migrations = get_all_migrations();  // Sorted by version

    for m in migrations {
        if m.version() > current {
            store.begin_transaction()?;
            m.up(store)?;
            store.set_schema_version(m.version())?;
            store.commit_transaction()?;
        }
    }
    Ok(())
}
```

### Backup/Restore Protocol

```bash
#!/bin/bash
# backup.sh — atomic, verified

DATA_ROOT="${ATOMA_WORKSPACE_ROOT:-$(discover_workspace_root)}"
BACKUP_DIR="${DATA_ROOT}/backups/$(date +%Y%m%d_%H%M%S)"

mkdir -p "${BACKUP_DIR}"

# 1. Snapshot each store (atomic)
cp "${DATA_ROOT}/events.sqlite" "${BACKUP_DIR}/"
cp "${DATA_ROOT}/mind.jsonl" "${BACKUP_DIR}/"
cp -r "${DATA_ROOT}/rag_collections" "${BACKUP_DIR}/"
cp "${DATA_ROOT}/workflows.db" "${BACKUP_DIR}/"
cp "${DATA_ROOT}/configurations.json" "${BACKUP_DIR}/"
cp "${DATA_ROOT}/.atoma_map" "${BACKUP_DIR}/"
cp "${DATA_ROOT}/.atoma_map.wal" "${BACKUP_DIR}/"

# 2. Verify each copy
for f in "${BACKUP_DIR}"/*; do
    if ! verify_store_integrity "$f"; then
        echo "FAIL: $f integrity check failed"
        exit 1
    fi
done

# 3. Manifest
cat > "${BACKUP_DIR}/MANIFEST.json" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "data_root": "${DATA_ROOT}",
  "stores": $(list_stores_with_hashes),
  "schema_version": $(get_schema_version)
}
EOF

echo "Backup complete: ${BACKUP_DIR}"
```

### Idempotency Keys

Every mutating operation carries an **idempotency key**:

```rust
struct IdempotencyKey {
    operation: OperationType,  // ToolCall, HoleCreate, WorkflowSubmit, etc.
    agent_id: AgentId,
    run_id: RunId,
    sequence: u64,             // Per-agent monotonic
    hash: Hash,                // Hash of operation parameters
}
```

**Replay on restart**: Operations with same key are deduplicated; interrupted packets marked `failed`.

---

## Contributor Deepening — Muse (2026-10-26)

### C17.8 Erasure mechanics per store (depth: implementing the C16.9 tombstone protocol)

C16.9 defines tombstone-append erasure; this section specifies per-store mechanics [PROPOSED — mechanics are my proposal; the tombstone protocol is the authority]:

| Store               | Erasure operation                                                              | Hash-chain preservation                                        | Verification                                                        |
| ------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------- |
| events.sqlite       | FORBIDDEN (no DELETE) — tombstone row only                                     | Chain intact by construction                                   | C15.1 property test: chain verifies after tombstone                 |
| mind.jsonl          | Append redaction record (kind = redaction, redacts_clock = N); tail suppresses | Original line untouched; redaction record hash-chains normally | tail-after-redaction returns no redacted content (integration test) |
| RAG collections     | Delete document + index entry; append ingest-receipt revocation                | Revocation receipt chains to original ingest receipt           | Query-after-erase returns no erased doc (A8 round-trip variant)     |
| workflows.db        | Mark packet failed/superseded; never DELETE rows                               | Status transition is an append in packet history               | Restart test: erased-subject packets stay failed                    |
| configurations.json | Remove subject data via forward migration (new version)                        | Old version retained in migration history                      | Migration test covers erasure version                               |
| .atoma_map + .wal   | Evict atoms by subject tag; WAL records Evict ops                              | Eviction is a normal WAL op (op = 1)                           | VP-tree query returns no erased atoms                               |
| backups             | Destroy whole backup sets past retention bound                                 | Manifest records destruction receipt                           | Retention audit (manual at P5)                                      |

Residual risk (stated, not hidden): between erasure request and roll-up compaction, redacted mind.jsonl bytes persist on disk. Mitigation is encryption-at-rest + bounded roll-up interval (from C07.7 budget). If the threat model (C16.1) rates this unacceptable, the alternative is immediate compaction-on-erasure (cost: O(log N) rewrite of the tail) — a C36 decision, not an implementation detail.

### C17.9 Schema-version registry (depth: migrations need a map)

Forward-only migrations (C17.4) without a version registry drift. Frozen registry rule [PROPOSED — rule is my proposal]:

- A single SCHEMA_VERSIONS table (or versions.json at data root) records (store, version, migration_hash, applied_at_clock) for every store. `run_migrations` refuses to run if the registry is missing or divergent — fail closed, not open.
- Erasure migrations (C17.8 configurations.json row) are versioned like any other: version N+1 removes the field, serde default keeps old binaries reading.
- The registry is itself append-only (new rows per migration, never UPDATE) — migrations of the migration log stay auditable.

What was NOT checked here: current Rust struct field lists (still TBD per above); whether versions.json exists anywhere (not in workspace).

## Open questions / gaps

- Extract current Rust struct schemas into a store catalog (append).
- Add backup/restore and erasure mechanics tests.
