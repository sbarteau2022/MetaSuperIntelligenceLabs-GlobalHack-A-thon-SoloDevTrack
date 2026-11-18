---
title: GlobalMind multimodel distillation and symplectic boundary handoff
document_type: implementation handoff
date: 2026-11-02
status: proposed plan; current receipts distinguished below
workspace: /Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack
---

# Decision and system boundary

There is one persistent GlobalMind in SurrealDB, with shared actionable atoms,
typed trajectories, local charts, and nested exact-model tensor K/V. Qdrant is
the research/fact library and HyperDX is telemetry. Keep `hypermind/core`
quarantined. All implementation and receipts belong in this repository; do not
create another `.hypermind` or a second mind authority.

**User correction:** the intended model I/O boundary has no tokenizer. A
bidirectional **Boundary Symplectic Transducer** converts semantic input into
typed symplectic MLIR and converts validated output back. Within that boundary
use native typed/binary representations, not Python or JSON. “As close as
possible to machine native” means preserving typed operations, values,
coordinates, momentum/phase and effects through dispatch; it does not mean
claiming natural language has a unique machine representation. A legacy model
may still require its native internal IDs/embeddings; that adapter is
model-bound and outside the shared atom identity. The transducer is the
converter/decoder in both directions, with one versioned round-trip contract.

**Retirement:** the backend `QwenVlTokenizer` module/test and dependency from
commit `2ead001` were removed in this handoff pass. Its receipt is historical.
The Qwen manifest and `globalmind-distill trace` still bind/use a tokenizer
and a `GMTT` token tape. These are historical/integration paths, **not** the
new boundary. Replace their function only with a typed transducer ingress and
real model adapter. The Qdrant dense encoder must
be revisited against the new transducer contract; old Qwen vectors remain
versioned legacy data until re-encoded or served through an admitted adapter.

# What exists, and what it proves

| Area | Current evidence | Limit |
| --- | --- | --- |
| Distillation CLI | `globalmind-runtime/src/bin/globalmind_distill.rs` has `verify`, `trace`, `extract`, `judge`, `open`, `replay`, `crystallize`, `audit`; binary artifacts and atomic Surreal admission exist. | `trace` is specific to Qwen3.6 Layer 3 and a token tape. The integration report's “DistillModel” is a stub fixture. |
| Atom map | Rank-0 atoms, rank-1 transitions, rank-2 loops, rank-8 chart types and immutable origins exist in `globalmind-runtime/src/model.rs`. | Current atom key incorporates tensor shape/layer/head/payload; hash placement is not semantic geometry; content can be only `replayed atom <digest>`. See `MODEL-AGNOSTIC-ATOM-DESIGN.md`. |
| Tensor K/V | `results/tensor-kv/tensor-kv-harness-bridge-report.txt`: real Layer-3 continuation consumed recalled atom and both persisted K/V heads with exact hidden/logit parity after isolated Surreal restart. | One layer and one model; K/V is exact-model state, never shared knowledge. |
| Transducer | `hypermind-nested-kv/src/boundary_lexical_transducer.rs` has byte-carrier ingress, checked expansion and strict UTF-8 egress; `PerfectBoundarySymplecticLexicalTransducer.md` describes drop handling. | This is **lexical carrier folding**, not semantic-to-symplectic-MLIR conversion. `SYMPLECTIC-ENVELOPE-SPEC.md` is a proposal, not proof of a symplectic map. |
| Resource policy | `research-grade-harness/src/active_inference.rs` has a Q32.31 bounded selector and component tests. | Not wired to metered live execution; no measured thermodynamic free-energy/Landauer result. |
| Scheduler | `Ares WebSite : App/backend/hypermind-toy` tests workflow and backend-minted receipt-chain parity. | Rehearsal, not full live execution. |

Read `results/GLOBALMIND-OVERALL-AUDIT-2026-11-02.md` before reusing any older “PASS” label. Do not promote the original Queue 2 benchmark or claim a model-neutral mind from current Layer-3 fixtures.

# Boundary Symplectic Transducer contract to implement

1. **Ingress:** accept bounded semantic text, multimodal observations or typed
   tool events, preserve source bytes/format and provenance, parse into a
   typed semantic graph, then lower to a versioned symplectic MLIR module.
   Every operation has an SSA ID, type, operands, effects, source span,
   position/address `(H3,T2)` and a separately typed momentum/cotangent
   component. Reject ambiguous or unrepresentable input with a receipt; never
   silently drop carriers. Preserve language and script without special-case
   exclusion.
2. **Validation:** type/arity/dominance/effect checks, coordinate-domain and
   finite-value checks, explicit symplectic-form residual test for any claim
   of canonical transport, budget and provenance checks, and an immutable
   digest over transducer version + schema + input + lowered module. A
   hyperbolic coordinate plus phase is not automatically a symplectic
   structure; define the actual two-form and prove nondegeneracy/preservation
   or call the representation phase-augmented rather than symplectic.
