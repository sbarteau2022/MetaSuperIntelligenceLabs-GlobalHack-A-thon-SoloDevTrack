# Vault Map Formalization: Geometry, Distillation, Dreamstate & Toroidal Attention

**Date.** 2026-10-27. **Author.** Muse (contributor lane).
**Scope.** `monster-hybrid-server` vault as read from disk: `core_lm/src/geometry/*`, `core_lm/src/cache/geometric.rs`, `orchestrator/src/toroidal_attention.rs`, `immune_system/implementation/src/{continuous_active_mind,map_native,distill_engine,compliance,silo,holonomy,superposition}.rs` (targeted reads), `design/{CARVED_SWARM_SPEC,QUALIFICATION_GATES,superpositionholding,Mathematical Certification #1-3,QUOTIENT_PROOF}.md`, map bytes of `global_active_mind.atoma_map` (+WAL, held-crystallizations log, outcomes ledger, glimmer report).
**Standards.** Documentation-standards (every claim labeled); research-evidence (primary sources preferred; Hub metadata = lead); C03.3 (this file only appends understanding — it modifies no vault file and no prior certification; corrections to the co-agent audit live in §8 as entries referencing it, not edits).
**Status.** PROPOSED formalization. Vault-read portions are VERIFIED-against-source; measurements the vault reports are REPORTED (test suites not re-run in this pass); design judgments are INFERRED (premises stated) or PROPOSED (falsifiability attached).

---

## 0. Map census (measured by me, just now)

| Artifact | Bytes / finding | Label |
|---|---|---|
| `global_active_mind.atoma_map` magic | `ATOMA_MAP_V1` (12 bytes, offset 0) | VERIFIED (xxd read) |
| SHA-256 (computed this pass) | `fbadb14d11f881f97b3f80b6453e1787c7efb4b3fc2dc19516f359404551da1c` | VERIFIED (shasum) |
| Size | 603,865,908 (≈604 MB); backup `pre-nemo-distilled` 577,791,270 (≈578 MB); delta **+26,074,638 bytes (+4.5%)** = the Nemo-distillation landing, 2026-10-26 | VERIFIED (ls + cmp: files differ at char 21 = header count field — additive growth, consistent with append-only) |
| Unified-mind label in bytes | `Qwen3.8-27B+Muse-Glimmer-UnifiedMind` (exactly once, offset ~44, followed by 32-byte digest) | VERIFIED (strings + header parse) |
| Teacher tags in bytes | `DistilledOpenSourceTeacher` ×16 occurrences; `QWQ` fragments ×2; `ATOMA_OP` ×16,830; `Muse-Glimmer-q4km-gs128-MLX` ×1 | VERIFIED (strings census) |
| Audit ledger claim | 218,908 0-cells / 216,432 1-cells, SHA `2db521b4…` | REPORTED (`produced_work/MONSTER_SERVER_AUTONOMOUS_AUDIT_REPORT.md`; NOT re-verified — my computed SHA differs because the map has grown since that receipt; **receipt-staleness flagged, see §8 M-11**) |
| WAL | 28,276 bytes; outcomes ledger present (binary); held-crystallizations log = `Tier2SuperpositionOperator` Q/A rows | VERIFIED (read; outcomes content is binary-protobuf class, not parsed) |
| Sibling vaults | `glimmer_100pc` (36 MB), `qwen_100pc` (41 MB), `unified_one_mind` (128 MB), `live_brain` (38 MB), `glimmer_token_trajectories` (998 MB + WAL) | VERIFIED (listing) |

---

## 1. The geometry, equation by equation, with proof status

Base manifold (as implemented): **H³ (Lorentz, canonical compute) × T² (toroidal phase) / Γ₁₂ (partially specified — see §1.7)**. Q32.31 fixed point throughout.

### 1.1 Lorentz hyperboloid (CERTIFIED — code read, proof shown)

State: `x ∈ ℝ⁴`, constraint `⟨x,x⟩_L = x₀²−x₁²−x₂²−x₃² = 1`, `x₀ > 0` (`lorentz.rs`).
Inner product in `i128` wide accumulator, `>> 31`, saturate to `i64` (FLAG-02). `validate()`: `x₀ > 0`, `|⟨x,x⟩−1| ≤ 0.1`.

