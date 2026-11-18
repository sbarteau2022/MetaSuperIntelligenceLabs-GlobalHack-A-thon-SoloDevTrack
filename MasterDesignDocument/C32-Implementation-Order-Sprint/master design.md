# HyperMind + Nested KV — Master Design

**Subject.** HyperMind (hyperbolic mind-mapping memory space) + Nested KV (hyperbolic
phase-state cache). One unified memory substrate: the KV cache is not a separate dense
structure; it is nested inside the same hyperbolic Atlas as the long-term memory map.

**Status.** PROPOSED design scaffold (no code written in this pass). Every performance
number below is PROPOSED until an ablation hash is filed and a receipt is produced.
**Date.** 2026-10-27.
**Parent.** `MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack` design set
(C01–C35), especially C02 (Difference 1), C05 (Atlas/KV), C12 (server), C25 (holes),
C35 (interop).
**License intent.** Built for the people: local-first, open, plugin-agnostic, no mandatory
external API.

---

## 0. Claim labels used throughout

Per the repository documentation standards (§7), every claim below carries one of:

- **VERIFIED** — measured or read from source in this workspace.
- **REPORTED** — a source says so; unchecked here.
- **INFERRED** — reasoned from evidence; stated where the evidence ends.
- **PROPOSED** — a plan, not a fact.

An unlabeled claim is treated as unverified. Corrections are appended, never silently
rewritten (§7.3).

---

## 1. FullAudit of v1

v1 = the current prototype stack audited in this repository: the ad-hoc single-binary
app, the `live_brain.atoma_map` vault, the Ares lane/hook/tool/hole scaffold, and the
hyperbolic oracle/Metal kernels. Evidence citations are VERIFIED where the artifact exists
in-tree.

### 1.1 Audit table

| ID     | Surface (v1)           | Finding                                                                                                                                                                                                  | Evidence                                              | Label                          | Disposition                                                                                                                        |
| ------ | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| HN-A01 | Native vault geometry  | Parent–child hyperboloid closeness 0.62 (not tight); radius↔depth correlation −0.04; 38.3% of quads order the wrong way. Native map lacks a geometric depth encoding.                                    | `A40-RunReport-2026-10-27.md` §3 (A40-RUN-01 Stage 1) | VERIFIED                       | Native map is a retrieval/plugin substrate, not a hyperbolic-retraining baseline. Geometry-change claims WITHDRAWN for this vault. |
| HN-A02 | VP-tree retrieval      | `O(log N)` expected retrieval holds only under a bounded expansion constant `c`; adversarially clustered atoms degrade toward `O(N)`.                                                                    | C05.12                                                | INFERRED                       | Add clustered stress case to A1; label complexity INFERRED until measured.                                                         |
| HN-A03 | Eviction               | Victim = `argmax x₀` unpinned is geometric **only if** insertion assigns radial coordinate by salience. Audit of insertion paths not yet complete.                                                       | C05 §5, C05.12(3)                                     | INFERRED                       | Property test: evicted atom is always the live minimum-salience atom.                                                              |
| HN-A04 | Capacity               | Exponential volume `Vol(B_R) ~ π(sinh(2R)−2R)` is a proof sketch; storage bound is physical: `N_max = cache_budget / mem(entry)`.                                                                        | C05.11                                                | INFERRED                       | No "exponential capacity" claim without accompanying `N_max`.                                                                      |
| HN-A05 | Retrieval honesty      | Throughput receipt template in C12 (`decode_tok_s` p50 68.2, ratio 1.61, status PASS) is unlabeled. Values are PROPOSED template shapes, not results.                                                    | C05.13                                                | VERIFIED (as schema)           | `claim-lint` (C15) must gate any receipt citing measured values.                                                                   |
| HN-A06 | Geometry proof         | `GAP-C05-01`: the on-disk systems proof for the Hyperbolic Memory Map has not been transcribed into the design. Blocks A1 preregistration freeze.                                                        | C05.14                                                | VERIFIED (gap open)            | Locate, transcribe, label each theorem VERIFIED or withdraw.                                                                       |
| HN-A07 | Interop geometry       | Canonical compute/storage form is Lorentz `H³`; Poincaré ball is projection only. Isometry gate 4.4e-9 over 900 pairs (monster-hybrid-server verified). Q32.31 fixed point, no overflow at `x₀ ≥ 65536`. | C05 §1–2                                              | VERIFIED                       | Required gate C35.10; canonical-form decision frozen.                                                                              |
| HN-A08 | Picard reduction       | Idempotence `reduce(reduce(x)) = reduce(x)` verified for radii `{25, 250, 5000, 50000}`. Cusp folding `w > 64`.                                                                                          | C05 §3                                                | VERIFIED (at those radii)      | Extend fuzz range in G-A40 addendum.                                                                                               |
| HN-A09 | ACFT depth gate        | Depth gate `k*` chosen by variational free-energy minimization; TwisterAttention uses k-NN, not full context. Correctness of the minimization objective unmeasured.                                      | C05 §6                                                | INFERRED/PROPOSED              | Add A1 ablation of depth-gate policy.                                                                                              |
| HN-A10 | Fallback               | Exact-attention fallback is specified as always available via kill switch (C10.4). Enforcement in code not verified present.                                                                             | C05 canonical decision                                | PROPOSED                       | Fail-closed gate: if hyperbolic path receipt missing, dispatch exact attention.                                                    |
| HN-A11 | Persistence            | Atomic snapshot (`.atoma_map`, tmp + rename) + WAL (`.atoma_map.wal`, SHA-256) + recovery replay. Concurrency: 8 threads snapshot isolation.                                                             | C05 §7                                                | REPORTED (code shape inferred) | Crash-injection test required before VERIFIED.                                                                                     |
| HN-A12 | Cross-silo containment | Forward closure leaks across overlap boundaries (`overlap_allowance > 0`); `primary_role` is first silo only, distorting relevance.                                                                      | AdvAudit AA-02/AA-08/AA-11                            | INFERRED                       | Containment guard + audit `Ω_{r,s}`; pick home well per atom.                                                                      |
| HN-A13 | Map starvation         | `λ_map` can clamp to 0 under token pressure; supporting recall vanishes silently with no receipted reason.                                                                                               | AdvAudit AA-09                                        | INFERRED                       | Emit `map_starved_reason` and capsule-only receipt flag.                                                                           |
| HN-A14 | Confound enforcement   | A39 token confound (`tokens_B > 1.10 × tokens_A`) is documented but NOT enforced in harness.                                                                                                             | ErrAndImprovements-AA E9, AdvAudit AA-10              | VERIFIED (open)                | Harness-level void-for-all-judges guard.                                                                                           |
| HN-A15 | Empty silo             | `allowed_silos = []` produces undefined carve instead of explicit error/fallback.                                                                                                                        | AdvAudit AA-01/AA-07                                  | INFERRED                       | Prefer error at Allocate/harness boundary; at minimum count `empty_carve`.                                                         |
| HN-A16 | Forward-closure bound  | Hardcoded `max_hops = 64` with no relation to `QualityTarget` or `max_atoms`; worst-case quadratic.                                                                                                      | AdvAudit AA-12                                        | INFERRED                       | Single `max_atoms` + hop cap patch.                                                                                                |
| HN-A17 | Hook drift             | Hand-edited `hooks.json` vs compile-time hook manifest (23 moments). Drift risk.                                                                                                                         | C02 D6                                                | DESIGNED                       | Build fails on drift.                                                                                                              |
| HN-A18 | Portability            | Absolute paths `/Users/...` in prototype; enterprise needs `ATOMA_WORKSPACE_ROOT`.                                                                                                                       | C02 D7                                                | FROZEN (invariant)             | Portability smoke (NF4).                                                                                                           |
| HN-A19 | Plugin kernel          | Hyperbolic microkernel plugin spec exists (oracle primacy, binary pinning, receipts) but is a PROPOSED specification.                                                                                    | `HyperbolicMicrokernelPlugin-Spec.md`                 | PROPOSED                       | Generalize its triple to every microkernel before ship.                                                                            |
| HN-A20 | CUDA verification      | FW-A1-CUDA scaffold compiles (16/16 tests) on Mac CPU fallback; NO CUDA executed; BM-01 Picard gap unresolved (verify_idempotence returns false by design).                                              | C06.22                                                | VERIFIED (as stated)           | Linux+NVIDIA/H100 run required; CUDA cannot run on Apple Silicon.                                                                  |

