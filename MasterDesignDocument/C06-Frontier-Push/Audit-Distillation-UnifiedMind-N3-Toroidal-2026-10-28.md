# Audit — Distillation into Unified Mind / N³ Store as Plugin / Toroidal Attention / Toroidal Phase Positional Encoder

**Date.** 2026-10-28. **Scope.** Four workstreams named by the steward: (W1) distillation of open-source teachers into the unified mind, (W2) the N³ store as a plugin (hypermind-nested-kv MCP server), (W3) toroidal attention, (W4) toroidal phase positional encoder (TPE). **Method.** Read-only audit of this repository (`MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack`) plus local toolchain checks; no vault/monster-server source reachable from this workspace, so vault claims are REPORTED as already audited in `VaultMapDreamstateFormalization.md` (not re-verified here). Labels per documentation standards: VERIFIED / REPORTED / INFERRED / PROPOSED / NOT FOUND.

---

## 0. Verification baseline (commands actually run this pass)

| Command                                                               | Result                                        |
| --------------------------------------------------------------------- | --------------------------------------------- |
| `cargo test --manifest-path hypermind-nested-kv/Cargo.toml`           | **12 passed / 7 failed** (0.40s)              |
| `cargo test --manifest-path research-grade-harness/Cargo.toml`        | **26 passed / 0 failed** (lib+tests+pressure) |
| `cargo test --manifest-path ablation_benchmarks/cuda_rust/Cargo.toml` | **30 passed / 0 failed** (lib+tests)          |

VERIFIED.

---

## 1. W1 — Distillation of models into the unified mind

**What the bytes establish (REPORTED, prior audit `VaultMapDreamstateFormalization.md` §0/§2, not re-verified here):**

| Teacher                             | Evidence strength                                                                        |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| GLIMMER (Muse-Glimmer)              | REPORTED — strongest single-teacher evidence (52-layer report, hidden cosine 0.99999994) |
| Qwen3.6 MoE                         | REPORTED — layer-scan comment + on-disk `Qwen3.6-MoE-Combined`; per-arc share unmeasured |
| 3.827B dense                        | **NOT FOUND** — no match in vault code/docs/bytes; identity unresolved (open question)   |
| DeepSeek (safetensors)              | REPORTED — code comment only; weakest link                                               |
| QwQ-32B math                        | REPORTED — byte fragments + Tier2 math rows                                              |
| Nemotron-3 (on disk: 3.5-Lightning) | REPORTED — +26 MB landing event; version-name mismatch flagged                           |

**Unified identity.** `Qwen3.8-27B+Muse-Glimmer-UnifiedMind` present once in the 604 MB map header, followed by a 32-byte digest — VERIFIED in the prior audit. Serving identity = Qwen3.8-27B + Glimmer-distilled, consistent with C01.

**Semantics.** Distillation is NOT a weight merge. VERIFIED per vault correction ledger + `distill_engine.rs`: one `FrozenBaseWeights` buffer (base_weight_copies = 1), teachers become provenance-tagged trajectory populations (`origin_model_tag`, `teacher_fingerprint` SHA-256) in a shared address space. Unification = shared address space + shared geometry + per-arc provenance. Any "merged weights" claim is contradicted and must not be asserted.

**Audit outcome W1:** mechanism and provenance architecture are well-specified and (REPORTED) vault-implemented; teacher-attribution completeness is uneven; one claimed teacher (3.827B dense) is NOT FOUND. Recommend the steward resolve Q-1 before any teacher-share figures are quoted in a submission.

---

## 2. W2 — N³ store as a plugin (`hypermind-nested-kv`)

**Implementation exists.** `hypermind-nested-kv/Cargo.toml` describes it as "hyperbolic memory substrate as a plugin MCP server. Reuses AtomaDB engine as immutable storage substrate." Binary `hypermind-mcp`. VERIFIED.

**MCP surface.** `map.ingest`, `map.attend`, `map.hyper`, plus capsule/dreamstate-related handlers present (read via `handle_tool` dispatch around lines 1560–2000). VERIFIED shape.

**Storage substrate.** AtomDB engine vendored (`vendor/atomadb-engine`); `Atom` carries coord[4], phase[2], text, salience, provenance, taint, domain, pinned. VERIFIED.

**Test health (VERIFIED this pass, `cargo test`):**

