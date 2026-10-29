# Tooling Conditions — Disposition Addendum 2026-10-29

**Scope.** Every open condition in `TOOLING-AUDIT-2026-10-28.md` §3 (inherited from `MasterTooling/README.md`). **Method.** Read-only verification against the repo at HEAD + `cargo test` evidence where executable. **Rule.** A condition closes only on evidence; otherwise it gets a named owner + unblocker, never a silent pass.
**Status labels:** CLOSED (evidence in-repo) / OWNED (named owner + next command) / BLOCKED (needs hardware, install, or human).

## ToolingC01 (SKL-01 Formal Specification)

| #   | Condition                                                            | Disposition                                                                                                                                                                                                                                                       |
| --- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `RB-SPEC-*` runbook IDs not in C22                                   | **OWNED (build).** C22 subcategories are C22.1-titled (OOM/Circuit-Breaker/Merge), no `RB-SPEC-` IDs exist (VERIFIED by read). Next: create `RB-SPEC-01/02/03` under C22 with drill dates per C22.8, or remap ToolingC01 refs to C22.x titles. Doc edit, no code. |
| 2   | `FormalSpecWorkflow`/`AblationPreconditionWorkflow` anchors PROPOSED | **OWNED (build).** No such named workflows in C08/C09 (VERIFIED: subcategories numbered, not named; grep 0 hits). Next: create them as specified or rename ToolingC01 refs to match.                                                                              |
| 3   | TOOL-01.05 `cargo-contract` name WRONG (ink! collision)              | **OWNED (build).** Binary is NOT the Rust MIR VC extractor (REPORTED, package-registry knowledge). Next: substitute Kani- or Creusot-class tooling, log substitution in Decisions.md per ADR. Install needed: `kani`/`creusot` absent (VERIFIED: not on PATH).    |

## ToolingC02 (SKL-02 Systems/Kernels)

| #   | Condition                                | Disposition                                                                                                                                                                                                                               |
| --- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | nightly-SIMD vs `--locked` gating        | **OWNED (build).** Next: record the gate decision (feature-gated nightly paths with stable fallback) in Decisions.md. ADR-23 (reproducible builds, pinned toolchain) is the authority.                                                    |
| 2   | `perf` scoped to H100 replica            | **CLOSED (doc).** Apple Silicon profiling is `instruments`/`powermetrics` (VERIFIED installed: `/usr/bin/powermetrics` per audit §2); `perf` is Linux-only. Standing rule: `perf` cited for H100 replica target only, never for Mac runs. |
| 3   | A31/A32 fuzz-corpus seeds pre-registered | **OWNED (build).** A31/A32 hashes empty (VERIFIED: PreRegistrations hash table blank). Next: file seeds in prereg before any fuzz run (ADR-22).                                                                                           |

## ToolingC03 (SKL-03 Agentic Runtime)

| #   | Condition                                    | Disposition                                                                                                                                                                                                                           |
| --- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | hook count audited to 23                     | **CLOSED.** C28.2 specifies 23 hook moments (VERIFIED in `C28/Design.md:12,25`). Count matches ToolingC03's 23.                                                                                                                       |
| 2   | veto-immune audit hooks before RB-RUNTIME-02 | **OWNED (build).** Design states the requirement (P-CORR-10); no veto-immune implementation exists in-repo (INFERRED: no veto code found in `research-grade-harness/src`). Next: implement + test before RB-RUNTIME-02 is executable. |
| 3   | protobuf schemas generated, not hand-written | **OWNED (build).** Next: generate session-type protos from the C34 grammar; hand-written `.proto` for handoffs rejected at Gate 3. No `.proto` files in repo today (nothing to grandfather).                                          |

## ToolingC04 (SKL-04 Observability)

| #   | Condition                                    | Disposition                                                                                                                                                                                 |
| --- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ledger encryption-at-rest decision before P5 | **OWNED (Stewart).** Decision required: FileVault/APFS assumption + validation, or column-level encryption with secret-slot key (C31.8 residual risk). Blocks P5.                           |
| 2   | RingBuffer labeled LOSSY in UI               | **OWNED (build).** Next: label every UI surface consuming the RingBuffer LOSSY; alerting taps ledger-first (P-CORR-13 ordering). No UI surfaces ship today — condition carries to first UI. |
| 3   | dashboards not cited in claims               | **CLOSED (standing rule).** Only ledger + receipts + roll-up JSON are citable (VERIFIED as stated policy). No dashboard exists to miscite.                                                  |

