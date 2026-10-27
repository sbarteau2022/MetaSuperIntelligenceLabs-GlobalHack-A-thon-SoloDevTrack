# C06 — Pre-Registration A39

**Status.** [PROPOSED] Pre-registered; no A39 execution has occurred in this workspace. **Date:** 2026-10-27.

**Rule.** [PROPOSED] The canonical SHA-256 of this file must be recorded in `Claims_Register.md` before any A39 run. Any later edit to this file creates a new registration version and invalidates results obtained under the old hash (ADR-19, ADR-22).

## A39 — Map-Carving Assignment A/B (Non-tuned Subagent Without Map vs. Map-Assigned)

**Hypothesis.** [PROPOSED] A non-tuned subagent that receives a lawfully carved swarm map `D_T` (Arm B) will complete the same paired task set with equal-or-better output quality while using no more than 1.10× the token spend and no more than 0.80× the tool calls and 0.70× the retries of an otherwise identical non-tuned subagent that receives no carved map (Arm A), because domain scoping and radial salience compress the reasoning trajectory space and reduce off-target exploration.

**Treatment.** [PROPOSED] Lawful reasoning distillation per `ReasoningDistillationMapCarving.md` §3: classify task → select silos → score basins → depth-gate → radius carve → serialize to `D_T` under `λ_map` (§6), plus dissolution per §3.2. Uses the non-tuned model; no prompt engineering beyond the frozen map-injection template.

**Baseline arm (Arm A).** [PROPOSED] Same subagent, same Continuity Capsule exact state, same task prompt, same model/decoding/tools/policy, but no `D_T` (`λ_map = 0`); context augmented with a flat, top-k BM25 dump over the selected domains. This isolates the *structural* contribution of the carved map from the capsule's exact-state contribution.

**Treatment arm (Arm B).** [PROPOSED] Arm A + carved `D_T` injected under the §6.2 budget, with dissolution and Merkle-fingerprint verification per §3.2.

**Paired workload.** [PROPOSED] At least 40 seeded tasks across the classes in `ReasoningDistillationMapCarving.md` §5.1, at least 8 per primary class (code_change, debug, research, review, plan, general). Each scenario MUST contain: canonical objective, at least three invariants, acceptance criteria, commitments, pending actions, one next action, decoy context, and at least one tool receipt. Seeds, task order, and expected values are frozen in the A39 run manifest before execution. Each task runs both arms (paired).

**Experimental sequence.** [PROPOSED] For each task: run Arm A to completion (record judge + telemetry); reset to the same initial capsule/prompt state (no cross-arm leakage); run Arm B to completion. Same model, decoding parameters, tool policy, and initial context. Arm B performs carve + dissolve round-trip; an Arm B run with a failed dissolution is a failed run, not a silently partial one.

**Primary judges.**

| ID | Metric | Frozen pass threshold | Deterministic check |
| --- | --- | --- | --- |
| J1 | Task pass rate (judge-gated) | Arm B ≥ Arm A + 10 points, OR Arm B ≥ 90% if Arm A ≥ 90% | Deterministic rubric; no LLM scoring of the primary verdict |
| J2 | Median tool calls per task | Arm B ≤ 0.80 × Arm A | Tool ledger count |
| J3 | Median retries per task | Arm B ≤ 0.70 × Arm A | Retry-signature ledger |
| J4 | Error rate (failed / total) | Arm B ≤ 0.50 × Arm A | Failed-task receipts |
| J5 | Total token spend per completed task | Arm B ≤ 1.10 × Arm A | Tokenizer-counted; confound control in §8.6 |
| J6 | Output quality (canonical rubric) | Arm B ≥ Arm A, no regression | Rubric check |

**Secondary judges.** [PROPOSED] p50/p95 wall-clock per task; peak Metal/VRAM; `weight_copies` invariant = 1 in both arms; fiber dissolution success rate = 100% in Arm B; digest fit `|D_T| ≤ B_map` for every Arm B run; necessity precision ≥ 0.60 (fraction of carved atoms actually used in the final answer) as a diagnostic; classifier-confidence gate trigger rate.

**Decision rule.** [PROPOSED]

1. A39 **FAILS** if any of J1–J4 or J6 misses threshold. Record in `Failures.md`; map-carving claim cannot ship.
2. A39 **PASSES CORE** if J1–J4 and J6 pass. If J5 misses, the claim ships as "trajectory efficiency" only (not "cost reduction"), and `λ_max` must be re-derived.
3. Scoping claim (J2 and J3 pass *and* necessity precision ≥ 0.60) is required before claiming the gain comes from *correct carving* rather than "more context."
4. Before the methodology becomes *required* (not optional), it must replicate on a second distinct machine and pass a necessity ablation showing that removing the carved map regresses J2 or J3 (C06.13).
5. A failed arm remains part of the record. A retry needs a new A39 version + newly filed hash; prior results are never overwritten.
6. **Confound control**: any run where Arm B receives > 10% more total tokens than Arm A is void for J5 but may still be evaluated for J1–J4/J6 with a recorded caveat.

**Dependencies.** [INFERRED] C05 Atlas (coordinates, VP-tree, depth gate); C06.12 Holonomy (fiber projection/dissolution, `weight_copies = 1`); C06.17 Continuity Capsule (exact critical state, M3 evidence paths); C24 compaction/fold; C29 effect system; C31 telemetry privacy (`M_T` carries ids/hashes only); C35 geometry interop (isometry gate must hold for carving distances); C34 lane personas (interference estimate). The exact critical-state plane must not depend on the carved map.

**What A39 deliberately does not test.** [PROPOSED] Hyperbolic-geometry speed (A1); weight-copy invariance (A4/A21); capsule exactness (A38); fine-tuned-agent necessity (both arms non-tuned); end-to-end "active inference swarms at scale" (requires A5 + A21 + A39 together).

**Not measured here.** [VERIFIED by workspace inspection] No implementation, benchmark, hardware measurement, hash-filed run, replication result, or necessity result exists for A39 in this workspace at registration time. No external literature search was performed, so public novelty is not claimed.
EOF
