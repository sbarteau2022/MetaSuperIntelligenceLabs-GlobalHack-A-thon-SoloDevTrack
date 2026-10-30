# PPC-ADOPT-RUN-02 Report (pressure-win meets session variance: tie)

**Date.** 2026-10-30. **Manifest.** `PPC-ADOPT-RUN-02-Manifest.md` (frozen; SHA-256 filed below before execution per C09.1/C11.3).
**Model.** `Qwen3.6-MoE-Combined` via `:8000`. **Binary.** `hardware_ppc_adopt` (PPC_BATTERY=run02, STABLE candidate).
**Commit.** Filed at commit time (with RUN-03 — see chain note).

## Manifest hash

filed 2026-10-30 BEFORE execution: `sha256:c6f2b31fe3d0c914c47d57c05ac27021cbf1559858b3ef6021afec236584550d` (recompute: `sha256sum PPC-ADOPT-RUN-02-Manifest.md`).

## Results (12 tasks × 1 pass × 4 bundles = 48 probes)

| Candidate | Passes | Misses | Adopt | Reasons |
|---|---|---|---|---|
| BASE | 11/12 | d-02 | kept | — |
| STABLE (recompute) | 11/12 | d-02 | **false** | noise band (gain 0.0000) |
| LOSE (weakened FINAL) | 10/12 | h-06, d-02 | **false** | leakage flagged; noise; pareto |
| JUNK (control) | 10/12 | h-06, d-02 | **false** | noise; pareto |

Adopted: **BASE** (gain 0.0000). Held-out: **4/4**. Receipt: `HARDWARE-PPC-ADOPT-RUN-02-RECEIPT.txt`.

## Verdict vs decision mapping

- **ABSTAIN-CORRECT.** STABLE ties base 11/12 (gain 0.0000). The pressure 15/15-vs-0/15 did NOT replicate single-pass: base passed h-06 in-run. Per mapping row 3 (**VARIANCE FINDING**): the h-06 discriminator measures the session, not the discipline — BASE went 0/15 (pressure) → pass (RUN-02); the d-06 precedent generalizes.
- **REJECT side strengthened:** LOSE fails h-06 5/5... (single-pass: 1 miss) while base passes it — genuine regression on the discriminator, plus the leakage flag. JUNK also misses h-06 (junk suffix breaks what base holds — session luck cuts both ways).
- **d-02 boundary holds:** fails all arms (predicted).
- **No guard regressed** (all 10 guards pass all arms). No GUARD-VETO triggered.
- **Chain note:** this report + receipt filed WITH RUN-03 (one commit) because RUN-02 executed while its report was still being drafted — the manifest hash was filed pre-execution (above), satisfying C09.1/C11.3; the commit groups RUN-02's close-out with RUN-03's variance fix. No measurement edited post-hoc.

## What this run licenses (and does not)

- Licenses: the variance finding (prescribed RUN-03's in-run repeats); LOSE's h-06 regression as a second rejection case.
- Does not license: any adoption-direction claim (tie at single-pass).
