# C09-FW — Framework Ablation Study Rules

**Status.** FROZEN. **Parent:** C09-FW Design.md

---

## Rule FW-1: No Python, No JSON — Enforced at Ablation Level

Every ablation workload, harness, and measurement pipeline **must not use Python or JSON**.

- Serialization: Custom mmap binary (Q32.31 fixed-point) or Cap'n Proto/FlatBuffers
- Configuration: TOML / binary schema / typed config structs
- Telemetry: Binary logs + Merkle receipts (C31)
- Benchmarking: Rust criterion / C++ Google Benchmark / Go testing — output as binary or TOML

**Violation**: Any ablation using Python or JSON is disqualified (FAIL with correction entry).

---

## Rule FW-2: Pre-Registration Hash Before Run

Per C09.1, **no ablation runs without a filed pre-registration hash**.

- Hash = `sha256(pre_registration.yaml)`
- Stored in `PreRegistrations/FW-A{X}.hash`
- CI Gate 1 verifies hash matches before ablation execution
- Null/negative results recorded identically to positive results

---

## Rule FW-3: Cross-Hardware Replication Required

Per C09.4, **every VERIFIED claim requires replication on second hardware**.

- Primary: MacBook Pro M3 Max, 36GB
- Replica: Mac Studio M2 Ultra, 96GB (minimum)
- Optional: Linux + NVIDIA H100
- **Replication criterion**: Effect size direction same, magnitude within 20% of primary
- Single-hardware PASS → claim stays PREREGISTERED (not VERIFIED)

---

## Rule FW-4: Necessity Testing for Every Cross-Language Boundary

Per C13.1 / C09.5, **every FFI/WASM/mmap boundary must pass necessity ablation**.

| Boundary | Necessity Ablation | Judge |
|----------|-------------------|-------|
| Rust ↔ C++ (kernels) | Remove FFI → inline Rust kernels | BF₀₁ > 10 for regression |
| Rust ↔ C++ (serialization) | Remove mmap → use FlatBuffers | BF₀₁ > 10 for regression |
| Rust → Go WASM (RPC) | Replace with Rust WASM | BF₀₁ > 10 for regression |
| TS ↔ Rust (IDE) | Replace with Rust/WASM frontend | BF₀₁ > 10 for regression |
| Go WASM ↔ DO | Replace with Rust WASM DO | BF₀₁ > 10 for regression |

**If BF₀₁ < 0.1**: Boundary is OPTIONAL — consolidate languages.

---

## Rule FW-5: Effect Size Reporting Mandatory

Per C09.7, **every ablation result must report standardized effect size with 95% CI**.

| Metric | Effect Size | CI Method | Threshold |
|--------|-------------|-----------|-----------|
| Throughput | Cohen's d | Bootstrap (10k) | d > 0.8 |
| Latency | % change | Percentile bootstrap | > 20% |
| Memory | % change | Percentile bootstrap | > 15% |
| Binary size | Ratio | Log-normal | > 1.5x |
| Cold start | Ratio | Log-normal | > 2x |

**No claim accepted without effect size + CI**.

---

## Rule FW-6: Hardware Normalization for All Claims

Per C09.8, **all performance numbers must be hardware-normalized**.

```
norm_metric = metric_target / metric_ref × spec_ref / spec_target
```

where `spec` includes:
- GPU compute (TFLOPS)
- Memory bandwidth (GB/s)
- CPU cores

Reference machine: **MacBook Pro M3 Max, 36GB Unified Memory**

---

## Rule FW-7: Claims Register Format

Per C09.6, **every claim enters the machine-readable claims register** with label:

- **VERIFIED**: Measured, replicated, pre-registered, effect size reported
- **REPORTED**: Source says so, unchecked (not acceptable for framework claims)
- **INFERRED**: Reasoned from evidence — state where evidence ends
- **PROPOSED**: Plan, not a fact (default for pre-registration)

**Unlabeled claims are treated as UNVERIFIED and blocked at Gate 1 (claim-lint)**.

---

## Rule FW-8: Failure Ledger — Corrections as Entries

Per C09.7, **failed ablations produce failure ledger entries, not deletions**.

```json
{
  "failure_id": "C09-FW-A1-kernel-fail",
  "ablation_id": "FW-A1",
  "date": "2026-10-27",
  "hypothesis": "C++ kernels achieve 1.5x throughput at equal memory",
  "result": "FAIL: 1.1x throughput, 1.15x memory",
  "root_cause": "Rust MLX kernel fusion outperformed hand-rolled C++ for Picard reduction",
  "correction": "Redesign C++ kernels using MLX-style fusion; re-run FW-A1",
  "status": "CORRECTED",
  "follow_up_ablation": "FW-A1-retry"
}
```

**Never silently rewrite history. Never delete the wrong claim.**

---

## Rule FW-9: Ablation Execution Order (Dependency DAG)

