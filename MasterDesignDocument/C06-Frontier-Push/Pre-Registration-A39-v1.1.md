# C06 — Pre-Registration A39 v1.1 (CANONICAL — supersedes v1)

**Status.** [PROPOSED] Pre-registered; no A39 execution has occurred. **Date:** 2026-10-27.
**Supersedes.** v1 (2026-10-26): N arithmetic fixed (40→48), power plan attached, token confound upgraded to void-for-all, run-readiness checklist added, attribution linker precondition added. v1 preserved in history per ADR-19; results under v1 hash are invalid for v1.1 and vice versa.

**Rule.** [PROPOSED] The canonical SHA-256 of THIS file must be recorded in `Claims_Register.md` before any A39 run. Any later edit creates v1.2 + new hash (ADR-19, ADR-22).

## A39 — Map-Carving Assignment A/B (Non-tuned Subagent Without Map vs. Map-Assigned)

**Hypothesis.** [PROPOSED] Unchanged from v1 (paired task set, equal-or-better quality, ≤1.10× tokens, ≤0.80× tool calls, ≤0.70× retries via domain scoping + radial salience).

**Treatment / Arms.** [PROPOSED] Unchanged from v1 (lawful distillation §3; Arm A = capsule + BM25 dump, λ_map = 0; Arm B = Arm A + carved D_T with dissolution + Merkle verification per §3.2 as amended by PATCH-02 accumulator).

**Paired workload (FIXED — v1 arithmetic corrected).** [PROPOSED] **48 seeded tasks**: 8 per primary class × 6 classes (code_change, debug, research, review, plan, general) per ReasoningDistillationMapCarving.md §5.1. (v1 stated "at least 40" with "8 per primary class" — 6×8 = 48, not 40. N = 48 paired runs.) Each scenario MUST contain: canonical objective, ≥3 invariants, acceptance criteria, commitments, pending actions, one next action, decoy context, ≥1 tool receipt. Seeds, task order, expected values frozen in the run manifest. Each task runs both arms (paired).

**Power plan (NEW — C09.9 sample-size requirement).** [PROPOSED] Before the run, file per-judge minimum detectable effects at 80% power, α = 0.05 (paired tests): J1 (+10 pts pass rate), J2 (0.80× median tools), J3 (0.70× median retries), J4 (0.50× error rate). N = 48 pairs; if power calc demands more, N grows BEFORE the run (preregistered amendment with new hash), never after. Interim analyses at 50% with Lan-DeMets α-spending (ρ = 2, house standard); futility stop if conditional power < 0.2.

**Experimental sequence.** [PROPOSED] Unchanged from v1 (Arm A → reset → Arm B; failed dissolution = failed run).

**Primary judges.** [PROPOSED] J1–J6 unchanged from v1 (thresholds frozen).

**Secondary judges.** [PROPOSED] Unchanged from v1, PLUS: answer-attribution linker output per run (see precondition 6); per-run comparator-depth log (which tie-break level decided — feeds M-RANK-2).

**Decision rule (AMENDED — item 6 upgraded to void-for-all per B-7).** [PROPOSED]

1.–5. Unchanged from v1 (FAIL/PASS-CORE/scoping/replication+necessity/record-keeping).
6. **Confound control (STRONG FORM):** any run where Arm B receives > 10% more total tokens than Arm A is VOID FOR ALL JUDGES (not J5 only) — the pair is dropped, not caveated. Rationale: token surplus confounds efficiency AND quality AND pass-rate (more context can rescue bad carving). A caveat cannot unconfound a paired comparison. The v1 weak form is withdrawn by this entry.
7. **Attribution precondition:** no scoping claim (decision rule 3) may pass unless the answer-attribution linker (precondition 6 below) reports necessity precision on that run. J2/J3 pass + missing precision = INCONCLUSIVE on scoping (efficiency without attribution), not evidence of correct carving.

**Run-readiness checklist (NEW — ALL must be true before the first run; harness enforces, not docs).**

1. `atoma_immune` + `orchestrator` test suites green at a recorded commit (pressure-test standard; no mid-edit-tree runs).
2. AA-02 calibrated guard landed (PATCH-01: basin + overlap, not flat 10.0) + property test green.
3. PATCH-02 accumulator landed (SHA256(pre ‖ write-set); XOR retired in code AND §3.2 text).
4. B-D1 receipt split landed (`bridges_discovered` vs `SYNTHESIZED` fallback; treatment counts discovery only).
5. Necessity formula (§5.2) + λ_map (§6.2) implemented and unit-tested (MethodAndMath §8 items 1–2).
6. **Answer-attribution linker built**: maps final-answer spans → carved atom ids (receptor-sketch cosine + receipt chain); reports necessity precision per run; validated on a synthetic set where ground-truth attribution is known (precision = 1.0 on the oracle set required before live use).
7. v1.1 hash filed in `Claims_Register.md` (C11.3).

**Dependencies / non-goals / honesty footer.** [PROPOSED] Unchanged from v1 (§ Dependencies, § What A39 deliberately does not test, Not measured here — re-affirmed).
