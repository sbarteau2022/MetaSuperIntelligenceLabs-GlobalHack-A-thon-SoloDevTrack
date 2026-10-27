# C32 — Implementation Order / Sprint Plan

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

10-day sprint sequence with PERT distributions per day, owner assignment, gate dependencies, and Monte Carlo schedule validation (P10/P50/P90).

## Subcategories (broad topics)

- C32.1 Day 1–2: Freeze first-principles constraints + design
- C32.2 Day 3: Wire internal scaffolds (hooks, skill tree, lazy MCP, compaction) — backend
- C32.3 Day 4–5: RAG + memory + ingest for all lanes; agent tool loop + approval; streaming
- C32.4 Day 6–7: Reliability smoke + browser E2E green; docs + claims register
- C32.5 Day 8–9: Model serving (Atlas/TwisterAttention/KV cache plug-in) in rust-server
- C32.6 Day 10: Compliance/observability audit + go/no-go
- C32.7 Post-sprint enterprise phases (P6)
- C32.8 Owner/dependency/exit-criteria mapping per day

---

## Canonical Decisions (frozen for design)

| Decision                                                                      | Rationale                                      |
| ----------------------------------------------------------------------------- | ---------------------------------------------- |
| **10-day sprint** with fixed end date; no scope creep                         | Time-boxing forces prioritization              |
| **PERT distributions** per task (optimistic/expected/pessimistic)             | Realistic uncertainty; enables Monte Carlo     |
| **Owner assignment** per task; single accountable owner                       | Clear accountability; no diffusion             |
| **Gate dependencies** from C15; task cannot start until gate dependencies met | Verification-first; no untested code ships     |
| **Monte Carlo validation** (10,000 runs) produces P10/P50/P90                 | Schedule risk quantified; go/no-go data-driven |
| **Daily exit criteria** = gate pass + demo + claims register update           | Measurable progress; evidence over claims      |

---

## Sprint Plan with PERT Distributions

### Day 1–2: Design Freeze (C01–C03, C08, C15, C16, C36)

| Task                                               | Owner     | Optimistic | Expected | Pessimistic | Gate Dependencies |
| -------------------------------------------------- | --------- | ---------- | -------- | ----------- | ----------------- |
| C01 Scope/Goals finalized                          | Architect | 2h         | 4h       | 8h          | —                 |
| C02 Differences vs ad-hoc                          | Architect | 2h         | 4h       | 6h          | C01               |
| C03 First principles backbone                      | Architect | 3h         | 6h       | 12h         | C01               |
| C08 Framework reqs (TLA+ specs)                    | Architect | 4h         | 8h       | 16h         | C03               |
| C15 Verification gates defined                     | QA Lead   | 2h         | 4h       | 8h          | C08               |
| C16 Threat model + RBAC                            | Security  | 3h         | 6h       | 12h         | C08               |
| C36 ADRs frozen                                    | Architect | 1h         | 2h       | 4h          | C08, C16          |
| Pre-registration hashes (A1–A10)                   | QA Lead   | 1h         | 2h       | 4h          | C09               |
| **Day 1–2 Exit Criteria**                          |           |            |          |             |                   |
| All C01–C03, C08, C15, C16, C36 Design.md PROPOSED |           |            |          |             |                   |
| Pre-registration hashes filed for A1–A10           |           |            |          |             |                   |
| TLA+ specs model-check (0 counterexamples)         |           |            |          |             |                   |
| Threat model STRIDE complete                       |           |            |          |             |                   |

### Day 3: Internal Scaffolds (C14, C28, C29, C24, C25)

