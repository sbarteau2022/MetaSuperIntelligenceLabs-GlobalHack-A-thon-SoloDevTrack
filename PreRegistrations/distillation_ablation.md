# Pre-Registration: Distillation Ablation (A39-facing)
**Date.** 2026-10-28. **Status.** PRE-REGISTERED (predictions written before any run; no execution beyond the simulated sweep above).

## Judge for success
- Structural: `knowledge_atoms_minted == modes_scanned` AND role-distribution sums (AUDIT §7) — via `DistillationJudge` (research-grade-harness).
- Capability: ΔCap ≥ 0 on held-out tasks, no regression vs frozen baseline (A39).

## Predictions (before run)
1. Greedy top-k retains ≥ 3× the energy of uniform stride at (N=1000,k=64,Zipf-1) [VERIFIED sim: 3.63×].
2. Mid-heavy band allocation (0.20/0.50/0.30) retains ≥ 1.10× uniform sweep for L=52 [INFERRED from T3].
3. Walks that span all three bands receive the +2 salience boost; single-band walks do not.
4. Raising Hmax from 2.75 to 3.5 increases walk count but lowers mean retained energy (tradeoff predicted, not measured).

## What was not pre-registered as measured
- Real teacher-checkpoint spectral skew; engine-level judge gate; twin-fingerprint test.
