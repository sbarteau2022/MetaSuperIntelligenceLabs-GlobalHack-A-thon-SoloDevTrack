# Certification Audit — ReasoningDistillationMapCarving.md (Math & Logic)

**Date.** 2026-10-27. **Auditor.** Muse (contributor lane).
**Scope.** `MasterDesignDocument/C06-Frontier-Push/ReasoningDistillationMapCarving.md` as read 2026-10-27 (§1–§10, incl. A39 + claims CC-010–CC-014). Companion ledger: `C06-Frontier-Push/Failures.md` (F-C06-01–F-C06-06), `Claims_Register.md`, `Design.md` CORR-C06-01.
**Standards.** Documentation-standards: every claim below carries a label; corrections are entries (none of the audited file's text was modified). Research-evidence: primary sources preferred; Hub metadata is a lead, not proof.
**Verdict.** CONDITIONALLY CERTIFIED as a PROPOSED methodology specification — coherent structure, honest scoping (§8.7, §9, §10), ablation-shaped (A39). NOT certified as verified system, NOT certified for `required` status. **7 blocking items** must clear before A39 runs or any CC-claim advances. Details per-item below; each item ends with disposition.

---

## 0. What this audit does and does not cover

- COVERS [VERIFIED — full file read]: Φ operator (§3), dissolution Φ⁻¹ (§3.2), M_T formalization (§4), necessity tables (§5), λ_map (§6), resource accounting (§7), A39 (§8), claims (§9), stated limits (§10).
- DOES NOT COVER (vault unreachable from this workspace): `monster-hybrid-server/design/global_active_mind.atoma_map` contents, the 4 minds (GLIMMER / Qwen3.6MoE / 3.827B dense / DeepSeek safetensors / QwQ-32B math / Nemotron3-30B-MoE) distillation identities, the 0-2+cell8 Jacobian, trajectory data, dreamstate/idletime mechanism, toroidal-attention invocation code. All task items (1)–(4) depending on vault contents remain OPEN pending mount. No vault claim is certified here.
- Primary-source grounding used [REPORTED — HF paper.md excerpts read 2026-10-27, first ~15k chars each]: RoPE (Su et al., arXiv:2104.09864), Hyperbolic Neural Networks (Ganea et al., arXiv:1805.09112, Möbius §§2.3–2.4), Poincaré Embeddings (Nickel & Kiela, arXiv:1705.08039, §§2–3). Hypformer (Lorentz transformer, arXiv:2407.01290) is title+summary only — NOT read, cited as lead only.
- Hub grounding used [REPORTED — Hub search 2026-10-27]: `Qwen/QwQ-32B` exists (Apache-2.0, base `Qwen2.5-32B`); `nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B-BF16` exists as MoE base. GLIMMER / Qwen3.6MoE / 3.827B-dense / DeepSeek-safetensors identities NOT FOUND in workspace or probes — distillation composition uncertifiable until vault mounted.

---

## 1. Geometry equations restated by the audited doc — correctness check

The audited doc inherits (not re-derives) C05/C35 geometry. Restated here from C05/C35 as read, with audit status:

1. **Lorentz model** `H³ = {x : ⟨x,x⟩_L = 1, x₀ > 0}`, `⟨x,y⟩_L = x₀y₀ − x₁y₁ − x₂y₂ − x₃y₃`, `d = arcosh⟨x,y⟩_L`. Status: CERTIFIED CORRECT [VERIFIED — standard hyperboloid model; matches Ganea §2.1 Riemannian apparatus and C35 §1].
2. **Stereographic pair** `π(x) = (x₁,x₂,x₃)/(x₀+1)`, `π⁻¹(p) = ((1+‖p‖²)/(1−‖p‖²), 2p/(1−‖p‖²))`. Status: CERTIFIED CORRECT [VERIFIED — standard; C35 gate `< 4.4e-9` is a REPORTED implementation measurement, not re-verified here].
3. **Picard action** `γ·(z,w) = ((az+b)/(cz+d), w/|cz+d|²)`, `Γ = PSL(2,ℤ[i])`. Status: CERTIFIED CORRECT as formula [VERIFIED — standard upper-half-space action]; idempotence/convergence behavior is REPORTED (vault code), not certified here.
4. **Toroidal phases** `θ_f = arg(x₁+ix₂)`, `θ_s = arg(√(x₁²+x₂²)+ix₃)`, `d_T² = (1−cos Δθ_f)+(1−cos Δθ_s)`. Status: CERTIFIED WELL-FORMED as definitions [VERIFIED — well-defined except on measure-zero branch cuts (x₁=x₂=0; √(…)=x₃=0); implementation must fix branch convention — minor, see M-1].
5. **Joint score** `score = ⟨q,a⟩_L + ¼·d_T²`. Status: **BLOCKING — sign/direction ambiguous, see B-1.**
6. **TwisterAttention** `α_i = exp(⟨q,k_i⟩_L/τ)/Σ exp(...)` over k-NN. Status: **BLOCKING — inverted weighting, see B-2** (same root cause as B-1).
7. **VP-tree retrieval** median-split on `d_H³`, prune by min-possible-score. Status: CERTIFIED STANDARD ALGORITHM [VERIFIED — textbook]; `O(log n)` average holds only under bounded expansion (already recorded C05.12) — the audited doc's §4 efficiency sentence must carry that qualifier, see B-3.
8. **Eviction** victim = max unpinned `x₀`. Status: CERTIFIED AS STATED POLICY [VERIFIED — matches C05/C35]; geometric-necessity conditional (insertion-by-salience) already recorded C05.12 — inherited, not re-litigated.
9. **Möbius apparatus** (⊕_c, ⊗_c, `d_c`, exp/log, parallel transport) used by citation. Status: CERTIFIED CORRECT AS CITED [VERIFIED — Ganea et al. §§2.3–2.4 excerpts read; formulas (6)–(9) match].

---

## 2. Blocking items (must clear before A39 or any claim advance)

### B-1. Score direction: similarity vs cost is undefined (math bug class)

**Finding.** [VERIFIED — C05 §4 + C35 §4 + carving §3 step 5 read] `score = ⟨q,a⟩_L + ¼·d_T²` grows with distance on BOTH terms (⟨,⟩_L = cosh d ≥ 1). If higher score means more relevant (retrieval takes top-k by score, §3 step 6 "top-k basins by score"), the ranking prefers FARTHER atoms — inverted. If score is a cost (lower better), step 6 must take bottom-k and the necessity `score(b)` in §5 (higher = more necessary) contradicts it. The audited doc uses score both for ranking-up (step 6, §5.2 relevance `exp(−d/τ)` is correctly oriented) and inherits the C05 additive form — the two orientations coexist without reconciliation.
**Evidence.** C05 §4 formula; carving §3.1 step 5 (`relevance = exp(−d/τ)` — correct decay) vs C05/C35 joint score (growth). Both read directly.
**Consequence.** [INFERRED — ends at the formulas] Any retrieval/threshold behavior (top-k, ρ_cutoff, α_T) is undefined up to sign until the convention is fixed.
**Required correction (entry, not edit).** [PROPOSED] Freeze ONE convention, e.g.: `cost(q,a) = d_H(q,a) + ¼·d_T²` (lower better; retrieve argmin), `relevance = exp(−cost/τ)` (higher better; rank argmax), and rewrite every selection rule (`top-k`, `≤ R`, `≥ α`, `geo_gate`) in cost-or-relevance terms consistently. Until the correction entry lands, B-1 blocks A39 (J2/J3/J5 all depend on which atoms the map contains).

### B-2. TwisterAttention weights attend to distance, not proximity (math bug class)

**Finding.** [VERIFIED — C05 §6 read] `α_i ∝ exp(+⟨q,k_i⟩_L/τ)`. Since `⟨q,k⟩_L = cosh(d) ≥ 1` and strictly increasing in `d`, the soft-max puts the LARGEST mass on the FARTHEST of the k neighbors — the opposite of attention. Standard hyperbolic attention (cf. Hypformer lead: Lorentz linear attention; Ganea GRU gating via Möbius) uses decaying kernels `exp(−d/τ)` or `exp(−⟨,⟩/τ)`-with-sign-fixed. The carving doc's own `relevance = exp(−d_H/τ)` (§3.1) confirms the intended direction.
**Evidence.** C05 §6 formula read; Ganea §§2.3–2.4 distance growth `d_c` read (distance grows toward boundary — far means dissimilar).
**Consequence.** [INFERRED] With the formula as written, depth-gated recall (C06.4) and the C06.15/A36 fusion built on it optimize toward the wrong neighbors. This also invalidates any A1/A33/A39 latency-quality reading that uses this kernel.
**Required correction (entry, not edit).** [PROPOSED] Replace with `α_i ∝ exp(−d_H(q,k_i)/τ)` (or equivalently `exp(−arcosh⟨,⟩/τ)`), re-derive the C03 `d*` gate with the corrected kernel, and re-issue affected preregs (A1, A33, A36) as new versions with new hashes (ADR-19/22 — no silent fix).

### B-3. Efficiency claim drops its own qualifier (logic)

**Finding.** [VERIFIED — carving §4 + §7 read] "Search space is `O(log|M_T|)` instead of `O(log|M_G|)`" and §7 "sublinear in N" are stated without the bounded-expansion / balanced-tree qualifier that C05.12 already requires, and without distinguishing retrieval (`O(log)`) from reasoning (`O(k)` attend + downstream tokens).
**Evidence.** Carving §4 sentence + §7 claim table read; C05.12 qualifier read.
**Consequence.** [INFERRED] As written the claim is FALSIFIABLE-by-construction on adversarial clustering (degenerate VP-tree → `O(n)`) and on concurrent-N peak (N simultaneous transient working sets scale as `O(N)`, however small the constant).
**Required correction (entry, not edit).** [PROPOSED] Restate §7 as: persistent cost `O(|A|)` (one map) + transient peak `O(N·k·log|M_T|)` (linear in N with small constants; sublinear claim WITHDRAWN — see also B-4); carry the C05.12 non-adversarial qualifier into §4 and A39. CC-014 (currently INFERRED) must be demoted to PROPOSED-with-correction or withdrawn by register entry.

### B-4. "Sublinear in N" is false as stated (math)

**Finding.** [VERIFIED — §7 table read] Fibers are transient per-agent views: peak resource = baseline + Σ_{i=1..N} fiber_i. A sum of N positive bounded terms is `Θ(N)`, not sublinear, regardless of how small each term is (`k ≤ 8` bounds the constant, not the order). Dissolution frees memory after use (good — bounds integral over time, not peak).
**Evidence.** §7 rows read (`Fiber views F_T: N × O(k log|M_T|) transient` — the table itself states linear-in-N transient).
**Consequence.** The §7 "Claim: peak resource growth is sublinear in N" contradicts its own table. [INFERRED — ends at the table] Correct statement: persistent `O(1)` in N (one weight set, one map — the real win, I1 holds structurally), transient peak `O(N)` with small constants + admission control (C30.9 backpressure) as the enforcement arm.
**Required correction (entry, not edit).** [PROPOSED] Withdraw "sublinear" (register entry against CC-014); replace with persistent-O(1)/transient-O(N) + admission rule: N admitted iff projected peak (baseline + Σ fiber) stays under the C21 ceiling with headroom (ties to my C34.7 admission rule, same class). A39 secondary judge `weight_copies = 1` already covers I1; add a peak-VRAM secondary gate per arm.

### B-5. Dissolution integrity uses XOR of hashes (security/logic bug)

**Finding.** [VERIFIED — §3.2 step 4 read] "Recompute Atlas Merkle fingerprint; it must equal `fingerprint_pre XOR intended_write_set_hash`." XOR of digests is linear, malleable (`a⊕b = c` has many preimages; order-insensitive; leaks via differential analysis), and is NOT a Merkle update. An attacker (or a bug) can craft a distinct write-set with the same XOR and pass the check — the exact tamper-evidence property ADR-15 exists to provide.
**Evidence.** §3.2 step 4 text read verbatim.
**Consequence.** [INFERRED] Step 4 as written voids the C06.12(d) "dissolution leaves the map consistent" judge: a passing check does not imply a correct merge.
**Required correction (entry, not edit).** [PROPOSED] Replace with a proper accumulator, e.g. `expected = SHA256(fingerprint_pre ‖ canonical_write_set_hash)` with canonical ordering, or recompute the Merkle root over the affected basin range and compare; on mismatch follow existing step 5 (quarantine + breaker). Re-run any prior dissolution test under the corrected check as a new version (no retroactive pass).

### B-6. Basin centroid / Fréchet mean undefined on H³ (math gap)

**Finding.** [VERIFIED — §3 steps 7–8 read] `centroid(B_sel)` for one basin and worse for a SET of basins is used in a distance predicate `d_H(x_a, centroid(B_sel)) ≤ R`. The Euclidean mean of Lorentz points is off-manifold (violates `⟨x,x⟩ = 1`); the correct object is the Fréchet/Karcher mean `argmin_y Σ d_H²(y,·)` (existence/uniqueness needs curvature + radius conditions) or a defined proxy (e.g. medoid atom, Lorentzian centroid projected back).
**Evidence.** Steps 7–8 text read; no centroid definition present in §2–§5.
**Consequence.** [INFERRED] Atom selection (§3 step 8, hence A39 Arm B contents, hence J2–J6) is not reproducible until the centroid is defined — two implementations choose different atoms.
**Required correction (entry, not edit).** [PROPOSED] Freeze ONE definition (recommendation: medoid of basin member atoms under `d_H`, ties by lowest atom id — deterministic, on-manifold, no optimization loop), name the multi-basin aggregation (per-basin radii instead of one centroid+R, or Fréchet with stated solver + seed), and pin all of it in the A39 run manifest.

### B-7. A39 is underpowered and arithmetically inconsistent as specified (logic)

**Finding.** [VERIFIED — §5.1 + §8.3 read] Six task classes × "8 per primary class" = 48, not "N ≥ 40". No power analysis justifies detecting J1's +10pt gain (or J2 0.80× / J3 0.70× / J4 0.50× ratios) at N=40/48 with paired tests; and §8.6(6) voids Arm B runs for J5 only when Arm B gets >10% more tokens, while J1–J4/J6 may still be evaluated "with a recorded caveat" — a token confound that can explain quality/efficiency deltas is admitted into the primary verdict path.
**Evidence.** §5.1 class rows (6) + §8.3 "N ≥ 40 … 8 per primary class" + §8.6(6) text read.
**Consequence.** [INFERRED] As specified, a passing J1–J4 could be attributable to extra context rather than correct carving (the very confound §8.3 claims to eliminate by pairing).
**Required correction (entry, not edit).** [PROPOSED] (i) Fix N (48 paired runs across 6 classes, or name the 5 primary classes for N=40); (ii) attach a power calculation per judge to the prereg (C09.9 sample-size plan — interim Lan-DeMets already required by house style); (iii) tighten §8.6(6): token-parity violation voids the PAIR for all judges (not J5 only), or pre-register a token-matched sensitivity arm. No A39 run until the v1.2 prereg with these three carries a filed hash.

---

## 3. Major gaps (must define before A39 manifest; individually minor, jointly blocking)

| # | Gap | Status | Required [PROPOSED] |
|---|---|---|---|
| M-1 | Branch cuts for `θ_f, θ_s` (x₁=x₂=0; √ term +x₃ = 0) | Definitions well-formed a.e.; implementation needs convention [VERIFIED — formulas read] | Fix `arg(0,0) := 0` + log the convention in the run manifest; phase_dist uses wrapped deltas (already `1−cos`, good). |
| M-2 | Free parameters without values: `τ_b`, `ρ_cutoff`, `δ_phase`, `α` per-sil–domain beyond code/other, `B_map_default`, `c`-normalization maxima, `e_T/e_b` encoders, basin-level `salience(b)` aggregation | All absent [VERIFIED — searched §2–§6] | Table every constant with value + owning prereg + amendment rule (§6.2 already models the right pattern — extend it to §5). Unvalued run = unrepeatable run. |
| M-3 | `k` reuse: TwisterAttention neighbor-count `k*` (§3 step 6) doubles as basin-count | Type mismatch [VERIFIED — step 6 text] | Split into `k_nn` (retrieval depth) and `k_b` (basin count, monotone in the same gate signal) with stated coupling; or prove the identification. |
| M-4 | `relevance/salience/geo_gate` product (§3 step 5): `salience(b)` undefined at basin level; hard `geo_gate` makes `softmax(score)` in §6.2's `h` a truncated distribution | Undefined + interaction [VERIFIED] | Define `salience(b)` (recommend: max member-atom salience, pinned floor) and define `h` over the post-gate candidate set with stated cardinality handling. |
| M-5 | `interf_safety = 1 − I(T_lane,b_lane)` with `I` an unbounded geodesic distance | Out-of-[0,1] possible [VERIFIED — §5.2 + C06.5 read] | Normalize, e.g. `exp(−I/τ_I)` or `1/(1+I)`, and re-check the §5.2 weight sum/threshold calibration after the change (weights currently sum to 1.0 — good — recalibrate θ_N = 0.50 post-change). |
| M-6 | `fold(A_sel into basins)` (§3 step 9) has no operator definition | Missing [VERIFIED] | Define as receipt-linked grouping (atoms keep ids; basin envelope = geodesic convex hull proxy or stated approximation) — F-C06-01 already forbids claiming geometric reconstruction from this step; the Capsule (I3) carries exact state, and the doc honors that. Keep it that way. |
| M-7 | Truncation "lowest-salience free atoms first" (§3 step 11) vs necessity precision secondary (≥0.60) | Interaction unproven [VERIFIED] | Pre-register the truncation order as part of the digest algorithm; necessity-precision < 0.60 on the tuning split blocks the run (same gate as §8.6(3)). |
| M-8 | `λ_map` coefficients/θ_N/α_T/necessity weights are uncalibrated constants | Acknowledged by doc as proposed [VERIFIED — §10 read] | No change to the doc's honesty; but A39 must treat them as preregistered constants with amendment rule (doc already requires this for §6.2 — extend to §5.2/§5.3). Sensitivity arm for `λ_max` on J5 already implied by §8.6(2) — make it explicit. |
| M-9 | Necessity-table classifier (`classify(T)`, confidence `< τ_class`) has no classifier | Open [VERIFIED — §3.1 step 2] | Name the classifier + `τ_class` value + fallback accounting (`C_general` rate reported as a secondary; if fallback > 30% of runs, scoping claim is void — the map is just "everything"). |
| M-10 | Phase coherence across carving (`T² weight = 0.25` inherited) | Provenance already demanded (my C35.12) [VERIFIED — C35 read] | A33 sensitivity arm (0.0/0.25/0.5/1.0) is a precondition for trusting `phase_dist ≤ δ_phase` filtering; record dependency A33 → A39 in the prereg. |

---

## 4. What the doc gets right (certified strengths — per-item disposition)

1. **Capsule/map separation (I3).** [VERIFIED — §2 + §5.4 read] Exact critical state travels by hash-validated capsule, approximate recall by carved map. This is the correct response to F-C06-01/CORR-C06-01 (no geometric reconstruction claim). CERTIFIED SOUND — hold as invariant.
2. **Necessity-first framing (`I_nec`, §5.4).** "Nothing not in I_nec may be assumed" is the right closed-world rule for a subagent context. CERTIFIED SOUND as policy; calibration of its components is M-8, not a structural flaw.
3. **Interference-aware scoping (`interf_safety`, I4 pins).** Deprioritizing high-interference foreign-lane basins while never dropping pinned content composes C06.5/C06.8/C30 correctly. CERTIFIED SOUND in structure; M-5 fixes the normalization.
4. **Budget guardrails (§6.3).** Floor/ceiling/pressure-term reasoning is coherent; `λ_max = 0.35` + never-truncate-pinned + zero-map-baseline (A39 Arm A) is a well-formed experimental control. CERTIFIED SOUND as design; constants are M-8.
5. **A39 scoping honesty (§8.7) + open ledger (§9–§10, Failures F-C06-01–F-C06-06).** Explicitly not testing speed/weights/capsule/fine-tuning; all claims PROPOSED/INFERRED with no VERIFIED overreach. CERTIFIED EXEMPLARY per C03.2/C33 — this is how PROPOSED work should read.
6. **Dissolution void-on-mismatch (§3.2 steps 5–6).** Quarantine + breaker + full-void (not partial apply) is the correct fail-closed semantic. CERTIFIED SOUND — contingent on B-5's accumulator fix.
7. **Paired design + frozen template + no-LLM-judging-primary (A39).** Eliminates task-difficulty and prompt-engineering confounds by construction. CERTIFIED SOUND — contingent on B-7's token-parity fix.

---

## 5. Dreamstate/idletime novelty (task item 3) — formalization direction, gated

**Steward's requirement.** The map must gain, in idle/dreamstate, the ability to CREATE connections from trajectories — otherwise the system only walks walked paths and "refuses" the unwalked, which kills real novelty.

**Formalization (PROPOSED — not certified, offered as the prereg-shaped seed for the vault pass).** [PROPOSED — my construction; authority: F-C06-01 (reconstruction ≠ creation), C06.2/C06.7 (self-tuning frontier), C22 runbook class]

1. **Separate creation from recall, architecturally.** Recall = constrained walk on existing geometry (receipted, interference-checked). Dreamstate = a HOLE-SUBSTRATE job (C25: seatbelt + budget + receipt) that PROPOSES new edges/basin-bridges and NEVER writes them to `M_G` directly — proposals land in a quarantine collection (my C26.7 UNVETTED tier) with provenance `(dream_cycle, parent_trajectories, operator_params, seed)`.
2. **Creation operator (sketch).** From two unconnected trajectories `γ₁, γ₂` (or basin envelopes), propose bridge `β` iff: (a) endpoint neighborhoods are within `ρ_dream` under `d_H` (near-miss, not leap); (b) phase coherence `d_T² ≤ δ_dream` (same phase gate as recall, tighter); (c) a surrogate judge (deterministic rubric or held-out task probe, NEVER the production judge alone) scores the bridged neighborhood above baseline on a probe set. All three thresholds preregistered; all proposals receipted with parent hashes.
3. **Admission to the map is promotion, not writing.** A proposal enters `M_G` only via the EXISTING merge path: Φ⁻¹ validation + receipt chain + (human/explicit approval where effect level demands it, C29) + interference re-check (C06.5). Dreamstate proposes; governance disposes. This preserves append-only truth (C03.3) and answers the "model refuses" dealbreaker WITHOUT breaking verifiability: refusal of unwalked paths during recall is CORRECT (fail-closed); creation happens in the dream hole, promotion happens through the gate.
4. **Anti-hallucination bound.** Cap dream proposals per cycle (budget, C25.8 class), cap promotion rate (e.g. ≤ k promotions/cycle), and monitor judge calibration (my C06.16/A37) over the surrogate judge — a drifted dream-judge manufacturing "novelty" is the precise failure C06.16 exists to catch.
5. **Falsifiability.** The dream mechanism earns its keep via a dedicated ablation (call it A40, PROPOSED): promoted bridges must improve held-out task success or retrieval precision vs a no-dream control at equal map budget; else dreamstate is ornament and ships OFF by default (C13.1).

**What this does NOT do.** It does not certify any vault trajectory/Jacobian content (unread), does not modify the audited file, and does not claim novelty generation works — it states the only architecture under which novelty generation would be ADMISSIBLE in this design (hole + quarantine + promotion gate). Vault pass must instantiate `ρ_dream/δ_dream`, the surrogate judge, and the 0-2+cell8 Jacobian role (likely: sensitivity of trajectory endpoints to initial conditions — useful as the near-miss detector's linearization; PROPOSED interpretation only, pending vault read).

---

## 6. Toroidal phase encoding + toroidal attention (task item 4) — source status

- **RoPE (Su et al. 2021).** [REPORTED — paper.md §§1–3 read] Rotation-matrix relative encoding; decay with relative distance; linear-attention compatibility. RELEVANCE [INFERRED]: RoPE is the Euclidean proof that rotation carries relative position inside attention — the toroidal `T²` phase channel is the hyperbolic analogue (phase rotations on `S¹×S¹` instead of token offsets). Cite as precedent, not as proof.
- **HNN (Ganea et al. 2018).** [REPORTED — §§1–2.4 read] Möbius ⊕_c/⊗_c, `d_c`, exp/log, parallel transport, gyroline element. RELEVANCE [VERIFIED by reading]: the audited stack's transport/projection language (Brigade `P_γ`, fiber dissolution) is this apparatus applied to weights/views. Use §§2.4 identities for any Jacobian work (D exp, D transport) in the vault pass.
- **Poincaré embeddings (Nickel & Kiela 2017).** [REPORTED — §§1–3 read] Hierarchy-via-geometry, `d(u,v)` Eq.(1), Riemannian optimization. RELEVANCE [INFERRED]: justifies radial salience (`1/x₀`) as hierarchy encoding — the eviction/salience policy's theoretical home.
- **Hypformer (2024/25, Lorentz transformer).** [REPORTED — title/summary only] Fully-hyperbolic Transformer with linear self-attention. RELEVANCE: closest published relative of TwisterAttention; vault pass should compare kernels term-by-term (esp. the B-2 sign question — check what Hypformer uses).
- **Invocation-gated dynamic depth ("allots only the attention necessary … depth/effort the goal requires").** Maps to C03 `d*` gate + C05 §6 + carving §3 step 6 + my C06.15 (endogenous k-draft). Status: PROPOSED objective with B-2/B-3/M-3 corrections pending. The active-inference framing (C03 free-energy contract) is the design's stated foundation [VERIFIED — C03 read]; its instantiation as code is vault-side, uncertified here.

---

## 7. Required register entries (to be appended by the owning agent — not applied here)

1. Demote CC-014 to PROPOSED-with-correction (B-3/B-4) or withdraw; block dependents.
2. File A39-v1.2 prereg (B-7: N fixed, power calc, token-parity void rule) with new hash; v1.1 stays in history (ADR-19).
3. File the B-1/B-2 sign-convention correction + affected-prereg re-issues (A1, A33, A36) as new versions.
4. File the B-5 accumulator correction; re-run dissolution checks under it.
5. Define M-2/M-6 constants + operators in the A39 run manifest before any run.
6. Open A40 (dreamstate promotion) as PROPOSED with the §5 sketch as its seed text.

---

## 8. What was NOT checked (explicit)

- Vault map bytes/trajectories/equations beyond in-workspace specs; 0-2+cell8 Jacobian (no definition found in workspace — term originates steward-side; needs vault definition before any certification).
- The 4-mind distillation composition (only QwQ-32B + Nemotron-3-MoE base existence REPORTED; GLIMMER/Qwen3.6MoE/dense/DeepSeek roles unknown).
- `monster-hybrid-server` code (lorentz.rs/geometric.rs/picard.rs), the 900-pair/4.4e-9 measurement, 13/13 and orchestrator test counts (all REPORTED per my C35.11).
- T² weight 0.25 sweep (my C35.12 stands).
- CORR-C06-02 content (cited by §3 step 12; not located in this pass — verify before relying on receipt-termination claim).
- Full RoPE/HNN/Poincaré proofs beyond excerpts read (sufficient for direction, not for line-by-line proof audit).

**Bottom line.** The co-agent's methodology is CERTIFIED as honest PROPOSED work with a sound skeleton (capsule-exactness, necessity closure, budget guardrails, paired ablation, open ledger) and SEVEN precisely-located defects that must be corrected by appended entries before it can run or earn. The skeleton is worth keeping; the defects are fixable without redesign — except B-4's scaling sentence, which must be withdrawn as written.
