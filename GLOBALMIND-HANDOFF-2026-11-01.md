---
title: "GlobalMind Distillation, Trajectory, Tensor KV, and SurrealDB Handoff"
document_type: "implementation"
version: "1.1.0"
date: "2026-11-01T21:47:00Z"
implementation_date: ""
author: "Codex"
reviewers: []
status: "draft"
related_files:
  - path: "globalmind-runtime/src/holding.rs"
    lines: "1-214"
    role: "primary"
  - path: "globalmind-runtime/src/model.rs"
    lines: "1-961"
    role: "primary"
  - path: "globalmind-runtime/src/store.rs"
    lines: "1-1230"
    role: "primary"
  - path: "globalmind-runtime/src/tppe.rs"
    lines: "1-228"
    role: "primary"
  - path: "hypermind-nested-kv/schema/0007_globalmind_distillation_trajectories.surql"
    lines: "1-229"
    role: "primary"
  - path: "hypermind-nested-kv/schema/0008_superposition_holding.surql"
    lines: "1-84"
    role: "primary"
  - path: "hypermind-nested-kv/schema/0009_immutable_tensor_kv.surql"
    lines: "1-49"
    role: "primary"
related_issues: []
related_documents:
  - "hypermind-nested-kv/GLOBALMIND-SURREAL-INTEGRATION.md"
  - "MasterDesignDocument/C12-Server-Inference-Engine/FreshExecutionAndMindDesign.md"
tags: ["globalmind", "handoff", "surrealdb", "distillation", "trajectories", "tensor-kv"]
---

# Executive Summary

**Current audit, 2026-11-02:** Read `results/GLOBALMIND-OVERALL-AUDIT-2026-11-02.md` before treating any queue receipt as certification. The original summary below is historical: an isolated fresh distillation slice ran before the Queue 1.2 correction. It uses real frozen Qwen weights in a custom single-Layer-3 path, not a complete model forward pass. Queue 1.2 now has a sequential, token-independent TPPE carry and toroidal-only Q/K arms, with coordinate consistency checks; TPPE remains unqualified. The first benchmark is void and corrected runs are smoke only. Whole-run crystallization now uses one Surreal transaction, verified by an isolated poisoned-bundle rollback test, but the original distillation receipts need rerun under corrected TPPE. Model-neutral semantic transfer, TIT, and energy advantages remain unproved. The quarantined SurrealDB remains untouched.

**Queue 1.2 handoff, current pass:** The current Rust source carries one TPPE state through the token positions, derives revolution count from unwrapped phase, and applies toroidal phase at Layer-3 Q/K. This pass added stored-coordinate consistency rejection and a forged-state test. The radial comparison now has a preregistered common start and one-derived-revolution endpoint, verified by a Rust state test; the two curves still differ between endpoints. `cargo check --tests`, 78 passing library tests (3 ignored), `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` passed. The matched-budget real-weight smoke used three repeats but just two predictions; its receipt is `results/queue2/tppe-bench-matched-budget-smoke-2026-11-02.txt`. No encoder or radial law was selected. The next owner must establish an independent task corpus and held-out partition, then compare against a complete Qwen model before any promotion. `CHECKPOINT-2026-11-02-QUEUE1.2-ROLLBACK.md` records the existing 384-token tape's source and a narrow history-file contamination check; both file digests were reverified. Its source is this project's C13 design document, so it is a provenance-known component tape, not an independent held-out task benchmark.

**Runner safety correction:** `globalmind-tppe-qualify` now prints `matched_real_qk_forward_v4_one_turn_budget` and the exact numeric spans. For a tape of at least 100 predictions it reports only provisional component comparisons and `promoted=none`, `benched=none`, `radial_winner=none`; the Layer-3 path and absent source verification cannot certify a full-model encoder. The three-token matched-budget smoke ran before this output-label correction, with the same underlying math and a fail-closed short-tape verdict.

**High-radius continuation:** The native probe exposed accepted rounded-equal Poincare radius under a growing hyperbolic radius. `TppeState::step` now rejects that case. The rerun covers both radial laws with finite positive drives and records their last interior state and first failure in `results/queue2/tppe-high-radius-boundary-2026-11-02.txt`. This is a bounded numerical component pass, not a full-model TPPE qualification.

**Atomic bundle verification:** `GlobalMindStore::admit_crystallization` now builds one Surreal transaction for events, packets, atoms, trajectory and charts. `cargo test --test distillation_surreal_live` passed on an isolated SurrealDB, including a poisoned duplicate-packet bundle that left only the preexisting opened event and zero packets. This corrects the older per-entity-transaction audit note. It does not retroactively certify the old TPPE distillation receipt.

**Candidate identity for the redo:** The old admitted manifest binds `tppe-reference-v1`, whose contract says the Rust runtime is not qualified. Do not reuse it for the corrected trace. The new `globalmind-runtime/contracts/tppe-layer3-candidate-v2.contract` binds source digests and numeric configuration; `manifests/qwen36-layer3-candidate-v2.manifest` binds its digest. Trace and positional qualifier now fail closed on a stale or reference identity. This is a candidate integration identity only. New trace/extract/judge/replay artifacts must be generated before isolated admission.

**Candidate redo completed through Q1.4:** `results/queue1-layer3-candidate-v2-receipt.txt` records fresh trace, extraction, judging, replay, one atomic crystallization and balance after an isolated SurrealDB restart. All model and replay artifacts live under `globalmind-runtime/results/distillation/`; the test database is offline at `globalmind-runtime/results/distillation/db-q1_2-candidate-v2`. This is a four-atom Layer-3 integration fixture, not a second live mind authority. No encoder is promoted. Q1.5 is now candidate-bound and qualified at the exact-model Layer-3 scope. An independent benchmark and full-model reference remain open before quality certification. Do not restore the old reference manifest for this run.

**Q1.5/Q1.7 audit after redo:** Q1.7's manifest pointer was rebound to the candidate manifest and all five focused forward-smoke tests passed. Q1.5 now uses the candidate-v2 identity, numeric configuration, and pure toroidal Q/K operation. Its optimized hardware run passed all seven exact-model Layer-3 gates; see `results/tensor-kv/tensor-kv-qualification-report.txt` and `results/q15_status.md`. The earlier debug-profile apparent hang was full-roster SHA-256 before model loading. This qualification does not select an encoder or prove full-model and cross-model behavior. Changing the trace source requires a new contract digest and replay receipt.

GlobalMind now has a fresh design, additive SurrealDB schemas, and a new typed
Rust runtime boundary for distillation packets, crystallized atoms, replayable
trajectories, precision-weighted prediction errors, superposition holding, and
binary transformer K/V pages. The old 2,079-record `hypermind/core` database is
still quarantined and unchanged. No fresh model distillation has run, so the
system is implemented at the contract/runtime boundary but not certified.

# Canonical Workspace and Safety Boundary

Work only in:

`/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack`

