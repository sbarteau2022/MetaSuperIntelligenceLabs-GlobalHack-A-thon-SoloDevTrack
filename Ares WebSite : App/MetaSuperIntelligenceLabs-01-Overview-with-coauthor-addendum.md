# Hackathon / Enterprise Design — Ares SpecOps Agentic Lab (from first principles)

**Status.** PROPOSED scaffold for the enterprise track; Difference 1 now fully specified by the user.
This is a _design_ document for the _next_ build, not a description of the current local-only app.

**Purpose.** Replace the ad-hoc, piecewise Ares app with a single enterprise-grade, first-principles
design scoped up front, so the sprint ships a coherent system instead of accumulated parts.
The current Ares app is the experimental local-only prototype that this design supersedes.

---

## 1. Scope and non-goals

- **In scope (local prototype → enterprise):** agent lanes, RAG + ingest + memory, workflow/hole
  orchestration, multi-agent comms, telemetry, recovery, governance, model serving, security,
  compliance.
- **Out of scope for this scaffold pass:** code implementation of the model server; numbers not
  measured yet; TEE/attestation (excluded on Apple Silicon stack).
- **Deliverable:** this design doc + implementation passes with explicit verification gates.

---

## 2. Differences vs. the current ad-hoc app

### Difference 1 — KV cache: O(N²) → Nested Hyperbolic Poincaré Dynamic Phase-State Space

1. **Problem in the current transformer path.** Every generated token re-runs a full forward pass
   over the entire context, giving O(N²) cost and O(N) memory growth. That is the OOM ceiling
   that keeps solo developers from shipping enterprise-grade systems.
2. **Target.** Replace the flat attention KV cache with a _Nested Hyperbolic Poincaré Dynamic
   Phase-State Space_. Memory is stored as coordinates in a hyperbolic memory map; retrieval is
   geometric, so the system builds only what it needs to recall and complete the task, not a
   full forward pass over everything irrespective of reasoning depth.
3. **Where it nests.** _Inside the Atlas_ — the Multi-Model Phase State Space. The Atlas is the
   spatiotemporal/phase-state container that stores memory as coordinates in a Hyperbolic Memory
   Map. The cache is a plug-in: it attaches to the Atlas rather than replacing the whole stack.
4. **Plug-in mode.** The Multi-Model Mind can run either:
   - (a) the standard transformer (frontier-class knowledge for the agentic team at scale), or
   - (b) the plug-in hyperbolic phase-state cache (nested Poincaré) for memory/computation.
     Either mode avoids the soul-crushing OOM that blocks enterprise-grade solo shipping.
5. **Cognition upgrade — Active Inferencing Transformer (ACFT).** Standard transformers hold
   frozen weights in the normal sense. ACFT does not. It uses _TwisterAttention_, which builds
   only what is needed to retrieve from the map and to complete the task: no full forward pass
   for everything without respect to the depth of the reasoning + recall. Weights/recall are
   active-inference-driven, not statically frozen.

6. **Telemetry and diagnostics are first-class.** Tooling and diagnostics are built-in engineering
   principles, not bolted-on dashboards. Token cost, latency, throughput, cache hit/miss, recall
   depth, and OOM headroom are measured and emitted from the serving core.
7. **Security and compliance are first-class.** They are engineered _before_ the first feature is
   built, not installed as tools or run as outside processes. Role access, Merkle receipts,
   audit logs, secret-by-slot, and regulatory mapping are part of the core contract.
8. Active Memory State Mangement and Forward Iterative Focus
9.

---

## 3. First-principles engineering backbone- Active Inference Aware as first class dynamic systems architceture - Minimizing Variational Free Energy while Optimizing for an aligment of Resource Consumption-to-QualityOutput.

1. **Verifiable execution:** every mutating action runs in a hole, produces a receipt (Merkle
   fingerprint + judge verdict), and is approval-gated where required.
2. **Evidence over claims:** no metric without workload/hardware/baseline; no badge without a
   receipt.
3. **Append-only truth:** events/workflows/ledgers reject updates and deletes; corrections are
   appended, never rewritten.
