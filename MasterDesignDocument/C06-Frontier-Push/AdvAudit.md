# C06 — Adversarial Audit: Carved Swarm Map, Distillation, and A39 Path

**Status.** PROPOSED audit artifact. **Date:** 2026-10-27. **Scope:** `carve_task_view` (monster-hybrid-server `silo.rs`), `Allocate` as specified in `CARVED_SWARM_SPEC.md` §6, dissolution/`compile_deposit`, A39 abuse paths.

---

## 0. Pressure-test baseline (executed)

| Command | Result |
|---|---|
| `cargo test --manifest-path .../immune_system/implementation/Cargo.toml --lib` | **59 passed / 0 failed** (0.75s–1.06s) |
| `cargo test --manifest-path .../orchestrator/Cargo.toml --lib` | **114 passed / 0 failed** (26.8s) |
| `refraction::tests::ball_and_hyperboloid_distances_agree_to_4_4e_9_across_900_pairs` | PASS (`≤ 4.4×10⁻⁹`) |
| `continuous_active_mind::tests::subagent_task_carved_view_isolates_silos_and_executes_in_motion` | PASS (6 silos, ≥2 arcs, ≥1 excluded) |

No code changes were made for this audit. All findings below are derived from reading the current source.

---

## 1. Adversarial findings

| ID | Surface | Vector | Impact | Reproduction / Evidence |
|---|---|---|---|---|
| AA-01 | `carve_task_view` empty `allowed_silos` | Caller passes `&[]`. `primary_role` defaults to `SensoryPerception`, no atoms carved, view is empty. | Subagent runs with **zero supporting knowledge** and no explicit failure; can hallucinate or over-rely on model prior. | Source: `if let Some(silo)` loop over empty slice yields no atoms; `TaskCarvedView.atoms` empty. |
| AA-02 | Forward closure (1-cell arcs) | An attacker seeds one atom in an allowed silo that has a reasoning arc into a **different basin**; closure pulls that `GlobalAtom` even though its (unknown) home silo was never allowed. | **Cross-silo knowledge leak**, violating task scoping and potentially privacy across lanes (violates I5 / §5). | Source: `carve_task_view` forward-closure loop calls `self.global_atom(arc.to_atom)` and inserts it unconditionally; no silo-membership or geometric well check. |
| AA-03 | No atom-count bound in `carve_task_view` | A malicious / huge map with long arc chains; closure runs up to 64 hops, inserting every reachable atom. | Unbounded memory/time; can breach the C21/C12 memory ceiling or OOM; violates `QualityTarget.max_atoms`. | Source: `while !frontier.is_empty() && hops < 64`, no `atoms.len()` cap. |
| AA-04 | Duplicate atom id across silos | Same `AtomId` appears in two allowed silos with different payloads/phase. | First silo wins non-deterministically w.r.t. input order; nondeterministic digest between runs with reordered `allowed_silos`. | Source: `atoms.entry(sa.id.0).or_insert_with(...)`. |
| AA-05 | `task_center` inner-product orientation | If a future refactor changes `lorentz_inner_product_wide` sign/convention, the horizon filter becomes inverted (keeps far atoms, drops near). | Silent recall inversion; A39 could pass while scoping is wrong. | Source: `if d.0 > max_cosh.0 { continue; }`. Correct today (hyperbolic `⟨x,y⟩ = cosh d`), but implicit and untested at the API boundary. |
| AA-06 | Dissolution / `compile_deposit` mid-failure | An agent returns `ΔM_T`; a crash after partial merge leaves global map inconsistent but no quarantine (unless caller handles). | Partial application; Merkle fingerprint drift; violates §3.2 step 4–5. | Source: `compile_deposit` coalesces unconditionally; caller must check fingerprint. There is no atomic transaction wrapper visible at this API. |
| AA-07 | Empty carve result not surfaced | `carve_task_view` returns an empty `TaskCarvedView` (no `Result`). Callers such as `execute_subagent_in_task_view` proceed with empty context. | Hallucination-by-default; A39 could report inflated J6 if judges accept fluent wrong answers. | Source: no emptiness guard on return; `ExecuteSubagentTaskView` receipt just reports counts. |
| AA-08 | `primary_role` is first silo only | Forward-closure uses `primary_role` well for all pulled atoms, distorting `well_cosh_dist` and phase alignment for atoms homed to other allowed silos. | Weaker relevance signal; could skew `λ`-effective recall but not correctness of IDs. | Source: `primary_role = allowed_silos.first()…`; `boost_to_well_frame(primary_role, …)`. |
| AA-09 | `λ_map` pressure term can starve the map | Under high token pressure, `λ_map → 0`; exact capsule still ships, but all supporting recall vanishes silently. | Task answers lose provenance support; M3 necessity may fail, but no graceful degradation receipt. | Methodology §6.2: λ can clamp to 0 without a logged reason. |
| AA-10 | A39 judge gaming | Arm B receives more total tokens than Arm A; J5 violated but other judges can still pass, allowing “efficiency” claims on an unfair comparison. | Confound: gain may come from extra context, not carving. | `Pre-Registration-A39-v1.md` §8.6 already forbids this for J5, but enforcement must be in harness, not docs. |
| AA-11 | `Allocate` (spec) leaks across overlap boundaries | `carve_task_view_for_role` includes peers where `overlap_allowance > 0`, i.e., shared boundary atoms. A restrictive task in one silo can still see neighboring-silo boundary atoms. | Overlap is intended for shared concepts, but policy values may be too generous; audit `Ω_{r,s}`. | Source: `overlap_allowance(primary, peer).0 > 0`. |
| AA-12 | 64-hop forward closure magnitude | Hardcoded `max_hops = 64` with no relation to `QualityTarget` or `max_atoms`. | Worst-case quadratic traversal; no bounded termination except hop count. | Source: `while … hops < 64`. |

