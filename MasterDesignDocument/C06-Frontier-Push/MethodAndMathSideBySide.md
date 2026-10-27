# C06 — Method & Math Side-by-Side (Global Map ↔ Carved Swarm Map)

**Status.** PROPOSED aligned specification. **Date:** 2026-10-27. Companion to `ReasoningDistillationMapCarving.md` and `AdvAudit.md`.

Every row below states the **mathematical object**, the **Rust/code invariant**, and the **validation gate** that ties them together. If code diverges from math, the math governs and the code is a bug.

---

## 1. Geometry and distance

| Item | Math | Code | Validation |
|---|---|---|---|
| Canonical geometry | `H³` future sheet, `⟨x,y⟩_ℒ = x₀y₀ − x·y`, `d_H = arcosh ⟨x,y⟩_ℒ` | `OpticalLens::lorentz_inner_product_wide`, `Q32_31` fixed point | `refraction::tests::ball_and_hyperboloid_distances_agree_to_4_4e_9_across_900_pairs` (PASS) |
| Projection | Stereographic `π`, inverse `π⁻¹`, agreement `≤ 4.4×10⁻⁹` | `refract`, `project` | Same 900-pair test |
| Phase | `(θ_f, θ_s) ∈ T²`, retrieval cost `d_H + ¼·d_T²` (argmin) / relevance `exp(−cost/τ)` (argmax). Vault sorts ascending inner product = nearest-first; the additive-⟨,⟩ form is retired (cert ruling M-1) | `Phase`, `Mathematical Certification #1-3` | Phase encoder tests |

**Identity to pin in code:** `lorentz_inner_product(x,y) = cosh(d_H(x,y))`. Any refactor must keep this or all horizon filters invert.

---

## 2. Global map and atoms

| Concept | Math | Code | Validation |
|---|---|---|---|
| Global map | `M_G = (A, d_H, Γ_rec)` over `H³ × T²` | `CarvedSwarmMap { global_atoms, reasoning_arcs, atlas, silos }` | `global_atom_count`, iter tests |
| Atom | `a = (id, x, φ, salience, pinned, receipt)` | `GlobalAtom { id, lorentz, phase, payload, … }` | round-trip tests |
| Salience | radial coordinate `x₀` (higher = more boundary = less salient) | eviction victim = `argmax x₀` among unpinned | C05 §5 + property test (planned) |

---

## 3. Distillation operator (core)

### 3.1 Carve
**Math**
$$
M_T = \Phi(M_G, T) = \{ a \in M_G \mid \text{domain}(a) \in D(T) \land d_H(x_a, \mu_T) \le R(T) \land (\text{pinned}(a) \lor \text{aff}(T,a) \ge \alpha_T) \}
$$
plus forward closure of 1-cells with endpoints in `M_T`, capped at `max_atoms`.

**Code**
```rust
CarvedSwarmMap::carve_task_view(&self, allowed_silos: &[RoleId], task_center: Option<(&LorentzVector, Q32_31)>) -> TaskCarvedView
```
- atoms selected per allowed silo (§3.1 of `ReasoningDistillationMapCarving.md`);
- optional horizon filter: `lorentz_inner_product(sa.global_lorentz, center).0 <= max_cosh.0`;
- arc endpoints both in `atoms`;
- forward closure up to 64 hops (to be capped by `max_atoms` per AdvAudit AA-03).

**Invariant (must hold):** `∀a ∈ M_T.atoms : d_H(a, μ_T) ≤ max_cosh` (when `task_center` is `Some`), and `domain(a) ∈ D(T)` for the initial seed set.

**Validation:** `continuous_active_mind::tests::subagent_task_carved_view_isolates_silos_and_executes_in_motion` (PASS); planned property tests P-CARVE-* in AdvAudit §2.

### 3.2 Dissolution
**Math**
$$
\text{fingerprint}_{post} = \text{fingerprint}_{pre} \oplus \text{intended\_write\_set\_hash}
$$
otherwise quarantine.