- 12 pass, 7 fail: `basins_fold_and_necessity`, `carve_view_weights_separated`, `dissolution_lifecycle`, `ledger_proofs_terminate_in_root`, `snapshot_recover_roundtrip`, `swarm_20_views_certification`, `concurrency_smoke`.
- Failures are seeded-state assertions (e.g., `selected_basin_ids` length 0 instead of ≥1; swarm atom count 0 instead of 30; concurrency 0 instead of 8). These suggest the test setup does not produce the atoms/basins the assertions expect under the current binary's ingestion path — a pre-existing break in the integration seed, not a compile error.

**Audit outcome W2:** the plugin exists, builds, and passes 12/19 tests, but the core invariants it claims (basin folding, swarm carve with weight_copies==1, dissolution lifecycle, ledger termination, snapshot round-trip, concurrency) are **currently red**. Per project standards this must be fixed or scoped down before submission; do not cite these gates as passing. Root cause needs a focused debug pass (deferred; not attempted to avoid speculative edits).

---

## 3. W3 — Toroidal attention

**Implementation found (VERIFIED this repo):**

- `hypermind-nested-kv/src/main.rs::toroidal_coherence(q, a)` computes `cos(Δθ_f) + cos(Δθ_s)` (chord-based coherence, higher = more coherent). Used to compute normalized `toroidal_weight` in `map.attend` after active-inference gating. VERIFIED.
- `ablation_benchmarks/cuda_rust/src/toroidal.rs` implements toroidal phase ops: `random_toroidal_phase`, `phase_coherence` (thresholded), `phase_distance_wrapped` with Q32.31 wrap handling. 14 (suite) tests pass. VERIFIED.
- `MasterDesignDocument/C05` defines `d_T² = (1−cos Δθ_f)+(1−cos Δθ_s)` and joint score `⟨,⟩_L + ¼·d_T²`, matching the code's use of `>> 2` (¼). VERIFIED consistency (prior audit B-1/B-2 resolved: retrieval is argmin cosine, doc sign corrected).

**NOT present in this repo:** `toroidal_attention.rs` (referenced in Dyptech certification as `toroidal_attention.rs:179-307`) — the monster-server module is not mounted in this workspace, so the per-layer phase-gain fns (`evaluate_map_attention_layer`, `fit_map_attention_gain`) cannot be re-verified here. The certification's "13/13 map-toroidal-attention tests pass" remains REPORTED.

**Audit outcome W3:** toroidal attention is implemented in the plugin (coherence + weighted attend) and the cuda reference has passing phase ops; the higher-fidelity per-layer gain analysis is off-disk and not independently verified.

---

## 4. W4 — Toroidal phase positional encoder (TPE)

**Spec exists (REPORTED/PROPOSED).** `MasterDesignDocument/C06-Frontier-Push/ToroidalPhasePositionalEncoding-*` (10+ files: concept formalization, concepts-to-proofs, standalone, math MVP, derivation chain, cheat sheet) and `PredictionErrorMinimization-Scope.md`. Describes position as `(angle, radius, amplitude)` with Kuramoto 2:1 lock, unwrapped arc, `r(w)` radius, η rapidity; falsifier angle-only ablation. VERIFIED file presence; content not math-checked here (prior certification treats phases 4/5 as PROPOSED).

**Code implementation:** NOT FOUND in this repo. Search across `research-grade-harness`, `hypermind-nested-kv`, `ablation_benchmarks` returned no TPE/positional-encoder module. The only toroidal phase engineering on disk is the cuda ops (phase distance/coherence) and the hypermind Atom.phase[2] + Kuramoto? — not encoder evolution. `evolve_vector` is referenced in certification as living in `torus.rs` (monster-server, off-disk).

**Kuramoto / 2:1 lock claims** likewise REPORTED off-disk (`torus.rs::evolve_vector`), with the 2:1 ratio being a design choice, not a theorem (per prior audit §1.4).

**Audit outcome W4:** TPE is a well-documented PROPOSED/REPORTED design with a stated falsifier, but **no implementation is present in this repository**. It cannot be demoed or benchmarked from this repo as-is; the encoder evolution, unwrapped-arc accounting, and `attn.linear` (Katharopoulos) microkernel are all unimplemented/off-disk. Recommend either (a) implement a minimal CPU reference in-repo, or (b) scope the submission to the phase-distance/toroidal-attention subset that does exist.

---

## 5. Cross-cutting claim hygiene

- Geometry is finite-feature (`φ`) by the holomorphic lock (`GeometryClaimBoundary.md`): kernels operate on the finite toroidal-phase construction; Lorentz/Hamiltonian work is roadmap only. VERIFIED boundary.
- `tit_bench` is structural/IR, not a Twister validator. VERIFIED boundary.
- No vault/freeze files modified by this audit (read-only). VERIFIED (this pass made no vault edits).