### 1.2 Audit summary

v1 is a **design-rich, evidence-partial** system. Its strongest VERIFIED surfaces are
Lorentz-isometry interop, Picard idempotence at named radii, vault parsing, harness
determinism, and gate arithmetic. Its weakest surfaces are the **native-vault geometry
encoding** (A40 null result), **unmeasured complexity/capacity claims**, **unenforced
harness confound guards**, and **unexecuted CUDA kernels**. These are the seams the
HyperMind master design must harden.

---

## 2. PressureTestAudit of v1

Adversarial and stress scenarios. Each states the pressure, the expected failure mode if
unhandled, and the mitigation required before any claim.

| ID    | Pressure                                         | Expected failure if unhandled                 | Mitigation (required)                                                               |
| ----- | ------------------------------------------------ | --------------------------------------------- | ----------------------------------------------------------------------------------- |
| PT-01 | Adversarial clustering of atoms (all within δ)   | VP-tree degrades O(N); latency spike, OOM     | Clustered stress case in A1; label `O(log N)` INFERRED with assumed `c`             |
| PT-02 | Memory pressure approaching cache ceiling        | `λ_map → 0`; silent loss of supporting recall | `map_starved_reason` telemetry + capsule-only receipt flag; admission control (C21) |
| PT-03 | 8 concurrent writer turns, interleaved snapshots | Cross-session leakage, torn state             | Snapshot isolation proof + concurrent-turn gate 100% isolation                      |
| PT-04 | Interrupted write mid-snapshot                   | Partial/corrupt `.atoma_map`                  | Atomic tmp+rename; WAL tail truncation; SHA-256 verify; 100% recovery gate          |
| PT-05 | Token inflation in treatment arm                 | Unfair "efficiency" claim                     | Harness void-for-all-judges when `tokens_B > 1.10 × tokens_A`                       |
| PT-06 | Cross-silo traversal under high overlap          | Boundary-atom leak between silos              | Containment guard per AA-02; receipted `Ω_{r,s}` policy                             |
| PT-07 | Empty `allowed_silos`                            | Undefined carve / panic                       | Explicit error at Allocate/harness boundary                                         |
| PT-08 | Forward closure without bound                    | Quadratic traversal                           | `max_atoms` + hop cap                                                               |
| PT-09 | Projection round-trip near boundary (`‖p‖ → 1`)  | Numerical blowup in `π⁻¹`                     | Clamp/verify; Picard reduction; tolerance gate 4.4e-9                               |
| PT-10 | MCP client sending malformed tool args           | State corruption or injection                 | JSON-schema validation before dispatch; fail-closed                                 |
| PT-11 | MCP client replaying stale checkpoint            | Resurrecting withdrawn geometry claims        | Checkpoint includes content hash + epoch; reject on mismatch                        |
| PT-12 | MCP caller exhausting retrieval budget           | Denial-of-service / cost blowup               | Per-caller rate limits, `k` cap, timeout, admission tokens                          |
| PT-13 | Model swap (new base model, same map)            | Unproven map portability                      | "Memory passport" portability test (Novel §12.2)                                    |
| PT-14 | Red-team poisoning of ingested text              | Memory becomes an attack channel              | Ingest sanitization + provenance receipts + taint labels                            |
| PT-15 | PII in memory retrieved by wrong lane            | Privacy breach                                | Basin firewall (RBAC per basin) + redaction by radial salience                      |

