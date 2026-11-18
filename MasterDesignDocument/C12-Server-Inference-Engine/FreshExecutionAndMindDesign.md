---
title: "Fresh GlobalMind Execution and Mind Rebuild"
document_type: "architecture"
version: "1.1.0"
date: "2026-11-01T21:34:29Z"
implementation_date: ""
author: "Codex"
reviewers: []
status: "draft"
related_files:
  - path: "../../hypermind-nested-kv/schema/0007_globalmind_distillation_trajectories.surql"
    lines: "1-229"
    role: "supporting"
related_issues: []
related_documents:
  - "../../hypermind-nested-kv/GLOBALMIND-SURREAL-INTEGRATION.md"
  - "TPPE-RADIUS-RULING.md"
tags: ["globalmind", "active-inference", "tpe", "surrealdb"]
---

# Fresh execution and HyperMind rebuild — design preregistration

Date: 2026-11-01. Status: DRAFT. Owner decision: rebuild the Monster execution
runtime and repeat distillation from live models. Preserve the existing
SurrealDB as a quarantined, read-only evidence source until qualification.

## Boundary and source of truth

- Model files and tokenizer assets are inputs, not an imported implementation.
- A new Rust execution runtime owns weight loading, tokenizer, full forward
  pass, sampling, session isolation, and typed per-layer/per-KV-head tensor I/O.
- One HyperMind graph owns teacher-neutral atoms, typed edges, and receipted
  trajectories in H3 x T2 addresses. Its tensor K/V population is separate
  from semantic recall, even when persisted in the same SurrealDB.
- Toroidal Phase Positional Encoder is the positional contract for the new
  attention path. Its recurrence, version, dimensions, and output parity must
  be specified and tested before use. No RoPE compatibility is presumed.
- TPPE/TPPE is a dynamic spiral-out, not a constant-radius corkscrew. Radius
  expands per revolution while the trajectory approaches the hyperbolic ideal
  boundary asymptotically without crossing it. In the hyperboloid chart this
  means preserving the admitted Lorentz-sheet invariant; in a Poincare-ball
  chart it means remaining strictly inside the ball. A trajectory receipt must bind the radius-law
  version, revolution count/state, boundary margin, and numerical-stability
  evidence. Constant radius, radius regression, or boundary crossing rejects
  replay and admission.
- The checked-in linear-plus-log radius law is a candidate, not a frozen
  optimum. Its prior “sublinear-asymptotic” label conflicts with the same
  specification’s stated nonzero `r(w)/w` limit. `TPPE-RADIUS-RULING.md`
  preregisters a matched linear-log versus log-only bakeoff; losing laws sit on
  the bench rather than being retained for architectural preference.
- Qdrant remains the research/fact library; telemetry and decision logs are
  independent observability stores. Neither becomes a mind-map authority.
- The r12 quotient is a named open mathematical contract until its generators,
  canonicalization, and invariants are executable and tested.

## Quarantine rule

The current `hypermind/core` SurrealDB is not a qualified input to new model
recall. Its 2,079 atoms and schema are retained without deletion or reingest.
No existing atom, edge, or receipt is grandfathered into the new graph.
The new runtime must use an isolated development database/namespace until the
owner approves promotion after the gates below. The old N3/CoreLM and archived
Monster sources may be inspected as failure evidence only; no code is copied.

## Interfaces to define before code

1. `ModelIdentity`: cryptographic content fingerprint, tokenizer/config hash,
   quantization, layer/head layout, and signed runtime version.
2. `ForwardStep`: typed token IDs and tensor buffers; explicit layer/head,
   causal interval, shape, dtype, layout, TPPE state, and numerical error budget.
3. `TensorPage`: immutable K and V bytes plus exact model/session/prefix/TPPE/
   address fingerprints. A mismatch is a miss; no cross-model tensor sharing.
4. `DistilledAtom`: teacher-independent address and content digest, judge
   receipt, and append-only audit lineage. Teacher identity cannot enter
   coordinate projection, novelty, retrieval score, or execution gate.
5. `Trajectory`: ordered atom IDs and typed transitions with source artifact,
   model receipt, judge verdict, and replay proof. Raw K/V bytes are excluded.
6. Native binary runtime transport for tensors; MCP may expose control and
   receipts but cannot carry the model's K/V payload through JSON.

## Preregistered acceptance gates

