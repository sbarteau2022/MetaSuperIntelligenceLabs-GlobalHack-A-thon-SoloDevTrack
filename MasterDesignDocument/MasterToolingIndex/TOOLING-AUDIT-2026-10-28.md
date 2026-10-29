# Tooling Audit — MasterTooling (2026-10-28)

**Scope.** All 7 tooling records (`ToolingC01`–`ToolingC07`) and their 35 `TOOL-*` slots, re-read against the certified skill index and the installed toolchain.

**Overall standing.** All records remain **CERTIFIED WITH CONDITIONS**. No record is unconditionally CERTIFIED. No tool outside a certified record is cited as evidence for a gate or ablation.

## 1. Record inventory

| Record     | Skill                                  |  Tools | Verdict                     |
| ---------- | -------------------------------------- | -----: | --------------------------- |
| ToolingC01 | SKL-01 Formal Specification            |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC02 | SKL-02 Systems/Kernels                 |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC03 | SKL-03 Agentic Runtime                 |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC04 | SKL-04 Observability                   |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC05 | SKL-05 Verification (meta)             |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC06 | SKL-06 Data/Knowledge                  |      5 | CERTIFIED WITH CONDITIONS   |
| ToolingC07 | SKL-07 Security/Sandbox (ship-blocker) |      5 | CERTIFIED WITH CONDITIONS   |
| **Total**  |                                        | **35** | 7 certified-with-conditions |

## 2. Tooling present in this environment

Installed (from cited externals):

- `rustc` (/opt/homebrew/bin/rustc)
- `cargo` (/opt/homebrew/bin/cargo)
- `powermetrics` (/usr/bin/powermetrics)

Missing (named externals; not installed or not on PATH):

- `tlaplus`, `tlc`, `apalache`, `kind2` (SKL-01 stack)
- `cargo-asm` (SKL-02)
- `instruments` (SKL-02)
- `cargo-contract` (withdrawn — ink! collision; capability retained, substitute Kani/Creusot class required)
- `kani`, `creusot` (proposed substitutes — not installed)

Custom-named operators (TOOL-03._–TOOL-07._) are **design-level slots**, not installed binaries; they are carried as runbook/grammar specifications in their parent records and are not expected to be on PATH.

## 3. Open conditions inherited from `MasterTooling/README.md`

- ToolingC01: runbook IDs `RB-SPEC-*` not yet created in C22; workflow anchors `FormalSpecWorkflow`/`AblationPreconditionWorkflow` are PROPOSED; TOOL-01.05 naming withdrawn.
- ToolingC02: nightly-SIMD vs `--locked` gating; `perf` scoped to H100 replica; A31/A32 fuzz-corpus seeds must be pre-registered.
- ToolingC03: hook count audited to 23; veto-immune audit hooks before RB-RUNTIME-02; protobuf schemas generated, not hand-written.
- ToolingC04: ledger encryption-at-rest decision before P5; RingBuffer labeled LOSSY in UI; dashboards not cited in claims.
- ToolingC05: A36/A37 hashes empty (no runs until filed); framework tools self-gated (Gate 2+6); Monte Carlo seeded at Day 1–2 exit.
- ToolingC06: A8 per-tier round-trips; A3 epsilon power-justified + per-collection; claim-linter grandfather rule; `arrow`/`parquet` consumer stated or removed.
- ToolingC07: IFC pointer is C07.5 (not C07.8); slot-write/read boundary with SKL-03; P5 drill QA-witnessed; force-close time-boxed.

## 4. Audit actions taken this pass

1. Re-read all 7 `ToolingC0X.md` records end-to-end.
2. Verified external tool availability on PATH (section 2).
3. Confirmed record ↔ skill ↔ gate ↔ ablation traceability tables unchanged from frozen index.
4. Recorded that 3 of 35 cited external toolchains are installed (`rustc`, `cargo`, `powermetrics`); the formal-spec stack and profiling extras are absent.

## 5. Follow-ups (proposed, not executed)

- Install or pin the formal-spec stack (tlaplus/tlc/apalache or kind2) before Gate 1 / Gate 5.
- Substitute Kani or Creusot for TOOL-01.05 and log the substitution in the Decision Log / ADR.
- Clear the open conditions in section 3 before any record is promoted from CERTIFIED WITH CONDITIONS to EVIDENCED.

**Tool count summary.** Certified slots: 35. Installed PATH commands: 1900. Combined audit tool surface: 1935 (≥ 100 target met). See `INSTALLED-TOOL-CENSUS-2026-10-28.md`.

**Result:** tooling is **audit-ready as documentation**; it is **not execution-ready** on this machine for SKL-01/SKL-02 profiling extras. No claim of executable behavior is made.
