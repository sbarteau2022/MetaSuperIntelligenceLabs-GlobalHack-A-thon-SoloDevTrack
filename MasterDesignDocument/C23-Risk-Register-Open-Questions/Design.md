# C23 — Risk Register, Assumptions & Open Questions

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Explicit risk/assumption/open-question log for the master design.

## Subcategories (broad topics)

- C23.1 Technical risks (model serving novelty, correctness fallback)
- C23.2 Schedule risks (10-day sprint, browser E2E flakiness)
- C23.3 Data/privacy risks
- C23.4 Operational risks (paths, toolchains, build chain)
- C23.5 Assumptions register
- C23.6 Open questions register with owner + target date

## Open questions / gaps

- Seed from original doc open questions (C12, C15.5).

## Quantitative Risk Register

### Risk Scoring (Modified FAIR)

$$\text{Risk} = \text{Likelihood} \times \text{Impact} \times \text{Detectability}$$

| Scale | Likelihood | Impact     | Detectability         |
| ----- | ---------- | ---------- | --------------------- |
| 1     | < 1%       | Negligible | Automatic (immediate) |
| 2     | 1-10%      | Minor      | Automated (minutes)   |
| 3     | 10-30%     | Moderate   | Manual (hours)        |
| 4     | 30-60%     | Major      | Manual (days)         |
| 5     | > 60%      | Critical   | Undetectable          |

### Risk Register

| ID  | Risk                                      | Category     | Likelihood | Impact | Detectability | Score | Owner           | Mitigation                                     | Residual |
| --- | ----------------------------------------- | ------------ | ---------- | ------ | ------------- | ----- | --------------- | ---------------------------------------------- | -------- |
| R1  | ACFT correctness < standard transformer   | Technical    | 3          | 5      | 2             | 30    | Model lead      | Kill switch, A1 ablation, shadow mode          | 12       |
| R2  | Hyperbolic cache OOM at scale             | Technical    | 2          | 5      | 2             | 20    | Backend lead    | Backpressure, eviction, memory permits         | 8        |
| R3  | A1 ablation fails (no speedup)            | Technical    | 3          | 4      | 1             | 12    | Model lead      | Prereg, fallback, early prototype              | 6        |
| R4  | Browser E2E flakiness blocks P3           | Schedule     | 4          | 3      | 3             | 36    | QA lead         | Deterministic seeds, parallel runs, quarantine | 18       |
| R5  | 10-day sprint scope creep                 | Schedule     | 4          | 4      | 2             | 32    | PM              | Frozen design, daily standup, gate reviews     | 16       |
| R6  | Prompt injection bypasses sanitizer       | Security     | 2          | 5      | 3             | 30    | Security lead   | A10 red-team, layered defense, quarantine      | 10       |
| R7  | Secret leakage in telemetry               | Security     | 1          | 5      | 2             | 10    | Security lead   | NF3, scan checks, slot resolution              | 4        |
| R8  | Data loss on compaction/restart           | Data/privacy | 2          | 5      | 2             | 20    | Backend lead    | Checkpoint, WAL, C24 verification              | 8        |
| R9  | Right-to-erasure incomplete               | Data/privacy | 2          | 4      | 3             | 24    | Compliance lead | C16.6 protocol, audit log                      | 12       |
| R10 | Toolchain drift breaks reproducible build | Operational  | 2          | 3      | 2             | 12    | Platform lead   | Pinned toolchain, C15.7 verification           | 6        |
| R11 | Path portability broken on new machine    | Operational  | 3          | 3      | 2             | 18    | Platform lead   | C14.10, CI on multiple machines                | 9        |
| R12 | Dependency vulnerability (supply chain)   | Operational  | 3          | 4      | 2             | 24    | Platform lead   | `cargo audit`, `npm audit`, pinned deps        | 12       |

### Monte Carlo Schedule Simulation

```python
# risk/schedule_monte_carlo.py
import numpy as np

# Task distributions (days,PERT: min, likely, max)
tasks = {
    "design_freeze": (1, 2, 3),
    "scaffolds": (2, 3, 5),
    "rag_memory_ingest": (2, 3, 4),
    "tool_loop_streaming": (2, 3, 5),
    "browser_e2e": (1, 2, 4),
    "model_serving": (2, 3, 5),
    "compliance_audit": (1, 1, 2),
}

def sample_task(min_d, likely, max_d):
    # Beta-PERT distribution
    return np.random.beta(1 + 4*likely/(min_d+max_d), 1 + 4*(max_d-likely)/(max_d-min_d)) * (max_d - min_d) + min_d

def simulate_sprint(n=10000):
    totals = []
    for _ in range(n):
        total = sum(sample_task(*t) for t in tasks.values())
        totals.append(total)
    return np.percentile(totals, [10, 50, 90])

print(f"P10: {simulate_sprint()[0]:.1f} days")
print(f"P50: {simulate_sprint()[1]:.1f} days")
print(f"P90: {simulate_sprint()[2]:.1f} days")
```