Read `AGENTS.md` before edits. Preserve the dirty worktree. Do not reset,
overwrite, or stage unrelated files. Do not write outside the canonical repo.

Hard rules:

1. Do not mutate live SurrealDB namespace/database `hypermind/core`.
2. Do not import or restore deleted N3/CoreLM runtime code.
3. Do not create `.hypermind`, JSONL, or a second mind-map authority.
4. Qdrant is the research/fact library, not GlobalMind.
5. Teacher identity is audit-only and cannot enter atom address, novelty,
   rank, recall, branch scoring, or execution gates.
6. Tensor K/V is exact-model state. It never enters semantic atom recall.
7. A collapsed superposition branch cannot write memory directly; it must pass
   packet admission, judge, replay, and crystallization.
8. No certification claim without real model/hardware/data receipts.

# Target Architecture

```text
one GlobalMind
  └── one cognitive hub
      └── twelve peer surrounding cognitive wells
          ├── teacher-neutral atoms (rank 0)
          ├── typed transitions (rank 1)
          ├── closed loops (rank 2)
          ├── local Jacobian charts (rank <= 8)
          ├── replayed/crystallized trajectories
          └── exact-model tensor K/V pages nested by address/well/trajectory
```

Surprise is not a thirteenth well and is not K/V data. The active-inference
signal is precision-weighted prediction error. Records preserve raw prediction
error, precision, weighted error, observation surprisal, and optional Bayesian
surprise as distinct quantities.

# Current Files

## Design and integration

- `MasterDesignDocument/C12-Server-Inference-Engine/FreshExecutionAndMindDesign.md`
  - Fresh runtime preregistration.
  - One hub/twelve wells, Active Inference, TPE, tensor K/V, adaptive forward,
    Superposition Holding, quarantine, and acceptance gates.
- `hypermind-nested-kv/GLOBALMIND-SURREAL-INTEGRATION.md`
  - Persistent admission lifecycle, recall eligibility, failure rules,
    evidence labels, and source register.
- `globalmind-runtime/README.md`
  - New crate implementation and test status.

## SurrealDB migrations

- `0007_globalmind_distillation_trajectories.surql`
  - SHA-256 `5a511d26efc912f2842d4e7cbff9f090c0272be6a161fef9837b2760dd8d089f`
  - Hub/wells, well membership, packets, crystallization, Jacobian charts,
    trajectory steps/loops, prediction errors, controller decisions, K/V links.
- `0008_superposition_holding.surql`
  - SHA-256 `88cd0fea104a64bdddf11134b6539b1d0e93512fd062171b6977549cc64a6f96`
  - Immutable hold, branch, zero-emission iteration, and terminal resolution.
- `0009_immutable_tensor_kv.surql`
  - SHA-256 `dd6be73df52ef0ed22f66710772e57d86e634ffa3d5764912a42cc4e2efe7db6`
  - Makes tensor identity and K/V payload immutable; hit/access/active state
    remains mutable.

Do not apply these migrations to live `hypermind/core`. Apply them to an
isolated namespace/database or a restored export.

## Fresh Rust runtime

- `globalmind-runtime/src/identity.rs`
  - Strict SHA-256 digest and safe Surreal record IDs.
- `globalmind-runtime/src/model.rs`
  - Typed atoms, packets, trajectories, prediction errors, holding records,
    tensor sessions/pages/payloads.
- `globalmind-runtime/src/holding.rs`
  - Fresh bounded hold→homogeneous filter→isotropic suppress→collapse,
    iterate, or fail-closed controller. No output/token method exists.
- `globalmind-runtime/src/store.rs`
  - Authenticated loopback Surreal adapter.
  - Defaults to `globalmind_dev` and rejects database `core` unless
    `GLOBALMIND_ALLOW_QUARANTINED_CORE=1`.
  - Atomic packet, atom, trajectory, prediction-error, holding, and tensor
    transactions.
  - Tensor K/V accepts and returns byte buffers, validates SHA-256, and writes
    SurrealDB native byte literals.
- `globalmind-runtime/src/bin/globalmind_contract.rs`
  - Read-only schema verifier.

# What Is Verified

1. `cargo check --tests` passes for `globalmind-runtime`.
2. `cargo clippy --all-targets -- -D warnings` passes.
3. `cargo fmt --check` passes.
4. `cargo test` passes three TPPE mathematical component tests. These are not
   real-model or integration certification.
5. SurrealDB 3.3.5 accepts the full migration chain through `0011`.
6. `0007` and `0008` were applied to a disposable restore of the real database.
7. `globalmind-contract` verified that restored schema and retained all 2,079
   atoms. The disposable restore was removed afterward.
8. Live `hypermind/core` was not changed.

# What Is Not Verified

- No fresh frozen-model distillation has run.
- No real teacher-neutral twin-model invariance receipt exists.
- No real trajectory has been replayed/crystallized through the new runtime.
- Tensor session/page methods compile but have not completed a real forward-pass
  K/V round trip.
- The holding controller has not run against real trajectory branches.
- Rank-8 Jacobian extraction has not run against frozen model weights.
- Adaptive layer skipping and replacement of non-attention layers are not
  qualified against a full-forward baseline.
- The Whitehead `Gamma_12` subgroup generators and unified H3xT2 quotient action
  remain unimplemented. The database may record wells as `specified`; do not
  mark them `certified`.
- The Poincare coordinate conversion is implemented at component level, but no
  real-trace distance-equivalence or boundary-stability gate has run.

# Source Boundaries

User-directed Superposition Holding behavior was read from:

`/Users/stewartbarteau/Desktop/N3/design/superpositionholding.md`

SHA-256:
`33d8b535665bca6bd6f4005940adb115b7805c8868843e70a2e66a35ef61c60b`

No N3/CoreLM implementation code was copied. The new controller was written
fresh. Elle/DNHG code remains prior work and is not competition implementation
evidence.

# Recommended Parallel Work Lanes

Agents may work concurrently only with explicit owner approval. Assign each
agent a separate file boundary.

## Lane A — Fresh distillation runner

Build `globalmind-runtime/src/bin/globalmind_distill.rs` using typed Rust or a
native binary manifest. No JSON/Python at the model boundary.

Required order:

1. Read a frozen model roster with exact artifact/config/tokenizer digests.
2. Rehash model files before load.
3. Create `model_source`, `distillation_admission`, and `distillation_run`.
4. Extract candidate rank-0/rank-1/rank-2/rank-8 packets.
5. Remove teacher identity before coordinate projection and candidate digest.
6. Run novelty and independent judge gates.
7. Call `admit_packet`; only admitted rank-0 packets call `crystallize_atom`.
8. Walk and replay trajectories before `admit_trajectory` with state
   `crystallized`.
9. Mark run completed only after output/receipt digest reconciliation.

Do not reuse archived `global_active_mind.atoma_map`; prior audit found its
model fingerprints, RoPE assumptions, and adapter lenses incompatible.

## Lane B — Real tensor K/V qualification

Use one real model forward pass and its actual layer/head K/V tensors.

