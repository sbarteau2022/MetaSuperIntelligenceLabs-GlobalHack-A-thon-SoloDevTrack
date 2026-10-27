# C30.9 — Failure as Data: Forward Dynamic Iteration After Failure

**Status.** PROPOSED contract. **Date:** 2026-10-27. **Parent:** C25 Hole Substrate, C27 Workflows, C30.8 Soft Budget, C06.17 Continuity Capsule.

---

## 1. Rule

A failed agentic attempt is **not** an error to be papered over; it is a first-class data point. No second attempt is ungated, and no Meta reallocation occurs, until the failed agent has submitted a **Failure Summary** plus a **`reiterative planning.md`** that reasons about the failure. Forward iteration is the only permitted motion; silent retry, backtracking, or overwrite of the prior attempt is forbidden.

## 2. Required artifacts (on any failed attempt)

Before the Meta considers re-allocation or a retry, the failing agent must produce:

1. **`FailureSummary.md`** (top-level of the worktree, above `Agents.md`):
   - attempt id + agent id + run id
   - failing judge / error signature (receipted)
   - root-cause hypothesis (data, not assertion)
   - what was tried and why it failed
   - least-privilege proposed delta (scope, tools, map, or approach)
   - fingerprint/hash of the failed attempt's state for audit

2. **`reiterative planning.md`** (pinned alongside `Agents.md`, immutable mid-run):
   - revised objective/invariants if changed (or explicit "unchanged")
   - revised approach derived from the failure
   - revised silo/basin scope and `λ_map` request (advisory only; Meta decides)
   - stop conditions and a new acceptance gate
   - explicit "no backtracking" clause: prior artifacts are preserved, not overwritten

## 3. Replanning execution (ungating condition)

The second approach is ungated **only** when all of the following hold:

- `FailureSummary.md` and `reiterative planning.md` are present and hashed (ADR-19/ADR-22).
- The failure is logged as a training entry (§4) with a ledger receipt terminating in the root.
- The Meta validates that the replan does not violate I-SB-1 (no budget leakage) or hard budget (C21).
- The Meta reallocates the soft budget (`B_a^soft`) according to the replan's advisory, bounded by hard limits and profitability trend.

Until then: retry is **blocked**, not merely discouraged. The breaker remains open.

## 4. Failure as training data

Every failure is appended to a training corpus (never by mutating prior entries) with:

- agent id, task class, scope, carved-map size, `λ_map`, tool sequence, retry count, error signature
- failure hypothesis and replan delta
- outcome of the subsequent attempt (success/fail) once known

This supports the A41 profitability loop and future task priors. Privacy: counts/ids/ratios and hashes only—no raw objective/invariant/commitment text (C06.17 M7, C24.8).

## 5. Forward dynamic iteration

Permitted motion is strictly forward:

- A failed attempt's artifacts and receipts are **preserved** (append-only; corrections are entries, not edits).
- The replan may narrow or restructure scope; it may not silently reuse a stale plan.
- "Backtracking" (re-running an identical approach expecting a different outcome) is treated as a null attempt and voided by the judge unless the replan differs materially.
- Each iteration must advance at least one of: scope precision, map precision, approach, or acceptance clarity. Stagnant loops trip the breaker.

## 6. Meta reallocation rule

On a replan-gated retry:

- Soft budget may be **tightened** (less spend) if the failure suggests over-scope, or **held/expanded** if it suggests under-resourcing—but the change must be justified by the failure data and recorded.
- Hard budget never expands mid-run (C21).
- Profitability estimate `Π` is updated with the failed attempt's cost (so failures are visible to the Meta economy, not free).

## 7. Invariants

- **I-FWD-1:** No retry without `FailureSummary.md` + `reiterative planning.md` + failure-training entry + Meta validation.
- **I-FWD-2:** Prior attempt artifacts are append-only; replan does not overwrite them.
- **I-FWD-3:** Each iteration materially advances scope/approach/acceptance; stagnant loops open the breaker.
- **I-FWD-4:** Meta reallocation occurs only through the soft-budget control plane, respecting I-SB-1 (no budget leakage to the agent).

## 8. Ablation ownership (proposed)

**A42 — Failure-driven replanning A/B**:

- Arm A: on failure, immediate retry with no summary/replan (current ad-hoc behavior).
- Arm B: on failure, require `FailureSummary.md` + `reiterative planning.md` + Meta reallocation before retry.

Judges: retry success rate, total tool calls to success, total tokens, time-to-success, quality of final solution, and forward-iteration count.

**Status:** proposed; not filed. One ablation per claim (CORR-C06-03).

---

_Last updated: 2026-10-27 (failure/replanning pass)._
EOF
