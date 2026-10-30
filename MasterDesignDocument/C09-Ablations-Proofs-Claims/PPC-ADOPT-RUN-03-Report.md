# PPC-ADOPT-RUN-03 Report (genuine-win run with in-run repeats: tie at rate level)

**Date.** 2026-10-30. **Manifest.** `PPC-ADOPT-RUN-03-Manifest.md` (frozen; SHA-256 filed below before execution per C09.1/C11.3).
**Model.** `Qwen3.6-MoE-Combined` via `:8000`. **Binary.** `hardware_ppc_adopt` (PPC_REPEATS + PPC_MANIFEST support; `hardware_ppc.rs` untouched).
**Commit.** Filed at commit time.

## Manifest hash

filed 2026-10-30 BEFORE execution: `sha256:fb0c1b2eb95958946a103e2a87ca53d69e20ed112e5310989d31f44ec11079c0` (recompute: `sha256sum PPC-ADOPT-RUN-03-Manifest.md`).

## Results (240 probes: 12 tasks × 5 repeats × 4 bundles; two consecutive sessions agree 100%)

| Candidate | Rate | Misses | Adopt | Reasons |
|---|---|---|---|---|
| BASE | 55/60 (0.9167) | d-02 ×5 | kept | — |
| STABLE (recompute) | 55/60 (0.9167) | d-02 ×5 | **false** | noise band (gain 0.0000) |
| LOSE (weakened FINAL) | 50/60 (0.8333) | h-04 ×5, d-02 ×5 | **false** | leakage flagged; noise; pareto dominated |
| JUNK (control) | 50/60 (0.8333) | h-06 ×5, d-02 ×5 | **false** | noise; pareto dominated |

Adopted: **BASE** (`kept_base=true`, `quality_gain=0.0000`). Held-out on base: **4/4**. Full receipt: `research-grade-harness/HARDWARE-PPC-ADOPT-RUN-03-RECEIPT.txt`.

## Verdict vs decision mapping

- **ABSTAIN-CORRECT (variance-controlled tie).** STABLE rate minus BASE rate = 0.0000 at 60 probes/arm × 2 agreeing sessions. The recompute discipline has no measurable effect at this rigor. ADOPT-DEMO INCONCLUSIVE (4th consecutive non-adoption: abstain/abstain/reject+abstain/tie-abstain).
- **REJECT-DEMO: PASS (strengthened).** LOSE regresses a saturated guard (h-04 5/5 misses — the weakened FINAL breaks a task the strict template holds 5/5) AND trips the leakage flag. Two independent rejection grounds, one mechanistic (format contract), one regulator. JUNK also regresses h-06 5/5 in-run (junk suffix breaks the discriminator while base passes it — session luck cuts both ways, exactly as pressure predicted).
- **d-02 boundary holds:** 0/20 across all arms both sessions (5-conversion chain defeats recompute — computation failure, not instruction failure).
- **Attempt 1 VOIDED (instrument error, not a run):** receipt manifest pointer said RUN-02 (binary derived it from battery). Binary fixed (PPC_MANIFEST env), re-ran clean as attempt 2 (filed). Attempt-1 numbers (identical verdict, 55/55/50/50) retained in `/tmp` only, not filed — superseded before filing, same doctrine as C-RUN v1→v4.

## Variance picture (complete: pressure 6+ sessions + RUN-02 + RUN-03 ×2 sessions)

- **In-run** (60 probes, one session): deterministic per arm (both RUN-03 sessions agree 100%, identical miss tables).
- **Cross-session**: h-06 flips for EVERY arm (each of BASE/STABLE/JUNK/LOSE has passed AND failed it 5/5 in some session). The discriminator measures the session, not the discipline.
- **Cross-session stable cells:** d-02 (always fail all arms), saturated guards (always pass all arms), h-06... unstable.
- **Prescription (filed, not in this run):** adoption claims on this model need multi-SESSION rate deltas (not multi-repeat single-session), or tasks whose base-failure replicates across sessions (none found in 10 candidates). The PPC prompt channel finding stands: format compliance YES (FINAL-line holds everywhere), arithmetic improvement NO (no wording beats base at rate level).

## What this run licenses (and does not)

- Licenses: 4th honest non-adoption with the strongest rigor yet (rate tie at 60/arm × 2 sessions); LOSE's guard-regression as a mechanistic rejection case; the variance prescription.
- Does not license: any adoption-direction claim; any claim that recompute (or any tested wording) improves arithmetic; cross-hardware claims.
