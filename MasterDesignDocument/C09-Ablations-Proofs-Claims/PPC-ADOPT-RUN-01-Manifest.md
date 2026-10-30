# PPC-ADOPT-RUN-01 Manifest (FROZEN — PPC adopt-demo: instructional candidates)

**Date.** 2026-10-30. **Status.** [PROPOSED] Manifest frozen; no RUN has occurred.
**Rule.** SHA-256 of THIS file recorded in the run report before execution (C09.1/C11.3). Any edit → PPC-ADOPT-RUN-02 + new hash.
**Parents.** `HARDWARE-GATE-RUNBOOK.md` (fixed PPC); `HARDWARE-PPC-RECEIPT.txt` v2 (saturation finding motivating this run); `rsi.rs::regularized_adopt` (adoption rule under demo, NOT under test — the rule is tested code, the DEMO proves it fires both ways live).
**Model.** `Qwen3.6-MoE-Combined` via `:8000` (Stewart directive). No other model.

---

## 1. What this run proves

The gate has abstained twice (gain 0.0000, KEEP_BASE) because every candidate tied 8/8 — candidates differed by junk suffixes with no instructional mechanism. This run tests whether the gate FIRES BOTH WAYS with instructional-difference candidates: ADOPT a genuine win, REJECT a genuine regression, ABSTAIN among ties.

## 2. Frozen parameters

| Parameter | Value | Provenance |
|---|---|---|
| Task set | v2 probe (8) + v2 held-out (4), UNCHANGED from `hardware_ppc.rs` (`PROBE_TASKS`/`HELDOUT_TASKS` byte-identical; python3-verified targets). Same judges (`last_number_eq`), same `bundle_system_prompt` channel (components → system prompt, FINAL-line discipline) | Comparability with the two abstention runs; no task-set confound |
| Base | `Bundle::base()` (same 8 components) | Same as abstention runs |
| WIN candidate | Base components + instructional discipline: `Show each calculation step on its own line, then double-check the last step before answering. End every reply with: FINAL <number> and nothing after it.` (appends a show-work-then-verify discipline to the FINAL-line instruction — genuine instructional mechanism targeting trap tasks where one-step-early stops fail) | Predicted: passes ≥ base; adoption iff gain clears the 0.02 noise band |
| LOSE candidate | Base components with the FINAL-line instruction WEAKENED: `End every reply with: FINAL <number> and nothing after it.` → `End your reply with the answer.` (removes the machine-checkable format contract; `last_number_eq` hunts the last number in free text) | Predicted: passes ≤ base, trap tasks break first; gate must NOT adopt (rejection demo). If LOSE ties 8/8, finding = model ignores the FINAL-line channel (PPC prompt channel suspect — escalate, don't re-run) |
| JUNK candidate | Base component 0 + ` + improve invariant` (same as current `propose_inv` — control arm proving the run reproduces the abstention baseline) | Predicted: ties base 8/8 (reproduces saturation finding) |
| k/rollouts/sessions | k=1, rollouts=4, sessions=3 (same as abstention runs) | Comparability |
| Adoption rule | `regularized_adopt` + `leakage_critic` + 0.02 noise band + pareto/security (same code path, candidates evaluated pairwise vs base) | The rule under DEMO (not test) |
| Held-out | Run ONCE on the adopted bundle (or base if kept), same 4 tasks | Same as abstention runs |
| Budget | EnergyBudget 50.0 (same) | Same |
| Timeout | 1200s wall (3 candidates × 8 probes + held-out, live model) | Generous; expected <600s |

## 3. Decision mapping (binds the report before it exists)

| Outcome | Reading |
|---|---|
| WIN adopted (gain > 0.02, regulators pass, held-out confirms) | **ADOPT-DEMO PASS**: gate fires on genuine wins. LOSE must simultaneously be rejected (else the gate is win-biased — file and investigate) |
| WIN ties, LOSE regresses, base kept | **REJECT-DEMO PASS, ADOPT-DEMO INCONCLUSIVE**: gate correctly rejects regression and abstains on ties; adoption direction unproven (no win observed). Escalate: harder tasks or stronger WIN discipline, next run |
| All three tie 8/8 | **ABSTAIN-CORRECT, CHANNEL-SUSPECT**: gate right to abstain; but LOSE tying means the model ignores the FINAL-line instruction channel — the PPC prompt channel itself is suspect. Finding filed; next material change is a different channel (few-shot examples? output-schema constraint?), not more wording |
| LOSE adopted | **GATE FAIL**: adoption rule broken live — ship-blocker, `regularized_adopt` defect filed, no further PPC runs until fix + re-verify on unit tests |
| JUNK adopted over tying WIN | Tie-break defect (should prefer... actually no preference specified — record, don't judge: adoption among exact ties is arbitrary by design; KEEP_BASE default should hold. If JUNK adopted on a tie → abstention-default defect, file) |

## 4. Instrument

New binary `hardware_ppc_adopt.rs` (same crate, same `chat_once`/`last_number_eq`/`bundle_system_prompt` helpers — copied verbatim, not refactored, to avoid touching the filed `hardware_ppc.rs`): builds the 4 bundles (base/WIN/LOSE/JUNK), probes each on the live model, runs `regularized_adopt` pairwise vs base through the same regulator stack, writes `HARDWARE-PPC-ADOPT-RECEIPT.txt` (same receipt shape + candidate instruction text). `rsi.rs` UNTOUCHED (frozen tested code).

## 5. Run report requirements (acceptance)

Per C09.3 + HARDWARE-GATE-RUNBOOK §5 (live status, log, why-receipt): per-candidate passes/tokens/quality, adoption verdict + regulator states, held-out on adopted, energy, model/server pins, manifest SHA-256, commit hash. Receipt file committed; raw model outputs NOT committed (blast radius — ledger carries verdicts).
