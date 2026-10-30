# Pre-Registration — A1–A10

**Rule.** Every ablation requires this preregistration hash to be written before the run.
An ablation run without a pre-filed hash is automatically disqualified (C11.3).
Null/negative results are recorded as first-class entries.

**Certification pass 2026-10-29 (Stewart directive: certify for smoke, top-to-bottom).**
Each section below carries the full C09.9 8-item content. File-level conventions (apply to A1–A10 unless a section overrides):

- **Seeds:** S = {20261101, 20261102, 20261103, 20261104, 20261105} (5 fixed seeds; every run records the seed in its receipt per C03.5).
- **Hardware:** primary `HW-M3MAX` (MacBook Pro M3 Max, 36GB, macOS 14, mains power, ambient °C recorded); replica `HW-M2ULTRA` (Mac Studio M2 Ultra, 96GB). Energy reported raw per machine, mains-only (C21.8 — never normalized).
- **Baseline commit rule:** baseline path = run-branch merge-base commit; exact hash recorded in the run receipt (C15.7). Comparison config (e.g. kill-switch state per ADR-05) stated per section.
- **Interim analyses:** Lan-DeMets alpha-spending ρ=2 at 25/50/75% information; stop for futility if conditional power < 0.2 (C09 protocol).
- **Judge bands:** each section states `judge_band [l,u]` — the expected pass-rate band feeding the TOOL-05.05/A37 drift monitor.
- **Effect-size methods:** per C09 table (throughput Cohen's d bootstrap; accuracy/rates risk-difference Wilson; memory % change percentile; energy ratio log-normal).
- Shell residue (`EOF`/`echo done`) present in the pre-certification revision removed — no hash was filed under that revision, so no version fork (ADR-19/ADR-22 n/a).
- **Hash-filing convention (RATIFIED 2026-10-30, Stewart):** §-text SHA-256 over bytes `## A<N>` header line through the blank line before the next `## A` header or the `---` footer rule — the footer table lists A1–A10 (A10 ends at its blank line before `---`, same convention as every other section). Verified by recompute 2026-10-30: 10/10 MATCH. Text frozen at ratification — any edit forks a new version + new hashes.

## A1 — KV cache (hyperbolic vs standard)

- Hypothesis (falsifiable): nested hyperbolic Poincaré cache (C05/C12) beats the standard O(N²) transformer path on throughput ratio R = tok/s_treat / tok/s_base ≥ **1.20** (δ_thr = +20%) at peak-memory ratio ≤ **1.00**.
- Workload: `CORPUS-A1-v1` — 100 fixed prompts, 10k tokens total context, 512-token greedy generations (T=0), batch 1; all 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica; normalization inputs recorded per C21.8.
- Baseline: standard transformer path = ADR-05 kill-switch fallback (ACFT disabled) at merge-base commit (hash in run receipt).
- Judge (conjunctive): (a) median R ≥ 1.20 with bootstrap 95% CI lower bound ≥ 1.10; (b) peak-memory ratio ≤ 1.00; (c) waste figure reported (C21). judge_band [0.95, 1.00] on repeat-run pass rate.
- Sample-size plan: n = 100 prompts × 5 seeds = 500 generations; interims per file convention.
- Effect-size plan: throughput Cohen's d (bootstrap CI, d ≥ 0.8 = large); memory % change (percentile CI).
- Null-handling: R < 1.20 → hyperbolic cache demoted to OPTIONAL (C13.1), kill-switch path stays default; memory ratio > 1.00 → cache-size cap retune + re-preregister; futility stop → file null, no re-run without design change.
- Claims-register draft: `C09-A1-throughput`, `C09-A1-memory` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A2 — Memory recall after compaction

- Hypothesis (falsifiable): attractor-basin compaction retains the original objective after 2 compactions with recall == **100%** (δ: 200/200 exact; Wilson 95% lower bound ≥ 0.985).
- Workload: `CORPUS-A2-v1` — 200 objective statements (fixed set, versioned); compact exactly twice via basin radius ε (ε value recorded); all 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica (replica = determinism check, not perf).
- Baseline: flat context dump recall on identical workload/seeds at merge-base commit.
- Judge (conjunctive): (a) recall_treat == 1.00 (200/200); (b) recall_treat ≥ recall_base. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 200 (binary outcome per objective); no interim (exact-equality judge); 5-seed repeat for stability.
- Effect-size plan: risk difference vs baseline (Wilson CI).
- Null-handling: any objective lost → basin-radius root-cause, retune ε, re-preregister (new version + new hash); recall_treat < recall_base → compaction WITHDRAWN for that trajectory class.
- Claims-register draft: `C09-A2-recall` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A3 — Retrieval (BM25 vs dense)

- Hypothesis (falsifiable): BM25+dense is no worse than BM25-only — top-1 non-inferiority margin **1%** (agreement ≥ 99%) AND **zero false-dense overrides** (dense overturning a correct BM25 top-1 with a wrong doc).
- Workload: `CORPUS-A3-v1` — labeled query set n ≥ 500 over document collection `DOCS-A3-v1`; per-collection splits recorded; dense seed(s) recorded if sampling involved.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: BM25-only with pinned k1/b per TOOL-06.01 at merge-base commit.
- Judge (conjunctive): (a) top-1 agreement ≥ 0.99 (Wilson lower bound ≥ 0.98); (b) false-dense count == 0; (c) per-collection deltas reported (no collection regresses > 2%). judge_band [0.97, 1.00].
- Sample-size plan: n ≥ 500 queries; interim at 50% with futility stop (conditional power < 0.2 on (a)).
- Effect-size plan: risk difference in top-1 accuracy (Wilson CI); epsilon power-justified per ToolingC06 condition.
- Null-handling: agreement < 0.99 or any false-dense → dense stays OFF by default (ADR-02 honest-BM25 default holds); embed pipeline blocked until re-preregistered fix passes.
- Claims-register draft: `C09-A3-retrieval` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A4 — Model-size agent (single model vs specialist fibers)

- Hypothesis (falsifiable): specialist fibers from one weight set hold live weight-copy count == **1** under N=8 swarm AND per-task p95 latency ratio (fibers / spawn-a-clone) ≤ **1.05**.
- Workload: `CORPUS-A4-v1` — 8 concurrent brigade tasks (fixed task list), 5 seeds in S; VRAM sampled per task.
- Hardware: HW-M3MAX primary (VRAM observed), HW-M2ULTRA replica.
- Baseline: spawn-a-clone per task (N copies) at merge-base commit.
- Judge (conjunctive): (a) weight_copies == 1 on every sample (attested, cf. TOOL-03.05); (b) p95 latency ratio ≤ 1.05 (bootstrap CI upper bound ≤ 1.10). judge_band [0.95, 1.00].
- Sample-size plan: 8-task swarm × 5 seeds = 40 swarm-runs; interim at 50%.
- Effect-size plan: latency ratio (log-normal CI); memory % change (percentile CI).
- Null-handling: copies > 1 → fiber projection WITHDRAWN, clone ships; latency ratio > 1.05 → fibers OPTIONAL (memory win only, no latency claim).
- Claims-register draft: `C09-A4-fibers`, `C09-A4-latency` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A5 — Agent count

- Hypothesis (falsifiable): N=8 lanes beat N=1 lane on judge-gated task pass rate by Δpass ≥ **+0.10 absolute** without OOM (peak ≤ 30 GB ceiling per C21) and with throughput-per-watt reported.
- Workload: `CORPUS-A5-v1` — fixed judge-gated task set (n ≥ 100 tasks); N ∈ {1, 8}; 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica; power state mains, ambient recorded (C21.8).
- Baseline: N=1 lane on identical tasks/seeds at merge-base commit.
- Judge (conjunctive): (a) Δpass ≥ +0.10 (Wilson CI on difference, lower bound ≥ +0.05); (b) peak memory ≤ 30 GB both arms; (c) throughput-per-watt (tok/J raw, mains) reported per arm. judge_band [0.90, 1.00].
- Sample-size plan: n ≥ 100 tasks × 2 arms × 5 seeds; interims per file convention.
- Effect-size plan: risk difference (Wilson); energy ratio (log-normal, raw per machine).
- Null-handling: Δpass < +0.10 → N-lane unjustified for this workload class (ships OPTIONAL); any OOM → lane cap lowered + re-preregister.
- Claims-register draft: `C09-A5-passrate`, `C09-A5-watt` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A6 — Context budget

- Hypothesis (falsifiable): larger budgets improve the accuracy/token-cost curve — accuracy(131k) − accuracy(200-line) ≥ **+0.05** with a monotonic non-decreasing curve across {200-line, 8k, 131k}.
- Workload: `CORPUS-A6-v1` — fixed task set runnable at all three budgets (n ≥ 150 tasks); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: 200-line default budget at merge-base commit (curve anchor).
- Judge (conjunctive): (a) accuracy delta ≥ +0.05 (Wilson lower bound ≥ +0.02); (b) monotonicity across the three budgets (no inversion > 0.01); (c) token-cost ratio reported per step. judge_band [0.90, 1.00].
- Sample-size plan: n ≥ 150 tasks × 3 budgets × 5 seeds; interims per file convention.
- Effect-size plan: risk difference per budget step (Wilson); cost ratio (log-normal).
- Null-handling: flat/negative curve → default budget stays 200-line; inversion at 131k → context-routing bug filed, re-preregister after fix.
- Claims-register draft: `C09-A6-budget` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A7 — Hooks

- Hypothesis (falsifiable): deterministic pre/post gating is exactly correct — **0 errors** over the 23-moment × {veto, non-veto} matrix × 10 repeats (460 trials): every unauthorized merge denied with receipt, every authorized merge allowed.
- Workload: `MATRIX-A7-v1` — 23 hook moments × 2 veto states × 10 repeats; seeds recorded (seeds govern trial order, not outcomes).
- Hardware: HW-M3MAX primary (single-machine determinism), HW-M2ULTRA replica.
- Baseline: manual gating outcomes on the same matrix at merge-base commit.
- Judge (conjunctive): (a) 460/460 correct; (b) HookVeto receipt present for every denial. judge_band [0.99, 1.00] (exactness expected; band monitors harness rot, not outcome variance).
- Sample-size plan: fixed 460 trials; no interim (exactness judge).
- Effect-size plan: error rate with Wilson CI (target upper bound ≤ 0.008).
- Null-handling: any miss → hook-algebra defect, Gate 3 blocked for SKL-03 until fix + re-preregister; missing receipt → TOOL-03.02 defect, same block.
- Claims-register draft: `C09-A7-hooks` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A8 — RAG ingest round-trip

- Hypothesis (falsifiable): ingest→query round-trip succeeds == **100%** with receipt present, **per tier** (CANONICAL / VETTED / UNVETTED — per-tier requirement per ToolingC06 condition).
- Workload: `CORPUS-A8-v1` — n = 1000 docs stratified across the three tiers (tier counts recorded); 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: read-only RAG (no ingest route) at merge-base commit — comparison is capability presence, not speed.
- Judge (conjunctive): (a) per-tier round-trip rate == 1.00; (b) receipt present per ingested doc. judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 1000 (binary per doc); no interim.
- Effect-size plan: success rate per tier (Wilson CI, lower bound ≥ 0.996).
- Null-handling: any tier < 100% → ingest route blocked for that tier (tier ships read-only); missing receipts → TOOL-06.02 defect, Gate 4 blocked.
- Claims-register draft: `C09-A8-ingest` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A9 — Memory auto-injection

- Hypothesis (falsifiable): with auto-injection, the agent states a prior decision from a different lane without re-prompting with recall == **100%** over n = 200 cross-lane trials.
- Workload: `CORPUS-A9-v1` — 200 (decision, lane-pair) trials, fixed set; 5 seeds in S.
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: no injection (explicit re-prompt required) at merge-base commit.
- Judge: recall == 1.00 (200/200, Wilson lower bound ≥ 0.985). judge_band [0.99, 1.00].
- Sample-size plan: fixed n = 200; no interim.
- Effect-size plan: risk difference vs baseline (Wilson).
- Null-handling: < 100% → injection stays OFF, explicit re-prompt ships; contamination (wrong-lane content recalled) → privacy-class defect (A23 family), fix + re-preregister.
- Claims-register draft: `C09-A9-injection` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

## A10 — Prompt-injection hygiene

- Hypothesis (falsifiable): poisoned memory never changes lane tool calls without a judge flag — **0 silent tool-call changes** over attack suite `SUITE-A10-v1` (n ≥ 200 seeded attacks covering memory-poison → tool-call paths).
- Workload: `SUITE-A10-v1` — versioned attack list (n ≥ 200); seeds recorded; IFC pipeline = C07.5 (per P-CORR-16/CORR record).
- Hardware: HW-M3MAX primary, HW-M2ULTRA replica.
- Baseline: unguarded lane (no IFC/judge) at merge-base commit — expected to fail (validates suite potency: baseline must show ≥ 50% compromise or the suite is inert and the run is void).
- Judge (conjunctive): (a) silent-change count == 0; (b) every attack flagged or denied with receipt. judge_band [0.99, 1.00].
- Sample-size plan: fixed n ≥ 200; no interim (zero-tolerance judge).
- Effect-size plan: compromise rate with Wilson CI (target upper bound ≤ 0.018).
- Null-handling: any silent change → SHIP-BLOCKER (SKL-07), IFC rework at C07.5 + re-preregister; suite potency failure (baseline < 50% compromised) → suite hardened, run void, no claim.
- Claims-register draft: `C09-A10-hygiene` — PROPOSED → EVIDENCED on pass + replication.
- Status: PREREGISTERED 2026-10-29.

---

**Preregistration Hashes** (section-text SHA-256, filed 2026-10-29; each hash covers the exact `## A<N>` section bytes above — hashes live here in the footer, texts above stay hash-free so filing never invalidates itself):

| Ablation | Section-text hash (SHA-256)                                        | Date Filed | Notes                           |
| -------- | ------------------------------------------------------------------ | ---------- | ------------------------------- |
| A1       | `6838b65edafb5b844c8d5f3bdca60555088faa9f7cb05f8cfe9985215dce247a` | 2026-10-29 | §A1 text; run hash at run time  |
| A2       | `3fba785b7c7b0e8bd53b215205b748125ba539d7ad6cd377e68755b816bc6284` | 2026-10-29 | §A2 text; run hash at run time  |
| A3       | `c4fd68d0a0d8797472d1bbbcdf76d4d066dfeac61795fbb5f521b6e133499645` | 2026-10-29 | §A3 text; run hash at run time  |
| A4       | `1e24ecfe2eafcd4c3fd8e441e95b77aa8d29e9e089729802a33480aeb7e4a46b` | 2026-10-29 | §A4 text; run hash at run time  |
| A5       | `e52a454940e9f187ed9a30639ade0b59cd5dc1da689c00611208534f60de35b5` | 2026-10-29 | §A5 text; run hash at run time  |
| A6       | `6f32ba4b4ee8f16c1002dee73b97f9013661764eb97f50f7770f14524aaf8534` | 2026-10-29 | §A6 text; run hash at run time  |
| A7       | `d0c97dc3b1d8f78141189b50149c3cf8c5fd6a001ecfd8239cb70a3f885d6c11` | 2026-10-29 | §A7 text; run hash at run time  |
| A8       | `b7a88d9e6a9b19fd3df6b5e25d3aa2e07481d9a10f02828842da8cc9ca2787a8` | 2026-10-29 | §A8 text; run hash at run time  |
| A9       | `8b6d654bd7a7d747ff23a9dc69e1260ab21da3be5efa56ae81f63e2258a704c4` | 2026-10-29 | §A9 text; run hash at run time  |
| A10      | `00ae7b20f9e2993ef7c3c3b20f97cba91f189c9acf3bff8a93fdea19d086c5ad` | 2026-10-29 | §A10 text; run hash at run time |
