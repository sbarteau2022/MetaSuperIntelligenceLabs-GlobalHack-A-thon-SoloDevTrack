# Conversion Math — Distillation to Capability (without fine-tuning)

**Stance.** Capability throughput DOES NOT REQUIRE FINE-TUNING. VERIFIED doctrine from vault docs: frozen weights read-only; capability arises from 30B-distilled trajectories (0/1/2-cells) stored in the carved swarm map and recalled by retrieval. No weight update occurs. A model is converted *to capability* by replacing recomputation with recalled, judge-passed reasoning arcs. This is PROPOSED as a claim; certification pending prereg.

## 1. Distillation operator

Let a teacher trajectory be a sequence of steps s_i with hidden states h_i ∈ R^d and next-op distribution p_i. The projector (MGS-orthogonalized, ε=0.08) maps each step into a global atom `a_i` carrying:
- coordinate in (H³ × T²)/Γ₁₂ (lorentz vector),
- tangent slots (top-k directions),
- certainty / teacher entropy,
- payload fingerprint.

Novelty gate: an atom mints only if its nearest existing atom distance ≥ δ (novel) OR it raises retained energy of an existing well (reinforcement).

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

One-pass is valid iff `Σ receipt_ok = atoms_minted` (zero dropped receipts, NN-G3 analog).

## 4. What is NOT fine-tuning

No gradient. No weight mutation. No optimizer state. The distillation is **extraction + storage + retrieval + judge**. "One pass" means one read of the teacher, not one training epoch.

## 5. Open calibration

- α (conversion coefficient) — measure via A/B (held-out).
- p_useful — measure via judge coverage report.
- δ (novelty threshold) — sweep in ablation.
- B_map budget — from C21.