3. **Model adapter:** translate typed MLIR to the frozen model's native input
   representation only at the last model-specific seam. Bind adapter digest,
   weights/config digest, shape and execution contract. Never put these
   model-bound bytes in the shared semantic key. If a frozen model cannot
   consume typed IR directly, disclose and test the adapter rather than
   claiming tokenless end-to-end execution.
4. **Egress:** decode model-native output to typed operations and evidence,
   check every carrier/SSA result, validate effects, then render semantic
   output. Require strict UTF-8 where text is emitted. Record unknown IDs,
   failed inverse mappings and loss; round-trip tests must cover multilingual
   text, code, tool calls, and malformed input. Output that cannot be mapped
   back is an explicit failure, not a plausible string.
5. **Versioning:** freeze ingress/egress schemas, semantic canonicalizer,
   MLIR dialect, geometry/projector, Q32.31 numeric contract, and adapter
   independently. A version change requires a new address revision and
   replay receipt, not an in-place rewrite.

# How to distill additional frozen models

This is the target production procedure. The current CLI can exercise its
Layer-3 integration slice; it cannot yet run this procedure for arbitrary
models.

1. **Register and freeze each teacher.** In a new admitted manifest list
   provider, revision, modality, architecture, weights/config digests,
   adapter and transducer digests, evidence corpus digest, TPPE/address
   contract. Rehash before database access. Never put teacher ID into shared
   semantic atom identity or ranking. Keep the model-specific witness link.
2. **Acquire real source behavior.** Run the frozen teacher on a preregistered
   held-out task set through the transducer adapter. Collect inputs,
   predictions, uncertainty/error, decisions, tool actions, observations,
   outputs and exact model trace bytes. A weight slice alone is not a fact.
   Record rank-0 typed proposition/procedure/constraint candidates; rank-1
   precondition/action/effect transitions; rank-2 verified closed loops;
   rank-8 *measured* local Jacobian/effect charts, only when marginal gain and
   numerical-stability receipts support them. Do not label eight sampled
   weights a Jacobian.
3. **Create shared candidates.** Canonicalize semantic payload and scope in
   deterministic native bytes. Stable key:
   `H(domain || canonicalizer_version || canonical_payload || scope)`.
   Exclude teacher, run, tensor shape, head, layer, tokenizer and raw weights.
   Exact canonical matches share one atom with append-only origins; judged
   paraphrases use an `equivalent_to` edge until equivalence is proven.
   Contradictions stay separate with `contradicts` edges.
4. **Judge independently.** A blind judge checks truth/entailment against
   cited Qdrant facts or observed action outcomes, scope, contradiction,
   replayability and abstention. Structural finite/mean-square checks alone
   cannot certify knowledge. Reject unsupported candidates before admission.
5. **Replay, crystallize, audit.** Execute the typed trajectory against the
   frozen adapter and compare predicted versus observed effects. Admit packets,
   shared atoms, origins, typed edges, charts and trajectory in one Surreal
   transaction; restart and reconcile counts/digests. Existing binary
   `.gmtrace → .gmcandidate → .gmverdict → .gmwalk` artifacts are reusable
   once the trace producer and semantic admission are upgraded. Keep the old
   `hypermind/core` untouched.
6. **Qualify cross-model reuse.** Test the same task with different teachers,
   architectures and tensor shapes. Shared semantic recall should reach one
   canonical atom or judged equivalent neighbor, while exact K/V reuse must
   MISS unless model, tensor shape, TPPE/address contracts and digests match.
   Run held-out task utility, citation, abstention, false-merge/split,
   latency, energy/resource and no-map baselines before calling it model
   agnostic. Any adverse result stays in the receipt set.

**Existing CLI sequence for the narrow Qwen Layer-3 integration lane only:**

```sh
cd '/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime'
cargo run --release --bin globalmind-distill -- verify manifests/qwen36-layer3-candidate-v2.manifest
cargo run --release --bin globalmind-distill -- trace <admitted-manifest> <existing-GMTT-token-tape> <run.gmtrace>
cargo run --release --bin globalmind-distill -- extract <run.gmtrace> <run.gmcandidate>
cargo run --release --bin globalmind-distill -- judge <run.gmcandidate> <run.gmverdict>
cargo run --release --bin globalmind-distill -- replay <run.gmtrace> <run.gmverdict> <run.gmwalk>
# Review generated run-specific admitted manifest and use only disposable SurrealDB.
cargo run --release --bin globalmind-distill -- open <run-specific-admitted-manifest>
cargo run --release --bin globalmind-distill -- crystallize <run.gmwalk>
# Restart the disposable SurrealDB, then audit the exact run.
cargo run --release --bin globalmind-distill -- audit <run-id>
```

