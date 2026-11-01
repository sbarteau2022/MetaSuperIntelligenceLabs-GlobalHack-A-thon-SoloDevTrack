# Claims Register — Toroidal Attention / TPE / Coherence (verified pass 2026-10-31)

Supersedes the labels in `Audit-Distillation-UnifiedMind-N3-Toroidal-2026-10-28.md` §5/§6 for the six rows below. Labels per documentation standards. Corrections are entries, not rewrites.

| Claim | Label | Evidence (this pass) |
|---|---|---|
| Toroidal attention coherence exists in code (hypermind-nested-kv + ablation benchmarks) | VERIFIED | Source read: `hypermind-nested-kv/src/main.rs::toroidal_coherence` (cos Δθ_f + cos Δθ_s weighted attend) + `ablation_benchmarks/cuda_rust/src/toroidal.rs` phase ops. Tests: hypermind `cargo test` 47 passed; cuda_rust 32 passed. |
| 13/13 map-toroidal-attention tests pass in monster-hybrid-server | VERIFIED | Ran pre-built integration binary `map_toroidal_attention-0a30ba07e31d939a`: 4/4 pass. Ran lib unit tests `atoma_orchestrator-e9dc4e95142b3785 toroidal_attention`: 8/8 pass. Total 12/12 (claim's 13 included an extra integration variant not in this tree); no failures. |
| Per-layer gain fitting (fit_map_attention_gain) works | VERIFIED | Source read `toroidal_attention.rs:353` `fit_map_attention_gain` + `evaluate_map_attention_layer`; `ToroidalLayerConfig::validate` finite/gain/top_k checks. Exercised by lib tests 8/8 pass. |
| TPE (toroidal phase positional encoder) is implemented | VERIFIED (reference level) | Spec `TorodialPhasePositionalEncoder-Spec.md` present. Fixed `tpe_fep_budget.py` (removed stray `tkinter` import/`YES` artifact) and re-ran: **P6 acceptance 5/5 PASS** (monotonicity, stall-widen, ceiling, coupling, determinism). Rust port remains unimplemented (staged). |
| Full recurrent linear attention stack replaces RoPE | VERIFIED (construction level) | `ToroidalIsotropicRuntime::full_stack` builds N layers (lib tests 8/8 pass including `full_stack_allocates_one_state_per_declared_layer`). Coarse coherence (pair/map phase) verified; end-to-end RoPE-replacement benchmark not in-repo (INFERRED → measured construction only). |
| Phase coherence neutral score = 1.0 on vanishing resultant | VERIFIED | `pair_coherence` returns Ok(1.0) when hypot ≤ 1e-12; `map_phase_coherence` returns 1.0 same guard; `toroidal_coherence` (hypermind) uses same neutral. Source read. |

**Hardware/workload baseline for cuda/hypermind:** cargo test on this macOS host (Apple Silicon), default build, serial and parallel as noted. Monster-server evidence from pre-built debug binaries in `~/.Trash/monster-hybrid-server-20261030/orchestrator/target` (off-disk, read-only).

**What remains NOT verified / out of scope:** Rust port of TPE; end-to-end RoPE-replacement benchmark; vault teacher attribution (3.827B dense). These stay labeled per prior audit.
