# TorodialPhasePositionalEncoder (TPE) — RoPE's Hyperbolic Replacement

**Date.** 2026-10-27. **Author.** Muse (contributor lane), on steward's directive.
**Status.** PROPOSED specification. **Parent.** Vault formalization (`VaultMapDreamstateFormalization.md` §7 staged this interface) + frozen math (`TorPhPoEnc/TwisterPhPE_Whitepaper_v4_1_MATH_FREEZE.md`, canonical; corpus copies verified byte-identical + 8-line extraction header).
**Scope.** Transforms the frozen TwisterPhPE core (§§1–9, 11–12) into the serving positional encoder that replaces RoPE: dynamic iteration with every rotation, spiral-out (radius expands per turn), NOT a corkscrew (constant radius). MATH_FREEZE §§1–9 are adopted intact and re-proved where the encoder extends them; §10 roadmap stays non-normative.
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no vault/corpus/freeze file modified — corrections live here as entries).

---

## 0. Provenance (VERIFIED this pass)

| Source | Hash / finding | Label |
|---|---|---|
| Freeze canonical file | `TwisterPhPE_Whitepaper_v4_1_MATH_FREEZE.md` (1176 lines) | VERIFIED (read in full, §§1–12 + refs + changelog) |
| Corpus copy | byte-identical + 8-line `extract_docs.py` header (2026-10-23) | VERIFIED (shasum + diff) |
| MATH_CORE v4.1 implementer sheet | dynamics/coherence/agg/C-Lock/code-gate summary | VERIFIED (read) |
| NONCOLLAPSE probe note | spiral-stays-open metrics, honest boundary | VERIFIED (read) |
| WORKING_THEORY (lemniscate) | hypothesis-grade framing, measured F17/F19 cited for 12-around-1 | REPORTED (read; F17/F19 artifacts not opened) |
| dhng_lattice.tex | IR-geometry paper, explicitly NOT about TwisterPhPE (scope boundary honored) | VERIFIED (read; excluded from normative chain) |
| Vault `torus.rs::evolve/phi` | recurrence + `[1,cos,sin]/√2` feature map live in serving code | VERIFIED (read; §2 below) |

---

## 1. What the freeze proves (adopted, not re-derived)

1. **2:1 resonant pair** (§3): `θ̃_f ← θ̃_f + Δθ_f + ω_f − K_f·sin ψ`, `θ̃_s ← θ̃_s + Δθ_s + ω_s + K_s·sin ψ`, `ψ = wrap(θ̃_f − 2θ̃_s)`; lock-error map `ψ ← wrap(ψ + δ − K·sin ψ)`, `K = K_f + 2K_s` (Prop 1, proved by subtraction); stationary lock `sin ψ* = δ/K`, `|δ| ≤ |K|`, attraction `0 < K·cos ψ* < 2` (Prop 2); effective carrier `ω_f^eff = ω_f − K_f·sin ψ*` under maintained lock C-Lock (Prop 3 + Definition).
2. **Telescoping path** (§4): `A_{i,j} = Σ increments = θ̃_f(i) − θ̃_f(j)` — exact, lock-free. Coherence `C_{i,j} = ½ + ½·cos(A_{i,j}) ∈ [0,1]`, no normalization in the normative core.
3. **Exact rank-3 feature map** (§5, Thm 1): `C_{i,j} = φ_iᵀφ_j`, `φ(θ) = [1,cosθ,sinθ]/√2` — proved by cosine subtraction formula. Kernel rank ≤ 3 unmasked.
4. **Linear-time causal aggregation** (§6): `S_i = S_{i−1} + φ_i·v_iᵀ`, `y_i = S_iᵀφ_i`; state `3·d_v` scalars + O(1) phase; `T_core = O(n(d+d_v))`, `O(nd)` when `d_v = Θ(d)`. Optional exact normalization `ȳ = Sᵀφ/φᵀz`, denominator ≥ 1 (no softmax ever).
5. **Boundary honesty** (§§9, 12): rank-3 = expressivity constraint; T² Fourier extension (`φ(θ_f)⊗φ(θ_s) ∈ ℝ⁹`, resonance mode `m=(1,−2)` → `cos(θ_f−2θ_s)`) is the named next experiment, not core; superiority claims reserved for bakeoff; 10-gate implementation ladder (§11) precedes any optimized kernel.

## 2. Vault-conformance proof (measured by reading, 2026-10-27)