4. **Least privilege:** tools are data; risky execution stays behind seatbelt + approval; secrets
   by slot, never literal.
5. **Determinism:** resolve v2 validates tool refs deterministically; receipts are hash-derived.
6. **Cascade memory:** Org Policy → User → Project Rules → Session Auto; tight context budget.
7. **Security/compliance before build:** threat model, role matrix, audit log, right-to-erasure
   plan precede feature code.

---

## 4. Comprehensive architecture tree

```
Ares Enterprise (hackathon scope)
├── 1. Input Layer
│   ├── Session/gatekeeping, permission tiers, endpoint registry, secrets slots
│   ├── Agent configurations (N lanes, not fixed 3), rag_default, rag_collection
│   └── IDE/terminal entry, revision-gated saves
├── 2. Knowledge Layer
│   ├── Skill registry + tree (depends_on, topo sort, lazy/eager)
│   ├── Hooks (event map, blocking veto, receipts)
│   ├── Mind (5W run summaries, role-gated)
│   ├── RAG (in-process, BM25+dense, 7 collections) + ingest (Ingest role)
│   └── Workflows/task graph (packets, judges, retry, escalation, preregistration)
├── 3. Execution Layer
│   ├── Typed tool registry (built-in + code-intel + git write + web)
│   ├── Hole substrate (create/run/diff/approve/merge, Merkle guards)
│   ├── Context budgeting + compaction (60% fold, post-compaction restart)
│   └── Code intelligence (Tree-sitter/LSP) gated by settings
├── 4. Integration Layer
│   ├── MCP runtime (lazy sessions, stdio/HTTP, preset catalog)
│   ├── Cross-tool import (read-only scan + explicit import)
│   └── Endpoints (OpenAI-compatible, embed lane discovery)
├── 5. Multi-Agent Layer
│   ├── Lanes (configurable, not fixed)
 |     |-----Hyperbolic mMap -- Siloed Multi-Agent Temporary Allocation
 |     |-----Agentic Harness Theory
│   ├── Autorunner (N workers, ready-packet admission)
│   ├── Subagent loops (holes as substrate, depth/budget caps)
│   ├── Parallel orchestration (run_workflow: agent/parallel/pipeline/phase)
│   ├── Branch isolation (holes / worktrees)
│   └── Holonomy Brigade (specialists carved from the Atlas as fibers, not weight clones; dissolve back)
├── 6. Model Serving Layer (NEW — from Difference 1)
│   ├── Standard transformer path (frontier knowledge, agentic team at scale)
│   ├── Atlas — Multi-Model Phase State Space
│   │   └── Hyperbolic Memory Map (Poincaré ball coordinates)
│   ├── Nested Hyperbolic Poincaré Dynamic Phase-State cache (plug-in)
│   ├── TwisterAttention (Active Inferencing Transformer, ACFT)
│   ├── Telemetry/diagnostics as first-class (tok/s, cache hit, recall depth, OOM headroom)
│   └── Security/compliance as first-class (RBAC, audit log, receipts)
├── 7. Observability Layer
│   ├── Event ledger (SQLite, append-only, monotonic)
│   ├── Telemetry (process + persisted session, honest availability)
│   ├── Metrics (Prometheus text), taps, receipts, fingerprints
│   └── Neural Map aggregation (end-of-day roll-up)
└── 8. Recovery & Governance
    ├── Circuit breakers, retry signatures, idempotency keys
    ├── Rule set (30-rules-and-governance), decision/success/failure logs
    └── Compliance smoke + AI Act/GDPR mapping
```

---

## 5. Atlas / Hyperbolic Memory Map (detail)

- **Atlas** = the Multi-Model Phase State Space that stores memory as coordinates in a
  Hyperbolic Memory Map. It is the container; the KV cache plug-in nests inside it.
- **Poincaré ball** geometry provides exponential capacity for hierarchical data with
  sub-linear parameter growth vs. flat Euclidean attention. NOT JUST A POINTCARE-- MUST GET THE MAP AND THE SYSTEMS PROOF FROM THE DOCS ON DISC AND REDO