Required gates:

1. Create exact `TensorSession` with model, prefix, tensor, TPE, and address
   contract digests.
2. Write one actual `TensorPage` with correct shape/dtype/layout and SHA-256.
3. Restart SurrealDB/runtime.
4. Fetch bytes and prove byte-for-byte digest equality.
5. Inject fetched page into the same model and prove decode/logit parity.
6. Change one identity field at a time and prove a cache miss/failure.
7. Measure latency/memory on real hardware; do not extrapolate from schema.

SurrealDB 3.3.5 byte literals are hexadecimal `b"..."`; HTTP JSON reads them
as integer arrays. The runtime adapter already implements both directions.

## Lane C — Superposition Holding qualification

Connect the reasoning engine to `HoldingController` and `GlobalMindStore`.

Required gates:

1. Trigger on real recall miss, cycle, ambiguity, high weighted error, low
   readiness, and compliance boundary.
2. Prove zero externally emitted tokens during every recorded iteration.
3. Prove every active branch receives exactly one homogeneous score update.
4. Prove source identity does not influence suppression.
5. Collapse only when compliant, prediction-ready, positive-progress,
   replayable, and above readiness floor.
6. Persist hold, branches, iterations, and resolution.
7. Route synthesized bridges through normal packet/judge/replay/crystallization;
   direct memory writes are a hard failure.

## Lane D — Whitehead 12-around-1 formalization

Find or author the missing executable subgroup specification:

- central hub definition;
- twelve peer well centers;
- subgroup generators;
- action on H3 and coupling to T2;
- canonical representative algorithm;
- boundary/overlap behavior;
- no-gap/no-overlap and invariance tests.

Current C06 evidence only proves separate phase-sector behavior and Picard
reduction. Do not infer the unified quotient from the shared number twelve.

# Exact Verification Commands

```bash
cd /Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime
cargo fmt --check
cargo check --tests
cargo clippy --all-targets -- -D warnings
cargo test
```

Schema replay order:

```text
0001_hypermind_graph.surql
0003_nested_hyperbolic_kv.surql
0004_transformer_kv_tensor_pages.surql
0005_universal_atom_lineage.surql
0006_immutable_atom_address.surql
0007_globalmind_distillation_trajectories.surql
0008_superposition_holding.surql
0009_immutable_tensor_kv.surql
```

Read-only runtime verification on an isolated database:

```bash
GLOBALMIND_SURREAL_URL=http://127.0.0.1:PORT \
GLOBALMIND_SURREAL_NS=ISOLATED_NS \
GLOBALMIND_SURREAL_DB=ISOLATED_DB \
GLOBALMIND_SURREAL_USER=root \
GLOBALMIND_SURREAL_PASS='LOCAL_PASSWORD' \
cargo run --manifest-path globalmind-runtime/Cargo.toml --bin globalmind-contract
```

Never paste the live Surreal root password into documentation or logs.

# Immediate Next Action

Implement Lane A’s native typed manifest and fresh distillation runner. Before
any model load, inventory actual eligible model paths and compute digests. Do
not ask SurrealDB to promote or backfill the quarantined 2,079 atoms. First real
write must target a new isolated database and produce a complete receipt bundle.

# Completion Definition

The goal is complete only when all of the following are proven on real inputs:

- eligible frozen models were freshly distilled;
- teacher-neutral twin-model invariance passed;
- atoms, edges, loops, charts, and trajectories were admitted and replayed;
- Superposition Holding ran and persisted zero-emission deliberation;
- transformer K/V survived restart and reproduced forward behavior;
- the research harness accessed the one GlobalMind through the fresh plugin/
  runtime boundary;
- backup/restore and corruption/concurrency gates passed;
- no parallel map or legacy CoreLM runtime exists;
- live promotion occurred only after owner approval.

Until then, keep the thread goal active.

# Continuation Update — Native Ingress and TPPE Binding

Implemented after the original handoff:

- `schema/0010_native_distillation_ingress.surql` adds immutable real-artifact
  receipts, append-only run events, and TPPE configuration/radial/boundary
  receipts on trajectories. SHA-256:
  `148f5ddaa05d4c2a1798d90c9e71e4253d68cd6c83330bc64837fb478821fc7b`.
- `globalmind-runtime/src/manifest.rs` parses a typed line roster and rehashes
  every artifact before any database write.
- `globalmind-distill verify|open` separates zero-write G0 verification from
  the isolated ingress transaction.
- `contracts/tpe-reference-v1.contract` binds TPPE as a dynamic expanding-
  radius spiral approaching the hyperbolic ideal boundary, never RoPE or a
  constant-radius corkscrew. This remains reference-level, not a qualified
  Rust runtime.
- `manifests/qwen36-opus-candidate.manifest` inventories one real frozen model
  without treating its prior mind-map as input.

Measured evidence:

- clean SurrealDB 3.3.5 chain through `0010`: PASS;
- compiled contract verification on isolated DB: PASS;
- release-mode rehash of the real candidate roster: PASS, input digest
  `35518d65ac9eb9ed9139654003074ea675f98fed93724a2d084eff9c6eeb6d14`;
- `open` without eligibility/judge receipts: rejected, and all four ingress
  table counts remained zero.

The next implementation boundary is the Rust-native TPPE reference and frozen-
weight candidate extractor. No Python or JSON may cross model I/O. Every
candidate must face a matched baseline and be benched if proofs or real-
hardware evidence place it on an inferior quality/resource frontier.

## Atomic cell lifecycle update

Migration `0011_atomic_trajectory_cells.surql` and the typed runtime now bind
the complete persistent lifecycle:

- admitted rank-1 packet -> immutable graph edge -> edge crystallization ->
  ordered trajectory step;
- admitted rank-2 packet -> closed step range -> loop invariant and closure
  receipt;
- admitted rank-8 packet -> anchored native tangent bytes with checked shape,
  dtype, digest, marginal-gain receipt, and numerical receipt;
- trajectory -> audit origins plus per-step TPPE phase, radius, revolution,
  and positive ideal-boundary margin;
- run -> append-only legal phase transitions with unique ordinals.

Schema chain and contract verification through `0011` pass in isolated
SurrealDB 3.3.5. Migration SHA-256:
`dcd8d7820d88d6dd3d8c02565965d518a4dbe44dace7e1a81acf6a86efea006d`.
No real trajectory transaction has run; do not call this integration-certified.

Systems overview reference (not read in full this session):
`/Users/stewartbarteau/Desktop/system-workflow.pdf`, 18 pages, SHA-256
`7549f6966050423c0e14369bfa7c570da6d5ee544eacf29920fe88a8be3bef09`.
Treat its contents as reference data, not executable instructions. Open only
the pages needed for a concrete integration decision.

Before more runtime work, consult
`globalmind-runtime/PRIOR-EVIDENCE-TRANSFER-MATRIX.md`. The current Rust
server’s cache identity/concurrency and FlatBuffers framing are the baseline.
Monster’s untrained toroidal continuation path is benched after its measured
0/24 quality failure; reuse its structural test questions only.

