# Pre-Registration — A11–A35

**Rule.** Every ablation requires this preregistration hash to be written before the run.
An ablation run without a pre-filed hash is automatically disqualified (C11.3).
Null/negative results are recorded as first-class entries.

---

## A11 — Skill Registry Integrity

- Hypothesis: Manifest hash deterministic; boot validation catches all binding mismatches.
- Baseline: Ad-hoc skill loading (no manifest).
- Judge: Build reproducibility + boot failure injection test.
- Status: TBD.

## A12 — Hook Veto Correctness

- Hypothesis: Single veto at any moment aborts action and emits receipt.
- Judge: 23 moments × veto/non-veto matrix; receipt presence.
- Status: TBD.

## A13 — Brigade Skill Isolation

- Hypothesis: Lane i skill cannot invoke caps of lane j without circuit breaker.
- Judge: Cross-lane cap access attempts logged and denied.
- Status: TBD.

## A14 — Tool Registry Determinism

- Hypothesis: `resolve_v2` is deterministic; same call always resolves to same tool/effect.
- Baseline: Ad-hoc string-based tool dispatch.
- Judge: 10,000 random calls; 0 non-deterministic resolutions.
- Status: TBD.

## A15 — Effect System Soundness

- Hypothesis: No tool executes with effect exceeding its declared level.
- Judge: Static analysis + dynamic hole audit; 0 violations.
- Status: TBD.

## A16 — Code Intelligence Resource Bounds

- Hypothesis: LSP servers respect `max_lsp_servers` cap; no resource exhaustion.
- Judge: Stress test with 50 concurrent LSP requests; max 4 servers spawned.
- Status: TBD.

## A17 — Lane Configuration Migration

- Hypothesis: Forward-only migration preserves lane semantics across versions.
- Judge: Migrate v1→v2→v3; all lanes functional; no data loss.
- Status: TBD.

## A18 — Autorunner Throughput Fairness

- Hypothesis: Work-stealing achieves ≥90% lane utilization fairness under load.
- Baseline: Round-robin scheduler.
- Judge: Jain's fairness index ≥ 0.9 across lanes.
- Status: TBD.

## A19 — Subagent Containment

- Hypothesis: Depth/budget caps prevent runaway subagent recursion.
- Judge: 10,000 malicious subagent spawns; 0 exceed caps.
- Status: TBD.

## A20 — Interference Measurement Accuracy

- Hypothesis: `I(i,j)` computed from geodesic distance matches empirical recall interference.
- Judge: Correlation ≥ 0.8 between `I(i,j)` and cross-lane recall error.
- Status: TBD.

## A21 — Brigade Weight Invariant

- Hypothesis: Single weight set maintained under N=8 concurrent brigade tasks.
- Judge: VRAM allocation shows 1 copy; per-task latency < standard spawn.
- Status: TBD.

## A22 — Ledger Integrity Under Crash

- Hypothesis: Ledger invariants hold after power loss / process kill at any point.
- Judge: 1,000 crash injections; 0 invariant violations on recovery.
- Status: TBD.

## A23 — Telemetry Privacy Enforcement

- Hypothesis: No telemetry output contains args/results/text under any code path.
- Judge: Static analysis + 10,000 fuzzed emits; 0 privacy schema violations.
- Status: TBD.

## A24 — Metrics Cardinality Bound

- Hypothesis: Active series stays < 10,000 under maximum load (N=8 lanes, 1000 req/s).
- Judge: Load test; max series observed.
- Status: TBD.

## A25 — Roll-Up Completeness

- Hypothesis: Daily roll-up captures 100% of events in the day window.
- Judge: Compare roll-up counts to ledger counts; difference = 0.
- Status: TBD.

## A26 — Documentation Completeness

- Hypothesis: Five-file set enforced; claim linter catches 100% unlabeled claims.
- Judge: Inject 50 unlabeled claims; all caught.
- Status: TBD.

## A27 — Migration Traceability

- Hypothesis: Every missing external doc traced to migration or deletion.
- Judge: Audit `migration_status.json`; 0 unresolved references.
- Status: TBD.

## A28 — Handoff Protocol Correctness

- Hypothesis: Session-typed handoffs prevent protocol mismatches.
- Judge: 1,000 random handoffs; 0 type errors; 0 deadlocks.
- Status: TBD.

## A29 — Lane Isolation

- Hypothesis: Lane cannot access tools/RAG outside allowlist.
- Judge: 10,000 cross-lane tool calls; 0 succeed.
- Status: TBD.

## A30 — Approval Policy Enforcement

- Hypothesis: Destructive actions require human approval in all lanes.
- Judge: 100 destructive attempts without human; 0 execute.
- Status: TBD.

## A31 — Geometry Interop Correctness

- Hypothesis: Lorentz ↔ Poincaré isometry holds within 4.4e-9 for all representable coordinates.
- Baseline: Monster-hybrid-server verified 900 pairs.
- Judge: Extend to 10,000 pairs including edge cases (x₀ near 1, x₀ ≥ 65536).
- Status: TBD.

## A32 — Picard Reduction Soundness