The serving `torus.rs` implements the frozen recurrence EXACTLY:
- `coupling_f = k_f·sin(θ_s−θ_f)`, `coupling_s = k_s·sin(θ_f−θ_s)` — matches freeze with `diff_sf = −ψ`, `diff_fs = +ψ` up to wrap convention (sin is odd; `sin(−ψ) = −sin ψ` absorbed in sign — VERIFIED term-by-term).
- `step_f = ω_f + coupling_f + df`, `next_f = θ_f + step_f` — matches `θ̃ ← θ̃ + Δθ + ω − K·sin ψ` with per-channel K. `psi = next_f − 2·next_s` — matches `ψ` definition (wrap deferred to use site; cosine use makes wrap unnecessary — consistent with freeze §4 "explicit wrapping is unnecessary in the final coherence evaluation").
- `phi(θ) = [1,cosθ,sinθ]/√2` — byte-identical to freeze Thm 1 feature map.
- Defaults `ω_f=0.1, ω_s=0.05, k_f=0.3, k_s=0.15` satisfy Prop 2 stability (`K = 0.3+2·0.15 = 0.6 > 0`; lock exists for `|δ| ≤ 0.6`, attraction `0 < 0.6·cos ψ* < 2` on the principal branch).
- `evolve_vector`: D/2 pair harmonics + RoPE-theta weighting + RMS-relative unclamped drives + unwrapped-arc accounting + helical tangent + `rapidity_for_state(w, amp, rms)` + `radius_for_rotations(w) = r_min + span·(0.35w + 0.65·ln(1+w))`.

**Conformance verdict.** [INFERRED — ends at the term-by-term reads] Vault encoder = frozen core + serving extensions (amplitude channel, radius/rapidity tracking, RoPE-pair harmonics, Q32_31). No contradiction found between code and freeze on any normative equation. The extensions are PROPOSED-for-spec below (§4), not freeze-covered.

---

## 3. RoPE replacement statement (formal — what "replacement" means and does not mean)

**Replaces** (PROPOSED, falsifiable): the *function* RoPE serves — position-dependent relative structure in attention — with a different *mechanism* (persistent dynamical phase + raised-cosine coherence + recurrent aggregation) satisfying: (i) relative displacement grows with separation (carrier term under C-Lock, exact path otherwise); (ii) decay with distance available (raised-cosine halves per half-period; no learned decay needed); (iii) linear-time causal evaluation with bounded state (§6: `O(nd)`, `3d_v` + O(1)).

**Does NOT claim** (adopted from freeze §1/§8/§12 + dhng scope boundary): algebraic equivalence to RoPE; transformer parity without bakeoff; Penrose derivation; cotangent/symplectic necessity; constant-total-storage; IR-geometry results transfer (dhng_lattice is a separate paper — cited only for shared H³×T² address language, never as evidence for TPE behavior).

**Falsifier.** [PROPOSED] Angle-only ablation (freeze core as-is) vs triple-channel TPE (§4) on the fixed corpus: if no task/latency/energy delta at equal budget, the extra channels ship OFF (C13.1) and the replacement claim reduces to "linear-time RoPE-class alternative," still falsifiable by bakeoff.

---

## 4. The spiral-out extension (PROPOSED normative addition to the freeze — the "not a corkscrew" clause)

The freeze core is rotation-only (phases advance; radius is implicit). The serving encoder adds explicit radial dynamics. This section freezes their form.

**4.1 Radius law (adopts vault `radius_for_rotations`, proves its shape).**
`r(w) = r_min + span·(0.35·w + 0.65·ln(1+w))`, `w ≥ 0` cumulative rotations.
Claim: `r` is strictly increasing (spiral-out) and sublinear-asymptotic (no blowup). Proof: `dr/dw = span·(0.35 + 0.65/(1+w)) > 0 ∀w ≥ 0` ✓; `r(w)/w → 0.35·span` linear-dominated early (escape), `ln` term dominates marginally late (bounded growth, never exponential) ✓. A corkscrew (`r = const`) is ruled out by the vault's own expansion test (radius strictly increases every turn — VERIFIED test read). The constants (0.35/0.65) are PREREGISTERED implementation constants (amendment rule per C09.9), not derived optima.