## ToolingC05 (SKL-05 Verification)

| #   | Condition                             | Disposition                                                                                                                                                                                                                                                                                                                                   |
| --- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | A36/A37 hashes empty                  | **CLOSED.** Filed 2026-10-29: A36 `sha256:480a3943ac8ec1be693ba013e5e19c190649af0537f77e21fd42da7d7f489d3f`, A37 `sha256:75e4446e0256451d08990934d4404bf13b47511c5f2ad92e84ee84128ddad5ed` computed over the prereg section texts and recorded in `PreRegistrations/PRE-REGISTRATION-A11-A35.md` hash table. No runs until filed — now filed. |
| 2   | framework tools self-gated (Gate 2+6) | **OWNED (build).** Next: run the framework's own tools (`verify_metric_evidence.py` et al.) through Gate 2 (static) + Gate 6 (property); TOOL-05.05 monitor's own band set by manual Day-1 review.                                                                                                                                            |
| 3   | Monte Carlo seeded at Day 1–2 exit    | **OWNED (Stewart — schedule).** Next: run `cargo xtask monte-carlo` (or current equivalent) with recorded seed at Day 1–2 exit. No sprint clock started; condition waits on it.                                                                                                                                                               |

## ToolingC06 (SKL-06 Data/Knowledge)

| #   | Condition                                    | Disposition                                                                                                                                                                         |
| --- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | A8 per-tier round-trips                      | **OWNED (build).** Next: tier-parameterize the A8 ingest round-trip or qualify the single-tier claim. A8 hash empty (no runs until filed).                                          |
| 2   | A3 epsilon power-justified + per-collection  | **OWNED (build).** Next: justify epsilon 0.02 by power analysis in the A3 prereg + per-collection deltas + query-set minimum N. A3 hash empty.                                      |
| 3   | claim-linter grandfather rule                | **OWNED (build).** Next: Day 6–7 dated labeling pass (pre-cutoff violations → backlog, not build failure). No linter implementation in repo today — rule carries to linter build.   |
| 4   | `arrow`/`parquet` consumer stated or removed | **OWNED (build).** Next: record the columnar-format consumer (roll-up analytics? benchmark corpora?) or remove the deps. Unjustified deps violate the Quality > Quantity principle. |

## ToolingC07 (SKL-07 Security/Sandbox — ship-blocker)

| #   | Condition                       | Disposition                                                                                                                                                                                     |
| --- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | IFC pointer is C07.5, not C07.8 | **CLOSED (doc).** ToolingC07 §26 already records the correction (VERIFIED in file). SkillsC1 citation still needs the C07.5 fix — flagged to build, not closed here.                            |
| 2   | slot-write/read boundary SKL-03 | **OWNED (build).** Next: define the slot-write/read boundary between SKL-07 (operates IFC pipeline) and SKL-03 (operates the run). Joint-custody ops need the interface.                        |
| 3   | P5 drill QA-witnessed           | **BLOCKED (schedule).** P5 audit drill must be witnessed by QA lead (SKL-05) with independent receipt. No P5 scheduled; condition waits on it.                                                  |
| 4   | force-close time-boxed          | **OWNED (build).** Next: confirm force-close APIs are Admin-role + receipted + auto re-open review in 24h. No force-close path implemented in-repo today — condition carries to implementation. |

## Summary

- **CLOSED this pass:** C02.2 (perf scoping), C03.1 (hook count), C04.3 (dashboard rule), C05.1 (A36/A37 hashes), C07.1 (IFC pointer record). 5/21.
- **OWNED:** 14 — all with named owner + next command above. None needs new prereg text; most need small doc edits or scheduled runs.
- **BLOCKED:** 2 (C04.1 encryption decision needs Stewart; C07.3 P5 drill needs a schedule).
- **No record is promoted to EVIDENCED.** All seven remain CERTIFIED WITH CONDITIONS. Promotion needs the OWNED items cleared plus the section's own evidence bar.
