---
title: "GlobalMind Typed Admission Runtime"
document_type: "implementation"
version: "0.1.0"
date: "2026-11-01T21:41:00Z"
implementation_date: ""
author: "Codex"
reviewers: []
status: "draft"
related_files:
  - path: "Cargo.toml"
    lines: "1-21"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-189"
    role: "config"
  - path: "src/lib.rs"
    lines: "1-28"
    role: "supporting"
  - path: "src/holding.rs"
    lines: "1-214"
    role: "primary"
  - path: "src/identity.rs"
    lines: "1-70"
    role: "supporting"
  - path: "src/model.rs"
    lines: "1-961"
    role: "primary"
  - path: "src/store.rs"
    lines: "1-1230"
    role: "primary"
  - path: "src/tppe.rs"
    lines: "1-228"
    role: "primary"
  - path: "src/bin/globalmind_contract.rs"
    lines: "1-15"
    role: "test"
  - path: "tests/tensor_kv_qualification.rs"
    lines: "1-2081"
    role: "test"
  - path: "../results/tensor-kv/tensor-kv-qualification-report.txt"
    lines: "1-90"
    role: "supporting"
related_issues: []
related_documents:
  - "../hypermind-nested-kv/GLOBALMIND-SURREAL-INTEGRATION.md"
  - "PRIOR-EVIDENCE-TRANSFER-MATRIX.md"
tags: ["globalmind", "rust", "surrealdb", "distillation", "trajectories"]
---

# Executive Summary

Glimmer native forward setup (2026-11-03):
`src/bin/glimmer_forward_preflight.rs` rehashes the frozen Glimmer roster,
checks its 52-layer architecture and pinned tokenizer/config, and refuses an
unpinned native executor. Mind-integrated mode also refuses the retired TPPE
reference contract. It performs
no inference or SurrealDB write. See `GLIMMER-FORWARD-SMOKE-DESIGN.md` and
`../results/glimmer-forward/preflight-2026-11-03.md`. The real Qwen Layer-3
SurrealKV bridge qualification remains a separate component baseline; it is
not Glimmer execution.
The optional `--native-baseline` preflight mode preserves Glimmer's trained
RoPE/NoPE layers and skips only the mind-address TPPE admission check; it
still requires a pinned Rust-native executable and performs no inference.
The 2026-11-03 follow-up pinned the existing local Glimmer executable,
passed that preflight, and ran one separate Metal-backed prompt to a
`Paris` prefill argmax. The exact executable hash, command, and limits are
recorded in `../results/glimmer-forward/native-baseline-preflight-2026-11-03.md`.
That local baseline does not connect the model to the mind or expose tensor K/V.
The later local Glimmer run generated four tokens through all 52 layers,
ending `The capital of France is Paris. It is`; it still used the quarantined
geometric cache and supplied no exact K/V restore. The exact-cache path in
this runtime now refuses pages whose token interval or head dimension differs
from their shape/session and reassembles bounded contiguous dense K/V spans
only when every page has the same model, request, TPPE, geometry, layer/head,
dtype and layout contract. The real Qwen Layer-3 qualification splits one
head's frozen-weight K/V into two pages, restarts disposable SurrealDB,
reassembles them, and checks decode/logit bit parity plus gap/overlap/mixed
model rejection. It also requires both heads of that layer before injection,
checks the alternate head-dimension-major layout, and rejects a missing head
or mixed layouts. This is a Layer-3 proof, not Glimmer's full nested K/V pass.
TPPE at the Q/K boundary is an unpromoted ablation, while TPPE memory
addressing remains a separate integration gate.
The Qwen Layer-3 smoke now checks MCP boundary authority through the typed
GlobalMind bridge. Its live qualification now also starts the real
`globalmind-mcp` child, attests the same disposable Surreal authority, and
receipts the prefix atom through `mind.recall` before the bridge hydrates
exact K/V and injects both heads into the real Layer-3 decode. MCP carries
recall metadata; tensor bytes use the typed bridge, so no text payload is
promoted into the model I/O boundary.
`globalmind-mcp` now reports `On` only after verifying the immutable atom,
address, and exact transformer K/V fields that its read tools need. Full
distillation schema verification remains a separate write admission gate.
An isolated SurrealKV integration test spawns the real stdio child and checks
its authority and read contract. The backend's automaton route separately
attests that child before dispatch; model-facing typed hydration and a full
Glimmer task with recalled evidence are still open.
The same child now exposes `mind.recall_scoped`, which projects only persisted
cognitive-well members and edges internal to the visible atom set. Its
isolated MCP test includes a hidden atom, a cross-scope edge, missing
membership, and duplicate-well rejection. The caller must still supply a
governor-issued scope; the backend does not yet allow a carved worker to call
this tool. See `CARVED-MCP-RECALL-DESIGN.md`.