---

## 3. Proposed fixes / How it was fixed and implemented

This section maps findings → fixes → implementation state. Statuses: **SHIPPED** (verified in
tree), **DESIGNED** (spec exists), **OPEN** (not yet implemented). Nothing is claimed
VERIFIED unless a receipt exists.

| Finding | Fix                                                                                                                                | Impl. location / shape                | Status                                                                |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- | --------------------------------------------------------------------- |
| HN-A01  | Treat native map as retrieval substrate; withdraw geometry-retraining claim for this vault; require v1.1 A40 leg before retraining | `A40-RunReport` §3–4                  | RECORDED                                                              |
| HN-A02  | State expansion constant `c` in prereg; add clustered stress corpus                                                                | A1 preregistration                    | OPEN                                                                  |
| HN-A03  | Audit all insertion paths; property test on eviction salience                                                                      | C25 hole substrate                    | OPEN                                                                  |
| HN-A04  | Publish `N_max` alongside any capacity claim                                                                                       | C21 admission control                 | OPEN                                                                  |
| HN-A05  | `claim-lint` gates unlabeled receipts                                                                                              | C15/C33                               | DESIGNED                                                              |
| HN-A06  | Transcribe on-disk systems proof; label theorems                                                                                   | C05 GAP-C05-01                        | OPEN                                                                  |
| HN-A10  | Exact-attention kill switch enforced in dispatch                                                                                   | C10.4                                 | DESIGNED                                                              |
| HN-A12  | Containment guard before insert in forward closure                                                                                 | AA-02                                 | OPEN (shipped per ErrAndImprovements: "shipped; needs property test") |
| HN-A13  | `map_starved_reason` telemetry + receipt flag                                                                                      | AA-09                                 | OPEN                                                                  |
| HN-A14  | Harness void-for-all-judges on token confound                                                                                      | AA-10                                 | OPEN                                                                  |
| HN-A15  | Explicit error for empty silo                                                                                                      | AA-01/AA-07                           | OPEN                                                                  |
| HN-A16  | `max_atoms` + hop cap                                                                                                              | AA-12/AA-03                           | OPEN (merge)                                                          |
| HN-A17  | Compile-time hook manifest; build fails on drift                                                                                   | C06 D6                                | DESIGNED                                                              |
| HN-A18  | `ATOMA_WORKSPACE_ROOT` + no absolute paths                                                                                         | C02 D7                                | FROZEN invariant                                                      |
| HN-A19  | Generalize microkernel plugin triple (oracle/verify/binary pin) to all kernels                                                     | `HyperbolicMicrokernelPlugin-Spec.md` | PROPOSED                                                              |
| HN-A20  | Run BM-01 Picard on Linux+CUDA; resolve full SL(2,Z[i]) action                                                                     | FW-A1-CUDA                            | OPEN (Mac cannot run CUDA)                                            |

**How the fixes compose (implementation order, PROPOSED):**

1. **Freeze the contract** (Rules §5) and the MCP tool schemas (§9) before code.
2. **Geometry layer**: canonical Lorentz `H³`, projection with verified isometry,
   Picard reduction, VP-tree with measured `c`, salience-radial insertion.
3. **Nested KV adapter**: attention queries the same VP-tree; depth gate `k*` via free
   energy; exact-attention fallback kill switch.
4. **Persistence + recovery**: snapshot/WAL with crash injection tests.
5. **MCP server**: stdio + HTTP, schema validation, receipts, auth, rate limits.
6. **Harness adapters**: Ares (hole substrate), Zed agent, generic MCP client.
7. **Ablations + receipts**: A1 geometry, clustered stress, portability, recovery.

---

## 4. Rules (normative invariants)

1. **Canonical geometry is Lorentz `H³`.** Poincaré `B³` is projection for UX/radial
   salience only. Isometry gate `< 4.4e-9` over 900 pairs is a ship gate.
2. **Nested, not separate.** The KV cache occupies coordinates in the same Atlas as
   long-term memory. No dense KV allocation may be introduced without an explicit
   ablation. (D1 frozen.)
3. **No hidden heuristics in eviction.** Victim = `argmax x₀` unpinned (max radial).
   Insertion must assign radial coordinate by salience or eviction is declared heuristic
   and labeled as such.
4. **Exact attention is always available.** Kill switch routes any span to exact
   attention. If a hyperbolic receipt is missing, dispatch exact attention (fail closed).
5. **No O(N²) full forward pass as default.** ACFT uses k-NN TwisterAttention with
   depth gate `k*`. Any full-pass path must be receipted and opt-in.
6. **Every claim labeled.** VERIFIED/REPORTED/INFERRED/PROPOSED. Unlabeled = unverified.
7. **No bulk.** Per-item disposition, per-claim evidence. No batch claims.
8. **Corrections are entries.** Append a dated correction naming what it replaces.
   Never delete the wrong claim.
9. **Receipts for dispatch.** A kernel/tool/retrieval that lacks a passing receipt
   cannot be dispatched. (Microkernel plugin rule, generalized.)
10. **Binary pinning.** Compiled artifacts (`.metallib`, map snapshots) are loaded by
    pinned hash, never recompiled at serve time.
11. **Workspace portability.** No absolute user paths. Use `ATOMA_WORKSPACE_ROOT`.
12. **Idempotent recovery.** `reduce² = reduce`; WAL replay truncates torn tails; snapshot
    SHA-256 verified.
