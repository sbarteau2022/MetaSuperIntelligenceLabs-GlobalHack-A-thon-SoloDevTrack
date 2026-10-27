# C13 — Testing for Necessity, Benchmarking, Tools/Skills, Self-Improving Harnesses

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Make necessity, benchmarking, and self-improvement first-class and measurable rather than asserted.

## Subcategories (broad topics)

- C13.1 Necessity testing (component removal causes held-out judge regression; otherwise optional/off-by-default)
- C13.2 Benchmarking in toroidal systems (cycle latency, wrap-around penalty, per-node imbalance, heat map, tail latency; worst arc not average)
- C13.3 Tools and skills contract (typed registry, skill tree depends_on/topo sort, risk level, params schema, JSON result, telemetry, receipt hash; high-risk hole/approval; provenance; revocation)
- C13.4 Self-improving harnesses (proposals only land in isolated holes; judge-verified before merge; introspection loop; guardrails; held-out judge improvement without prior regression)
- C13.5 Test matrix as first-class artifact (property tests on hole/merge/Merkle; deterministic mock-lane loop test)
- C13.6 Property/invariant tests and fuzzing

## Open questions / gaps

- Toroidal benchmarks require ring-arranged workloads; harness not yet built.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 16. Testing for Necessity, Benchmarking, Tools/Skills, Self-Improving Harnesses (PROPOSED)

### 16.1 Testing for necessity, not just sufficiency

- **Sufficiency test:** "does the whole system pass with component X present?" (current bar).
- **Necessity test:** "does removing/disabling component X cause a measurable regression on a
  held-out judge?" If not, X is not necessary and should be deleted or made optional.
- Apply to every feature: memory injection (§10), holonomy brigade (§8.13), ACFT (§5), hooks
  (§2.7), RAG ingest, compaction. Necessity requires a registered ablation (§12) where the
  component is turned off and the judge must fail or regress beyond tolerance.
- Rule: no component ships as "required" unless a necessity ablation passes; otherwise it ships
  as optional with an explicit off-by-default flag and a measured cost.

### 16.2 Benchmarking in toroidal systems

- A toroidal topology means work/attention flows around a closed loop with wrap-around, so cache
  and memory traffic has cyclic structure (no cold-edge bias). Benchmarking must report not just
  average throughput but **per-phase variance around the torus** (min/max ratio, stall hotspots).
- Metrics to add: cycle latency, wrap-around penalty, per-node (lane) imbalance, memory-traffic
  heat map around the ring, tail latency under cyclic load.
- Use the existing pressure-test + load-smoke harness (docs 18/19) but arrange agents/workloads
  in a ring and report wrap-aware numbers; a claim of "fast" must include the worst arc, not the
  average.
- Safety: toroidal retry patterns can spin; tie to retry signatures/idempotency and circuit
  breaker to avoid unbounded looping.

### 16.3 Tools and skills

- **Canonical tool registry** remains the typed surface (§3 Execution Layer); skills layer over it
  via skill tree (depends_on, topo sort, lazy/eager).
- Every skill/tool requires: name, description, risk level (low/medium/high), typed params schema,
  JSON-only result, telemetry (name/ok/latency/risk only), receipt hash.
- High-risk (write/terminal/hole/ingest) require hole binding or approval gate, never implicit.
- Skill activation is scoped to the brigade's geodesic neighborhood (§8.13) to avoid pulling
  unneeded map regions into context.
- Tool/skill provenance must be recorded so an agent can answer "which tool produced this?" and
  the system can revoke a compromised skill without rewriting history.

### 16.4 Self-improving agentic harnesses and agent systems

- Harnesses may _propose_ modifications to skills/prompts/configs, but mutations land only in an
  isolated realm (hole) and are verified by judges before merge — mirroring the D-21 self-evolving
  harness rule (metatheory ≠ permission slip).
- Retain reusable experience across runs (memory map + mind tail) and replay it into future runs;
  measure improvement as judge-pass-rate delta over a fixed benchmark, not self-reported better.
- Introspection loop: after each run, deposit the structured §10.2 summary (brainstorm, attempts,
  plan, executed, conclusions, coming plan); feed failures into a triage queue (queues doc 13).
- Guardrails: depth/budget caps on subagents, circuit breaker on repeated failure, human approval
  for any mutation crossing lane boundaries, Merkle receipt on every accepted self-modification.
