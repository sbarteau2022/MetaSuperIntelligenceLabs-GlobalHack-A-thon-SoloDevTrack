---
title: "GlobalMind SurrealDB Distillation and Trajectory Integration"
document_type: "architecture"
version: "1.0.0"
date: "2026-11-01T21:34:29Z"
implementation_date: ""
author: "Codex"
reviewers: []
status: "draft"
related_files:
  - path: "schema/0007_globalmind_distillation_trajectories.surql"
    lines: "1-229"
    role: "primary"
  - path: "../MasterDesignDocument/C12-Server-Inference-Engine/FreshExecutionAndMindDesign.md"
    lines: "1-175"
    role: "supporting"
related_issues: []
related_documents:
  - "HYPERBOLIC-KV-SURREAL-DESIGN.md"
  - "../DistillationProtocol/PROTOCOL.md"
tags: ["globalmind", "surrealdb", "distillation", "trajectories", "tensor-kv"]
---

# Executive Summary

GlobalMind now has an additive SurrealDB contract for one cognitive hub with
twelve surrounding wells, frozen-weight and trajectory distillation packets,
surprise observations, reconstructable atoms,
rank-8 local charts, replayable trajectories, and exact-model tensor K/V
bindings. The migration leaves the current database quarantined and does not
promote old atoms merely because they predate the new fields.

# Problem Statement

The prior schema could store atoms and an ordered array of trajectory atom IDs,
but it could not prove how an atom was distilled, whether its payload was
reconstructable, how each trajectory step replayed, whether a loop closed, or
whether a tensor cache page matched the recalled trajectory and cognitive well.
That made the intended integration representable only in prose.

# Solution Overview

Migration `0007` adds immutable records around the existing graph. It separates
semantic memory from model-specific tensor cache pages while allowing both to
share the same address, well, TPPE, and trajectory identifiers. Existing rows
remain valid historical data but are not recall-admissible under this contract.

# Research Question and Acceptance Criteria

**Question.** [PROPOSED] What minimum persistent contract lets multiple frozen
models contribute model-agnostic knowledge and replayable trajectories to one
GlobalMind while exact transformer K/V remains isolated by model identity?

**Decision unblocked.** [PROPOSED] This decides what a fresh distillation runner
must write transactionally and what the runtime may recall.

**Criteria, preregistered before migration validation.** [PROPOSED]

1. One central hub has exactly twelve unique surrounding wells inside one
   geometry contract.
2. Every admitted atom traces to a judged packet and reconstruction receipt.
3. Every admitted trajectory has ordered typed steps and a replay receipt.
4. A rank-2 claim has an explicit loop record and closure receipt.
5. Rank-8 charts store binary tangent data and marginal-gain evidence.
6. Tensor pages bind to exact model/TPPE/address contracts and remain outside
   semantic atom recall.
7. Legacy/quarantined records are not silently upgraded or overwritten.

# Source Provenance Register

| ID | Source | Version | License | Claim used |
| --- | --- | --- | --- | --- |
| S1 | `DistillationProtocol/PROTOCOL.md` | current checkout, 2026-11-01 | not stated | Teacher identity is audit-only; atoms and trajectories are teacher-neutral. |
| S2 | `schema/0001_hypermind_graph.surql` | current checkout | not stated | Atom, address, embedding, and typed-edge base schema. |
| S3 | `schema/0004_transformer_kv_tensor_pages.surql` | SHA-256 `31c467e26dd81d9fddf61e10b605e6726a25b7c0cb573c399d73d79fec7f1246` | not stated | Exact model/session layer-head tensor cache identity. |
| S4 | `schema/0005_universal_atom_lineage.surql` | SHA-256 `a2c4a85303874233048b73d18e36538f38c05e36b2503c37499b6489a92729c0` | not stated | Immutable admission, origin, and trajectory headers. |
| S5 | `C06/TrajectoryToCapabilityFormalization.md` | current checkout, PROPOSED | not stated | Rank-0 atoms, rank-1 transitions, rank-2 loops, and rank-8 charts. |
| S6 | `C06/VaultMapDreamstateFormalization.md` | current checkout | not stated | Twelve-sector evidence exists; unified quotient remains unproved. |
| S7 | AtomaApp `MADMIND-STRUCTURE.md` and `backend/src/mind.rs` | local checkout, 2026-11-01 read | not stated | No twelve-well or `r12` formalism was found; these describe a workflow bus and JSON run summaries. |
| S8 | Friston et al., “Active Inference: A Process Theory,” DOI `10.1162/NECO_a_00912` | Neural Computation, 2017 | publisher license | Active inference propagates precision-weighted prediction errors; variational free energy bounds observation surprise. |
| S9 | `/Users/stewartbarteau/Desktop/N3/design/superpositionholding.md` | SHA-256 `33d8b535665bca6bd6f4005940adb115b7805c8868843e70a2e66a35ef61c60b` | not stated | User-directed behavioral requirement: stop emission, hold candidate branches, iterate, collapse on a replayable branch, or fail closed. No implementation code imported. |