13. **Concurrency honesty.** 8-thread turn execution with snapshot isolation; any
    cross-session leakage fails the gate.
14. **Compatibility.** Plugin speaks MCP (JSON-RPC) and exposes a stable versioned schema.
    Breaking changes require a major version bump and migration note.
15. **Privacy by construction.** Per-basin RBAC; no cross-lane retrieval without an explicit
    overlap allowance `Ω > 0` that is listed and receipted.

---

## 5. Overview

**What HyperMind + Nested KV is.** A geometric memory substrate that replaces the flat
`O(N²)` attention KV cache with coordinates in a hyperbolic Atlas (`H³ × T²`). Memory
(atoms) and their attention keys/values share one space; retrieval is k-NN in hyperbolic
distance, so the system builds only what it needs to recall and complete the task, not a
full forward pass over everything regardless of reasoning depth.

**Why "Nested".** The KV cache nests inside the Atlas rather than living as a separate
dense tensor. Nested minimization (task objective over a carved map view) replaces
minimizing free energy over the whole Atlas.

**For whom.** Solo developers and small teams who want enterprise-grade agentic memory
without external APIs or GPU clusters; any harness that can speak MCP.

**Hardware target (PROPOSED).** Apple Silicon, 36 GB+ unified memory, Metal 3.0+.
CUDA path is a separate, opt-in lane on Linux+NVIDIA and is NOT runnable on Apple
Silicon (VERIFIED limitation).

**Non-goals.**

- No claim of measured throughput superiority until A1 runs.
- No replacement of the exact-attention safety path.
- No mandatory cloud dependency for the core loop.

---

## 6. Workflow

1. **Negotiate** capabilities over MCP (`initialize`).
2. **Open/Create map**: load `.atoma_map` + replay WAL, or initialize empty with canonical
   geometry + epoch.
3. **Ingest**: turn segments, tool results, and retrieved docs are folded into atoms with
   salience-radial coordinates; provenance + taint labels attached.
4. **Recall**: `query(q, k)` returns k-NN atoms with scores (hyperbolic + toroidal phase).
5. **Attend (nested KV)**: TwisterAttention uses the recalled set; depth gate `k*` chosen
   by free-energy minimization.
6. **Compact**: attractor-basin compaction folds middle turns; original objective and
   recent state pinned (anti-lobotomy checkpoint).
7. **Checkpoint**: durable `BrainContinuityCheckpoint` (original objective, verified tool
   ledger, active basin coords, recent state).
8. **Snapshot**: atomic write `.tmp` → rename; WAL append; hash recorded.
9. **Recover**: SHA-256 verify snapshot; truncate torn WAL tail; replay; pin checkpoint.
10. **Federate (optional)**: export/import memory passport between harnesses with a
    compatibility report (Novel §12.2).

---

## 7. How it works

### 7.1 Geometry

Canonical compute/storage: Lorentz hyperboloid

$$
H^3 = \{ x \in \mathbb{R}^4 : \langle x,x\rangle_{\mathcal L}=1,\ x_0>0 \},
\quad
\langle x,y\rangle_{\mathcal L}=x_0y_0-x_1y_1-x_2y_2-x_3y_3.
$$

Poincaré projection (UX/salience): `π(x) = (x₁,x₂,x₃)/(x₀+1)`, with
`π⁻¹(p) = ((1+‖p‖²)/(1−‖p‖²), 2p/(1−‖p‖²))`. Isometry gate `< 4.4e-9`.

Fixed-point: Q32.31 with wide i128 accumulator; overflow-safe at `x₀ ≥ 65536`.

Picard fundamental-domain reduction for unbounded radii (`x₀ > 8`): `Γ = PSL(2, Z[i])`
action; moderate radii fixed-point iteration (≤32 steps); cusp folding `w > 64`.
Idempotence gate 100% at tested radii.

Product manifold `H³ × T²`: toroidal phase `φ = (θ_f, θ_s)`; joint score

$$
\mathrm{score}(q,a)=\langle q_{H^3},a_{H^3}\rangle_{\mathcal L}
+\frac14 d_{T^2}^2(\phi_q,\phi_a).
$$

### 7.2 Storage and retrieval

Geometric memory map: vantage-point tree on `H³` with hyperbolic distance
`d_{H³}(x,y)=arcosh(⟨x,y⟩_ℒ)`. Eviction: geometric forgetting — victim = argmax
`x₀` unpinned (closest to boundary = lowest salience).

Nested KV: atoms carry key/value references; attention consumes the recalled subset:

$$
\mathrm{TwisterAttention}(q,K,V)=\sum_{i\in \mathcal N_k(q)} \alpha_i V_i,\quad
\alpha_i = \frac{\exp(\langle q,k_i\rangle_{\mathcal L}/\tau)}{\sum_j \exp(\langle q,k_j\rangle_{\mathcal L}/\tau)}.
$$

Depth gate:

$$
k^*=\arg\min_k\bigl(C_{\mathrm{retrieve}}(k)+C_{\mathrm{attend}}(k)
-\mathbb E[\log p(\mathrm{answer}\mid \mathrm{context}_k)]\bigr).
$$

### 7.3 Persistence

Snapshot `.atoma_map` (HashMap<AtomId,Atom> + VP-tree index) written to `.tmp` then
atomic rename. WAL `.atoma_map.wal` append-only `AuditEntry` (Write/Evict + logical clock).
Recovery: verify SHA-256 snapshot; truncate torn WAL tail; replay from last valid clock.
Concurrency: 8 threads, snapshot isolation.

### 7.4 Active inference and compaction