| Gate | Required evidence | Failure result |
| --- | --- | --- |
| G0 identity | Rehash live model bytes and reconcile every roster receipt | No distillation for that model |
| G1 forward | Real-weight reference parity on fixed token tapes, full logits and generation, per supported model | No serving promotion |
| G2 TPPE | Frozen recurrence oracle, phase/radius bounds, causal position tests, real-weight RoPE baseline ablation | No TPPE claim |
| G3 tensor KV | Layer/head isolation, prefix/session separation, restart recovery, K/V digest equality, decode parity | Cache disabled |
| G4 universal atoms | Same trace under two teacher fingerprints yields identical address and one semantic atom; teacher swap leaves recall/rank unchanged | No shared mind admission |
| G5 trajectories | Source-to-atom mapping, typed edge replay, judge receipts, no model-specific recall dependency | Reject trajectory |
| G6 persistence | Append-only migrations, drift rejection, backup/restore, corruption and concurrent-writer tests | No Surreal promotion |
| G7 end-to-end | Fresh multi-model distillation into isolated DB, restart, harness recall, output-quality and latency measurements against preregistered baseline | No certification |

Every gate needs a command receipt, exact artifact hashes, and a stated
hardware/model configuration. Passing unit tests or schema creation alone
cannot promote a gate. Any change to the gates requires a dated amendment
before running the affected experiment.

## Evidence-over-architecture rule

Every proposed mechanism is provisional, including TPPE, hyperbolic charts,
rank-8 Jacobians, isotropic attention, adaptive forward depth, and each
distillation projector. Push the candidate to its preregistered numerical and
empirical limits against the strongest matched reference implementation. If a
proof falsifies its required invariant, or matched real-hardware evidence shows
an inferior quality/resource frontier after fair tuning, bench it. Preserve the
reference route and the falsification receipt. Do not weaken a gate, change the
workload after results, or retain a component because it is architecturally
preferred.

## Build order

Freeze model/attention contracts; implement a small real-weight forward
reference; prove TPPE and tensor K/V parity; implement teacher-neutral
distillation and trajectory replay; ingest into the isolated DB; run G0–G7;
then decide whether to replace the quarantined SurrealDB records or retain
them as historical evidence. No deletion is implied by this draft.

## Amendment C — native distillation ingress (2026-11-01)

The first writable boundary is a two-phase, fail-closed ingress. Phase one
parses a native typed roster, requires absolute artifact paths, rehashes every
artifact from disk, and derives one aggregate roster digest per frozen model.
It performs no database write on any mismatch. Phase two writes only immutable
model, artifact, admission, run-envelope, and run-event receipts into an
isolated database. Candidate extraction is a later phase and may begin only
after the opened receipt reconciles all roster digests.

The manifest is operator-authored evidence, not a model payload. JSON and
Python remain forbidden at the model runtime boundary. Teacher provider,
model, revision, paths, and artifact digests are audit lineage. Candidate
identity must be derived from a canonical teacher-neutral preimage that
excludes all of those fields. A runner that merely hashes weights is not
distillation and must not emit atoms, edges, loops, charts, trajectories, or a
completed receipt.

Run state is append-only: an immutable run envelope is followed by ordered
`opened`, `extracting`, `judging`, `replaying`, `completed`, `failed`, or
`rejected` events. Completion requires an output digest and receipt digest;
failure cannot rewrite or delete prior events. Retry creates a new run ID.

Ingress qualification gates are: real artifact rehash; zero writes on roster
mismatch; isolated-database enforcement; idempotent collision rejection;
source/admission/run/event readback; restart persistence; and receipt digest
reconciliation. This proves ingress only. It does not prove candidate
extraction, teacher invariance, trajectory replay, or useful recall.

## Amendment D — atomic cell and trajectory crystallization (2026-11-01)

A recallable trajectory is one atomic receipt bundle, not a header pointing at
unproven IDs. Before the transaction, each step must reference an immutable,
admitted rank-1 packet; every declared loop must reference an admitted rank-2
packet; and every local chart must reference an admitted rank-8 packet. The
transaction creates typed atom edges, ordered steps, trajectory origins,
closed-loop receipts, and the final crystallized trajectory together. A
missing atom, endpoint discontinuity, duplicate ordinal, non-closed loop,
packet-kind mismatch, replay failure, or TPPE receipt mismatch yields no
partial graph write.

Rank-8 means a local chart with admitted rank no greater than eight, not an
automatic eight-dimensional claim. Its tensor shape is checked with bounded,
overflow-safe integer arithmetic; its byte digest must match the actual native
buffer. No floating-point Jacobian property is inferred by storage. Marginal
gain, numerical stability, and held-out benefit remain independent receipts.

Run lifecycle is also append-only. Each event must extend the prior ordinal and
legal phase transition. `completed` requires reconciled output and receipt
digests. Competing writers collide on the unique ordinal and fail closed.

## Amendment A — GlobalMind target architecture (2026-11-01)