# Observed Findings

**[VERIFIED; S2–S4]** The old persistent contract had atom provenance and an
ordered trajectory atom list, but no typed step, replay state, loop invariant,
atom reconstruction proof, chart record, or well membership.

**[VERIFIED; S6]** The checked-in mathematics supports separate twelve-sector
phase behavior and hyperbolic reduction. It explicitly does not prove the
unified quotient action. A well may therefore be `specified` before it is
`certified`; its boundary digest cannot be invented by the database.

**[VERIFIED; S7]** The inspected AtomaApp files do not supply the missing
twelve-well formalism. They are not evidence for cognitive-well geometry.

# Integration Contract

## One authority and two populations

**[PROPOSED]** SurrealDB contains one GlobalMind graph. A `cognitive_hub`
records the central reference state; twelve peer `cognitive_well` records
surround it through `hub_has_well`. No well is reserved as the K/V cache.
Semantic atoms,
transitions, loops, and charts form the teacher-neutral recall population.
Transformer K/V pages form an exact-model cache population. Sharing an address
or well creates a traversal link, not permission to blend the populations.

## Distillation lifecycle

**[PROPOSED]** A fresh runner performs these state changes in order:

1. Verify live model artifact digest and create `model_source` plus
   `distillation_admission`.
2. Open `distillation_run` with frozen configuration and input digests.
3. Emit immutable `distillation_packet` candidates from frozen weights or
   walked trajectories. Teacher identity is stored through `admission`; the
   teacher-neutral digest is computed without it.
4. Run novelty and independent judge gates. Persist admitted and rejected
   verdicts; rejected packets mint nothing.
5. For `rank0_atom`, create atom/address/origin plus `atom_crystallization` in
   one transaction. Recall requires the reconstruction receipt.
6. For `rank1_transition`, create the typed `atom_edge` and ordered
   `trajectory_step`. Endpoint continuity must hold between adjacent ordinals.
7. For `rank2_loop`, require identical start/end atom and persist the tested
   loop invariant in `trajectory_loop`.
8. For `rank8_jacobian`, write binary tangent data only after marginal-gain
   admission; `chart_rank` may stop below eight.
9. Replay the full ordered walk. Only then set a new trajectory header to
   `replay_passed`; crystallization creates a separate receipt and state.
10. Complete the run with output and receipt digests. A partial failure marks
    the run failed and leaves no partially admitted semantic records.

Steps 6–9 form one crystallization transaction. Each transition carries its
admitted rank-1 packet and audit admission; the transaction creates the graph
edge rather than trusting a pre-existing edge ID. Rank-2 loop ranges must close
on the same atom and carry their admitted packet. Trajectory origins are
audit-only joins and never enter ranking. Rank-8 charts are separate immutable
binary records anchored to a crystallized atom and admitted packet. Candidate
and rejected walks cannot enter the recall population.

### Native ingress transaction boundary

Before step 1, the runner must rehash every real rostered artifact and
reconcile the aggregate model roster digest without writing to SurrealDB.
After that gate, one transaction creates the immutable model source, each
artifact receipt, the distillation admission, the run envelope, and its
`opened` event. A digest mismatch, missing artifact, non-absolute path, schema
drift, or quarantined database target yields zero writes. Later lifecycle
changes append run events; they never update an earlier receipt. The opened
event is ingress evidence only and cannot be reported as distillation.

