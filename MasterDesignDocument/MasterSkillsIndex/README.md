# MasterSkillsIndex — Skill Taxonomy for Ares SpecOps Agentic Lab

**Status.** FROZEN. **Parent:** Master Design Document index.  
**Principle.** Tree-like categories consistent with C01–C36 structure. Discipline is non-negotiable.  
**Organization Logic.** 7 top-level categories derived from **workstream clusters** in the Master Design Document, not from external precedents. Each category maps to a contiguous block of Cxx categories and owns specific C15 gates / C09 ablations.

---

## Category Tree (7 Top-Level)

```
MasterSkillsIndex
├── 01 Formal Methods & Specification
│   ├── 01.1 TLA+ / PlusCal Specification
│   ├── 01.2 Refinement Proof Engineering
│   ├── 01.3 Verification Condition Generation
│   └── 01.4 Ablation Precondition Formalization
│
├── 02 Systems Programming & Kernels
│   ├── 02.1 Rust/MLX Kernel Development
│   ├── 02.2 Fixed-Point Arithmetic (Q32.31)
│   ├── 02.3 Metal/GPU Shader Integration
│   ├── 02.4 Kernel Fusion Algebra (ILP Scheduling)
│   └── 02.5 Hyperbolic Geometry Kernels (Lorentz/VP-tree/Picard)
│
├── 03 Agentic Runtime & Orchestration
│   ├── 03.1 Skill Registry (DAG + Hooks + Manifest)
│   ├── 03.2 Tool Registry (Effect Lattice + Schemas)
│   ├── 03.3 Lane Configuration & Autorunner
│   ├── 03.4 Subagent Loops (Holes + Depth/Budget Caps)
│   ├── 03.5 Brigade Orchestration (Fiber Projection + Weight Invariant)
│   ├── 03.6 Branch Isolation (Worktrees + Hole Sandbox)
│   └── 03.7 Cross-Lane Comms (Geodesic Bridges + Interference)
│
├── 04 Observability & Telemetry
│   ├── 04.1 Event Ledger (SQLite + Merkle + WAL)
│   ├── 04.2 Telemetry Process (Separate PID + Honest Availability)
│   ├── 04.3 Prometheus Metrics (Text Format + Cardinality Bounds)
│   ├── 04.4 Taps (Structured Event Streams)
│   ├── 04.5 Receipts & Fingerprints (Merkle Paths)
│   ├── 04.6 Neural Map Roll-Up (Daily Aggregation)
│   └── 04.7 Privacy Enforcement (Non-Interference Schema)
│
├── 05 Verification & Quality Gates
│   ├── 05.1 Ablation Framework (Prereg + Group Sequential + Bayes)
│   ├── 05.2 Necessity Testing (Counterfactual + Circular Stats)
│   ├── 05.3 Verification Gates (8-Gate Pipeline + Artifact Verifiers)
│   ├── 05.4 Monte Carlo Schedule (PERT + P10/P50/P90)
│   ├── 05.5 Compliance Smoke (Secrets + Paths + Privacy + Claims)
│   └── 05.6 Build Receipts (Git + Toolchain + Lock Hashes)
│
├── 06 Data & Knowledge Engineering
│   ├── 06.1 RAG Pipeline (BM25 + Honest Dense Opt-In)
│   ├── 06.2 CRDT Ingest + Receipt Round-Trip
│   ├── 06.3 A3-Labeled Evaluation Harness
│   ├── 06.4 SQL Schema + Forward-Only Migrations
│   ├── 06.5 OpenAPI 3.1 + Session Types
│   ├── 06.6 Documentation Governance (5-File Set + Claim Labeling)
│   ├── 06.7 Migration Tracking + Crosslink Validation
│   └── 06.8 MCP Lazy Sessions + Lane Discovery
│
└── 07 Security, Sandboxing & Compliance
    ├── 07.1 STRIDE Threat Model Maintenance
    ├── 07.2 Formal RBAC + Secret Slots
    ├── 07.3 Seatbelt Sandbox Profiles (5 Levels)
    ├── 07.4 Hole Isolation + Capability Subsets
    ├── 07.5 Merkle Receipts for Mutating Actions
    ├── 07.6 Kill Switch (Config Flag + Fallback + No Data Loss)
    ├── 07.7 Prompt-Injection IFC (C07.8)
    ├── 07.8 Circuit Breaker Logging
    └── 07.9 GDPR / AI Act Compliance Mapping
```

---

## Category ↔ MasterDesignDocument Mapping (Traceability)

