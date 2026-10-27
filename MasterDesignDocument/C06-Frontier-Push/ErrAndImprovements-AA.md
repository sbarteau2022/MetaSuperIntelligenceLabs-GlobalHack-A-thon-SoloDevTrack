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

## 4. What was NOT done

- No A39 execution, no claims flipped to VERIFIED.
- Cross-silo leak guard and atomic dissolution not yet coded.
- No external literature search; novelty claims remain internal only.

## 5. Verification (this pass)

- `cargo test --manifest-path .../immune_system/implementation/Cargo.toml --lib` → **59 passed, 0 failed** (after edits).
- Orchestrator tests run after the signature change: **114 passed, 0 failed** (compile risk cleared).

Recommend running orchestrator tests next before declaring hardening complete.

---

_Last updated: 2026-10-27 (hardening partial pass)._
EOF
