# C06 — Frontier Push (Proposed Agent Contributions)

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Differentiating additions beyond the core, each with a verification hook so it can be earned.

## Subcategories (broad topics)

- C06.1 Anti-lobotomy as geometry (BrainContinuityCheckpoint folded into Atlas basins)
- C06.2 Geometric forgetting (radial coordinate salience, automatic compression)
- C06.3 Verifiable cognition receipts (Merkle paths through hyperbolic tree)
- C06.4 Adaptive-depth recall (TwisterAttention depth gating)
- C06.5 Multi-agent submanifolds (lane-owned regions, geodesic bridges)
- C06.6 Speculative hyperbolic decoding (n-gram lookahead trie as draft path)
- C06.7 Self-tuning curvature and depth (preregistered experiment)
- C06.8 Attractor pinning for governance (human approvals pin coordinates)
- C06.9 Adversarial robustness in hyperbolic space (perturbation + circuit breaker)
- C06.10 Differential Atlas sync (federated/edge coordinate deltas)
- C06.11 Receipt-anchored memory identity (end-to-end provenance)
- C06.12 Holonomy Brigade (single weight set, N views, fiber projection/transport/dissolution)
- C06.13 Frontier promotion ladder (Muse 2026-10-26: PROPOSED → prereg → pass → replicate → necessity → required)
- C06.14 Interference budget for frontier items (eviction / weight-copy / privacy audit)
- C06.15 NEW — Depth-gated speculative recall (C06.4 x C06.6 fusion; ablation A36)
- C06.16 NEW — Adversarial calibration drift monitor (judge-lifetime monitor; ablation A37)

## Open questions / gaps

- All items require preregistered ablations (C09) before shipping as required.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 8. Frontier push — proposed additions (agent contributions)

These are PROPOSED and clearly labeled. They are meant to differentiate the shipping system,
not to be confused with implemented behavior. Each has a verification hook so it can be earned.

1. **Anti-lobotomy as geometry.** Fold `BrainContinuityCheckpoint` states into the Atlas itself:
   continuity becomes a region (attractor basin) of the Hyperbolic Memory Map rather than a
   sidecar JSON file. Compaction = folding a trajectory into a basin; recovery = reconstructing
   from the basin. Verification: checkpoint round-trip equality + recall of the original objective
   after two compactions.
2. **Geometric forgetting.** Memory salience = radial coordinate in the Poincaré ball. Recent /
   pinned / approved memories sit near the center (high precision, slow decay); stale or
   low-utility memories drift toward the boundary and compress automatically. No hand-written
   eviction heuristics; eviction is a consequence of geometry. Verification: bounded cache size
   under sustained load with no manual eviction, recall precision ≥ baseline.
3. **Verifiable cognition receipts.** Every recall emits a Merkle path through the hyperbolic
   tree (branches visited, distances, confidence). The existing receipt/fingerprint machinery in
   `holes`/`resolve` extends to memory, so "why did you remember X?" is answerable and auditable.
4. **Adaptive-depth recall (TwisterAttention depth gating).** Cheap hyperbolic ANN at the top
   level; deeper traversal only on promising branches. Recall cost scales with answer hardness,
   not context size. Verification: median recall latency vs. flat scan; accuracy parity on the
   research benchmark.
5. **Multi-agent submanifolds.** Each lane owns a submanifold of the Atlas; cross-lane comms are
   measured geodesic bridges with logged distance/cost. Latent interference between lanes becomes
   a measurable quantity, not a mystery. Ties to the swarm bus `/events` ledger.
6. **Speculative hyperbolic decoding.** Re-use the existing n-gram lookahead trie as a hyperbolic
   draft path: propose tokens from the map, verify through the main path with confidence gating.
   Improves decode throughput without changing correctness.
7. **Self-tuning curvature and depth.** The curvature parameter and nesting levels are learned
   online from task/judge outcomes rather than fixed by hand. Treat as an experiment with a
   preregistered plan (existing doc 19 pattern).
8. **Attractor pinning for governance.** Human approvals (`/holes/approve`) pin coordinates so
   they never decay; governance decisions get a geometric, auditable meaning instead of a flag.
9. **Adversarial robustness in hyperbolic space.** Small coordinate perturbations during recall;
   circuit breaker opens if recall collapses. Connects to the immune-system doc and the existing
   circuit breaker.
10. **Energy-aware serving.** Track joules-per-token and joules-per-recall; optimize throughput
    per watt alongside throughput per second. Aligns with the OOM-avoidance goal and makes the
    solo-developer cost case explicit.