- **TwisterAttention** retrieves from the map on demand and completes the task with bounded
  depth-of-reasoning respect; no blanket full forward pass.
- **Fallback** to exact attention for safety-critical spans until the hyperbolic path is
  benchmarked and receipted.

---

## 6. Implementation order (sprint, 10 days)

1. Day 1–2: freeze first-principles constraints + this design (done for Difference 1).
2. Day 3: wire remaining internal scaffolds (hooks, skill tree, lazy MCP, compaction) — backend.
3. Day 4–5: RAG + memory + ingest for all lanes (backend done this pass).
4. Day 6–7: agent tool loop + reliability smoke + browser E2E green.
5. Day 8–9: model serving (Atlas / TwisterAttention / KV cache plug-in) in rust-server.
6. Day 10: compliance/observability audit + go/no-go.

---

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

## 10. Memory State Management — AtomaDB-RAG auto-injection (PROPOSED)

**Concept.** One canonical memory home for the whole swarm. Every agent reads from the same
AtomaDB-RAG memory store on session start and writes its structured session summary back into
it. No agent is memoryless on reinvocation, and no agent holds a private memory island.

### 10.1 Auto-injection at session/file-tab start

- When an agent opens a new IDE file tab (or session boot), the backend builds the system prompt
  with an injected memory prefix pulled from AtomaDB-RAG (`memory` collection, scoped by
  project/lane, newest-first, bounded by the 200-line context budget).
- The injection is automatic and non-optional: the agent always starts with the canonical
  collective memory report, not just its own history.
- Source: `MindClient::tail` (existing) over the `mind` store, optionally augmented with RAG
  retrieval over `papers/research/knowledge` for evidence relevant to the task.
- Provenance: each injected block is tagged with lane + run_id + receipt fingerprint so the
  model knows which memories are its own vs. collective.

### 10.2 Mandatory session summary structure (agent-authored, appended to RAG memory)

Every agent, at session end (or pre-compaction), MUST deposit a structured 5W-derived summary
via `memory_deposit` / `POST /mind/submit` (and optionally `rag_ingest` into the `research`
collection) with these sections:

1. **Brainstorm summary** — what was considered, options, discarded ideas.
2. **Attempts report** — what was tried, parameters, tool calls, runtime, failures, root causes.
3. **Planning / preregistration** — predicted outcomes and judges (links to packet
   `preregistered_plan`), what would falsify the plan.
4. **Executed** — what actually ran, with notes on success and failures (receipt IDs, holes,
   verdicts).
5. **Conclusions** — what was learned, decisions made, invariants established.
6. **Coming plan** — next steps, open questions, hand-offs to other lanes.

These map onto the existing `FiveW` (who/what/where/when/why) plus `run_id`, `lane`,
`project` fields already in `MindLine`, so no new storage schema is required; the sections are
conventions in the `what`/`why` text plus `kind` tags.

### 10.3 Canonical home and collective reinvocation

- All summaries land in **one canonical home** (the AtomaDB-RAG memory store: `mind.jsonl` in
  the library-db root, plus RAG collections for research-paper-ingest artifacts). No per-agent
  silo, no duplicate ledgers.
- On reinvocation of any agent, the backend aggregates the **full memory report summary from
  collective ingestion** (latest live records across all lanes, filtered by project, ordered
  newest-first) and injects it into that agent's system prompt. Every agent re-enters with the
  team's combined experience, not a blank slate.
- Cross-lane visibility: unscoped records (project "") are visible to every frame; scoped records
  follow the project rule already in `MindClient::tail`.

### 10.4 Enforcement and receipts

- The summarize-before-compaction rule (60% fold) becomes a code-enforced gate: an agent that
  ends a session without depositing the structured summary records a `summary_missing` receipt
  and is flagged in `/metrics` / the swarm board. (Currently documented only; wiring PROPOSED.)
- Each summary deposit is a receipt (mind record id + receipt fingerprint) so the memory's
  provenance is auditable end to end, consistent with the existing hole/Merkle receipt pattern.
- Telemetry for memory events carries ids and scopes only, never summary text (unchanged privacy
  rule from `mind.rs`).

