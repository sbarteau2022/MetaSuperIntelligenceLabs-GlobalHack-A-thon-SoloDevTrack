# C15 — Verification Gates

**Status.** PROPOSED gates (to be made executable). **Parent:** Master Design Document index.

## Purpose

Concrete pass/fail gates for the sprint and release.

## Subcategories (broad topics)

- C15.1 Backend `cargo test --locked` (0 failures)
- C15.2 `scripts/main_ui_e2e.py` (PASS)
- C15.3 `scripts/compliance_smoke.sh` (0 FAIL)
- C15.4 Browser E2E (parallel-agent owned)
- C15.5 No metric without workload/hardware/baseline
- C15.6 Security/compliance checklist signed before Day 1 feature code
- C15.7 Build receipt with measured throughput + smoke results
- C15.8 Bench/pressure-test harness green

## Open questions / gaps

- Convert each gate into CI command with artifact output.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 7. Verification gates

- `cargo test --locked` (backend) — 0 failures.
- `scripts/main_ui_e2e.py` — PASS.
- `scripts/compliance_smoke.sh` — 0 FAIL.
- Browser E2E (owned by parallel agent).
- No metric claimed without workload/hardware/baseline.
- Security/compliance checklist signed before Day 1 feature code.

---

## Executable Gate Definitions

### Gate Schema (Machine-Readable)

```yaml
# .github/gates.yml
gates:
  - id: C15.1
    name: "Backend Tests"
    command: "cargo test --locked --workspace"
    artifacts:
      - path: "target/test-results.xml"
        type: "junit"
    threshold: "failures == 0"
    timeout: 600
    required: true

  - id: C15.2
    name: "UI E2E Tests"
    command: "python scripts/main_ui_e2e.py --headless"
    artifacts:
      - path: "ui_e2e_results.xml"
        type: "junit"
      - path: "ui_e2e_screenshots/"
        type: "directory"
    threshold: "status == PASS"
    timeout: 300
    required: true

  - id: C15.3
    name: "Compliance Smoke"
    command: "bash scripts/compliance_smoke.sh"
    artifacts:
      - path: "compliance_report.json"
        type: "json"
    threshold: "fail_count == 0"
    timeout: 120
    required: true

  - id: C15.4
    name: "Browser E2E"
    command: "python scripts/browser_e2e.py --parallel"
    artifacts:
      - path: "browser_e2e_results.xml"
        type: "junit"
      - path: "browser_e2e_trace.zip"
        type: "playwright_trace"
    threshold: "status == PASS"
    timeout: 600
    required: true
    owner: "parallel-agent"

  - id: C15.5
    name: "Metric Evidence"
    command: "python scripts/verify_metric_evidence.py"
    artifacts:
      - path: "CLAIMS_REGISTER.md"
        type: "markdown"
    threshold: "all_claims_have_evidence == true"
    timeout: 60
    required: true

  - id: C15.6
    name: "Security/Compliance Checklist"
    command: "bash scripts/security_checklist.sh"
    artifacts:
      - path: "security_checklist_signed.pdf"
        type: "pdf"
      - path: "threat_model.md"
        type: "markdown"
      - path: "role_matrix.csv"
        type: "csv"
    threshold: "all_signed == true"
    timeout: 60
    required: true
    day: 1

  - id: C15.7
    name: "Build Receipt"
    command: "./scripts/generate_build_receipt.sh"
    artifacts:
      - path: "BUILD-RECEIPT.json"
        type: "json"
      - path: "BUILD-RECEIPT.sig"
        type: "signature"
    threshold: "receipt_valid == true AND measurements_present == true"
    timeout: 60
    required: true

  - id: C15.8
    name: "Bench/Pressure Test"
    command: "cargo run --release --bin pressure_test -- --duration 300"
    artifacts:
      - path: "pressure_test_results.json"
        type: "json"
    threshold: "oom_free == true AND throughput_stable == true"
    timeout: 400
    required: true
```

### Gate Dependency Graph

```
C15.6 (Security) ──► C15.1 (Backend Tests)
                         │
                         ▼
C15.5 (Metric Evidence) ◄── C15.2 (UI E2E)
                         │
                         ▼
C15.3 (Compliance) ◄───── C15.8 (Pressure Test)
                         │
                         ▼
C15.7 (Build Receipt) ◄── C15.4 (Browser E2E) [parallel]
```