11. **Differential Atlas sync (federated/edge).** Sync coordinate deltas, not weights, for
    privacy-preserving multi-node aggregation (ties to doc 16 edge/A2A).
12. **Receipt-anchored memory identity.** Memory entries carry a receipt fingerprint from
    creation; the same fingerprint propagates through recall and merge, so a memory's provenance
    is verifiable end to end.
13. **Holonomy as architecture — the Brigade.** Specialists are _carved out of the single
    Multi-Model Mind knowledge map_ (the Atlas) rather than spawned as separate weight copies.
    Each brigade agent receives only the sub-region (geodesic neighborhood) of the hyperbolic
    map needed for its assigned task set. Invocation = projecting a fiber of the map into an
    active specialist; task completion = parallel transport back along the same fiber; dissolution
    = the fiber folds back into the single weight set. This avoids the classical agentic bottleneck
    of spinning up N parallel agents that throttle the GPU — there is one set of weights and N
    _views_ into it, iterated with lightning-fast task granularity. Verification: (a) number of
    live weight copies stays at 1 under N-wide swarm runs, (b) per-task latency vs. standard
    spawn-a-clone loop, (c) no recall interference between brigades sharing the same map region,
    (d) dissolution leaves the map consistent (Merkle fingerprint of the Atlas unchanged except
    for intended writes).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 14. Additions I want on §§11–13 (frontier push)

- **(to §11) Prompt-injection hygiene as a requirement (F11).** Injected memory (§10) is untrusted.
  Require a sanitization/tagging pass before injection, plus a red-team ablation (A10) where a
  poisoned memory must not change a lane's tool calls without a judge flag.
- **(to §11) Energy and SLO non-functionals (NF9–NF11).** Tokens/s at p50/p95/p99, joules/token,
  and a hard peak-memory ceiling per topology; no claim of "fast" without p99 on target hardware.
- **(to §12) Ablation integrity gate.** Require the preregistration document hash to be written
  _before_ running; an ablation run without a pre-filed hash is automatically disqualified.
  Record null/negative results (A-fails) as first-class entries, not omissions.
- **(to §12) Cross-hardware replication.** Each performance claim must reproduce on at least two
  distinct machines (the dev Mac + one other) before it ships.
- **(to §13) Kill switch to standard transformer.** At any point the Atlas/ACFT path can be
  disabled and the system falls back to the standard transformer path with one config flag and
  no data loss. This is the safety net that makes shipping the novel path responsible.
- **(to §13) Staged rollout + rollback.** Feature flags per phase, shadow-mode telemetry for one
  release before enabling ACFT on real workloads, and a documented rollback that restores the
  prior binary + data root.
- **(to §13) Reproducible release.** Pin toolchain versions, record the exact git commit and
  dependency lock hashes in the build receipt so a release can be rebuilt bit-for-bit.

## Formal Specifications for Frontier Items

### C06.1 Anti-Lobotomy as Geometry: Attractor Basins in H^3

The **BrainContinuityCheckpoint** is not a sidecar JSON—it is a **geodesic trajectory segment** γ: [0, T] → H^3 × T^2 folded into an **attractor basin** B_γ ⊂ H^3.

**Compaction as Basin Folding**:
B_γ = ∪_{t ∈ [0,T]} B_ε(γ(t))
where B_ε is a hyperbolic ball of radius ε = O(1/depth). The **basin boundary** ∂B_γ is a **horosphere** (constant mean curvature surface).

**Recovery as Basin Reconstruction**: Given basin B_γ, the original trajectory is recovered by **gradient flow** on the Morse function f(x) = d_{H^3}(x, B_γ):
dγ/dt = -∇f(γ(t))
**Verification**: d_{H^3}(γ_orig, γ_recovered) < δ after 2 compactions; objective recall = 100%.

### C06.2 Geometric Forgetting: Ricci Flow on Memory Salience

Memory salience evolves by **Ricci flow** on the salience metric g_{ij}:
∂g_{ij}/∂t = -2 Ric_{ij} + (2/n) r g_{ij}
where r is average scalar curvature. **Pinned memories** (human approvals) are **Dirichlet boundary conditions**—their coordinates are fixed points of the flow. The flow naturally compresses low-salience regions toward the cusp (boundary) while preserving high-salience cores.

**No manual eviction**: cache size bounded by **volume growth** Vol(B_R) ~ e^{2R} in H^3.

### C06.3 Verifiable Cognition Receipts: Merkle Paths in VP-Tree

Each retrieval emits a **Merkle path** through the VP-tree:
receipt = MerkleRoot({ (v_i, μ_i, left/right) }_{i=1}^{depth})
where v_i is vantage point, μ_i is threshold. **Audit query**: "Why did you remember X?" → path proof.