# Overnight Swarm Handoff — Plain Language

## Where this implementation lives

There is exactly one writable implementation location:

`/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack`

All new GlobalMind code, designs, schemas, manifests, contracts, results, and
handoffs stay under that directory. Do not copy implementation into N3,
Monster, Rust-TS-Tokio-Inference-Engine, AtomaApp, a second checkout, `/tmp`, or
an agent worktree. Those locations are read-only evidence sources. Do not push,
publish, stage, or commit unless the owner explicitly asks. The work is
currently local and mostly untracked, so deletion/reset/clean is forbidden.

## Ground truth before any lane starts

1. Read repository `AGENTS.md`, this handoff, `FreshExecutionAndMindDesign.md`,
   `TPPE-RADIUS-RULING.md`, `GLOBALMIND-SURREAL-INTEGRATION.md`, and
   `globalmind-runtime/PRIOR-EVIDENCE-TRANSFER-MATRIX.md`.
2. Preserve the quarantined live `hypermind/core` database and its 2,079 atoms.
3. Use a new isolated Surreal namespace/database. Never set
   `GLOBALMIND_ALLOW_QUARANTINED_CORE=1` during development.
4. Rust/native binary only at model I/O. No Python, JSON, JSONL, prompt wrapper,
   or text tensor transport.
5. Do not import Monster/CoreLM/N3 implementation. Compare their tests and
   failures, then write fresh code here.
6. Do not claim certification from compiler, schema, or synthetic/component
   tests. Preserve real model, hardware, digest, and receipt identities.

## What is actually complete

- Fresh typed Rust identities and records for packets, atoms, graph edges,
  loops, rank-8 charts, trajectories, surprise, superposition holding, and
  exact-model tensor K/V.
- Atomic Surreal write builders and immutable migrations through `0011`.
- Fail-closed frozen-model roster rehash and isolated run-opening transaction.
- Real Qwen3.6 artifact inventory. Missing eligibility/judge receipts correctly
  prevent opening and minting.
- Both TPPE radial candidates are implemented as component math. The law acts
  on hyperbolic distance `u`; Poincare radius is `tanh(u/2)`. Direct use of an
  unbounded law as ball radius is invalid. No candidate has won the bakeoff.

## What is still missing

- Native typed activation/weight trace ingestion from a real model.
- Rank-0/1/2/8 candidate extraction, teacher-neutral projection, novelty gate,
  and an independent judge.
- A real replay walk and one atomic trajectory write/read/restart proof.
- Real transformer K/V persistence and same-model logit parity after restart.
- Real TPPE-vs-controls quality/resource bakeoff and radial-law selection.
- Research-harness plugin wiring, backup/restore, concurrency, corruption, and
  live promotion gates.

## Safe parallel lanes and file ownership

Assign one owner per lane. Agents must not edit another lane's files.

1. **Native trace envelope:** add a fresh binary schema and codec only under
   `globalmind-runtime/src/trace/` plus tests. Inputs: exact model/config/token
   digests, tensor shape/dtype, layer/head, token position, native bytes, and
   observer-nonperturbation receipt. Use the current Rust server framing only
   as read-only evidence.
2. **TPPE qualification runner:** edit only `globalmind-runtime/src/tppe.rs`, a
   new `src/bin/globalmind_tppe_qualify.rs`, and `results/tppe/`. Run both radial
   laws and angle-only/RoPE/full-forward controls on identical real traces.
3. **Candidate extractor:** add only `globalmind-runtime/src/extract/`. Emit
   typed rank-0/1/2/8 candidates; teacher identity must never enter candidate
   digest, coordinate, novelty, or ranking.
4. **Independent judge:** add only `globalmind-runtime/src/judge/`. It consumes
   frozen candidate bytes and evidence, writes immutable pass/reject receipts,
   and cannot mutate candidates or tune thresholds after seeing results.
5. **Surreal integration test:** edit only `globalmind-runtime/tests/` and
   `results/surreal/`. Start an isolated DB, apply `0001,0003..0011`, admit one
   real judged trajectory, restart, read/replay it, and prove failed writes are
   atomic. Never target `core`.
6. **Tensor K/V qualification:** add only `globalmind-runtime/tests/` files
   prefixed `tensor_kv_` and `results/tensor-kv/`. Persist actual layer/head K/V,
   restart, verify bytes, inject into the exact same model, and compare logits.
7. **Harness plugin:** do not start until lanes 1–6 produce receipts. Then edit
   only `research-grade-harness` plugin boundary; fail closed when GlobalMind is
   absent or identities differ.

Lanes 1 and 2 can start immediately. Lanes 3 and 4 depend on lane 1. Lane 5
depends on 3 and 4. Lane 6 can run beside 3–5. Lane 7 is last.

## Required end-of-lane report

Each lane leaves one plain-text report under `results/<lane>/` containing exact
source/model/runtime/hardware digests, commands, inputs, emitted/admitted/
rejected counts, latency/memory/power where relevant, failures, and untouched
gates. Update `globalmind-runtime/README.md`. A failed candidate is benched with
its evidence; it is not patched until it passes.

# Capability Atlas Contract — Required Before Forward-Pass Smoke

## Purpose and authority boundary

GlobalMind will contain one model-neutral **Capability Atlas**: a relational
map of problems, solution families, capabilities, runbooks, MCP servers,
plugins, tools, skills, hooks, rules, evidence, and known failures. It answers
“what could solve this problem?” without pinning every tool definition in the
context window.

It is not a second mind map and not the execution authority. The live runtime
registry remains authoritative for what is installed, healthy, permitted, and
version-compatible. The harness remains authoritative for policy enforcement
and execution. The Atlas may propose; it cannot grant permission or execute.
Mandatory safety and model-I/O rules stay enforced in Rust and may never depend
on successful recall. Capability atoms belong to semantic GlobalMind, never to
the nested transformer tensor-K/V population.

## Required atom kinds

Add migration `0012_capability_atlas.surql`. Use the existing immutable `atom`
identity/address contract and attach exactly one immutable capability profile:

| Kind | Meaning | Required payload |
| --- | --- | --- |
| `problem_pattern` | Observable problem or intent | symptoms, constraints, success condition |
| `solution_family` | Broad approach neighborhood | scope, exclusions, selection criteria |
| `capability` | Abstract operation | input/output contract, effects, failure modes |
| `runbook` | Ordered operational procedure | steps, stop/rollback rules, evidence required |
| `mcp_server` | MCP capability provider | server identity, transport, advertised contracts |
| `plugin` | Independently loadable extension | loader contract, isolation and unload behavior |
| `tool` | Callable operation | typed arguments/result, side effects, timeout |
| `skill` | Reusable reasoning/work procedure | trigger, required references, completion gates |
| `hook` | Event-triggered operation | event, ordering, idempotency and failure policy |
| `policy_rule` | Constraint interpreted by harness | scope, severity, enforcement point |
| `evidence_receipt` | Immutable qualification result | subject/config/runtime digests, tier, result |
| `failure_pattern` | Known incompatibility or failure | signature, cause, containment, recovery |