---

## 6. Findings summary (action items)

| ID  | Severity | Workstream | Finding                                                                      |
| --- | -------- | ---------- | ---------------------------------------------------------------------------- |
| F-1 | High     | W2         | 7/19 hypermind tests red — core store invariants unproven on current binary  |
| F-2 | Medium   | W1         | "3.827B dense" teacher not found anywhere; identity unresolved               |
| F-3 | Medium   | W3         | Per-layer toroidal attention gains off-disk; only coarse coherence verified  |
| F-4 | High     | W4         | TPE has no implementation in repo; spec-only                                 |
| F-5 | Low      | W1         | Teacher per-arc share unmeasured (Qwen/DeepSeek/QwQ/Nemotron)                |
| F-6 | Low      | W1         | Nemotron version-name mismatch (on disk 3.5-Lightning vs claimed Nemotron-3) |

**Bottom line.** The repository is rich in design/certification artifacts and a working (partially green) N³ plugin prototype; toroidal attention coherence exists in code but its deeper layers are off-disk; the toroidal phase positional encoder is spec-only with no implementation here. Submission claims should be limited to what is VERIFIED on disk (cuda phase ops, hypermind coherence + 12 passing tests, geometry boundary) plus clearly-labeled REPORTED vault evidence, and must not cite the 7 red store tests or TPE as implemented.

---

_Last updated: 2026-10-28 (audit pass, read-only)._

---

## 7. Audit-of-audit corrections (2026-10-28, second pass — entries, original findings preserved above)

Each item below WITHDRAWS or NARROWS a claim in §§0–6. Nothing above was rewritten.

### C-1 (F-1 WITHDRAWN as stated — the "7 red tests" were parallel-run state bleed)

Re-running `cargo test --manifest-path hypermind-nested-kv/Cargo.toml -- --test-threads=1` gives **19 passed / 0 failed** (VERIFIED this pass). The default parallel run is flaky (observed 12/19 one run, 17/19 the next — the _varying_ failure set is itself evidence of interference, not of seven broken invariants). Root cause [INFERRED — ends at the test code]: every test shares fixed `std::env::temp_dir()` directories (`hm_unit_attend`, `hm_unit_swarm20`, …), so parallel tests ingest into each other's state. Consequence: the W2 outcome in §2 is too harsh. Corrected: the plugin is **19/19 green under serial execution**; the defect is test hygiene (no per-test isolation), severity Low, fix = unique temp dirs or a documented single-threaded gate. §0's baseline table stands as the parallel-runner measurement; it must now be read as "flaky under parallelism," not "7 invariants red."

### C-2 (W4 verdict NARROWED — "no implementation" overstated the gap)

§4 claimed ~10 TPE files and no implementation. Re-check (VERIFIED this pass): there is exactly **one** 101-line spec, `TorodialPhasePositionalEncoder-Spec.md` (note steward spelling "Torodial"), **plus** executable in-repo evidence I missed: `tpe_fep_budget.py` (134-line f64 reference controller for the spiral-out budget law) and `tpe_rope_sidebyside_results.json` (seed-41 measurements: angle-only T0 vs RoPE fidelity ≈ 0.999, triple-channel T1 vs RoPE ≈ 0.46, quadratic-vs-recurrent max-abs ≈ 1e-14). BUT: the side-by-side generator script is absent — only its `.pyc` and results JSON remain (`tpe_rope_sidebyside.py` source missing; `__pycache__` proves it ran on this machine). Corrected labels: TPE in-repo status = spec (PROPOSED) + reference controller (un-re-run here — its "5/5 PASS" stays REPORTED) + orphaned side-by-side measurements (REPORTED, generator missing). Still missing: Rust port, encoder recurrence in-repo (off-disk `torus.rs`). F-4 severity stands (no shippable encoder), but "spec-only" is withdrawn — it is spec + reference + orphaned measurements.

### C-3 (W3 joint-score claim refined — two different mechanisms, one construction)