ACFT replaces blanket forward pass with task-scoped retrieval. Compaction folds middle
turns into durable checkpoint; original objective (messages[0]/[1]) and recent state
pinned to keep prefix cache warm. Anti-lobotomy: pre-compaction fold at ~52% context, well
before 88% auto-compact.

### 7.5 Verification gates

| Gate             | Test                         | Threshold                 |
| ---------------- | ---------------------------- | ------------------------- |
| Isometry         | `π ∘ π⁻¹` round-trip         | `< 4.4e-9`                |
| Picard           | `reduce² = reduce`           | 100%                      |
| VP-tree          | Phase coherence on retrieval | 13/13 tests               |
| Full chain       | E2E session hygiene + replay | 13/13                     |
| Atomic vault     | Interrupted write + restart  | 100% recovery             |
| Concurrency      | 8 threads, no leakage        | 100% isolation            |
| Clustered stress | A1 adversarial clustering    | Reported with assumed `c` |
| Portability      | `ATOMA_WORKSPACE_ROOT` smoke | pass (NF4)                |

---

## 8. Pieces (components)

| Piece             | Responsibility                                       | Technology (PROPOSED)    |
| ----------------- | ---------------------------------------------------- | ------------------------ |
| `hypermind-core`  | Geometry, VP-tree, atoms, scoring, eviction          | Rust (safe core)         |
| `nested-kv`       | Attention adapter, depth gate, fallback routing      | Rust + optional Metal    |
| `hyperkernel`     | Metal kernels + Rust oracles + receipts              | Rust + Metal `.metallib` |
| `map-store`       | Snapshot/WAL/recovery, checkpoints                   | Rust fs + SHA-256        |
| `mcp-server`      | MCP transport, schema validation, tools/resources    | Rust (stdio + HTTP/SSE)  |
| `harness-adapter` | Ares lanes/holes, Zed agent, generic clients         | Rust + JSON-RPC          |
| `passports`       | Export/import maps with compatibility report         | Rust                     |
| `telemetry`       | tok/s, cache hit, recall depth, OOM headroom, energy | Rust + metrics endpoint  |
| `claim-lint`      | Receipt/label gate (C15)                             | Rust + CI                |
| `bench`           | Ablation runners, golden inputs                      | Rust + scripts           |

---

## 8.5 Language stack — scaffold for immutable, secure, compliance-first engineering (added)

The working set that makes HyperMind + Nested KV immutable, secure, and compliance-first
by construction rather than by audit after the fact:

| Layer                                                              | Language / format                                                                                                   | Why (first principles)                                                                                                                                                                                                    |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core substrate, MCP server, persistence, policy gateways, receipts | **Rust (stable, `#![forbid(unsafe_code)]` outside vetted FFI)**                                                     | Memory safety without GC pauses; deterministic ownership makes aliasing bugs compile errors; `Result`/`Option` make failure explicit; no hidden runtime.                                                                  |
| Hyperbolic kernels (Metal)                                         | **MSL + precompiled `.metallib`**                                                                                   | Deterministic binary artifacts ("Cubin" analogue); loaded by hash, never JIT-compiled at serve time.                                                                                                                      |
| Portable kernel fallback                                           | **WGSL**                                                                                                            | Cross-platform browser/WASM target without rewriting math; same oracle gate applies.                                                                                                                                      |
| Wire + storage schemas                                             | **Protocol Buffers (canonical) + JSON Schema (MCP boundary)**                                                       | Backward/forward compatible evolution; unknown fields preserved; canonical encoding for receipts. MCP itself requires JSON; the control plane still uses TOML.                                                            |
| Authorization & privacy policy                                     | **Cedar (policy-as-code)** (Rego acceptable)                                                                        | Decisions are data, testable, and auditable; RBAC/basin firewall rules are versioned and receipted.                                                                                                                       |
| Configuration                                                      | **TOML**                                                                                                            | Human-edited, diffable, schema-validatable; avoids JSON's foot-guns for hand editing.                                                                                                                                     |
| Harness / IDE adapters & MCP clients                               | **TypeScript 7.1 (experimental beta)** — pinned to whatever Cloudflare just released as `experimental` for v7.1beta | Thin translators only; no business logic; type-checked against the proto schema. Verify wrangler/worker-types compatibility before ship; fall back to the Cloudflare-supported stable TS if the beta breaks adapters.     |
| Sandbox for third-party microkernels                               | **WASM (wasm32-wasi)**                                                                                              | Capability-based isolation; a kernel cannot escape its memory or filesystem grant.                                                                                                                                        |
| Exiled from the core                                               | **Python** (no interpreter/process) and **JSON** outside the MCP envelope                                           | Python's GC and JSON's parsing failures are reliability hazards; both are exiled from systems architecture. AtomaDB's JSONL segments are wrapped behind binary snapshot/WAL/passport and never exposed on the serve path. |

Compliance-as-first-principle rules for the stack:

1. No `unsafe` in core crate except the single, reviewed memmap site (AtomaDB engine, reviewed).
2. All persistence writes are atomic (tmp + fsync + rename) and hashed; no in-place mutation.
3. Policy decisions come from Cedar evaluation, not `if` chains.
4. Schemas are versioned and additive only within a major; breaking changes require migration notes + receipt.
5. Kernels ship only with oracle-equality receipts; no unverified acceleration.
6. Every mutating call returns a receipt hash; audit is append-only.
7. Secrets never enter the map; taint labels propagate.

## 8.6 v1 build evidence (implemented)

v1 has been built top-to-bottom as a real, runnable plugin MCP server:

- **Crate.** `hypermind-nested-kv/` (Rust 2021), binary `hypermind-mcp` (stdio JSON-RPC 2.0).
- **Substrate decision.** AtomaDB engine (`vendor/atomadb-engine`, from `~/Desktop/AtomaDB-RAG-PAMI`) is reused
  verbatim as the immutable segment/mind store. Rationale: it already provides atomic writes, SHA-256/FNV
  segment fingerprints, role gating, append-only graph sidecars, and dense retrieval; rebuilding these would
  add risk without benefit. A "better" workflow would need to beat it on (a) atomic recovery, (b) identity
  enforcement at ingest, (c) segment targeting. None of those have been disproven yet, so AtomaDB stands.
- **Tools live.** `map.ingest`, `map.query`, `map.attend` (depth gate `k*`), `map.compact`,
  `map.checkpoint`, `map.snapshot`, `map.recover`, `map.health`, `map.export_passport`, `map.create`.
- **Smoke.** initialize → ingest → query → health all returned receipted JSON on local hardware
  (2026-10-27). Geometry is Lorentz `H³` canonical with Poincaré projection.
- **Location.** `/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-Global-HackAthon-Design/hypermind-nested-kv`.
- **Open before production.** `#![forbid(unsafe_code)]` across the crate (currently inherits AtomaDB's reviewed
  memmap site), Cedar policy engine wired instead of hardcoded roles, Curvature and `N_max` measured,
  crash-injection recovery test, MCP HTTP transport, and portability passport round-trip.

## 9. Hardening & plugin MCP to the Harness

### 9.1 Threat model (summarized)

Assets: memory contents, map integrity, model safety. Adversaries: malicious MCP clients,
poisoned ingest text, confused deputy across lanes, resource exhaustion. Trust boundaries:
process boundary between harness and MCP server; lane boundary within map; kernel oracle vs
accelerator.

### 9.2 MCP server contract

Transport: stdio (default, local) and HTTP/SSE (optional, loopback-first). Protocol:
JSON-RPC 2.0. Capability negotiation at `initialize`. Tools are **versioned**; schemas
frozen per major version.

Core tools (v1):

- `map.open` / `map.create` — load/create map at path (path validated under
  `ATOMA_WORKSPACE_ROOT`).
- `map.ingest` — fold turn/doc/tool-result into atoms with provenance + taint.
- `map.query` — k-NN recall with score and depth hint.
- `map.attend` — nested KV attention over recalled set with exact-attention fallback flag.
- `map.compact` — attractor-basin compaction; returns checkpoint.
- `map.checkpoint` — read/write continuity checkpoint.
- `map.snapshot` — atomic snapshot + WAL; returns hash.
- `map.recover` — verify + replay; returns status.
- `map.export_passport` / `map.import_passport` — portable map interchange.
- `map.evict_report` — explain last eviction decision (auditability).
- `map.health` — gates status, curvature, capacity `N_max`.

Resources: `map://atoms/{id}`, `map://basins/{role}`, `map://checkpoint/latest`,
`map://receipts/{id}`. Prompts: `compact-summary`, `recall-trace`.

### 9.3 Hardening checklist

- Input validation against frozen JSON schemas before dispatch (fail closed).
- Path confinement under `ATOMA_WORKSPACE_ROOT`; no absolute client paths.
- Per-caller rate limits and token budgets on `query`/`attend`.
- Depth `k` capped server-side; free-energy selection logged.
- Exact-attention kill switch honored even if client requests hyperbolic.
- Receipts (hash of inputs+outputs) on every mutating call.
- Taint labels propagated through ingest; quarantine for high-risk content.
- RBAC per basin; cross-basin access requires listed `Ω`.
- Atomic writes + WAL; crash injection tested.
- Content-addressed snapshots; reject on hash mismatch.
- Deterministic seeds logged for reproducibility.
- Side-effect free `query`/`health`; mutating calls idempotent where possible.
- Telemetry without secret material.
- Energy/joules-per-token accounting exposed.
- Backpressure: queue + degrade to exact attention under load, with receipted reason.

### 9.4 Harness integration

- **Ares**: map lives under hole substrate; lanes carve views; overlap policy receipted.
- **Zed/agent loop**: prefix cache stays warm by pinning messages[0]/[1] during compaction.
- **Generic (MCP clients)**: any client speaks the tool list above; no SDK required.

---

## 10. Plugin-Agnostic Exponentiator

HyperMind ships as a **tiny, dependency-light plugin** that _exponentiates_ whatever
harness hosts it: a harness gains geometric memory, nested KV, recovery, and receipts by
adding one MCP server entry. Design commitments:

- One integration surface (MCP JSON-RPC) — no language-specific SDK required.
- Stable versioned schemas; additive changes only within a major version.
- No global state in the server; map instances addressed by handle.
- Capability negotiation: older clients get a downgraded tool set, not a crash.
- Adapter shims for common hosts (Ares, Zed, LangChain, Claude Desktop) are thin
  translators, not forks.
- "Exponentiator" property: memory grows and compounds across sessions while code size
  and integration cost stay small; the burden of correctness moves into receipts and gates,
  not into the host.

---

## 11. Novel high-value additions (must be novelty, high value)

These are PROPOSED additions that extend the v1 design with high leverage.

11.1 **Geometric receipts.** Every retrieval emits a receipt containing query coordinate,
k, neighbor ids, scores, and `c` (expansion constant estimate). Makes memory auditable
like a transaction ledger, not a black box.

11.2 **Memory passport.** Portable `.atoma_map` + manifest with geometry version,
curvature, epoch, atom count, and a compatibility report on import (PASS/FAIL with reasons).
Enables "your memory follows you" across harnesses and machines without re-embedding.

11.3 **Basin firewall.** Per-basin RBAC with least-privilege defaults; cross-basin access
requires an explicitly listed, receipted `Ω`. Secret-bearing content is quarantined into
a high-radius basin with stricter access.