Queue 1.2 update (2026-11-02): the current position ablation uses a sequential,
token-independent TPPE carry and applies each arm at the same Layer-3 Q/K
boundary. `TppeState::validate` now checks cross-coordinate consistency,
including derived revolutions, Poincare radius, boundary margin and hyperboloid
lift; forged coordinate states fail closed. The radial candidates now share
the same start and one-derived-revolution endpoint by scaling `log_v1` at that
preregistered horizon; their intermediate radii remain distinct. This is component and partial-model
evidence only. The three-token tape is a smoke and cannot select or bench a
radial law; held-out data and full-model comparison
remain open. The source-and-receipt scope is in
`../results/GLOBALMIND-OVERALL-AUDIT-2026-11-02.md`.
The qualifier prints its one-turn budget and versioned identity. A long tape
still yields component-only output because this runner cannot establish corpus
provenance or complete-model quality; it never certifies promotion.
The high-radius Rust probe now requires strict increase in both hyperbolic
distance and represented Poincare radius; a rounded-equal stored radius fails
closed. Both candidate laws rejected at the represented boundary in the
bounded component probe (`../results/queue2/tppe-high-radius-boundary-2026-11-02.txt`).
The existing 384-token tape has recorded provenance in
`../CHECKPOINT-2026-11-02-QUEUE1.2-ROLLBACK.md` and verified file digests; its
source is this project's C13 design document. It remains a component tape,
not an independent held-out task benchmark. The current crystallization path
is one Surreal transaction and passed the isolated poisoned-bundle rollback
integration test; older per-entity receipts describe superseded code.
For the isolated Queue 1.2 redo, `contracts/tppe-layer3-candidate-v2.contract`
binds the current TPPE and Layer-3 trace source digests and numeric candidate
configuration. `manifests/qwen36-layer3-candidate-v2.manifest` binds its SHA-256.
The trace producer and positional comparator refuse the historical
`tppe-reference-v1` manifest or a stale source/contract digest. This identity
is explicitly a Layer-3 candidate, not full-model encoder qualification.
The candidate run passed trace, extract, judge, replay, atomic crystallization,
and isolated SurrealDB restart audit (`../results/queue1-layer3-candidate-v2-receipt.txt`).
It is an integration fixture. The complete-model reference and independent
held-out task comparison remain open; no encoder has been promoted.
The Q1.7 forward smoke now resolves the candidate manifest instead of the
historical reference manifest; its five focused library tests pass. Q1.5
tensor K/V requalification now uses the candidate-v2 manifest, contract,
configuration, and pure toroidal Q/K operation. The optimized hardware test
passed all seven exact-model Layer-3 gates, including isolated SurrealDB restart,
byte equality, injected decode parity, and identity-mutation rejection. The
current receipt is `../results/tensor-kv/tensor-kv-qualification-report.txt`;
`../results/q15_status.md` records the earlier debug-profile hash stall and
its resolution. This does not establish full-model parity or cross-model K/V
reuse. Run this hardware gate with `cargo test --release --test
tensor_kv_qualification -- --nocapture --test-threads=1`; the debug profile
rehashes the full model roster slowly before loading weights.

This fresh Rust library provides typed GlobalMind identities, distillation
packets, replayable trajectories, prediction-error observations, and atomic
SurrealDB writes. It defaults to an isolated database and refuses the
quarantined `hypermind/core` database unless an explicit override is present.

The model-agnostic atom admission gap and its proposed correction are specified
in `MODEL-AGNOSTIC-ATOM-DESIGN.md`. Current hash-derived, run-scoped Layer-3 atoms
are trace fixtures and do not yet carry independently verified shared knowledge.

`GlobalMindStore::fetch_tensor_payload_exact` now performs a read-only lookup
over the full model/request/prefix/TPPE/session/page identity before exposing
model-bound K/V bytes. It checks expected key/value digests again after fetch.
The candidate-v2 hardware qualification exercises the new lookup with a real
stored baseline and 20 single-field identity misses after SurrealDB restart.
The research harness exposes this through `GlobalMindBridge::fetch_tensor_exact`;
full-model and deployed harness forward-pass gates remain open.

# Problem Statement

The SurrealDB schema had no fresh typed runtime writer. The legacy MCP ingest
surface accepts text-shaped requests and cannot establish the packet,
trajectory, replay, and prediction-error invariants required by GlobalMind.

# Solution Overview

`globalmind-runtime` validates digest and record identity types before building
transactions. It writes distillation packets, crystallized atoms with their
address/origin/well proof, whole trajectory headers and ordered steps,
prediction-error observations, and superposition hold/branch/iteration/
resolution records through one policy boundary. Every admitted schema field is
checked as immutable before operation.