Every run and trajectory is bound to TPPE/TPPE: dynamic toroidal phase with a
radius that expands per revolution. Constant-radius corkscrew behavior is
invalid. Replay must reject a different encoder version/configuration, radius
regression, non-finite state, Lorentz-sheet invariant failure, or ideal-boundary
crossing. The current
reference contract is not runtime qualification; it prevents silent RoPE or
constant-radius fallback while the Rust encoder and hardware gates remain open.

## Recall eligibility

**[PROPOSED]** Semantic recall may use an atom only when it has an active
immutable address, at least one `atom_crystallization`, and a judge-passed
packet. Trajectory recall additionally requires `state = crystallized`, a
non-empty replay receipt, contiguous step ordinals, matching endpoints, and
the current TPPE/address contract. Audit lineage may be returned after ranking
but cannot participate in coordinate projection, rank, novelty, or recall.

## Tensor K/V integration

**[PROPOSED]** The runtime creates `transformer_kv_session` only after exact
model digest, tensor shape, prefix, TPPE configuration, and address contract
match. A page binds to one immutable address and may bind to its classified
well and crystallized trajectory. Any mismatch is a miss. Page bytes are never
embedded, distilled into atoms, or returned by semantic recall.

## Active-inference controller

**[PROPOSED]** `active_inference_decision` records selected wells and
trajectories, retrieval depth, forward mode, controller digest, and separate
quality/resource receipts. The database records the decision; it does not
claim that variational free energy equals physical energy or measured heat.

## Surprise lifecycle

**[PROPOSED]** The active-inference routing signal is immutable,
precision-weighted prediction error attached to the active decision, hub,
selected well, and optional trajectory. The record preserves the signed raw
prediction error, estimated precision, weighted error, and nonnegative
surprisal in nats or bits. Optional Bayesian surprise is recorded separately
because posterior/prior belief change is not the same quantity as observation
surprisal. These values are bound to prior, observation, estimator, and receipt
digests; the database never recomputes them. The record is not K/V tensor data
and is not a thirteenth semantic well. It may cause deeper retrieval,
trajectory extension, a full forward pass, or quarantine. Only a separately
judged and replayed result may crystallize into atoms or trajectories.

## Superposition-holding lifecycle

**[PROPOSED; S9]** A recall miss, cycle, branch ambiguity, high
precision-weighted prediction error, low readiness, or compliance boundary may
open one `superposition_hold`. External emission stops. The controller harvests
at most twelve branches and scores every branch with the same versioned scorer
and compliance gate. Each `superposition_iteration` records counterfactual
rollout, loop check, isotropic suppression, surviving branches, and exactly zero
external emissions. The terminal `superposition_resolution` either collapses
onto one compliant branch, mandates another reasoning cycle, or fails closed.

**[PROPOSED]** Collapse is not automatically learning. A resulting bridge may
enter GlobalMind only through the normal distillation packet, independent
judge, replay, and crystallization gates. This keeps runtime deliberation from
silently writing its own preferred trajectory into shared memory.

## Failure and quarantine rules

**[PROPOSED]** Missing receipts, non-contiguous steps, endpoint mismatch,
uncertified boundary use, teacher-dependent ranking, or tensor identity drift
fail closed. Existing 2,079 atoms remain quarantined because they lack the new
admission proofs. No backfill may fabricate those proofs.

# Implementation Details

| File | Lines | Change Type | Description |
| --- | --- | --- | --- |
| `schema/0007_globalmind_distillation_trajectories.surql` | 1-229 | new | Additive hub/well, surprise, distillation, trajectory, chart, controller, and K/V bindings. |
| `schema/0008_superposition_holding.surql` | 1-84 | new | Immutable hold, branch, counterfactual iteration, and resolution receipts. |
| `schema/0009_immutable_tensor_kv.surql` | 1-49 | new | Immutable exact-model tensor identity and K/V payload. |
| `schema/0010_native_distillation_ingress.surql` | 1-61 | new | Frozen artifact receipts, append-only run events, and TPPE-bound trajectories. |
| `schema/0011_atomic_trajectory_cells.surql` | 1-49 | new | Atomic rank-1 edge proofs, rank-2 loop bindings, rank-8 chart lineage, and per-step TPPE radial state. |
| `GLOBALMIND-SURREAL-INTEGRATION.md` | 1-265 | new | Persistent lifecycle and evidence boundary. |
| `README.md` | 39-55 | modified | Operator-facing status and migration reference. |

