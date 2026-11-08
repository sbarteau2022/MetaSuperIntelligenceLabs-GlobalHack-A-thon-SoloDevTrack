# 05 — swift-mlx audit (Swift MLX stack)

**Not on this machine.** No `mlx-swift` or `mlx-swift-lm` checkout was found (a filesystem search of `~/Desktop`, `~/models`, `~/Developer`, `~/src`, `/opt`, `/usr/local` and a depth-6 `/` search returned only the SGLang checkout and the Python/Homebrew MLX installs). Per the brief, this document is research from primary sources. **Everything below is [unverified]**: it comes from search-result excerpts of the GitHub pages, not from reading the files in full. Each claim carries its URL so it can be checked.

Assumption: "swift-mlx" = Apple's MLX Swift stack maintained under the `ml-explore` GitHub organisation. If the intended target is a different Swift project, this doc needs redoing.

## 1. Repositories and their roles

| Repo | What the page says (excerpt) | Source |
|---|---|---|
| `ml-explore/mlx-swift` | "Swift API for MLX"; MIT; default branch `main`; created 2023-12-12; 485 commits; "LLM and VLM implementations are available in mlx-swift-lm" | https://github.com/ml-explore/mlx-swift |
| `ml-explore/mlx-swift-lm` | "LLMs and VLMs with MLX Swift"; MIT; created 2025-10-02; 575 commits; `main` is a **new major version 3.x**; products `MLXLLM`, `MLXVLM`, `MLXLMCommon`, `MLXEmbedders`, `MLXFoundationModels` (Apple FoundationModels bridge, gated to the macOS 27 SDK), `MLXGuidedGeneration` (xgrammar), `MLXRerankers` | https://github.com/ml-explore/mlx-swift-lm ; `Package.swift` https://github.com/ml-explore/mlx-swift-lm/blob/main/Package.swift |
| `ml-explore/mlx-swift-examples` | Example apps (LLMEval, MLXChatExample, LoRATrainingExample, llm-tool). LLM/VLM libraries **moved** to `mlx-swift-lm` | https://github.com/ml-explore/mlx-swift-examples , README https://github.com/ml-explore/mlx-swift-examples/blob/main/README.md |
| `mlx-community/speculative-decoding` | "Native speculative decoding implementation for fast LLM inference on Apple Silicon using MLX-Swift"; created 2025-11-28; 27 commits; README claims "2-3x speedups" **[vendor claim]** | https://github.com/mlx-community/speculative-decoding |

## 2. Qwen3.5 / Qwen3.6 support (release notes, excerpts)

From the `mlx-swift-lm` releases page https://github.com/ml-explore/mlx-swift-lm/releases (excerpt-level, **[unverified]** beyond titles):
- Qwen3.5 & Qwen3.5 MoE (text-only) added (PR #97); Qwen3.5 & Qwen3.5 MoE (vision) added (PR #120).
- "Optimize Qwen3.5 interleaved M-RoPE" (PR #442).
- "Qwen3.5/3.6: run the decode step through compiled traces" (PR #467).
- "Qwen3.5/3.6 MoE: fused router top-k kernel for decode (bit-identical, CI-pinned)" (PR #469).
- Qwen3-Next-80b added (PR #70).
- Speculative decoding: "Add speculative decoding" (PR #173); "expose speculative decoding in ChatSession" (PR #193); "speculative decoding telemetry and memory gating" (PR #314); MTP speculative decoding for Gemma 4 (PR #415).

Whether `Qwen3_5` in Swift implements Gated DeltaNet correctly was **not** read. The release titles suggest a GDN implementation exists (compiled traces and fused router for the MoE decode), but this is inference from titles.

## 3. Cache and speculative state

From `Libraries/MLXLMCommon/KVCache.swift` (2825 lines), excerpts only **[unverified]**: https://github.com/ml-explore/mlx-swift-lm/blob/main/Libraries/MLXLMCommon/KVCache.swift
- `KVCacheSimple` is "based on Python's KVCache" (mlx-lm reference) — the same design lineage as doc 06.
- KV quantisation requires head dimensions divisible by one of group sizes 32/64/128 (error text in the excerpt).
- `saveSpeculativeCheckpoint(convState:recurrentState:advancedBy:)` stores a checkpoint of **both a conv state and a recurrent state** plus the offset, for rollback inside a speculative verification pass (package-internal). This indicates the Swift stack already models GDN-style recurrent state rollback for speculative decoding, which the Python seat does not.
- Rotating/windowed caches: a documented caveat that after a partial rewind "the window is up to `n` rows short until it refills" (excerpt).

## 4. Relevance to the unified engine (facts only)

1. Swift stack is structurally the same as Python mlx-lm: `KVCacheSimple`, a recurrent-state checkpoint for speculative rollback, quantised KV with group-size constraints. The *concepts* transfer; the code does not.
2. A Swift path would have to reimplement the scheduler and SSD tier from scratch; no SSD KV store was seen in the excerpts (not confirmed absent).
3. Feature cadence is high (Qwen3.5/3.6 compiled decode and fused MoE kernels landed as separate PRs in the releases list). Expect Swift to track Qwen3.6 quickly, but its serving layer is not established.

## 5. Not verified

- Any file in the repos was not read in full; any API name above is from excerpts.
- No Swift build was run; no Xcode or SwiftPM resolution was attempted.
- No performance numbers (the speculative-decoding README figure is a vendor claim).
- Whether `mlx-swift-lm` 3.x `main` compiles against the local MLX (`mlx-0.32.2` in oMLX's site-packages) — not checked.
