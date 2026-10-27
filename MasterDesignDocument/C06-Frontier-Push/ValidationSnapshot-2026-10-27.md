# Validation Snapshot — 2026-10-27 (actionable data)

**Status.** Measured by direct execution this pass. **Label:** VERIFIED (command output captured in this session).

## Rust test runs

| Suite | Command | Result |
|---|---|---|
| `atoma_immune` lib | `cargo test --manifest-path .../immune_system/implementation/Cargo.toml --lib` | **59 passed / 0 failed** |
| `atoma_orchestrator` lib | `cargo test --manifest-path .../orchestrator/Cargo.toml --lib` | **114 passed / 0 failed** |
| `full_chain_qualification` | `cargo test --manifest-path .../orchestrator/Cargo.toml --test full_chain_qualification` | **13 passed / 0 failed** |
| `map_toroidal_attention` | `cargo test --manifest-path .../orchestrator/Cargo.toml --test map_toroidal_attention` | **4 passed / 0 failed** |

## Subsystem evidence (VERIFIED this pass)

- Isometry gate: `refraction::tests::ball_and_hyperboloid_distances_agree_to_4_4e_9_across_900_pairs` → PASS (`≤ 4.4×10⁻⁹`).
- Carved view integration: `continuous_active_mind::tests::subagent_task_carved_view_isolates_silos_and_executes_in_motion` → PASS (6 allowed silos, ≥2 arcs, ≥1 atom excluded).
- Dawn of the test evidence includes: 12-around-one, governor recovery <50ms, zero-copy base weights, silo isolation, loss-accounting, non-abelian holonomy, agent dissolution/deposit coalescence, native MLIR SSA, open-weights distillation, map-toroidal attention, vault round trip, cross-epoch adaptation.

## Build status

- `atoma_immune` lib: builds clean (warnings only, 5 auto-fix suggestions from clippy-style lint).
- `atoma_orchestrator` lib + integration tests: builds clean.

## What these data do and do not prove

- They **do** prove the hardened carve/closure path (empty silo fallback, atom cap, salience tie-break, cross-silo well guard) compiles and keeps 59/59 tests green, and that the integration qualification suite is green.
- They **do not** prove A39/A40/A41/A42 efficacy, `λ_map` fitness, or necessity precision; those require the (not-yet-filed) paired ablation harness and real hardware runs. No claim has been flipped to VERIFIED.

---

_Last updated: 2026-10-27 (actionable data snapshot)._
