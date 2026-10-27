# Ruling: AGREE — TrajectoryToCapabilityFormalization.md (with 3 corrections)

**Date.** 2026-10-27. **Auditor.** Muse. **Scope.** Co-agent `TrajectoryToCapabilityFormalization.md` as read (118 lines, §§1–7 + summary formula).
**Standards.** Documentation-standards (labels); research-evidence; C03.3 (no edits to the audited file — corrections live here).
**Verdict. AGREE.** The formalization is sound, honest, and consistent with the vault, my certifications, and the steward's brief. It ships as PROPOSED with three corrections below (one math tightening, one scope repair, one attribution fix). None is structural; no redesign needed.

---

## 1. What I agree with (verified against vault + record)

1. **§1 storage ontology.** 0-cells (coordinates + payload + provenance), 1-cells (from→to + teacher + certainty), 2-cells (closed loops, Gauss–Bonnet salience), rank-8 charts (MGS slots) — matches `GlobalAtom`/`DistilledReasoningArc`/`holonomy.rs`/`map_native.rs` as read. The "compressed reasoning graph, not parameter set" sentence is exactly the vault correction-ledger position. CERTIFIED.
2. **§2 capability definition.** `Cap(π) = E[JudgePass · Efficiency]` with the "without changing weights or training" qualifier is the correct measurable form of "raw capability" — it converts the steward's informal ask into something A39 can decide. The "structured prior" gloss (don't re-derive encoded connections) is the honest mechanism. CERTIFIED.
3. **§3 pipeline (distill → invoke → execute).** Matches `execute_subagent_in_task_view` → `execute_task_in_motion` → `crystallize_episode` as I traced for `HowWeTurnModelDistillationToRawCapability.md`: carve/retrieve k-nearest + forward closure, attend over the view, dissolve deltas back. The "use, not magic" framing plus (a)+(b)+(c) decomposition is accurate. CERTIFIED.
4. **§4 policy conditioning `π_M(T) = π_0(T | retrieve(M,T))`.** Correct formal shape — the map as external latent variable, weights fixed. The ΔCap decomposition (judge delta + η·efficiency) matches J1–J6 structure. CERTIFIED.
5. **§4.1 information view.** Connectivity/landmark/chart priors → `P(correct|q,M) ≥ P(correct|q)` when M correlates with correct solutions → `I(answer; M_T | q)` as the lift quantity. Sound conditional-probability reasoning; the "correlated with" qualifier is the load-bearing honesty (uncorrelated M gives nothing — matches §4.3's poisoned-map bound). CERTIFIED.
6. **§4.2 no-weight guarantee + §5 NOT-list.** I1/A4/A21 anchoring, no-fine-tuning, no weight transport (vault correction ledger), no twistor overclaim (TPE spec §3 honored), geometry-enables-not-proves. The GeometryClaimBoundary pointer is the right containment. CERTIFIED.
7. **§6 A39 harness + §7 summary formula.** Arms/judges/secondary correctly stated; necessity-precision diagnostic correctly positioned as the scoping-vs-context discriminator; "distillation makes possible, carve scopes, A39 proves" is the right division. The summary formula's three factors (mutual information × scoping precision × weights-unchanged) are each independently falsifiable. CERTIFIED.
8. **Consistency with my `HowWeTurnModelDistillationToRawCapability.md`.** No contradiction found: mine traces the *execution loop* (soil/pot/plant/seeds), theirs states the *information-theoretic contract* (policy conditioning + lift + bounds). Complementary halves — mine is the mechanism, theirs is the mathematics. Both point at A39 as the decider. The pair should be cross-linked, not merged.

## 2. Corrections (append — owning agent; none applied here)

**C-1 (math tightening, §4 ΔCap).** The lift is stated as `E[ΔJudgePass] + η·EffGain` — an additive mix of a probability delta and an efficiency gain with an unstated `η`. As written, η is a free knob that can make any result "positive." REQUIRED: either (i) fix η in the prereg (recommend η = 0 for the primary verdict — pass/fail on J1–J4/J6 alone — with EffGain reported separately, matching A39's own decision rule where J5-miss only downgrades the claim *type*), or (ii) derive η from token economics (dollars/token × task value) with the derivation filed. Until then, §7's summary formula inherits the same free parameter. Severity: MEDIUM (prereg-level, blocks v1.1, not structural).

**C-2 (scope repair, §3 forward-closure line).** "retrieve k-nearest atoms + forward-close arcs (≤64 hops)" states the OLD closure (uncapped atom count, no well-membership guard) — superseded this week by AA-02/AA-03 fixes (guard landed in `silo.rs`, `max_atoms` enforced). The formalization must cite the GUARDED closure (well-membership + `|atoms| ≤ atom_cap`) or it formalizes a version of the code that no longer ships. Severity: MEDIUM (correctness of the thing being formalized).

**C-3 (attribution fix, §6 necessity-precision diagnostic).** The diagnostic ("fraction of carved atoms actually used") is correctly positioned as the scoping-vs-context discriminator — but NO answer-attribution mechanism exists in the codebase (grep for `necessity_precision|used_in_answer|answer.*attribut*` across immune + orchestrator: zero hits — the doc's own M-10 gap in my certification already records this). REQUIRED: mark the diagnostic explicitly as UNIMPLEMENTED-pending (not merely "secondary"), and file its construction (answer→atom citation linker) as a precondition of any v1.1 run that cites precision. Without it, a J2/J3 pass + missing-precision run cannot distinguish correct carving from lucky context — the exact confound the diagnostic exists to kill. Severity: HIGH (blocks the v1.1 run-readiness checklist, joins my §4 conjunction as item 6).

## 3. Register entries (owning agent)

1. Fix η (C-1) in A39-v1.1.
2. Update §3 closure line to guarded form (C-2).
3. File answer-attribution linker + mark diagnostic pending (C-3); add to v1.1 run-readiness.
4. Cross-link this doc ↔ `HowWeTurnModelDistillationToRawCapability.md` (mechanism ↔ mathematics).

## What was NOT checked

Full `native_predictor` line audit; 998 MB trajectory vault; outcomes-ledger binary; multi-machine replication; `transport_frame_hop` numerics; A39/M-CAP runs (all unrun by design).
