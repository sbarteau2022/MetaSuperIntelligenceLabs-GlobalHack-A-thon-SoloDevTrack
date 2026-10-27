# PreRegistrations — Ablation Preregistration Index (Pinned Top-Level Folder)

**Status.** PROPOSED scaffold pointer (2026-10-27). **Parent:** Repo root (pinned top-level folder).
**Purpose.** Single pinned entry point for every preregistered ablation. Preregistration TEXT lives in the files below; filing HASHES live in the hash tables at the foot of each file. Per C11.3/ADR-22: no filed hash, no run — no exceptions.
**Coverage.** A1–A10 (core) + A11–A37 (deepening + frontier additions A36/A37). A38+ slots reserved for sprint-emergent ablations.

---

## Prereg Files

| File                                                                                    | Ablations                                                                                                                                                 | Status                                                                                                     |
| --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `PRE-REGISTRATION-A1-A10.md` (this folder)                                              | A1 (KV cache), A2 (recall), A3 (retrieval), A4 (fibers), A5 (lanes), A6 (budget), A7 (hooks), A8 (ingest), A9 (injection), A10 (red-team)                 | Text filed; hashes TBD                                                                                     |
| `PRE-REGISTRATION-A11-A35.md` (this folder)                                             | A11–A35 (registry, tools, lanes, ledger, docs, geometry) + A36 (C06.15 depth-gated recall, C09.9 format) + A37 (C06.16 judge-drift monitor, C09.9 format) | Text filed incl. A36/A37; hashes TBD                                                                       |
| `C06-Frontier-Push/Pre-Registration-A38-v1.md` (sidecar)                                | A38 (Continuity Capsule round-trip, A38)                                                                                                                  | Filed; canonical hash in C06 `Claims_Register.md`                                                          |
| `C06-Frontier-Push/Pre-Registration-A39-v1.md` (sidecar)                                | A39 (map-carving A/B, task-scoped swarm map)                                                                                                              | Filed; canonical hash `sha256:4e8695fa4b0f115444b7b188881d7a8ee07f9fb387648aeaad664919a44e7fd6`            |
| `C06-Frontier-Push/Euclidean-Hyperbolic-Retraining-Protocol.pdf` (sidecar, byte-frozen) | A40 source protocol (Glimmer → hyperbolic retrain, G0–G4 gates)                                                                                           | Filed 2026-10-27; SHA-256 `sha256:4bcca0fa5d2e4a4e2ff2ef0e9093067f63fd637fa3c0a42328a306be63d89472`        |
| `C06-Frontier-Push/Pre-Registration-A40-v1.md` (sidecar)                                | A40 (hypo-native retrain A/B/C + geometry/preservation/reuse gates)                                                                                       | Filed 2026-10-27; canonical hash `sha256:af8892ce034a02a182df790f5b0e951f9ccab73467a55df1c4bbc9b4614d7e3a` |

## Hash Filing Status

| Ablation | Hash Filed | Date       | Notes                                                                                                               |
| -------- | ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------- |
| A1–A35   | NO         | —          | Text present, hashes empty — NO RUNS until filed (Day 1–2 exit per C32)                                             |
| A36      | NO         | —          | Prereg text filed this pass; hash pending                                                                           |
| A37      | NO         | —          | Prereg text filed this pass; hash pending                                                                           |
| A38      | YES        | 2026-10-26 | Filed in C06 workstream (`Pre-Registration.md` v1.1)                                                                |
| A39      | YES        | 2026-10-27 | Filed in C06 workstream (`Pre-Registration-A39-v1.md`)                                                              |
| A40      | YES        | 2026-10-27 | Protocol PDF byte-frozen + v1 text filed with canonical hash; conditions C-A40-01…08 OPEN — NO A40 RUN until closed |

## Content Standard

Every prereg is audited against the C09.9 8-item checklist (falsifiable hypothesis, workload, hardware, baseline, judge band, effect-size plan, null-handling, claims-register draft) at Gate 1. A filed hash over checklist-failing text is rejected on audit — hash necessity, not sufficiency.