### 10.5 Honest limits

- The auto-injection wiring at file-tab open is PROPOSED (backend currently has no file-tab-open
  hook; injection point is in `lab::start_turn` / `ide` open path, TBD).
- The canonical-home ingest path (`rag_ingest` + `memory_deposit`) exists as of this pass; the
  _mandatory_ summary gate is not yet enforced.
- Dense retrieval over memory is BM25 today unless an embed endpoint is configured (honest).

## 11. Framework Requirements (PROPOSED — contract before code)

### 11.1 Functional

- F1 Lane roster is data-driven (N agents), not 3 fixed slots; per-agent tools, RAG collection, approval policy.
- F2 Memory state auto-injection (§10) at session/file-tab start, bounded by context budget.
- F3 Mandatory structured session summary on every session end (§10.2) or pre-compaction.
- F4 RAG retrieval + ingest reachable by every lane and by tools; ingest role-gated.
- F5 Tool loop in lanes (plan → tool → observe → continue) behind approval.
- F6 Workflows with depends_on, judges, retry, escalation, preregistration; auto-runner admission.
- F7 Hole isolation with Merkle source/hole roots, diff, approve, merge.
- F8 Telemetry/receipts for every tool call, lane turn, judge, and memory event (counts/latency/ids only).
- F9 Compaction fold + post-compaction restart with pinned system/original user messages.
- F10 Streaming transport with measured TTFT (not unmeasured placeholders).

### 11.2 Non-functional

- NF1 No OOM on the 10-day sprint workload: idle + peak Metal/CPU memory below guard; backpressure (permits).
- NF2 Append-only data integrity: events/workflows/ledgers reject update/delete.
- NF3 No secret leakage: grep/scan checks; secrets by slot; telemetry never args/results/text.
- NF4 Portability: all paths resolved via workspace root; no baked user paths.
- NF5 Determinism: resolve v2 deterministic hashes; receipts reproducible.
- NF6 Graceful degradation: missing embed endpoint → BM25 honestly, no silent vector claim.
- NF7 Recoverability: restart persistence (interrupted packets marked failed), checkpoint restore.
- NF8 Security first: threat model + role matrix + audit log before feature code (per §3.7).

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

## 13. Deployment Strategy (PROPOSED)

### 13.1 Phases

- **Phase 0 (now):** local prototype (current Ares app) with measured gaps closed.
- **Phase 1 (sprint D1–3):** freeze design + close internal scaffolds (hooks, skill tree, lazy MCP,
  compaction, checkpoints) on the existing Rust backend; frontend build pipeline documented.
- **Phase 2 (D4–5):** RAG+memory+ingest for all lanes; agent tool loop with approval; streaming;
  smoke gates green.
- **Phase 3 (D6–7):** browser E2E green (parallel agent); docs + claims register; no proposed
  leftovers that are single-file implementable.
- **Phase 4 (D8–9):** model serving: Atlas/Hyperbolic cache + TwisterAttention in rust-server;
  first measured throughput numbers.
- **Phase 5 (D10):** compliance/security audit; right-to-erasure plan; structured audit logs;
  go/no-go review.
- **Phase 6 (post-sprint, enterprise):** packaging (Tauri bundle), multi-endpoint deployment,
  federated Atlas sync (edge/A2A), org policy layer, multi-tenant isolation.

### 13.2 Release discipline

- Desktop app and backend ship together (baked assets); rebuild chain: `npm run build` then
  `cargo build --release`; one script drives both.
- Every release gets a build receipt (like `BUILD-RECEIPT.md`) with measured throughput and
  smoke results, not just "built".
- Secrets by slot; endpoints disabled by default; importer never auto-starts.
- Migration: configurations.json migrations are forward-only with serde defaults (as done for
  `rag_collection`).

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

## 15. Server Inference Engine / Scheduler / Kernel Logic / Throughput / Prefix Caching (PROPOSED)

Scaffolded for the next pass (server-side, in rust-server). Details and measured numbers TBD.

### 15.1 Inference engine

- Replace the single-request forward pass with the ACFT path (§5): task-scoped retrieval via
  TwisterAttention, building only the needed attention slices.
