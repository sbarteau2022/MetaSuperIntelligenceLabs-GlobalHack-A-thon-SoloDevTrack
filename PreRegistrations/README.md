# PreRegistrations — Ablation Preregistration Index (Pinned Top-Level Folder)

**Status.** PROPOSED scaffold pointer (2026-10-27). **Parent:** Repo root (pinned top-level folder).
**Purpose.** Single pinned entry point for every preregistered ablation. Preregistration TEXT lives in the files below; filing HASHES live in the hash tables at the foot of each file. Per C11.3/ADR-22: no filed hash, no run — no exceptions.
**Coverage.** A1–A10 (core) + A11–A37 (deepening + frontier additions A36/A37). A38+ slots reserved for sprint-emergent ablations.

---

## Prereg Files

| File                                                                                    | Ablations                                                                                                                                                 | Status                                                                                                     |
| --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `PRE-REGISTRATION-A1-A10.md` (this folder)                                              | A1 (KV cache), A2 (recall), A3 (retrieval), A4 (fibers), A5 (lanes), A6 (budget), A7 (hooks), A8 (ingest), A9 (injection), A10 (red-team)                 | Text filed; hashes TBD                                                                                     |
| `PRE-REGISTRATION-A11-A35.md` (this folder)                                             | A11–A35 (registry, tools, lanes, ledger, docs, geometry) + A36 (C06.15 depth-gated recall, C09.9 format) + A37 (C06.16 judge-drift monitor, C09.9 format) | Text + section hashes filed 2026-10-29 (see file hash table)                                               |
| `C06-Frontier-Push/Pre-Registration-A38-v1.md` (sidecar)                                | A38 (Continuity Capsule round-trip, A38)                                                                                                                  | Filed; canonical hash in C06 `Claims_Register.md`                                                          |
| `C06-Frontier-Push/Pre-Registration-A39-v1.md` (sidecar)                                | A39 (map-carving A/B, task-scoped swarm map)                                                                                                              | Filed; canonical hash `sha256:4e8695fa4b0f115444b7b188881d7a8ee07f9fb387648aeaad664919a44e7fd6`            |
| `C06-Frontier-Push/Euclidean-Hyperbolic-Retraining-Protocol.pdf` (sidecar, byte-frozen) | A40 source protocol (Glimmer → hyperbolic retrain, G0–G4 gates)                                                                                           | Filed 2026-10-27; SHA-256 `sha256:4bcca0fa5d2e4a4e2ff2ef0e9093067f63fd637fa3c0a42328a306be63d89472`        |
| `C06-Frontier-Push/Pre-Registration-A40-v1.md` (sidecar)                                | A40 v1 SUPERSEDED, PRESERVED (thresholds frozen)                                                                                                          | Filed 2026-10-27; hash `sha256:af8892ce034a02a182df790f5b0e951f9ccab73467a55df1c4bbc9b4614d7e3a`           |
| `C06-Frontier-Push/Pre-Registration-A40-v1.1.md` (sidecar)                              | A40 v1.1 CANONICAL (all 8 conditions closed by construction)                                                                                              | Filed 2026-10-27; canonical hash `sha256:ac991e6e8874c5d59034346826f5122a95b2144fa47879ae2c59095c623a125c` |

## Hash Filing Status

| Ablation | Hash Filed                                                                                         | Date       | Notes                                                                                                                                                 |
| -------- | -------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1–A10   | YES — per-section text hashes in file footer, C09.9-complete per-section, PREREGISTERED 2026-10-29 | 2026-10-29 | Hashes live in `PRE-REGISTRATION-A1-A10.md` footer (section-text scheme); run hashes filed at run time; no run without text hash + run hash (C11.3).  |
| A11–A35  | YES — per-section text hashes in file footer, C09.9-complete per-section, PREREGISTERED 2026-10-29 | 2026-10-29 | Hashes live in `PRE-REGISTRATION-A11-A35.md` footer (section-text scheme); run hashes filed at run time; no run without text hash + run hash (C11.3). |
| A36      | YES — §A36 text hash in `PRE-REGISTRATION-A11-A35.md` footer                                       | 2026-10-29 | `ca78e271…3d57217529` over §A36 bytes; NO RUN until run manifest                                                                                      |
| A37      | YES — §A37 text hash in `PRE-REGISTRATION-A11-A35.md` footer                                       | 2026-10-29 | `eca33754…59c42646` over §A37 bytes; NO RUN until run manifest                                                                                        |
| A38      | YES                                                                                                | 2026-10-26 | Filed in C06 workstream (`Pre-Registration.md` v1.1)                                                                                                  |
| A39      | YES                                                                                                | 2026-10-27 | Filed in C06 workstream (`Pre-Registration-A39-v1.md`)                                                                                                |
| A40      | YES                                                                                                | 2026-10-27 | v1.1 CANONICAL (8/8 conditions closed by construction); RUN-READY, APPROVED by Stewart — next step is frozen run manifest                             |

## Content Standard

Every prereg is audited against the C09.9 8-item checklist (falsifiable hypothesis, workload, hardware, baseline, judge band, effect-size plan, null-handling, claims-register draft) at Gate 1. A filed hash over checklist-failing text is rejected on audit — hash necessity, not sufficiency.

## Ratification 2026-10-30 (Stewart directive: RATIFY)

[VERIFIED — recompute script over worktree files, canonical convention §-bytes `## A<N>` → blank line before next header / `---` rule, no trailing newline] Footer hashes MATCH section text 37/37 (A1–A10: 10/10; A11–A37: 27/27, incl. A36 `ca78e271…` + A37 `eca33754…`). Convention recorded in both files' preamble ("Hash-filing convention (RATIFIED 2026-10-30)"). The §37 Correction below is SUPERSEDED in its disposition ("BOTH SCHEMES WITHDRAWN") but RETAINED as history: the committed `480a3943…`/`75e4446e…` remain superseded (match no text on disk); the footer scheme stands CANONICAL. Text frozen at ratification — any edit forks a new version + new hashes per ADR-19/ADR-22. NO RUNS until run manifest per ablation (unchanged).

## Correction 2026-10-29 — hash-scheme reconciliation (SUPERSEDED IN DISPOSITION 2026-10-30 — retained as history)

[VERIFIED — `git show HEAD:PreRegistrations/README.md` + worktree diff read directly] Two hash schemes were filed for the same A11–A37 text in the same session: (a) HEAD-committed section hashes `480a3943…` (A36) / `75e4446e…` (A37), which match NO section text currently on disk (recomputed `ca78e271…` / `eca33754…` over current §A36/§A37 — neither equals the committed values, and no revision history records the hashed text); (b) worktree file-level hashes `abbe048b…` / `a3ff2598…`, which are STALE by construction (computed before the hash-table edit that changed the files' bytes — self-referential filing, invalid pattern).

**Disposition [PROPOSED — requires Stewart ratification]:** BOTH schemes WITHDRAWN as unverifiable. Single canonical scheme going forward: per-ablation section-text hashes, computed over exact section bytes and recorded in the hash tables at the foot of each prereg file (the A38–A40 / file-footer pattern, ADR-19 clean — hashes live in footers, texts stay hash-free so filing never invalidates itself). A1–A37: NO RUNS until the footer tables carry fresh section hashes over frozen text — current status returns to the pre-session state (text present, hashes empty) until the footer pass completes. The C09.9 per-section text upgrades from this session are RETAINED (content work, not hash work).
