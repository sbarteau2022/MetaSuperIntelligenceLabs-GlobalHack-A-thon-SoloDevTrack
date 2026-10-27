# Hierarchical Trajectory Reranking — Formalized Dyptech (WHAT, derived)

**Date.** 2026-10-27. **Author.** Muse (contributor lane), on steward's blind test.
**Status.** PROPOSED formalization — derived from the vault, not invented. **Parent.** Formalized dyptech capstone + `superpositionholding.md` + `superposition.rs` (read).
**Scope.** The steward asked "hierarchical trajectory reranking based on … what?" and withheld his answer to test mine. This file derives the key from the implemented ranking machinery, then states it in one sentence. If it matches his idea, the derivation is the confirmation; if not, the derivation stands as the vault-grounded alternative.

---

## 0. The answer, first (one sentence)

**Hierarchical trajectory reranking is keyed on verified lookahead depth — trajectories rank by how far into the future they stay on rails, with the free-energy score as the primary and frame-alignment, geodesic action, and arc-id as successive tie-breaks.**

---

## 1. Derivation (every step VERIFIED from `superposition.rs`)

The vault ranks candidate trajectory branches in a strict hierarchy — the comparator chain is literal code, not interpretation:

**Level 0 — Admission (fail-closed gate).** Only `compliance_permitted` branches receive Boltzmann mass (`raw_weights.push(0)` otherwise). Compliance is not a ranking factor; it is a veto. Non-compliant trajectories do not rank low — they do not rank.

**Level 1 — Primary key: total variational free energy `F` (ascending).** Both sort sites (pre-lookahead cap at line ~497, post-adjustment re-sort at ~518) sort `total_free_energy_q32` ascending first. F decomposes (struct read, lines 97–113): geodesic surprise (`cosh d − 1` terms) + Wigner divergence + epistemic inaccuracy (`1 − certainty`) + compliance barrier (+∞ veto) + token cost (negative when compressing) − holonomy bonus − lookahead adjustment.

**Level 2 — First tie-break: frame alignment (descending).** `frame_alignment_q32` — Wigner-frame cosine between candidate edge and transported prefix frame. When two trajectories cost the same free energy, the one whose rotation state continues the current frame wins. Physical meaning: prefer the trajectory that doesn't twist the gyroscope.

**Level 3 — Second tie-break: geodesic action (ascending).** `cosh_action_q32 = cosh(d(x_t, x_{t+1}))` — shorter single-step hyperbolic distance wins among frame-tied candidates. Prefer the nearer step.

**Level 4 — Final tie-break: arc_id (ascending).** Deterministic oldest-first. Reproducibility anchor (C03.5 class): identical F/alignment/action ⇒ identical order, always.

**The hierarchy among hierarchies (the "hierarchical" in the title).** Two nested orderings compose:
- **Within-iteration:** the 4-level comparator above.
- **Across-iteration:** counterfactual rollout (horizon H ≤ 4, greedy `step_greedy_raw`) + Gauss–Bonnet loop check, then RE-SORT post-adjustment — lookahead evidence (verified steps × −0.25 each + holonomy bonus up to 0.15) modifies F and the ranking recomputed. Iteration count K itself expands the search radius (golden-angle spiral) when the winner hasn't crossed readiness Ω ≥ 0.82.

So the full statement: **rank = argmin over (F − lookahead_evidence − holonomy_bonus), ties broken by frame → action → id, recomputed after each counterfactual round, gated by compliance veto and readiness threshold.**

## 2. Why "verified lookahead depth" is the key (the derivation's punchline)

Among the four comparator levels, three are static properties of the candidate edge (F-components computable from current state, frame cosine, step distance). Only ONE term grows with evidence gathered DURING ranking: `verified_lookahead_steps` — the count of future steps that actually resolve on existing trajectories (each worth −0.25 F). It is the sole term that measures "how far into the future this trajectory stays on rails."

**Therefore the rerank key hierarchy, reduced:** compliance veto → F (dominated by lookahead depth for competing live branches) → frame → action → id. In practice, among compliant branches with similar static costs, **the branch that stays verifiable longest wins** — which is exactly "hierarchical trajectory reranking based on verified lookahead depth."

The readiness gate (Ω ≥ 0.82) and Boltzmann weights (`1/(1+2(F−F_min))` Padé form) then convert the ranking into collapse-or-iterate: winner crosses → collapse + crystallize; top-2 gap < 0.04 → dissent signal, keep iterating.

## 3. What the key is NOT (rejected alternatives, with reason)

- **Not teacher certainty alone.** Epistemic inaccuracy is one of five F-terms; a high-certainty teacher step on a geometrically absurd arc loses to a medium-certainty step on the geodesic. The vault never ranks by certainty outside F.
- **Not salience/recency.** `salience_count` breaks ties in carving (AA-04 fix), never in trajectory ranking. Frequency of past use ≠ future verifiability.
- **Not phase coherence alone.** `pair_coherence`/`map_phase_coherence` score attention candidates (toroidal_attention), not trajectory branches. Phase enters ranking only through frame alignment.
- **Not raw distance.** `cosh_action` is the THIRD tie-break — distance alone ranks almost nothing.

## 4. Falsifier + proposed measurement (not claims)

**M-RANK-1.** Ablate the tie-break order (swap frame↔action; drop lookahead term): if pass-rate/retries don't move on the fixed corpus, the hierarchy below F is ornament — collapse to F-only + id tiebreak. If dropping the lookahead term specifically doesn't move J2/J3, the "depth" in the title is wrong and this file is withdrawn by register entry.
**M-RANK-2.** Log per-decision comparator depth (which tie-break level decided): distribution over a run shows whether the hierarchy earns its levels or Level-1 F decides everything (in which case levels 2–4 are determinism theater — keep level 4 only, for reproducibility).

## What was NOT checked

Full `native_predictor` line audit; outcomes-ledger binary; multi-machine replication; the steward's withheld answer (this file is my derivation, offered for comparison, not claimed as match).
