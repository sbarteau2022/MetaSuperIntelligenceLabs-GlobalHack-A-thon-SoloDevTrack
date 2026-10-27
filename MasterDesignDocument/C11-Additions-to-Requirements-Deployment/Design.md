# C11 — Frontier Additions to §§Requirements/Deployment

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Frontier-push additions to requirements and deployment (prompt-injection hygiene, energy SLOs,
ablation integrity, cross-hardware replication, kill switch, staged rollout, reproducibility).

## Subcategories (broad topics)

- C11.1 F11 prompt-injection hygiene requirement
- C11.2 NF9–NF11 energy and SLO non-functionals
- C11.3 Ablation integrity gate (prereg hash before run)
- C11.4 Cross-hardware replication requirement
- C11.5 Kill switch + staged rollout + rollback
- C11.6 Reproducible release artifacts

## Open questions / gaps

- Merge into C08/C10 once wording stabilizes.

## Formal Integration (Merged into C08/C10)

### F11 Prompt-Injection Hygiene → C08.F11 + C16.5 + C07.7

- **Requirement**: All injected memory passes through sanitization pipeline (C07.7)
- **Non-functional**: Red-team ablation A10 with ASR < 1%
- **Verification**: `cargo test --test prompt_injection_redteam`

### NF9-NF11 Energy/SLO → C08.NF9-11 + C21

- **NF9**: `tok_s_p99 >= config.min_tok_s_p99` on target hardware
- **NF10**: `joules_per_token <= config.max_joules_per_token` (measured via `powermetrics`)
- **NF11**: `peak_memory_gb <= config.peak_memory_gb_max` per topology

### Ablation Integrity Gate → C09.1 + CI

- **Precondition**: Pre-registration hash recorded before ablation run
- **Enforcement**: CI gate rejects runs without valid hash
- **Null results**: Recorded as first-class failure entries (C09.7)

### Cross-Hardware Replication → C09.4 + Release

- **Requirement**: Primary (M3 Max) + Replica (M2 Ultra) both pass
- **Tolerance**: Effect size within 20% of primary
- **Documentation**: Replication results in `CLAIMS_REGISTER.md`

### Kill Switch + Staged Rollout + Rollback → C10.3-10.5

- **Kill switch**: Atomic config flag `acft_enabled`, default `false`
- **Staged rollout**: Canary (5%) → Shadow (100% mirror) → Full
- **Rollback**: Deterministic reversion script tested in CI

### Reproducible Release → C10.2 + Build Receipt

- **Toolchain pinning**: `rust-toolchain.toml`, `package-lock.json`
- **Build receipt**: Signed artifact with git commit, lock hashes, measurements
- **Verification**: `./scripts/verify_build.sh` bit-for-bit check

## Merge Status: COMPLETE

All C11 items have been formally integrated into:

- C08 (Framework Requirements) — F11, NF9-11
- C09 (Ablations) — Integrity gate, cross-hardware replication
- C10 (Deployment) — Kill switch, staged rollout, rollback, reproducibility

This category is now a historical record. Future references should point to the integrated locations.

---

## Contributor Deepening — Muse (2026-10-26)

### C11.7 Merge verification (depth: a merge claim is itself a claim)

"Merge Status: COMPLETE" above is PROPOSED until verified [PROPOSED — verification is my proposal; the merge targets are frozen]. Each bullet must resolve to a live section:

| C11 item             | Claimed target   | Verified present?                                                                                                 | Note                                                  |
| -------------------- | ---------------- | ----------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| F11                  | C08.F11          | VERIFIED — F11 invariant + matrix row + checklist row present in C08/Design.md                                    | —                                                     |
| NF9-11               | C08.NF9-11 + C21 | VERIFIED — present in both (read 2026-10-26)                                                                      | —                                                     |
| Integrity gate       | C09.1 + CI       | PARTIAL — C09 methodology + C11.3 text exist; gate-level CI binding unconfirmed (no workflows/ tree in workspace) | Confirm before P1                                     |
| Replication          | C09.4 + Release  | VERIFIED — C09.4 protocol + C01 criterion 4 present                                                               | Release-side record location (CLAIMS_REGISTER.md) TBD |
| Kill switch etc.     | C10.3–10.5       | VERIFIED — invariant + rollout + rollback present in C10/Design.md                                                | Drill schedule added C10.8                            |
| Reproducible release | C10.2 + receipt  | VERIFIED — receipt schema + verify script text present                                                            | Script files unconfirmed (no scripts/ tree)           |

Stale pointer found and recorded (not silently fixed): C11/Formal Integration cites "C07.7" as the sanitization pipeline location, but C07.7 in the current C07/Design.md is my 2026-10-26 capacity-accounting addition — the pipeline lives at C07.5. Correction: the F11 → C07 pointer should read C07.5 (sanitization as information-flow control). Recommend a C36 correction entry; leaving the original text intact per C03.3.

What was NOT checked here: scripts/ and workflows/ trees (absent from workspace listing — confirm whether they live outside this workspace).