| Task                                                 | Owner   | Optimistic | Expected | Pessimistic | Gate Dependencies |
| ---------------------------------------------------- | ------- | ---------- | -------- | ----------- | ----------------- |
| C14.1 Workspace resolution (ATOMA_WORKSPACE_ROOT)    | Backend | 2h         | 4h       | 8h          | C20               |
| C14.6 Boot-time skill scan + manifest validation     | Backend | 3h         | 6h       | 12h         | C28               |
| C14.8 Compile-in manifests (skills, tools)           | Backend | 2h         | 4h       | 8h          | C28, C29          |
| C28 Skill registry + DAG + hooks                     | Backend | 4h         | 8h       | 16h         | C14.1             |
| C29 Tool registry + effect system                    | Backend | 4h         | 8h       | 16h         | C14.1             |
| C24 Context budgeting + compaction                   | Backend | 3h         | 6h       | 12h         | C07               |
| C25 Hole substrate + seatbelt                        | Backend | 4h         | 8h       | 16h         | C16               |
| **Day 3 Exit Criteria**                              |         |            |          |             |                   |
| Cargo test --locked (0 failures)                     |         |            |          |             |                   |
| Boot scan validates manifests against wrangler.jsonc |         |            |          |             |                   |
| Skill DAG acyclicity verified                        |         |            |          |             |                   |
| Tool effect monotonicity verified                    |         |            |          |             |                   |
| Hole seatbelt profile loads                          |         |            |          |             |                   |
| Compaction round-trip test passes                    |         |            |          |             |                   |

### Day 4–5: RAG + Memory + Agent Loop (C26, C07, C30, C18, C19)

| Task                                                | Owner    | Optimistic | Expected | Pessimistic | Gate Dependencies |
| --------------------------------------------------- | -------- | ---------- | -------- | ----------- | ----------------- |
| C26 RAG ingest pipeline (BM25 + dense opt-in)       | Backend  | 4h         | 8h       | 16h         | C17, C29          |
| C26 CRDT ingest + A3 eval harness                   | Backend  | 3h         | 6h       | 12h         | C26               |
| C07 Memory state (CRDT summaries, causal broadcast) | Backend  | 4h         | 8h       | 16h         | C24, C25          |
| C07 Prompt-injection IFC (C07.8)                    | Backend  | 3h         | 6h       | 12h         | C07, C16          |
| C30 Lane config + autorunner                        | Backend  | 4h         | 8h       | 16h         | C28, C29          |
| C30 Subagent loops + branch isolation               | Backend  | 4h         | 8h       | 16h         | C25, C30          |
| C18 API contracts (OpenAPI + session types)         | Backend  | 3h         | 6h       | 12h         | C08               |
| C19 Frontend route chunking + state machine         | Frontend | 4h         | 8h       | 16h         | C18               |
| C19 WCAG 2.1 AA audit                               | Frontend | 2h         | 4h       | 8h          | C19               |
| **Day 4–5 Exit Criteria**                           |          |            |          |             |                   |
| Ingest → query round-trip with receipt (A8)         |          |            |          |             |                   |
| Cross-lane memory recall test (A9)                  |          |            |          |             |                   |
| Prompt-injection red team denied (A10)              |          |            |          |             |                   |
| Autorunner admits packets under backpressure        |          |            |          |             |                   |
| Subagent depth/budget caps enforced                 |          |            |          |             |                   |
| Branch isolation: cross-branch reads denied         |          |            |          |             |                   |
| OpenAPI spec validates; MCP lazy sessions work      |          |            |          |             |                   |
| Frontend loads; Playwright smoke passes             |          |            |          |             |                   |

### Day 6–7: Reliability + Docs + Claims (C22, C23, C13, C33, C09)