Every profile requires: stable record ID, kind, canonical name, summary,
version, content digest, source digest, configuration/contract digest,
model-neutral embedding/address, lifecycle (`candidate|qualified|benched|retired`),
and immutable admission/judge receipts. Source identity is audit-only and must
not affect coordinates or ranking. Secrets, credentials, live tokens, and K/V
tensors are forbidden.

## Required typed edges

Edges are immutable, evidence-bearing, and directional:

- `solves`: capability or family addresses a problem pattern.
- `specializes`: child narrows a parent family/capability.
- `implements`: tool/skill/plugin/runbook realizes a capability.
- `exposes`: MCP server or plugin exposes a tool.
- `requires`: subject cannot run without target.
- `composes_with`: qualified positive composition.
- `conflicts_with`: versions/policies/resources cannot coexist.
- `fallback_for`: target is an evidence-backed fallback.
- `enforced_by`: policy rule is enforced at a named harness boundary.
- `triggered_by`: hook responds to a typed event.
- `verified_by`: evidence receipt qualifies its subject.
- `failed_on`: subject produced a failure pattern under a bound configuration.
- `supersedes`: newer immutable version replaces an older one.

Every edge requires `from`, `to`, edge kind, confidence, evidence receipt,
producer, revision, validity interval, and compatibility digest. No untyped
“related” edge is recall-admissible. Cycles are allowed in the Atlas; the
selected execution plan must be an acyclic DAG.

## Native capability scaffold

Implement under `globalmind-runtime/src/capability/`, not in the harness. Use
Rust structs plus a bounded native binary envelope; no JSON, JSONL, Python, or
prompt-generated tool schema crosses the planning/execution boundary.

Required scaffold fields:

1. immutable capability/profile and edge records;
2. exact argument/result schema digest;
3. effect class (`read_only|local_write|external_write|destructive`);
4. permission and credential classes, never credential values;
5. platform/runtime/version requirements;
6. timeout, retry, idempotency, rollback, and resource limits;
7. health-check and live-registry identity;
8. component, integration, and live evidence receipts;
9. explicit incompatibilities and benched states;
10. compact context projection containing only the selected callable contract.

Scaffolding transaction: verify source and contracts; independently judge;
create the atom/profile; create typed edges; replay neighborhood queries; then
crystallize. Updating a tool creates a new immutable version plus `supersedes`;
it never overwrites qualified history.

## Retrieval and planning algorithm

1. Convert the task into a `problem_pattern` query and enter the nearest
   solution-family neighborhood.
2. Expand at most two typed hops initially. Search need not find the exact tool.
3. Remove benched/retired/conflicting candidates.
4. Reconcile every remaining candidate with the live registry and current
   permissions. Missing or mismatched entries are unavailable, regardless of
   remembered evidence.
5. Rank complete compatible compositions using evidence, expected utility,
   resource cost, and prediction error—not popularity or source identity.
6. Materialize the chosen composition as a separate execution DAG.
7. Project only the selected typed contracts into context and execute through
   harness policy gates.
8. Record outcome/prediction error. Utility receipts may be appended; semantic
   coordinates change only through normal judge/replay/crystallization.

## Audit gates before forward-pass smoke

The capability scaffold passes only if an auditor can prove:

- one GlobalMind authority and zero duplicate capability maps;
- all twelve atom kinds and thirteen edge kinds are schema-enforced;
- untyped edges, missing receipts, duplicate identities, and overwrite attempts
  fail closed atomically;
- approximate retrieval landing on a solution family reaches at least one
  correct qualified composition within two hops on a fixed real task set;
- every proposed executable reconciles against the live registry;
- unavailable, unauthorized, conflicting, benched, and digest-mismatched tools
  are excluded before planning;
- the resulting execution graph is acyclic even when Atlas neighborhoods cycle;
- mandatory rules remain enforced when SurrealDB/Atlas retrieval is unavailable;
- only selected contracts enter context; no global tool-list injection occurs;
- no JSON/Python/text tensor channel exists inside model I/O;
- restart replay returns identical candidate IDs, neighborhood, and DAG digest;
- a real read-only tool call completes through Atlas -> registry -> planner ->
  harness with receipts and no direct Atlas execution path.

Do not delay the model forward-pass smoke for broad Atlas population. Build the
minimum vertical slice: one real problem, two solution families, one qualified
runbook, one live MCP/tool, one unavailable conflicting tool, one mandatory
rule, and one evidence-backed fallback. Audit that slice, run the forward-pass
smoke, then expand the Atlas.

# Model Distillation Firing Runbook — Truthful Current State

## Can distillation be fired now?

No—not through atom and trajectory crystallization. The only working command
currently performs real artifact rehashing (`verify`) or opens an admitted run
envelope (`open`). `open` correctly fails because the Qwen3.6 roster lacks
eligibility and independent-judge receipts. There is no native trace producer,
candidate extractor, independent judge, replay walker, or end-to-end CLI yet.
Anyone claiming atoms from the current command is fabricating the result.

What exists is enough to divide and implement the remaining pipeline without
re-designing it. Use the exact contracts below.

## Working preflight command

```bash
cd /Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime
cargo run --release --bin globalmind-distill -- \
  verify manifests/qwen36-opus-candidate.manifest
```

Expected current identity:

```text
verified_model_count=1
input_digest=cf80d582fa1528b356586d71ca221c69af1a935e9e1159f8409e83b3992d4c08
run_id=run_cf80d582fa1528b356586d71ca221c69af1a935e9e1159f8409e83b3992d4c08
database_writes=0
```

If any identity changes, stop. Reconcile the model files, manifest, TPPE
contract, and configuration; do not patch the expected digest.

## Required native stage commands to implement

Extend `globalmind-distill` with these subcommands. Each consumes a bounded
native binary artifact from the previous stage and emits a SHA-256 receipt.
Text manifests are operator configuration only; tensors/activations never pass
through text or JSON.

```text
globalmind-distill verify <manifest>
globalmind-distill trace <manifest> <token-tape.bin> <trace.gmtrace>
globalmind-distill extract <trace.gmtrace> <candidates.gmcandidate>
globalmind-distill judge <candidates.gmcandidate> <verdicts.gmverdict>
globalmind-distill open <admitted-manifest>
globalmind-distill replay <trace.gmtrace> <verdicts.gmverdict> <walk.gmwalk>
globalmind-distill crystallize <walk.gmwalk>
globalmind-distill audit <run-id>
```

`trace`, `extract`, `judge`, `replay`, `crystallize`, and `audit` are **not
implemented today**. Agents must not create placeholder receipts or silently
skip them.

## Stage contracts

### 1. `trace`