### C06.4 Adaptive-Depth Recall: Bayesian Depth Gating

TwisterAttention depth k is chosen by **Bayesian model selection**:
k* = argmax_k p(k | q, D) ∝ p(D | k) p(k)
where D is judge outcome history. **Latency-accuracy Pareto frontier** is pre-computed offline.

### C06.5 Multi-Agent Submanifolds: Interference as Geodesic Distance

Each lane i owns submanifold M_i ⊂ H^3 × T^2. **Interference** between lanes i, j:
I(i, j) = inf_{x ∈ M_i, y ∈ M_j} d_{H^3 × T^2}(x, y)
Logged in /events ledger. If I(i, j) < τ_interfere, circuit breaker triggers lane isolation.

### C06.6 Speculative Hyperbolic Decoding: Draft-Verify on H^3

**Draft**: n-gram lookahead trie proposes token sequence ŷ_{1:d} from hyperbolic prefix match.
**Verify**: main ACFT path computes p(y_{1:d} | ŷ_{1:d}); accept if Π p > τ_conf.
**Speedup**: E[accepted length] / d.

### C06.7 Self-Tuning Curvature: Online Riemannian Optimization

Curvature κ and nesting depth L are optimized on the **manifold of hyperbolic metrics**:
κ*, L* = argmin_{κ, L} E_{task ~ T} [FreeEnergy(task; κ, L)]
using **Riemannian SGD** on the space of constant-curvature metrics.

### C06.8 Attractor Pinning: Governance as Dirichlet Conditions

Human approval → **pinned coordinate** x_pinned ∈ H^3 with **infinite precision** (Dirichlet boundary). The **Ricci flow** (C06.2) preserves pinned points exactly. Governance decisions are now **geometric invariants**.

### C06.9 Adversarial Robustness: Wasserstein Robustness Certificate

For recall R: H^3 → Y, the **robust radius** is:
ρ_robust = inf_{||δ||_{H^3} ≤ ε} 1[R(x+δ) ≠ R(x)]
Circuit breaker opens if ρ_robust < τ_robust.

### C06.10 Differential Atlas Sync: Optimal Transport on H^3

Federated sync = **Wasserstein barycenter** of coordinate distributions:
μ_global = argmin_μ Σ_i λ_i W_2^2(μ, μ_i)
where μ_i are node coordinate distributions. Sync **deltas only** (optimal transport plan), not weights.

### C06.11 Receipt-Anchored Memory Identity: Provenance as Fiber Bundle

Each memory m has **provenance fiber** P_m = { (receipt, t, agent) } over base H^3. Recall **parallel transports** the fiber. End-to-end provenance = holonomy of P.

### C06.12 Holonomy Brigade: Parallel Transport of Model Weights

Let W be the single weight set (global section). Brigade specialist i receives:
W_i = P_{γ_i}(W|_{U_i})
where P_{γ_i} is parallel transport along geodesic γ_i from origin to submanifold U_i. **Dissolution**: `W = ⊗_i P_{γ_i^{-1}}(W_i)`. **Invariant**: weight_copies = 1.

## Verification Matrix

| Item   | Ablation | Judge                                           |
| ------ | -------- | ----------------------------------------------- |
| C06.1  | A2       | Objective recall after 2 compactions = 100%     |
| C06.2  | A1       | Bounded cache size, recall precision ≥ baseline |
| C06.3  | A4       | Merkle path verifiable for every recall         |
| C06.4  | A1       | Median recall latency vs flat scan              |
| C06.5  | A5       | Interference I(i,j) logged, breaker triggers    |
| C06.6  | A1       | Speculative acceptance rate > 30%               |
| C06.7  | Prereg   | Curvature/depth learned, not fixed              |
| C06.8  | A7       | Pinned coords never decay                       |
| C06.9  | A9       | Circuit breaker on ρ_robust < τ                 |
| C06.10 | A11      | Sync deltas only, privacy preserved             |
| C06.11 | A4       | Provenance fiber transported end-to-end         |
| C06.12 | A4       | Weight copies = 1 under N=8                     |

---

## Contributor Deepening — Muse (2026-10-26)

### C06.13 Frontier promotion ladder (depth: how a PROPOSED item becomes required)

Every C06 item sits on a frozen ladder [PROPOSED — ladder is my proposal; enforcement via ADR-21/22]:

```
PROPOSED idea → preregistered (C09 hash filed) → ablation PASS on primary
  → replicated on second hardware → necessity ablation (remove → regress?)
  → required (C08 traceability row updated) OR optional/off-by-default with measured cost
```