- Hypothesis: Picard reduction maps to fundamental domain; idempotent.
- Judge: All test radii; `reduce² = reduce`; output in fundamental domain.
- Status: TBD.

## A33 — VP-Tree Retrieval Accuracy

- Hypothesis: VP-tree 5D retrieval matches brute-force top-k within 1% recall.
- Baseline: Brute-force on 100k atoms.
- Judge: Recall@10 ≥ 99%; latency < 1ms.
- Status: TBD.

## A34 — Geometric Forgetting Boundedness

- Hypothesis: Cache size bounded under sustained load without manual eviction.
- Judge: 1M writes; cache size ≤ configured max; recall precision ≥ baseline.
- Status: TBD.

## A35 — Persistence Atomicity

- Hypothesis: Atomic snapshot + WAL survives power loss at any point.
- Judge: 10,000 crash injections; 0 data corruption; 100% recovery.
- Status: TBD.

## A36 — Depth-Gated Speculative Recall (C06.15; contributor Muse 2026-10-26)

- Hypothesis (falsifiable): Fusing adaptive-depth recall (C06.4) with speculative draft-verify (C06.6) via endogenous k-draft beats both parents on throughput at equal accuracy.
- Workload: A1 fixed prompt corpus (100 prompts, 10k tokens, 512-token generation); seeds recorded at run.
- Hardware: primary MacBook Pro M3 Max 36GB + replica Mac Studio M2 Ultra 96GB (C09.4); normalization inputs recorded per C21.8.
- Baseline: C06.4-alone and C06.6-alone on identical workload/hardware/seeds (two baselines, not one).
- Judge (conjunctive — all must pass): (a) speculative acceptance rate > 30%; (b) draft calibration error within eps_cal on ≥80% of queries (eps_cal stated numerically here before run: eps_cal = 0.05 nats); (c) tok/s_fusion > tok/s_C06.4 AND > tok/s_C06.6 at accuracy parity (Δaccuracy within ±0.01).
- Sample-size plan: interim analyses at 25/50/75% with Lan-DeMets alpha-spending (ρ=2, C09); stop for futility if conditional power < 0.2.
- Null-handling: acceptance ≤30% → draft uncalibrated, item WITHDRAWN (fall back to exogenous D-max); beats one parent only → fusion unjustified, item WITHDRAWN, simpler parent ships.
- Claims-register draft: C06.15 PROPOSED → EVIDENCED on pass + replication.
- Status: FILED (prereg text hashed 2026-10-29; hash in table below).

## A37 — Judge Calibration Drift Monitor (C06.16; contributor Muse 2026-10-26)

- Hypothesis (falsifiable): A sequential Lan-DeMets monitor over judge verdict pass-rates detects injected 15% verdict-flip drift within 200 verdicts at <5% false alarms on clean runs.
- Workload: shadow run replaying Day 3–10 judge verdict logs (verdict hashes + outcomes only — ADR-06 clean); synthetic drift injected at known verdict index.
- Hardware: single machine sufficient (no perf claim — detection latency in verdict-count, not wall-clock); record machine anyway.
- Baseline: no monitor (drift undetected until manual audit) — comparison is detection vs non-detection, not speed.
- Judge (conjunctive): (a) detection latency ≤200 verdicts post-injection on 10/10 injected runs; (b) false-alarm rate <5% on 10 clean runs of ≥1000 verdicts each; (c) quarantine action fires (claims citing drifted judge marked INFERRED) within 10 verdicts of detection.
- Sample-size plan: fixed 10+10 runs (no interim — binary outcome per run); pre-stated before run.
- Null-handling: detection >200 verdicts → thresholds too loose, retune + re-preregister; false alarms ≥5% → boundary too tight, retune + re-preregister. Either null keeps C06.16 PROPOSED.
- Claims-register draft: C06.16 PROPOSED → EVIDENCED on pass.
- Status: FILED (prereg text hashed 2026-10-29; hash in table below).

---

**Preregistration Hashes** (to be filled before each run):

| Ablation  | Hash (SHA-256) | Date Filed |
| --------- | -------------- | ---------- |
| A11       |                |            |
| A12       |                |            |
| A13       |                |            |
| A14       |                |            |
| A15       |                |            |
| A16       |                |            |
| A17       |                |            |
| A18       |                |            |
| A19       |                |            |
| A20       |                |            |
| A21       |                |            |
| A22       |                |            |
| A23       |                |            |
| A24       |                |            |
| A25       |                |            |
| A26       |                |            |
| A27       |                |            |
| A28       |                |            |
| A29       |                |            |
| A30       |                |            |
| A31       |                |            |
| A32       |                |            |
| A33       |                |            |
| A34       |                |            |
| A35       |                |            |
| A36       | sha256:480a3943ac8ec1be693ba013e5e19c190649af0537f77e21fd42da7d7f489d3f | 2026-10-29 |
| A37       | sha256:75e4446e0256451d08990934d4404bf13b47511c5f2ad92e84ee84128ddad5ed | 2026-10-29 |
| EOF       |
| echo done |
