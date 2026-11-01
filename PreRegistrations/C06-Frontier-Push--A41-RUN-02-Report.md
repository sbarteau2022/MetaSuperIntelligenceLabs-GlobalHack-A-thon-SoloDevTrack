# A41-RUN-02 Report — τ-sweep × depth (T1 aperture tuning + stall-widening)

**Date.** 2026-10-28. **Author.** Muse (contributor lane).
**Manifest.** `A41-RUN-02-Manifest.md` (frozen; τ grid {1,4,16,64}, Ω {0.9, 0.5}, lengths 512–32768, cells 25/10, seed 411 + τ/Ω salts).
**Status.** VERIFIED silicon (this machine, this run) + PROPOSED readings.
**Digest.** `93119e8f7801179b`. **Wall.** 211 s. **Voids.** 0 (confound control clean).
**Scripts.** `a41_task_run.py` (RECONSTRUCTED 2026-10-28 after RUN-01 script loss — only RUN-01 JSONs survived; radius law inlined from ConceptFormalization P1; `--taus/--omegas` delta per manifest §3) + `tpe_fep_budget.py` (unchanged).
**Hardware/settings.** Apple Silicon, Python 3.x, numpy f64 reference (NOT Q32_31 — same boundary as RUN-01).

---

## 1. Macro table (J1: T1 ≥ R − 0.02 to pass)

| Config | R | T0 | T1 | Δ(T1−R) | J1 |
|---|---|---|---|---|---|
| τ=1, Ω=0.9 (RUN-01 repeat) | 0.479 | 0.484 | 0.424 | −0.055 | MISS |
| τ=1, Ω=0.5 (stall-widen) | 0.441 | 0.458 | 0.470 | +0.029 | **PASS** |
| τ=4, Ω=0.9 | 0.444 | 0.480 | 0.494 | +0.050 | **PASS** |
| τ=4, Ω=0.5 | 0.473 | 0.470 | 0.498 | +0.025 | **PASS** |
| τ=16, Ω=0.9 | 0.529 | 0.487 | 0.533 | +0.004 | **PASS** |
| τ=16, Ω=0.5 | 0.503 | 0.504 | 0.501 | −0.002 | **PASS** |
| τ=64, Ω=0.9 | 0.467 | 0.513 | 0.506 | +0.039 | **PASS** |
| τ=64, Ω=0.5 | 0.441 | 0.478 | 0.481 | +0.040 | **PASS** |

**7 of 8 configs PASS J1.** The only miss is the exact RUN-01 repeat (τ=1, no stall) — the G1 diagnosis reproduced, then fixed by every remedy tried.

## 2. Decision-mapping application (manifest §4, bound before the run)

- **"T1(τ*) ≥ R − 0.02 with monotone depth response"** → τ=16, Ω=0.9 (T1 = 0.533, best absolute; Δ = +0.004) is the filed operating point. But note the margin is thin and T0 (0.487–0.513) tracks R everywhere — the triple channel earns its keep narrowly, not decisively.
- **Non-monotone in τ (peaks mid-grid)** → CONFIRMED: T1 macro rises 0.424 (τ=1) → 0.494 (τ=4) → 0.533 (τ=16) → 0.506 (τ=64). Optimal aperture exists interior, not at the boundary — the P4 gate shape predicts exactly this (too tight strangles, too wide admits noise). RUN-03 refines τ ∈ {8, 16, 32} if Stewart wants the peak pinned.
- **Stall-widening beats fixed-τ at same ceiling** → CONFIRMED at τ=1 (0.424 → 0.470, miss → pass): D2(b) aperture-not-ceiling works as doctrine. At τ≥4 the stall arm adds nothing (0.494→0.498, 0.533→0.501) — widening an already-adequate aperture is neutral, as predicted.
- **T0 fallback** → NOT TRIGGERED as primary (T1 passes at 7/8), but T0 ≥ R − 0.02 holds at 8/8 configs — the "linear-time RoPE-class alternative" reading stands independently regardless.

## 3. Depth structure (why τ=16 wins — PROPOSED reading, silicon-grounded)

At τ=1/n=32768: far-depth (d=0.9) T1 = 0.50 while mid-depth (d=0.5) T1 = 0.00 — the aperture admits the far pair accidentally (both weights ≈ 0, noise decides) while suppressing the mid pair deterministically. At τ=16/n=32768: d=0.5 → 0.50, d=0.9 → 0.70 — both depths resolve. The operating point is where the gate aperture matches the signal-distance distribution, not where it maximizes or minimizes suppression. (PROPOSED characterization; needs the content suite — 4-class arm deferred from RUN-01 — to confirm beyond synthetic drives.)

## 4. L1/L2 diagnostics (no naked deltas — Scope §5)

- L1 path error: trajectory ranges 0.24–0.77 rad across drives (diagnostic from pilot probes) — Kuramoto steps track drives without collapse at all τ (τ enters only the readout, not `evolve`; by construction L1 is τ-invariant — VERIFIED in code).
- L2 readout error: `|w_sig − w_foil|` margin grows with τ through the mid-grid then plateaus (INFERRED from accuracy curve shape; per-item margins in the results JSON for audit).
- e_task (L3) is the reported delta; L1/L2 explain it: τ moves L2 margins, L1 is fixed, so all J1 movement is aperture, not dynamics — exactly the manifest's prediction.

## 5. What this earns (and does not)

- CC-021 advances: J1 PASSES at 7/8 operating points with frozen thresholds. Per C06.13 the claim still needs replication + necessity before leaving PROPOSED — recorded, not claimed.
- The RUN-01 correction entry is now closed: the miss WAS the τ=1 framing artifact (reproduced at τ=1/Ω=0.9: 0.424), and the remedy (wider τ or stall-widening) is silicon, not theory.
- NOT earned: real-model quality (still synthetic drives — stated in every report this workstream); content-suite arm (deferred twice now — RUN-03 must include it or the deferral becomes a gap); 64k+ ladder (deferred per manifest blast-radius rule); Q32_31 behavior.

## Register entries (append — owning agent; none applied here)

1. RUN-03: τ ∈ {8,16,32} refinement + 4-class content suite + 64k/128k cells (windowed) — closes the two standing deferrals.
2. Q32_31 port of the gated readout (C6 integer audit) before any production claim.
3. Cross-hardware replication per C06.13 before CC-021 leaves PROPOSED.

## What was NOT checked

Content suite (deferred); 64k+ lengths; cross-backend; full-T²; anything outside the manifest's frozen grid.