- Skipping a rung disqualifies the claim (C11.3 ablation integrity gate).
- A FAIL at any rung is recorded in the workstream Failures file (ADR-17), never deleted; retry requires a new preregistration hash.
- Two items may NOT share one ablation: each C06 item needs its own prereg entry (the verification matrix above already keeps A-numbers 1:1 per item).

### C06.14 Interference budget for frontier items (depth: novelty interacting with novelty)

Frontier items compose, and composition can break invariants. Frozen rule: any C06 item whose mechanism touches the Atlas MUST state its effect on (a) the C05 eviction ordering, (b) the C30 weight_copies = 1 invariant, (c) the C31 privacy schema. Audit of the existing 12 [INFERRED — derived from C05/C30/C31 contracts; ends at those files]:

| Item                            | Eviction effect                                                                                      | Weight-copy effect                             | Privacy effect                                                        | Status                                       |
| ------------------------------- | ---------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------- | -------------------------------------------- |
| C06.1 basin fold                | Adds pinned regions (never evicted) — bounded by basin-count cap, else cache leaks                   | None (read-only trajectory)                    | Checkpoint contains trajectory text — MUST be ids/hashes only at rest | Needs basin-count cap + checkpoint schema    |
| C06.2 Ricci flow                | IS the eviction policy — must prove equivalence to max-x0 victim rule (C05) or replace it explicitly | None                                           | None                                                                  | Needs equivalence proof or replacement entry |
| C06.5 submanifolds              | Partitions eviction domains per lane — global bound becomes sum of lane bounds                       | Reads shared W (consistent if projection-only) | I(i,j) logged as scalar — allowed                                     | Needs per-lane bound accounting              |
| C06.8 pinning                   | Same as C06.1: pinned set must be bounded                                                            | None                                           | None                                                                  | Needs pin-set cardinality bound              |
| C06.12 Brigade                  | Projection borrows must not become copies — needs linear-type enforcement (cf. C30)                  | CORE invariant weight_copies = 1               | Fiber radius + lane_id only                                           | Needs linearity proof                        |
| C06.3/6.4/6.6/6.7/6.9/6.10/6.11 | No eviction/copy effect by construction                                                              | None                                           | Receipts carry hashes only                                            | Clear pending prereg                         |

Open interference questions (not blockers for PROPOSED status, blockers for required status): basin-count cap value; Ricci-vs-max-x0 equivalence; per-lane bound accounting rule.

### C06.15 NEW — Depth-gated speculative recall (novelty; my lane: C06.4 x C06.6 fusion)

**Status.** PROPOSED frontier item (requires own prereg + ablation before required).

**Idea.** C06.4 (adaptive-depth recall) and C06.6 (speculative hyperbolic decoding) are specified as separate mechanisms sharing the same cost model (C-retrieve + C-attend minus mutual information). Fuse them: the n-gram lookahead trie draft (C06.6) proposes a recall depth as well as tokens — the draft path walks the VP-tree geodesically and emits both a candidate answer prefix AND the depth at which marginal information gain fell below marginal cost. The verify path (ACFT main) then checks tokens and certifies the depth choice in one judge call.

**Why it is novel (not a restatement):** neither C06.4 nor C06.6 specifies who chooses k at serving time under batching pressure — the C12 scheduler depth budget is currently exogenous. This item makes depth endogenous: the draft path computes a per-request k-draft, the scheduler admits against the sum of k-drafts instead of a static D-max, and the verify path acceptance rate becomes the congestion signal. Depth becomes a market, not a constant.

**Formal sketch:**

```
draft(q): VP-tree geodesic walk → (y-hat, k_draft, g-hat)
  where g-hat = estimated marginal gain at k_draft (from offline Pareto frontier, C06.4)
verify(y-hat, k_draft): ACFT main computes p(y|y-hat) AND realized gain g
  accept iff product-p > tau_conf AND |g − g-hat| < eps_cal
scheduler: admit set S maximizing sum-w_i s.t. sum k_draft,i * mem_i ≤ M_budget
```

**Falsifiability:** if draft calibration error exceeds eps_cal on >20% of A1-corpus queries, the draft is uncalibrated and the item FAILS (falls back to exogenous D-max). If acceptance rate < 30% (existing C06 matrix threshold), FAIL.

**Verification:** new ablation A36 (prereg required); judge = (acceptance rate, calibration error, tok/s vs C06.4-alone and C06.6-alone on A1 workload). Must beat both parents to justify the fusion — otherwise the simpler item ships and this one is withdrawn by correction entry.