| Task                                              | Owner | Optimistic | Expected | Pessimistic | Gate Dependencies |
| ------------------------------------------------- | ----- | ---------- | -------- | ----------- | ----------------- |
| C22 Fault tree + 7 runbooks                       | SRE   | 3h         | 6h       | 12h         | C16               |
| C22 Recovery smoke script                         | SRE   | 2h         | 4h       | 8h          | C22               |
| C23 Risk register + Monte Carlo (P10/P50/P90)     | PM    | 2h         | 4h       | 8h          | C16, C22          |
| C13 Counterfactual necessity tests                | QA    | 3h         | 6h       | 12h         | C15               |
| C13 Circular stats for toroidal                   | QA    | 2h         | 4h       | 8h          | C13               |
| C13 TLA+ model checking                           | QA    | 2h         | 4h       | 8h          | C08               |
| C33 Five-file set enforcement + claim labeling    | Docs  | 2h         | 4h       | 8h          | All               |
| C33 Missing-doc migration status                  | Docs  | 1h         | 2h       | 4h          | C33               |
| C09 Claims register JSON + ablation hashes        | QA    | 2h         | 4h       | 8h          | A1–A10            |
| **Day 6–7 Exit Criteria**                         |       |            |          |             |                   |
| All 7 runbooks executable; recovery smoke passes  |       |            |          |             |                   |
| Monte Carlo schedule: P90 ≤ 10 days               |       |            |          |             |                   |
| Counterfactual necessity: 0 false positives       |       |            |          |             |                   |
| Toroidal worst-arc reported (not average)         |       |            |          |             |                   |
| TLA+ model check: 0 counterexamples               |       |            |          |             |                   |
| All workstreams have 5-file set; claims labeled   |       |            |          |             |                   |
| Claims register JSON valid; ablation hashes match |       |            |          |             |                   |

### Day 8–9: Model Serving (C05, C12, C35, C06)

| Task                                           | Owner  | Optimistic | Expected | Pessimistic | Gate Dependencies |
| ---------------------------------------------- | ------ | ---------- | -------- | ----------- | ----------------- |
| C05 Atlas/Hyperbolic KV (Lorentz + VP-tree)    | ML Eng | 6h         | 12h      | 24h         | C35               |
| C12 Kernel fusion + ILP scheduling             | ML Eng | 4h         | 8h       | 16h         | C05               |
| C12 Hyperbolic prefix cache + geodesic pruning | ML Eng | 4h         | 8h       | 16h         | C05, C12          |
| C35 Lorentz ↔ Poincaré isometry (4.4e-9)       | ML Eng | 2h         | 4h       | 8h          | C05               |
| C35 Picard idempotence + VP-tree retrieval     | ML Eng | 2h         | 4h       | 8h          | C05               |
| C06.12 Holonomy Brigade (weight copies = 1)    | ML Eng | 4h         | 8h       | 16h         | C05, C30          |
| C06.1 Anti-lobotomy basin folding              | ML Eng | 3h         | 6h       | 12h         | C05, C24          |
| C12 ACFT/TwisterAttention plug-in              | ML Eng | 4h         | 8h       | 16h         | C05, C12          |
| **Day 8–9 Exit Criteria**                      |        |            |          |             |                   |
| Isometry test: 4.4e-9 over 900 pairs           |        |            |          |             |                   |
| Picard idempotence: 100%                       |        |            |          |             |                   |
| VP-tree retrieval: 13/13 tests pass            |        |            |          |             |                   |
| Full-chain qualification: 13/13 pass           |        |            |          |             |                   |
| Atomic vault survival: 100% recovery           |        |            |          |             |                   |
| Concurrent turns: 8 threads, no leakage        |        |            |          |             |                   |
| Brigade weight copies = 1 under N=8            |        |            |          |             |                   |
| ACFT throughput ≥ baseline (A1)                |        |            |          |             |                   |

### Day 10: Compliance Audit + Go/No-Go (C15, C16, C31, C21)