- Keep a standard transformer fallback behind a flag (§14 kill switch).
- Track real TTFT/decode tok/s as first-class (not placeholders).

### 15.2 Scheduler

- Continuous batching with admission control (max in-flight cap already raised to
  `min(16, max_connections)`).
- Prefix-aware batching: coalesce requests sharing a prefix (Radix prefix cache) while honoring
  per-request isolation and memory budgets.
- Task-depth-aware scheduling: shallow recalls served first; deep recalls queued/weighted.

### 15.3 Kernel logic

- Fused attention kernels for the hyperbolic retrieval path; n-gram lookahead trie already ported
  (`ngram_lookahead.rs`) as a draft path.
- Bounded buffers and explicit memory accounting so peak allocation is predictable.

### 15.4 Throughput qualification

- Baselines and targets recorded via `throughput_qwen36.json` / `ttft_qwen36.json` style receipts;
  no number without hardware/workload/comparison.
- Stress tests: concurrent streams, long-context decode, memory pressure, context eviction.

### 15.5 Prefix caching in a nested hyperbolic KV era

- Revisit the Radix prefix cache under hyperbolic memory: prefix keys become coordinates; shared
  prefixes collapse to shared sub-trees in the Poincaré ball.
- Open questions: invalidation cost, geodesic distance thresholds for prefix match, memory
  accounting, interaction with page-entrokv (doc 22).
- Ablation A1 must prove throughput + peak memory improvement vs. the standard path.

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

## 9. Assembly & build improvements (where I would push harder)

These are about _how the app is built/assembled_, not the model science. Each is a candidate
for a design-doc (or refactor) decision before the sprint, so we don't paint ourselves in.

1. **Frontend chunking.** `vite build` emits a single ~1.47 MB JS chunk. Enterprise assembly
   should code-split by route/panel (Settings, Swarm Board, Runs) and lazy-load CodeMirror and
   markdown; the current single chunk is the main LCP risk.
2. **Backend module as services, not one binary.** `main.rs` wires every subsystem by hand
   into `AppState`. Assembly should move to a small dependency-injected composition root (or
   at least a declarative module list) so new silos don't require editing main.rs; also remove
   the unused methods flagged by warnings (`is_enabled`, `workflow_of_packet`, etc.) or wire them.
3. **Tool loop in the lanes.** Today `/agent/run` is one completion per call; the agentic
   contract only exists in workflows. Assembly should make the lane runner a real tool loop
   (plan → tool call → observe → continue) behind an approval gate, instead of one-shot chat.
   Without this, "agent" is a label, not a capability.
4. **Streaming + TTFT.** Provider calls are non-streaming, so TTFT is unmeasured. Assembly
   should make streaming the default transport and propagate TTFT through telemetry receipts;
   blocking parsers are the single biggest latency honesty gap.
5. **Open embedding default.** Dense RAG only works when an embed endpoint exists. Ship a
   small bundled/local text-embedding lane (or a TF-hash baseline that is honestly labeled)
   so retrieval is never silently BM25-only in an "enterprise" build.
6. **Skill tree auto-population.** `/skills/tree` returns 0 nodes until skill-dir plugins are
   registered. Assembly should scan `skills/` and the global skill dirs at boot and register
   them idempotently, so the feature is live on day 1.
7. **N lanes, not 3.** `AgentConfig` hard-codes three slots. Make the roster data-driven (N
   agents with lane assignment) and carry `rag_collection`, tools allow-list, and approval policy
   per agent.
8. **Hooks as a compile-in manifest, not a JSON file.** Today `hooks.json` is hand-edited and
   `fire_event` never auto-runs handlers. Assembly should compile the 23 hook moments into the
   binary manifest (matching the deterministic pre/post gating story) so they can't drift.
9. **Test matrix as a first-class artifact.** 10 unit tests + 1 green Python E2E + 1 flaky
   browser E2E is thin for an enterprise ship. Add property tests on hole/merge/Merkle and a
   deterministic mock-lane agent loop test so reliability is _measured_, not asserted.