Trajectory admission is now an atomic cell bundle. It validates admitted
rank-1 packets before creating graph edges and steps, admitted rank-2 packets
before creating closed-loop receipts, audit origins, TPPE radial continuity,
and positive boundary margins. Rank-8 local charts accept native bytes only
after shape, dtype, digest, packet, run-phase, and numerical/marginal-gain
receipt checks. Append-only run events enforce legal phase transitions.

The native `globalmind-distill` ingress parses a typed line manifest, rehashes
every frozen artifact before any database access, binds the run to an exact
TPPE/TPPE configuration, and opens only immutable source/admission/run receipts.
Missing eligibility or independent-judge receipts fail before the transaction.
This is roster ingress, not model distillation or candidate extraction.

`tppe.rs` implements both preregistered radial candidates without selecting a
winner. The expanding law acts on hyperbolic distance `u`; the stored Poincare
radius is `tanh(u/2)`. Boundary saturation, non-finite state, radius regression,
and Lorentz-sheet failure are rejected rather than clamped. These are component
invariants only; real-model quality and resource bakeoffs remain mandatory.

# Implementation Details

| File | Lines | Change Type | Description |
| --- | --- | --- | --- |
| `Cargo.toml` | 1-21 | new | Fresh crate manifest, verifier, and ingress binaries. |
| `Cargo.lock` | 1-189 | new | Reproducible dependency resolution. |
| `src/identity.rs` | 1-70 | new | Validated SHA-256 and Surreal record identifiers. |
| `src/holding.rs` | 1-214 | new | Bounded zero-emission hold, suppress, collapse/iterate/fail-closed controller. |
| `src/model.rs` | 1-961 | new | Typed atom, packet, atomic cell/TPPE trajectory, chart, lifecycle, holding, and tensor contracts. |
| `src/store.rs` | 1-1488 | new | Authenticated Surreal client and atomic admission/cell/lifecycle/Whitehead-topology transactions. |
| `src/tppe.rs` | 1-228 | new | Checked T2 phase recurrence and both preregistered hyperbolic radial candidates. |
| `src/whitehead.rs` | 1-1530 | new | Executable Whitehead 12-around-1 index-12 subgroup $\Gamma_{12} = \Gamma_0(2+2i) \subset \mathrm{PSL}(2,\mathbb{Z}[i])$, $(H^3 \times T^2)/\Gamma_{12}$ quotient action, central hub, 12 icosahedral peer wells, canonical representative reduction, and Voronoi boundary/overlap classifier. |
| `src/lib.rs` | 1-47 | new | Public library surface. |
| `src/bin/globalmind_contract.rs` | 1-15 | new | Read-only live contract verifier. |
| `src/manifest.rs` | 1-416 | new | Typed frozen-artifact roster, SHA-256 rehash, and deterministic ingress identity. |
| `src/bin/globalmind_distill.rs` | 1-41 | new | Verify-only and fail-closed isolated ingress commands. |
| `contracts/tpe-reference-v1.contract` | 1-17 | new | TPPE spiral-out reference binding; explicitly not runtime qualification. |
| `contracts/whitehead-gamma12-v1.contract` | 1-31 | new | Immutable contract binding `geometry_space:h3_t2_r12_v1`, `cognitive_hub:l0_coordinator`, and `cognitive_well:well_00..11`. |
| `manifests/qwen36-opus-candidate.manifest` | 1-56 | new | Real frozen-model candidate roster; intentionally lacks admission receipts. |
| `tests/tensor_kv_qualification.rs` | 1-2081 | new | Lane B real-model tensor K/V qualification across isolated SurrealDB 3.3.5 restart and logit parity. |
| `tests/whitehead_12_around_1.rs` | 1-787 | new | Lane D 7-gate executable verification of $\Gamma_{12}$, $H^3 \times T^2$ coupling, canonical reduction, Voronoi partition, and isolated SurrealDB 3.3.5 admission. |
| `../results/tensor-kv/tensor-kv-qualification-report.txt` | 1-90 | new | Measured hardware, digest, parity, and identity-miss receipts for Lane B. |
| `../results/whitehead/whitehead-12-around-1-report.txt` | 1-157 | new | Measured subgroup, generator, invariance, partition, and SurrealDB 3.3.5 receipts for Lane D. |
| `src/trace/mod.rs` + `src/trace/codec.rs` | 188 + 359 | new | Lane 1 fresh binary trace envelope (`GMTR` v1): typed kind/dtype, shape/layer/head/position, native payload bytes, payload SHA-256, observer-nonperturbation receipt digest, teacher-neutral content digest, strict `decode` and walking `decode_frame`. |
| `../results/trace/trace-envelope-report.txt` | 1-67 | new | Lane 1 end-of-lane report: source/toolchain/hardware digests, lib-scoped gate receipts, admit/reject counts, and untouched gates. |
| `src/capability/vertical_slice.rs` | 1-1554 | new | Minimum Capability Atlas vertical slice (13 profiles, 17 edges) plus 15 audit tests: 2-hop planning, registry reconciliation, fallback, DAG acyclicity, offline policy gate, context projection, restart replay, real read-only harness-bridge call, and live SurrealDB 0012 schema audit. |
| `../results/capability/vertical-slice-audit-report.txt` | 1-93 | new | Slice audit report: per-gate results, digests, commands, and untouched gates. |
| `src/capability/forward_smoke.rs` | 1-431 | new | Forward-pass smoke on the audited slice: manifest presence binding, dead-endpoint fail-closed probe, offline mandatory gate, 8-class identity-mismatch matrix, registry-less execution refusal, and receipted read-only execution. |
| `../results/forward-pass-smoke/forward-pass-smoke-report.txt` | 1-98 | new | Smoke receipts: verbatim observed values, fail-closed results, and untouched gates. |
| `src/extract/mod.rs` | 1-410 | new | Lane 3 candidate extractor: teacher-neutral `atom_key` (position/teacher excluded), rank-0 coalescence, rank-1 walk transitions, rank-2 loop closure, rank-8 weight charts; K/V traces excluded from semantic extraction. Report: `results/extract/REPORT.txt`. |
| `src/judge/mod.rs` | 1-230 | new | Lane 4 independent judge: frozen policy at construction, no mutation/tuning API, immutable pass/reject receipts binding candidate, evidence, novelty, and policy digests. Report: `results/judge/REPORT.txt`. |
| `src/bin/globalmind_tppe_qualify.rs` | current | corrected | Matched real Q/K-boundary positional ablation using actual 248,320-vocabulary next-token NLL, real wall time, logit deltas, rotated arm order, and no fabricated zero-cost control. The three-token smoke does not promote or bench an encoder. |
| `tests/surreal_integration.rs` | 1-620 | new | Lane 5 Surreal integration test: isolated file-backed SurrealDB 3.3.5, schema 0001+0003..0011, one judged trajectory admitted, duplicate write proven atomic, restart read/replay verified. Report: `results/surreal/REPORT.txt`. |
| `results/tensor-kv/REPORT.txt` | current | qualified | Candidate-v2 exact-model Layer-3 hardware gates passed; scope and timing recorded. |
| `results/harness-plugin/REPORT.txt` | 1-25 | gated | Lane 7 not started: gate requires lane-6 qualification receipt, which does not yet exist. No `research-grade-harness` file edited. |
| `../hypermind-nested-kv/schema/0012_capability_atlas.surql` | 1-196 | new | Capability Atlas schema enforcing all 12 atom kinds, 13 directional edge kinds, 1-to-1 `atom` binding, and `READONLY` fields. |
| `src/capability/mod.rs` + `src/capability/store.rs` + `src/capability/vertical_slice.rs` | 1150 + 380 + 420 | new | Model-neutral Capability Atlas scaffold (`GMCS`/`GMCP`/`GMCE`/`GMCT` binary envelopes), 8-step two-hop planner, `LiveRegistry` reconciler, `MandatoryPolicyGate`, and vertical slice. |
| `tests/capability_atlas_audit.rs` | 1-740 | new | 3-gate Capability Atlas audit against isolated SurrealDB 3.3.5 (12 kinds, 13 edges, 2-hop retrieval, live registry reconciliation, offline mandatory rules, restart replay, version supersede, and read-only tool call). |
| `../results/capability-atlas/capability-atlas-qualification-report.txt` | 1-118 | new | Measured schema, retrieval, DAG, restart, and execution receipts for the Capability Atlas contract. |
| `src/replay/mod.rs` + `src/replay/artifact.rs` | 1-1590 + 1-956 | new | Stage-5 replay walker: ordered rank-1 walk with real TPPE stepping (exact T2 continuity, radial expansion), rank-2 closure proofs, rank-0 reconstruction, rank-8 chart validation, identity-miss bindings, `GMRW` v1 artifact codec, run-scoped ids. |
| `tests/distillation_surreal_live.rs` | 1-555 | new | Integration: extract->judge->replay->artifact->ingress->packets->atoms->trajectory on isolated SurrealDB 3.3.5, duplicate-write atomicity, process restart, persisted-row re-verification, completed event, exact reconciliation. |
| `src/bin/globalmind_distill.rs` (replay/crystallize/audit) | isolated patch | new | Stage CLI branches: replay (rehash, in-process extract+judge, walk emit, zero writes), crystallize (binding check, replay re-verify, ordered-atomic admit, readback), audit (per-id reconciliation, terminal event, plain-text report). |
| `../results/distillation/distillation-integration-report.txt` | 1-13 | new | Integration test receipts: 5/5 pass, 2 atoms, 2 steps, 1 loop, restart-identical, balance=PASS. |
| `../results/distillation/cli-e2e-audit-report.txt` | 1-14 | new | CLI E2E audit after real server restart: per-entity counts balance, balance=PASS. |
| `../results/distillation/replay-crystallize-audit-report.txt` | 1-127 | new | Stage report: design, honesty notes, observed evidence, digests, open gates. |
| `src/retrieval/mod.rs` | 1-1103 | new | Queue 2 read path: isolated-DB atom neighborhood retrieval (M-1 cost d_H3 + 0.25*d_T2^2 via Whitehead primitives, flat Euclidean control), budget/quarantine fail-closed, deterministic receipted ranking, `GMRC` v1 binary recall bundle. |
| `../results/retrieval/minimal-read-path-report.txt` | 1-55 | new | Retrieval receipts: 6/6 tests, live order/determinism proofs, allocator note, open gates. |
| `../results/queue2/queue2-ablation-report.txt` | 1-61 | new | Queue 2 execution: G0 real-forward verification, Q2.4 INVALIDATED_BY_AUDIT (v1 bench void; corrected v2 smoke decision-open, held-out rerun required), Q2.3 retrieval slice, fail-explained dispositions for the rest. |
| `src/retrieval/mod.rs` (scale probe) | +107 | status | 200-atom dual-metric probe: top-10 overlap 5/10, ~27/~25 ms per retrieval, rerun-identical digests. |