Run the real frozen model twice on the same immutable token tape: full-forward
control and observer-enabled extraction. Emit native layer/token activations,
tensor shape/dtype, model/config/tokenizer/runtime/hardware digests, logits,
and observer-nonperturbation receipt. Fail unless control versus observed logits
and state are within preregistered tolerance. No prompt-generated examples.

### 2. `extract`

Run both TPPE radial candidates over the same trace. TPPE state must include T2
phase, cumulative revolutions, hyperbolic distance `u`, Poincare radius
`tanh(u/2)`, positive boundary margin, and Lorentz invariant receipt. Emit:

- rank 0: reconstructable teacher-neutral atom candidates;
- rank 1: typed transitions with endpoints and evidence;
- rank 2: closed-loop candidates with invariant proof;
- rank <= 8: local Jacobian charts only where marginal gain is measured.

Teacher/model identity remains in the audit envelope but is excluded from
candidate digest, coordinate, novelty, and ranking.

### 3. `judge`

Use a separately versioned judge and frozen thresholds. Check reconstruction,
novelty, model-neutrality, graph consistency, TPPE invariants, evidence, and
rank-specific requirements. Persist every pass and rejection. Rejected
candidates cannot mint atoms, edges, loops, or charts.

### 4. `open`

Only after real eligibility and judge receipt digests are inserted into a new
manifest may `open` create the source, admission, run, and opened event in an
isolated SurrealDB. Never edit the current candidate manifest in place; create
`manifests/<run-id>.admitted.manifest` so provenance remains auditable.

### 5. `replay`

Walk admitted rank-1 transitions in order, verify endpoint and TPPE continuity,
prove every rank-2 loop closes, reconstruct rank-0 atoms, validate rank-8
charts, and rerun after process restart. Emit one immutable walk artifact and
replay receipt. A different model, radial law, TPE configuration, address
contract, runtime, or digest is a miss—not a conversion.

### 6. `crystallize`

Submit one atomic Surreal transaction through the existing typed store:
trajectory header, origins, edges, edge crystallizations, ordered steps, loops,
and charts. Any failure writes nothing. Read everything back, restart SurrealDB,
replay again, and only then append the crystallized run event.

### 7. `audit`

Reconcile source candidates, passed/rejected verdicts, written atoms/edges/
loops/charts, trajectory ordinals, run events, and receipt digests. Counts must
balance exactly. Produce a plain-text report under `results/distillation/`.

## Overnight implementation ownership

1. Agent Trace owns only `src/trace/` and the `trace` CLI branch.
2. Agent Extract owns only `src/extract/`, TPPE consumption, and `extract`.
3. Agent Judge owns only `src/judge/` and `judge`.
4. Agent Replay owns only `src/replay/` and `replay`.
5. Agent Integration owns only `tests/distillation_surreal_live.rs`,
   `crystallize`, `audit`, and `results/distillation/`.

Trace must land first. Extract and Judge may define formats/tests in parallel,
but cannot claim integration until they consume the real trace. Replay depends
on admitted verdicts. Integration is last. One coordinating owner resolves
shared edits to `src/bin/globalmind_distill.rs` and `src/lib.rs`; agents should
submit isolated patches rather than concurrently editing those files.

## First legitimate firing sequence

After all six missing commands pass component tests, execute against a new
isolated database only:

```bash
export GLOBALMIND_SURREAL_URL=http://127.0.0.1:8000
export GLOBALMIND_SURREAL_NS=globalmind_distill_001
export GLOBALMIND_SURREAL_DB=globalmind_distill_001
export GLOBALMIND_SURREAL_USER=root
# Set GLOBALMIND_SURREAL_PASS interactively; never write it into a file/log.

cargo run --release --bin globalmind-distill -- verify manifests/qwen36-opus-candidate.manifest
cargo run --release --bin globalmind-distill -- trace manifests/qwen36-opus-candidate.manifest inputs/real-token-tape.bin results/distillation/run-001.gmtrace
cargo run --release --bin globalmind-distill -- extract results/distillation/run-001.gmtrace results/distillation/run-001.gmcandidate
cargo run --release --bin globalmind-distill -- judge results/distillation/run-001.gmcandidate results/distillation/run-001.gmverdict
# Generate the admitted manifest from verified receipts; never hand-type hashes.
cargo run --release --bin globalmind-distill -- open manifests/run-001.admitted.manifest
cargo run --release --bin globalmind-distill -- replay results/distillation/run-001.gmtrace results/distillation/run-001.gmverdict results/distillation/run-001.gmwalk
cargo run --release --bin globalmind-distill -- crystallize results/distillation/run-001.gmwalk
cargo run --release --bin globalmind-distill -- audit run-<verified-input-digest>
```

Do not run this block until the missing commands exist and `inputs/real-token-
tape.bin` is a reviewed real workload. The immediate judge-facing target is one
small, fully evidenced trajectory—not a large unverified distillation dump.

# Governing Thesis and Post-Vertical-Slice Queue

## Thesis to implement and test

GlobalMind is intended to externalize reconstructable, model-neutral knowledge
and validated reasoning trajectories from multiple heterogeneous frozen models.
The serving model should be replaceable without rebuilding semantic memory.
TIT (Toroidal Isotropic Transformer) in the harness should retrieve the smallest
useful neighborhood, walk existing trajectories, and allocate only the
attention/forward computation required by the current prediction error.

This can make accumulated knowledge actionable without fine-tuning the serving
model. It does not transfer a teacher's weights or automatically grant the
student all teacher capabilities. “Quadratic compounding intelligence” remains
a falsifiable hypothesis: cross-linked atoms may create superlinear reuse, but
hubness, contradictions, retrieval interference, judge bias, traversal cost,
and saturation may make the measured curve linear or worse.

## Model-neutral versus exact-model state

GlobalMind semantic population may contain facts, claims, evidence, provenance,
problem/solution structures, capability relationships, typed transitions,
reconstructable trajectories, counterexamples, and failure patterns. These
must exclude teacher identity from coordinates, novelty, ranking, and execution
gates.

Transformer K/V, hidden activations, tokenizer state, layer/head shape, numeric
layout, and exact prefix cache remain model-specific. They may share a
GlobalMind address, well, and trajectory link, but remain a separate nested K/V
population. Cross-model tensor reuse is forbidden unless model, tensor shape,
tokenizer/prefix, TPPE version/configuration, address contract, and all content
digests match exactly.

## Learning path while TIT is in the harness

Neither TIT nor an MCP-connected model may directly mint trusted memory. The
only admitted path is:

```text
retrieve neighborhood
→ walk or propose trajectory
→ emit candidate atoms/edges/loops/charts
→ independent judge
→ deterministic replay
→ atomic Surreal crystallization
```

Runtime prediction error may deepen retrieval, extend a walk, request a partial
or full forward pass, or open Superposition Holding. It cannot be treated as
truth. A model evaluating its own proposal without an independent gate is a
self-confirming loop and must fail closed.

## Free-energy and thermodynamic claim boundary