11.4 **Phase clocks.** Atoms carry a phase clock (toroidal) for staleness; decay is geometric
(increasing `x₀`), not heuristic. Old, low-salience memories fade by construction.

11.5 **Memory immune system.** Ingest sanitization + taint labels + adversarial-cluster
detection (`c` spike) + quarantine basin. A poisoned ingest cannot silently dominate
retrieval because its cluster trips the stress gate.

11.6 **Memory views (not copies).** Lanes carve views (`carve_task_view_for_role`) over
one physical map, eliminating weight-copy proliferation (A4). Views are read-only unless
explicitly forked; forks are receipted.

11.7 **Energy ledger.** Joules/token and joules/recall tracked per query and per lane;
targets efficiency claims that are otherwise unfalsifiable.

11.8 **Self-tuning curvature.** Curvature/depth self-tuning experiment (Frontier §8.7)
preregistered with a falsifier; curvature changes require a passport version bump.

11.9 **Explainable recall trace.** `map.query` can return an optional trace: path through
VP-tree, branch decisions, pruning bound. Debugging memory becomes geometric, not
archaeological.

11.10 **Golden oracle portability.** The Rust oracle outputs are the specification for every
kernel on every platform; a kernel ships only after oracle-equality. This is the "no hopeful
JIT" rule generalized from the microkernel plugin spec.

11.11 **Anti-lobotomy checkpoints across harnesses.** Standardized
`BrainContinuityCheckpoint` schema so compaction in any harness preserves objective +
recent state + verified tool ledger.

11.12 **Golden-empty map boot.** A well-defined empty map with canonical geometry, proven
recovery, and a baseline gate; booting "from nothing" is a tested path, not an accident.

---

## 12. Verification & receipts

### 12.1 Ablation hooks

- **A1** KV cache: standard transformer O(N²) vs nested hyperbolic cache. Throughput +
  peak memory on fixed corpus, baseline number on same hardware/workload.
- **Clustered stress**: adversarial clustering included in A1 corpus.
- **Portability**: `ATOMA_WORKSPACE_ROOT` smoke (NF4).
- **Recovery**: interrupted write + restart = 100%.
- **Concurrency**: 8 threads, no leakage.
- **Portability passport**: import/export compatibility report on two machines.

### 12.2 Success criteria (all must be receipted)

1. Isometry gate pass.
2. Picard idempotence 100% at extended radii.
3. VP-tree tests 13/13 + clustered stress reported.
4. Atomic vault recovery 100%.
5. Concurrency 100%.
6. Nested KV attention reproduces or improves on exact attention for the safety-critical
   spans listed in Rule 4 (no regression).
7. MCP server schema validation + rate limit tests pass.
8. No unlabeled value cited anywhere in docs or receipts (claim-lint).

---

## 13. Open gaps (tracked)

| Gap ID     | Description                                                | Blocks                 |
| ---------- | ---------------------------------------------------------- | ---------------------- |
| GAP-C05-01 | Transcribe on-disk systems proof for Hyperbolic Memory Map | A1 freeze              |
| GAP-HM-01  | Define measured expansion constant `c` for corpus          | Complexity claim       |
| GAP-HM-02  | Complete insertion-path audit (salience-radial)            | Eviction claim         |
| GAP-HM-03  | Publish `N_max` per cache budget                           | Capacity claim         |
| GAP-HM-04  | Enforce exact-attention kill switch in code                | Safety claim           |
| GAP-HM-05  | Implement `map_starved_reason` telemetry                   | Pressure honesty       |
| GAP-HM-06  | BM-01 Picard resolution on CUDA (Linux)                    | Geometry proof on CUDA |
| GAP-HM-07  | MCP server implementation                                  | Plugin claim           |

---

## 14. Sources (key)

- `MasterDesignDocument/C02-Differences-vs-Ad-Hoc-App/Design.md` (D1–D9)
- `MasterDesignDocument/C05-Atlas-Hyperbolic-KV/Design.md` (geometry, cache, gates)
- `MasterDesignDocument/C06-Frontier-Push/AdvAudit.md`, `ErrAndImprovements-AA.md`
- `MasterDesignDocument/C06-Frontier-Push/A40-RunReport-2026-10-27.md`
- `MasterDesignDocument/C06-Frontier-Push/HyperbolicMicrokernelPlugin-Spec.md`
- `MasterDesignDocument/C06-Frontier-Push/DyptechCertificationAndFormalization.md`
- `AUDIT-RECEIPT.md` (repo-wide audit posture)

---

_Label policy: claims in this document are PROPOSED unless explicitly VERIFIED/INFERRED above.
Measured numbers require a filed ablation hash and a passing receipt before citation._

---

## Appendix P — CodeBasePressureTest audit of HyperMind + Nested KV v1 (dated record, 2026-10-28)

Pressure audit run against `hypermind-nested-kv` (Rust 2021, vendored `atomadb` engine),
release binary + `cargo test -- --test-threads=1`. All verdicts below are evidence from this pass.