Rust is used for strong types and fail-closed validation. SurrealQL supplies
transactions, schema assertions, relations, and immutable record fields.
JSON appears only inside the private SurrealDB HTTP adapter; it is not exposed
as a model, tensor, or harness protocol.

This implementation applies established typed-boundary and transactional
storage patterns. The Active Inference terminology follows Friston et al.,
“Active Inference: A Process Theory,” Neural Computation 2017,
DOI `10.1162/NECO_a_00912`.

Before adding runtime machinery, use
[`PRIOR-EVIDENCE-TRANSFER-MATRIX.md`](PRIOR-EVIDENCE-TRANSFER-MATRIX.md). It
puts current Rust-server tests beside Monster’s historical evidence so cache,
wire, toroidal-control, and failure work is not duplicated. Monster/CoreLM code
remains excluded; its failed 0/24 language gate is a bench decision, not a
starting implementation.

# Configuration Changes

The runtime reads `GLOBALMIND_SURREAL_URL`, `GLOBALMIND_SURREAL_NS`,
`GLOBALMIND_SURREAL_DB`, `GLOBALMIND_SURREAL_USER`, and
`GLOBALMIND_SURREAL_PASS`. `GLOBALMIND_ALLOW_QUARANTINED_CORE=1` is the explicit
override required to address the quarantined database.