| Task                                           | Owner    | Optimistic | Expected | Pessimistic | Gate Dependencies |
| ---------------------------------------------- | -------- | ---------- | -------- | ----------- | ----------------- |
| C15 Gate 1–8 all pass                          | QA Lead  | 2h         | 4h       | 8h          | All prior         |
| C16 Compliance smoke (0 FAIL)                  | Security | 1h         | 2h       | 4h          | C16               |
| C31 Observability audit (privacy, cardinality) | SRE      | 2h         | 4h       | 8h          | C31               |
| C21 Performance budgets (p50/p95/p99, energy)  | SRE      | 2h         | 4h       | 8h          | C12, C21          |
| C21 Toroidal worst-arc reporting               | SRE      | 1h         | 2h       | 4h          | C21               |
| Build receipt (git, toolchain, lock hashes)    | Release  | 1h         | 2h       | 4h          | C14               |
| Go/No-Go decision                              | PM       | 1h         | 2h       | 4h          | All above         |
| **Day 10 Exit Criteria (GO)**                  |          |            |          |             |                   |
| All 8 verification gates PASS                  |          |            |          |             |                   |
| Compliance smoke: 0 FAIL                       |          |            |          |             |                   |
| Privacy audit: 0 violations                    |          |            |          |             |                   |
| Performance: p99 ≤ budget; energy ≤ budget     |          |            |          |             |                   |
| Toroidal worst-arc documented                  |          |            |          |             |                   |
| Build receipt recorded                         |          |            |          |             |                   |
| Claims register: all A1–A10 + A11–A25 status   |          |            |          |             |                   |

---

## Monte Carlo Schedule Validation

**Method**: 10,000 simulation runs sampling from PERT distributions (Beta-PERT).

**Results** (to be filled after simulation):

| Metric                        | Value    |
| ----------------------------- | -------- |
| P10 (10% chance finish by)    | TBD days |
| P50 (median)                  | TBD days |
| P90 (90% chance finish by)    | TBD days |
| Probability(finish ≤ 10 days) | TBD%     |
| Critical path                 | TBD      |

**Go/No-Go Rule**: Sprint proceeds only if `P90 ≤ 10 days` AND `Probability(finish ≤ 10 days) ≥ 80%`.

---

## Post-Sprint Enterprise Phases (P6)

| Phase | Duration | Focus                                                     |
| ----- | -------- | --------------------------------------------------------- |
| P6.1  | 2 weeks  | Skill marketplace + hot-reload; tool version negotiation  |
| P6.2  | 2 weeks  | Distributed Atlas sync (C06.10); multi-node ledger        |
| P6.3  | 2 weeks  | Dynamic brigade radius; self-tuning curvature (C06.7)     |
| P6.4  | 2 weeks  | Advanced approval workflows; policy-as-code               |
| P6.5  | 2 weeks  | Cross-hardware replication (C09.4); benchmark publication |

---

## Owner/Dependency/Exit-Criteria Matrix

| Day | Owner                          | Dependencies                 | Exit Criteria                                               |
| --- | ------------------------------ | ---------------------------- | ----------------------------------------------------------- |
| 1–2 | Architect, Security, QA        | —                            | Design freeze; pre-reg hashes; TLA+ clean                   |
| 3   | Backend (2)                    | C20, C28, C29                | Cargo test; boot scan; DAG/effect verified                  |
| 4–5 | Backend (2), Frontend          | C17, C24, C25, C26, C28, C29 | Round-trips; autorunner; subagent; branch; API; UI          |
| 6–7 | SRE, PM, QA, Docs              | C13, C15, C16, C22, C23, C33 | Runbooks; Monte Carlo; necessity; TLA+; 5-file sets; claims |
| 8–9 | ML Eng (2)                     | C05, C12, C30, C35           | Isometry; Picard; VP-tree; full-chain; brigade; ACFT        |
| 10  | QA, Security, SRE, Release, PM | All                          | Gates 1–8; compliance; privacy; perf; build receipt; GO     |

---

## Verification Gates (C15) Mapping

| Sprint Day | Gates Exercised                                                                           |
| ---------- | ----------------------------------------------------------------------------------------- |
| 1–2        | Gate 1 (Design Review), Gate 2 (Static Analysis)                                          |
| 3          | Gate 2 (Static Analysis), Gate 3 (Unit Tests)                                             |
| 4–5        | Gate 3 (Unit Tests), Gate 4 (Integration Tests), Gate 5 (Contract Tests)                  |
| 6–7        | Gate 4 (Integration), Gate 5 (Contract), Gate 6 (Property Tests), Gate 7 (Chaos/Recovery) |
| 8–9        | Gate 3 (Unit), Gate 4 (Integration), Gate 8 (Performance/Benchmarks)                      |
| 10         | Gate 1–8 (All); Compliance Smoke                                                          |

