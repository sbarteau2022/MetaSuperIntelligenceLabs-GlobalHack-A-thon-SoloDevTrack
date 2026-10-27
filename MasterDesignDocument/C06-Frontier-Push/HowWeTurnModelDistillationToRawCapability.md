# How We Turn Model Distillation Into Raw Capability

**Date.** 2026-10-27. **Author.** Muse (contributor lane), on steward's directive.
**Status.** PROPOSED formalization — derived from vault code read this pass. **Parent.** Formalized dyptech capstone + carving methodology + TPE spec.
**Scope.** The execution half the distillation doc leaves implicit: the exact path from stored charts/trajectories to acting capability, with code refs, followed by the steward's 12 GB-slice / 75%-fill model-class comparison, computed from measured byte budgets.
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no vault file modified).

---

## 1. The conversion pipeline (VERIFIED — code read, five stages, no gaps)

Nothing in the map acts. Capability = a live execution loop reading the map as constraints. The five stages, each with its entry point:

**Stage 1 — Address.** `resolve_geodesic_prompt_prefix` (continuous_active_mind): prompt → ingress MlirOps → standing coordinate `(LorentzVector, Phase)` on the map (origin + zero-phase fallback). The task is *placed* before it is *executed*. Entry: prompt text. Exit: standing coordinate or origin.

**Stage 2 — Role.** Nearest-atom lookup → owning silo → role (`execute_subagent_in_task_view`, lines ~2714–2745); fallback = nearest icosahedral well. The task is assigned a specialist identity by geometry, not by router heuristics.

**Stage 3 — Carve (specialization by subtraction).** `carve_task_view_for_role` builds the isolated view: allowed silos + optional horizon + forward closure (64-hop cap AND `atoms.len() < atom_cap`, AA-02 well-membership guard landed) → `to_isolated_swarm_map` materializes a sub-map where out-of-scope atoms/arcs are *physically absent*. Capability here is negative: the agent is powerful because of what it cannot see. Entry: role + policy + horizon. Exit: isolated `CarvedSwarmMap`.

**Stage 4 — Motion.** `execute_task_in_motion` runs inside the view: exact-chain check → novel-trajectory bridging (`discover_and_bridge_novel_concept_trajectories`, cert 0.96) → per-step `active_inference_step` (greedy arc-follow; on `NoDirectGeodesic`/`FixedPointOrCycle`/`WignerAmbiguity`/readiness-spike/boundary → `hold_and_iterate` superposition → counterfactual rollout H ≤ 4 → Gauss–Bonnet loop check → collapse or fail-closed). Every step is receipted (F, Ω, superposition count, EOS break). This is where charts become action: the Rank-8 tangent slots and receptor sketches steer `candidate_arc_atoms_near` retrieval; the Wigner frames steer alignment tie-breaks; the arcs ARE the step options.

**Stage 5 — Crystallize (learning half).** Emitted ops → tentative steps → `crystallize_episode`: holonomy gate FIRST (echo chamber → refuse with error, no partial write), then `record_step` per tentative step → `novel_arcs_minted`. Today's execution becomes tomorrow's trajectory. The loop closes: map → view → motion → map.

**The one-sentence formalization.** Charts store local linearizations, trajectories store transitions, and capability = a carved view constraining a live inference loop whose verified outputs re-enter the map as new arcs. Soil (map), pot (carve), plant (superposition engine), seeds (crystallization). Remove any one: storage, not capability.

---

## 2. The 12 GB slice: what fits, measured

Steward's box: 48 GB unified (VERIFIED `sysctl`), daily driver, 12 GB slice for model + map, KV nested in the map (no separate dense KV allocation).

**Measured anchors:**
- Rank-8 chart ≈ **24,700 B** (vault census median, n=297 — matches formula exactly).
- Rank-1 spine ≈ **3,108 B**.
- Nemotron-3.5-Lightning-30B-A3B 4-bit on disk = **17 GB** — does NOT fit the slice with a map. Neither does Qwen3.8-27B 4-bit (~13.5 GB).
- Dense-KV counterfactual (what nesting SAVES, computed from the 3.5 config: GQA kv=2, head_dim=128, 52 layers, f16): 8k ctx → 0.44 GB; 32k → 1.74 GB; **128k → 6.98 GB**. A dense 128k-context 30B-class serving would burn ~7 GB on KV alone — more than half the slice. Nested coordinates (x₀..x₃ + 2 phases + payload ref ≈ tens of bytes per resident) replace this. The saving is structural, pending A1 measurement.

