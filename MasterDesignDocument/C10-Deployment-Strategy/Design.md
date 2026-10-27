# C10 — Deployment Strategy & Release Discipline

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Phase-gated rollout from local prototype to packaged multi-endpoint deployment.

## Subcategories (broad topics)

- C10.1 Phases P0–P6 (now → post-sprint enterprise)
- C10.2 Release discipline (desktop+backend together, build receipt, secrets by slot, endpoints disabled by default, importer not auto-start)
- C10.3 Staged rollout + rollback (feature flags per phase, shadow-mode telemetry, documented rollback)
- C10.4 Kill switch to standard transformer (one config flag, no data loss)
- C10.5 Reproducible release (pinned toolchain, exact git commit, dependency lock hashes)
- C10.6 Multi-tenant / org policy layer (post-sprint)
- C10.7 Federated Atlas sync (edge/A2A) (post-sprint)

## Open questions / gaps

- Packaging (Tauri bundle) and multi-endpoint deployment post-sprint.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 13. Deployment Strategy (PROPOSED)

### 13.1 Phases

- **Phase 0 (now):** local prototype (current Ares app) with measured gaps closed.
- **Phase 1 (sprint D1–3):** freeze design + close internal scaffolds (hooks, skill tree, lazy MCP,
  compaction, checkpoints) on the existing Rust backend; frontend build pipeline documented.
- **Phase 2 (D4–5):** RAG+memory+ingest for all lanes; agent tool loop with approval; streaming;
  smoke gates green.
- **Phase 3 (D6–7):** browser E2E green (parallel agent); docs + claims register; no proposed
  leftovers that are single-file implementable.
- **Phase 4 (D8–9):** model serving: Atlas/Hyperbolic cache + TwisterAttention in rust-server;
  first measured throughput numbers.
- **Phase 5 (D10):** compliance/security audit; right-to-erasure plan; structured audit logs;
  go/no-go review.
- **Phase 6 (post-sprint, enterprise):** packaging (Tauri bundle), multi-endpoint deployment,
  federated Atlas sync (edge/A2A), org policy layer, multi-tenant isolation.

### 13.2 Release discipline

- Desktop app and backend ship together (baked assets); rebuild chain: `npm run build` then
  `cargo build --release`; one script drives both.
- Every release gets a build receipt (like `BUILD-RECEIPT.md`) with measured throughput and
  smoke results, not just "built".
- Secrets by slot; endpoints disabled by default; importer never auto-starts.
- Migration: configurations.json migrations are forward-only with serde defaults (as done for
  `rag_collection`).

## Formal Deployment Calculus

### Deployment as State Transition System

A **deployment** is a tuple $D = (V, C, S, K, R)$ where:

- $V$ = version (git commit + semver)
- $C$ = configuration (feature flags, secrets slots, endpoints)
- $S$ = state (data root, Atlas snapshot, WAL position)
- $K$ = kill switch state ($\text{acft\_enabled} \in \{\top, \bot\}$)
- $R$ = rollback point (previous deployment $D_{prev}$ or $\bot$)

**Transition**: $D_{old} \xrightarrow{\text{deploy}} D_{new}$ where:

- $V_{new} \succ V_{old}$ (git history)
- $C_{new}$ migrates from $C_{old}$ via forward-only migration
- $S_{new} = \text{Migrate}(S_{old}, C_{old} \to C_{new})$
- $K_{new}$ defaults to $\bot$ (safe default)
- $R_{new} = D_{old}$ (automatic rollback point)

### Phase Gates as Preconditions

| Phase | Precondition              | Postcondition                      | Rollback Trigger     |
| ----- | ------------------------- | ---------------------------------- | -------------------- |
| P1    | Design frozen, C03 signed | Scaffolds green (C15.1-2)          | Any scaffold fail    |
| P2    | P1 postcondition          | RAG+memory+ingest green (A8, A9)   | A8/A9 fail           |
| P3    | P2 postcondition          | Browser E2E green, claims register | E2E fail, claims gap |
| P4    | P3 postcondition          | Atlas/ACFT measured (A1)           | A1 fail              |
| P5    | P4 postcondition          | Compliance/security audit pass     | Audit fail           |
| P6    | P5 postcondition          | Tauri bundle, multi-endpoint       | Packaging fail       |

### Kill Switch as Safety Invariant

The **kill switch** is a **safety invariant** that must hold at all times:

$$\text{KillSwitchInvariant} \equiv \Box \left( \text{acft\_enabled} = \bot \implies \text{StandardTransformerActive} \right)$$

**Properties**:

- **Atomic flip**: `acct_enabled` toggle is a single atomic config write
- **No data loss**: Atlas state $S$ is compatible with both paths (Lorentz geometry shared)
- **Instant fallback**: $< 100\text{ms}$ switch time (no model reload)
- **Observable**: Telemetry emits `kill_switch_toggled` event with reason

### Staged Rollout as Probabilistic Deployment

For each phase $p$, deployment follows **canary → shadow → full**:

```yaml
# deployment policy per phase
phase_p:
  canary:
    traffic: 5%
    duration: 2h
    metrics: [error_rate, latency_p99, memory_gb]
    threshold: error_rate < 0.01 AND latency_p99 < 2x_baseline
  shadow:
    traffic: 100% (mirrored, not served)
    duration: 24h
    metrics: [tok_s, ttft_ms, recall_accuracy]
    threshold: tok_s >= 0.9 * baseline AND recall >= baseline
  full:
    traffic: 100%
    rollback_window: 48h
```