**Execution order**: C15.6 → C15.1 → (C15.2, C15.8) → C15.3 → C15.5 → C15.7 → C15.4 (can run in parallel with C15.2/C15.8)

### CI Integration

```yaml
# .github/workflows/verification_gates.yml
name: Verification Gates
on: [push, pull_request, workflow_dispatch]

jobs:
  gate-C15.6-security:
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: bash scripts/security_checklist.sh
      - uses: actions/upload-artifact@v4
        with:
          name: security-checklist
          path: security_checklist_signed.pdf

  gate-C15.1-backend:
    needs: gate-C15.6-security
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --locked --workspace
      - uses: actions/upload-artifact@v4
        with:
          name: backend-test-results
          path: target/test-results.xml

  gate-C15.2-ui-e2e:
    needs: gate-C15.1-backend
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: python scripts/main_ui_e2e.py --headless
      - uses: actions/upload-artifact@v4
        with:
          name: ui-e2e-results
          path: ui_e2e_results.xml

  gate-C15.8-pressure:
    needs: gate-C15.1-backend
    runs-on: [self-hosted, macos, m3-max]
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - run: cargo run --release --bin pressure_test -- --duration 300
      - uses: actions/upload-artifact@v4
        with:
          name: pressure-test-results
          path: pressure_test_results.json

  gate-C15.3-compliance:
    needs: [gate-C15.2-ui-e2e, gate-C15.8-pressure]
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: bash scripts/compliance_smoke.sh
      - uses: actions/upload-artifact@v4
        with:
          name: compliance-report
          path: compliance_report.json

  gate-C15.5-metrics:
    needs: gate-C15.3-compliance
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: python scripts/verify_metric_evidence.py
      - uses: actions/upload-artifact@v4
        with:
          name: claims-register
          path: CLAIMS_REGISTER.md

  gate-C15.7-build-receipt:
    needs: gate-C15.5-metrics
    runs-on: [self-hosted, macos, m3-max]
    steps:
      - uses: actions/checkout@v4
      - run: ./scripts/generate_build_receipt.sh
      - uses: actions/upload-artifact@v4
        with:
          name: build-receipt
          path: |
            BUILD-RECEIPT.json
            BUILD-RECEIPT.sig

  gate-C15.4-browser-e2e:
    needs: gate-C15.1-backend
    runs-on: [self-hosted, macos, m3-max]
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - run: python scripts/browser_e2e.py --parallel
      - uses: actions/upload-artifact@v4
        with:
          name: browser-e2e-results
          path: |
            browser_e2e_results.xml
            browser_e2e_trace.zip
```

### Gate Artifact Verification

```python
# scripts/verify_gate_artifacts.py
def verify_gate(gate_id: str, artifacts: dict) -> GateResult:
    """Verify gate artifacts meet threshold."""

    verifiers = {
        "C15.1": lambda a: parse_junit(a["target/test-results.xml"]).failures == 0,
        "C15.2": lambda a: parse_junit(a["ui_e2e_results.xml"]).status == "PASS",
        "C15.3": lambda a: json.load(a["compliance_report.json"])["fail_count"] == 0,
        "C15.4": lambda a: parse_junit(a["browser_e2e_results.xml"]).status == "PASS",
        "C15.5": lambda a: verify_claims_register(a["CLAIMS_REGISTER.md"]),
        "C15.6": lambda a: verify_signatures(a["security_checklist_signed.pdf"]),
        "C15.7": lambda a: verify_build_receipt(a["BUILD-RECEIPT.json"], a["BUILD-RECEIPT.sig"]),
        "C15.8": lambda a: verify_pressure_results(a["pressure_test_results.json"]),
    }

    if gate_id not in verifiers:
        return GateResult(SKIPPED, "No verifier")

    try:
        passed = verifiers[gate_id](artifacts)
        return GateResult(PASSED if passed else FAILED, "")
    except Exception as e:
        return GateResult(ERROR, str(e))
```

### Gate Status Dashboard