# Testing Evidence

`cargo check --tests` and `cargo clippy --all-targets -- -D warnings` pass for
the fresh crate. The Surreal schema chain through `0010` also passes on
SurrealDB 3.3.5. The compiled `globalmind-contract` binary verified migrations
`0007` and `0008` against an isolated restore of the real database, which
retained all 2,079 atoms. Real-model distillation and live typed-write receipts
remain pending, so runtime certification is not claimed.

The release-mode verifier rehashed the real Qwen3.6 candidate roster and
derived input digest
`cf80d582fa1528b356586d71ca221c69af1a935e9e1159f8409e83b3992d4c08`
with `database_writes=0`. An isolated `open` then failed because the model has
no eligibility receipt; readback confirmed zero model sources, admissions,
runs, and run events. No atom or trajectory was minted.

SurrealDB 3.3.5 accepted the isolated schema chain through `0011`; the compiled
contract verifier accepted all required immutable fields. A parse-only,
non-executed branch accepted the exact fixed-ID `RELATE` form used for atomic
edges. Migration `0011` SHA-256 is
`dcd8d7820d88d6dd3d8c02565965d518a4dbe44dace7e1a81acf6a86efea006d`.
No real trajectory has yet exercised the write transaction, so integration
certification remains open.

Three mathematical component tests cover both expanding-radius candidates,
Poincare/Lorentz invariants, saturation rejection, and invalid inputs. They do
not count as the required real frozen-model bakeoff or integration proof.

Lane 1 native trace envelope: `cargo fmt --check`, `cargo check --lib`, and
`cargo clippy --lib -- -D warnings` pass; `cargo test --lib` passes 24/24
including 9/9 trace codec tests (roundtrip, 3x5 kind/dtype matrix, frame
walking, tamper/truncation/padding/shape rejection, teacher-neutral digest).
`cargo check --tests` passes crate-wide (lane 5 repaired their file after
the lane-1 report noted its syntax errors; that file remains untouched by
other lanes). No real model trace ingested; component evidence only, not
certification.

