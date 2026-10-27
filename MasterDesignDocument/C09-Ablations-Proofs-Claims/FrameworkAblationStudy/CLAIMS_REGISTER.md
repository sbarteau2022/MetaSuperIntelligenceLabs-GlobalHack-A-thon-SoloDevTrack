# C09-FW — Framework Ablation Study Claims Register

**Status.** LIVE — updated per ablation run.  
**Parent:** C09-FW FrameworkAblationStudy/Design.md  
**Standard:** Per C09.6 / AGENTS.md §7 — every claim labeled VERIFIED/REPORTED/INFERRED/PROPOSED with evidence.

---

## Claim Label Definitions

| Label | Meaning | Standard |
|-------|---------|----------|
| **VERIFIED** | Measured, replicated, pre-registered, effect size + CI reported | Gold standard |
| **REPORTED** | Source says so, unchecked | Not acceptable for framework claims |
| **INFERRED** | Reasoned from evidence — state where evidence ends | Acceptable with explicit boundary |
| **PROPOSED** | Plan, not a fact (default for pre-registration) | Must be upgraded or withdrawn |

---

## Claims Register (Machine-Readable Index)

| Claim ID | Ablation | Label | Statement | Evidence Link | Status |
|----------|----------|-------|-----------|---------------|--------|
| C09-FW-A1-throughput | FW-A1 | PROPOSED | C++ kernels achieve 1.5x throughput at 0.97x memory vs Rust MLX | ablation_results/FW-A1/claim.json | Pending |
| C09-FW-A1-instructions | FW-A1 | PROPOSED | C++ instruction count ≤ 0.8x Rust for kernel fusion ops | ablation_results/FW-A1/claim.json | Pending |
| C09-FW-A1-compile | FW-A1 | PROPOSED | C++ build time ≤ 1.5x Rust build time | ablation_results/FW-A1/claim.json | Pending |
| C09-FW-A2-throughput | FW-A2 | PROPOSED | Zig kernels achieve parity with C++ on VP-tree + Lorentz ops | ablation_results/FW-A2/claim.json | Pending |
| C09-FW-A3-throughput | FW-A3 | PROPOSED | Swift MPS achieves parity with C++ on hyperbolic kernels | ablation_results/FW-A3/claim.json | Pending |
| C09-FW-A4-inference | FW-A4 | PROPOSED | Rust ACFT inference achieves target tok/s + TTFT vs C++ | ablation_results/FW-A4/claim.json | Pending |
| C09-FW-A5-inference | FW-A5 | PROPOSED | Zig ACFT inference achieves parity with Rust | ablation_results/FW-A5/claim.json | Pending |
| C09-FW-A6-harness | FW-A6 | PROPOSED | Rust harness ≤2x lane spawn + ≤1.5x tool dispatch vs Zig | ablation_results/FW-A6/claim.json | Pending |
| C09-FW-A6-allowlist | FW-A6 | PROPOSED | Both Rust/Zig enforce tool allowlist at compile-time | ablation_results/FW-A6/claim.json | Pending |
| C09-FW-A7-harness | FW-A7 | PROPOSED | Go harness achieves parity with Rust on lane/tool metrics | ablation_results/FW-A7/claim.json | Pending |
| C09-FW-A8-serialization | FW-A8 | PROPOSED | Custom mmap achieves ≤50ns decode vs FlatBuffers | ablation_results/FW-A8/claim.json | Pending |
| C09-FW-A9-serialization | FW-A9 | PROPOSED | Custom mmap achieves ≤50ns decode vs Cap'n Proto | ablation_results/FW-A9/claim.json | Pending |
| C09-FW-A10-frontend | FW-A10 | PROPOSED | TS frontend ≤170KB gz + <100ms tab-open + WCAG AA | ablation_results/FW-A10/claim.json | Pending |
| C09-FW-A11-frontend | FW-A11 | PROPOSED | Swift frontend achieves parity with TS on IDE metrics | ablation_results/FW-A11/claim.json | Pending |
| C09-FW-A12-workers | FW-A12 | PROPOSED | Go WASM ≤2ms cold start + ≤10MB + ≤5ms CPU on DO | ablation_results/FW-A12/claim.json | Pending |
| C09-FW-A13-workers | FW-A13 | PROPOSED | TS Workers achieves parity with Go on DO metrics | ablation_results/FW-A13/claim.json | Pending |
| C09-FW-A14-integration | FW-A14 | PROPOSED | Multi-lang stack Pareto-dominates Rust-only full stack | ablation_results/FW-A14/claim.json | Pending |
| C09-FW-A15-integration | FW-A15 | PROPOSED | Multi-lang stack Pareto-dominates Zig-only full stack | ablation_results/FW-A15/claim.json | Pending |
| C09-FW-NEC-KERNELS | FW-NEC-KERNELS | PROPOSED | C++ kernels necessary (BF₀₁ > 10) | ablation_results/FW-NEC-KERNELS/claim.json | Pending |
| C09-FW-NEC-HARNESS | FW-NEC-HARNESS | PROPOSED | Rust harness necessary (BF₀₁ > 10) | ablation_results/FW-NEC-HARNESS/claim.json | Pending |
| C09-FW-NEC-FRONTEND | FW-NEC-FRONTEND | PROPOSED | TS frontend necessary (BF₀₁ > 10) | ablation_results/FW-NEC-FRONTEND/claim.json | Pending |
| C09-FW-NEC-WORKERS | FW-NEC-WORKERS | PROPOSED | Go Workers necessary (BF₀₁ > 10) | ablation_results/FW-NEC-WORKERS/claim.json | Pending |
| C09-FW-STACK-PARETO | FW-A14+FW-A15 | PROPOSED | Multi-lang stack is Pareto-optimal (no single-lang dominates) | ablation_results/FW-A14/claim.json + FW-A15/claim.json | Pending |

