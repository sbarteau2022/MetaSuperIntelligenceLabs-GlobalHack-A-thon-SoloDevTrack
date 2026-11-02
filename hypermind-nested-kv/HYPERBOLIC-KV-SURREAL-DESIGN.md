---
title: "Nested Hyperbolic KV in the Multi-Model HyperMind Graph"
document_type: "architecture"
version: "1.0.0"
date: "2026-11-01T15:46:44Z"
implementation_date: "2026-11-01T15:46:44Z"
author: "OpenAI Codex"
reviewers: []
status: "draft"
related_files:
  - path: "schema/0001_hypermind_graph.surql"
    lines: "1-91"
    role: "supporting"
  - path: "schema/0003_nested_hyperbolic_kv.surql"
    lines: "1-156"
    role: "distillation provenance"
  - path: "schema/0004_transformer_kv_tensor_pages.surql"
    lines: "1-80"
    role: "primary"
  - path: "README.md"
    lines: "7-28"
    role: "supporting"
  - path: "../research-grade-harness/README.md"
    lines: "48-68"
    role: "supporting"
related_issues: []
related_documents:
  - "PLAIN-LANGUAGE-SPEC.md"
  - "SYMPLECTIC-ENVELOPE-SPEC.md"
tags: ["surrealdb", "hypermind", "hyperbolic-kv", "mcp", "research-harness"]
---

# Executive Summary

HyperMind's distilled multi-model graph remains the owner of atoms, typed edges,
and their `H3xT2/r_12` addresses. The nested KV is formalized inside the same
SurrealDB namespace and can point to those atoms without copying the graph. The
research harness reaches both services through the existing `hypermind-mcp`
process, loaded as a harness plugin; the harness never receives database
credentials or writes SurrealQL directly.

# Certification Amendment — 2026-11-01

**Question.** Do the current Surreal atoms implement model-independent distilled
recall, and can the tensor cache reuse gate and provenance survive version
updates? This decides whether existing trajectories may be attached or whether
a new receipted ingest is required.

**Pre-registered judges (before live record inspection).**

1. Count live atoms, addresses, edges, model sources, distillation runs, tensor
   sessions/pages, and verify whether live atoms carry admitted model lineage.
2. Code-path audit: teacher identity must not enter atom coordinate, novelty,
   retrieval, or rank decisions. A twin-fingerprint behavioral receipt is
   required before claiming universal cross-model atom availability.
3. Tensor reuse must compare exact model digest, tensor shape/layout/dtype,
   TPE version, address contract, and K/V digests; a mismatch must be a miss.
4. Version startup must reject an unknown or drifted schema/contract digest;
   immutable historical provenance and trajectory revisions must survive
   migration and restart without overwrite.
5. Trajectory attachment requires source artifact hash, model roster receipt,
   mapped atom endpoints, judge result, and legal competition eligibility.
   Missing evidence means no mint, no retroactive attribution, and no
   certification claim.

**Source provenance register.** `DistillationProtocol/PROTOCOL.md` (local git
source, current checkout, license not stated) specifies teacher-invariant atom
addressing and audit-only lineage. `DistillationProtocol/model_roster.json`
(local git source, current checkout, license not stated) supplies candidate
teacher fingerprints. `schema/0001_hypermind_graph.surql` and
`schema/0004_transformer_kv_tensor_pages.surql` (local git sources, current
checkout, license not stated) specify current atom and cache fields. Live
SurrealDB `hypermind/core` (local operational data, current server, license
not stated) is the source for actual record counts. Claims below must label
VERIFIED, REPORTED, INFERRED, or PROPOSED.

**VERIFIED live audit.** `hypermind/core` contains 2,079 atoms and 2,079
addresses, but zero atom edges, model sources, distillation runs, admitted
origins, teacher-neutral trajectories, or tensor pages. The 2,079 atoms are
1881 `atoma-mind` and 198 `user` provenance strings. None has a
`distillation_run` link. This settles judge 1 as unmet for existing model
lineage; it does not establish that an individual atom came from a teacher.