Capability Atlas vertical slice: `cargo test --lib` passes 40/40 including
16/16 capability tests. The live audit applies migrations 0001,0003..0012 to
an ephemeral isolated SurrealDB 3.3.5, verifies all 12 atom kinds and 13 edge
kinds as READONLY, rejects untyped/missing/duplicate/overwrite writes with
zero partial state, and completes a real read-only tool call through the
Atlas -> registry -> planner -> harness bridge with receipts. Full per-gate
results are in `../results/capability/vertical-slice-audit-report.txt`.
Broad Atlas population and harness plugin wiring remain open. The
forward-pass smoke ran on the unmodified 13-profile slice (`cargo test
--lib` 45/45): dead-endpoint verification fails closed, the mandatory gate
enforces with zero GlobalMind contact, all 8 identity-drift classes refuse,
registry-less execution never invokes the tool, and the healthy read-only
call completes with policy/response/execution/utility receipts. No model
inference is claimed; the native trace CLI is still unimplemented.

Lane B real tensor K/V qualification (`cargo test --release --test tensor_kv_qualification -- --nocapture`)
passed all 7 required gates against the real frozen Qwen3.6-MoE-Combined
weights on disk (`model_roster_digest = 2341d9dfe7723e73e6f145b8a71446b4e2476a4b2a6f041c0af1380629a70cba`)
and an isolated SurrealDB 3.3.5 `surrealkv://` instance (never touching port `8000`
or `hypermind/core`). Actual layer 3 (`self_attn`, 2 KV heads, shape `[4, 256]`,
`f32`, `token_major`, `4,096` bytes per K/V tensor) pages survived a real
SurrealDB process stop/restart (`pid 56049 -> 56065`, `589.142 ms`), matched
byte-for-byte and by SHA-256 on readback (`12.040 ms` / `11.952 ms`), and
reproduced exact bit-for-bit decode output and `248,320`-vocabulary `lm_head`
logits (`max_abs_diff = 0.0`, logit SHA-256
`d9d8221492ba87cc18ba1d839f0aa52c700c803a4171194c71f57ea8fc93b5ac`, top-1
token `96156` logit `13.812788`). Single-field identity mutations missed
`20/20` times on lookup and failed closed `11/11` times on write/readonly
enforcement. Full receipts are recorded in
`../results/tensor-kv/tensor-kv-qualification-report.txt`.

Lane D Whitehead 12-around-1 formalization (`cargo test --test whitehead_12_around_1 -- --nocapture`)
passed all 7 required gates:
1. Central hub (`cognitive_hub:l0_coordinator` at $h_{\text{hub}} = (1, 0, 0, 0) \in H^3$, $\phi_{\text{hub}} = (0, 0) \in T^2$) and 12 peer well centers (`cognitive_well:well_00..11`) on the golden icosahedral shell $r_{\text{sph}} = 2 - \varphi = (3 - \sqrt{5})/2$ ($r_{H^3} \approx 0.7294228634$, $\cosh(r_{H^3}) = 1.2777777778$, Welch coherence $|\langle \mathbf{d}_r, \mathbf{d}_s \rangle| = 1/\sqrt{5}$, 30 adjacent pairs with $\Omega_{r,s} = 0.08$, 30 non-adjacent pairs with $\Omega_{r,s} = 0$, 6 antipodal pairs).
2. Explicit index-12 congruence subgroup $\Gamma_{12} = \Gamma_0(2+2i) \subset \mathrm{PSL}(2, \mathbb{Z}[i])$ stabilizing $[1 : 0]^T$ on the 12-element projective line $\mathbb{P}^1(\mathbb{Z}[i]/(2+2i))$, with 12 explicit left-coset representatives $C_0, \dots, C_{11}$, unique factorization $M = C_k \gamma$ across all 661 generated Picard elements of word length $\le 3$, and 20 deduplicated Schreier + Whitehead subgroup generators (`gamma12_generators()`).
3. Exact $H^3$ spinor congruence $X' = M X M^\dagger$ (verified identical $< 10^{-12}$ to Poincaré Upper Half-Space Möbius extension and Lorentz metric preservation) coupled to the sign-invariant $(1+i)$-parity character homomorphism $\chi: \Gamma_{12} \to T^2$ along the 2:1 Kuramoto resonant direction and left-coset holonomy $\psi_{\text{coupled}}(x, \phi) = (\psi(\phi_{\text{can}}) + 2\pi k(x)/12) \bmod 2\pi$.
4. Deterministic canonical representative reduction (`canonicalize`) and Voronoi well assignment (`classify_well`), invariant across $128 \times 20 = 2,560$ generator applications ($\max d_{H^3} = 3.2464 \times 10^{-14}$, $\max d_{T^2} = 0.0$, well ordinal matches $2560/2560$).
5. Exhaustive no-gap/no-overlap partition across $4,608$ deterministic $(H^3 \times T^2)$ samples ($384$ per well, zero unassigned gaps, zero multi-assigned points), exact boundary tie-breaking to `min(r, s)` on all 66 midpoints, and positive neighbor overlap $\Omega_{r,s} = 0.08$ strictly on the 30 icosahedral edges.
6. Isolated SurrealDB 3.3.5 admission of `geometry_space:h3_t2_r12_v1` (upgrading pre-seeded `quotient_status` from `'opaque'` to `'specified'` with contract digest `76b3f57fe3f1502edd63bc011a4de249a211ef154f6848c9795f5d7eb9cd8d30`), `cognitive_hub:l0_coordinator`, all 12 `cognitive_well` records (`status = 'specified'`), and all 12 `belongs_to_well` edges with fail-closed tamper detection. Full receipts are recorded in `../results/whitehead/whitehead-12-around-1-report.txt`.

