# C09 — Ablations, Proofs & Claims

**Status.** PROPOSED (methodology). **Parent:** Master Design Document index.

## Purpose

Every performance/quality claim must be backed by a pre-registered ablation with a passing judge.

## Subcategories (broad topics)

- C09.1 Ablation integrity gate (prereg doc hash before run; null/negative results recorded)
- C09.2 Ablation matrix A1–A10 (KV cache, recall, retrieval, model-size, agent count, context budget, hooks, RAG ingest, memory injection, prompt injection)
- C09.3 Proof artifacts (hardware/settings/workload/comparison, receipts, pre-registration under AtomaDB/packs or docs)
- C09.4 Cross-hardware replication (dev Mac + one other)
- C09.5 Necessity vs sufficiency testing (remove component → judge regression)
- C09.6 Claims register format (VERIFIED/REPORTED/INFERRED/PROPOSED labels)
- C09.7 Failure ledger and corrections-as-entries

## Open questions / gaps

- Necessity ablations not yet run for any component.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 12. Ablations and Proofs (how we earn claims)

Every performance/quality claim in this doc must be backed by a pre-registered ablation with a
passing judge. Use the existing doc-19 preregistration pattern (409 without plan, 400 without
problem, pass with plan).

### 12.1 Proposed ablation matrix

| ID  | Axis under test  | Baseline                   | Treatment                                                | Judge (must pass)                                                                                       |
| --- | ---------------- | -------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| A1  | KV cache         | Standard transformer O(N²) | Nested Hyperbolic Poincaré cache (§5)                    | Throughput (tok/s) + peak memory on fixed prompt corpus, with baseline number on same hardware/workload |
| A2  | Memory recall    | Flat context dump          | Attractor-basin compaction (§8.1)                        | Recall of original objective after 2 compactions = 100%                                                 |
| A3  | Retrieval        | BM25-only                  | BM25+dense (qvec when embed configured)                  | Top-1 doc identical or better on labeled queries; no false-dense claims                                 |
| A4  | Model size agent | Single model               | Specialist fibers from one weight set (§8.13 / holonomy) | Live weight-copy count = 1 under N=8 swarm; per-task latency vs spawn-a-clone                           |
| A5  | Agent count      | 1 lane                     | N lanes via swarm                                        | Judge-gated task pass rate; throughput-per-watt                                                         |
| A6  | Context budget   | 200-line default           | 8k / 131k budgets                                        | Accuracy vs. token cost curve                                                                           |
| A7  | Hooks            | Manual                     | Deterministic pre/post gating                            | Veto correctness (unauthorized merge denied, authorized merge allowed)                                  |
| A8  | RAG ingest       | Read-only RAG              | + ingest route (this pass)                               | Round-trip: ingest text → query returns it; receipt present                                             |
| A9  | Memory injection | No injection               | Auto-inject canonical memory (§10)                       | Agent can state a prior decision from a different lane without re-prompting                             |

### 12.2 Proof artifacts

- All numbers shipped in docs must include hardware, settings, workload, and comparison number.
- Pre-registration stored under `AtomaDB/packs/` or `docs/`, with judge results recorded.
- Receipts (Merkle + fingerprint) for every claim-affecting change.
- A failing ablation is recorded as a failure entry, not hidden.

## Statistical Rigor for Ablations

### Sequential Testing with Alpha Spending

Ablations use **group sequential design** with **Lan-DeMets alpha spending** to allow early stopping without inflating Type I error:

$$\alpha(t) = \alpha \cdot t^\rho$$

where $t \in [0,1]$ is information fraction, $\rho = 2$ (conservative). At each interim analysis (every 25% of planned samples), we compute conditional power and stop for futility if $< 0.2$.

### Bayesian Model Comparison for Necessity

For **necessity testing** (C13.1), we compute **Bayes factor** comparing:

- $H_0$: Component is necessary (removal causes regression)
- $H_1$: Component is not necessary (no regression)

$$\text{BF}_{01} = \frac{p(\text{data} | H_0)}{p(\text{data} | H_1)}$$

With prior $p(H_0) = p(H_1) = 0.5$. **Decision rule**:

- $\text{BF}_{01} > 10$: Strong evidence for necessity → component REQUIRED
- $\text{BF}_{01} < 0.1$: Strong evidence against necessity → component OPTIONAL
- Otherwise: INCONCLUSIVE → more data needed

### Effect Size Reporting

Every ablation reports **standardized effect size** with confidence intervals:

| Metric            | Effect Size     | CI (95%)   | Interpretation              |
| ----------------- | --------------- | ---------- | --------------------------- |
| Throughput ratio  | Cohen's d       | bootstrap  | $d > 0.8$ = large           |
| Recall accuracy   | Risk difference | Wilson     | $\Delta > 0.1$ = meaningful |
| Memory reduction  | % change        | percentile | $> 20\%$ = significant      |
| Energy efficiency | Ratio           | log-normal | $> 1.2$ = worthwhile        |

### Hardware-Normalized Metrics

All performance numbers are **hardware-normalized** to a reference machine:

$$\text{norm\_metric} = \frac{\text{metric}_\text{target}}{\text{metric}_\text{ref}} \times \frac{\text{spec}_\text{ref}}{\text{spec}_\text{target}}$$

where $\text{spec}$ includes GPU compute (TFLOPS), memory bandwidth (GB/s), and CPU cores.

### Claims Register Schema (Machine-Readable)