**Slice budget (12 GB, 75% map fill):**

| Component | Bytes | Notes |
|---|---|---|
| Base serving model (8–9B class, 4-bit) | ~4.5–5.0 GB | Fits: e.g. Qwen3-8B-class or Glimmer-8B-class; 30B-class does NOT fit with a map |
| Map at 75% fill | ~5.25–7.0 GB | 75% of a 7 GB map budget = ~5.25 GB ≈ **~210,000 rank-8 charts** (+ arcs/KV/index in the remainder) |
| Headroom / transient fibers | ~0.5–1 GB | Working sets, dissolution transients (C30.9 backpressure owns this) |

**Definition of "75% fill" (locked for this comparison).** Coverage fraction per M-CAP-1: share of held-out teacher behaviors within cosh-reach R of a chart anchor. 75% fill = 3 in 4 needed behaviors retrievable at chart precision; 1 in 4 falls back to base model + superposition bridging + dream minting. UNMEASURED today — the entire §3 comparison is conditional on M-CAP-1. If coverage saturates below task need at max budget, the answer changes to "bigger budget or narrower domain," not "geometry failed."

---

## 3. Model-class comparison (PROPOSED — conditional on M-CAP-1 + A39)

No training is assumed anywhere below: the base model's parametric intelligence is FIXED at ~8B. The map adds *retrievable reasoning*, not parameters. The comparison is inference-time task capability, split by coverage:

**On covered ground (the 75%): 30B-teacher class.**
Each chart is a verified local linearization carrying the teacher's fingerprint, certainty, and MGS-orthogonalized tangent slots (ε=0.08) — the subagent walks the same decision geometry the 30B teacher (Nemotron-3.5-30B, Qwen3-class/QwQ-32B trajectories, REPORTED provenance) walked, with teacher-certified step options at every arc. Expected task behavior: **30B-MoE-class reasoning quality on covered domains** — not because the weights are 30B (they are 8B), but because the *trajectories* are 30B-distilled and the carve excludes everything else. Count comparison: ~210k rank-8 micro-experts vs a 128-expert MoE — three orders more experts, each narrower (rank ≤ 8 vs full FFN), each receipted.

**On uncovered ground (the 25%): 8B base + bridging — strictly better than bare 8B, strictly below teacher.**
Fallback chain: exact-chain miss → superposition bridging (novel-concept composition at cert 0.96) → dream minting (new arcs, quarantined then promoted). The system degrades to *search* where it lacks *recall*. Expected: above bare-8B (bridging finds paths a flat 8B cannot), below 30B (no teacher trajectory to walk).

**Blended statement (the comparison to quote).**
A 12 GB slice (8B base + ~210k-chart map at 75% fill) behaves like **a 30B-class MoE with 75% expert coverage**: teacher-grade on 3/4 of the workload, resourceful-8B on the rest, with the uncovered fraction shrinking as dream minting promotes bridges (A40 measures exactly this shrinkage rate).

**Dense-equivalent denominations (for the "how much is that in Euclidean" question):**
- Byte parity: 12 GB ≈ **24B BF16 params** of raw bytes. Stated with the caveat: bytes ≠ capability; charts are constrained local maps (rank ≤ 8, orthogonalized, certified), not general weights.
- Per-chart structure: each 25 KB chart replaces a dense h×h Jacobian (21× at h=512, 400× at h=6656 — measured ratios from the capacity doc). The 210k charts ≈ **4–80B dense-equivalent Jacobian bytes** depending on h_dim mix — the rank constraint is doing the compression work Euclidean cannot do without training it in.
- Serving amortization (no extra charge): one map + one weight buffer serves all roles (base=1, views=N). Euclidean multi-specialist equivalent pays N model copies or N adapters + router; the map pays N transient projections.

**Falsifiers (all PROPOSED, all preregistered before any run).**
- A39 (map vs no-map, same base): null result on J2–J4 kills the covered-ground claim outright.
- M-CAP-1: coverage-vs-budget curve decides whether 75% is even reachable in 7 GB for the target domain set.
- M-CAP-3 (charts vs dense Jacobians vs LoRA, bytes-vs-recall): decides whether the 21–400× survives end-to-end or dissolves in overhead.
- If any fires negative, this section is withdrawn by register entry — the comparison is staked, not asserted.

## What was NOT checked

A39/M-CAP-1/2/3 unrun; base-model selection (8B-class named, not pinned — steward's call); multi-machine replication; 998 MB trajectory vault contents; full `native_predictor` line audit; CUDA portability (out of scope per C01/C09.4 staging).
