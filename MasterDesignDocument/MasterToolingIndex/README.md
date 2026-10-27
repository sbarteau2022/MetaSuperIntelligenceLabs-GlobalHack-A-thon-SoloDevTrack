# MasterToolingIndex — Tooling Certification Index for Ares SpecOps Agentic Lab

**Status.** PROPOSED (companion to FROZEN `MasterSkillsIndex.md`). **Parent:** Master Design Document index (scaffold appendix).
**Principle.** One certified tooling record per necessary skill: `ToolingC0X.md` certifies `SKL-0X`. No tooling record without a parent skill; no parent skill without a tooling record once its gates go live.
**Naming.** `ToolingC0X` = tooling certification for `SKL-0X` — the `C0X` mirrors the skill number, NOT the C-series design categories (C01–C35).

---

## Tooling Record Tree (7 Certified)

```
MasterToolingIndex
├── ToolingC01 → SKL-01 Formal Specification & Model Checking
│   ├── TOOL-01.01 tlaplus (TLA+ Toolbox, spec authoring + parsing)
│   ├── TOOL-01.02 apalache (symbolic model checker)
│   ├── TOOL-01.03 tlc (explicit-state model checker)
│   ├── TOOL-01.04 kind2 (Lustre/SMT contract checker)
│   └── TOOL-01.05 Rust-VC extractor (CAPABILITY certified; `cargo-contract` NAME withdrawn — ink! collision, substitute Kani/Creusot-class at implementation)
├── ToolingC02 → SKL-02 Systems Programming & Kernel Fusion
│   ├── TOOL-02.01 rustc + cargo (pinned toolchain, ADR-23)
│   ├── TOOL-02.02 mlx + metal-rs (Metal shader path)
│   ├── TOOL-02.03 cargo-asm (instruction inspection, claim evidence)
│   ├── TOOL-02.04 instruments + powermetrics (profiling + energy, mains-state gated)
│   └── TOOL-02.05 criterion (benchmark harness, baseline-bound)
├── ToolingC03 → SKL-03 Agentic Runtime Orchestration
│   ├── TOOL-03.01 cargo-xtask-manifest (compile-in manifest builder, DAG + effect monotonicity)
│   ├── TOOL-03.02 hook-dispatch (23-moment runtime, veto-immune audit hooks)
│   ├── TOOL-03.03 seatbelt-profile-gen (sandbox profiles from shared constants)
│   ├── TOOL-03.04 autorunner-admit (admission + stealing arbiter, backpressure-first)
│   └── TOOL-03.05 brigade-project (fiber projection/dissolution, weight_copies attestation)
├── ToolingC04 → SKL-04 Observability & Telemetry Engineering
│   ├── TOOL-04.01 ledger-append (append-only writer, Merkle sealer, privacy fail-closed)
│   ├── TOOL-04.02 telemetry-proc (separate-PID health + session supervisor)
│   ├── TOOL-04.03 prom-export (Prometheus text, cardinality guard at registration)
│   ├── TOOL-04.04 rollup-daily (neural-map aggregation, aggregates-only schema)
│   └── TOOL-04.05 privacy-fuzz (A23 operator, 10k fuzzed emits, seeded)
├── ToolingC05 → SKL-05 Verification & Ablation Framework (meta-skill, stricter bar)
│   ├── TOOL-05.01 prereg-file (C11.3 hash filing + C09.9 checklist audit)
│   ├── TOOL-05.02 ablation-run (baseline/treatment executor + judge binder, Lan-DeMets interim)
│   ├── TOOL-05.03 gate-exec (Day-gate/artifact-gate executor, no-leapfrog)
│   ├── TOOL-05.04 monte-carlo-sched (PERT over C04.10 DAG, seeded, Day 1–2 exit)
│   └── TOOL-05.05 judge-drift-monitor (C06.16/A37 operator, incl. self-watch)
├── ToolingC06 → SKL-06 Data & Knowledge Engineering
│   ├── TOOL-06.01 bm25-retriever (deterministic baseline, fixed k1/b, tie-break)
│   ├── TOOL-06.02 ingest-pipeline (tiered CRDT ingest CANONICAL/VETTED/UNVETTED)
│   ├── TOOL-06.03 eval-harness-A3 (labeled evaluation, per-collection deltas, power-gated)
│   ├── TOOL-06.04 claim-lint (claim-label + evidence linter, grandfather-aware)
│   └── TOOL-06.05 migration-audit (missing-doc + crosslink + five-file enforcer)
└── ToolingC07 → SKL-07 Security, Sandboxing & Compliance (ship-blocker skill)
    ├── TOOL-07.01 secret-scan (literal-secret + baked-path grep gate, content-blind output)
    ├── TOOL-07.02 seatbelt-exec (sandbox confinement operator, generated profiles only)
    ├── TOOL-07.03 ifc-sanitize (C07.5 pipeline operator — NOT C07.8; versioned classifier)
    ├── TOOL-07.04 kill-switch-drill (C10.8 drill executor, loaded + witnessed at P5)
    └── TOOL-07.05 compliance-smoke (Gate-3 smoke orchestrator, 0 FAIL, skips recorded)
```