**One mind.** `GlobalMind` is the sole multimodel knowledge map. Distillation
admits model-agnostic atoms, typed 1-cell transitions, 2-cell trajectory loops,
and local rank-8 Jacobian charts. Rank 0 is an atom with address, evidence and
usable payload; rank 1 is a directed, typed transition; rank 2 is a verified
closed path with its loop invariant. Each atom must carry sufficient evidence,
payload and reconstruction contract to answer its admitted task class; a bare
label, embedding or coordinate is insufficient. Teacher identity remains in
audit provenance, never in the shared retrieval key.

**Nested tensor cache.** Model K/V pages live under GlobalMind addresses and
use the same hyperbolic metric and T2 phase indexing as atoms. They remain
exact binary tensors indexed by model digest, layer, head, token interval,
shape, dtype, TPPE version, address contract and content digest. A page mismatch
is a cache miss. Shared geometric placement does not imply cross-model tensor
reuse. Poincare-ball coordinates are a candidate chart for H3, subject to
distance equivalence with the chosen reference metric and boundary-stability
tests. TPPE supplies toroidal phase positions; it is not RoPE.

**Distillation order.** Rehash frozen model artifacts; extract rank-0 atoms,
rank-1 transitions, rank-2 loops and rank-8 local Jacobian candidates with
source/teacher receipts; independently judge semantic invariance; walk new
model trajectories; crystallize only replayable, useful outcomes. A weight
projection alone is not proof of model-agnostic capability. The quarantined
SurrealDB contributes no recalled content until it passes the same admission.

**Active inference and computation.** A controller selects retrieval depth,
attention span, chart use, and whether to invoke a partial or full forward
pass using a preregistered quality, latency, memory and measured-energy budget.
Toroidal isotropic attention is generated at runtime to retrieve atoms and
execute the chosen computation. Replacing every non-attention layer and
skipping arbitrary layers changes a frozen transformer's function; this is a
new architecture hypothesis, not certified equivalence. Keep full real-weight
forward as the reference and fallback until held-out task quality, logits,
failure cases, heat/energy and latency qualify each adaptive route. Landauer's
bound is a physical floor for irreversible erasure, not a measurement of
system heat or a guarantee that variational free-energy minimization reduces
electrical consumption. Record both the inference objective and wall-power/
temperature measurements rather than conflating them.

**Twelve around one.** The product-space design assigns one central cognitive
hub as the active GlobalMind reference state and twelve peer surrounding wells
as address/routing regions. No surrounding well is reserved for K/V. The nested
tensor cache attaches to the address, well, and trajectory actually executed.
Surprise is the precision-weighted prediction-error signal on the hub-to-well
routing decision. Its record keeps raw error, precision, weighted error, and
surprisal distinct; optional Bayesian surprise remains a separate belief-update
measure. It may trigger deeper retrieval, trajectory extension, full forward,
or quarantine and may crystallize only after judgment and replay. The existing C06
formalization supports a 12-sector phase tiling but says the unified H3 x T2
quotient action and its generators are unproved. Therefore hub/well boundaries,
canonicalization, overlap, and transition rules need an executable contract
before `r12` is used as an address invariant. The inspected AtomaApp files do
not supply that missing proof.

**Additional gates.** Preregister (a) cross-teacher atom identity and held-out
transfer, (b) K/V model isolation and same-metric address parity, (c) rank-8
chart marginal gain, (d) full-versus-adaptive forward quality and measured
energy at equal task sets, and (e) well-boundary and quotient invariants.
Failing a gate disables the corresponding optimization, not the reference
forward path.

## Amendment B — superposition holding (2026-11-01)

**Controller placement.** Superposition holding runs after recall/carving and
before external emission. A recall miss, cycle, branch ambiguity, high
precision-weighted prediction error, low readiness, or compliance boundary
opens a hold. Candidate atom/trajectory branches are retained concurrently,
scored homogeneously, counterfactually rolled forward, checked against rank-2
loops, and suppressed only by the shared scalar decision contract.

**Collapse rule.** The controller collapses only onto a compliant,
prediction-ready branch with a replay receipt. If no branch qualifies, it
mandates further iteration or fails closed; it cannot emit a degraded token to
escape the hold. No external emission occurs during holding iterations.

**Memory rule.** Collapse does not directly mutate GlobalMind. A synthesized
bridge or trajectory re-enters the same teacher-neutral distillation, judge,
replay, and crystallization pipeline as every other candidate. SurrealDB stores
immutable hold, branch, iteration, and terminal-resolution receipts. The fresh
implementation uses the user-directed behavior from N3
`design/superpositionholding.md`; it imports no N3/CoreLM implementation code.
