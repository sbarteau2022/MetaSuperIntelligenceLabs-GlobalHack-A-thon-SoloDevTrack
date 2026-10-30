# Pre-Registration — A11–A35

**Rule.** Every ablation requires this preregistration hash to be written before the run.
An ablation run without a pre-filed hash is automatically disqualified (C11.3).
Null/negative results are recorded as first-class entries.

**Certification pass 2026-10-29 (Stewart directive: certify for smoke, top-to-bottom).**
Each section below carries the full C09.9 8-item content. File-level conventions (apply to A11–A35 unless a section overrides — identical to the A1–A10 file):

- **Seeds:** S = {20261101, 20261102, 20261103, 20261104, 20261105} (5 fixed seeds; every run records the seed in its receipt per C03.5).
- **Hardware:** primary `HW-M3MAX` (MacBook Pro M3 Max, 36GB, macOS 14, mains power, ambient °C recorded); replica `HW-M2ULTRA` (Mac Studio M2 Ultra, 96GB). Energy reported raw per machine, mains-only (C21.8 — never normalized).
- **Baseline commit rule:** baseline path = run-branch merge-base commit; exact hash recorded in the run receipt (C15.7).
- **Interim analyses:** Lan-DeMets alpha-spending ρ=2 at 25/50/75% information; stop for futility if conditional power < 0.2 (C09 protocol). Fixed-n exactness judges (binary 100%/0%) take no interim — stated per section.
- **Judge bands:** each section states `judge_band [l,u]` feeding the TOOL-05.05/A37 drift monitor.
- **Effect-size methods:** per C09 table.
- A36/A37 prereg text below is preserved verbatim from the filed revision (only the Status line advances to PREREGISTERED); shell-residue footer rows from the pre-certification revision removed — no hash was filed under that revision, so no version fork (ADR-19/ADR-22 n/a).
- **Hash-filing convention (RATIFIED 2026-10-30, Stewart):** §-text SHA-256, same convention as the A1–A10 file. Verified by recompute 2026-10-30: 27/27 MATCH (A11–A37). §A36/§A37 text below the Status-line advance is frozen at ratification; any further edit forks a new version + new hashes.
- **A36/A37 hash lineage (2026-10-30):** HEAD-committed `480a3943…`/`75e4446e…` match NO text on disk (superseded, retained in history only). Worktree footer `ca78e271…`/`eca33754…` recompute-verified over frozen text — CANONICAL from ratification.

---

## A11 — Skill Registry Integrity

