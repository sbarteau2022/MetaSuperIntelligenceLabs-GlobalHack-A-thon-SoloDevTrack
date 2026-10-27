# C06 — Errors & Improvements (Adversarial Audit Follow-up)

**Status.** PROPOSED follow-up backlog. **Date:** 2026-10-27. Complements `AdvAudit.md` and `MathProofs-Dyptech.md`.

## 1. Errors discovered (concise)

- **E1 Empty-silo hallucination risk** (AA-01/AA-07): `carve_task_view` could return an empty view with no signal.  
  **Fix (shipped partial):** empty `allowed_silos` now falls back to `RoleId::CARVED_12` (coordinator/global view) instead of a single default role.
- **E2 Cross-silo leak via 1-cell closure** (AA-02): forward closure inserts any reachable `GlobalAtom` even if its home silo was not allowed.  
  **Status:** **shipped** — forward closure now requires the candidate atom to lie within cosh distance `≤ 10.0` of at least one _allowed_ silo well before insertion.
- **E3 Unbounded carve** (AA-03/AA-12): no `max_atoms` enforcement in `carve_task_view`.  
  **Fix (shipped partial):** `max_atoms: Option<usize>` parameter added (default 4096), enforced in seed loop and 64-hop closure.
- **E4 Nondeterministic duplicate tie-break** (AA-04): first silo won for duplicate `AtomId`.  
  **Fix (shipped):** deterministic higher-`salience_count` wins (then existing key order).
- **E5 Default role smell** (minor): fallback used `SensoryPerception`; changed to `Coordinator` to avoid silent sensory-only bias.
- **E6 Implicit horizon identity** (AA-05): code assumes `⟨x,y⟩ = cosh d` without an assertion.  
  **Fix (open):** add debug assertion / regression test pinning the identity.
- **E7 Non-atomic dissolution** (AA-06): `compile_deposit` commits without a fingerprint transaction.  
  **Fix (open):** wrap in atomic helper + quarantine.
- **E8 λ starvation silent** (AA-09): `λ_map = 0` gives no reason.  
  **Fix (open):** telemetry field + capsule-only receipt flag.
- **E9 A39 confound not enforced in code** (AA-10): harness must void unfair runs.  
  **Fix (open):** harness-level guard.

## 2. Improvements shipped this pass

1. `carve_task_view` signature: added `max_atoms: Option<usize>` (backward compatible at call sites updated).
2. Empty allowed silo set now falls back to coordinator/global view.
3. Seeding loop now enforces atom cap and deterministic salience tie-break.
4. Forward-closure loop now bounded by `atoms.len() < atom_cap`.
5. Primary role fallback changed from `SensoryPerception` to `Coordinator`.
6. `effective_silos` now drives `allowed_silos` field (reflects fallback).
7. Build + 59 immune + 114 orchestrator tests still green.

## 3. Still required before A39 trust (priority)

| Priority | Item                                           | Owner         | Gate                             |
| -------- | ---------------------------------------------- | ------------- | -------------------------------- |
| P0       | Geometric cross-silo containment guard (AA-02) | atoma_immune  | **shipped; needs property test** |
| P0       | Atomic dissolution transaction (AA-06)         | atoma_immune  | crash injection test             |
| P1       | Horizon identity debug assert (AA-05)          | atoma_immune  | property test                    |
| P1       | `λ_map` + `D_T` fit implementation (§5)        | atoma_immune  | 1k random specs                  |
| P1       | Necessity formula implementation (§4.2)        | atoma_immune  | unit tests                       |
| P2       | A39 harness with confound voiding (AA-10)      | evaluation    | A39 run                          |
| P2       | Overlap policy audit (AA-11)                   | immune_system | policy review                    |

## 4. What was NOT done (updated 2026-10-27, hardening pass B)

- No A39/A40 efficacy execution on retrained weights; no C06 claims flipped to VERIFIED (A40-RUN-01 Stage 1 is a recorded NULL on the native-map geometry leg — `A40-RunReport-2026-10-27.md`).
- E6 (horizon-identity debug assert), E8 (λ-starvation telemetry), E9 (A39 confound guard in code) remain open — NOT done this pass.
- Cross-silo leak guard and atomic dissolution: NOW CODED this pass (see §6); crash-injection test for the transaction path still open.
- No external literature search; novelty claims remain internal only.

