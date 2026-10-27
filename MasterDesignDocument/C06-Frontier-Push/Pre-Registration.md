# C06 — Pre-Registration

**Status.** PRE-RUN registration for C06 frontier experiments. **Date:** 2026-10-26. **Version:** A38-v1.1.

**Rule.** [PROPOSED] The canonical SHA-256 of this file must be recorded in `Claims_Register.md` before any A38 run. Any later edit to this file creates a new registration version and invalidates results obtained under the old hash (ADR-19, ADR-22).

## A38-v1.1 Amendment — 2026-10-26

- **Supersedes.** A38-v1, preserved verbatim as `Pre-Registration-A38-v1.md` with SHA-256 `28716ac077059c248a0720565dde032cc4341212a0ec82ab8543b4155a89eea8`.
- **Reason.** [VERIFIED by adversarial review] The first filed version did not define the deterministic response parser for M1 and did not state whether M3 is required in an Atlas-disabled run.
- **Clarification 1.** [PROPOSED] M1 parses the agent's final response as strict JSON with the fixed schema `{"objective": string, "invariants": string[], "acceptance_criteria": string[], "commitments": string[], "pending_actions": string[], "next_action": string}`. Parse errors, extra keys, missing keys, or duplicate list entries count as field-check failures.
- **Clarification 2.** [PROPOSED] M3 is mandatory for every Atlas-enabled treatment run and is reported `NOT_APPLICABLE` only for the explicitly Atlas-disabled fallback arm. An Atlas-enabled run with missing or invalid supporting proofs fails M3.
- **Scope of amendment.** [PROPOSED] No baseline, workload, experiment sequence, numerical threshold, or decision rule changes. The v1 hash is retained as history; only the v1.1 hash may authorize a run.

## A38 — Continuity Capsule Round-Trip, Boundedness, and Proof

**Status.** [PROPOSED] Pre-registered; no A38 execution has occurred in this workspace.

**Hypothesis.** [PROPOSED] A content-addressed Continuity Capsule can preserve the exact critical state of a long-horizon agent through two forced compactions and one process restart, while using less prompt context than a flat transcript and exposing a verifiable evidence path for every recalled supporting atom.

**Treatment.** [PROPOSED] Continuity Capsule control plane (typed manifest + minimal protected payload) plus optional Atlas supporting-atom recall and ledger-backed evidence receipts, as specified in `Design.md` C06.17.

**Baseline arms.** [PROPOSED]

1. **B0 — flat transcript:** the same conversation history is supplied as a flat context dump, subject to the same post-compaction token budget.
2. **B1 — checkpoint-only:** the existing C24 structured checkpoint without the capsule manifest, protected payload, parent chain, or per-atom evidence proof.

**Paired workload.** [PROPOSED] A fixed corpus of at least 30 seeded scenarios: 10 research-synthesis tasks, 10 multi-step code-change tasks, and 10 incident-response tasks. Every scenario must contain a canonical objective, at least three invariants, acceptance criteria, commitments, pending actions, one next action, decoy context, and at least one tool receipt. The corpus, seeds, task order, and expected values are frozen in the A38 run manifest before execution.

**Experimental sequence.** [PROPOSED] Each scenario runs all three arms with the same model, decoding parameters, tool policy, and initial context. The treatment performs forced compaction after the first phase, a process restart before the second compaction, and a second forced compaction. The final query asks for the objective, invariants, acceptance criteria, commitments, pending actions, and next action.

**Primary judges.** [PROPOSED]

| ID  | Metric                                      | Frozen pass threshold                                                                                                                 | Deterministic check                                                                                     |
| --- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| M1  | Critical-state recall after two compactions | 100% of scenario × field checks                                                                                                       | Strict-JSON schema parse plus canonical field equality; LLM scoring is not used for the primary verdict |
| M2  | Capsule manifest and parent-chain integrity | 100% valid hashes and parent links                                                                                                    | Recompute SHA-256 and walk the parent chain                                                             |
| M3  | Supporting-atom evidence proof              | 100% of recalled supporting atoms have a verifiable receipt path; `NOT_APPLICABLE` only in the explicitly Atlas-disabled fallback arm | Recompute the receipt path against the cited ledger root                                                |
| M4  | Prompt-token reduction versus B0            | Median reduction ≥ 50%                                                                                                                | Same tokenizer; paired per-scenario values                                                              |
| M5  | Invalid-tail recovery                       | 100% recovery to the latest valid parent capsule; 0 silent corruptions                                                                | Inject truncated/torn capsule and WAL-tail fixtures                                                     |
| M6  | Fallback parity                             | 100% critical-state recall through the standard-transformer fallback                                                                  | Replay the same capsule view through the fallback path                                                  |
| M7  | Privacy non-interference                    | 0 raw objective, invariant, commitment, pending-action, or next-action values in Atlas telemetry or metrics                           | Schema inspection plus seeded canary scans                                                              |

**Secondary judges.** [PROPOSED] p50/p95/p99 capsule creation latency, p50/p95/p99 recovery latency, token reduction versus B1, Atlas supporting-recall precision, and number of raw payload fields loaded per recovery.

**Decision rule.** [PROPOSED]

1. A38 **FAILS** if any of M1, M2, M3, M5, M6, or M7 misses its threshold. A failed treatment cannot become required and must be recorded in `Failures.md`.
2. A38 **PASSES CORE** if all correctness, recovery, fallback, and privacy judges pass. A38 **FAILS EFFICIENCY** if M4 misses its threshold; the treatment may remain optional but no boundedness claim may ship.
3. Before C06.17 becomes required, the result must replicate on a second distinct machine and a necessity ablation must show that removing the capsule regresses M1 or M3 (C06.13).
4. A failed arm remains part of the record. A retry requires a new A38 version and a newly filed hash; prior results are never overwritten.

**Dependencies.** [INFERRED from the cited category contracts] C17 persistence, C24 compaction, C25 verified execution, C29 tool receipts, C31 telemetry privacy, C35/A33 retrieval when the optional Atlas adapter is enabled, and A10 injection hygiene. The exactness of the critical-state plane must not depend on ACFT, hyperbolic geometry, or self-tuned curvature.

**Not measured here.** [VERIFIED by workspace inspection] No implementation, benchmark, hardware measurement, hash-filed run, replication result, or necessity result exists for A38 in this workspace at registration time.
