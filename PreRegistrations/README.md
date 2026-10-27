# PreRegistrations — Ablation Preregistration Index (Pinned Top-Level Folder)

**Status.** PROPOSED scaffold pointer (2026-10-27). **Parent:** Repo root (pinned top-level folder).
**Purpose.** Single pinned entry point for every preregistered ablation. Preregistration TEXT lives in the files below; filing HASHES live in the hash tables at the foot of each file. Per C11.3/ADR-22: no filed hash, no run — no exceptions.
**Coverage.** A1–A10 (core) + A11–A37 (deepening + frontier additions A36/A37). A38+ slots reserved for sprint-emergent ablations.

---

## Prereg Files

| File | Ablations | Status |
| --- | --- | --- |
| `PRE-REGISTRATION-A1-A10.md` (this folder) | A1 (KV cache), A2 (recall), A3 (retrieval), A4 (fibers), A5 (lanes), A6 (budget), A7 (hooks), A8 (ingest), A9 (injection), A10 (red-team) | Text filed; hashes TBD |
| `PRE-REGISTRATION-A11-A35.md` (this folder) | A11–A35 (registry, tools, lanes, ledger, docs, geometry) + A36 (C06.15 depth-gated recall, C09.9 format) + A37 (C06.16 judge-drift monitor, C09.9 format) | Text filed incl. A36/A37; hashes TBD |

## Hash Filing Status

| Ablation | Hash Filed | Date | Notes |
| --- | --- | --- | --- |
| A1–A35 | NO | — | Text present, hashes empty — NO RUNS until filed (Day 1–2 exit per C32) |
| A36 | NO | — | Prereg text filed this pass; hash pending |
| A37 | NO | — | Prereg text filed this pass; hash pending |

## Content Standard

Every prereg is audited against the C09.9 8-item checklist (falsifiable hypothesis, workload, hardware, baseline, judge band, effect-size plan, null-handling, claims-register draft) at Gate 1. A filed hash over checklist-failing text is rejected on audit — hash necessity, not sufficiency.