Capability Atlas Contract qualification (`cargo test --test capability_atlas_audit -- --nocapture`)
passed all 3 audit gates against isolated SurrealDB 3.3.5 (migration `0012_capability_atlas.surql`
SHA-256 `0bac7e11dcf1e5dd0ec999904d152faa59bfd6de8481018b8a39b89879ad05de`):
1. Schema & fail-closed gate (`gate1`): verified all 12 atom kinds (`17` profiles) and all 13 typed edge kinds (`22` edges) in a single atomic scaffolding transaction (`bundle_digest = 54e3c1ff6461f8ff5e216ff894a99d7a2b7cc57f067778638d1f0ded6bae6040`, `replay_digest = 6f9c246c5f122194f4a38b556795dea4fa06977b7d24f45940e4e3103338e45c`), while rejecting untyped `'related'` edges, `READONLY` overwrites, duplicate `(workspace, atom)` attachments, inline secrets/credentials, and `transformer_kv_page` material.
2. Two-hop retrieval, live registry reconciliation & acyclic DAG gate (`gate2`): approximate retrieval landing on the `solution_family` neighborhood reached the qualified composition within $\le 2$ typed hops across all `3/3` task queries (`primary_dag_digest = 27046201f29e22af1ab59ee346617d33ad7f17ffeec23946eb79d7101a12aa6f`), excluded benched, superseded (`0.9.0`), and conflicting/unregistered tools before planning, promoted the evidence-backed fallback (`fallback_dag_digest = 5074371f15408cd22f62739b408e93b716bebba0d61093fd3895bfec22ec9cad`) when the primary tool had a live schema digest mismatch, proved `source_digest` invariance, verified an acyclic `ExecutionDag` despite an intentional `skill <-> hook` cycle in the Atlas, and projected only the `3` selected callable contracts into a binary `GMCT` context frame.
3. Offline mandatory rules, restart replay, version supersede & live execution gate (`gate3`): verified `MandatoryPolicyGate` blocks JSON/Python I/O and write effects while SurrealDB is stopped, proved identical candidate IDs, 2-hop neighborhood, DAG digest, and context frame digest (`b1bc28c7a49351443b106486080ab8739b1f7c18474779a25ab244d736e7dec1`) across a real SurrealDB restart (`restarted_pid = 74354`), completed a real read-only tool call through `Atlas -> LiveRegistry -> AtlasPlanner -> MandatoryPolicyGate -> Harness` (`policy_receipt = 8d9ef3192873c74d53c1fcf3dbc51ba2d49015d967a18d49af96e7ce9b84ae18`, `response_digest = d53f95c84cf84305b3026a3beab1e18a4e4262475784977e015ee6bb88ec5bc3`), appended `capability_utility_receipt` without coordinate mutation, and superseded `1.0.0 -> 1.1.0` (`v110_digest = c3c4fc6bff008d1eac01996e91afa9705b37c397ac0366d45a859dc855ce526e`) while preserving `1.0.0` in qualified history. Full receipts are recorded in `../results/capability-atlas/capability-atlas-qualification-report.txt`.