**VERIFIED protocol boundary.** `DistillationProtocol/PROTOCOL.md` §8 requires
teacher identity to remain out of atom coordinates, novelty, rank, and recall.
Its separate no-cross-identity-KV rule means tensor page compatibility is an
exact model/TPE contract, while atom/trajectory access is model independent.
The current HyperMind query tool accepts text/rank, not a teacher id; the
required twin-fingerprint real-run receipt is still absent. Judge 2 is open.

**VERIFIED migration protection.** Additive schemas `0005` and `0006` are live.
Lineage, trajectory, migration digests, original atom content, and original
H3/T2 address fields are `READONLY`; an attempted digest overwrite in a
cancelled live transaction was rejected and the digest remained intact.
MCP startup now pins append-only migration correction/version digests and
checks critical live field definitions. Judge 4 passes for accidental version
drift through this runtime; a root administrator can still deliberately
redefine schema, so backups and privileged change control remain necessary.

**VERIFIED candidate vault boundary.** The archived 706 MB
`global_active_mind.atoma_map` matches the 2026-10-28 distillation receipt's
SHA-256 `e14d6417...`. Its reader authenticates the binary SHA-256. Its
`ATOMA_KV_V1` atoms and `ModelAdapterLens` use model fingerprints, head-size
adaptation, and RoPE pairs. Those records do not satisfy the HyperMind tensor
contract or model-independent recall requirement. Its 32,675 source atoms and
109,300 arcs have no certified mapping into the 2,079 live HyperMind atoms.
Judge 5 therefore forbids bulk attachment of that vault. A fresh, receipted
HyperMind distillation is required; no archived records were imported.

**PROPOSED remaining certification run.** Verify roster fingerprints against
live weights; run the protocol's teacher-neutral projector, novelty, judge,
and twin-fingerprint gates; mint 0-cells and 1-cell trajectories into the
single Surreal map with immutable admissions; prove recall/rank invariance
under teacher swaps; then connect and benchmark exact-model binary K/V pages
in the real forward pass. Judges 2, 3, and 5 remain unmet until that run.

# Problem Statement

The graph schema already persists atoms, addresses, embeddings, and typed edges,
but the legacy `map.kv_store` cache stores strings rather than transformer
layer/head K and V tensors. It is therefore inadmissible as the SurrealDB cache
authority. Calling a separate database directly from the harness would also
split authority and bypass HyperMind's receipts and policy gates.

# Context

- **VERIFIED:** `schema/0001_hypermind_graph.surql` defines `atom`, immutable
  versioned `atom_address`, `atom_embedding`, and typed `atom_edge` records.
- **VERIFIED:** legacy string-KV tools fail closed in Surreal authority mode.
- **USER-SPECIFIED:** the graph space is written `H3xT2/r_12`.
- **UNRESOLVED:** `r_12` does not yet have an executable group action or
  canonicalization rule in the checked-in design. The schema therefore records
  it as a named quotient contract, not as implemented mathematics.

# Decision

## Ownership boundary

SurrealDB namespace `hypermind`, database `core`, is the persistent authority.
The Rust HyperMind runtime remains the only write-policy authority. The harness
loads that runtime as a plugin and speaks MCP to it:

```text
research harness -> plugin registry -> hypermind-mcp -> typed Rust policy -> SurrealDB
```

"Plugin" describes harness lifecycle and admission. "MCP server" describes the
transport. They are the same canonical executable, not two implementations.

## Runtime persistence invariant

When `HYPERMIND_SURREAL=on`, `hypermind-mcp` hydrates its volatile indexes from
SurrealDB and must never create `.hypermind`, a JSONL ledger, or a file snapshot.
`map.create` is rejected, `map.snapshot` reports database-native durability, and
`map.recover` reloads SurrealDB. `shadow` is a migration posture and is rejected
at MCP startup because it would require a second persistent authority. Explicit
`HYPERMIND_SURREAL=off` retains the isolated file-backed test/development path.

