# Goal: Qwen3.6-MoE native engine (Rust, Apple Silicon)

Target: `core_lm/` (`src/bin/serve_qwen.rs`, `src/qwen_forward.rs`, `src/qwen_model.rs`).
Reference: oMLX's bundled mlx_lm Qwen3.5-MoE (bf16), at `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/mlx_lm`.
Checkpoint: `~/models/fused/Qwen3.6-MoE-Combined`.

Status is recorded in `docs/frontier-loop/LOG.md` as receipts. This file holds the gates.

## Steps

1. **Forward matches the reference architecture (layer-local).**
   Gate: `cargo test --test qwen_layer_local -- --ignored`. Every layer's relative L2 error, fed the reference's own input, is under 1e-2, except the documented exceptions in that test (layer 31: bf16 residual cancellation; layer 35: router tie).
   Expected: 37 of 40 layers under 1%; layer 31 near 1.4%; layer 35 near 3.8%.

2. **Attention output and MoE half, split.**
   Gate: `cargo test --test qwen_layer31_split -- --ignored` (attention output rel L2 <= 1e-2; MoE half rel L2 <= 1e-2).
   Expected: attention output near 0.25%; MoE half near 0.34%.

3. **Greedy parity, multi-token.**
   Gate: `cargo test --test qwen_greedy_parity -- --ignored`. Eight continuation ids for "The capital of France is" equal the reference's ids exactly.

4. **Serve and time.** Blocked: needs an approved port (8003 conflicts with the standing port key).
   Gate: serve on the approved port; warm and cold timed runs with raw logs; greedy parity over the wire.

5. **Hyperbolic KV, nested in the multimodel mind.** Blocked on a decision: retrieval index over exact-softmax candidates (keeps reference parity) or attention score itself (changes the model, drops parity). Recommended: index.
   Gate (if index): recall of the hyperbolic candidate set against exact softmax, plus the steps 1-3 gates still passing.

6. **Shape change Euclidean to hyperbolic.** Assess what the change implies for the forward pass, per step 5's decision.
   Gate: steps 1-3 still pass, plus step 5's gate.

## Documented exceptions (decided 2026-11-05)

- Layer 31 and layer 35 in step 1, as recorded in `qwen_layer_local.rs`.
- The element-wise bf16 residual step check (`qwen_layer31_split.rs`, `layer31_attention_residual_within_one_bf16_step`) is kept as an ignored test: unattainable at bf16 precision, 1360 of 2048 elements over one step.
- The cumulative 41-norm oracle (`qwen_layer0_oracle.rs`) is superseded by `qwen_layer_local.rs`.
