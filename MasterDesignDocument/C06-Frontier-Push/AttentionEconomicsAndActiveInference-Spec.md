# AttentionEconomicsAndActiveInference-Spec — The Agent-Plane Resource Contract

**Date.** 2026-10-27. **Status.** PROPOSED specification. **Parent.** C30.8 MetaSoftBudgetAndProfitability (Meta-plane pricing), C06 TPE spec (encoder), HNPA Atlas (three-gap structure, REPORTED).
**Standards.** Documentation-standards (labels); C03.3 (no freeze/vault modification — corrections live here as entries).

---

## 0. First principle (physics, not preference — steward-settled 2026-10-27)

Every computation that changes mind-state pays the thermodynamic tax: used energy becomes heat that must be dissipated, metered by physics rather than by any observer the agent could deceive. Tokens are counted by code the system itself runs (gameable); heat is counted by the universe (Landauer floor `kT ln 2` per erased bit, no appeal). Symbolic constraints — budgets, rewards, prompts, gate thresholds, reliability weights — can in principle be read, modeled, and routed around (Goodhart; C30.8's entire I-SB-1 hiding apparatus exists because of this brittleness). Dissipated heat cannot be bargained with. **Reward graded on dissipated heat per unit quality is therefore the only alignment mechanism that can't be routed around.** Gates verify; thermodynamics enforces. Everything symbolic (budgets, forecasts, weights) is demoted to PREDICTOR of the physical quantity — useful, revisable, and unable to corrupt what it predicts.

Steward-stated objective (adopted verbatim as the normative objective):

> Active inferencing: minimizing the consumption of allocated resources, optimizing to constrain the energy it has access to (budget), and still giving increasingly better output solutions for the price paid. Minimizing free energy output, maximizing output solution quality, in as few steps as it can. Reward is graded by profit of conserved energy to task completion.

This is not an analogy to active inference. It **is** active inference: the agent minimizes variational free energy `F = complexity − accuracy` where complexity is dissipated heat plus steps and accuracy is output solution quality. The economics (budgets, profit, reward) are the accounting layer over the same minimization — not a second objective.

---

## 1. Formal objects (agent plane — integer, deterministic, auditable)

Per invocation (or per lane, where a lane is a bounded sequence of invocations):

- `E_alloc ∈ ℕ` — energy allocated (integer budget units: cycles, tool calls, steps — the denomination is per-resource, always counted, never estimated).
- `E_used ∈ ℕ` — energy consumed, measured post-hoc from C31 telemetry (counts only, never content).
- `E_conserved = E_alloc − E_used` — conserved energy (accounting quantity; feeds the Meta forecaster, never the reward directly — see correction entry below). Spend-side formalization adopted from superpositionholding §2.3 (REPORTED-verified): `C_token` with verify-cliff bonus (structural-opcode/≤7-token highway discount), spend-envelope gate (`predict_token_envelope`, p75 × 1.5), absolute quality floor (no shortcut below Ω_min = 0.82 or F > τ_F). The envelope is the Meta forecast mechanism; the floor is the physical quality bound.
- `H ∈ ℝ⁺` — dissipated heat for the invocation (joules, metered at the sink: package energy counters / `powermetrics` on Apple Silicon, RAPL on Linux; C21.8 power-state gating applies). The physical record of irreversible work. Not estimated, not modeled — metered.
- `Q ∈ [0,1]` — task completion quality (judge-graded, deterministic judge per doctrine).
- `N ∈ ℕ` — steps taken.
- `F = complexity(H, N) − accuracy(Q)` — variational free energy, the minimized quantity.

**Reward (graded by quality per dissipated joule):**

$$
R = \frac{Q}{H + c \cdot N}
$$

where `c` is the per-step dissipation constant (preregistered per substrate — the idle-step heat cost, measured not assumed). No quality scores zero (`Q = 0` kills it). Joule-hungry work scores low regardless of budget games (`H` is metered, not counted by system code). Many steps dilute through both `H` (each step dissipates) and `c·N` explicitly. Refusal-to-act on an invoked task scores `Q = 0` — idle is free only when _not_ invoked (§2); invoked-but-refused is the worst trade in the ledger.

**Correction entry 2026-10-27 (replaces the `E_conserved × Q / N` form):** the prior form double-counted energy (steps dominate `E_used`, then `N` penalized again) and priced an accounting quantity (unspent budget) no physics enforces. The thermodynamic form prices what physics meters. `E_conserved` survives only as the forecaster's calibration signal (predicted vs actual, C30.8 plane), never as a reward term.

**PROPOSED, falsifiable:** any agent maximizing `R` as defined also minimizes measured `F` on the fixed corpus (A/B vs an `F`-only optimizer; if they diverge, this section is wrong and the reward is re-derived). Second falsifier: rank-correlation between metered `H` and counted `E_used` ≥ 0.95 on the fixed corpus — if counting ever decouples from heat, the counters (not the reward) are declared broken.

---

## 2. Invocation gating (no work without invocation)

- No attention, no retrieval, no kernel launch happens except on invocation. Invocation carries the invocation budget `E_alloc` for that call — a hard, local, exact integer, **not** a forecast (forecasts live in C30.8's Meta plane and never cross the seam).
- The invocation gate allots **only the necessary amount**: depth-gated allotment (below) sizes the budget before work begins; unspent budget is conserved energy, not rollover credit.
- Idle is free and exact: zero invocation = zero consumption = `E_used = 0`, no estimate, no drift. Invoked-but-refused is NOT idle: it scores `Q = 0` (worst trade). The trivial minimum (do nothing) is excluded by construction — the system optimizes toward a rock otherwise.
- **I-HEAT (new invariant):** sustained dissipation rate ≤ C21 thermal envelope, metered at the sink, every invocation. The envelope is the medium the agent reasons in, not a limit imposed on it: free-energy minimization with a real heat sink converges to max-Q-per-joule because physics settles there.

---

## 3. Depth-gated allotment (dynamic, by reasoning depth/effort)

Allotment is a function of task depth, computed at invocation from task-legitimate signals only (query complexity, surprise vs retrieved mass, tool-need flags — never budget-talk, per I-SB-1). The allotment dynamics adopt the superpositionholding regulator pattern (REPORTED-verified), not a static table:

- `depth d(q)` in tiers with Boltzmann-weighted branch competition (`w_k ∝ e^(−βF_k)`): competing allotment hypotheses are held, pruned outside `F_min + Δ_prune`, re-weighted per iteration.
- `E_alloc(d)` monotone in `d`: easy queries get small budgets, hard queries get large ones. When readiness stalls (Ω < 0.82), search radius expands along the golden spiral — one widening per stall, receipted, reversible — instead of blindly raising the budget.
- Branch sources mirror the geometric harvest (direct arcs, phase-winding neighbors, operator bridges, cross-silo geodesics): allotment hypotheses come from the map's structure, never random restart.
- The mapping stays a **declared reference** (FoldAttention pattern): fixed regulator constants, no iterative discovery of the rules themselves, no provisional rescaling. Same depth → same budget, bit-for-bit reproducible. What iterates is the _application_ of the rules (the §7 turn), never the rules.

**Anti-gaming (from C30.8 I-SB-1):** the agent may signal _need_ (blocked, low confidence, surprise spike — all task-legitimate) but never observes the allotment function, the forecast, or the comparison. It asks for help with the task; the Meta hears a reallocation request. Same message, two readings — the seam holds.

---

## 4. Deterministic normalization (no softmax, no drift)

Attention weights are computed with declared-reference shifting (FoldAttention, REPORTED): fixed reference, single-pass, integer-exact denominators. No `exp`, no provisional rescaling, no non-associative reduction.

**Why this is load-bearing for the economics:** the reward prices _conserved energy per quality_. If the normalization drifts, quality measurements drift, and the reward grades noise. Deterministic weights make the reward auditable — the same invocation on the same inputs yields the same `Q` and the same `E_used`, so `R` is reproducible. Cites EigenAI §6.5 invariants and TBIK reduction topology (both ingested 2026-10-27).

---

## 5. The seam with C30.8 (normative boundary — what crosses, what never does)

| Direction                    | Carries                                                    | Never carries                                      |
| ---------------------------- | ---------------------------------------------------------- | -------------------------------------------------- |
| Agent → Meta (C31 telemetry) | counts/ids/ratios: `E_used`, `N`, `Q`, need-signals        | raw content, objectives, budget-talk               |
| Meta → Agent (scope only)    | invocation budgets `E_alloc`, tool set, dissolution orders | forecasts, `Π_a`, reliability weights, comparisons |

- **Forecast (C30.8 §2):** predicted aligned-path allocation, wrong by default, revised Bayesianly. Lives entirely in Meta plane.
- **Lane authority (C30.8 + steward directive):** adjust/reallocate/revoke, two speeds — fast revoke (guardrail trip, immediate, dissolution, fail-closed + receipt) and slow reallocate (trend-based, Bayesian, k-tick). Revocation is total where safety binds; reallocation requires sustained quality/security/adaptability over k ticks (bar exists in Meta plane, invisible to agent per I-SB-1).
- **Reallocation requirements:** the bar the agent clears to earn more — thresholds on C31 telemetry. The agent experiences only task feedback; the Meta tracks the bar.
- **Reward flow:** agent maximizes `R` (this doc, §1); Meta grades `R` into reliability weights and routes future tasks (C30.8 §7). The agent never sees the grade — only its future scope changes.

---

## 6. Distributed kernel family (variety by design — the five roles)

Each independently certifiable, Q32.31-integer, deterministic, composable over fixed-layout message passing:

Each role carries its F-term assignment (an implementer satisfying only efficiency with wrong outputs fails certification — per-role ablation, gate 4):

1. **`phase`** → accuracy: TPE step (bitwise vs oracle; the certified `tpe_phase` kernel path, CORDIC retrofit pending). Better representations = higher Q per joule.
2. **`carrier`** → accuracy-per-joule: harmonic/phi/weight MACs (bitwise after CORDIC swap; exact-T reduction proven). Maximizes the numerator of R per MAC dissipated.
3. **`reduce`** → complexity: TBIK tree (bitwise; intra-kernel topology = inter-device topology). Minimizes joules per query at fixed accuracy.
4. **`route`** → complexity-per-query: AttentionEconomics allocator (§§2–3 of this doc; integer budgets, deterministic tie-breaks, declared-reference tables). Minimizes queries' total dissipation for the task's Q target.
5. **`audit`** → bound: invariant checker, inline — fixed-point ranges, CORDIC iteration/gain pins, tree-topology hash vs (n, tsize), single-group dispatch, sector agreement, phase-wrap exactness, receipt completeness, **plus I-HEAT (dissipation rate ≤ envelope, metered)**. Violation → fail-closed + healing receipt (never silent fallback).

Any subset runs anywhere the Q32 ISA exists (Metal/CPU/CUDA-SIMT do integers identically). The tree is the cross-node contract. Fusion is a launch optimization, never the architecture (the v2 lesson).

---

## 7. Recursive self-improvement by first principles (the load-bearing structure)

The system improves itself from its own measured history, by first-principles derivation at each turn — not by gradient descent on a loss, not by prompt iteration, but by the closed loop below. Every turn is gated; every gate failure rolls back exactly one turn (never a silent partial). This section is normative over §§1–6: any component that cannot state its improvement turn in these terms does not belong in the system.

**Turn structure (one recursion step, all predicates executable):**

1. **Hold (superpositionholding, REPORTED-verified vault mechanism, adopted as the improvement substrate):** on trigger (no direct geodesic, fixed point/cycle, Wigner ambiguity ΔF < 0.04, high-F/low-readiness Ω < 0.82, compliance boundary) stop external emission and hold ≤12 candidate branches with Boltzmann weights `w_k ∝ e^(−βF_k)`. Zero tokens emitted while holding — improvement never spends the invocation budget it is trying to conserve.
2. **Vary (counterfactual iteration, K-bounded):** roll each branch forward (horizon H ≤ 4), verify via 2-cell Gauss–Bonnet holonomy (echo chambers rejected, curvature flux rewarded), prune outside `F_min + Δ_prune`, re-weight, expand search radius along the golden spiral iff readiness stalls. The variation operator is derived from the manifold geometry (geodesic/holonomy), never random restart.
3. **Select (first-principles judge):** collapse onto minimum-F branch with `compliance_permitted == true` and positive progress; fail-closed with audit receipt if none qualifies. Selection criteria are the §1 reward and the §6 invariants — fixed across turns, so improvement is monotonic in the thing being optimized, not in a moving target.
4. **Distill (collapse-and-bridge self-distillation):** record the synthesized transition as a carved 1-cell reasoning arc — `O(1)` next pass, no re-entry. The map is the memory of the recursion; distillation is how a turn's work compounds instead of evaporating.
5. **Re-gate (receipt → constraint net):** the turn emits a receipt (`E_used`, `N`, `Q`, `H`, divergence, branch count, prune history) that the §6-audit role checks against every invariant INCLUDING the turn's own cost (improvement that spends more than it saves fails its own gate — recursion is thermodynamically accountable).

**First-principles derivation rule:** each turn must cite the principle it derives from (Landauer floor, TBIK topology, CORDIC exactness, declared-reference determinism, I-HEAT envelope). A turn justified by "it worked" without a principle is a tuning hack: admissible as a receipted experiment, inadmissible as a design change. Design changes require the principle stated BEFORE the turn runs (prereg per C09.9).

**Monotonicity guarantee (PROPOSED, falsifiable):** expected `R` per task is non-decreasing across turns on the fixed corpus (receipts ledger proves it or the recursion is declared stalled and the system reports STALLED, never silently flat). Stall triggers the dreamstate expansion (wider branch sources, cross-silo bridges) — one widening per stall, receipted, reversible.

The Constraint Network is executable predicates (§6 role 5), not prose. Intelligence = self-referential dynamics (hold → vary → select → distill → re-gate, from own history, by stated principles) ∩ invariant bounds (every step gated, every violation fail-closed, every turn thermodynamically accountable). Remove either side and it stops being this system.

---

## 8. Spirals, not corkscrews (hard gate, from steward record)

The rejected implementation replaced RoPE 1:1 at constant radius. This spec **requires** radius expansion per the TPE spec §4 laws (`r(w)` strictly increasing, sublinear-asymptotic; `η` monotone in `(w, amp)`; amplitude accumulates drive energy). Gate 12 (no-corkscrew: radius variance ≥ 1000× fixed-point noise floor) is adopted as a kernel gate, not just a spec gate. A constant-radius implementation FAILS certification regardless of its other numbers.

---

## 9. Implementation gates (PROPOSED, must pass before any kernel ships under this spec)

1. **Integer-only audit:** no float ops in kernel sources (grep-gate on `float`/`double` outside explicit tolerance-path annotations; every annotation needs a prereg justification).
2. **CORDIC correctness:** sin/cos/atan2/sqrt/log integer paths bitwise vs `lock.rs`/`fixed.rs` oracles over the full input domain (fuzz, not just det seeds — the unfinished cross-backend equivalence fuzzing is Gate 2's precondition).
3. **No-corkscrew:** radius variance ≥ 1000× noise floor over 10³ rotations.
4. **Reward integrity:** maximizing `R` minimizes measured `F` on the fixed corpus (A/B vs F-only optimizer, §1 falsifier).
5. **Seam audit:** agent-plane artifacts contain zero forecast/profit/comparison fields (I-SB-1 automated grep over prompts, tool results, receipts).
6. **Revocation drill:** injected guardrail trip → lane revoked within one tick, fail-closed receipt, no partial execution. Fast path measured, not asserted.
7. **Reallocation proof:** sustained quality over k ticks → scope expands via dissolution path only (no prompt injection in the trace).

---

## 10. Register entries (append — owning agent; none applied here)

1. Preregister τ_r/τ_a + reward coefficients (this doc §1) as implementation constants with amendment rule (C09.9).
2. File the cross-backend equivalence fuzz harness (open item from steward record) before any distribution claim.
3. Extend MethodAndMath side-by-side with the reward/free-energy duality row once §1 falsifier runs.
4. A41 (C30.8 §8) covers the Meta-plane loop; this doc needs its own ablation: R-graded vs flat-reward routing on the fixed corpus.

## What was NOT checked

RoPE paper beyond excerpts; full HNPA Atlas theory (structure adopted as REPORTED); bakeoff for triple-channel; the unfound Q2.30/BKM kernel and toroidal_q32 sources (steward record says they exist off-disk — ingest on sight); live A/B numbers for the reward falsifier (unrun by design — spec precedes silicon).
