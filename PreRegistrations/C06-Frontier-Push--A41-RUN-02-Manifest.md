# A41-RUN-02 Manifest (FROZEN — τ-sweep × depth + budget ledgers)

**Date.** 2026-10-28. **Status.** [PROPOSED] Manifest frozen; no RUN-02 execution has occurred.
**Rule.** [PROPOSED] SHA-256 of THIS file + the two scripts recorded below in the run report before execution. Any edit → RUN-03 + new hashes. Prereg A41 v1.1 (`sha256:246cabae…f02531ae14`) governs; this manifest instantiates it — frozen J1, no threshold changes, so no reprereg required.
**Parents.** `Pre-Registration-A41-v1.1.md` (J1 + confound control); `PredictionErrorMinimization-Scope.md` (§5 acceptance); `A41-RunReport-v1.1-2026-10-28.md` (G1–G4 motivate the sweep).

---

## 1. Frozen parameters

| Parameter | Value | Provenance |
|---|---|---|
| `τ` grid | {1, 4, 16, 64} × per-arm (R arm: n/a — no gates) | Amendment-rule grid (G1: τ=1.0 strangles far recall) |
| Lengths | 512, 1024, 2048, 4096, 16384, 32768 (short ladder; 64k+ deferred to RUN-03 — blast-radius control) | v1.1 ladder subset |
| Depths | 0.1, 0.5, 0.9 signal depth | v1.1 workload |
| Cells | 25 per (length × depth × τ) at ≤4096; 10 at 16k/32k | v1.1 (short/long split) |
| Seed | 411 (same seed family as RUN-01: seed = 411·1_000_003 + n·131 + depth·17 + cell + τ_idx·7919) | Reproducibility; τ_idx added to the RUN-01 salt — disjoint streams per τ |
| Budget | Controller-internal (FEP law `B(d̂,Ω)`); per-item ledger records `B` per arm; **void rule: any item with >10% cross-arm `B` asymmetry is void for J1** (kept for diagnostics with caveat) | v1.1 confound control |
| Window rule | Full matrices <16k; readout-windowed ≥16k (positions = signal/foil/query only) | v1.1 workload (RUN-01 implementation) |
| J1 | UNCHANGED: T1 ≥ R − 0.02 macro-averaged over (length × depth) cells at fixed budget | v1.1 frozen |
| J2–J8 | Rerun as confirmation (mechanism gates; all PASS in prior silicon) | v1.1 carryover |

## 2. Arms

- **R** (RoPE reference, no gates — τ n/a). **T0** (angle-only, τ → ∞ limit = P2). **T1(τ)** (triple-channel at each grid τ). T1-SWAP control at τ=16 only (exchange symmetry spot-check, not full grid — cost control, stated).
- Stall-widening arm (D2b): T1 with `τ` scaled by controller widen factor `1 + 2·(0.82 − Ω)` at Ω ∈ {0.5, 0.7} on depth 0.5/0.9 cells only (aperture-not-ceiling path motivated by G1). Budget ceiling enforced; widening logged per item.

## 3. Scripts (frozen for this run)

- `a41_task_run.py` — instrument (2AFC, content-coupled drives, windowed readout). **RUN-02 delta (preregistered here, not a silent change):** `--taus` grid flag + `--omega` stall arm + per-item `B` ledger already present; no J1/threshold edits.
- `tppe_rope_sidebyside.py` — analytic confirmation (unchanged from RUN-01).
- `tppe_fep_budget.py` — budget law (unchanged; P6 5/5).

## 4. Decision mapping (binds the report before it exists)

| Outcome | Reading |
|---|---|
| T1(τ*) ≥ R − 0.02 at some τ* with monotone depth response | τ* filed as operating point; CC-021 advances toward VERIFIED (pending replication) |
| T1 flat ≈ 0 at all τ ≤ 64 | Gate aperture insufficient even at wide τ — mechanism limitation recorded, triple channel's scope restricted to local precision (G4), no further τ run without redesign |
| T1(τ) non-monotone in τ (peaks mid-grid) | Optimal-aperture finding; follow-up grid refines around peak (RUN-03) |
| Stall-widening beats fixed-τ at same ceiling | D2(b) confirmed as the budget-use doctrine; controller adopts widen-by-default |
| T0 ≥ R − 0.02 while T1 misses everywhere | H0 fallback ships: T0 as linear-time RoPE-class alternative; triple channel retired to local-precision role (NOT ornament — G4 near-field wins stand) |

## 5. Run report requirements (acceptance)

Per `PredictionErrorMinimization-Scope.md` §5: ledger receipt per item, `e_task` delta at fixed `B`, L1/L2 diagnostics explaining each J1-relevant delta, void count, wall time, hardware pins, script hashes. No naked deltas.