```
Phase 1 (Parallel, Independent):
  FW-A1  FW-A2  FW-A3   Kernels (C++ vs Rust/Zig/Swift)
  FW-A8  FW-A9            Serialization (mmap vs FlatBuffers/Cap'n Proto)
  FW-A12 FW-A13           Cloudflare Workers (Go vs Rust/TS)

Phase 2 (Depends on Phase 1 results):
  FW-A4  FW-A5           Inference Engine (Rust vs C++/Zig)
  FW-A6  FW-A7           Agent Harness (Rust vs Zig/Go)
  FW-A10 FW-A11          Frontend (TS vs Rust/WASM/Swift)

Phase 3 (Integration):
  FW-A14                Full stack: Rust+C+++TS+Go vs Rust-only
  FW-A15                Full stack: Rust+C+++TS+Go vs Zig-only
```

**Gate 3 requires Phase 1 complete. Gate 4 requires Phase 2 complete. Gate 5 requires Phase 3 complete.**

---

## Rule FW-10: Statistical Rigor — Sequential Testing

Per C09.5, **group sequential design with Lan-DeMets alpha spending (ρ=2)**.

- Interim analyses at 25%, 50%, 75%, 100% of planned samples
- Futility stop if conditional power < 0.2
- Early efficacy stop if p < α(t) at interim
- Final analysis at 100% with full α

**Sample size determined by power analysis (80% power, α=0.05, d=0.8)**.

---

## Rule FW-11: Bayesian Necessity Decision Rule

Per C09.6, **Bayes factor BF₀₁ for necessity**:

- BF₀₁ > 10 → **REQUIRED** (strong evidence component is necessary)
- BF₀₁ < 0.1 → **OPTIONAL** (strong evidence component is not necessary)
- 0.1 ≤ BF₀₁ ≤ 10 → **INCONCLUSIVE** (more data needed)

**Prior**: p(H₀) = p(H₁) = 0.5

---

## Rule FW-12: No Single-Language Claim Without FW-A14/FW-A15

**No claim that "Rust-only" or "Zig-only" or "C++-only" is optimal may be made** without passing FW-A14 (Rust-only) and FW-A15 (Zig-only) integration ablations.

The hypothesis is that the **multi-language stack is Pareto-optimal**. Single-language alternatives must be tested as full-system integrations, not component-wise.

---

## Rule FW-13: Dev Velocity as Measured Metric

For frontend ablations (FW-A10, FW-A11), **dev velocity is a measured metric**:

- HMR cycle time (ms) — measured
- Time to add new panel (minutes) — measured via controlled task
- A11y compliance score (axe-core) — measured
- LSP integration latency — measured

**Survey scores are supplementary, not primary.**

---

## Rule FW-14: Binary Size Budget

Per C21 (Performance Budgets), **binary size budgets are enforced**:

| Artifact | Budget | Ablation Check |
|----------|--------|----------------|
| Backend (Rust) | ≤50 MB | FW-A4, FW-A6 |
| Kernels (C++) | ≤10 MB | FW-A1, FW-A2, FW-A3 |
| Frontend (TS gz) | ≤170 KB | FW-A10, FW-A11 |
| Go WASM | ≤200 KB | FW-A12, FW-A13 |
| Rust WASM | ≤500 KB | FW-A12, FW-A14 |

**Exceeding budget = FAIL for that ablation.**

---

## Rule FW-15: Build Reproducibility

Per C10.5, **all ablation builds must be reproducible**.

- Pinned toolchain versions (rustc, clang, zig, tinygo, node)
- Locked dependencies (Cargo.lock, package-lock.json, go.sum)
- `verify_build.sh` produces bit-for-bit identical artifacts
- Build receipt signed (ed25519) with git commit + toolchain + lock hashes

---

## Rule FW-16: Zero-Trust Ablation Data

**Ablation data is untrusted until verified**:

- Raw measurements signed by runner (ed25519)
- Analysis notebooks executed in isolated environment
- Claims register entries require pre-registration hash match
- Independent auditor can re-run analysis from raw data

---

## Rule FW-17: Scope Boundaries

This ablation study **covers only framework/language selection** for:

- Hyperbolic KV kernels (C05, C12)
- Inference engine (C12)
- Agent harness (C07, C14, C25, C27, C28, C29, C30, C34)
- Serialization/IPC (C17, C18)
- Frontend/IDE (C19)
- Cloudflare Workers (C10)

**Out of scope**: Model selection, training, data pipeline, security policy, compliance mapping.

---

## Rule FW-18: Documentation Standard

Per user AGENTS.md §7, **all claims in this study follow documentation standards**:

1. Label every claim: VERIFIED / REPORTED / INFERRED / PROPOSED
2. Evidence or it didn't happen: hardware, settings, workload, comparison
3. Corrections are entries, not edits
4. Five-file set per workstream (Design.md, Pre-Registration.md, CLAIMS_REGISTER.md, Successes.md, Failures.md)
5. Date everything. Name files, paths, versions exactly.
6. No bulk anything — per-item disposition, per-claim evidence.

---

**End of Rules.** These rules are FROZEN for the sprint. Changes require ADR (C36).
