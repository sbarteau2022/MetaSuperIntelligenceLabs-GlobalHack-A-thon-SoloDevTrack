# Verification Commands (design-level)

- Backend tests: `cargo test --locked` (0 failures).
- UI E2E: `scripts/main_ui_e2e.py` (PASS).
- Compliance smoke: `scripts/compliance_smoke.sh` (0 FAIL).
- Browser E2E: parallel-agent owned.
- Bench/pressure: existing docs 18/19 harness; cross-hardware replication required (C09.4).
- Build receipt: record git commit, toolchain, lock hashes, measured throughput, smoke results.

## Cross-cutting checks

- `grep`/scan for literal secrets and baked user paths.
- Determinism replay for receipts and resolve v2.
- Property tests: hole/merge/Merkle, compaction restart, ingest round-trip.
- Toroidal worst-arc reporting (not average only).

## Category-Specific Verification Commands

### C28 Skill Registry & Hooks

```bash
# Manifest validation
cargo xtask skills-manifest --check

# DAG acyclicity
cargo xtask skills-dag --verify

# Hook veto test
cargo xtask hooks-test --moments=23

# Brigade projection audit
curl localhost:8088/api/lanes/{id}/brigade/audit
```

### C29 Tool Registry & Code Intel

```bash
# Manifest validation
cargo xtask tools-manifest --check

# Effect monotonicity
cargo xtask tools-effects --verify

# Schema validation
cargo xtask tools-schema --validate

# LSP server stress
cargo xtask lsp-stress --concurrent=50 --max-servers=4
```

### C30 Multi-Agent Lanes & Autorunner

```bash
# Lane config migration test
cargo xtask lanes-migrate --from=1 --to=3

# Autorunner admission test
cargo xtask autorunner-test --packets=10000

# Subagent depth/budget caps
cargo xtask subagent-caps --depth=3 --budget-tokens=50000

# Branch isolation test
cargo xtask branch-isolation --lanes=4

# Interference matrix
curl localhost:8088/api/lanes/interference

# Brigade weight copies
curl localhost:8088/api/brigade/audit | jq '.weight_copies'
```

### C31 Observability & Telemetry

```bash
# Ledger verification
cargo xtask ledger-verify --db=data/event_ledger.db

# Telemetry health
curl localhost:8088/api/telemetry/health

# Privacy schema validation
cargo xtask telemetry-privacy --fuzz=10000

# Metrics cardinality
curl localhost:8088/api/telemetry/metrics/cardinality

# Rollup generation
cargo xtask rollup-generate --date=2026-10-26
```

### C32 Implementation Order / Sprint

```bash
# Monte Carlo schedule simulation
cargo xtask monte-carlo --runs=10000 --tasks=sprint_tasks.json

# PERT parameter validation
cargo xtask pert-validate --file=sprint_tasks.json

# Critical path analysis
cargo xtask critical-path --file=sprint_tasks.json
```

### C33 Knowledge Base / Documentation

```bash
# Five-file set check
cargo xtask five-file-check --root=MasterDesignDocument

# Claim linting
cargo xtask claim-lint --all

# Migration audit
cargo xtask doc-audit --fix

# Crosslink validation
cargo xtask crosslink-validate --root=MasterDesignDocument
```

### C34 Agent Lanes / Personas

```bash
# Session type checking
cargo xtask session-type-check --protocols=handoffs

# Lane config validation
cargo xtask lanes-validate --config=configurations.json

# Tool allowlist enforcement
cargo xtask lanes-tools-test --lanes=4 --calls=10000

# Approval policy test
cargo xtask approval-test --effects=all --modes=all
```

### C35 Hyperbolic Geometry Interop

```bash
# Isometry test (900+ pairs)
cargo xtask geometry-isometry --pairs=10000

# Picard idempotence
cargo xtask picard-idempotent --radii="25,250,5000,50000"

# VP-tree retrieval accuracy
cargo xtask vptree-accuracy --atoms=100000 --k=10

# Fixed-point overflow
cargo xtask fixed-point-overflow --x0=65536

# Persistence atomicity
cargo xtask persistence-crash --injections=10000

# Concurrent turns
cargo xtask concurrent-turns --threads=8 --turns=1000
```

### Decision-Log-ADR Scaffold (C36 ID retained)

```bash
# ADR traceability
cargo xtask adr-traceability --root=MasterDesignDocument

# ADR-category consistency
cargo xtask adr-consistency --root=MasterDesignDocument

# Correction format validation
cargo xtask adr-corrections --validate
```

## Gate Commands (C15)

```bash
# Gate 1: Design Review + Static Analysis
cargo xtask gate1 --categories=all

# Gate 2: Static Analysis + Unit Tests
cargo xtask gate2 --categories=all

# Gate 3: Unit Tests + Integration Tests
cargo xtask gate3 --categories=all

# Gate 4: Integration Tests + Contract Tests
cargo xtask gate4 --categories=all

# Gate 5: Contract Tests + Property Tests
cargo xtask gate5 --categories=all

# Gate 6: Property Tests + Chaos/Recovery
cargo xtask gate6 --categories=all

# Gate 7: Chaos/Recovery + Performance
cargo xtask gate7 --categories=all

# Gate 8: Performance/Benchmarks + Build Receipt
cargo xtask gate8 --categories=all

# All gates (Day 10 go/no-go)
cargo xtask all-gates --categories=all
```

## Compliance Smoke Script (`scripts/compliance_smoke.sh`)

```bash
#!/bin/bash
set -euo pipefail

# 1. No literal secrets
grep -r "sk-" --include="*.rs" --include="*.toml" --include="*.json" . && exit 1

# 2. No baked user paths
grep -r "/Users/" --include="*.rs" --include="*.toml" --include="*.json" . && exit 1

# 3. Telemetry privacy schema
cargo xtask telemetry-privacy --check || exit 1

# 4. Claim labeling
cargo xtask claim-lint --check || exit 1

# 5. Five-file set
cargo xtask five-file-check --check || exit 1

# 6. Crosslink validation
cargo xtask crosslink-validate --check || exit 1

# 7. ADR traceability
cargo xtask adr-traceability --check || exit 1

echo "COMPLIANCE SMOKE: PASS"
exit 0
```

EOF
echo done