| ID | Surface | Pressure | Expected | Observed | Verdict |
|----|---------|----------|----------|----------|---------|
| PA-01 | Build | `cargo build --release` | 0 errors, binary produced | Release build finishes, 2 pre-existing warnings (unused var, AtomaDB dead code) | VERIFIED PASS |
| PA-02 | Test suite | `cargo test -- --test-threads=1` | 11/11 pass | 11/11 pass (geometry, depth bound, snapshot/recover, VP-tree exactness, 6 gate tests) | VERIFIED PASS |
| PA-03 | VP-tree exactness | 16-atom ingest, 5-NN vs brute force | Identical result sets | `vp_tree_matches_brute_force` passes | VERIFIED PASS |
| PA-04 | Depth gate | 5 atoms, attend k=3 | `depth_k ≤ n`, data-dependent | `depth_k` selected from score-mass curve, bounded ≤ n | VERIFIED PASS |
| PA-05 | Geometry | Any text, any salience | `⟨x,x⟩_L = 1` and radius ∝ (1−salience) | `hyperboloid_constraint`, `salience_sets_radius` pass | VERIFIED PASS |
| PA-06 | Receipts | All mutating tools | receipt sha256 + timestamp | ingest/query/attend/compact/snapshot/recover/health all receipted (recover fixed this pass) | VERIFIED PASS (was FAIL, fixed) |
| PA-07 | Health honesty | `map.health` | No unmeasured superlatives | Reports `exact_attention_measured: false`, VP-string corrected to exact-prune semantics (was FAIL, fixed) | VERIFIED PASS (was FAIL, fixed) |
| PA-08 | Storage integrity | snapshot → corrupt 1 byte → recover | `hash-mismatch`, no silent load | Stored sha256 `538842c1…` vs corrupted `432abde8…` → `hash-mismatch` returned, vault untouched | VERIFIED PASS |
| PA-09 | WAL shape | snapshot | Atomic tmp+rename, length-prefixed bincode WAL | 1 record, len 104, clean parse, 0 trailing bytes | VERIFIED PASS |
| PA-10 | Malformed input | `text: 42`, `qtext: [..]`, unknown tool, unknown method, garbage line | Structured errors / JSON-RPC errors, no crash | `schema: … must be string`, `unknown tool`, `-32601 method not found`, `-32700 parse error`, blank lines skipped, stderr empty | VERIFIED PASS |
| PA-11 | Path confinement | `subpath: ../etc` | Reject | `path traversal rejected` | VERIFIED PASS |
| PA-12 | Role gating | `HYPERMIND_ROLE=ingest` → write | Deny | `role may not deposit to mind` | VERIFIED PASS |
| PA-13 | Concurrency | 6 sequential ingests + 8-thread mutexed test | No loss, no panic | 6 ids assigned; `concurrency_smoke` 8/8 atoms present | VERIFIED PASS (lock-level, see PA-18) |
| PA-14 | Oversized payload | 200 KB text ingest | Accept with receipt (bounded by disk) | Accepted, receipted | VERIFIED PASS (capacity unbounded by config — see PA-19) |
| PA-15 | Replay | snapshot twice, no change | Identical digest | Same digest `18500b32…` both times | VERIFIED PASS |
| PA-16 | Portability | `ATOMA_WORKSPACE_ROOT` redirect | Works under new root | `portability_smoke` passes | VERIFIED PASS |
| PA-17 | HTTP transport | JSON-RPC POST to loopback server | Same results as stdio | ingest + health return receipted content | VERIFIED PASS |
| PA-18 | True parallelism | 8 OS processes writing one vault | IPC-safe isolation | **Not claimed.** Mutex covers in-process threads only; multi-process writers share `mind.jsonl` without locking. | OPEN GAP G-HM-08 |
| PA-19 | Budgets | Per-caller rate limits, token budgets, k cap, DB size ceiling | Enforced | **Absent.** Only `k` naturally bounds retrieval width; no rate/token/size limits exist. | OPEN GAP G-HM-09 |
| PA-20 | Complexity honesty | Health/advertised complexity | Match implementation | Was `O(N) brute-force … VP-tree planned` while code already ran VP-tree. Corrected to exact-prune VP semantics; average `O(log N)` stated with bounded-expansion condition per C05.12. | VERIFIED PASS (was FAIL, fixed) |

## Appendix C — CertifyOrProposeGaps verdict (2026-10-28)

**Certified to ship as a local plugin (PROVEN):** build, 11/11 tests, VP-tree exactness,
data-dependent depth gate, hyperboloid geometry + salience-radius, receipted tools,
hash-verified binary snapshots, tamper-visible recovery, malformed/traversal/role
rejections, stdio + HTTP JSON-RPC, portability, replay-stable digests.

**Blocked from certified claims (OPEN, with owner and what each waits on):**
- **G-HM-08 multi-process write isolation** — waits on a file lock or single-writer service around `mind.jsonl` + a 2-process writer test. Owner: hypermind core.
- **G-HM-09 budgets** — waits on per-caller rate limits, `k` cap, and DB size ceiling in config + denial receipts. Owner: hypermind core.
- **G-HM-10 Cedar policy** — waits on wiring the policy engine instead of hardcoded roles. Owner: compliance lane.
- **G-HM-11 TLS/auth for HTTP** — waits on auth + TLS/reverse-proxy posture and an auth-negative test. Owner: security lane.
- **G-HM-12 measured performance** — waits on the A1 ablation (fixed corpus, same hardware) before any throughput claim. Owner: evaluation lane.
- **G-HM-13 external geometry retraining claims** — unchanged from the A40 null result; v1 makes no retraining claims and earns none. Owner: research lane.
- **Pre-existing AtomaDB warnings** (unused import/var in engine, `AnchorVec.text` dead) — cosmetic, evidence they do not affect v1 paths. Owner: vendored engine.

**Next steps (dependency order):**
1. File-lock `mind.jsonl` appends; add the 2-process writer test (closes G-HM-08).
2. Add config budgets + denial receipts (closes G-HM-09).
3. Wire Cedar over the three roles; keep the denial test (closes G-HM-10).
4. Document the HTTP trust model (loopback-only) or add auth (closes G-HM-11).
5. Run A1 + clustered stress + cross-hardware replication; only then cite performance (closes G-HM-12/G-HM-13).

**Single next command to run:** `cargo test -- --test-threads=1` in `hypermind-nested-kv`
(then `cargo build --release`), repeating the PA-01/PA-02 evidence after any change.