- Verification: a self-improvement must improve a held-out judge without regressing a prior judge;
  otherwise it is rejected and recorded as a failure.

## Formal Testing Framework

### C13.1 Necessity Testing as Counterfactual Causal Inference

**Necessity** of component $X$ is defined via **counterfactual**:

$$\text{Necessary}(X) \iff \mathbb{E}[Y_{do(X=0)}] < \mathbb{E}[Y_{do(X=1)}] - \delta$$

where $Y$ is judge score, $do(X=v)$ is intervention setting $X$ to value $v$, and $\delta$ is minimal meaningful difference.

**Estimation via ablation**: $\mathbb{E}[Y_{do(X=0)}] \approx \frac{1}{n}\sum_{i=1}^n Y_i^{(X=0)}$ (held-out judge).

**Decision rule**: If 95% CI for $\mathbb{E}[Y_{do(X=1)}] - \mathbb{E}[Y_{do(X=0)}]$ excludes $\delta$, then $X$ is NECESSARY.

### C13.2 Toroidal Benchmarking as Circular Statistics

The **toroidal topology** means workload phases $\theta \in [0, 2\pi)$ with wrap-around.

**Metrics as circular random variables**:

| Metric              | Circular Statistic    | Formula                                                              |
| ------------------- | --------------------- | -------------------------------------------------------------------- |
| Cycle latency       | Mean resultant length | $\bar{R} = \|\frac{1}{n}\sum e^{i\theta_k}\|$                        |
| Wrap-around penalty | Circular variance     | $V = 1 - \bar{R}$                                                    |
| Per-node imbalance  | Rayleigh test         | $R = n\bar{R}^2$ (p-value for uniformity)                            |
| Stall hotspots      | KDE on circle         | $\hat{f}(\theta) = \frac{1}{n h}\sum K(\frac{\theta - \theta_k}{h})$ |

**Worst-arc reporting**: Report $\max_{\theta \in [0,2\pi)} \text{latency}(\theta)$ and the arc $[\theta_{start}, \theta_{end}]$ containing it.

### C13.3 Tools/Skills as Capability Type System

```rust
// Tool capability types
trait Tool {
    type Params: Serialize + Deserialize + Schema;
    type Result: Serialize + Deserialize + Schema;
    const RISK_LEVEL: RiskLevel;
    const TELEMETRY: TelemetrySpec;
}

#[derive(Serialize, Deserialize, Schema)]
struct TelemetrySpec {
    name: String,
    ok: bool,
    latency_ms: u64,
    risk_level: RiskLevel,
    receipt_hash: Hash,
}

// Skill as monoid over tools
trait Skill {
    type Tools: ToolSet;
    fn compose(self, other: Self) -> Self where Self: Sized;
    fn activate(&self, neighborhood: GeodesicNeighborhood) -> SkillInstance;
}

// Provenance as linear type
struct Provenance {
    tool_id: ToolId,
    skill_id: SkillId,
    agent_id: AgentId,
    receipt_chain: Vec<Hash>,
    revoked: bool,
}
```

### C13.4 Self-Improving Harness as Verified Optimization

The **self-improvement loop** is a **verified optimization** on the space of configurations $\mathcal{C}$:

$$\mathcal{C}^* = \arg\max_{c \in \mathcal{C}} \mathbb{E}[\text{JudgePassRate}(c)]$$

subject to:

- **Safety**: $\forall c' \in \text{path}(c_0 \to c): \text{JudgePassRate}(c') \geq \text{JudgePassRate}(c_0) - \epsilon$
- **Guardrails**: Depth $\leq D_{max}$, Budget $\leq B_{max}$, CircuitBreaker not triggered
- **Human approval**: Required for any mutation crossing lane boundaries
- **Audit**: Merkle receipt on every accepted mutation

**Verification theorem**: If the loop terminates with $c^*$, then:
$$\text{JudgePassRate}(c^*) \geq \text{JudgePassRate}(c_0) \quad \text{and} \quad \forall j \in \text{held-out}: \text{Judge}_j(c^*) \geq \text{Judge}_j(c_0)$$

### C13.5 Test Matrix as TLA+ Model Checking

The **test matrix** includes **model-checked invariants**:

