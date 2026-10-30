# PPC-ADOPT-RUN-02 Manifest (FROZEN — genuine-win run: recompute discipline)

**Date.** 2026-10-30. **Status.** [PROPOSED] Manifest frozen; no RUN has occurred.
**Rule.** SHA-256 of THIS file recorded in the run report before execution (C09.1/C11.3). Any edit → PPC-ADOPT-RUN-03 + new hash.
**Supersedes.** PPC-ADOPT-RUN-01 (reported, INCONCLUSIVE on adoption) — NOT before-execution: RUN-01 executed and filed. This is the follow-up run the RUN-01 report prescribed ("escalate task difficulty or WIN discipline").
**Parents.** `PPC-ADOPT-RUN-01-Report.md` (pressure findings driving this battery); `HARDWARE-GATE-RUNBOOK.md`; `rsi.rs::regularized_adopt` (rule under demo).
**Model.** `Qwen3.6-MoE-Combined` via `:8000` (Stewart directive). No other model.

---

## 1. Pressure summary (what broke, what fixed — all VERIFIED live pre-manifest)

- Base saturates v2 tasks 8/8 (no headroom). Harder tasks (h-03/h-05) fail under EVERY discipline (computation/token-limit failures, not instruction failures — excluded).
- RUN-01 WIN (show-work) REGRESSED probe-02 (171≠172): verbosity feeds last-number traps.
- RUN-01 LOSE tied 8/8: the strict template FINAL line downstream overrides component looseness; leakage flag was a substring false-positive (filed, not in run).
- **STABLE (recompute discipline): h-06 BASE 0/15 vs STABLE 15/15 across 3 sessions** (45/45 deterministic within block). Siblings h-06b/h-06c fail under both arms (recompute only as good as the model's arithmetic on that recompute — mechanism boundary filed).
- STABLE regresses d-05/d-06 (recompute derives wrong alternatives on already-complete totals). Coverage-check variant (CHECK) fixes nothing. The win is CONDITIONAL on the task needing re-derivation — the battery reflects exactly that.

## 2. Frozen parameters

| Parameter | Value | Provenance |
|---|---|---|
| Discriminator (1 task) | h-06: 3-area carpet, target 1155. BASE 0/15, STABLE 15/15 pre-manifest | §1 pressure |
| Regression guards (10 tasks) | v2 probe set (8: probe-01..08) + probe-02-class saturated tasks h-04, d-07 (both 5/5 all arms pre-manifest). STABLE must not regress any (RUN-01 lesson: verbosity breaks probe-02) | Guards carry the adoption veto: any guard regression → NO ADOPT regardless of h-06 |
| Boundary task (1 task, diagnostic) | d-02 (5-conversion chain): both arms 0/15 pre-manifest. Records where recompute stops working. NOT counted in the win; a STABLE pass here would exceed prediction (filed as surprise) | Mechanism boundary |
| WIN candidate | STABLE discipline: base components + ` Before answering, recompute the total a second time from the original numbers and use the recomputed value.` | The pressure-proven discipline |
| LOSE candidate | RUN-01 LOSE verbatim (weakened FINAL). Predicted: ties (template overrides) + leakage-flagged → rejected | Rejection control |
| JUNK candidate | RUN-01 JUNK verbatim. Predicted: ties → abstain | Saturation control |
| Base | `Bundle::base()` | Same |
| k/rollouts/sessions | k=1, rollouts=4, sessions=3 (same as all PPC runs) | Comparability. NOTE: gate probes each task ONCE per candidate per the probe_fn (rollouts/sessions are gate-internal repeats of the decision, not per-task repeats). The 15-repeat pressure data is the variance evidence; the run itself is single-pass per candidate (same as RUN-01 and the abstention runs — comparability preserved) |
| Held-out | RUN-01's 4 held-out tasks (unchanged) run ONCE on the adopted bundle | Same |
| Judges | `last_number_eq` (float compare — e-03 lesson: "246.40" == "246.4" passes; string-equality probe scripts were wrong, the bin's judge is correct) | Same judge, lesson recorded |
| Timeout | 1200s wall | Same |

## 3. Decision mapping (binds the report before it exists)

| Outcome | Reading |
|---|---|
| STABLE passes h-06 AND all 10 guards, LOSE/JUNK controlled, gain clears 0.02 → ADOPTED | **ADOPT-DEMO PASS**: genuine win, gate fires on it. Held-out confirms. CC draft `C09-PPC-adopt` PROPOSED → run-evidenced (replica per C09.4 for EVIDENCED) |
| STABLE passes h-06 but regresses ≥1 guard → NOT adopted | **GUARD-VETO**: win with unacceptable cost. Correct non-adoption; file which guard + pressure-vs-run delta (if pressure said clean, that's session variance biting — file it) |
| STABLE fails h-06 (pressure said 15/15) | **VARIANCE FINDING**: session variance dominates the discipline (the d-06 lesson: BASE went 0/5→5/5 across sessions). No verdict on the discipline; prescribe multi-session gate repeats before any adoption claim |
| LOSE adopted (any path) | **GATE FAIL**: ship-blocker (same as RUN-01 mapping) |
| d-02 passes under either arm | **SURPRISE** (predicted fail both): file the delta, do not fold into the win (one session is not evidence against 0/15 pressure) |

## 4. Instrument

Same `hardware_ppc_adopt` binary + `ppc_probe` lib (no changes needed — battery differs by TASK SET, and the task set lives in `ppc_probe.rs` shared constants... h-06/d-02 are NOT in PROBE_TASKS). **Deviation filed here (not silent):** the discriminator/guard/boundary tasks are new constants. Options were (a) new binary (duplication) or (b) extend `ppc_probe.rs` with a `PRESSURE_TASKS`/`ADOPT_RUN02_TASKS` const + a `--battery` flag on the binary. (b) chosen: `ppc_probe.rs` gains `ADOPT_RUN02_TASKS: &[PpcTask]` (h-06, d-02, h-04, d-07 + v2 set referenced, not duplicated), binary gains `--battery run01|run02` (default run01 — old behavior byte-identical). `hardware_ppc.rs` untouched. Pressure-probe scripts (socket one-offs) are NOT committed (blast radius — their verdicts live in this manifest §1 + the report).
