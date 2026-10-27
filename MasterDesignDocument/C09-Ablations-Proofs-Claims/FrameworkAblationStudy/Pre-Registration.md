# C09-FW — Framework Ablation Study Pre-Registration Index

**Status.** LIVE — tracks pre-registration hashes for all ablations.  
**Parent:** C09-FW FrameworkAblationStudy/Design.md  
**Standard:** Per C09.1 / C09.9 — **no ablation runs without filed hash**. Hash = `sha256(pre_registration.yaml)`.

---

## Pre-Registration Checklist (Per C09.9)

Every pre-registration MUST contain:

- [ ] Hypothesis in falsifiable form (treatment X beats baseline Y on metric M by ≥ δ — δ stated numerically)
- [ ] Workload: fixed corpus/benchmark ID + seed(s) + generation parameters
- [ ] Hardware: primary + replica target IDs; normalization formula inputs recorded
- [ ] Baseline: exact commit + config flag state of comparison path
- [ ] Judge: pass/fail predicate with thresholds, confidence band, sample-size plan with alpha-spending interim points
- [ ] Effect-size plan: metric-specific effect size + CI method
- [ ] Null-handling: pre-stated interpretation of each plausible null/negative outcome
- [ ] Claims-register draft: claim IDs this run could create, in PROPOSED state

---

## Pre-Registration Status Tracker