## Graph and address contract

`geometry_space` names the coordinate contract. `atom_address.space` binds each
address revision to it. The initial row is `h3_t2_r12_v1`, with `r_12` explicitly
marked opaque until its action and canonical representative algorithm are
specified and tested. Existing Lorentz and torus coordinate fields are unchanged.

Repository formalism exists in `C06/VaultMapDreamstateFormalization.md` and the
12-around-one construction is discussed in the toroidal positional-encoder
specification. The checked-in formalism itself limits the current proof to the
separate Picard reduction and 12-sector torus quotient; it states that the
unified `Gamma_12` generators/action are not implemented. Database metadata
therefore remains `opaque` rather than overstating that evidence boundary.

## Distillation provenance

`model_source` identifies each contributing model/revision. `distillation_run`
records the admitted sources, method, configuration digest, and receipt digest.
Atoms and KV revisions may bind to one run. This records collective provenance;
it does not turn a model claim into evidence without a receipt.

## Nested KV contract

- `transformer_kv_session` binds an exact model/revision/request, tensor shape,
  H3 geometry contract, and Toroidal Phase Positional Encoder version.
- `transformer_kv_page` stores binary K and V tensors for one layer, KV head,
  and token interval. Its address and trajectory are links into the mind map,
  not semantic atoms.
- `transformer_kv_next` records causal page continuation.
- Tensor pages are a separate retrieval population: they are never embedded,
  distilled, or returned by semantic atom recall.
- Cross-model reuse fails closed unless model, tensor, address-contract, and TPE
  fingerprints match exactly.

The authoritative nested KV is a transformer attention cache: binary K and V
tensor pages are keyed by model/request, layer, KV head, and token interval,
then attached to the canonical H3 address, Toroidal Phase Positional Encoder
(T2 phase), and causal trajectory. RoPE and Euclidean positional encodings are
inadmissible. String values are forbidden in Surreal authority mode. The legacy
string `KvEntry` tools remain available only in explicit file-backed test mode
and fail closed in Surreal mode until the binary tensor transport is connected
to the inference runtime.

# Alternatives Considered

| Alternative | Decision | Reason |
| --- | --- | --- |
| Harness connects directly to SurrealDB | Rejected | Leaks credentials and bypasses receipts/policy. |
| Separate KV database/service | Rejected now | Creates two authorities and cross-store transactions. |
| Separate plugin binary plus MCP binary | Rejected | Duplicate implementations drift. |
| One plugin-managed MCP process | Selected | Matches the existing harness loader and keeps one runtime authority. |
| Embed KV as a recursive object on each atom | Rejected | Poor independent revisioning, indexing, and invalidation traversal. |

# Consequences

- Graph and KV revisions share transactions, provenance, backup, and tenant
  boundaries.
- The harness has one admitted capability surface and no database credentials.
- SurrealDB can traverse graph/KV relations natively while Rust retains policy.
- Native binary forward-pass transport still must be implemented and proven.
- The `r_12` quotient is stored honestly but is not computationally certified.

# Implementation Details

## Files Modified

| File | Lines | Change Type | Description |
| --- | --- | --- | --- |
| `HYPERBOLIC-KV-SURREAL-DESIGN.md` | 1-241 | new | Architecture and rollout contract. |
| `schema/0003_nested_hyperbolic_kv.surql` | 1-156 | new | Geometry and distillation provenance. |
| `schema/0004_transformer_kv_tensor_pages.surql` | 1-80 | new | Binary layer/head tensor cache and TPE identity. |
| `src/main.rs` | startup and Surreal gates | modified | Single authority, hydration, and string-KV rejection. |
| `src/surreal.rs` | authentication and hydration | modified | Root authentication and atom loading. |
| `README.md` | authority sections | modified | Operator-facing tensor and harness boundary. |
| `../research-grade-harness/README.md` | 48-68 | modified | Canonical one-binary plugin/MCP setup. |