§3 implied the `¼·d_T²` factor is shared code. VERIFIED this pass: the `>> 2` (÷4) penalty lives in the cuda reference (`ablation_benchmarks/cuda_rust/src/hyperbolic.rs:199-203`, `retrieval_score = h3 − (t2_dist >> 2)`). The hypermind `map.attend` path uses a _different_ weighting (`(coh+2)^1.5` normalized power weights over `toroidal_coherence = cos Δθ_f + cos Δθ_s`). Same phase construction, different score algebra — the C05 sign-resolution (argmin-cost, doc fixed) applies to the cuda reference; the attend path is coherence-reward, not distance-penalty. No contradiction, but they must not be cited interchangeably. Also correcting counts: toroidal unit tests are 4 in `toroidal.rs` (+ toroidal-phase hyperbolic test), not "14 (suite)".

### C-4 (post-audit evidence the report missed — strengthens W1/W2 mechanism, honestly bounded)

`hypermind-nested-kv/MAP-PROOF.md` + `map_proof.py` + `map_proof_results.json` (VERIFIED present; run itself NOT re-executed here): real Qwen3-Embedding-8B weights → map geometry, 40 atoms, mean p@4 = 0.81 vs 0.25 chance. The doc honestly flags gate under-selection (k*=1 everywhere, mean p@k* = 0.50) and stages the mass-coefficient sweep. Label: REPORTED mechanism evidence with stated calibration gap — admissible as "real-weights pipeline discriminates at depth," not as retrieval-quality proof. Also: `DistillationProtocol/model_roster.json` now carries 5 fingerprinted teachers (Glimmer, Qwen3.6-MoE, Qwen3.8-27B, Nemotron-3.5-Lightning, QwQ-32B) + `PER_MODEL_KNOWLEDGE_AND_TRAJECTORY_WALKS_PARAM_PROOF.md` names Qwen3-Embedding-8B as a weight-modes-only teacher — W1 attribution is better than §1 suggests, except the 3.827B-dense gap (F-2) which stands: still no roster entry, receipt, or vault match.

### Corrected findings table (supersedes §6 for F-1 and F-4 only; all other rows stand)

| ID  | Old severity/claim                      | Corrected                                                                                                                                                                         |
| --- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F-1 | High — 7/19 red                         | **Low — test-hygiene defect; 19/19 green serial. Fix isolation, then CI-gate serial.**                                                                                            |
| F-4 | High — TPE spec-only, no implementation | **High (unchanged severity, narrowed scope) — spec + f64 reference + orphaned side-by-side; no Rust port, no in-repo encoder recurrence; side-by-side generator source missing.** |

---

## 8. Next steps (priority order, each independently shippable)

1. **Isolate hypermind tests (½ day, unblocks every green claim).** Give each test a unique temp dir (or `serial_test`), then CI-gate `cargo test -- --test-threads=1` until isolation lands. Converts the whole W2 story from "flaky 12/19" to deterministic 19/19. Highest claim-value per hour.
2. **Rescue or rewrite the side-by-side generator (1 day, unblocks TPE credibility).** Either recover `tpe_rope_sidebyside.py` (decompile the `.pyc` only as a last resort — prefer rewrite from the results schema + spec §3 falsifier) and re-run to reproduce `tpe_rope_sidebyside_results.json` byte-shape, or mark the JSON orphaned and withdraw its fidelity figures from submission claims. In parallel, re-run `python3 tpe_fep_budget.py` and receipt the P6 acceptances — converts REPORTED → VERIFIED.
3. **Keep MAP-PROOF honest at scale (staged RUN-03, already preregistered in-doc).** 400+ atoms, instruction-prefixed embeddings, mass-coefficient sweep → p@k\* at scale. Do not quote p@4 = 0.81 beyond "40-atom petri dish" until then.
4. **Resolve Q-1 before any teacher-share figure ships.** The 3.827B-dense identity is still open; roster + receipts now cover the other five. One steward answer closes F-2/F-5.
5. **Judge gate in the mint path (from `DistillationProtocol/AUDIT.md` finding 4).** Blocks `p_useful` calibration and the full `receipt_ok` validity condition. Independent of steps 1–4.
6. **Rust TPE port only after step 2 is green.** Port the f64 reference (Q32_31, integer audit C6) with the P6 acceptances as the gate — the spec already stages exactly this. Do not start before the Python reference is reproduced in-repo.

**Submission-claim guidance (updated).** May cite as VERIFIED on disk: cuda phase ops + `retrieval_score` ÷4 penalty; hypermind coherence-weighted attend + 19/19 serial green (state the serial condition); TPE spec + f64 reference-controller presence. Must still label REPORTED: vault encoder conformance, all teacher-share numbers, map-proof retrieval quality beyond 40 atoms. Must not cite: the orphaned side-by-side fidelity figures until step 2 reproduces them.