| Ablation        | Component                                          | Pre-Reg File                            | Hash Filed | Hash Value                                                                | Status        |
| --------------- | -------------------------------------------------- | --------------------------------------- | ---------- | ------------------------------------------------------------------------- | ------------- |
| FW-A1           | Hyperbolic KV Kernels (C++ vs Rust)                | `PreRegistrations/FW-A1.yaml`           | ✅         | `sha256:98468bd5a735f7aa88ef960fb0a61a7a9d4998b7a6e5b659f4786915019e0355` | PREREGISTERED |
| FW-A1-CUDA      | Hyperbolic KV Kernels (CUDA Rust cutile-rs vs C++) | `PreRegistrations/FW-A1-CUDA.yaml`      | ☐          | —                                                                         | DRAFT         |
| FW-A2           | Hyperbolic KV Kernels (C++ vs Zig)                 | `PreRegistrations/FW-A2.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A3           | Hyperbolic KV Kernels (C++ vs Swift)               | `PreRegistrations/FW-A3.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A4           | Inference Engine (Rust vs C++)                     | `PreRegistrations/FW-A4.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A4-CUDA      | Inference Engine (CUDA Rust vs C++)                | `PreRegistrations/FW-A4-CUDA.yaml`      | ☐          | —                                                                         | DRAFT         |
| FW-A5           | Inference Engine (Rust vs Zig)                     | `PreRegistrations/FW-A5.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A6           | Agent Harness (Rust vs Zig)                        | `PreRegistrations/FW-A6.yaml`           | ✅         | `sha256:ec2ce78730f7c34adb0d41d31f4ec1be24e3496882bd2cfbbda6bd4f785c958c` | PREREGISTERED |
| FW-A7           | Agent Harness (Rust vs Go)                         | `PreRegistrations/FW-A7.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A8           | Serialization (mmap vs FlatBuffers)                | `PreRegistrations/FW-A8.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A9           | Serialization (mmap vs Cap'n Proto)                | `PreRegistrations/FW-A9.yaml`           | ☐          | —                                                                         | DRAFT         |
| FW-A10          | Frontend (TS vs Rust/WASM)                         | `PreRegistrations/FW-A10.yaml`          | ✅         | `sha256:2b676c1df5da5810c253ff233cd8d0f72e6658dd83b0eb749ad1f33ebd7c65e4` | PREREGISTERED |
| FW-A11          | Frontend (TS vs Swift)                             | `PreRegistrations/FW-A11.yaml`          | ☐          | —                                                                         | DRAFT         |
| FW-A12          | Cloudflare Workers (Go vs Rust WASM)               | `PreRegistrations/FW-A12.yaml`          | ✅         | `sha256:f51cb56b6d22ea8345448625a2a764650ba61319dea533b421a590d5f0cfad1b` | PREREGISTERED |
| FW-A13          | Cloudflare Workers (Go vs TS)                      | `PreRegistrations/FW-A13.yaml`          | ☐          | —                                                                         | DRAFT         |
| FW-A14          | Full Stack (Multi-lang vs Rust-only)               | `PreRegistrations/FW-A14.yaml`          | ☐          | —                                                                         | DRAFT         |
| FW-A14-CUDA     | Full Stack (Multi-substrate vs Rust-only)          | `PreRegistrations/FW-A14-CUDA.yaml`     | ☐          | —                                                                         | DRAFT         |
| FW-A15          | Full Stack (Multi-lang vs Zig-only)                | `PreRegistrations/FW-A15.yaml`          | ☐          | —                                                                         | DRAFT         |
| FW-NEC-KERNELS  | Necessity: C++ kernels                             | `PreRegistrations/FW-NEC-KERNELS.yaml`  | ☐          | —                                                                         | DRAFT         |
| FW-NEC-HARNESS  | Necessity: Rust harness                            | `PreRegistrations/FW-NEC-HARNESS.yaml`  | ☐          | —                                                                         | DRAFT         |
| FW-NEC-FRONTEND | Necessity: TS frontend                             | `PreRegistrations/FW-NEC-FRONTEND.yaml` | ☐          | —                                                                         | DRAFT         |
| FW-NEC-WORKERS  | Necessity: Go Workers                              | `PreRegistrations/FW-NEC-WORKERS.yaml`  | ☐          | —                                                                         | DRAFT         |
| FW-NEC-FFI-RC   | Necessity: Rust↔C++ FFI                            | `PreRegistrations/FW-NEC-FFI-RC.yaml`   | ☐          | —                                                                         | DRAFT         |
| FW-NEC-FFI-RG   | Necessity: Rust→Go WASM                            | `PreRegistrations/FW-NEC-FFI-RG.yaml`   | ☐          | —                                                                         | DRAFT         |

---

## Critical Path (Phase 1) — Must Complete First

| Priority | Ablation | Rationale                                                      |
| -------- | -------- | -------------------------------------------------------------- |
| 1        | FW-A1    | Kernels are the compute foundation; all else builds on this    |
| 2        | FW-A6    | Agent harness is the orchestration backbone; locks Rust vs Zig |
| 3        | FW-A10   | Frontend is the human interface; locks TS vs Rust/WASM         |
| 4        | FW-A12   | Workers is the deployment target; locks Go vs Rust WASM        |

**Gate 3 Requirement**: All 4 critical-path ablations PASS (primary judge) with replication underway.

---

## Hash Filing Procedure

```bash
# 1. Finalize pre-registration YAML (review all checklist items)
# 2. Compute hash
sha256sum PreRegistrations/FW-A1.yaml > PreRegistrations/FW-A1.hash

# 3. Verify hash in YAML matches
#    Edit YAML: pre_registration_hash: "sha256:$(cat FW-A1.hash | cut -d' ' -f1)"

# 4. Commit hash file (immutable record)
git add PreRegistrations/FW-A1.hash
git commit -m "chore: file pre-registration hash for FW-A1"

# 5. CI Gate 1 verifies hash match before ablation run
```

---

## Hash Verification (CI Gate 1)

```yaml
# .github/workflows/ablation_gate1.yml
name: Ablation Gate 1 — Pre-Registration Verification
on:
  workflow_dispatch:
    inputs:
      ablation_id:
        type: choice
        options:
          [
            FW-A1,
            FW-A1-CUDA,
            FW-A2,
            FW-A3,
            FW-A4,
            FW-A4-CUDA,
            FW-A5,
            FW-A6,
            FW-A7,
            FW-A8,
            FW-A9,
            FW-A10,
            FW-A11,
            FW-A12,
            FW-A13,
            FW-A14,
            FW-A14-CUDA,
            FW-A15,
            FW-NEC-KERNELS,
            FW-NEC-HARNESS,
            FW-NEC-FRONTEND,
            FW-NEC-WORKERS,
            FW-NEC-FFI-RC,
            FW-NEC-FFI-RG,
          ]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify pre-registration hash
        run: |
          ABLATION=${{ inputs.ablation_id }}
          EXPECTED_HASH=$(cat PreRegistrations/${ABLATION}.hash | cut -d' ' -f1)
          ACTUAL_HASH=$(sha256sum PreRegistrations/${ABLATION}.yaml | cut -d' ' -f1)
          if [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
            echo "HASH MISMATCH: expected $EXPECTED_HASH, got $ACTUAL_HASH"
            exit 1
          fi
          echo "Hash verified: $EXPECTED_HASH"
```

---

## Next Actions

1. **Review all 4 critical-path pre-registrations** (FW-A1, FW-A6, FW-A10, FW-A12) for completeness
2. **File hashes** for all 4 → commit to git
3. **Implement baseline workloads** (hyperbolic_kv_bench, agent_harness_bench, ide_frontend_bench, cf_workers_bench)
4. **Run FW-A1 first** (kernels are the dependency for FW-A4, FW-A14)
5. **Record results** in Claims Register, Successes.md, Failures.md

---

**Gate 1 Blockers**: Any pre-registration missing checklist items → FAIL at Gate 1 document review.