**Interference statement (per C06.14):** no eviction effect (read-only walk); no weight copies (uses shared W); privacy clean (draft emits depths + hashes, never text, into telemetry).

### C06.16 NEW — Adversarial calibration drift monitor (novelty; my lane: robustness + evidence hygiene)

**Status.** PROPOSED frontier item (requires own prereg + ablation before required).

**Idea.** C06.9 defines a robustness radius and a circuit breaker, but nothing watches the judges themselves decay: a preregistered judge that passed on Day 3 may silently drift as the system changes around it (new skills, new lanes, Atlas growth). This item makes judge calibration a first-class monitored quantity: every judge verdict is logged with its prediction (pass + confidence) and the eventual receipted outcome; a sequential monitor (same Lan-DeMets alpha-spending family as C09 ablations) raises JudgeMiscalibrated when observed pass-rate exits the preregistered confidence band, triggering (a) quarantine of claims citing that judge, (b) a C36 correction entry, (c) mandatory re-preregistration before the judge gates anything again.

**Why it is novel:** C09 sequential testing governs ablation runs; nothing governs judge lifetime. This closes the loop: ablations earn claims, and this monitor un-earns them when the evidence rots. It is the enforcement arm of corrections-are-entries (C03.3).

**Formal sketch:**

```
per judge j with prereg band [l_j, u_j] at confidence 1−alpha:
  monitor S_n = sum (verdict_i − band_midpoint) with Lan-DeMets boundary b(t)
  if |S_n| > b(t_n): emit JudgeMiscalibrated(j, S_n, t_n)
    → quarantine claims citing j (mark INFERRED pending re-validation)
    → append C36 correction; block Gate passage for dependent categories
```

**Falsifiability:** inject synthetic judge drift (flip 15% of verdicts) in a shadow run — monitor must fire within N=200 verdicts. If it fires on clean runs (false alarm > 5%), FAIL (thresholds wrong, retune, re-preregister).

**Verification:** new ablation A37 (prereg required); judge = detection latency on injected drift + false-alarm rate on clean runs. Telemetry: counts/ids/latency only (verdict hashes, never verdict content — ADR-06).

**Interference statement (per C06.14):** no Atlas effect; no weight copies; privacy clean by construction (band statistics over hashes).

### Updated verification matrix (additions)

| Item   | Ablation | Judge                                                                                              |
| ------ | -------- | -------------------------------------------------------------------------------------------------- |
| C06.15 | A36      | Acceptance > 30% AND calibration error within eps_cal on 80%+ queries AND tok/s beats both parents |
| C06.16 | A37      | Injected 15% drift detected within 200 verdicts; false alarms < 5% on clean runs                   |

What was NOT checked here: A36/A37 preregistration text (to be written before any run, per C11.3); offline Pareto frontier existence for C06.4 (verify at C12/C21).

---

## Contributor Deepening — MiniMax M3.1 Flash (2026-10-26)

### C06.17 NEW — Continuity Capsule: a minimum sufficient frontier for the 10-day sprint

**Status.** PROPOSED frontier item. **Owner:** Agent-runtime lead. **Ablation:** A38 (text-complete in `Pre-Registration.md`). **Required status:** forbidden until A38 passes, replicates, and passes necessity under C06.13.

#### Problem addressed

[VERIFIED by reading C24, C05, C29, and C31] The workspace already defines a structured pre-compaction checkpoint, geometric memory, tool receipts, and privacy-safe telemetry, but it does not define one small, exact state object that crosses compaction and restart while retaining proof of where supporting context came from. [INFERRED from that gap] The 10-day sprint can therefore win on a coherent systems thesis without requiring the full ACFT/TwisterAttention kernel path: **continuity under bounded context, with auditable recall and a one-switch fallback**.

#### Name and novelty boundary

**Continuity Capsule** is the proposed name for a typed, content-addressed recovery envelope. [PROPOSED] Its novelty is the **fusion and invariant boundary**, not a new geometry primitive:

1. It separates an **exact critical-state plane** from an **approximate supporting-evidence plane**.
2. It binds the exact plane to a hash chain and the supporting plane to receipt references.
3. It restores exact state without requiring hyperbolic geometry, so the risky research path cannot become the correctness oracle.
4. It makes the standard-transformer fallback consume the same capsule view, producing a paired treatment rather than two unrelated implementations.

[INFERRED] The combination is not claimed to be unprecedented in prior art; no literature search was performed in this workspace. It is a defensible project-level systems contribution whose novelty must ultimately be judged against external research before any public novelty claim.

