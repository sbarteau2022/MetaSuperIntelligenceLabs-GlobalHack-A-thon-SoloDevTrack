# Checkpoint — Queue 1.2 Rollback Redo (2026-11-02, session paused by owner)

Status: IN PROGRESS. Nothing committed, staged, or pushed. Do not reset/clean
the dirty worktree. Resume from step 5 below.

## Scope

Rollback boundary is Queue 1.2 TPPE qualification — NOT Queue 1.1 tracing
machinery. Redo and overwrite all work from Q1.2 onward. Three blockers:

1. TPPE state was rebuilt per token using that token's ID for earlier steps.
2. Revolution count was supplied independently of phase turns.
3. Q/K operation was a rotary hybrid; crystallization used separate entity
   transactions (crash = partial run). Original bench void; corrected bench
   was only a 2-prediction smoke.

## Done and verified this session

- [`TppeState::step()`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/tppe.rs:89)
  now takes only `(config, drive_fast, drive_slow)`. Revolutions are derived:
  `w = (theta_fast_unwrapped + theta_slow_unwrapped) / TAU`. Phase retreat
  fails closed. New tests: derived-revolutions identity, phase-retreat,
  boundary saturation via real steps.
- [`tppe_states_for_positions()`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/trace/qwen36.rs)
  replaces the per-token rebuild: one sequential carry, purely positional
  drives `(step+1)*0.017 / (step+1)*0.009`, token ID never drives any step.
  Same helper now used by
  [`globalmind_stage.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/bin/globalmind_stage.rs)
  and [`tensor_kv_qualification.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/tests/tensor_kv_qualification.rs)
  (both carried private copies of the flawed rebuild — removed).
- Pure Q/K boundary in
  [`apply_position_encoder_in_place()`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/trace/qwen36.rs):
  native RoPE arm unchanged (theta=10,000,000, 32 pairs, head_dim 256,
  partial 0.25); angle-only = T2 phase rotation only; TPPE arms = T2 phase
  rotation + scale `1 + 0.05 * poincare_radius`. No `position * inv_freq`,
  no `0.01 * phase` hybrid term in any TPPE arm.
- [`replay/mod.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/replay/mod.rs):
  `REPLAY_REVOLUTIONS_PER_STEP` deleted; context digest now binds
  `derived-revolutions-v2` (old walk artifacts are therefore invalid and
  must be regenerated). Replay test updated from the fake `(0, 1)`
  revolutions assertion to derived-count monotonicity.
- [`admit_crystallization()`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/store.rs):
  ONE Surreal transaction containing phase events 1–3, packets, atoms,
  trajectory (header/edges/steps/loops/origins), charts. Per-entity SQL
  builders (`packet_sql`, `atom_sql`, `trajectory_sql`, `chart_sql`,
  `run_event_sql`) extracted; single-entity methods retained for non-
  crystallization callers. Both crystallize paths in
  [`globalmind_distill.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/bin/globalmind_distill.rs)
  now call the bundle (with already-crystallized pre-check; failed event
  remains post-hoc terminal bookkeeping only).
- [`distillation_surreal_live.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/tests/distillation_surreal_live.rs)
  crystallizes via the bundle and includes a poisoned-bundle test
  (duplicate packet id) proving zero events/packets/atoms/trajectory
  persist on failure. PASS (7.77s, isolated SurrealDB 3.3.5).
- [`globalmind_tppe_qualify.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/bin/globalmind_tppe_qualify.rs)
  rewritten as v3 with preregistered decision rule in its header:
  <100 predictions = smoke OPEN; bench if paired ΔNLL 95% CI entirely
  above +0.01 nats vs RoPE; promote only if CI entirely ≤ 0 and no wall
  regression; radial winner by paired linear-vs-log CI, else tie.
  Repeats < 3 rejected. `paired_nll_stats()` added and exported via
  [`mod.rs`](GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime/src/trace/mod.rs).

Gates run so far: `cargo check --all-targets` PASS;
`cargo test --lib` 76 passed / 0 failed / 3 ignored;
`cargo test --test distillation_surreal_live` PASS.
NOT yet run after these changes: clippy, full `cargo test`, fmt check,
`git diff --check`.

## Held-out tape (created, qualification NOT yet run)

- File: `globalmind-runtime/inputs/heldout-token-tape.bin` (GMTT v1,
  384 tokens, 383 predictions, 1,578 bytes).
- Source text: `MasterDesignDocument/C13-Testing-Necessity-Benchmarks/Design.md`,
  sha256 `be3a37092bbde203d5808a133c7079bf2bb2ecd846af6a02c3bf4a76f873b4bf`.
- Tape sha256 `93098d3455074ae37e01dfeb0da9434f31670e9cca7a054f71dea87be0284f6b`.
- Contamination check: 314 sliding 80-char windows, 0 hits in `history.txt`.
- Tokenized with the roster Qwen `tokenizer.json` via `tokenizers 0.23.3`
  in `/tmp/gm-tok-venv` (ephemeral; recreate with
  `python3 -m venv /tmp/gm-tok-venv && /tmp/gm-tok-venv/bin/pip install tokenizers`).

## Resume here — exact next steps

1. Run the held-out bakeoff (est. 20–30 min; 4 arms × 3 counterbalanced
   repeats × 384 tokens on the real Layer-3 path):

   ```bash
   cd /Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/globalmind-runtime
   cargo run --release --bin globalmind-tppe-qualify -- \
     manifests/qwen36-opus-candidate.manifest inputs/heldout-token-tape.bin 3
   ```

2. Record the verdict in a new `results/queue2/tppe-bench-v3-heldout.txt`
   (overwrite the v2 smoke's role; keep v2 file as provenance). Promoted /
   benched / radial_winner come from the runner's decision lines only.
3. Re-execute Q1.2–Q1.5 under the winning radial law (or
   `linear_log_v1` still labeled candidate on a tie): trace → extract →
   judge → replay (law argument) → open new admitted manifest →
   crystallize → audit on a FRESH isolated SurrealDB
   (`surreal` is at `/opt/homebrew/bin/surreal`; never target
   `hypermind/core`), then `cargo test --test tensor_kv_qualification`.
   Q1.6/Q1.7 rerun only if their contracts embed the old TPPE digest.
4. Overwrite receipts: `results/queue1-vertical-slice-receipt.txt`,
   `results/queue2/queue2-ablation-report.txt` (A4),
   `globalmind-runtime/results/tppe/REPORT.txt`,
   `results/distillation/*`, `globalmind-runtime/README.md` Queue 2 rows,
   and the handoff Queue 2 TPPE correction section. Fixture label
   `tppe_linear_log_v1_candidate` stays until the bakeoff selects a winner.
5. Final gates, all boolean: `cargo fmt --check`,
   `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`,
   full `cargo test`, `git diff --check`. Report promoted/benched/none
   exactly as measured. Nothing ships on the 3-token smoke.

## Changed files this session (uncommitted)

`globalmind-runtime/src/tppe.rs`, `src/trace/qwen36.rs`, `src/trace/mod.rs`,
`src/replay/mod.rs`, `src/store.rs`, `src/bin/globalmind_distill.rs`,
`src/bin/globalmind_stage.rs`, `src/bin/globalmind_tppe_qualify.rs`,
`tests/distillation_surreal_live.rs`, `tests/tensor_kv_qualification.rs`,
`inputs/heldout-token-tape.bin` (new), plus the prior session's receipt
corrections (queue1 receipt, queue2 report, tppe/harness REPORTs, README,
handoff rollback-boundary section).