---

## Tooling Record ↔ Skill ↔ Gate ↔ Ablation Traceability

| Tooling Record | Skill | Day Gates → Artifact Gates (C15.9 dual) | Ablations |
|---|---|---|---|
| `ToolingC01.md` | SKL-01 | G1→C15.6/C15.1, G5→C15.5, G6→C15.8 | A1–A10 preconditions, A22, A25 |
| `ToolingC02.md` | SKL-02 | G3→C15.1, G4→C15.2/C15.4, G8→C15.7/C15.8 | A1, A31–A35 (+A36 parents) |
| `ToolingC03.md` | SKL-03 | G3→C15.1/C15.2, G4→C15.2/C15.4, G6/G7→C15.8 | A11–A15, A17–A21 (A16 shared w/ SKL-06) |
| `ToolingC04.md` | SKL-04 | G5→C15.5, G6/G7→C15.8, G8→C15.7/C15.8 | A22–A25 |
| `ToolingC05.md` | SKL-05 | G1–G8 → C15.1–C15.8 (ALL) | A1–A37 (ALL, incl. A36/A37) |
| `ToolingC06.md` | SKL-06 | G3→C15.1/C15.2, G4→C15.2/C15.4, G5→C15.5, G6→C15.8 | A3, A8, A26, A27 (A16 shared w/ SKL-03) |
| `ToolingC07.md` | SKL-07 | G2→C15.1, G6/G7→C15.8, G8→C15.7 | A10, A23 (joint), A22 (reads) |

---

## Tool ID Namespace Convention

```
TOOL-<Skill>.<Sequence>
```

Examples:
- `TOOL-01.02` = SKL-01 → apalache → second tool of the formal-spec record
- `TOOL-03.04` = SKL-03 → autorunner-admit → fourth tool of the runtime record
- `TOOL-07.03` = SKL-07 → ifc-sanitize → third tool of the security record

Future tool granularity (sub-tools, variants) extends as `TOOL-<Skill>.<Sequence>.<Sub>` — never by inventing a new top-level record without a parent skill.

---

## Certification Standing (per record)

| Record | Verdict | Open Conditions |
|---|---|---|
| ToolingC01 | CERTIFIED WITH CONDITIONS | Named workflows RB-SPEC/RB-* do not exist in C22; TOOL-01.05 name withdrawn (substitute at implementation) |
| ToolingC02 | CERTIFIED WITH CONDITIONS | Nightly-SIMD vs `--locked` gating decision; `perf` scoped to H100 replica; A31/A32 fuzz-corpus seeds in prereg |
| ToolingC03 | CERTIFIED WITH CONDITIONS | Hook count audited to 23 (P-CORR-1); veto-immune audit hooks before RB-RUNTIME-02 executable; protobuf schemas generated not hand-written |
| ToolingC04 | CERTIFIED WITH CONDITIONS | Ledger encryption-at-rest decision before P5; RingBuffer labeled LOSSY in UI; dashboards never cited in claims |
| ToolingC05 | CERTIFIED WITH CONDITIONS (meta-skill) | A36/A37 hashes empty — no runs until filed; framework tools self-gated (Gate 2+6); Monte Carlo at Day 1–2 exit with seed |
| ToolingC06 | CERTIFIED WITH CONDITIONS | A8 per-tier round-trips; A3 epsilon power-justified + per-collection; claim-linter grandfather rule; `arrow`/`parquet` consumer stated or removed |
| ToolingC07 | CERTIFIED WITH CONDITIONS (ship-blocker) | IFC pointer is C07.5 not C07.8 (P-CORR-16); slot-write/read boundary with SKL-03; P5 drill QA-witnessed; force-close time-boxed |

---

## Discipline Rules (Non-Negotiable — mirrors MasterSkillsIndex §Discipline)

1. **No tool outside a certified record** may be cited as evidence for a gate or ablation.
2. **Every new tool proposal** must: (a) map to a `TOOL-<Skill>.<Sequence>` slot in its parent record, (b) cite the gate(s) and ablation(s) it serves, (c) name its grammar mask + parser requirement explicitly (the 7-field spec is mandatory, not advisory).
3. **Record boundaries are fixed** for the sprint. New tools extend existing records; new records require a new skill (which requires ADR + necessity review).
4. **Cross-record tool reuse** (e.g., SKL-02 consuming TOOL-01.01 outputs) is recorded as a dependency edge, not a re-certification.
5. **Draft code is structural** — conveys scope/shape/idea for the implementation language decision; it is not shippable code and never cited as behavior evidence.

---

## Final-Pass Certification (2026-10-26)

- All seven tooling records (`ToolingC01`–`ToolingC07`) were re-audited against every C01–C35
  category via the skill-to-category traceability matrix.
- Result: no category is orphaned from a tooling record; no additional tool record is required for
  coverage completeness.
- Necessity (not sufficiency) is the standing verdict for every TOOL-XX.YY entry. Records remain
  "CERTIFIED WITH CONDITIONS" only where an explicit open condition is logged; those conditions
  are tracked and are not cleared silently.
- No tool outside a certified record is cited as evidence for a gate or ablation.
