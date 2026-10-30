# Decisions — Canonical Decision Index (Pinned Scaffold File)

**Status.** FROZEN content, condensed in place (2026-10-27; CORR-17). **Parent:** Master Design Document index — scaffold appendix, not a numbered C-series category.
**Purpose.** Single pinned entry point for every canonical decision: the 25 frozen ADRs, the 17 ratified corrections, and the 18 proposed corrections awaiting ratification. This file IS the log of record (condensed from `Decision-Log-ADR/`, retired to git history).
**Note.** `(C36)` short ID retained per CORR-16; all inline `(C36)` references across C01–C35 resolve here.

---

## ADR Index (25 frozen — this file §Decisions-Log is the log of record)

| ADR    | Decision (short)                                                                                                               | Primary Categories | Gate (C15)     | Ablation (C09) | Related                                    |
| ------ | ------------------------------------------------------------------------------------------------------------------------------ | ------------------ | -------------- | -------------- | ------------------------------------------ |
| ADR-01 | Lorentz H³ canonical compute/storage; Poincaré projection for UX/radial salience; isometry within 4.4e-9                       | C35, C05           | Gate 3, Gate 8 | A1, A31        | C12, A1, A31                               |
| ADR-02 | Honest BM25 default; dense only when embed endpoint or bundled lane configured                                                 | C26                | Gate 4         | A3             | C14.5, NF6, A3                             |
| ADR-03 | Data-driven N-lane roster, no fixed 3 slots; `configurations.json` with forward-only migration                                 | C30, C34           | Gate 3, Gate 4 | A5, A17, A28   | C14.7, C20, C34                            |
| ADR-04 | Data root via workspace discovery + `ATOMA_WORKSPACE_ROOT`; no baked user paths                                                | C17                | Gate 3         | —              | C14.10, NF4                                |
| ADR-05 | Kill switch to standard transformer via one config flag, no data loss                                                          | C10                | Gate 7         | —              | C14 (§14)                                  |
| ADR-06 | Telemetry = ids/scopes/latency only; never args, results, text                                                                 | C31, All           | Gate 6         | —              | F8 (§10.4), all TelemetryAndDiagnostics.md |
| ADR-07 | Missing external "Read next" docs migrated or removed; no dangling references in ship artifacts                                | C33                | Gate 1         | A27            | Read next                                  |
| ADR-08 | Skill registry = compile-in manifest (`skills.manifest.json`); hand-edited JSON forbidden in ship builds                       | C28                | Gate 2, Gate 3 | A11, A12       | C14.6, C14.8                               |
| ADR-09 | Tool registry = compile-in manifest (`tools.manifest.json`) + effect system (Pure/ReadOnly/WriteLocal/WriteRemote/Destructive) | C29                | Gate 2, Gate 3 | A14, A15       | C14.6, C14.8, C25                          |
| ADR-10 | 23 hook moments fixed at compile time; no runtime registration; blocking veto                                                  | C28                | Gate 3, Gate 4 | A12            | C15, C25                                   |
| ADR-11 | Brigade skill activation scoped to lane geodesic neighborhood; single weight set invariant (weight_copies = 1)                 | C30, C06.12        | Gate 4, Gate 8 | A21            | C28, C25                                   |
| ADR-12 | Subagent loops in holes; depth cap (default 3) + budget cap (tokens/time/calls)                                                | C30                | Gate 4         | A19            | C25, C22                                   |
| ADR-13 | Branch isolation via git worktrees + hole sandbox; merge = explicit approval + receipt                                         | C30                | Gate 4, Gate 7 | A19            | C25, C22                                   |
| ADR-14 | Cross-lane comms = geodesic bridges on H³×T²; interference `I(i,j)` logged; breaker at `I < 0.1`                               | C30                | Gate 4, Gate 6 | A20            | C06.5, C22                                 |
| ADR-15 | Event ledger = SQLite append-only monotonic; Merkle root every 1000 entries; no UPDATE/DELETE                                  | C31                | Gate 6, Gate 7 | A22            | C15, C17                                   |
| ADR-16 | Telemetry process = separate PID; honest availability; persisted session across restarts                                       | C31                | Gate 6         | A22            | C15                                        |
| ADR-17 | Five-file set (Design, Pre-Registration, Claims Register, Successes, Failures) mandatory per workstream                        | C33                | Gate 1         | A26            | All Cxx                                    |
| ADR-18 | Claim labeling (VERIFIED/REPORTED/INFERRED/PROPOSED) + evidence pointer for non-PROPOSED                                       | C33                | Gate 1, Gate 2 | A26            | All Cxx                                    |
| ADR-19 | Corrections are entries (append-only); never edit/delete original claims                                                       | C33                | Gate 1         | A26            | All Cxx                                    |
| ADR-20 | 10-day sprint; PERT per task; Monte Carlo P90 ≤ 10 days required                                                               | C32                | Gate 1         | —              | C15, C23                                   |
| ADR-21 | 8 verification gates all pass Day 10 for go/no-go; each gate maps to C15                                                       | C15                | All            | —              | All                                        |
| ADR-22 | Preregistration hashes required before run; null/negative results first-class                                                  | C09                | Gate 5         | All A1–A35     | All A1–A37 (A36/A37 added 2026-10-26)      |
| ADR-23 | Reproducible builds: pin toolchain, record git commit + lock hashes in build receipt                                           | C10, C14           | Gate 8         | —              | C15                                        |
| ADR-24 | Toroidal worst-arc reporting mandatory (not average); phase coherence verified                                                 | C21                | Gate 8         | —              | C05, C35                                   |
| ADR-25 | Claim linter + five-file enforcer + crosslink validator in CI (Gates 1–2)                                                      | C33                | Gate 1, Gate 2 | A26            | C15                                        |