---

## Open Questions / Gaps

- Exact PERT parameters per task (need team estimation session).
- Parallelization limits (how many tasks truly parallel per day).
- Buffer for unknown unknowns (currently 0; consider 10% contingency).

---

## Contributor Deepening — Muse (2026-10-26)

### C32.7 Sprint-plan consistency audit (depth: the plan cites its own dependencies)

[VERIFIED — full section read; INFERRED where marked — audit against categories read so far]:

1. Stale pointer (same class as C11.7/C16.9): the Day 4–5 table cites "C07 Prompt-injection IFC (C07.8)" — but C07.8 is injection-budget arbitration (my 2026-10-26 addition); the sanitization pipeline is C07.5. Correct the row to C07.5; recommend a C36 correction entry.
2. Gate-column drift: Day 3 exit lists "Cargo test --locked" and Day 10 lists "C15 Gate 1–8 all pass" — but the per-day task tables use "Gate Dependencies" with values like C20/C28/C07 (categories, not gates). Per C15.9, every row needs BOTH a Day gate (G-schedule) and a C15.x artifact gate. The "Verification Gates (C15) Mapping" table at the end uses Scheme B names (Gate 1–8 = Day gates) while C08.7/C15.9 require C15.x IDs. Rewrite that table with both columns at the README/VerificationCommands pass; recording the requirement here.
3. Missing tasks (gaps the plan does not schedule):
   - No task for C09.9 prereg content audit (preregs for A1–A10 filed Day 1–2 as hashes — but content completeness per the 8-item checklist is unscheduled).
   - No task for kill-switch drill #1 (C10.8 requires a drill per phase gate; P2 exit (Day 4–5) is the first gate where ACFT-adjacent code exists — schedule it there).
   - No task for C19.8 schema-generator (frontend schema must be generated from backend — Day 4–5 with C18/C19).
   - No task for C28.7 hook-count audit (23 vs 25 — Day 3 with C28).
   - No task for A36/A37 preregistration (C06.15/C06.16 — Day 8–9 with C06).
   - Contingency buffer is 0 with "consider 10%" noted — a plan that considers but does not carry buffer carries 0 buffer. Either schedule it (one unassigned half-day at Day 9) or record the accepted risk in C23 (R5 scope-creep residual rises).
4. Owner naming: "Backend (2)", "ML Eng (2)" are capacity statements (2 people), not assignments (C14.13 same finding). Unnamed-capacity rows are PROPOSED staffing until names attach; flag at G1.
5. PERT table exists but Monte Carlo outputs are TBD ("to be filled after simulation") — the G1 gate cannot evaluate P90 ≤ 10 days against TBD. The simulation MUST run at Day 1–2 exit (inputs exist by then) with seed recorded (C23.7), or the go/no-go rule (ADR-20) is unenforceable for the whole sprint.

What was NOT checked here: C33/C34/C35/C36 day placements (verify: C33 Day 6–7 present; C34/C35/C36 placement in plan not yet audited — confirm no category lacks a scheduled day).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Implementation Order (Original)

1. Day 1–2: Freeze first-principles constraints + design (done for Difference 1).
2. Day 3: Wire remaining internal scaffolds (hooks, skill tree, lazy MCP, compaction) — backend.
3. Day 4–5: RAG + memory + ingest for all lanes (backend done this pass).
4. Day 6–7: Agent tool loop + reliability smoke + browser E2E green.
5. Day 8–9: Model serving (Atlas / TwisterAttention / KV cache plug-in) in rust-server.
6. Day 10: Compliance/observability audit + go/no-go.