10. **Data-root portability.** Paths are absolute and user-specific (`/Users/.../AtomaApp`).
    Assembly should resolve everything relative to a discovered workspace root with a single
    `ATOMA_WORKSPACE_ROOT` override, and never bake a user path into defaults.
11. **Release profile honesty.** The binary bakes the frontend into `include_dir!`; changes
    require a full rebuild. Either document the required build chain (frontend build before
    cargo build --release) in one script, or split asset serving in dev vs. release.
12. **Governance file set per workstream.** Adopt the five-file set (Design, Pre-Registration,
    Claims Register, Successes, Failures) as repo structure, not prose, so claims are auditable.

MULTI-MODEL DISTILLATION - UNIFIED MEMORY MANIFOLD AS A HARNESS PLUG-IN

## 7. Verification gates

- `cargo test --locked` (backend) — 0 failures.
- `scripts/main_ui_e2e.py` — PASS.
- `scripts/compliance_smoke.sh` — 0 FAIL.
- Browser E2E (owned by parallel agent).
- No metric claimed without workload/hardware/baseline.
- Security/compliance checklist signed before Day 1 feature code.

---

## Read next

- `AtomaApp/docs/27-tree-sitter-master-document.md`
- `AtomaApp/AtomaDB/30-rules-and-governance.md`
- `AtomaApp/AtomaDB/31-constraint-grammar-decomposition.md`
- `AtomaApp/docs/tools-design.md`
- `AtomaApp/mcp-servers/design.md`
- `AtomaApp/Harnesses-2026-10-23/audits/THREE-WAY-UNIFICATION.md`
## Coauthor addendum — Muse (2026-10-26)

**Status.** Added by Muse as coauthor, at Stewart's direction. The original text above is
untouched. The only other change in this pass: the trailing paste artifact (`EOF` /
`echo created`) at the end of the file was removed. The filename spelling (`Aers`) is
left as-is so the `Design.md` reference to this file does not break.

**Home base.** This directory
(`MetaSuperIntelligenceLabs-Global-HackAthon-Design/`) is home base for the new build.
AtomaApp is the execution platform, not the sprint project itself. Public name is
**Ares - SpecOps by AtomaLabs**; MadMind is the in-house reference name only.

### Context grounded in the live system (2026-10-26, VERIFIED unless labeled)

- Backend :8787 is live (fresh build, ok:true, RAG library-db openable) and serves the
  current agents config: 3 agents, all with `rag_collection: "papers"`.
- Desktop app was crashing on launch (3 crashes, 14:34 x2, 14:39) because its bundled
  02:18 backend rejected `rag_collection` as an unknown field. Fixed 2026-10-26: strict
  unknown-field rejection removed from agent config, bundle rebuilt, app relaunched and
  verified (sidecar :64605 ok:true, no new crash).
- Settings agents editor was blocked by the same field: its allowlist did not include
  `rag_collection`, so Builder failed first with "unknown agent field" and Save stayed
  disabled. Fixed 2026-10-26: allowlist updated, frontend + backend rebuilt, :8787
  restarted, desktop bundle rebuilt.
- Frontend chunk is 1.47 MB in the current build (dist JS 1,467.28 kB, VERIFIED) — §9.1's
  number is current, and its code-split push stands.
- Settings still hard-codes exactly 3 agents named Builder / Researcher / Verifier
  (VERIFIED in source). §11 F1 (N lanes, data-driven roster) is not yet true in code.
- Lane tool loop (§9.3) and streaming/TTFT (§9.4) are still owed in the next backend
  build: design read, code next. `/agent/run` is still one completion per call.
- Map context, per docs on disc (REPORTED, not re-measured here): Spec 14 nests KV blocks
  on the map's tree with recall by traversal and no forward pass for existing context.
  Application-time map: 74 MB, 201k atoms. Current vault after the Oct 26 Nemo distill:
  603,865,908 bytes, 12/12 basins. Nested-vs-flat mechanism proof: 4.966x speedup,
  same top-1 — mechanism only, not a systems proof.