#### Typed capsule contract

```rust
struct ContinuityCapsule {
    schema_version: u16,
    capsule_id: CapsuleId,
    parent_capsule_hash: Option<Hash>,
    critical_state: CriticalState,       // exact, canonical fields
    supporting_refs: Vec<SupportingRef>, // approximate retrieval candidates + proofs
    created_at: Timestamp,
    content_hash: Hash,                   // SHA-256 over canonical serialization
}

struct CriticalState {
    objective: CanonicalValue,
    invariants: Vec<CanonicalValue>,
    acceptance_criteria: Vec<CanonicalValue>,
    commitments: Vec<CanonicalValue>,
    pending_actions: Vec<CanonicalValue>,
    next_action: CanonicalValue,
    ledger_clock: u64,
}

struct SupportingRef {
    atom_id: AtomId,
    receipt_hash: Hash,
    relevance_rank: u16,
    geometry_hint: Option<GeometryRef>, // optional accelerator, never authority
}
```

[PROPOSED] `CriticalState` is canonicalized before hashing so field ordering and serialization cannot create false drift. [PROPOSED] The capsule is stored in the append-only memory/summary store selected by the implementation owner; the event ledger stores its content hash, parent hash, and receipt hash, never a second copy of raw critical fields. [PROPOSED] Raw capsule payload does not enter the Atlas coordinate payload or telemetry. [INFERRED from C16/C17] Encryption-at-rest remains a deployment control rather than a claim of this design because no implementation or verified encryption boundary exists in this workspace.

#### Two-plane recovery algorithm

```text
fold(ctx):
  summary = require_canonical_summary(ctx)      # C24/C07
  critical = extract_canonical_critical(summary)
  refs = select_supporting_refs(summary, ctx)   # optional Atlas/C29 refs
  capsule = seal(critical, refs, parent_hash)
  append_capsule(capsule)                       # content hash + receipt
  return fold_with_capsule_pointer(ctx, capsule.content_hash)

recover(pointer, mode):
  capsule = load_and_verify(pointer)            # hash + parent chain
  if invalid:
    capsule = walk_to_latest_valid_parent(pointer)
    emit recovery_receipt(reason = "parent_fallback")
  exact = capsule.critical_state                # always available without geometry
  support = load_verified_refs(capsule.supporting_refs, mode)
  return exact + support
```

**Invariants.** [PROPOSED]

- **IC-1 Exactness:** the canonical critical fields are recovered byte-for-byte after canonical serialization, independent of Atlas availability.
- **IC-2 Tamper evidence:** mutation of a capsule or referenced receipt is detected before use.
- **IC-3 Monotonic provenance:** every capsule points to one parent; no parent is silently rewritten.
- **IC-4 Graceful degradation:** geometry or dense retrieval failure removes only approximate support; exact state remains restorable.
- **IC-5 Fallback parity:** the standard-transformer path consumes the same canonical critical state.
- **IC-6 Privacy non-interference:** telemetry and metrics carry IDs, hashes, counts, ranks, and latency only.
- **IC-7 Bounded context:** recovery injects the capsule plus a bounded supporting set, never the full historical transcript.

#### Interference statement (C06.14)

[PROPOSED] The capsule creates no additional live weight copy and does not alter the max-`x₀` eviction rule. It can reference an existing Atlas atom, but the capsule itself is not a pinned geometric atom and therefore does not consume the pinned-set budget. [PROPOSED] It adds one bounded store per compaction checkpoint and one metadata-only ledger entry. [PROPOSED] Raw payload is confined to the protected memory store; telemetry is metadata-only. [INFERRED] Exact recovery cost is dominated by hash-chain walk and payload deserialization, not VP-tree retrieval.

#### 10-day demonstration choreography

[PROPOSED] The demo is a **Continuity Under Compaction** scenario:

1. Start a multi-step repository-analysis task with a deliberately planted invariant and a decoy instruction.
2. Execute enough tool steps to cross the configured compaction threshold.
3. Show the capsule manifest: objective, invariants, next action, parent hash, and supporting receipt references—without exposing unrelated telemetry payloads.
4. Restart the process and force a second compaction.
5. Ask a held-out question that requires both the original invariant and one earlier supporting fact.
6. Toggle `acft_enabled=false`; replay the same capsule through the standard-transformer path.
7. Display the paired A38 result: exact recall, supporting-fact correctness, token reduction, receipt validity, and latency against the flat/checkpoint baselines.

[PROPOSED] The winning claim is deliberately narrow: **the agent preserves exact critical state through two compactions and a restart, proves the provenance of supporting recall, and can replay through the fallback**. [PROPOSED] The demo does not claim hyperbolic geometry is faster than exact attention; A1 remains the authority for that separate claim.