These commands describe the existing experimental CLI, **not** a green light
to use its token tape for the new tokenless boundary or to write live
SurrealDB. `replay` currently hardcodes `LinearLogV1`; do not imply a selected
TPPE winner. A new model needs a real trace producer and frozen adapter before
this sequence can be used honestly.

# Turning atoms into actionable intelligence without fine-tuning

The frozen model weights stay fixed. On a task, the transducer lowers the
request; the mind retrieves a small typed neighborhood using semantic
identity, versioned geometry and typed edges; a capability gate carves a
permitted view. A controller chooses an action/trajectory within Q32.31
quality/resource limits, cites the supporting fact records in Qdrant, and
uses exact-model K/V only if its full identity matches. The executor runs the
typed action, observes the effect, judges it, and writes a new origin or
trajectory only after replay and atomic admission. A miss or contradiction
causes abstention, expansion or a fresh model call. This is inference-time
retrieval and constrained execution, not weight update. It may improve task
utility; the improvement must be measured against the same frozen model with
the map disabled. “Larger model knowledge in one mind,” model swapping and
quadratic compounding remain hypotheses until that comparison passes.

Each **actionable atom** needs a kind (`proposition`, `procedure`,
`constraint`, `tool_capability`, `observation`), canonical subject/predicate/
object or precondition/action/effect, scope and validity, cited evidence,
confidence/calibration, abstain condition, immutable origin set and typed
edges. A tensor witness or address alone is not actionable. Each trajectory
step needs input state, selected edge, action digest, predicted effect,
observed effect, judge receipt and replay receipt. Use `supports`,
`contradicts`, `derived_from`, `references`, `part_of`, `causal`, `temporal`,
`next`, and a separately judged `equivalent_to` edge; add precondition and
effect relation types before automatic execution.

# Immediate work queue and acceptance gates

1. Freeze the transducer dialect/two-form and reversible ingress/egress
   contract in design, with CAS/numerical receipts for any geometric claim.
   First test is real multilingual/code/tool round-trip with zero silent loss.
2. Replace GMTT-dependent production assumptions with an explicit model
   adapter behind the transducer. The backend tokenizer module is retired;
   preserve its historical receipt but never use it as a live gate. Compile,
   adversarially test, update READMEs and commit the adapter separately.
3. Implement native semantic payload/canonical key, stable cross-run atom ID,
   append-only origins and `equivalent_to`/contradiction admission in isolated
   SurrealDB. Test competing writers, restart and provenance mutation.
4. Implement two real teacher adapters and trace producers, then rerun
   extract/judge/replay/crystallize/audit on the same held-out tasks. Require
   cross-model semantic recall and exact-model K/V misses on swaps.
5. Wire the recalled typed atom/trajectory into the live scheduler and full
   model forward smoke; retain the existing Layer-3 exact K/V receipt as a
   component gate. Compare against frozen-model/no-map baseline; measure
   answer/action utility, latency, resource cost and abstention.
6. Only after those gates, move dense Qdrant ingestion/query onto the admitted
   transducer/model adapter representation. Maintain source provenance and
   versioned legacy vectors; PAMI must show a benefit beyond a rotated rank
   duplicate on held-out retrieval. Do not delete the BM25 fact lane early.

## N3 capability stubs: explicit exclusions

Source inspection confirms the user's reported gap in `N3/Custom Capabilities/Custom Plugins/mcp-tools/`:

| Tool | What is present | Required live gate |
| --- | --- | --- |
| `carve-specialist` | `tool.rs` returns hard-coded domain basins, percentages, holonomy and endpoint strings. | Compute a bounded carved view from real map atoms/edges, verify included/excluded membership and geometry from receipts, expose and exercise the actual MCP round-trip. Do not treat canned holonomy as Gauss–Bonnet evidence. |
| `checkpoint-status` | `tool.rs` returns constants/defaults; `status_value()` is always `ok`, and the `/tmp` path is a string. | Read a real checkpoint through an approved typed boundary, verify digest/schema/chain and report missing or corrupt state as failure. No new `/tmp` source of authority. |
| `dream-consolidate` | `tool.rs` builds request defaults and `/v1/mind/dream` string. | Govern and execute a real consolidation call, validate observed chart/bridge outputs, require independent judge/replay before atomic map writes, and receipt the HTTP/MCP round-trip. |

None is a working live capability, and none may be counted as evidence that the
inference loop can carve, resume or consolidate. Do not patch these generated
wrappers to return more realistic constants; connect them to the one mind and
prove the real effect.

**Stop conditions:** no shared-atom promotion on tensor-hash identity alone;
no “symplectic” certification without an actual preserved nondegenerate form;
no model-agnostic claim from one teacher; no K/V cross-model reuse; no live
`hypermind/core` migration without isolated proof; no fine-tuning claim when
weights stayed fixed; no synthetic or partial-model receipt labeled as full
execution.