**Proof the model is what it claims (shown here, not merely cited):**
`from_f64` projects arbitrary spatial input by `x₀ = √(1+‖x_spatial‖²)` (tolerance path preserves caller x₀ within 0.05). Then `x₀² − ‖x_spatial‖² = 1` **by construction** — every constructed vector satisfies the constraint up to Q32.31 rounding, and `validate()` checks the residual. The construction is a projection onto the manifold, not an approximation of one. Preservation under boosts: `boost_rapidity` recomputes `x₀ = √(1+‖x'‖²)` after the boost step, so the output is re-projected every hop — drift cannot accumulate across hops by construction (each hop's output is a fresh projection). The 10-step monotonicity + `validate().is_ok()` test (`test_boost_rapidity_unbounded_stays_on_hyperboloid`) exercises exactly this invariant.

**Finding F-1 (minor, monotonic-but-discontinuous fold).** [VERIFIED — `boost_rapidity` read] The unbounded-rapidity fold `η_eff = 8.5 + ln(1+|η|−8.5)` for `|η| > 11.5` maps the boundary `11.5 ↦ 9.886` — a **jump of −1.61**, not a smooth fold ("smoothly" in the comment is inaccurate). Monotonicity is preserved (`dη_eff/dη = 1/(1+|η|−8.5) > 0`), and the existing test only reaches `η = 8.5`, so the fold region is **untested**. Disposition: continuity-sensitive consumers (Jacobian ray perturbations, §5) must treat `η = 11.5` as a known kink; add a fold-region test or a `τ`-continuity gate. Not blocking — recorded.

### 1.2 Stereographic isometry + optical form (CERTIFIED — proof shown)

`π(x) = x/(x₀+1)`, `π⁻¹(p) = ((1+r²)/(1−r²), 2p/(1−r²))`. Optical form (`dhng2.html` via spec §2.1): `n(r) = 2/(1−r²)`, `x₀ = n−1`, `x = n·p`.

**Proof (algebra, verified against the stated formulas):**
`n − 1 = 2/(1−r²) − 1 = (1+r²)/(1−r²) = x₀` ✓. `n·p = 2p/(1−r²) = x` ✓.
Manifold check: `x₀² − ‖x‖² = [(1+r²)² − 4r²]/(1−r²)² = (1−r²)²/(1−r²)² = 1` ✓.
So the optical map is **exactly** the stereographic projection, and every projected point satisfies the hyperboloid constraint identically. The `4.4e-9` figure is the *fixed-point round-trip error* (`π⁻¹(π(x))` in Q32.31 vs `x`), not a model error — the model is exact, the arithmetic is approximate. Status: equations CERTIFIED; `4.4e-9`/900-pair figures REPORTED (vault README + spec; suites not re-run here).

### 1.3 Picard reduction (CERTIFIED structure — code read; one design note)

`map_to_fundamental_domain`: `x₀ ≤ 8` → 32-step fixed-point UHS iteration (Gaussian-integer strip + inversion when `|τ|² < 1`); else exact-`f64` path with cancellation-free `x₀+x₃`; then strip reduction + cusp rule `w > 64 ↦ 1+ln(w−1)`.

**Proof sketch of idempotence (the gate the vault tests):** strip reduction moves `z` into `[-½,½]²` (fixed point of further shifts); inversion only fires when `|τ|² < 1`, and after inversion + strip the point satisfies `|τ|² ≥ 1` or exits via `break`; the cusp fold maps into `[1,64]` where it is a fixed point of the rule. So a second application finds no applicable move — `reduce² = reduce` follows from *each move's output being outside its own trigger set*. The vault test checks this at radii {25, 250, 5000, 50000} + origin-fixedness.

**Design note F-2.** [INFERRED — ends at the code] The `w > 64` cusp fold (`64 ↦ ~5.14`) is discontinuous BY DESIGN (quotient identification, not a continuous map) — legitimate for a fundamental-domain map, but it means the image is a *collar*, and strict "maps into the Picard fundamental domain" language overstates it. The vault's own QUOTIENT_PROOF already concedes the analogous boundary on the H³ side (see §1.7). No action beyond precise language.

### 1.4 Toroidal encoder + Kuramoto 2:1 lock (CERTIFIED — code read, lock proved)

`TorPhaseEncoder::{new: ω_f=0.1, ω_s=0.05, k_f=0.3, k_s=0.15}`, `evolve`: `step = ω + K·sin(diff) + drive`, `ψ = θ_f − 2θ_s`, amplitude accumulates `arc + drive_mod` monotonically; `radius_for_rotations(w) = r_min + span·(0.35w + 0.65·ln(1+w))` monotone in `w`; `rapidity = 0.22 + 0.38w + 0.28·amp + 0.20·ln(1+rms)`.

**Proof of the ψ* invariant (the "address that survives"):** under exact 2:1 locking (`Δθ_f = 2δ`, `Δθ_s = δ` per step), `Δψ = 2δ − 2·δ = 0`. Under angle wrapping, equality holds mod 2π — which is the correct statement on T² (ψ lives on S¹, so mod-2π invariance IS invariance). The vault proves it computationally over 10,000 pairs including wrap arithmetic (QUOTIENT_PROOF §2, REPORTED). The `evolve_vector` test asserts monotonic radius/amplitude/rapidity over 8 turns + zero-discarded-dimensions sensitivity (index 127 flips the trajectory). Status: mechanism CERTIFIED as implemented; the 2:1 ratio is a *choice* (matches the `θ_f/θ_s` fast/slow semantics), not a theorem — alternatives (3:1 etc.) would give different ψ*; the quotient's 12-fold structure is downstream of this choice (see §1.7).

### 1.5 Bridge f32 → H³×T² (CERTIFIED — code read)

Full-dimension (`0` tossed): RoPE-pair harmonics → `helical_tangent_h3` + `rapidity` → `boost_rapidity` lift → Picard if `x₀ > 16` or `η > 3.5`; phase from `phase_drive·π + 0.05·state`; `BridgeLossReport` with `retained_5d = 1.0` claimed by construction (all dims preserved).

**Note F-3.** [VERIFIED — `f32_to_atom_address` read] `retained_5d_energy_ratio = ONE` is *asserted*, not measured (all dims flow through, so loss is definitionally the projection residual, which is not computed). The honest field is `retained_3d` (measured, floored at 0.25). The 5D claim should be relabeled "preservation by construction (no drop stage)" vs "measured ratio" — one-line correction entry in the vault's ledger. Minor.

### 1.6 Retrieval: VP-tree + 5D score (CERTIFIED — code read; resolves co-agent B-1/B-2)

`retrieve`: Picard-normalize query → ascending-`⟨,⟩_L` sort (linear) or `search_vp` with triangle-inequality pruning on `hyp_distance` (`acosh(max(ip,1))` — the `max` guards domain). `retrieve_5d`: 4× oversample → `combined = ip + (t2_dist >> 2)` → ascending sort. `evict`: max unpinned `x₀`.

**Vault-grounded resolution of my certification B-1/B-2** (filed as entries here, co-agent file untouched):
- The vault's retrieval sorts **ascending inner product = nearest-first** (cosh is monotone in distance). **The code is correct; the C05 Design.md formula (`α ∝ exp(+⟨,⟩)`, "top-k by score") is what's inverted.** Fix direction CONFIRMED by the implementation: doc must adopt `cost = d_H + ¼·d_T²` (retrieve argmin) / `relevance = exp(−cost/τ)` (rank argmax). The co-agent's carving doc already uses `exp(−d_H/τ)` correctly in its own step 5 — the inconsistency is C05-text-vs-code, not code-vs-code.
- `torus_distance_sq = (1−cos Δf) + (1−cos Δs)` matches C05/C35 `d_T²` exactly (factor ¼ applied at use site via `>> 2`). Consistent.
- `O(log n)` holds under the same bounded-expansion qualifier already recorded (C05.12); `search_vp` pruning is standard best-first with `tau_of`. Degenerate-input behavior untested — inherited open item, not new.

### 1.7 The quotient (H³×T²)/Γ₁₂ — where proof ends (REPORTED + vault's own boundary, quoted)

Proven in-VM per QUOTIENT_PROOF (REPORTED, code at `core_lm/src/geometry/quotient_proof.rs`, "Five tests, all passing" per the writeup): 12-sector tiling (no gaps/overlaps), ψ* invariance under 2:1 lock incl. wrapping, address well-definedness. The vault's own honest boundary — **adopted verbatim as this formalization's boundary**:
- Γ₁₂-specific generators (Whitehead-link subgroup): NOT in code — needs steward specification. `picard.rs` implements full `PSL(2,ℤ[i])`, not the index-12 subgroup.
- Quotient action on the H³ factor: NOT implemented (Picard handles H³, torus is separate).
- Unified quotient + manifold volume 3.6639: theory/unverified.
- **Formal consequence.** [INFERRED] Every claim of the form "X holds on (H³×T²)/Γ₁₂" currently reduces to "X holds on H³/PSL(2,ℤ[i]) × (T²/∼₁₂)" with the coupling unproven. The 12 in phase quantization and the 12 in Γ₁₂ sharing a value is NECESSARY but not SUFFICIENT for the identification — sufficiency needs the generators. All downstream uses (silo addressing, 12-role structure) inherit this ceiling: they are well-defined on the product of the two proven quotients, not on the unified manifold. Nothing breaks; the label changes.

### 1.8 Icosahedral 12-fold (REPORTED as cited mathematics + vault measurements)

6 duality axes → 12 vertices; min-max coherence `1/√5` (Welch/simplex bound, equality iff icosahedral — standard result, REPORTED as cited); kissing number 12 (Schütte–van der Waerden 1953, REPORTED as cited); measured mean |cos| = 0.079, max 0.177 in embedding space (REPORTED: vault spec F17); mask-diversity correction `4 = 4 variants × 3 phase rotations` (REPORTED: vault correction ledger, `silo.rs` refs). The vault's correction ledger is exemplary — four overclaims already withdrawn with evidence pointers (weights-transport, physical-absence, 12-distinct-specialists, weight-copies-split). **This formalization adopts those corrections as settled; nothing in §§2–7 re-opens them.**

---

## 2. The four minds: what the vault bytes + code actually establish

Distillation architecture (VERIFIED from `distill_engine.rs` + `map_native.rs` + spec §5): teacher taps → entropy sieve (`H(p) ≤ 2.75 bits`, certainty `1/(1+H)`) → optical refraction → keep-and-carve → Levi-Civita Wigner transport between consecutive steps → `DistilledReasoningArc` (1-cell); closed role loop → Gauss-Bonnet holonomy (2-cell) → salience. Pipeline purely additive; LoRA/QLoRA/adapters never created. `origin_model_tag` (u32 short hash) + `teacher_model_name` + `teacher_fingerprint` (SHA-256) per arc — provenance is per-arc, auditable.

| Mind (steward's name) | Vault/code evidence | Label |
|---|---|---|
| GLIMMER | `map_native_glimmer_report.json`: 52-layer Glimmer, 1-layer replace → hidden cosine 0.99999994, top-1 agreement, top5 overlap 5; model dir `Muse-Glimmer-q4km-gs128-MLX` on disk | REPORTED (report file; suites not re-run). Strongest single-teacher evidence in the vault. |
| Qwen3.6MoE | `distill_engine.rs` layer-scan comment names `Qwen3.6-35B-A3B-OptiQ-4bit` (40 layers); `Qwen3.6-MoE-Combined` dir on disk; `CrossSiloGeodesicBridge` doc names `glimmer ↔ qwen` bridging | REPORTED. Distillation *event* evidenced; per-arc share unmeasured. |
| 3.827B dense | **No match for `3.827B`, `3.8B dense`, or equivalent in vault code/docs/bytes probed** | NOT FOUND — identity unresolved. Question Q-1 for steward. |
| DeepSeek safetensors | `distill_engine.rs` names `DeepSeek-V4-Pro-4bit` as a config.json example for the layer-scan path | REPORTED (code comment only — weakest link; ingestion receipts not located). |
| QwQ-32B math | `QWQ` byte fragments in vault (`H`=QWQ=`, `?=qwQ=`, `<QWQ<y1`); Hub `Qwen/QwQ-32B` exists (Apache-2.0, base `Qwen2.5-32B`); held-crystallization log shows Tier2 math-reasoning rows | REPORTED. Role (math-teacher taps) INFERRED from fragments + log content class. |
| Nemotron-3 30B MoE | Vault backup `…pre-nemo-distilled` (2026-10-26) +26 MB delta = the Nemo landing event; `NVIDIA-Nemotron-3.5-Lightning-30B-A3B` + `Qwen3-Embedding-8B` + `nemotron_router_entropy.json` on disk; Hub `nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B-BF16` exists | REPORTED (event evidenced by bytes; composition/teacher-share unverified). NOTE: on-disk is **3.5-Lightning**, steward named **Nemotron-3** — version mismatch flagged, Q-2. |
| Unified identity | `Qwen3.8-27B+Muse-Glimmer-UnifiedMind` (once, header-adjacent + digest) | VERIFIED bytes. The serving identity is Qwen3.8-27B + Glimmer-distilled — consistent with C01's serving model. |

**Formal distillation statement (what "distilled into one mind" means here, precisely).** [INFERRED from code+bytes; ends at cited sources] There is ONE weight buffer (`FrozenBaseWeights`, fingerprint-pinned, `base_weight_copies = 1` VERIFIED per vault ledger). The "minds" are NOT weight merges — they are **provenance-tagged trajectory populations** in one address space: atoms/arcs carrying `origin_model_tag` + teacher fingerprints, addressable by the same geometry, retrievable across tags via `CrossSiloGeodesicBridge`. Unification = shared address space + shared geometry + per-arc provenance, NOT weight fusion. Any "merged weights" reading is contradicted by the vault's own correction ledger and must not be asserted.

---

## 3. Dreamstate/idletime novelty: formalization (PROPOSED — falsifiable; mechanism EXISTS, measurement missing)

### 3.1 What the vault already implements (VERIFIED — `continuous_active_mind.rs` read, lines cited)

`execute_autonomous_dream_cycle(epoch, max_charts, max_bridges, map, policy)`:

1. **Chart completion.** Candidate `ATOMA_OP_V3` atoms with `rank < 8` → 7-ray icosahedral perturbation sweep (`freq = r·2π/256`, `phase = r·π/4`, 0.24-sketch / 0.18-effector modulation) → MGS accretion (`accrete_mgs_subspace_rank`, ε = 0.08, seed `0xD2EA_0000|r`) → payload update iff rank increased. Receipt: `charts_upgraded_to_multi_rank`, `tangent_ranks_accreted`.
2. **Empty-space bridging.** Curated concept list (24 entries: 20 ML + 4 off-domain: photosynthesis, gravitational attraction, thermodynamic entropy, quantum superposition) → epoch-rotated pairs → ingress-transduce prompt → **dedup: skip if `has_exact_trajectory_chain`** → `discover_and_bridge_novel_concept_trajectories` (N-gram concept extraction → facet templates → trajectory search, cap 96 ops) → coherence gate (`is_coherent_egress_text`) → `crystallize_episode` → count iff `novel_arcs_minted > 0`. Receipt: `empty_space_bridges_minted`, `novel_dream_propositions`.
3. **Span folding + holonomy.** Recurring high-certainty spans → macro-opcodes; 5-role loop holonomy check → `DialecticalSynthesisVerified` verdict. Receipt: `holonomy_verified`, before/after atom+arc counts.

### 3.2 Precise characterization of the novelty mechanism (INFERRED — ends at the code above)

Dreamstate novelty in this vault is **combinatorial proposal + geometric grounding + absence-gated minting**, NOT ex-nihilo geometric creation:
- **Combinatorial:** cross-domain pairs come from a *curated* list (the 4 off-domain seeds are the novelty fuel — without them the mechanism only bridges ML↔ML).
- **Grounded:** a bridge mints arcs only if trajectory search + coherence gate + crystallization succeed; the `has_exact_trajectory_chain` dedup guarantees minted bridges are *absent-path* by construction (this is exactly the steward's "unwalked path" condition, operationalized).
- **Governed:** everything lands via `crystallize_episode` (receipted, policy-checked) or mutable-access-gated self-distillation (`active_inference_step_mut`); holonomy verdict recorded per cycle.

This SATISFIES the steward's requirement structurally: unwalked paths CAN enter the map, through the dream hole, never through recall. Recall stays fail-closed (`NoDirectGeodesic` → superposition → `SuperpositionFailClosed` with audit receipt — refusal is correct behavior, §4.3).

### 3.3 The one defect that must be fixed before A40 (BLOCKING — B-D1)

**Finding.** [VERIFIED — lines ~3443–3475 read] When discovery fails, the cycle mints arcs for a **template sentence** ("Cross-cluster synthesis (epoch N): `A` connects to `B` via product-manifold geodesic routing and Rank-8 Local Jacobian subspace projection") with cert 0.95. Template-minted arcs are **indistinguishable in kind** from discovered bridges in the receipt (`empty_space_bridges_minted` counts both) — novelty theater: the counter increments even when no discovery occurred.
**Required correction (entry, not edit).** [PROPOSED] (i) Separate receipt fields: `bridges_discovered` vs `bridges_synthesized_fallback`; (ii) fallback cert capped below any discovery cert AND labeled `SYNTHESIZED` in provenance; (iii) A40 counts ONLY `bridges_discovered` toward the treatment effect (fallback arcs are control-class content). Until this lands, any dream-cycle bridge count is inadmissible as a novelty measure.

### 3.4 A40 (dream gain) reframed: mechanism EXISTS, measurement missing (PROPOSED prereg seed)

My certification §5 proposed A40 as mechanism+measurement. Vault-grounded correction: the mechanism exists and is READABLE (receipts already emitted); A40 measures its *gain*:
- Treatment: dream cycle ON (≥1 epoch, fallback counting per B-D1 fix) vs Control: dream OFF, equal map budget.
- Primary: held-out task success or retrieval precision delta at equal budget; Secondary: `bridges_discovered` count, promotion survival rate (bridges still present + still coherent after K cycles — anti-chaff), judge-calibration (C06.16/A37 over the coherence gate).
- Falsifier: no delta at equal budget → dreamstate ships OFF (C13.1). Promotion-gate rule (my cert §5 item 4) stands unchanged and is COMPATIBLE with the vault (crystallize + policy + holonomy are the existing gates).

---

## 4. Toroidal phase encoding + toroidal attention (formalized from the implementation)

### 4.1 Phase encoding (CERTIFIED as implemented — `torus.rs` + `bridge.rs` read)

- Per-atom phase: `θ_f = atan2(x₂,x₁)`, `θ_s = atan2(x₃,√(x₁²+x₂²))` (`Phase::from_lorentz_spatial`; origin → (0,0) convention — this FIXES my cert M-1 branch-cut question: the convention exists in code).
- Toroidal distance: `(1−cos Δf)+(1−cos Δs)` — chord-based, wrapped-safe (cosine kills the branch cut). `>> 2` (×¼) weighting at use site, H³-primary.
- Kuramoto evolution + amplitude + rapidity + full-dimension RoPE-pair harmonics (`evolve_vector`, 2560 pairs for 5120D, 0 tossed) — the "invocation" carrier: every key/query is transduced through the SAME encoder before addressing, so phases are comparable by construction.
- RoPE correspondence (REPORTED from RoPE paper §§1–3 + vault code): RoPE rotates pair `(x_{2i},x_{2i+1})` by `m·θ_i`; vault `pair_coherence`/`radial_pair_toroidal_coherence` score `cos(θ_q − θ_k)` per pair (amplitude-weighted in the radial variant). **Same mathematical object** (relative rotation angle per 2D pair); vault generalizes it to a toroidal attention score with learned gain. Precedent, not proof — stated as such.

### 4.2 Toroidal attention evaluator (CERTIFIED as implemented — `toroidal_attention.rs` read)

Per-layer, per-role, per-(head, sequence): causal latest-by-position KV assembly from silo atoms (typed `KV_MAGIC` payloads, non-finite REJECTED) → candidate ranking by ascending Lorentz IP → `adapt_kv_payload` cross-epoch adaptation (immune-authorized) → dual weights: `softmax(dot·scale)` baseline vs `power_weights(coherence, gain)` toroidal (`w ∝ score^gain` normalized — tempered voting, gain fit on held-out half with `g=1` arm always measured) → relative-L2 receipt; intervention ONLY at `target_layer`.
- Invocation gating (the steward's "allots only the attention necessary"): `DepthHint→LayerMask` (Early/Mid/Late/Auto, spec §6.4) + `QualityTarget` (max_atoms, cosh threshold) + causal position filter + top_k + role-confinement (Coordinator REJECTED, non-running role REJECTED, immune forward-permit REQUIRED). Six independent throttles, all fail-closed. The C03 `d*` gate is this machinery's objective function; C06.15's endogenous k-draft is its proposed automation (PROPOSED, unimplemented — correctly labeled).
- Failure modes are explicit: `Err("no causal typed KV atoms…")`, non-finite rejection, permit denial. No silent degradation path found in the read sections.

### 4.3 Refusal semantics (answers the dealbreaker directly)

Greedy recall with no outgoing arc (`cosh d > 1.25`) STOPS (`NoDirectGeodesic`) → superposition hold (≤12 branches from 4 geometric sources incl. cross-silo bridges) → counterfactual rollout (H ≤ 4) + Gauss–Bonnet loop check (echo chambers, Ω=0, REJECTED) → collapse or `SuperpositionFailClosed` + audit receipt. **Refusal is a receipted verdict, not a dead end, and the unwalked path's remedy is the dream cycle (§3), not recall relaxation.** The architecture already encodes the steward's dealbreaker as a division of labor. What is missing is ONLY the measurement (A40) and the B-D1 receipt fix — not the mechanism.

---

## 5. The 0-2+cell8 Jacobian, stated plainly (CERTIFIED structure — `map_native.rs` + harness read)

- **0-cell** = `GlobalAtom` (address + payload). **1-cell** = `DistilledReasoningArc` (from→to + teacher provenance + certainty). **2-cell** = closed role loop with Gauss–Bonnet holonomy verdict (flux + Wigner angle + discrepancy `|Ω_GB − θ_W|` vs 1e-7 gate, `holonomy.rs` read).
- **cell8** = `ATOMA_OP_V3` Rank-8 Local Jacobian Chart per atom: principal spine (rank 1: receptor sketch 256d + attn/mlp effectors) + up to 7 MGS-orthogonal tangent slots (ε = 0.08, teacher certainty + origin tag per slot).
- **Local Jacobian model** (from code comments + struct, VERIFIED read): `Δx^(l)(x_in) = Σ_m w_m(x_in)·(‖x_in‖/‖x_m‖)·Σ_{r=1..R_m} σ_{m,r}·⟨Π(x_in), u_{m,r}⟩·ψ_{m,r}` — a locally-linear operator model with rank ≤ 8 per atom, gated by entropy sieve (2.75 bits) and MGS orthogonality.
- **What the Jacobian buys** (INFERRED — ends at Suite 1 + dream step 1): (i) Suite-1 ablation substrate (rank r₁ vs r₄ vs r₈ on real silicon layers — the vault HAS a Jacobian measurement harness, `atoma_eval_harness.rs` Suite 1); (ii) dream chart-completion target (sub-rank charts upgraded by icosahedral perturbation + accretion); (iii) sensitivity semantics for near-miss detection (my cert §5 item 5's "linearization" proposal is implementable HERE: `σ`-weighted projection residual as the near-miss score — PROPOSED instantiation, needs prereg).
- **0-2 ("0-2+cell8")** = the trajectory→loop closure (atoms → arcs → holonomy-verified 2-cells) that dream bridging must satisfy before minting (holonomy check in the dream receipt). The "+" is promotion-gated accumulation. Stated, not mystical.

---

## 6. Necessity of the geometry (the "why hyperbolic" argument, closed)

1. **Capacity.** Trees embed in H² with arbitrarily low distortion; Euclidean needs unbounded dimension (REPORTED: Nickel & Kiela §2 + vault spec §2.2; classical: Adcock δ-hyperbolicity, Sarkar trees). The map stores reasoning *trajectories* (trees/chains) — the workload IS the tree case. Necessity: any Euclidean map of the same trajectories pays dimension blowup or distortion; hyperbolic pays neither.
2. **Hierarchy coordinate.** Radial `x₀` (salience `1/x₀`, eviction = max-`x₀`, blast `B = 1/(x₀+1)`, origin = authority) gives hierarchy as a *coordinate*, not a learned feature. Euclidean has no distinguished radial semantics — hierarchy must be learned per-task and re-learned per prompt. Necessity: governance (pins/Dirichlet), forgetting (eviction), and authority (blast ceiling) all read the SAME coordinate — one mechanism, three functions. That unity exists nowhere in Euclidean attention.
3. **Phase channel.** T² carries position/role/cycle information the H³ radial coordinate cannot (radial is 1-D salience; phase is 2-D relational). RoPE correspondence (§4.1) shows rotation-carried relative position is independently validated machinery; the vault extends it to cross-mind bridging (`glimmer ↔ qwen` bridges are phase-routed).
4. **Quotient addressing.** The 12-fold discretization (proven T² half) turns continuous retrieval into addressed sectors — O(1) sector routing before O(log n) VP-tree descent. The unified-quotient gap (§1.7) bounds but does not void this: sector routing works on the proven T² factor today.
5. **Falsifier.** [PROPOSED] If A1 (or vault Suite-1 analogues) shows no throughput/recall advantage vs Euclidean KNN at equal memory on the fixed corpus, the geometry is ornament and ships as OPTIONAL (C13.1). The claim is staked, not assumed.

---

## 7. Questions for steward (only you can answer)

- **Q-1.** What is the 3.827B dense mind? No match in vault code/docs/bytes. Name, checkpoint, teacher-tag, layer count?
- **Q-2.** Nemotron version: on-disk is 3.5-Lightning; you named Nemotron-3 30B MoE; Hub has 3-Nano-30B-A3B. Which checkpoint distilled, and is `nemotron_router_entropy.json` its router trace?
- **Q-3.** Γ₁₂ generators: QUOTIENT_PROOF says they need your specification. Where are they (file/mind/notebook), and is volume 3.6639 derivable once supplied?
- **Q-4.** Dream judge calibration: any recorded `holonomy_verified=false` cycles or bridge-promotion survival data? (Determines whether A40 starts from zero or from priors.)
- **Q-5.** `3.827B` and any DeepSeek ingestion receipts: which arcs carry their `origin_model_tag`s? (Lets me close the teacher-share table instead of leaving two rows REPORTED.)

---

## 8. Register entries (append — owning agent to file; none applied here)

1. B-D1 receipt split (`bridges_discovered` vs `bridges_synthesized_fallback`, SYNTHESIZED provenance cap) — BLOCKS A40 until landed.
2. C05 doc fix: cost-form retrieval (`d_H + ¼·d_T²`, argmin) + corrected TwisterAttention kernel (`exp(−d/τ)`) + re-issued A1/A33/A36 preregs (resolves my cert B-1/B-2 in favor of the vault's implementation).
3. `retained_5d = ONE` relabel (measured vs by-construction) — F-3.
4. Receipt-staleness policy: map SHA `fbadb14d…` supersedes audit `2db521b4…`; re-issue the 218,908/216,432 census or mark it historical (M-11).
5. A40 prereg (dream gain, §3.4) + C06.16 monitoring over the coherence gate.
6. F-1 fold-kink test + F-2 cusp-language precision (minor; batch with next vault pass).

## What was NOT checked

Suites not re-run (all test counts REPORTED from vault docs); RoPE/HNN/Poincaré proofs beyond excerpts read; Hypformer beyond title/summary; outcomes-ledger binary content; `transport_frame_hop` line-level numerics; the 998 MB token-trajectory vault (not opened — next pass on steward's call); full `native_predictor`/`refraction` line audit; multi-machine replication; Q-1–Q-5 answers.