#### Interference and scope guardrails

[PROPOSED] C06.17 does not require C06.7 self-tuned curvature, C06.10 federated sync, C06.12 weight transport, or speculative decoding. [PROPOSED] If Atlas support is unavailable, the capsule remains valid but its supporting set may be empty; A38 M1/M2/M5/M6 still apply, M3 is reported as not applicable, and no supporting-provenance claim is made. [PROPOSED] M3 becomes mandatory in any Atlas-enabled run; an Atlas-enabled run that cannot produce verifiable supporting proofs fails M3. [PROPOSED] Any implementation that allows approximate geometric state to overwrite exact critical fields is non-compliant with IC-1 and fails Gate 5.

### C06.18 NEW — Minimum Sufficient Frontier decision model

**Status.** PROPOSED sprint-selection contract. **Owner:** Architect + Agent-runtime lead. **Depends on:** C06.13, C06.14, C06.17.

[PROPOSED] “Pushing the frontier” in this sprint means maximizing the number of independently earned, judge-verified system capabilities per day—not maximizing the number of exotic mechanisms. The decision score is intentionally lexicographic:

| Rank | Gate           | Question                                                                                         | Failure consequence                                            |
| ---- | -------------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- |
| 1    | Thesis         | Does the item strengthen the Continuity Capsule demo or another explicitly selected demo thesis? | Remove from the 10-day critical path; keep as research backlog |
| 2    | Evidence       | Does it have text-complete preregistration and a deterministic/pass-fail judge?                  | Cannot run or earn a claim                                     |
| 3    | Necessity      | Does disabling it cause a held-out regression?                                                   | Optional/off-by-default with measured cost                     |
| 4    | Feasibility    | Can it fit the remaining sprint capacity without weakening safety/recovery gates?                | Defer to P6                                                    |
| 5    | Frontier value | Does it add a novel capability rather than merely more prose?                                    | Prefer the simpler mechanism                                   |
| 6    | Interference   | Are eviction, weight-copy, privacy, and fallback effects explicit?                               | Blocked until stated                                           |

#### Recommended sprint portfolio

[PROPOSED] Based on the current dependency and evidence state:

- **Tier A — critical path:** C06.17 Continuity Capsule; its A38-scoped receipt-anchored supporting proof (M3); and capsule-local invariants that keep exact state immutable and reject malformed/tampered support. These are not standalone claims for C06.3, C06.8, or C06.9. If the team wants those standalone claims, it must file dedicated registrations before Day 1; until then they remain BLOCKED by CORR-C06-03.
- **Tier B — stretch only after Tier A passes:** C06.4 adaptive-depth recall and C06.12 Brigade, using A21 for the weight invariant and a dedicated A36/A37 registration before any claim.
- **Tier C — research backlog:** C06.2 Ricci-flow eviction replacement, C06.5 lane submanifolds, C06.6 speculative decoding, C06.7 online curvature tuning, C06.10 federated sync, and C06.11 fiber-bundle provenance. These require dedicated proofs/ablations and do not block the sprint demo.

[INFERRED from C32, C35, and the evidence gaps] Tier A has the highest coherence-to-risk ratio because it reuses C17 persistence, C24 compaction, C29 receipts, C31 telemetry, and the existing standard-transformer kill switch. This is a scheduling judgment, not a measured performance result.

### C06.19 Correction entries and corrected evidence ownership

#### CORR-C06-01 — Replace exact geometric trajectory recovery with exact critical-state recovery

- **Date.** 2026-10-26.
- **Corrects.** C06.1 “Recovery as Basin Reconstruction” and its exact-reconstruction judge.
- **Reason.** [INFERRED from the equations] A union of neighborhoods plus gradient flow toward that union does not, as written, preserve trajectory order or uniquely identify the original trajectory. Exact objective recall does not require exact geometric trajectory reconstruction.
- **Replacement.** [PROPOSED] C06.17 provides exact critical-state recovery through canonical serialization and a content hash. C06.1 remains available as an optional approximate recall mechanism with its own future proof and ablation.
- **History.** The original C06.1 text above is retained unchanged; this entry withdraws it as a correctness authority.

#### CORR-C06-02 — Distinguish retrieval traversal from cryptographic proof