**4.2 Rapidity law (adopts vault `rapidity_for_state`, bounds it).**
`η = 0.22 + 0.38·w + 0.28·amp + 0.20·ln(1+rms)`. Claim: monotone in `(w, amp)` jointly (both coefficients positive) ✓; rms term bounded by log (energy spikes cannot detonate η) ✓. Interaction with my F-1 (fold kink at |η| = 11.5): the TPE spec REQUIRES the kink documented in the run manifest whenever η crosses 11.5 during a trajectory (deterministic given the law — no surprise), plus the F-1 fix (test/smooth-or-gate) before any bakeoff cites η-driven distances.

**4.3 Amplitude channel (adopts vault `PhaseState.amplitude`, states its semantics).**
`amp ← amp + arc_step + drive_mod`, monotone non-decreasing. Semantics (PROPOSED, locked by this spec): amplitude is *accumulated drive energy*, NOT confidence and NOT salience — it feeds radius (`+0.05·amp` shell term) and η (`0.28·amp`), never attention weights directly. Any future use of amplitude as a score multiplier requires a new prereg (it would break the freeze's unnormalized-core honesty).

**4.4 Triple-channel attention score (PROPOSED — the RoPE-replacement kernel).**
For query/key with `(θ, r, amp)` triples from the same encoder (comparability by construction, cf. bridge `f32_to_atom_address`):
`score(q,a) = cos(θ_q − θ_a)·κ_r(r_q,r_a)·κ_a(amp_q,amp_a)`, with radial gate `κ_r = exp(−|r_q − r_a|/τ_r)` and amplitude gate `κ_a = exp(−|amp_q − amp_a|/τ_a)`, `τ_r, τ_a` preregistered.
Reductions (proof of backward compatibility): `τ_r, τ_a → ∞` recovers freeze `cos` coherence exactly; angle-only ablation = this limit. The T² Fourier extension (freeze §9, `φ(θ_f)⊗φ(θ_s)`, resonance `m=(1,−2)`) lifts to triples by product with the radial/amplitude gates — same falsifier as §3.

**4.5 C-Lock discipline (adopts freeze C-Lock, extends to radius).**
`ω_f^eff` interpretation valid ONLY under maintained stationary lock (freeze Prop 3 + "do not claim always-on RoPE equivalence" — adopted verbatim). Extension: radius/rapidity readouts are valid WITHOUT lock (they accumulate from actuals, like `A_{i,j}`); only the *effective-frequency* story needs C-Lock. Unlocked trajectories use §4.4 with actual phases — well-defined by the telescoping identity.

---

## 5. Implementation gate (adopts freeze §11, adds spiral-specific tests)

Freeze gates 1–10 stand unchanged (phase identity, lock-error, lock regimes, telescoping, 2π-invariance, feature-map identity, quadratic-vs-recurrent equivalence, unlocked trajectories, long-horizon numerics, linear scaling). ADDED for TPE (PROPOSED, must pass before bakeoff):

11. **Spiral monotonicity**: `r(w)` strictly increasing over 10³ rotations; `dr/dw` matches analytic form to Q32_31 precision.
12. **No-corkscrew**: radius variance across a trajectory exceeds fixed-point noise floor by ≥ 10³× (constant-radius implementation FAILS).
13. **Kink discipline**: η-trajectories crossing 11.5 log the crossing; distances computed across the kink match recomputation from raw phases (no hidden discontinuity in scores).
14. **Triple-vs-angle ablation**: §3 falsifier run on the fixed corpus; result determines whether §4.4 ships or stays PROPOSED.
15. **Amplitude semantics**: attention weights computed with and without amplitude gating must differ ONLY where `|Δamp|` exceeds threshold — amplitude never silently dominates angle.

---

## 6. Register entries (append — owning agent; none applied here)

1. Preregister TPE bakeoff (angle-only vs triple, §3 falsifier + §5 gates 11–15) with filed hash (C11.3).
2. Preregister `(τ_r, τ_a)` + (0.35/0.65, rapidity coefficients) as implementation constants with amendment rule (C09.9).
3. File F-1 kink fix/test; file F-3 `retained_5d` relabel (carried from formalization §8).
4. Extend MethodAndMath side-by-side with the PhaseSource-duality row's TPE successor (M-4 → triple-channel row) once §4.4 runs.
5. dhng_lattice stays OUT of the TPE evidence chain (scope boundary reaffirmed — shared address language only).

## What was NOT checked

RoPE paper beyond §§1–3 excerpts; HNN/Poincaré beyond excerpts read earlier; Hypformer beyond title/summary; 998 MB trajectory vault; outcomes-ledger binary; multi-machine replication; bakeoff itself (unrun by design — spec precedes silicon).