| Gate  | Status | Last Run | Duration | Artifacts                 |
| ----- | ------ | -------- | -------- | ------------------------- |
| C15.1 | ⏳     | —        | —        | JUnit XML                 |
| C15.2 | ⏳     | —        | —        | JUnit XML + screenshots   |
| C15.3 | ⏳     | —        | —        | JSON report               |
| C15.4 | ⏳     | —        | —        | JUnit + Playwright trace  |
| C15.5 | ⏳     | —        | —        | CLAIMS_REGISTER.md        |
| C15.6 | ⏳     | —        | —        | Signed PDF + threat model |
| C15.7 | ⏳     | —        | —        | JSON + Ed25519 sig        |
| C15.8 | ⏳     | —        | —        | JSON results              |

---

## Contributor Deepening — Muse (2026-10-26)

### C15.9 Gate-numbering collision (depth: two schemes, one workspace)

There are TWO gate numbering schemes in this workspace and they collide [VERIFIED — both read directly]:

- **Scheme A (C15):** C15.1–C15.8, artifact gates (backend tests → build receipt). Defined in this file's Executable Gate Definitions.
- **Scheme B (README + VerificationCommands.md):** Gate 1–8, phase gates (design review → build receipt) mapped to sprint days (README lines 122–136; VerificationCommands.md lines 172–201, `cargo xtask gate1…gate8`).

Cross-references are currently ambiguous: C08's traceability matrix cites "C15.1-2" (Scheme A) while the README maps "Gate 1" to "Design Review + Static Analysis" (Scheme B); C08.7 already flagged rows citing ablation IDs where gate IDs belong. Frozen disambiguation [PROPOSED — naming is my proposal; both schemes are frozen content]:

- Scheme A keeps the name **C15.x artifact gates** (what runs).
- Scheme B is renamed **Day gates G1–G8** (when it runs) at the README/VerificationCommands pass.
- Every C08 matrix row and every C32 day plan must cite BOTH: the Day gate (schedule) and the C15.x artifact gate (evidence). A row citing only one is an orphan (C08.7).
- The mapping table lives HERE (single source of truth), not in the README:

| Day gate | Sprint days (README) | C15.x artifact gates consumed                         | Evidence artifact                |
| -------- | -------------------- | ----------------------------------------------------- | -------------------------------- |
| G1       | 1–2, 3, 6–7, 10      | C15.6 (security sign-off), C15.1 (static/test subset) | signed checklist, lint reports   |
| G2       | 3, 10                | C15.1 (full backend suite)                            | JUnit XML, 0 failures            |
| G3       | 3, 4–5, 8–9, 10      | C15.1 + C15.2 (unit + integration)                    | JUnit + UI E2E PASS              |
| G4       | 4–5, 6–7, 10         | C15.2 + C15.4 (integration + contract)                | E2E + browser traces             |
| G5       | 6–7, 10              | C15.5 (contract + property)                           | CLAIMS_REGISTER evidence check   |
| G6       | 6–7, 10              | C15.8 (property + chaos/recovery)                     | pressure results, drill logs     |
| G7       | 8–9, 10              | C15.8 (chaos/recovery + perf)                         | perf numbers + kill-switch drill |
| G8       | 8–9, 10              | C15.7 (perf + build receipt)                          | BUILD-RECEIPT.json + sig         |

[INFERRED — mapping derived from README day ranges + C15 gate purposes; ends at those files. Requires team ratification: if the README day ranges shift, this table is stale and needs a C36 correction.]

### C15.10 Gate skip protocol (depth: what happens when a gate cannot run)

Gates will fail to _run_ (hardware down, harness missing — e.g. C13 toroidal harness TBD). Frozen skip rule [PROPOSED — rule is my proposal; append-only truth C03.3 is the authority]:

1. Gates are never silently skipped. A non-run gate is recorded as `SKIPPED(reason, owner, remediation_date)` in the gate dashboard + a C36 correction entry.
2. Dependent gates (per the dependency graph above) cannot PASS while an upstream gate is SKIPPED — they inherit SKIPPED. No leapfrogging.
3. Day-10 go/no-go with any REQUIRED gate SKIPPED = no-go, same as FAIL (ADR-21). The only exception is a gate whose requirement row was WITHDRAWN via C08.6 lifecycle + C36 correction before Day 10.

What was NOT checked here: C32 day assignments vs the G1–G8 ranges above (verify at C32); whether `cargo xtask gateN` commands exist as code (no xtask tree in workspace — confirm before P1).

## Open questions / gaps

- Convert each gate into CI command with artifact output.