---

## Claim Detail Template (Per Ablation)

Each ablation produces a `claim.json` with this structure:

```json
{
  "claim_id": "C09-FW-A1-throughput",
  "ablation_id": "FW-A1",
  "label": "VERIFIED",
  "statement": "C++ hyperbolic kernels achieve 1.61x throughput at 0.97x peak memory vs Rust MLX on M3 Max",
  "evidence": {
    "hardware": "MacBook Pro M3 Max, 36GB",
    "workload": "1M vectors, H³×T², mixed insert/search/evict/picard",
    "baseline_throughput_mvs": 42.3,
    "treatment_throughput_mvs": 68.1,
    "baseline_peak_gb": 28.1,
    "treatment_peak_gb": 27.4,
    "effect_size": 1.42,
    "ci_95": [1.1, 1.8],
    "p_value": 0.003,
    "bayes_factor": 47.2,
    "replication": "Mac Studio M2 Ultra: 1.52x confirmed (within 20%)"
  },
  "pre_registration_hash": "sha256:abc123...",
  "timestamp": "2026-10-27T14:30:00Z",
  "corrections": []
}
```

---

## Necessity Claims (Bayesian)

| Claim ID | Component | BF₀₁ | Label | Interpretation |
|----------|-----------|------|-------|----------------|
| C09-FW-NEC-KERNELS | C++ kernels | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |
| C09-FW-NEC-HARNESS | Rust harness | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |
| C09-FW-NEC-FRONTEND | TS frontend | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |
| C09-FW-NEC-WORKERS | Go Workers | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |
| C09-FW-NEC-FFI-RC | Rust↔C++ FFI | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |
| C09-FW-NEC-FFI-RG | Rust→Go WASM | TBD | PROPOSED | BF₀₁ > 10 → REQUIRED |

---

## Correction Ledger (Append-Only)

| Date | Claim ID | Previous Label | New Label | Correction | Author |
|------|----------|----------------|-----------|------------|--------|
| — | — | — | — | No corrections yet | — |

---

## Verification Checklist

- [ ] All claims have pre-registration hash
- [ ] All VERIFIED claims have replication on second hardware
- [ ] All claims report effect size + 95% CI
- [ ] All claims hardware-normalized to M3 Max reference
- [ ] No PROPOSED claims cited in design docs without ablation
- [ ] claim-lint passes (Gate 1)
- [ ] Failure ledger entries for any FAIL results

---

**Next Update**: After FW-A1, FW-A6, FW-A10, FW-A12 complete (Phase 1).