Stages 5-7 replay/crystallize/audit: full `cargo test` green (lib 52,
capability audit 3, distillation integration 1, lane-5 integration 1,
tensor_kv 1, whitehead 7; 0 failed). Replay verifies ordered walks with
real TPPE stepping, loop closure, atom reconstruction, and chart
validation; crystallize admits through ordered-atomic store calls with
readback; audit reconciles every packet/atom/step/loop/edge/chart/event
per id and appends the terminal event. CLI E2E on isolated SurrealDB
3.3.5 passed across a real server restart (balance=PASS, exit 0);
unknown laws, tampered/stale walks, and terminal reruns fail closed
(exit 1). Envelopes are constructed and the manifest artifact is a temp
stub (same level as the lane-5 walk); crystallize is ordered-atomic,
not single-transaction (store.rs is another lane's file). Evidence:
`../results/distillation/`. No certification claimed.

Queue 2 retrieval read path (`cargo test --lib` 58/58): `retrieve()`
ranks persisted atom neighborhoods on isolated SurrealDB (0001 tables)
with receipted deterministic order; `GMRC` v1 bundle crosses toward
model I/O in binary only. Step log (allocating-weights): fusion weight
H3 vs T2^2 set 1.0 -> 0.25 (M-1 cost form); probe
`t2_weight_probe_before_after_1p0_vs_0p25` executes both weights on the
fixed 5-point seed with identical ranking order under both; only the T2
term scale changes; quality-gated sweep stays open until ablation 1's
forward harness exists. Recall-into-forward injection and
forward-derived queries are not claimed. Evidence:
`../results/retrieval/minimal-read-path-report.txt`.

Queue 2 (`cargo test --lib` 64/64): G0 verified the real frozen
forward E2E (bit-exact nonperturbation, 9 envelopes, 37.2s wall).
Q2.4 v1 bench is INVALIDATED_BY_AUDIT and void: no encoder promoted,
no encoder benched. The corrected v2 runner measures real next-token
NLL at the same Layer-3 Q/K boundary; its three-token smoke is
decision-open and Q2.4 requires a complete held-out rerun. Q2.3 measured at retrieval level only
(5/10 top-10 overlap, no task quality). A1/A2/A5/A6/A7/A8/A9/A10
are fail-explained with named missing substrate (injection, hops,
TIT executor, second model/judge, controller). Evidence:
`../results/queue2/queue2-ablation-report.txt`.

# Deployment / Rollout

| Aspect | Detail |
| --- | --- |
| Deployment | Not deployed. |
| Strategy | Compile, apply schemas to a real-data clone, run typed write/read qualification, then connect the fresh distiller. |
| Rollback | Drop the isolated development database. |
| Monitoring | Transaction failures, identity mismatches, replay failures, and prediction-error resolutions. |
| Promotion | Requires real model distillation and real trajectory replay. |

# Next Stage of Implementation

Implement and qualify the Rust-native TPPE encoder, real frozen-weight
candidate extractor, independent judge, and replay walker. Only then may the
runner append packets, atoms, transitions, loops, charts, and crystallized
trajectories. Compiler success and roster ingress do not establish reasoning
quality.

# What We Learned

Trajectory admission must be atomic across its header and ordered steps. A
prediction-error observation needs separate raw, precision-weighted, surprisal,
and Bayesian-update fields to prevent incompatible quantities from being
silently compared.

The optional `live-forward-model` integration gate uses the research harness bridge to retrieve both exact-model prefix K/V heads from isolated SurrealKV and inject them into a real Qwen3.6 Layer-3 continuation. The 2026-11-02 release run matched the uninterrupted hidden output and all 248,320 logits bit for bit; see `../results/tensor-kv/tensor-kv-harness-bridge-report.txt`. The default Q1.5 qualification remains separate, and no full-model claim follows.

### Scoped Glimmer recall seam

The model-independent native execution contract is specified in
[UNIVERSAL-DECODER-DESIGN.md](UNIVERSAL-DECODER-DESIGN.md). It separates the
model-neutral harness/MCP ABI from an operator graph compiled from each
checkpoint and requires a second architecture conformance run. No universal
executor or full Glimmer forward is certified yet.

`src/universal_decoder.rs` is currently graph admission and digest code only.
It now rejects empty/oversized graphs, duplicate or malformed node IDs,
invalid dimensions, and duplicate tensor bindings. The v2 graph digest
includes explicit field lengths and shape boundaries; a regression test
distinguishes shape partitions that the former concatenation aliased. It
does not execute operators, attest shard bytes, or satisfy model I/O.

`retrieval::scoped::retrieve_scoped` reads persisted cognitive-well memberships and exposes only member atoms and active edges whose endpoints both survive the scoped top-k. It does not write SurrealDB. It rejects empty scopes, missing well memberships, malformed rows, and candidate-budget saturation. The caller must validate the governor permit, one Glimmer weight owner, and `McpSurrealTarget` against the bridge before calling it. This is a read-only component seam for a carved worker; the harness permit binding and live Glimmer forward proof remain open. See [SCOPED-RECALL-DESIGN.md](SCOPED-RECALL-DESIGN.md).