```json
{
  "claim_id": "C05-A1-throughput",
  "label": "VERIFIED",
  "statement": "Hyperbolic cache achieves 1.5x throughput at equal peak memory vs standard transformer",
  "evidence": {
    "ablation_id": "A1",
    "hardware": "MacBook Pro M3 Max, 36GB Unified Memory",
    "workload": "Fixed prompt corpus (10k tokens, 100 prompts)",
    "baseline_tok_s": 42.3,
    "treatment_tok_s": 64.7,
    "baseline_peak_gb": 28.1,
    "treatment_peak_gb": 27.4,
    "effect_size": 1.2,
    "ci_95": [0.9, 1.5],
    "p_value": 0.003,
    "bayes_factor": 47.2,
    "replication": "MacBook Pro M2 Ultra, 96GB: 1.4x confirmed"
  },
  "pre_registration_hash": "sha256:abc123...",
  "timestamp": "2026-10-26T14:30:00Z",
  "corrections": []
}
```

### Failure Ledger Entry Schema

```json
{
  "failure_id": "C05-A2-recall-fail",
  "ablation_id": "A2",
  "date": "2026-10-25",
  "hypothesis": "Attractor-basin compaction retains 100% objective recall after 2 compactions",
  "result": "FAIL: 94% recall (6/100 objectives lost)",
  "root_cause": "Basin radius epsilon too small for deep reasoning trajectories",
  "correction": "Increase epsilon from O(1/depth) to O(log(depth)/depth); re-run A2",
  "status": "CORRECTED",
  "follow_up_ablation": "A2-retry"
}
```

### Ablation Execution Protocol

```yaml
# .github/workflows/ablation.yml
name: Ablation Runner
on:
  workflow_dispatch:
    inputs:
      ablation_id:
        type: choice
        options: [A1, A2, A3, A4, A5, A6, A7, A8, A9, A10]
      pre_reg_hash:
        type: string
        required: true

jobs:
  run:
    runs-on: [self-hosted, macos, m3-max] # + replication target
    steps:
      - uses: actions/checkout@v4
      - name: Verify pre-registration
        run: |
          hash=$(cat PreRegistrations/${{ inputs.ablation_id }}.json | sha256sum)
          if [ "$hash" != "${{ inputs.pre_reg_hash }}" ]; then exit 1; fi
      - name: Run ablation
        run: cargo run --release --bin ablation_${{ inputs.ablation_id }}
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ablation-${{ inputs.ablation_id }}-results
          path: ablation_results/
      - name: Record claim/failure
        run: |
          python scripts/record_ablation.py \
            --id ${{ inputs.ablation_id }} \
            --results ablation_results/ \
            --hash ${{ inputs.pre_reg_hash }}
```

### Cross-Hardware Replication Protocol

For each claim, **two distinct hardware targets** must pass:

| Target   | Spec                      | Role                          |
| -------- | ------------------------- | ----------------------------- |
| Primary  | MacBook Pro M3 Max, 36GB  | Development + primary results |
| Replica  | Mac Studio M2 Ultra, 96GB | Independent replication       |
| Optional | Linux + NVIDIA H100       | Architecture portability      |

**Replication criterion**: Effect size direction same, magnitude within 20% of primary.

---

## Contributor Deepening — Muse (2026-10-26)

### C09.8 Regression ablations (depth: earning the right to change a PASS)

Ablations A1–A35 prove components once. Nothing specifies what happens when the system changes under a passed claim — the exact decay C06.16 monitors at runtime. Frozen re-validation rule [PROPOSED — complements C06.16; triggers are my proposal]:

Any of these events invalidates EVIDENCED/REQUIRED status back to PREREGISTERED for affected claims until re-run:

1. Lane roster change (new lane, allowlist change, approval-policy change) → re-run A5, A7, A10, A29, A30.
2. Atlas/cache code change (retrieval, eviction, geometry) → re-run A1, A2, A4, affected C06 ablations.
3. New skill/tool registered → re-run A7 + necessity check on the new component (C13.1).
4. Embed endpoint added/removed/changed → re-run A3 (dense-vs-BM25 boundary moved).
5. Prompt corpus or judge change → re-run every ablation citing that corpus/judge.

The affected-claim list is computed from the traceability matrix (C08), not from memory: claims_citing(changed_component). CI enforces this at Gate 2 (static analysis + unit tests) by diffing configurations.json, skills.manifest.json, tools.manifest.json, and the Atlas kernel sources against the commit recorded in the last build receipt.

### C09.9 Preregistration freeze checklist (depth: what a valid prereg contains)

A filed hash is necessary but not sufficient — the content behind the hash must be complete or the run is disqualified on audit even with a hash. Frozen checklist; a prereg missing any item FAILS Gate 1 document review [PROPOSED — checklist is my proposal; enforcement via existing Gate 1]:

- [ ] Hypothesis in falsifiable form (treatment X beats baseline Y on metric M by ≥ δ — δ stated numerically).
- [ ] Workload: fixed corpus/benchmark ID + seed(s) + generation parameters.
- [ ] Hardware: primary + replica target IDs (C09.4); normalization formula inputs recorded.
- [ ] Baseline: exact commit + config flag state (e.g. kill-switch flag value) of the comparison path.
- [ ] Judge: pass/fail predicate with thresholds, confidence band (feeds C06.16 monitor), and sample-size plan with alpha-spending interim points.
- [ ] Effect-size plan: metric-specific effect size + CI method (per the C09 effect-size table).
- [ ] Null-handling: pre-stated interpretation of each plausible null/negative outcome (which design decision each null would trigger).
- [ ] Claims-register draft: the claim IDs this run could create, in PROPOSED state.

What was NOT checked here: current PreRegistrations/A1-A10 content vs this checklist (audit at PreRegistrations pass); A11–A35 coverage of the C06.15/A36 + C06.16/A37 additions (prereg text for A36/A37 still to be written).

## Open questions / gaps

- Necessity ablations not yet run for any component.