---

## 2. Hardening plan (apply before A39 data)

1. **Empty-silo guard (AA-01, AA-07):** If `allowed_silos` is empty, return a `TaskCarvedView` flagged `unscoped_fallback = true` built from `RoleId::Coordinator` silos **or** return an explicit error. Prefer error at the `Allocate`/harness boundary; at minimum, count and log `empty_carve` in the receipt.
2. **Cross-silo leak stop (AA-02):** In forward closure, before inserting `ga`, verify it lies within at least one allowed silo’s basin (via `atlas.well(role)` cosh distance ≤ basin radius + overlap allowance). If not, skip and increment `cross_silo_leak_blocked`.
3. **Bounded carve (AA-03, AA-12):** Add `max_atoms` enforcement inside `carve_task_view` (default `4096` or pass-through from `QualityTarget`). Stop expansion once `atoms.len() >= max_atoms`; expose in receipt.
4. **Deterministic tie-break (AA-04):** Replace `or_insert_with` first-wins with a canonical rule (highest `salience_count`, then lowest `AtomId`), so reordered inputs cannot change the digest.
5. **Horizon assertion (AA-05):** Add a debug assertion / property test that for any `center, atom`, `included ⇔ lorentz_inner_product ≤ max_cosh`. Pin the hyperbolic identity `⟨x,y⟩ = cosh d_H` in a regression test.
6. **Atomic dissolution (AA-06):** Wrap `compile_deposit`/merge in a transactional helper: compute candidate fingerprint, compare to `expected = fingerprint_pre XOR intended_write_set_hash`, commit only on equality; otherwise quarantine and emit receipt.
7. **λ transparency (AA-09):** When clamping yields `λ = 0`, emit a telemetry field `map_starved_reason = {pressure|none}` and fall back to capsule-only with a receipt flag.
8. **A39 confound enforcement (AA-10):** In the harness, void J5 (and caveat J1–J4/J6) whenever `tokens_B > 1.10 × tokens_A`, per the filed preregistration.
9. **Overlap policy audit (AA-11):** Sweep `Ω_{r,s}` against a least-privilege default; require any `> 0` allowance to be explicitly listed and receipted.

Status of these: **PROPOSED constraints.** None is implemented yet; each should be a small, independently testable patch plus a property test before A39 data is trusted.

---

## 3. Side-by-side formalization commitment

After hardening, the final methodology will be presented as (a) mathematical statement, (b) code-level invariant, and (c) validation test, in lockstep:

| Concept | Math | Code invariant | Validation |
|---|---|---|---|
| Carve projection | `M_T = Φ(M_G,T)` with `∀a∈M_T : d_H(x_a,μ_T) ≤ R` and domain filter | `carve_task_view` output satisfies horizon + allowed-silo predicates | AA-05 property test |
| No duplication | `weight_copies = 1` (C06.12) | `Arc<[u8]>` sharing, single `FrozenBaseWeights` | existing `holonomy` tests + A4/A21 |
| Dissolution safety | `fingerprint_post = fingerprint_pre ⊕ intended_write_set_hash` | atomic `compile_deposit` | AA-06 property test |
| Bounded resource | `|M_T| ≤ max_atoms` | cap in carve + receipt | AA-03 property test |
| Scoping efficiency | A39 J2–J4 thresholds | carved path vs flat path | A39 run |

---

_Last updated: 2026-10-27 (adversarial audit pass)._
EOF
