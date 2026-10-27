# A39 Run Certification — 2026-10-27 All-Fail Run (Artifacts Verified, Claim Denied)

**Date.** 2026-10-27. **Auditor.** Muse. **Scope.** `A39_CORPUS.json`, `A39_CONTROL.jsonl`, `A39_TREATMENT.jsonl`, `A39_RunReport_2026-10-27.md` (found in working tree 01:38, authored by another agent).
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no file modified).
**Verdict. RUN OCCURRED — VERIFIED (80/80 rows present, paired, timestamped). EFFICACY CLAIM DENIED (0/80 pass; degenerate comparison).** The report's own interpretation (§3–§5) is honest and CERTIFIED accurate. This is a failed run, correctly reported — worth more than a success asserted.

---

## 1. Artifact verification (measured this pass)

| Check | Result | Label |
|---|---|---|
| 40 control + 40 treatment rows, IDs A39-001–A39-040 both arms | Present, paired | VERIFIED (parsed both files) |
| All rows `success: false`, identical error `superposition fail-closed at mlir_op#26 after 3 iterations` | Uniform | VERIFIED |
| Control window 06:38:10–06:38:13 (~3s); treatment 06:38:19–06:38:34 (~15s) | Consistent with report's latency table (59.5 vs 370 ms medians) | VERIFIED |
| Corpus: dict with `schema/description/tasks` | Present | VERIFIED (structure; task count = 40 per report) |
| Report §2 table vs JSONL | Medians/means consistent with row data | VERIFIED (spot-checked) |

## 2. What the run proves and does not prove

**Proves (VERIFIED):** the A39 harness executes end-to-end (corpus → both arms → paired JSONL → report) on laptop hardware in seconds; the subagent path fail-closes deterministically at mlir_op#26 (same op, same 3 iterations, 80/80 — a systematic gate, not flakiness); treatment overhead (~6× latency) is carve/probe/allocation cost on a failing path.
**Does NOT prove:** J1–J6 (all degenerate at 0/0), necessity precision (no successes to attribute), ΔCap, profitability, or any CC-013 movement. The run is a harness shakedown + a null result, correctly labeled as such by its author.

## 3. Required follow-up (unchanged from report §4, plus two additions)

Report items 1–5 stand (fix op#26 path or known-good suite; canonical judge; J2/J3/J5 instrumentation; re-run paired; file frozen prereg). Additions:
1. **The op#26 fail-closed is itself a finding:** `NoDirectGeodesic`-class stop at the same op across all 40 tasks × both arms suggests the corpus tasks genuinely lack map coverage (novel-trajectory path) OR the ingress transducer cannot reach op 26's neighborhood. Diagnose which before choosing "fix the path" vs "choose easier tasks" — the wrong choice manufactures passes.
2. **Treatment latency (+307.5 ms median) must be baselined separately** once runs succeed: carve overhead is currently measured only on failure paths. A successful-path overhead number is a distinct M-CAP-class measurement.

## 4. Status consequence

A39 remains UNRUN for efficacy purposes (null run ≠ evidence run). v1.1 (my prereg) is now the canonical registration; this run executed under NEITHER v1 nor v1.1 (40 tasks not 48; proxy judge not canonical rubric; no v-hash filed). Correct disposition: record as `A39-pilot-00` (harness validation), not as an A39 attempt. No Failures.md entry required beyond this certification (the report already self-records); CC-013 untouched.