| Skill Category | Cxx Range | Primary Categories | Gates Owned | Ablations Owned |
|---|---|---|---|---|
| **01 Formal Methods** | C03, C08, C09, C15, C16, C23, C36 | First Principles, Framework Reqs, Ablations, Gates, Security, Risk, ADR | Gate 1, Gate 5, Gate 6 | A1–A10, A22, A25 |
| **02 Systems/Kernels** | C05, C06, C12, C35 | Atlas/Hyperbolic, Frontier, Inference Engine, Geometry | Gate 3, Gate 4, Gate 8 | A1, A31–A35 |
| **03 Agentic Runtime** | C07, C14, C25, C27, C28, C29, C30, C34 | Memory, Assembly, Hole Substrate, Workflows, Skills, Tools, Lanes, Personas | Gate 3, Gate 4, Gate 6, Gate 7 | A11–A21 |
| **04 Observability** | C15, C31 | Gates, Observability | Gate 5, Gate 6, Gate 7, Gate 8 | A22–A25 |
| **05 Verification** | C09, C13, C15, C23 | Ablations, Testing, Gates, Risk | **ALL Gates (1–8)** | **ALL (A1–A35)** |
| **06 Data/Knowledge** | C17, C18, C26, C33 | Data Model, API, RAG, Docs | Gate 3, Gate 4, Gate 5, Gate 6 | A3, A8, A26, A27 |
| **07 Security/Sandbox** | C10, C16, C22, C25, C36 | Deployment, Security, Failure Modes, Hole Substrate, ADR | Gate 2, Gate 6, Gate 7, Gate 8 | A10, A23 |

---

## Skill ID Namespace Convention

```
SKL-<Category>.<Subcategory>.<Sequence>
```

Examples:
- `SKL-01.1.01` = Formal Methods → TLA+ Specification → Skill 1
- `SKL-03.4.02` = Agentic Runtime → Subagent Loops → Skill 2
- `SKL-07.3.01` = Security → Seatbelt Profiles → Skill 1

**SkillsC1.md (this folder)** uses abbreviated `SKL-XX` (01–07) for the 7 necessary top-level skills. The full taxonomy above provides the namespace for future skill granularity.

---

## Discipline Rules (Non-Negotiable)

1. **No skill outside this taxonomy** may be claimed as "necessary" for sprint execution.
2. **Every new skill proposal** must: (a) map to a leaf in this tree, (b) cite the C15 gate(s) and C09 ablation(s) it enables, (c) pass necessity review (Quality > Quantity).
3. **Category boundaries are fixed** for the sprint. Post-sprint (P6) may add leaves only — no new top-level categories without ADR.
4. **Cross-category skills** (e.g., "Rust + TLA+") are represented as **skill compositions** in runbooks, not new categories.
5. **Skill registry (C28)** uses this taxonomy for `depends_on` edges and brigade scoping.

---

## Category Design Rationale (Why 7, Not 6 or 8)

| Alternative | Rejected Because |
|---|---|
| **6 categories** (like Claude) | Merges Formal Methods (01) + Verification (05) — but 01 is *specification* (upstream), 05 is *execution* (downstream); conflating them loses traceability to C08 vs C15. Merges Systems (02) + Agentic Runtime (03) — but 02 is *kernels* (MLX/Metal), 03 is *orchestration* (skills/lanes/holes); different toolchains, different failure modes. |
| **8 categories** (split Security from Sandboxing) | Security (STRIDE/RBAC) and Sandboxing (Seatbelt/Holes) are **co-deployed** in C16/C25/C22 — same runbooks, same gate (Gate 7), same compliance smoke. Splitting creates artificial boundary. |
| **8 categories** (split Data from Knowledge) | RAG (C26) and Docs (C33) share **governance infrastructure** (claim labeling, five-file set, migration tracking, crosslink validation) — same tools (SKL-06), same gates. |
| **5 categories** (merge Observability into Verification) | Observability (C31) is **infrastructure** (always-on, separate PID); Verification (C15) is **execution** (gate-triggered). Different SLAs, different runbooks, different failure modes. |

**The 7 categories reflect the 7 distinct toolchain/SLA/runbook clusters in the Master Design Document.** This is not aesthetic — it is architectural.

---

## Quick Reference: Skill → Gate → Ablation

| Skill | Gates | Ablations |
|---|---|---|
| SKL-01 Formal Spec | 1, 5, 6 | A1–A10, A22, A25 |
| SKL-02 Systems/Kernels | 3, 4, 8 | A1, A31–A35 |
| SKL-03 Agentic Runtime | 3, 4, 6, 7 | A11–A21 |
| SKL-04 Observability | 5, 6, 7, 8 | A22–A25 |
| SKL-05 Verification | 1–8 (ALL) | A1–A35 (ALL) |
| SKL-06 Data/Knowledge | 3, 4, 5, 6 | A3, A8, A26, A27 |
| SKL-07 Security/Sandbox | 2, 6, 7, 8 | A10, A23 |

**SKL-05 (Verification) is the meta-skill** — it operates the gates that validate all other skills. It is necessary because without it, **no other skill's output can be verified**.

---

## Final-Pass Certification (2026-10-26)

- The seven necessary skills (`SKL-01`–`SKL-07`) were re-audited against every C01–C35 category.
- Result: no category is orphaned; the skill taxonomy provides coverage for all 36 category slots.
- Frontend/UI engineering remains SUFFICIENT, not necessary (per SkillsC1 necessity proof).
- No new necessary skill was identified; discipline rule "no skill outside this taxonomy without
  necessity review" is respected. Records remain FROZEN.