## Languages Chosen and Why

| Language | Component | Rationale |
| --- | --- | --- |
| SurrealQL | Persistent schema | Native records, relations, constraints, and indexes. |
| Rust | Runtime policy | Existing HyperMind authority and harness binary. |
| MCP | Harness transport | Existing admitted tool and receipt boundary. |

## Grounding in Verifiable Literature

This implementation was not directly grounded in published literature. It
formalizes the user-specified topology and applies established graph database,
append-only revision, and provenance patterns. No claim is made that the opaque
`r_12` quotient has been mathematically implemented.

## Configuration Changes

No defaults change. Existing `HYPERMIND_SURREAL_*` variables select the endpoint,
namespace, database, authentication, and shadow/on mode.

# Testing Evidence

## Test Files and Locations

| Test File | Test Type | Coverage |
| --- | --- | --- |
| `schema/0004_transformer_kv_tensor_pages.surql` | live migration | Binary tensor/session/trajectory schema and TPE identity. |
| Live MCP startup | integration | Hydrates 2,079 atoms with no `.hypermind` side store. |
| Legacy KV call | adversarial | String-KV write is rejected in Surreal mode. |
| Shadow startup | adversarial | Split-authority mode is rejected without creating `.hypermind`. |

## What the Tests Showed

The authenticated live migration completed on SurrealDB 3.3.5. `INFO FOR TABLE`
confirmed binary tensor fields plus `toroidal_phase_encoder_version` and
`address_contract_digest`. MCP hydrated all 2,079 atoms without creating a file
store. String-KV and shadow-mode adversarial checks failed closed. Migration
digest `31c467e...f1246` was recorded. Native forward-pass tensor I/O is not yet
implemented, so these checks do not certify live attention-cache reuse.

## Test Command to Reproduce

Apply migration `0004`, inspect both tensor tables, then launch `hypermind-mcp`
with `HYPERMIND_SURREAL=on`; call health and an attempted legacy KV write while
asserting that no `.hypermind` path appears.

## Was It Proven Necessary?

Yes at the schema and authority boundary: the previous cache stored strings and
startup created a file authority before Surreal hydration. Forward-pass binary
transport remains unproven.

# Deployment / Rollout

| Aspect | Detail |
| --- | --- |
| Rollout strategy | Surreal-only authority; add native tensor transport behind fail-closed admission. |
| Rollback plan | Revert migration/code from source backup; do not revive a parallel file authority. |
| Monitoring | Compare tensor digests, model/TPE/address fingerprints, hits, and trajectory receipts. |
| Post-deploy validation | Real harness plugin run against persistent SurrealKV after restart. |

# Next Stage of Implementation

| Next Step | Description | Depends On This? |
| --- | --- | --- |
| Native tensor API | Transfer K/V bytes outside JSON/MCP into the runtime hot path. | Yes |
| Exact-match gate | Require model, layer/head shape, TPE, address, and tensor digests. | Yes |
| Restart/latency gate | Prove durable reuse, output parity, and forward-pass speedup. | Yes |
| Harness certification | Bind exact HyperMind receipt digests into harness evidence. | Yes |
| `r_12` definition | Specify executable action, representative, invariants, and tests. | No |

This implementation sets up the persistent graph/KV architecture, but it does
not by itself certify the runtime or the research harness.

# What We Learned

| Category | Learning |
| --- | --- |
| Technical | Tensor pages belong beside the graph but in a non-semantic retrieval plane. |
| Architecture | Plugin lifecycle plus MCP transport preserves one executable authority. |
| Evidence | A named quotient is metadata until executable invariants exist. |
| Rollout | Single authority prevents silent blending and split-brain recovery. |