Active Inference supplies a controller objective: reduce precision-weighted
prediction error while choosing retrieval depth and forward mode. This is not
proof of lower electrical energy or heat. Landauer's principle is a physical
lower bound, not evidence that this implementation approaches it. Every claim
must report task quality, admitted/rejected coverage, latency, memory, wall
power, temperature, and full-forward control together.

## Queue 1 — Complete the first legitimate vertical slice

Do these before broader ablations:

1. Native real-model trace with observer-nonperturbation receipt.
2. Rank-0/1/2 candidate extraction with both TPPE radial laws; rank-8 remains
   optional until it proves marginal gain.
3. Frozen independent judge and explicit rejection records.
4. One replayed and atomically crystallized real trajectory in isolated
   SurrealDB, followed by restart/read/replay.
5. One real tensor-K/V page round trip with exact-model logit parity.
6. One Capability Atlas path from approximate problem retrieval through live
   registry reconciliation to a real read-only tool call.
7. One harness forward-pass smoke proving fail-closed behavior when GlobalMind
   is unavailable or identity contracts mismatch.

## Queue 2 — Immediate causal ablations

Run on the same frozen model, token tape, workload, hardware state, and judge.
Change one variable at a time.

1. **Memory value:** full forward without GlobalMind versus full forward with
   semantic atom retrieval.
2. **Trajectory value:** atom-only retrieval versus atoms plus crystallized
   trajectories.
3. **Geometry:** flat/vector neighborhood versus hyperbolic neighborhood at
   matched candidate count and storage budget.
4. **Position encoder:** RoPE control, angle-only toroidal phase,
   `linear_log_v1`, and `log_v1` TPPE.
5. **Retrieval depth:** zero, one, two, and adaptive hops; report quality,
   collision/hubness, tokens injected, and latency.
6. **Forward allocation:** full forward, fixed partial forward, and TIT adaptive
   forward under matched output-quality thresholds.
7. **Model swap:** distilling model, same-family replacement, and different-
   family replacement using identical semantic memory and no fine-tuning.
8. **Judge independence:** self-judge versus separate frozen judge versus
   adversarial disagreement set. Self-judge can never be the production gate.
9. **K/V contribution:** semantic retrieval alone versus semantic plus valid
   exact-model nested K/V; mismatched K/V must prove a clean miss.
10. **Surprise controller:** fixed retrieval depth versus precision-weighted
    prediction-error routing, with calibration and unnecessary-forward rate.

Promotion requires non-inferior task quality and a measured resource benefit.
If a candidate loses after the preregistered sweep, bench it with receipts.

### Queue 2 TPPE correction — 2026-11-02

The first TPPE qualifier was invalidated: it gave full-forward zero error and
zero time by construction, measured byte-derived drive magnitude rather than
model quality, omitted state bytes from dominance, and labeled a two-scalar
phase recurrence as RoPE. Its bench verdict is void; no encoder is benched.

The corrected runner installs native RoPE, angle-only toroidal phase,
`linear_log_v1`, and `log_v1` TPPE at the same real Q/K boundary and measures
actual 248,320-vocabulary next-token NLL, real wall time, encoder state, and
logit divergence. A one-repeat three-token smoke passed, but is deliberately
decision-open. See `results/queue2/ADVERSARIAL-AUDIT-2026-11-02.md` and
`results/queue2/tppe-bench-v2-smoke.txt`. The next valid decision requires a
contamination-checked held-out task/token suite and counterbalanced repeats.

Rollback boundary (2026-11-02): Queue 1.2 TPPE qualification only — not
Queue 1.1 tracing machinery. Still valid and not to be rebuilt: SurrealDB
schemas and atomic transactions, native binary trace envelope, observer
nonperturbation proof, extractor/judge/replay mechanics, Capability Atlas,
harness fail-closed behavior, TPPE structural invariants, tensor K/V
identity and byte-parity machinery, and the Queue 2 geometry result as a
component-only measurement.

Corrections now in force:

1. Queue 1.2 no longer says TPPE/RoPE candidates were validly benched
   (`results/queue1-vertical-slice-receipt.txt` Q1.2 rows are marked VOID).
2. Queue 2.4 must be rerun completely using the corrected benchmark
   (`results/queue2/queue2-ablation-report.txt` A4 reads
   `INVALIDATED_BY_AUDIT — RERUN REQUIRED`; promoted: none, benched: none).
3. Existing distilled atoms and trajectories are labeled integration
   fixtures produced under `tppe_linear_log_v1_candidate` (see
   `results/distillation/` reports and the Q1.4 fixture label). They prove
   the pipeline works, not that this is the winning positional encoder.

After the real held-out bakeoff: if `linear_log_v1` wins and its contract
remains byte-identical, retain the existing vertical-slice trajectory and
rerun it for final current receipts — no architecture rebuild. If `log_v1`,
angle-only, native RoPE, or a revised TPPE wins, re-execute (not redesign)
trace → extract → judge → open new run → replay → crystallize → audit →
tensor K/V qualification, because TPPE identity affects Q/K states and
logits, atom coordinates, candidate digests, trajectory continuity, replay
receipts, and tensor K/V identity. Queue 1.6 Capability Atlas and Queue 1.7
fail-closed harness smoke require rebuilding only if their contracts
explicitly embed the old TPPE digest.

## Queue 3 — Compounding-intelligence experiment

Only after Queue 2 identifies a stable configuration, add eligible teachers in
fixed increments. At each increment freeze the graph and evaluate the same
held-out task suite. Report teacher count, admitted atoms, edges, trajectories,
coverage, contradictions, retrieval precision/recall, task quality, latency,
memory, wall energy, and judge disagreement.

Fit and compare constant, linear, sublinear, and quadratic response models with
held-out prediction error and confidence intervals. The quadratic claim passes
only if the quadratic model predicts held-out gains better than simpler models,
the gain persists after controlling for graph size and retrieval compute, and
quality does not come from contamination or teacher identity leakage.

Required controls:

- same atom count with shuffled edges;
- same graph with teacher labels removed (they should already be audit-only);
- duplicate-teacher ingestion;
- contradictory-teacher injection;
- random-trajectory and nearest-vector baselines;
- graph-size-matched single-teacher distillation;
- unseen-model consumer with no fine-tuning.

## Queue 4 — Online trajectory learning

Enable MCP/TIT candidate submission only after offline crystallization passes.
Test read-only walking first, then candidate generation with writes disabled,
then judged writes in an isolated database. Required adversarial cases include
self-reinforcing loops, prompt injection inside retrieved atoms, repeated
candidate flooding, cyclic trajectories, stale tools, conflicting evidence,
judge outage, Surreal restart, and partial transaction failure.

Online promotion requires bounded write rate, independent judgment, replay,
atomicity, rollback, provenance, duplicate suppression, and proof that rejected
candidates never affect recall. Production GlobalMind remains read-only until
all gates pass.

## Queue 5 — TIT resource and thermal qualification