## Correction Entries (CORR-01–CORR-17, ratified — log of record)

| Correction | Date       | ADR Modified   | Reason                                                                                                                                                                                          |
| ---------- | ---------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CORR-01    | 2026-10-26 | ADR-01         | Clarified isometry threshold applies to all representable coordinates, not just test set                                                                                                        |
| CORR-02    | 2026-10-26 | ADR-03         | Added forward-only migration requirement via `config_version`                                                                                                                                   |
| CORR-03    | 2026-10-26 | ADR-06         | Extended to all TelemetryAndDiagnostics.md sidecars                                                                                                                                             |
| CORR-04    | 2026-10-26 | ADR-08         | Added bootstrap validation against `wrangler.jsonc` bindings                                                                                                                                    |
| CORR-05    | 2026-10-26 | ADR-09         | Added effect monotonicity: caller effect ≤ tool effect                                                                                                                                          |
| CORR-06    | 2026-10-26 | ADR-10         | Specified 23 hook moments explicitly                                                                                                                                                            |
| CORR-07    | 2026-10-26 | ADR-11         | Added brigade radius and submanifold anchor to lane config                                                                                                                                      |
| CORR-08    | 2026-10-26 | ADR-12         | Added budget cap dimensions (tokens, time, tool calls)                                                                                                                                          |
| CORR-09    | 2026-10-26 | ADR-14         | Added interference metric `I(i,j)` to event ledger                                                                                                                                              |
| CORR-10    | 2026-10-26 | ADR-15         | Added Merkle root every 1000 entries                                                                                                                                                            |
| CORR-11    | 2026-10-26 | ADR-17         | Added automated validation gate (C15)                                                                                                                                                           |
| CORR-12    | 2026-10-26 | ADR-18         | Added claim linter automation                                                                                                                                                                   |
| CORR-13    | 2026-10-26 | ADR-20         | Added PERT parameters and Monte Carlo method                                                                                                                                                    |
| CORR-14    | 2026-10-26 | ADR-22         | Extended to A11–A35 from deepening passes                                                                                                                                                       |
| CORR-15    | 2026-10-26 | ADR-24         | Clarified worst-arc = max geodesic distance on T², not average                                                                                                                                  |
| CORR-16    | 2026-10-27 | — (structural) | Relabeled C36-Decision-Log-ADR → Decision-Log-ADR scaffold subcategory; numbered series now C01–C35; C36 retained as stable short ID                                                            |
| CORR-17    | 2026-10-27 | — (structural) | Condensed `Decision-Log-ADR/` into pinned `Decisions.md`; folder retired to git history; `Decisions.md` is the log of record; traceability subsumed by ADR table Category/Gate/Ablation columns |