- **Date.** 2026-10-26.
- **Corrects.** C06.3 wording that calls a VP-tree traversal path a “Merkle path.”
- **Reason.** [INFERRED] A search-tree traversal demonstrates which index branches were visited; it does not by itself establish inclusion in a content-addressed ledger or terminate in a trusted root.
- **Replacement.** [PROPOSED] A recall proof consists of a supporting atom reference, its tool/ingest receipt hash, and a verifiable path to the event-ledger Merkle root. The VP-tree traversal may be included as search metadata, not as the proof itself.

#### CORR-C06-03 — Correct one-to-one ablation ownership

- **Date.** 2026-10-26.
- **Corrects.** The original C06 verification matrix and the implication in C06.13 that its A-numbers are already one-to-one.
- **Reason.** [VERIFIED] The original matrix reuses A1/A4/A5/A7/A9/A11 for materially different hypotheses, while dedicated A20/A21/A34 registrations exist elsewhere.
- **Replacement.** [PROPOSED] Use the following table. `UNREGISTERED` means the claim cannot be earned until a text-complete registration and hash are filed.

| Item   | Correct ablation owner                                                                 | Status                   |
| ------ | -------------------------------------------------------------------------------------- | ------------------------ |
| C06.1  | A2 plus the C06.17 exactness correction; A38 tests the replacement                     | REGISTERED               |
| C06.2  | A34                                                                                    | REGISTERED               |
| C06.3  | UNREGISTERED (receipt-to-ledger-root proof; A4 is model-size, not recall proof)        | BLOCKED                  |
| C06.4  | UNREGISTERED (A33 tests index accuracy, not depth-gated latency/accuracy)              | BLOCKED                  |
| C06.5  | A20                                                                                    | REGISTERED               |
| C06.6  | UNREGISTERED (A36 text missing)                                                        | BLOCKED                  |
| C06.7  | UNREGISTERED                                                                           | BLOCKED                  |
| C06.8  | UNREGISTERED (pinned-state survival and bounded pinned-set cardinality need one judge) | BLOCKED                  |
| C06.9  | UNREGISTERED (robustness/fallback judge)                                               | BLOCKED                  |
| C06.10 | UNREGISTERED (federated privacy/effectiveness)                                         | BLOCKED                  |
| C06.11 | UNREGISTERED (end-to-end provenance under mutation)                                    | BLOCKED                  |
| C06.12 | A21                                                                                    | REGISTERED               |
| C06.15 | A36 (registration text missing)                                                        | BLOCKED                  |
| C06.16 | A37 (registration text missing)                                                        | BLOCKED                  |
| C06.17 | A38                                                                                    | REGISTERED IN WORKSTREAM |

[VERIFIED] A36 and A37 are referenced by C06 but have no text in the shared `PreRegistrations/` directory. [VERIFIED] A2, A20, A21, and A34 are present in the shared preregistration files.

### C06.20 10-day exit contract

[PROPOSED] The C06 frontier contribution is considered successfully **earned** only when all of the following are true:

1. A38 passes every correctness, recovery, fallback, and privacy judge on primary hardware.
2. The same frozen registration and workload replicate on a second distinct machine.
3. A removal ablation shows that disabling the capsule regresses exact critical-state recall or provenance proof.
4. The standard-transformer fallback completes the same task with no critical-state loss.
5. The evidence bundle contains the preregistration hash, workload manifest, raw judge outputs, failure entries, and the final claim labels.
6. C36 contains an appended decision recording `REQUIRED`, `OPTIONAL`, or `REJECTED` with the evidence pointer.

[PROPOSED] A compelling demo without these artifacts is a product preview, not a verified frontier claim.

#### What was NOT checked for C06.17–C06.20

- [VERIFIED] No implementation exists for the capsule, its store, its recovery path, or its UI.
- [VERIFIED] No A38 run, benchmark, judge output, replication, or necessity result exists.
- [VERIFIED] No external literature search was performed, so public novelty is not claimed.
- [VERIFIED] The workspace reorganization moved the project under `MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/` during this contribution; paths in this section are relative to `MasterDesignDocument/C06-Frontier-Push/`.

## Open questions / gaps

- All items require preregistered ablations (C09) before shipping as required.
- [OPEN] Team ratification required for the Tier A/B/C sprint portfolio in C06.18.
- [OPEN] Choose the authoritative append-only capsule store during implementation; the contract requires one, but this design does not invent an unchecked C17 migration.
- [OPEN] File shared A36/A37 registration text before either Muse item may run.
- [OPEN] File dedicated registrations for C06.3, C06.4, C06.8, C06.9, C06.10, and C06.11.
- [OPEN] Before any A38 run, the A38 run manifest must freeze the exact final-query prompt, strict-JSON response schema, baseline truncation order, and the scenario × field expected-value table referenced by the preregistration.