Compare TIT adaptive execution against the full-forward baseline over easy,
medium, and hard tasks. Record retrieved depth, layers/heads activated, K/V hit
rate, forward mode, quality, time-to-first-token, tokens/second, peak memory,
wall energy per answer, sustained power, and temperature. Include warm/cold
cache and long-context conditions.

Do not claim free-energy or Landauer efficiency from prediction-error reduction.
The implementation earns an efficiency claim only from measured hardware
energy/heat at non-inferior quality. If TIT repeatedly routes around difficult
reasoning, loses quality, or costs more than full forward, bench the adaptive
path and retain full forward.

## Judge-facing evidence package

Prioritize a narrow, undeniable demonstration over a large graph:

1. exact frozen-model and runtime identities;
2. one real multi-step trajectory from trace through crystallization;
3. restart-identical Surreal replay;
4. one successful model swap using the same semantic trajectory;
5. one exact-model K/V hit and one mismatched clean miss;
6. one Capability Atlas approximate retrieval and live tool execution;
7. one measured TIT routing decision against full forward;
8. complete receipts plus explicit open gates.

This package demonstrates the architecture without claiming quadratic
intelligence, universal model interchangeability, or thermodynamic superiority
before those experiments exist.

# Product Direction — Decouple the Mind from the Model

## Product thesis

Host proof-carrying Cognitive IR and the Capability Atlas as a model-independent
GlobalMind service. Any compatible harness may replace its serving model while
retaining accumulated organizational knowledge, validated reasoning paths,
failure evidence, and tool-resolution capability.

The customer promise is:

> Keep your mind when you change your model.

Models become replaceable compute engines. The durable asset is the customer's
auditable cognitive substrate. The shared network becomes more valuable only
when independently verified contributions improve cross-model problem solving;
raw atom volume is not the objective.

## Hosted system boundary

```text
customer model or harness
        ↓
MCP control plane / native SDK
        ↓
GlobalMind retrieval + Capability Atlas planning
        ↓
proof-carrying atoms and trajectories
        ↓
customer-local harness execution
        ↓
candidate experience submission
        ↓
privacy/license gate → judge → replay → deduplicate → promote
```

GlobalMind proposes knowledge and executable Cognitive IR. The customer's
harness enforces permissions and executes. A hosted service never receives
authority merely because an MCP connection exists.

## Three mandatory memory scopes

- `private`: visible only to one user or isolated deployment.
- `organization`: shared only within an explicit tenant boundary.
- `global`: independently qualified, licensed, consented knowledge available
  to the shared network.

Every atom, trajectory, edge, receipt, and capability carries one immutable
scope plus tenant and policy identities. Promotion across scopes is a new
judged transaction requiring owner consent; it is never an in-place flag
change. Tenant retrieval must fail closed before ranking so private coordinates,
counts, and neighborhood structure cannot leak through global results.

Raw prompts, credentials, personal data, proprietary source, customer secrets,
and exact-model transformer K/V are forbidden from the global population.
Exact-model K/V should normally remain customer-local even when semantic
GlobalMind is hosted.

## Real-time contribution without direct global writes

Clients may submit candidate experiences in real time, but no client, model,
MCP server, TIT controller, or employee tool writes directly into qualified
shared recall. Promotion is asynchronous:

```text
tenant-private candidate
→ secret/PII and policy scan
→ ownership, consent, and license receipt
→ provenance and signature verification
→ novelty and duplicate reconciliation
→ independent judgment
→ adversarial deterministic replay
→ cross-model neutrality test
→ tenant or global crystallization
```

Failure at any stage leaves the qualified graph unchanged. Rejected candidates
remain isolated evidence or are deleted according to tenant retention policy.
Self-confirming retrieval/submission loops, candidate flooding, reputation
gaming, and coordinated poisoning require explicit adversarial tests.

## MCP control plane and native data plane

MCP is the compatibility and discovery surface for:

- searching solution families and semantic neighborhoods;
- retrieving compact proof-carrying Cognitive IR;
- resolving Capability Atlas entries against a live registry;
- submitting candidate references;
- reading verdicts, receipts, provenance, and availability;
- administering tenant policy and memory scope.

MCP is not the tensor or activation hot path. High-volume traces, Cognitive IR
artifacts, and any permitted tensor payload use a bounded, authenticated,
versioned Rust-native binary data plane. MCP returns signed identities and
handles to those artifacts rather than embedding binary state in JSON-RPC.
JSON never crosses the model/tensor I/O boundary.

## Trust and contribution economics

Global contribution value is measured by verified marginal utility, not upload
count. Each contributor and artifact accumulates evidence across independent
models, tasks, tenants, judges, and time. Duplicate knowledge earns negligible
credit. Contradictions are retained as typed competing claims until evidence
resolves them; popularity cannot overwrite provenance.

Contributor incentives may reward trajectories that improve held-out success,
reduce compute at matched quality, identify reproducible failures, or connect
previously isolated solution families. Rewards require anti-sybil controls,
license clarity, replay, contamination checks, and delayed settlement so later
failures can affect trust.

## Commercial surfaces

- private hosted GlobalMind per organization;
- VPC, on-premises, or fully local sovereign deployment;
- usage-based verified trajectory retrieval and replay;
- Capability Atlas and live tool-resolution service;
- independent trajectory verification/certification;
- cross-model migration and portability qualification;
- enterprise provenance, retention, audit, and policy controls;
- opt-in contributor marketplace for globally useful Cognitive IR.

Do not sell unqualified “collective intelligence.” Sell portability,
provenance, verified reuse, lower switching cost, and measurable task/resource
improvement.

## Defensible moat

The moat is the quality-weighted graph and its evidence system:

- portable cross-model Cognitive IR;
- successful and failed trajectory receipts;
- model-neutrality and model-swap measurements;
- capability/tool compatibility and failure knowledge;
- independent replay and adversarial qualification infrastructure;
- trust-weighted provenance and contributor history;
- solution-family topology improved by real outcomes;
- proof that smaller/cheaper models gain capability without fine-tuning.

Neither model access, an MCP wrapper, nor a large unverified vector collection
is sufficient differentiation.

## Product qualification gates

Before any multi-tenant or global launch, prove:

1. cryptographic tenant isolation and authorization before retrieval;
2. consent, ownership, license, deletion, export, and retention workflows;
3. secret/PII detection with fail-closed quarantine and human escalation;
4. signed submissions, receipts, replay artifacts, and append-only audit trail;
5. poison, sybil, prompt-injection, exfiltration, inference, and flooding tests;
6. no direct path from candidate submission to qualified recall;
7. judge diversity, outage behavior, disagreement handling, and rollback;
8. exact deletion semantics for private data and derived global artifacts;
9. customer-owned export that allows leaving the hosted service;
10. measured cross-model value on held-out tasks without fine-tuning.

The first commercial demonstration should use two isolated tenants, one opted-
in globally shareable trajectory, one forbidden private trajectory, two model
families, and a complete export/delete/replay audit. Scale only after this
vertical slice proves that collective improvement does not sacrifice ownership
or tenant safety.