## Change Control

- Changes are appended as dated correction entries; never silently rewrite (docs standards §3/C03.3).
- Each correction references the ADR it modifies and the reason.

## Traceability (ADR → Category → Gate → Ablation)

Subsumed by the ADR Index table above (Primary Categories / Gate / Ablation columns). Single table, no drift between index and matrix — the two-table form retired with `Decision-Log-ADR/`.

## Verification Gates (C15 — ADR-log self-checks)

| Gate               | Test                                                    | Threshold |
| ------------------ | ------------------------------------------------------- | --------- |
| ADR consistency    | All ADRs referenced in categories match this log        | 100%      |
| ADR traceability   | Every ADR maps to category + gate + ablation            | 100%      |
| Correction format  | All corrections have date, ADR, reason                  | 100%      |
| No silent rewrites | Git history shows no force-push/rewrite on Decisions.md | 100%      |

## Proposed Corrections (18 pending ratification)

P-CORR-1 through P-CORR-18 (Muse, 2026-10-26): hook-count audit, generated-manifest rule, gate disambiguation, prereg checklist, kill-switch drills, lane admission, external-evidence pointers, five-file amendment, web.fetch reclassification, veto-immune hooks, stealing authority-transfer, ledger restriction, liveness-modulo-isolation, blast-class receipts, `acft_enabled` alias deprecation, pointer fixes, erasure tombstone protocol, RAG trust tiers.

**Ratification protocol:** each P-CORR is adopted (appended as CORR-18+ with sequential numbers) or REJECTED with reason — rejected items are marked, never deleted. No P-CORR is individually summarized here; the per-item table lives in git history (`Decision-Log-ADR/Design.md` §Proposed Corrections Appendix at retirement).

## Open Questions / Gaps (carried from retired log)

- ADR for dynamic lane creation (deferred to P6).
- ADR for curvature self-tuning (C06.7).
- ADR for distributed Atlas sync (C06.10).
- ADR for telemetry identity rotation (C34).

## Naming Decision ND-01 (2026-10-30, Stewart directive — agent-recorded, PENDING Stewart ratification)

- **N³** = "Nested Neural Net": the nested retrieval layer (vetted RAG + hyperbolic memory + capability DAG + telemetry sink). Substrate only. Binary `hypermind-mcp`. UNCHANGED.
- **Nested KV** = the key-ordered storage shape (`BTreeMap` + snapshot/WAL + SQLite sink). UNCHANGED.
- **Multi-Model Mind** = the single unified object: the multi-model knowledge map with atoms + Nested KV as 1 object. CANONICAL NAME for the unified object going forward. Source: C06 Brigade text ("carved out of the single Multi-Model Mind knowledge map (the Atlas)") — the only candidate with a frozen design definition.
- **Deprecated (do not use):** "Hyperbolic Global Mind" (agent-coined 2026-10-30, no source in repo — withdrawn same session), "HyperMind" as the unified-object name (collides with the N³-side binary/crate prefix; reserved for the substrate side).
- **Conversion note:** the Multi-Model Mind is to become an updateable MCP server. This entry records the name only; the conversion is unscoped work.
- **The dissolve rule:** N³ ≠ Multi-Model Mind ≠ Nested KV in every doc, receipt, and prereg going forward. Any text equating them is a defect to be filed, not edited silent (corrections are entries).
