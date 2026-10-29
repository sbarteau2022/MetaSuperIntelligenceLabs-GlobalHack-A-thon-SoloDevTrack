# Conversion Math — Distillation to Capability (without fine-tuning)

**Stance.** Capability throughput DOES NOT REQUIRE FINE-TUNING. VERIFIED doctrine from vault docs: frozen weights read-only; capability arises from 30B-distilled trajectories (0/1/2-cells) stored in the carved swarm map and recalled by retrieval. No weight update occurs. A model is converted _to capability_ by replacing recomputation with recalled, judge-passed reasoning arcs. This is PROPOSED as a claim; certification pending prereg.

**Steward directive (binding, 2026-10-27).** Distilled knowledge is INVARIANT to the state space (`PROTOCOL.md` §8): knowledge and reasoning are separated from the model at absolute depth. Retrieval never consults teacher identity; the atom is a learned symbol to inherit. The math below is therefore teacher-free by construction — no equation takes a teacher argument.

## 1. Distillation operator (teacher-free)

Let a teacher trajectory be a sequence of steps s_i with hidden states h_i ∈ R^d and next-op distribution p_i. The projector (MGS-orthogonalized, ε=0.08) maps each step into a global atom `a_i` carrying:

- coordinate in (H³ × T²)/Γ₁₂ (lorentz vector) — function of CONTENT ONLY (§8: no teacher argument),
- tangent slots (top-k directions),
- certainty / teacher entropy — entropy of the DISTRIBUTION, not identity of the teacher,
- payload fingerprint.

Formally the mint map factors as `mint = carve(project(content)) × audit(teacher)`: the coordinate half is teacher-invariant, the audit half (lineage + `origin_model_fingerprint` field) is teacher-tagged and execution-invisible. Any implementation where `∂(coordinate)/∂(teacher) ≠ 0` is non-conformant.

Novelty gate: an atom mints only if its nearest existing atom distance ≥ δ (novel) OR it raises retained energy of an existing well (reinforcement). Distance is computed over (coordinate, invariant) ONLY — teacher tag excluded, so cross-teacher duplicates coalesce (§8).

## 2. Capability conversion

For a downstream task T with required reasoning sub-problems U(T):

Retrieval covers U iff there exists a 1-cell path through carved wells whose operators match each sub-problem. Coverage:

```
cov(T) = |{u in U(T) : exists carved well + arc covering u}| / |U(T)|
```

Capability score (judge-gated):

```
Cap(T) = Cap0(T) + α * cov(T) * R̄
```

where

- Cap0 = baseline capability of the frozen 8B model (measured, not asserted),
- α = conversion coefficient (PROPOSED, must be calibrated),
- R̄ = mean retained energy of minted atoms (receipted).

Energy budget: a map of capacity B bytes fits at most N = B / mem(entry) atoms (C05.11). Thus

```
N_max = B_map / mem(atom)
cov_ceil = min(1, N_max * p_useful / |U(T)|)
```

where p_useful = fraction of minted atoms that cover a required sub-problem (measured by judge).

## 3. One-pass distillation to capability

Single pass conditions:

- teacher provides trajectories OR weight modes,
- projector + novelty + judge gates fire online,
- atoms mint into carved wells,
- receipt chain (sha-chained) emitted,
- after pass, a fresh model run with the carved map installed must show ΔCap ≥ 0 and no regression on held-out tasks.

One-pass is valid iff ALL FOUR interim gates pass on the filed run receipt (AUDIT.md §7; enforced in-repo by `research-grade-harness/src/distill_judge.rs::DistillationJudge`):

```
knowledge_atoms_minted == modes_scanned            (no dropped receipts, NN-G3 analog)
novel_atoms_minted + coalesced_atoms == knowledge_atoms_minted (novelty accounting: coalescing is counted, not hidden; CORR-2026-10-29: with weight-mode fan-out one mode yields many atoms, so the balance is against MINTED, not scanned — trajectory 1:1 is the special case)
sum(role_counts) == knowledge_atoms_minted          (role distribution balances)
0 <= mean_retained_energy <= 1                      (Cap(T) requires Rbar in [0,1])
```

CORR-2026-10-29 (supersedes the old `Σ receipt_ok = atoms_minted` line, which referenced nonexistent receipt fields per AUDIT.md C-AUD-5): `WeightDistillationReceipt` has no per-atom `receipt_ok` field, so the old condition was vacuous. The four-gate form above is computable TODAY from fields the receipt actually carries. Full per-atom `receipt_ok` gating returns when the in-engine judge lands (AUDIT finding 4).

## 4. What is NOT fine-tuning

No gradient. No weight mutation. No optimizer state. The distillation is **extraction + storage + retrieval + judge**. "One pass" means one read of the teacher, not one training epoch.

## 5. Open calibration

- α (conversion coefficient) — measure via A/B (held-out). **Units (declared 2026-10-29):** capability-points per unit of (coverage × retained-energy), where Cap(T) and Cap0(T) are scored on the SAME task rubric (same units), `cov(T)` is dimensionless in [0,1], and `R̄` is dimensionless in [0,1]. So `[α] = [Cap]`: α converts one unit of (coverage × energy) into the rubric's capability points. If Cap is a normalized score, α is dimensionless. The A/B MUST state the rubric scale at prereg or α is uninterpretable.
- p_useful — measure via judge coverage report.
- δ (novelty threshold) — sweep in ablation.
- B_map budget — from C21.