SurrealQL is used for constraints, relations, indexes, and append-only records.
Rust remains the required policy and binary tensor transport implementation.

This implementation was not directly grounded in published literature. It
formalizes the user-specified architecture and checked-in protocol; mathematical
and empirical claims remain gated by the referenced project formalizations.

# Testing Evidence

**[VERIFIED]** SurrealDB 3.3.5 accepted migration `0007` on an isolated database
restored from a live `hypermind/core` export. Migration `0007` has SHA-256
`5a511d26efc912f2842d4e7cbff9f090c0272be6a161fef9837b2760dd8d089f`.

**[VERIFIED]** A clean SurrealDB 3.3.5 schema replay accepted migrations
`0001`, `0003`, `0004`, `0005`, `0006`, `0007`, and `0008` in order.
Migration `0008` has SHA-256
`88cd0fea104a64bdddf11134b6539b1d0e93512fd062171b6977549cc64a6f96`.

**[VERIFIED]** A fresh isolated schema replay accepted the chain through
`0010`. Migration `0010` has SHA-256
`148f5ddaa05d4c2a1798d90c9e71e4253d68cd6c83330bc64837fb478821fc7b`.
The release verifier rehashed a real frozen Qwen3.6 candidate roster and
produced input digest
`35518d65ac9eb9ed9139654003074ea675f98fed93724a2d084eff9c6eeb6d14`.
Because eligibility and independent-judge receipts are absent, `open` failed
closed; readback showed zero source, admission, run, and event records. This is
ingress evidence, not distillation or TPPE runtime proof.

**[VERIFIED]** SurrealDB 3.3.5 accepted the isolated schema chain through
`0011`; the compiled contract verifier accepted its immutable fields. Migration
`0011` SHA-256 is
`dcd8d7820d88d6dd3d8c02565965d518a4dbe44dace7e1a81acf6a86efea006d`.
The fixed-ID relation statement was parsed in a non-executed branch. No real
trajectory transaction has run, so this is schema/component evidence only.

**[VERIFIED]** The restored database retained all 2,079 atoms with the same
ordered `(id, content_digest)` SHA-256,
`3c1361753219a58a8d4c56f3fde68ba0bf2af3dd3425b9c61c94dc0e15274581`.
It contained zero crystallizations, trajectories, and wells after migration,
which proves legacy rows were not silently promoted. Separate schema checks
rejected ordinal `12`, mutation of a READONLY well label, and duplicate
`(geometry_space, t2_sector)`. This does not prove a real distillation, runtime
recall, tensor reuse, or twelve-well mathematics.

Reproduce by exporting `hypermind/core`, restoring it under an isolated
SurrealDB 3.3.5 endpoint, applying `0007`, and comparing ordered atom IDs and
content digests to the live read-only query. Do not run `0007` against the live
database while it is quarantined.

# Deployment / Rollout

| Aspect | Detail |
| --- | --- |
| Deployment | Not deployed; live `hypermind/core` remains quarantined. |
| Strategy | Apply to an isolated namespace, run fresh real-model distillation, then qualify promotion. |
| Rollback | Drop the isolated namespace; no legacy row is modified. |
| Monitoring | Admission failures, step continuity, replay verdicts, tensor misses by mismatch reason, resource receipts. |
| Promotion | Requires real model, real trajectory, restart, recall-invariance, and forward-cache evidence. |

# Next Stage of Implementation

| Next Step | Description | Depends On This? |
| --- | --- | --- |
| Rust admission API | Typed transaction commands for packets, atoms, steps, loops, and charts. | Yes |
| Native tensor transport | Binary K/V write/read in the real forward hot path. | Yes |
| Fresh distillation | Frozen-weight and walked-trajectory run into an isolated DB. | Yes |
| Qualification | Cross-teacher invariance, replay, restart, latency, quality, and resource gates. | Yes |

This implementation sets up the requested integration because every intended
artifact now has a persistent identity and admission path. It does not complete
the runtime or certify the live system.

# What We Learned

The prior trajectory array was an index, not a replayable reasoning record.
AtomaApp is not the source of the twelve-well formalism in its current local
files. The twelve-well boundary contract must be supplied or implemented from
the canonical C06 evidence before those wells can move from `specified` to
`certified`.
