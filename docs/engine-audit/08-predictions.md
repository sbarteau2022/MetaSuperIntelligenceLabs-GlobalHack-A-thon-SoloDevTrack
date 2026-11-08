# 08 — Predictions: the next implementations in this space

Each prediction names the evidence in docs 01–06 that drives it, a confidence, and what would falsify it. Confidence is a judgement, not a measurement.

## P1. A single "verify window" primitive becomes the shared core of every speculative path — high confidence

- **Evidence.** oMLX has three separate verify-and-draft paths: mlx-lm MTP patch (`omlx/patches/mlx_lm_mtp/batched_head.py:90`), VLM MTP (`omlx/speculative/vlm_mtp.py:266`), DFlash (`omlx/speculative/dflash_drafter.py:170`) — all of which reuse "the shared verify forward, acceptance, cache rollback and emission" (`dflash_drafter.py:1–12`). SGLang routes every drafter through `spec_registry.py` (`sglang/.../speculative/spec_registry.py:78, 181–183`). rapid-mlx has `request_policy.py` to decide per request. The pattern is converging on proposer-plugin + shared verify.
- **Next implementation.** A proposer trait with `propose(k)` and a shared `verify_window(k+1)`, adopted in mlx-swift-lm (which already exposes speculative decoding in `ChatSession`, releases PR #193) and in mlx-lm.
- **Falsified if** a major engine ships a second, independent verify path for a new drafter and keeps it.

## P2. Deferred-commit recurrent state becomes standard for hybrid (GDN) models — high confidence

- **Evidence.** oMLX has a deferred-commit fused verify kernel (`omlx/patches/qwen35_gdn_verify_fused.py:1`), GDN checkpoints in the prefix cache (`prefix_cache.py:312–461`) and SSD-persisted GDN snapshots (`paged_ssd_cache.py:1323–1361`). mlx-swift-lm's `KVCache.swift` stores a conv+recurrent checkpoint for speculative rollback (`saveSpeculativeCheckpoint`, excerpt, unverified). The seat has neither (doc 01 §2.1).
- **Next implementation.** Checkpoint-granularity knobs (per block, per window) and GDN snapshot persistence in the other engines; the seat's atomic-commit design (DESIGN-SPEC §3, I2) becomes table stakes.
- **Falsified if** speculative decoding on Qwen3.5/3.6 is shipped without any recurrent-state rollback mechanism and remains exact under greedy (A5-style check).

## P3. Verify-shape kernels, not draft-shape kernels, become the hot path — high confidence

- **Evidence.** oMLX has `qwen35_verify_qmm.py` (1579 lines, "Verify-shape quantized-matmul kernels for native MTP", `:1`), `qwen35_gdn_prework.py` (2097 lines, "for Qwen3.5/3.6 verify and selected decode paths"). rapid-mlx has the fused single-token GDN decode (`qwen35_fused_gdn_decode.py:1–12`) and a prefill kernel (`gdn_prefill.py:1–10`). The speedup claims for MTP depend on verify throughput, not draft throughput (vendor README `rapid-mlx/README.md:25–33`, unverified).
- **Next implementation.** Fused k+1-position verify for the MoE router and routed experts (oMLX has the router/experts for decode only, `qwen35_moe_routed_decode.py:1`), and a verify-shaped GDN recurrence in Swift (release #467 "compiled traces" is decode-only per its title).
- **Falsified if** draft-side (1-token) cost dominates measured end-to-end time on this hardware.

## P4. Attention stays exact in every production engine; geometric retrieval will not become attention — high confidence

- **Evidence.** oMLX and rapid use exact paged or slab KV (`paged_cache.py`, `compiled_decode.py:1–25`); SGLang uses radix-tree prefix sharing with exact attention (`radix_cache.py:281`); mlx-lm and mlx-vlm use exact KV caches (`cache.py:339, 423`). The seat's geometric map is the only approximate attention in the set (doc 01 §2.2 item 4), and it scans every atom per query (`geometric.rs:259–281`).
- **Next implementation.** Approximate retrieval (ANN, VP-tree) used as **memory** over evicted or cross-session KV, not as the layer's attention. This is consistent with SSD tiering already present in oMLX (`paged_ssd_cache.py:1623`).
- **Falsified if** an engine ships approximate attention for the full-attention layers of Qwen3.5/3.6 and matches the reference greedy output on A4-style prompts.

## P5. Compiled whole-step replay spreads, with explicit cache inputs — medium confidence

- **Evidence.** rapid-mlx `compiled_decode.py:1–25` ("Shape-stable, whole-step compiled replay…; mutable cache arrays explicit inputs and outputs of `mx.compile`"); mlx-swift-lm releases "run the decode step through compiled traces" (PR #467, title only, unverified body).
- **Next implementation.** oMLX and the Swift stack converge on the same contract (explicit cache I/O, no request state in closures). The seat should adopt the contract when it reaches P9 (DESIGN-SPEC §8).
- **Falsified if** the compiled path is abandoned for the eager path in a later release of rapid-mlx or mlx-swift-lm.

## P6. Bit-exact, fail-closed kernel enrollment becomes the norm — medium-high confidence

- **Evidence.** rapid `qwen35_fused_gdn_decode.py:1–12`: "Enrollment is model-local and fail-closed. A real Metal probe must reproduce the stock output… bit-for-bit". oMLX `mlx_lm` GDN kernels pin packed-vs-explicit-tree equality (`gated_delta.py:1–15`). Swift release notes say "bit-identical, CI-pinned" (PR #469).
- **Next implementation.** Every custom kernel ships with a parity test against the reference and a kill-switch env var (both `gated_delta.py:15` and rapid use this pattern).
- **Falsified if** a new Qwen3.6 kernel lands without a reference-parity test in the same release.

## P7. The MTP head becomes the default draft source for Qwen3.6-MoE across MLX engines — medium confidence

- **Evidence.** The checkpoint ships `model-mtp.safetensors` (doc 01 R5); `mlx_vlm/split_mtp.py:1–8` extracts it into a standalone drafter; oMLX makes MTP "Lightning" the default text path with depth ceiling 8 (`model_settings.py:37`); rapid-mlx lists `--enable-mtp` and an "MTP text mode" as its default serving mode (`cli_parser.py:1130`, vendor README). SGLang loads the MTP head in a separate class (`qwen3_5_mtp.py:94`).
- **Next implementation.** The seat's P6 (DESIGN-SPEC §8) is the same move; the bottleneck for the seat is correctness, not the drafter.
- **Falsified if** a Qwen3.6 benchmark on this machine shows MTP slower than the no-draft path at all depths under greedy.

## P8. Adaptive draft depth from acceptance statistics — medium confidence

- **Evidence.** SGLang `speculative/adaptive_spec_params.py` and `adaptive_runtime_state.py` (names only, unverified contents); oMLX `set_mtp_depth` / `is_mtp_depth_fixed` (`patches/mlx_lm_mtp/__init__.py:76–86`) — a depth that can be fixed or adapted; rapid's per-request MTP counter (`scheduler.py:1032`).
- **Next implementation.** A controller like DESIGN-SPEC §4.1 becomes common; the open question is the objective (accepted tokens per verify-second is the one that matters).
- **Falsified if** all three engines keep fixed depth in their defaults through the next two releases.

## P9. Speculation without receipts is the gap nobody fills in the engines — medium confidence

- **Evidence.** None of the audited engine sources (oMLX, rapid-mlx, SGLang, mlx-lm, mlx-vlm) emits a per-step binding receipt on the wire; their outputs are plain OpenAI-style JSON or SSE (`oMLX/server.py:3997`, `rapid_mlx/server.py:29`). Receipt-bound output is what the seat and the harness need (AGENTS §3). This is a negative finding from the read paths; a grep over the full trees was not run.
- **Next implementation.** Receipt-bound speculation stays on the harness side (the seat, the governor) and is not pushed into upstream engines. Expect upstream to add telemetry, not receipts.
- **Falsified if** an upstream engine adds signed or hashed per-token records to its public API.

## P10. The Rust seat's real competitor is a Swift port of the same loop, not the Python engines — low-medium confidence

- **Evidence.** mlx-swift-lm 3.x is an active, separately-versioned library with Qwen3.5/3.6 MoE, speculative decoding, and a KV checkpoint for recurrent state (doc 05 §2–3; all unverified beyond excerpts). The Python engines are heavier to embed in a native typed-binary boundary.
- **Next implementation.** A Swift engine that consumes the same typed frames as the seat, or a Rust port of mlx-swift-lm's MoE router.
- **Falsified if** mlx-swift-lm does not run Qwen3.6-MoE on this machine when a reviewer tries it (not tested here).

## Summary table

| # | Prediction | Confidence | Main evidence |
|---|---|---|---|
| P1 | Shared verify-window primitive | high | oMLX three verify paths; SGLang registry |
| P2 | Deferred-commit GDN state standard | high | oMLX `qwen35_gdn_verify_fused.py`, GDN checkpoints |
| P3 | Verify-shape kernels are the hot path | high | oMLX `qwen35_verify_qmm.py`, `qwen35_gdn_prework.py` |
| P4 | Attention stays exact; geometric retrieval becomes memory, not attention | high | exact KV in all engines; seat's full scan |
| P5 | Compiled whole-step replay with explicit cache I/O | medium | rapid `compiled_decode.py`; Swift #467 |
| P6 | Bit-exact, fail-closed kernel enrollment | medium-high | rapid `qwen35_fused_gdn_decode.py`; Swift #469 |
| P7 | MTP head is the default Qwen3.6 draft source | medium | checkpoint `model-mtp`; oMLX depth 8; rapid `--enable-mtp` |
| P8 | Adaptive draft depth | medium | SGLang adaptive params; oMLX depth controls |
| P9 | Receipts stay outside upstream engines | medium | none of the audited wire APIs emit receipts |
| P10 | Swift port is the real native competitor | low-medium | mlx-swift-lm release list (excerpt) |