| Invariant                             | TLA+ Spec | Model Checked | Fuzzed |
| ------------------------------------- | --------- | ------------- | ------ |
| Hole merge preserves Merkle roots     | ✓         | 10^6 states   | ✓      |
| Compaction restores pinned messages   | ✓         | 10^5 states   | ✓      |
| Memory injection causal consistency   | ✓         | 10^5 states   | ✓      |
| Workflow packet DAG acyclicity        | ✓         | 10^4 states   | ✓      |
| Skill activation neighborhood scoping | ✓         | 10^4 states   | ✓      |
| Tool provenance chain integrity       | ✓         | 10^5 states   | ✓      |
| Deterministic mock-lane loop          | ✓         | 10^6 states   | ✓      |

### C13.6 Property/Invariant Tests as QuickCheck Generators

```rust
// Hole/Merkle property tests
#[quickcheck]
fn hole_merge_merkle_preservation(
    source: HoleState,
    hole: HoleState,
    approval: ApprovalReceipt
) -> TestResult {
    let merged = merge_holes(source, hole, approval)?;
    TestResult::from_bool(
        MerkleRoot(merged) == ComputeMergedRoot(source, hole, approval)
    )
}

#[quickcheck]
fn compaction_restores_pinned(
    context: Context,
    pinned: Vec<Message>
) -> TestResult {
    let compacted = compact(context, 0.6)?;
    TestResult::from_bool(
        compacted.pinned_messages == pinned &&
        compacted.checkpoint.objective_recall == 1.0
    )
}

// Fuzzing: AFL++ on API endpoints
# fuzz_targets:
#   - api_chat_completions
#   - api_tools_call
#   - api_holes_create
#   - api_workflows_submit
#   - api_rag_ingest
```

### Self-Improvement Verification Protocol

```yaml
# self_improve_verification.yml
improvement_id: "improve-20261026-001"
base_config: "config.v12"
proposed_config: "config.v13"
mutation:
  type: "skill_addition"
  skill: "hyperbolic_sketch_retrieval"
  target_lane: "researcher"
guardrails:
  depth_cap: 5
  budget_cap_ms: 5000
  circuit_breaker: true
  human_approval: true # cross-lane
judges:
  held_out: ["reasoning_bench_v2", "coding_bench_v1"]
  prior: ["regression_suite_v3"]
results:
  held_out_pass_rate_delta: +0.07
  prior_pass_rate_delta: -0.01
  verdict: "ACCEPT" # held-out improved, no prior regression
receipt: "merkle:sha256:abc123..."
```

---

## Contributor Deepening — Muse (2026-10-26)

### C13.7 Necessity-ablation scheduling (depth: when removal tests run)

C13.1 defines necessity (remove → regress) but no schedule: necessity ablations compete with feature work for the same hardware. Frozen scheduling rule [PROPOSED — schedule is my proposal; the necessity definition is frozen]:

- Every component promoted toward REQUIRED gets its necessity ablation scheduled in the same phase as its sufficiency ablation (C08.6 EVIDENCED stage carries both results).
- Necessity ablation reuses the sufficiency workload + judge with the component disabled via its off-flag (kill switch for ACFT, config flag for lanes/hooks/RAG). No separate workload allowed — same corpus, same hardware, same seeds, or the comparison is invalid.
- Minimal meaningful difference δ is stated in the prereg (C09.9), not chosen after seeing the numbers.
- A component whose necessity ablation is INCONCLUSIVE (Bayes factor between 0.1 and 10, C09) stays OPTIONAL/OFF-BY-DEFAULT through the sprint; re-test post-sprint, do not promote on a hunch.

### C13.8 Self-improvement harness vs C06.16 monitor (depth: two loops, one contract)

C13.4 allows harnesses to propose config mutations; C06.16 monitors judge calibration. They compose into one contract [PROPOSED — composition is my proposal]:

1. An accepted self-modification (C13.4 receipt) INVALIDATES the calibration baseline of every judge that evaluated it — the judge's prereg band was earned on the old config. The C06.16 monitor restarts its sequential test at t=0 for affected judges.
2. The self-improvement verdict (ACCEPT/REJECT) is itself a judge verdict fed into the C06.16 monitor: a harness whose ACCEPT rate exits its prereg band gets quarantined like any miscalibrated judge.
3. Self-modifications that touch the Atlas, lane roster, or tool registry additionally trigger the C09.8 regression re-validation list.

Without this composition, the harness can silently walk the system out from under its own evidence — improvement theater with receipts.

## Open questions / gaps

- Toroidal benchmarks require ring-arranged workloads; harness not yet built.