## 5. Verification (this pass + pass B)

- `cargo test --manifest-path .../immune_system/implementation/Cargo.toml --lib` → **59 passed, 0 failed** (after edits).
- Orchestrator tests run after the signature change: **114 passed, 0 failed** (compile risk cleared).

Recommend running orchestrator tests next before declaring hardening complete.

## 6. Hardening pass B — AA-02 calibrated + B-5 transaction (2026-10-27, VERIFIED by test output)

- **Finding B-1 (from pressure test).** The flat `ip <= 10.0` forward-closure guard admitted nearly everything (cosh 10.0 ≈ d ≤ 3.0 — ~3 nats beyond small basins) and ignored `OverlapPolicyMatrix` entirely. Direction right, calibration unprincipled.
- **Fix B-1 (APPLIED, in monster-hybrid-server tree, NOT in this repo).** `silo.rs::carve_task_view_with_bands` + `specialist_carving.rs::Allocate` forward closures now admit iff `ip(ga, well(role)) <= basin(role) + overlap(primary_role, role)`, where `basin` is the role's `SiloView::base_basin_cosh_radius` (fallback `default_basin_cosh` = 1.22) and overlap comes from policy (minimal-icosahedral 0.08). `policy` threaded through `carve_task_view_with_bands`; legacy `carve_task_view` builds a zero-overlap policy. Flat thresholds forbidden by code comment.
- **Evidence B-1 (VERIFIED).** New property test `calibrated_guard_admits_in_basin_excludes_antipode`: in-basin atom passes, far-field hyperboloid point (8x spatial scale, renormalized x0) excluded. Note: naive spatial negation is NOT far-field (wells sit near origin, |s|≈0.4 — negated point lands in opposite basin); test uses true far-field. `atoma_immune --lib`: **66 passed, 0 failed**. Orchestrator `--lib`: **114 passed, 0 failed**.
- **Correction entry (append-only, not a rewrite).** Pre-existing test `subagent_task_carved_view_isolates_silos_and_executes_in_motion` asserted `carved_arc_count >= 2` under the 10.0 regime; under the calibrated guard the recorded 2-arc chain spans a cross-basin gap wider than ~1.30 cosh, so closure correctly retains 1 arc and excludes 1. Expectation updated to `>= 1` with `CORR-AA02-2026-10-27` comment in code. The test still asserts antipode exclusion + 6 silos + nonzero atoms.
- **Finding B-2 (from C25 doc audit).** The `expected_post = pre_fp XOR hash(ws)` accumulator existed ONLY in `DissolutionTransactionAndTelemetry.md` — the code had NO fingerprint transaction at all (`compile_deposit` returned counts, no pre/post binding). Doc specified a malleable construction; code had nothing.
- **Fix B-2 (APPLIED).** `DepositCompilationReceipt` gains `pre_fp` + `expected_post = SHA256(pre_fp || canonical_write_set)` (pure-Rust in-tree SHA-256, zero new deps). Canonical encoding: sorted-by-global-id `(id u64 LE || clock u64 LE || lorentz 4x i64 LE || len u64 LE || payload)`. `canonical_vault_image_for_receipt()` (epoch || next_id || clock || per-atom id/clock/coords/payload) binds the pre-image to live vault bytes.
- **Evidence B-2 (VERIFIED).** New test `sha256_accumulator_resists_xor_aliasing_and_ordering`: order-invariance holds, one-byte change avalanches, duplicate entries do NOT cancel (XOR would give X^X=0). Included in the 66-green run above.
- **Still open after pass B.** E6, E8, E9 (unchanged); crash-injection test for the transaction path; `Allocate`'s `volumetric_cone_ball_coord` import now unused (warning only, cleanup deferred); second-machine replication (C-A40-08).

---

_Last updated: 2026-10-27 (hardening pass B: AA-02 calibrated, B-5 transaction coded, 66+114 green)._