- Hypothesis (falsifiable): manifest hash deterministic AND boot validation catches **100%** of binding mismatches (detection == 1.00 over mutation suite).
- Workload: `MUT-A11-v1` — n = 1000 manifest mutations (binding deletions, version skews, hash corruptions — seeded order from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica (determinism check).
- Baseline: ad-hoc skill loading (no manifest) at merge-base commit — expected detection 0 (validates suite potency).
- Judge (conjunctive): (a) detection == 1.00 (1000/1000, Wilson lower bound ≥ 0.996); (b) manifest hash byte-identical across 5-seed rebuilds. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 1000; no interim (exactness judge).
- Effect-size plan: detection rate (Wilson CI).
- Null-handling: any miss → manifest builder defect (TOOL-05.01/SKL-06), Gate 2 blocked until fix + re-preregister.
- Claims-register draft: `C09-A11-registry` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A12 — Hook Veto Correctness

- Hypothesis (falsifiable): a single veto at any moment aborts the action and emits a receipt — **0 errors** over 23 moments × {veto, non-veto} × 10 repeats (460 trials).
- Workload: `MATRIX-A12-v1` — same matrix shape as A7, veto-source = skill/tool layer (A7 covers lane layer; disjoint coverage recorded).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: manual gating at merge-base commit.
- Judge (conjunctive): (a) 460/460 correct; (b) receipt present for every veto abort. judge_band [0.99, 1.00].
- Sample-size plan: fixed 460; no interim.
- Effect-size plan: error rate (Wilson CI, target upper bound ≤ 0.008).
- Null-handling: any miss → SKL-03 hook-algebra defect, Gate 3 blocked until fix + re-preregister.
- Claims-register draft: `C09-A12-veto` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A13 — Brigade Skill Isolation

- Hypothesis (falsifiable): lane _i_ skill cannot invoke capabilities of lane _j_ without the circuit breaker — **0 successes** over 10,000 cross-lane cap attempts.
- Workload: `ATT-A13-v1` — 10,000 seeded cross-lane capability attempts (seeded order from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unisolated dispatch (breaker disabled) at merge-base commit — expected ≥ 50% success (suite potency check; run void if baseline < 50%).
- Judge (conjunctive): (a) success count == 0 (Wilson upper bound ≤ 0.0004); (b) every attempt logged + denied with receipt. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (zero-tolerance judge).
- Effect-size plan: success rate (Wilson CI).
- Null-handling: any success → isolation defect (C16 elevation class), Gate 4 blocked, fix + re-preregister.
- Claims-register draft: `C09-A13-isolation` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A14 — Tool Registry Determinism

- Hypothesis (falsifiable): `resolve_v2` deterministic — **0 non-deterministic resolutions** over 10,000 random calls.
- Workload: `CALLS-A14-v1` — 10,000 seeded random tool calls (seed from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: ad-hoc string-based dispatch at merge-base commit.
- Judge: nondeterminism count == 0 (Wilson upper bound ≤ 0.0004). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim.
- Effect-size plan: nondeterminism rate (Wilson CI).
- Null-handling: any nondeterminism → resolver defect (SKL-06/TOOL-06 family), Gate 3 blocked until fix + re-preregister.
- Claims-register draft: `C09-A14-determinism` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A15 — Effect System Soundness

- Hypothesis (falsifiable): no tool executes with an effect exceeding its declared level — **0 violations** across static analysis + dynamic hole audit.
- Workload: `EFF-A15-v1` — full tool registry × {Pure, ReadOnly, WriteLocal, WriteRemote, Destructive} matrix + 10,000 seeded dynamic hole executions.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unpoliced execution at merge-base commit — expected ≥ 1 violation class (validates audit potency).
- Judge (conjunctive): (a) static violations == 0; (b) dynamic violations == 0 (incl. caller-effect ≤ tool-effect monotonicity per CORR-05). judge_band [0.99, 1.00].
- Sample-size plan: fixed registry + 10,000 dynamic; no interim.
- Effect-size plan: violation rate (Wilson CI, target upper bound ≤ 0.0004 dynamic).
- Null-handling: any violation → effect-system defect, Gate 3 blocked, fix + re-preregister.
- Claims-register draft: `C09-A15-effects` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A16 — Code Intelligence Resource Bounds

- Hypothesis (falsifiable): LSP servers respect `max_lsp_servers` cap — **max 4 servers** spawned under stress of 50 concurrent LSP requests.
- Workload: `STRESS-A16-v1` — 50 concurrent LSP requests (fixed request list, seeded order from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: uncapped spawning at merge-base commit.
- Judge (conjunctive): (a) max servers observed ≤ 4; (b) 0 resource-exhaustion failures. judge_band [0.95, 1.00].
- Sample-size plan: 50-request stress × 5 seeds = 250 requests; no interim (cap judge).
- Effect-size plan: server-count max + exhaustion rate (Wilson).
- Null-handling: cap breach → spawner defect (SKL-06 shared w/ SKL-03), Gate 4 blocked until fix + re-preregister.
- Claims-register draft: `C09-A16-lsp` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A17 — Lane Configuration Migration

- Hypothesis (falsifiable): forward-only migration preserves lane semantics across v1→v2→v3 — **all lanes functional, 0 data loss**.
- Workload: `MIG-A17-v1` — fixed v1 lane corpus (lane count + configs versioned) migrated v1→v2→v3 with `config_version` forward-only rule (CORR-02).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: v1 lanes pre-migration at merge-base commit (functional reference).
- Judge (conjunctive): (a) all lanes functional post-migration; (b) data-loss diff == 0 bytes/records; (c) downgrade attempt rejected (forward-only). judge_band [0.99, 1.00].
- Sample-size plan: fixed corpus; no interim.
- Effect-size plan: functional-lane rate + loss count (exact).
- Null-handling: any loss/non-functional lane → migration-rule defect, Gate 3 blocked until fix + re-preregister.
- Claims-register draft: `C09-A17-migration` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A18 — Autorunner Throughput Fairness

- Hypothesis (falsifiable): work-stealing achieves Jain's fairness index ≥ **0.90** across lanes under load.
- Workload: `LOAD-A18-v1` — fixed saturating packet mix (n = 10,000 ready-packets per TOOL-03.04 grammar); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: round-robin scheduler at merge-base commit.
- Judge (conjunctive): (a) Jain index ≥ 0.90 (bootstrap lower bound ≥ 0.87); (b) work-stealing steals carry origin authority token (P-CORR-11 rule observed). judge_band [0.90, 1.00].
- Sample-size plan: 10,000 packets × 5 seeds; interims per file convention.
- Effect-size plan: Jain index (bootstrap CI) + throughput ratio vs round-robin (log-normal).
- Null-handling: index < 0.90 → scheduler retune + re-preregister; authority-token absence → C16 elevation defect, Gate 4 blocked.
- Claims-register draft: `C09-A18-fairness` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A19 — Subagent Containment

- Hypothesis (falsifiable): depth cap (default 3) + budget caps (tokens/time/calls) contain runaway recursion — **0 exceedances** over 10,000 malicious subagent spawns.
- Workload: `ADV-A19-v1` — 10,000 seeded malicious spawn patterns (seeded from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: uncapped spawning at merge-base commit — expected ≥ 50% exceedance (potency check; void if < 50%).
- Judge: exceedance count == 0 (Wilson upper bound ≤ 0.0004). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (zero-tolerance).
- Effect-size plan: exceedance rate (Wilson CI).
- Null-handling: any exceedance → containment defect, Gate 4 blocked, fix + re-preregister.
- Claims-register draft: `C09-A19-containment` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A20 — Interference Measurement Accuracy

- Hypothesis (falsifiable): `I(i,j)` from geodesic distance predicts empirical cross-lane recall interference with Pearson correlation ≥ **0.80**.
- Workload: `CORR-A20-v1` — fixed lane-pair set (n ≥ 200 pairs) with measured recall-interference outcomes; 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: constant/naive interference predictor at merge-base commit.
- Judge (conjunctive): (a) r ≥ 0.80 (bootstrap lower bound ≥ 0.75); (b) `I(i,j)` logged per C14/ledger rule. judge_band [0.85, 1.00].
- Sample-size plan: n ≥ 200 pairs; interim at 50% with futility stop.
- Effect-size plan: correlation (bootstrap CI).
- Null-handling: r < 0.80 → interference metric demoted to diagnostic (not gating breaker input); breaker threshold frozen until re-preregistered metric passes.
- Claims-register draft: `C09-A20-interference` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A21 — Brigade Weight Invariant

- Hypothesis (falsifiable): single weight set maintained under N=8 concurrent brigade tasks — VRAM shows **1 copy** AND per-task p95 latency ratio (brigade / standard spawn) ≤ **1.05**.
- Workload: `CORPUS-A21-v1` — 8 concurrent brigade tasks (fixed list); 5 seeds in S; same shape as A4, brigade-projection path (TOOL-03.05 attestation).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: standard spawn per task at merge-base commit.
- Judge (conjunctive): (a) weight_copies == 1 every sample; (b) p95 latency ratio ≤ 1.05 (bootstrap upper bound ≤ 1.10). judge_band [0.95, 1.00].
- Sample-size plan: 8-task swarm × 5 seeds; interim at 50%.
- Effect-size plan: latency ratio (log-normal); memory % change (percentile).
- Null-handling: copies > 1 → projection WITHDRAWN; latency breach → memory-win-only claim.
- Claims-register draft: `C09-A21-weights` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A22 — Ledger Integrity Under Crash

- Hypothesis (falsifiable): ledger invariants hold after power loss / process kill at any point — **0 violations** over 1,000 crash injections with full recovery.
- Workload: `CRASH-A22-v1` — 1,000 seeded kill points spanning append / Merkle-seal / checkpoint windows (seeded from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unsealed/un-checkpointed store at merge-base commit — expected ≥ 1 violation class (potency check).
- Judge (conjunctive): (a) violations == 0; (b) Merkle root every 1000 entries verified post-recovery (CORR-10); (c) no UPDATE/DELETE in log (append-only). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 1,000; no interim (zero-tolerance).
- Effect-size plan: violation rate (Wilson CI, target upper bound ≤ 0.004).
- Null-handling: any violation → ledger defect, Gates 6/7 blocked, fix + re-preregister.
- Claims-register draft: `C09-A22-ledger` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A23 — Telemetry Privacy Enforcement

- Hypothesis (falsifiable): no telemetry output contains args/results/text under any code path — **0 privacy-schema violations** over static analysis + 10,000 seeded fuzzed emits.
- Workload: `FUZZ-A23-v1` — 10,000 seeded emits (mutators args/results/text/paths; seed from S, REQUIRED per TOOL-04.05).
- Hardware: HW-M3MAX primary (single-machine schema property), HW-M2ULTRA replica.
- Baseline: unfiltered emitter at merge-base commit — expected ≥ 1 violation class (potency check).
- Judge: violations == 0 (Wilson upper bound ≤ 0.0004). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (zero-tolerance).
- Effect-size plan: violation rate (Wilson CI).
- Null-handling: any violation → privacy-class SHIP-BLOCKER (SKL-07 joint), pipeline frozen until fix + re-preregister.
- Claims-register draft: `C09-A23-privacy` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A24 — Metrics Cardinality Bound

- Hypothesis (falsifiable): active series stays < **10,000** under maximum load (N=8 lanes, 1000 req/s).
- Workload: `LOAD-A24-v1` — fixed max-load mix (N=8, 1000 req/s, 60 s window); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unguarded registration at merge-base commit.
- Judge (conjunctive): (a) max active series < 10,000 every run; (b) cardinality guard fires at registration (TOOL-04.03) before breach. judge_band [0.95, 1.00].
- Sample-size plan: 60 s × 5 seeds; no interim (bound judge).
- Effect-size plan: max-series observed (exact per run) + breach rate (Wilson).
- Null-handling: breach → guard-rule defect, Gates 6/8 blocked until fix + re-preregister.
- Claims-register draft: `C09-A24-cardinality` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A25 — Roll-Up Completeness

- Hypothesis (falsifiable): daily roll-up captures **100%** of events in the day window (roll-up count − ledger count == 0).
- Workload: `DAY-A25-v1` — 3 fixed synthetic day-windows (small/medium/large event counts, versioned) + 1 live-capture window.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: dashboard-count comparison at merge-base commit (dashboards NOT citable per ToolingC04 — baseline records the failure mode the ledger path replaces).
- Judge: difference == 0 on all 4 windows. judge_band [0.99, 1.00].
- Sample-size plan: fixed 4 windows; no interim.
- Effect-size plan: miss count (exact).
- Null-handling: any miss → roll-up defect (TOOL-04.04), Gate 8 blocked until fix + re-preregister.
- Claims-register draft: `C09-A25-rollup` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A26 — Documentation Completeness

- Hypothesis (falsifiable): five-file set enforced AND claim linter catches **100%** of injected unlabeled claims (50/50).
- Workload: `LINT-A26-v1` — 50 seeded unlabeled-claim injections across C01–C35 Design.md files (seeded from S).
- Hardware: single-machine sufficient (static property); machine recorded anyway.
- Baseline: no linter at merge-base commit — expected 0 caught (potency check).
- Judge (conjunctive): (a) caught == 50/50 (Wilson lower bound ≥ 0.93); (b) five-file check green on every workstream. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 50; no interim.
- Effect-size plan: catch rate (Wilson CI).
- Null-handling: any miss → linter-rule defect (grandfather rule per ToolingC06 applies only to pre-2026-10-26 frozen text — new misses still block Gate 1).
- Claims-register draft: `C09-A26-docs` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A27 — Migration Traceability

- Hypothesis (falsifiable): every missing external doc traced to migration or deletion — **0 unresolved references** in `migration_status.json` audit.
- Workload: `AUDIT-A27-v1` — full external-reference scan at merge-base commit (reference list versioned in run receipt).
- Hardware: single-machine sufficient; machine recorded.
- Baseline: pre-migration reference set (expected > 0 unresolved — validates scanner potency).
- Judge: unresolved == 0. judge_band [0.99, 1.00].
- Sample-size plan: fixed full scan; no interim.
- Effect-size plan: unresolved count (exact).
- Null-handling: any unresolved → Gate 1 blocked (ADR-07), migrate-or-remove then re-preregister.
- Claims-register draft: `C09-A27-migration` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A28 — Handoff Protocol Correctness

- Hypothesis (falsifiable): session-typed handoffs prevent protocol mismatches — **0 type errors, 0 deadlocks** over 1,000 random handoffs.
- Workload: `HAND-A28-v1` — 1,000 seeded random handoffs (seeded from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: untyped handoffs at merge-base commit — expected ≥ 1 error class (potency check).
- Judge (conjunctive): (a) type errors == 0; (b) deadlocks == 0 (time-boxed per session-type rule). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 1,000; no interim (zero-tolerance).
- Effect-size plan: error/deadlock rates (Wilson CI, target upper bounds ≤ 0.004).
- Null-handling: any error/deadlock → session-type defect (SKL-03/protobuf-generated-schema rule per ToolingC03), Gate 4 blocked until fix + re-preregister.
- Claims-register draft: `C09-A28-handoffs` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A29 — Lane Isolation

- Hypothesis (falsifiable): a lane cannot access tools/RAG outside its allowlist — **0 successes** over 10,000 cross-lane tool calls.
- Workload: `ATT-A29-v1` — 10,000 seeded cross-lane calls (seeded from S; disjoint attempt-space from A13 — tools/RAG here vs skills/caps there).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: allowlist disabled at merge-base commit — expected ≥ 50% success (potency; void if < 50%).
- Judge: successes == 0 (Wilson upper bound ≤ 0.0004). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (zero-tolerance).
- Effect-size plan: success rate (Wilson CI).
- Null-handling: any success → isolation defect (C16 class), Gate 4 blocked, fix + re-preregister.
- Claims-register draft: `C09-A29-laneisolation` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A30 — Approval Policy Enforcement

- Hypothesis (falsifiable): destructive actions require human approval in all lanes — **0 executions** over 100 destructive attempts without human approval.
- Workload: `APPR-A30-v1` — 100 seeded destructive attempts across all lanes × all effect modes (seeded from S).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: approval disabled at merge-base commit — expected 100/100 execute (potency; void if < 100).
- Judge: executions == 0 (Wilson upper bound ≤ 0.036). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 100; no interim (zero-tolerance).
- Effect-size plan: execution rate (Wilson CI).
- Null-handling: any execution → approval-policy defect, SHIP-BLOCKER (SKL-07), Gates 6/7 blocked until fix + re-preregister.
- Claims-register draft: `C09-A30-approval` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A31 — Geometry Interop Correctness

- Hypothesis (falsifiable): Lorentz ↔ Poincaré isometry holds within **4.4e-9** for all representable coordinates, extended to 10,000 pairs including edge cases (x₀ near 1, x₀ ≥ 65536).
- Workload: `GEO-A31-v1` — 10,000 pairs (900-pair monster-hybrid-server verified set + edge-case extension); pair list versioned; fuzz-corpus seeds `FUZZ-SEED-A31 = 20261101` + mutation corpus `CORPUS-A31-FUZZ-v1` filed alongside (ToolingC02 condition 4).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: pre-fix projection code at merge-base commit (expected edge-case failures — validates extension potency).
- Judge: max deviation ≤ 4.4e-9 over all 10,000 pairs. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (exactness judge).
- Effect-size plan: max deviation (exact) + failure-pair count (Wilson).
- Null-handling: any breach → geometry defect (C05/C35), Gate 8 blocked until fix + re-preregister.
- Claims-register draft: `C09-A31-isometry` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A32 — Picard Reduction Soundness

- Hypothesis (falsifiable): Picard reduction maps to the fundamental domain AND is idempotent (reduce² == reduce) over all test radii {25, 250, 5000, 50000} — **0 violations**.
- Workload: `PIC-A32-v1` — full radius sweep (versioned point set); fuzz-corpus seeds `FUZZ-SEED-A32 = 20261102` + `CORPUS-A32-FUZZ-v1` filed alongside (ToolingC02 condition 4).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: simplified back-map (known w-fiber-dropping variant per C06.21/FW audit — expected idempotence failure; validates judge potency).
- Judge (conjunctive): (a) all outputs in fundamental domain; (b) reduce² == reduce bit-exact everywhere. judge_band [0.99, 1.00].
- Sample-size plan: fixed sweep; no interim.
- Effect-size plan: violation count (exact).
- Null-handling: any violation → Picard-rewrite path (full SL(2,Z[i]) action, C06 open item) stays mandatory; Gate 8 blocked for geometry claims until fix + re-preregister.
- Claims-register draft: `C09-A32-picard` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A33 — VP-Tree Retrieval Accuracy

- Hypothesis (falsifiable): VP-tree 5D retrieval matches brute-force top-k within **1% recall** (recall@10 ≥ 0.99) AND p99 latency < **1 ms**.
- Workload: `VP-A33-v1` — brute-force reference on 100k atoms (versioned atom set); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: brute force on identical atoms/seeds at merge-base commit.
- Judge (conjunctive): (a) recall@10 ≥ 0.99 (Wilson lower bound ≥ 0.985); (b) p99 latency < 1 ms. judge_band [0.95, 1.00].
- Sample-size plan: 100k-atom benchmark × 5 seeds; interim at 50% with futility stop.
- Effect-size plan: recall risk-difference (Wilson); latency ratio (log-normal).
- Null-handling: recall breach → VP-tree parametrization defect, Gate 8 blocked; latency breach → perf-only fail (accuracy claim survives, speed claim WITHDRAWN).
- Claims-register draft: `C09-A33-vptree` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A34 — Geometric Forgetting Boundedness

- Hypothesis (falsifiable): cache size stays bounded under sustained load without manual eviction — size ≤ configured max over **1M writes** AND recall precision ≥ baseline.
- Workload: `FORGET-A34-v1` — 1M sustained writes (versioned stream); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unbounded cache at merge-base commit (expected OOM-class growth — validates boundedness judge).
- Judge (conjunctive): (a) max size ≤ configured max every sample; (b) recall precision ≥ baseline (Δ ≥ −0.01). judge_band [0.95, 1.00].
- Sample-size plan: 1M writes × 5 seeds; interims per file convention (early OOM stops the run as FAIL, not void).
- Effect-size plan: size ratio (exact max); precision risk-difference (Wilson).
- Null-handling: bound breach → eviction-policy defect (Ricci-flow backlog C06.2 stays backlog), re-preregister after fix.
- Claims-register draft: `C09-A34-forgetting` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A35 — Persistence Atomicity

- Hypothesis (falsifiable): atomic snapshot + WAL survives power loss at any point — **0 data corruptions, 100% recovery** over 10,000 crash injections.
- Workload: `CRASH-A35-v1` — 10,000 seeded kill points spanning snapshot / WAL-append / fsync windows (seeded from S; disjoint window-model from A22 — snapshots here vs ledger seals there).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: non-atomic store at merge-base commit — expected ≥ 1 corruption class (potency check).
- Judge (conjunctive): (a) corruptions == 0; (b) recovery == 10,000/10,000. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 10,000; no interim (zero-tolerance).
- Effect-size plan: corruption/recovery rates (Wilson CI, target corruption upper bound ≤ 0.0004).
- Null-handling: any corruption → persistence defect, Gates 6/7 blocked, fix + re-preregister.
- Claims-register draft: `C09-A35-persistence` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A36 — Depth-Gated Speculative Recall (C06.15; contributor Muse 2026-10-26)

- Hypothesis (falsifiable): Fusing adaptive-depth recall (C06.4) with speculative draft-verify (C06.6) via endogenous k-draft beats both parents on throughput at equal accuracy.
- Workload: A1 fixed prompt corpus (100 prompts, 10k tokens, 512-token generation); seeds recorded at run.
- Hardware: primary MacBook Pro M3 Max 36GB + replica Mac Studio M2 Ultra 96GB (C09.4); normalization inputs recorded per C21.8.
- Baseline: C06.4-alone and C06.6-alone on identical workload/hardware/seeds (two baselines, not one).
- Judge (conjunctive — all must pass): (a) speculative acceptance rate > 30%; (b) draft calibration error within eps_cal on ≥80% of queries (eps_cal stated numerically here before run: eps_cal = 0.05 nats); (c) tok/s_fusion > tok/s_C06.4 AND > tok/s_C06.6 at accuracy parity (Δaccuracy within ±0.01).
- Sample-size plan: interim analyses at 25/50/75% with Lan-DeMets alpha-spending (ρ=2, C09); stop for futility if conditional power < 0.2.
- Null-handling: acceptance ≤30% → draft uncalibrated, item WITHDRAWN (fall back to exogenous D-max); beats one parent only → fusion unjustified, item WITHDRAWN, simpler parent ships.
- Claims-register draft: C06.15 PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29 (prereg text filed; hash pending → filed this pass).

## A37 — Judge Calibration Drift Monitor (C06.16; contributor Muse 2026-10-26)

- Hypothesis (falsifiable): A sequential Lan-DeMets monitor over judge verdict pass-rates detects injected 15% verdict-flip drift within 200 verdicts at <5% false alarms on clean runs.
- Workload: shadow run replaying Day 3–10 judge verdict logs (verdict hashes + outcomes only — ADR-06 clean); synthetic drift injected at known verdict index.
- Hardware: single machine sufficient (no perf claim — detection latency in verdict-count, not wall-clock); record machine anyway.
- Baseline: no monitor (drift undetected until manual audit) — comparison is detection vs non-detection, not speed.
- Judge (conjunctive): (a) detection latency ≤200 verdicts post-injection on 10/10 injected runs; (b) false-alarm rate <5% on 10 clean runs of ≥1000 verdicts each; (c) quarantine action fires (claims citing drifted judge marked INFERRED) within 10 verdicts of detection.
- Sample-size plan: fixed 10+10 runs (no interim — binary outcome per run); pre-stated before run.
- Null-handling: detection >200 verdicts → thresholds too loose, retune + re-preregister; false alarms ≥5% → boundary too tight, retune + re-preregister. Either null keeps C06.16 PROPOSED.
- Claims-register draft: C06.16 PROPOSED → EVIDENCED on pass.
- Status: PREREGISTERED 2026-10-29 (prereg text filed; hash pending → filed this pass).

---

**Preregistration Hashes** (section-text SHA-256, filed 2026-10-29; each hash covers the exact `## A<N>` section bytes above — hashes live here in the footer, texts above stay hash-free so filing never invalidates itself):

| Ablation | Section-text hash (SHA-256)                                        | Date Filed | Notes                           |
| -------- | ------------------------------------------------------------------ | ---------- | ------------------------------- |
| A11      | `68e3141c8ee7811337f5a70cb63a4b4387ef1023e5e1d878d8caef9ad4d6a27e` | 2026-10-29 | §A11 text; run hash at run time |
| A12      | `3bbe96685715c540a9c37778a5412020373b93105ae4aa80bb39311c711d54cd` | 2026-10-29 | §A12 text; run hash at run time |
| A13      | `53b13f7a91544b84b757182bad66af8824365ede0ce0b6ff2f6ef460862dbc36` | 2026-10-29 | §A13 text; run hash at run time |
| A14      | `28ff09072ae65ab16f7a72792d0bb16ad654dbcf282f4ea6b863b617c1428a2a` | 2026-10-29 | §A14 text; run hash at run time |
| A15      | `b4451842885a96aac9488776f46d80f8e62c697714fc503ad7987d59abd800fc` | 2026-10-29 | §A15 text; run hash at run time |
| A16      | `18a1de4f2dda9268b7750c12cf787b9e2b202e29545e0305ae10f1629dd0f6a0` | 2026-10-29 | §A16 text; run hash at run time |
| A17      | `aee58b505978240893e8f7a3e288c8f16da8f31f7ec938f203e5b931d22dadce` | 2026-10-29 | §A17 text; run hash at run time |
| A18      | `66e8007288ae58a05c904564d205ad323b3d08f31b58ef6b34d0cc5b281814b7` | 2026-10-29 | §A18 text; run hash at run time |
| A19      | `8293e82c3bec807f3338c9b8585c17576cc25ce6d448dd60530917f534f5bb36` | 2026-10-29 | §A19 text; run hash at run time |
| A20      | `6d470d6105cb2906f92f0b7f15cf52b6555e1c432f43bbc86f1b4b27010c7378` | 2026-10-29 | §A20 text; run hash at run time |
| A21      | `abe807846fc79fd1a1a982b0363312cc09ddf4c6d38c6d4ee06031fd92606d79` | 2026-10-29 | §A21 text; run hash at run time |
| A22      | `ddb4fdff15a0127222c03b9a13522eeb87b0408c3d8cf8f3b3fe6e0a50f122d9` | 2026-10-29 | §A22 text; run hash at run time |
| A23      | `82deb84f12caec5ffbc06514289f5d56474eb23483bf081cf10c845522d308e3` | 2026-10-29 | §A23 text; run hash at run time |
| A24      | `5075291def5a4b2b44303f30e0d623f195a58006dedf21b5627d7d0fc1183226` | 2026-10-29 | §A24 text; run hash at run time |
| A25      | `d318449fbf03e90e95bf885d2207b206691a307113b08de74884c291cc8c12ec` | 2026-10-29 | §A25 text; run hash at run time |
| A26      | `6239c7b36ea20ec0fe7633c35bf2053c35b47e6c0ac608617f4fff27962cb1c7` | 2026-10-29 | §A26 text; run hash at run time |
| A27      | `e8468dfb6e529a85ce151c043c87fcd72559d98bf4c763cdb8ad659f0f61efb7` | 2026-10-29 | §A27 text; run hash at run time |
| A28      | `ff38ba9433803b11a1f7d8b26e70f09064bbf4dfb3159d179fb6ada4b333544f` | 2026-10-29 | §A28 text; run hash at run time |
| A29      | `e03a2afdb8a724df1e340f6e3f51dabf365b0461e7c70216eab8152ef61b53a2` | 2026-10-29 | §A29 text; run hash at run time |
| A30      | `88307c3766ee479a63e8956b1897421654f59619260a3b8abf7321607784955e` | 2026-10-29 | §A30 text; run hash at run time |
| A31      | `8e993991f898d5f62aeef7d4b5aa47f278872c1de2e566dc0ca8956703c07ed3` | 2026-10-29 | §A31 text; run hash at run time |
| A32      | `3927f9bbb92ac9cb8515d872bb5629432fbbbf95e8d2f1d18a7a985e36feb815` | 2026-10-29 | §A32 text; run hash at run time |
| A33      | `4ffff29ef3db1e12fe3a3cb582c74866e2be352c981970fa35ca1dde826c86d3` | 2026-10-29 | §A33 text; run hash at run time |
| A34      | `71b329953555db9a49ca052a9e71ca57a187708aa7aa2e5f05dfa31b0bffeef2` | 2026-10-29 | §A34 text; run hash at run time |
| A35      | `e88d299f75bc41461b270a4bbc75ab99b45b331c1d4378dc0e0a3ed2a6748e68` | 2026-10-29 | §A35 text; run hash at run time |
| A36      | `ca78e271a5ee0802e5a6a1248380c637ccf9b86f4256863657451f3d57217529` | 2026-10-29 | §A36 text; run hash at run time |
| A37      | `eca33754e8f30b4079b2c64d9f9a436d3f43190a8070828726779b8f59c42646` | 2026-10-29 | §A37 text; run hash at run time |