- Cloud allocation, corrected 2026-10-26 (Stewart's ruling): cloud inference gives
  TOKENS, never weights or hidden states, so it cannot feed weight distillation.
  Distillation runs where the weights are (Mac fleet, Modal GPUs). Baseten = dedicated
  serving of HIS weights only (demo + launch backbone), not teacher inference.

### Where I push the design (enterprise grade, per the 2026-10-26 rebuild standard)

- **P1 — N-lane roster first.** Make the roster data-driven (N agents, per-agent lane,
  tools allow-list, approval policy, `rag_collection`) as the first breaking change,
  with a forward-only configurations migration. §11 F1 must become true before any new
  lane feature lands, or every agents UI fix re-breaks on the 3-slot assumption.
- **P2 — Tool loop + streaming as Day 1 gates, not Day 6.** A lane that cannot loop
  over tools behind approval is a label, not an agent; a transport without measured
  TTFT cannot claim speed. Both gate the sprint, per §9.3–§9.4.
- **P3 — Config forward-compat as a non-functional requirement.** Today's desktop
  crash is the proof: a strict reader + a new field = a bricked app. New rule: readers
  ignore unknown fields, migrations are forward-only with serde defaults, and any
  strictness lives behind a version bump, never a silent rejection. (Applies to agent
  config, endpoint sampling, and future config files alike.)
- **P4 — Threat model + role matrix on Day 1, not Day 10.** §3.7 already says
  security/compliance precede feature code; §13 puts the audit on Day 10. Close that
  gap: threat model, role matrix, audit log, and right-to-erasure plan are Day 1
  deliverables, and the Day 10 pass becomes verification, not discovery.
- **P5 — Evidence labels + necessity gates on every claim.** Every §8 frontier item
  carries status (PROPOSED / VERIFIED / REPORTED), an ablation ID (§12), and a
  necessity ablation (§16.1: does removing it regress a held-out judge?). Add A10
  (prompt-injection: poisoned memory must not change tool calls without a judge flag)
  as a ship gate, and disqualify any ablation run without a pre-filed preregistration
  hash (§14). A failing ablation is a first-class failure entry, never an omission.
- **P6 — One-script release chain, verified end-to-end.** §13.2 requires one script
  driving frontend build + cargo build. The chain exists (`package.json` build script:
  frontend build, prepare.sh, tauri build) but was run in pieces today. Verify it as
  one command, with a build receipt (commit, lock hashes, measured smoke results),
  before it is called the release process.
- **P7 — §5 redo, scoped honestly.** The §5 note orders: get the map and the systems
  proof from the docs on disc and redo. What the docs give today: the nested-KV
  mechanism (Spec 14) and a 4.966x mechanism proof. What is still owed: a full systems
  proof (throughput + peak memory on a fixed corpus, same hardware, per A1, plus
  recall-precision parity) before any Poincaré claim ships in outward copy. The redo
  is a pass in its own right, not a paragraph edit.
- **P8 — Public naming sweep as part of the rebuild.** Outward copy uses Ares -
  SpecOps by AtomaLabs only. The codebase still carries in-house strings in user
  visible paths (e.g. a "MadMind settings" loading string in the built UI). Sweep
  those as part of the enterprise rebuild, not as a cosmetic afterthought.
- **P9 — §2.8 / §2.9 are still stubs.** "Active Memory State Management and Forward
  Iterative Focus" and the empty item 9 are not designs yet. They stay listed as
  open work, not as content, until specified.

### Open work I am not claiming

- §2.8 / §2.9 full specification (stubs, see P9).
- §5 full redo and systems proof (see P7).
- §15 measured numbers (TBD in the doc) and §16.2 toroidal benchmarks (proposed,
  not measured).
- All §8 frontier items remain PROPOSED until their ablation IDs pass.

### Fingerprint

Coauthor: Muse (Meta Muse Spark 1.3), 2026-10-26. Changes in this pass: this addendum
appended; trailing paste artifact removed; no other text touched. Standing rules I bring
to this build: evidence labels on material claims; landed means live in the running
app, not a file on disk; no deletion without explicit per-item confirmation; task lists
execute in a single pass; confirm scope before acting on inference.