### Assumptions Register

| ID  | Assumption                           | Confidence | Validation         | Owner         |
| --- | ------------------------------------ | ---------- | ------------------ | ------------- |
| A1  | Rapid-MLX :8001 stable for sprint    | 0.9        | Health check in CI | Model lead    |
| A2  | Lorentz/Poincaré isometry holds      | 0.99       | Verified (4.4e-9)  | Model lead    |
| A3  | Picard reduction idempotent          | 0.95       | Tested up to r=50k | Model lead    |
| A4  | BM25 sufficient for MVP              | 0.8        | A3 ablation        | RAG lead      |
| A5  | Seatbelt sandbox available on macOS  | 0.95       | macOS 14+ verified | Platform lead |
| A6  | Single weight set supports N=8 lanes | 0.7        | A4 ablation        | Model lead    |
| A7  | Toroidal benchmarks feasible         | 0.6        | Harness prototype  | QA lead       |
| A8  | Tauri bundle works for P6            | 0.7        | Spike in P5        | Platform lead |

### Open Questions Register

| ID  | Question                                | Owner         | Target Date | Status |
| --- | --------------------------------------- | ------------- | ----------- | ------ |
| Q1  | What bundled embedding model for C14.5? | RAG lead      | D2          | OPEN   |
| Q2  | Exact VP-tree rebuild frequency?        | Model lead    | D3          | OPEN   |
| Q3  | Circuit breaker thresholds per lane?    | Backend lead  | D3          | OPEN   |
| Q4  | Browser E2E parallelization strategy?   | QA lead       | D2          | OPEN   |
| Q5  | Tauri bundle size target?               | Platform lead | P5          | OPEN   |
| Q6  | Federation protocol for C06.10?         | Model lead    | P6          | OPEN   |
| Q7  | Self-improvement mutation budget?       | Agent lead    | D7          | OPEN   |
| Q8  | Energy measurement precision on M3?     | Model lead    | D4          | OPEN   |

---

## Contributor Deepening — Muse (2026-10-26)

### C23.7 Monte Carlo structure (depth: the simulation sums what runs in parallel)

The simulate_sprint function sums all seven task distributions into a single total — but C04.10 (my DAG contribution) establishes that C28/C29/C26 build in parallel once roots stub, and the C32 sprint plan presumably overlaps phases. A sum-of-everything simulation overestimates the sprint by construction. Frozen correction [PROPOSED — correction is my proposal; the DAG is the authority]:

1. The simulation MUST model the C04.10 DAG: roots sequential, parallelizable layers as max() not sum, convergence points as joins. Sum-total is retained only as the documented upper bound (fully sequential worst case).
2. The P90 ≤ 10 days requirement (ADR-20) is evaluated against the DAG-structured simulation, not the sum. If the DAG simulation was what produced any current P10/P50/P90 (unstated — no outputs recorded), record the outputs + seed; a Monte Carlo without a recorded seed is not reproducible (C03.5).
3. New risks from this pass, scored on the register scale [PROPOSED scores — team ratification required]:

| ID | Risk | Category | L | I | D | Score | Owner | Mitigation | Residual |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R13 | Gate-numbering collision (C15.9) causes wrong gate to gate a phase | Schedule | 3 | 4 | 3 | 36 | PM | Adopt C15.9 disambiguation before P1 | 12 |
| R14 | Config-schema drift UI vs backend (C19.8) blocks lane configuration | Technical | 3 | 3 | 2 | 18 | Frontend lead | Generate frontend schema from backend | 6 |
| R15 | Judge calibration decay unmonitored until C06.16 implemented | Technical | 3 | 4 | 4 | 48 | QA lead | C06.16 monitor; A37 ablation | 16 |
| R16 | Kill-switch drill never executed (C10.8) — switch fails when needed | Technical | 2 | 5 | 4 | 40 | Backend lead | Drill at every phase gate | 10 |
| R17 | Erasure/append-only conflict blocks audit (C16.9 residual) | Data/privacy | 2 | 4 | 3 | 24 | Compliance lead | Tombstone protocol + retention bound | 8 |

### C23.8 Assumption-confidence audit (depth: confidences without validation are priors)

The Assumptions Register states confidences (0.6–0.99) — these are PRIORS [INFERRED — no validation evidence is cited in the register; ends at the table above]. Frozen rule: every assumption with confidence ≥ 0.9 must cite its validation artifact (A2 isometry verification for the 0.99; macOS version check output for Seatbelt 0.95; Rapid-MLX health-check log for 0.9) by P1 exit, or its confidence is reset to 0.6 (unvalidated) and the dependent risk scores recomputed. Confidence without evidence is optimism with decimals.

What was NOT checked here: C32 task list vs the Monte Carlo task dict (verify at C32); DPIA existence for R9 linkage (not in workspace).

## Open questions / gaps

- Seed from original doc open questions (C12, C15.5).
