# Prediction-Error Minimization — Scope (TPPE + FEP + harness)

**Date.** 2026-10-28. **Author.** Muse (contributor lane). **Status.** PROPOSED scope (no silicon claimed).
**Parents.** `ToroidalPhasePositionalEncoding-ConceptFormalization.md` §§1–2 (D1–D3 budget law); `ToroidalPhasePositionalEncoding-ConceptsToProofs.md` P6 (budget monotonicity + spiral coupling); `tppe_fep_budget.py` (reference controller, P6 5/5 PASS — VERIFIED controller logic, f64 reference, not production); HEAR full text (REPORTED 2026-10-28 read: `d_i = π(h_i, s_j)`, intent≠control, Guard requirements override preferences); JAZ full text (REPORTED: pass-by-reference history, selective read wins on cost); GenMem full text (REPORTED: addressing invariance, evolution without drift).
**Standards.** Documentation-standards (labels); C03.3 (no vault/freeze file modified).

---

## 1. What "prediction error" means here (three levels, not one)

| Level | Symbol | Definition | Where it lives |
|---|---|---|---|
| L1 path error | `e_path` | `\|θ_predicted(t+1) − θ_actual(t+1)\|` per pair-plane under the Kuramoto 2:1 step | `torus.rs::evolve` (VERIFIED read) |
| L2 readout error | `e_read` | `\|affinity_predicted(sig) − affinity_measured(sig)\|` on the 2AFC contrast (signal vs foil weight) | `a41_task_run.py::run_item` (VERIFIED code read, synthetic drives — NOT a model run) |
| L3 task error | `e_task` | `1 − span_recall` / `1 − signal_recall` on the v1.1 workload (planted-signal + content suite) | Prereg A41 v1.1 J1 (PROPOSED, unrun on real content) |

**Scope rule.** [PROPOSED] Minimization targets `e_task` under a fixed FEP budget; `e_path` and `e_read` are diagnostics, never the objective. Optimizing L1/L2 alone is out of scope (goodhart risk: exact phase prediction with zero task gain is a recorded failure, not a win).

## 2. Minimization statement (bounded, not zero-seeking)

**Claim.** [PROPOSED] Controller holds `B(d̂, Ω)` fixed per item (ConceptFormalization D2) and minimizes `E[e_task]` by choosing *aperture* (`τ_r, τ_a` widening) and *admission* (which pairs enter the window), never by exceeding `B_max`. Free-energy surprise `F = |predicted − observed|` on the trajectory readout raises `d̂`, which raises `B` monotonically (P6a) up to the ceiling; stall (`Ω < 0.82`, REPORTED AttnEcon threshold) widens aperture at fixed ceiling (P6b).

**What minimization is NOT.** [PROPOSED] Not driving `F → 0` globally (unbounded budget); not tuning `τ` per item to fit noise (amendment rule C09.9 — `τ` changes require reprereg); not rewriting history to flatter `e_task` (ADR-19: corrections are entries).

## 3. Levers in scope (all real-runnable, none synthetic)

1. **Aperture law** (`τ` sweep × depth, A41-RUN-02 manifest staged). Real control: `causal_scores_windowed` path in `a41_task_run.py` (VERIFIED code) + Q32_31 port (staged, C6 integer audit).
2. **Budget law** (`B(d̂,Ω)`, `B_max` fail-closed). Real control: `tppe_fep_budget.py::FEPBudgetController` (VERIFIED 5/5) + madmind `Budget` command + receipt log (VERIFIED `cargo test` 30/30, 2026-10-28).
3. **Address stability** (GenMem principle, PROPOSED adoption). Real control: fixed SID-style addresses for fibers across revisions; content churn must not reassign addresses (GenMem Table 4 pattern: SID-addressed rises 30.56→36.20 while content-coupled decays — REPORTED).
4. **Engine feedback** (HEAR contract, PROPOSED adoption). Real control: harness→engine intent/requirements + engine→harness residency/queue state; Guard-style requirement overrides preference (HEAR Table 3: 1.61× + TTFT 63.1→4.6s — REPORTED). Your controller currently speaks only to itself; hearing back is the gap.
5. **Selective read** (JAZ pattern, PROPOSED adoption). Real control: meta-agent reads sub-traces selectively instead of full replay (JAZ: ACE reflector 10×/task cost vs batched selective read — REPORTED).

## 4. Out of scope (explicitly not claimed)

- Real-model quality (no model run has occurred in this workstream — VERIFIED by absence; A41 v1/v1.1 silicon is mechanism-vs-mechanism on synthetic drives).
- Q32_31 bit-behavior of the controller (staged port, C6 audit not yet run).
- Cross-hardware replication (C06.13 ladder, not started).
- Full-T² extension, cotangent roadmap (freeze §§9–10, non-normative).

## 5. Acceptance (what "scoped" means for the next run)

- [PROPOSED] RUN-02 manifest pins: `τ` grid, depth cells, frozen J1, per-item budget ledgers, void rule (>10% budget asymmetry voids J1 for that item).
- [PROPOSED] Every minimization claim ships with: ledger receipt, `e_task` delta at fixed `B`, and the L1/L2 diagnostics that explain *why* (no naked deltas).
- [PROPOSED] Any `τ` change beyond the grid = amendment + new hash, never a silent retune.

## Register entries (append — owning agent; none applied here)

1. File A41-RUN-02 manifest (τ-sweep × depth + budget ledgers).
2. Port FEP controller to Q32_31 (C6 integer audit gate).
3. Adopt HEAR force distinctions (intent≠control, preference≠requirement) in the controller's message contract before any engine coupling.