### Rollback as Deterministic Reversion

**Rollback procedure** (guaranteed $< 5$ minutes):

```bash
#!/bin/bash
# rollback.sh — deterministic, tested in CI

set -euo pipefail

PREV_DEPLOYMENT=$1  # git commit hash from build receipt
DATA_ROOT=${ATOMA_WORKSPACE_ROOT:-$(discover_workspace_root)}

# 1. Stop current services
systemctl stop atoma-backend atoma-frontend

# 2. Restore binary
cp artifacts/${PREV_DEPLOYMENT}/atoma-backend /usr/local/bin/
cp artifacts/${PREV_DEPLOYMENT}/atoma-frontend /usr/local/bin/

# 3. Restore config (forward-only migration is invertible by design)
#    Migrations are pure functions: Config_v{n} -> Config_v{n+1}
#    We store inverse functions in migration registry
python scripts/migrate_config.py --to-version $(git show ${PREV_DEPLOYMENT}:Cargo.toml | grep version)

# 4. Data root compatibility check
#    Atlas format is versioned; auto-migrate if needed
atoma-backend --check-data-root ${DATA_ROOT} --target-version ${PREV_DEPLOYMENT}

# 5. Start services
systemctl start atoma-backend atoma-frontend

# 6. Smoke test
sleep 10
curl -f http://localhost:8088/health || exit 1

echo "Rollback to ${PREV_DEPLOYMENT} complete"
```

### Build Receipt as Signed Artifact

```json
{
  "build_id": "build-20261026-abc123",
  "git_commit": "a1b2c3d4e5f6...",
  "git_tag": "v0.1.0-rc.3",
  "toolchain": {
    "rustc": "1.82.0 (stable)",
    "cargo": "1.82.0",
    "node": "20.17.0",
    "npm": "10.8.2"
  },
  "dependencies": {
    "Cargo.lock": "sha256:...",
    "package-lock.json": "sha256:..."
  },
  "artifacts": {
    "backend": "sha256:...",
    "frontend": "sha256:...",
    "assets": "sha256:..."
  },
  "measurements": {
    "throughput_tok_s": 64.7,
    "ttft_ms": 42,
    "peak_memory_gb": 27.4,
    "smoke_tests": "PASS"
  },
  "signatures": {
    "builder": "ed25519:...",
    "timestamp": "2026-10-26T14:30:00Z"
  }
}
```

### Reproducible Build Verification

```bash
# verify_build.sh — bit-for-bit reproducibility
git checkout ${BUILD_COMMIT}
./scripts/build_all.sh
diff -r artifacts/${BUILD_COMMIT}/ artifacts/verified/
# Exit code 0 = bit-for-bit identical
```

---

## Contributor Deepening — Muse (2026-10-26)

### C10.8 Kill-switch drill protocol (depth: a safety invariant that is never tested is a hope)

C10.4 defines the kill-switch invariant (atomic flip, no data loss, <100ms, observable) but no drill schedule. Frozen drill rule [PROPOSED — schedule is my proposal; the invariant it tests is frozen]:

1. Every phase gate (P1–P5) includes a live kill-switch drill: flip acft_enabled to false under load, assert (a) switch time < 100ms, (b) zero dropped receipts, (c) Atlas state readable by the standard path, (d) kill_switch_toggled event in the ledger. Drill result is a Gate 7 (chaos/recovery) artifact.
2. A failed drill blocks phase exit exactly like a failed ablation — no "we will drill later" waivers, because P4 (Atlas/ACFT measured) is the phase that makes the switch meaningful and P5 (audit) must witness a passing drill.
3. The rollback script above is executed — not reviewed — at least once before P5 on a scratch data root. An unexecuted rollback script is PROPOSED text, not a procedure.

### C10.9 Phase-exit evidence checklist (depth: what each P-gate consumes and produces)

| Phase exit | Consumes (entry evidence)                               | Produces (exit evidence)                                                                       | Judge             |
| ---------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------- |
| P1         | Design frozen (C36 sign-off), C03.7 threat model signed | Scaffolds green, C09.9 preregs filed for A1–A10                                                | Gates 1–2         |
| P2         | P1 exit                                                 | A8/A9 PASS (primary), memory capacity counters live (C07.7)                                    | Gate 3            |
| P3         | P2 exit                                                 | Browser E2E green, CLAIMS_REGISTER.md started, no PROPOSED leftovers single-file implementable | Gates 3–4         |
| P4         | P3 exit                                                 | A1 PASS + replication underway, kill-switch drill 1 PASS                                       | Gates 7–8         |
| P5         | P4 exit                                                 | Audit pass, erasure plan, drill 2 PASS on scratch root, all-gates green                        | Gate 8 + go/no-go |

Skipped evidence is recorded as a C36 correction with a named owner and dated remediation — never silently deferred.

What was NOT checked here: C32 day-to-phase mapping consistency (verify at C32); whether rollback.sh/verify_build.sh exist as files (workspace has no scripts/ tree — confirm before P1).

## Open questions / gaps

- Packaging (Tauri bundle) and multi-endpoint deployment post-sprint.