**Code**
```rust
CarvedSwarmMap::compile_deposit(...)  // coalesces deposit → global map
```
Caller must verify fingerprint before commit (currently implicit; hardening AA-06 makes it atomic).

**Validation:** planned property test P-DISSOLVE-*.

---

## 4. Task → silo / basin / depth

### 4.1 Allocation
**Math:** `Allocate: (TaskSpec, M_G, Ω) → TaskCarvedView` deterministic, side-effect free.

**Code (spec-level, `CARVED_SWARM_SPEC.md` §6):**
```rust
pub fn Allocate(task: TaskSpec, map: &CarvedSwarmMap, policy: &OverlapPolicyMatrix) -> TaskCarvedView
```

**Invariant:** identical `(task, map, policy)` → byte-identical view; no model forward pass; no randomness.

**Code (runtime):**
```rust
CarvedSwarmMap::carve_task_view_for_role(primary_role, policy, task_center)
```
selects `primary_role` plus peers with `overlap_allowance > 0`.

### 4.2 Necessity score
**Math**
$$
\mathcal{N}(T,b)=0.35\,\text{aff}+0.25\,\text{dep}+0.20\,\text{rec}+0.10\,\text{pin}+0.10\,\text{interf\_safety},\quad \mathcal{N}\ge0.5 \Rightarrow \text{necessary}
$$

**Code (today):** approximated by `probe_and_carve_specialist` (activation share ≥ 13.5%), not yet the explicit formula. **Gap:** implement and test the formula before claiming A39 isolates scoping.

---

## 5. Map weight

**Math**
$$
\lambda_{\text{map}} = \text{clamp}(0.05 + 0.20c + 0.15h + 0.10u - 0.15p,\,0,\,0.35)
$$
$$
B_{\text{map}} = \lfloor \lambda_{\text{map}} B_{\text{ctx}} \rfloor
$$

**Code (today):** not implemented in Rust; exists only in the methodology doc. **Gap:** implement `map_weight(task, pressure)` and serialize `D_T` to exactly fit `B_map`, never truncating pinned/capsule content. Validate with a unit test that `|D_T| ≤ B_map` for 1k random task specs.

---

## 6. Swarm at scale

**Math**
- `weight_copies = 1` (one `FrozenBaseWeights`; `map_knowledge_copies = N` disjoint subsets per overlap policy).
- Persistent `O(1)` in N (one map, one weight set — the real claim); transient peak `Θ(N·k)` with `k ≤ 8` + admission gate. "Sublinear" withdrawn (cert ruling M-2/P-6).

**Code**
- `FrozenBaseWeights`, `CarvedWeightView`, `Arc<[u8]>` payloads.
- `recarve_all_twelve` preserves atoms/arcs on hot-swap (verified by construction tests).

**Validation:** A21 (VRAM one copy) + A5 (lanes) + A39 J2–J4 (trajectory).

---

## 7. A39 (the claim)

**Claim (CC-013, PROPOSED):** Non-tuned subagent + carved map achieves equal-or-better quality with fewer tool calls, retries, and errors than without.

**Judges:** J1 pass-rate +10pts (or ≥90%), J2 tool calls ≤0.8×, J3 retries ≤0.7×, J4 errors ≤0.5×, J5 tokens ≤1.1×, J6 no quality regression.

**Status:** Filed (`Pre-Registration-A39-v1.md`, hash `4e8695fa…`). **No run yet.** Must replicate + necessity ablation before required status (C06.13).

---

## 8. Known open gaps (do not claim yet)

1. Necessity formula (§4.2) not yet implemented/tested.
2. `λ_map` (§5) not yet implemented.
3. AA-02 cross-silo leak not yet blocked in code.
4. AA-03 atom cap not yet enforced.
5. Atomic dissolution (AA-06) not yet transaction-wrapped.
6. A39 manifest/harness not yet built or run.

Only items 1–6 above plus a green A39 run can flip CC-013 (and CC-010/CC-011/CC-012) from PROPOSED to VERIFIED. Claims must not be asserted otherwise.

---
_Last updated: 2026-10-27._
EOF
