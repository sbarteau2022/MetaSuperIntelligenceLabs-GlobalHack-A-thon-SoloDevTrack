# Engine audit — Qwen3.6-MoE inference (2026-11-05)

Read in order:

1. [01-rust-seat-core-lm.md](01-rust-seat-core-lm.md) — the Rust seat (`core_lm`): what it computes, what is wrong, receipts.
2. [02-omlx.md](02-omlx.md) — oMLX 0.7.0 (installed locally): server, scheduler, paged/SSD KV, MTP, Qwen3.5 path.
3. [03-rapid-mlx.md](03-rapid-mlx.md) — rapid-mlx 1.0.0 (installed locally).
4. [04-sglang.md](04-sglang.md) — SGLang checkout at `~/src/sglang` (CUDA-first; design reference).
5. [05-swift-mlx.md](05-swift-mlx.md) — Swift MLX stack. Not on this machine; research from primary sources, **unverified**.
6. [06-mlx-vlm-mlx-lm.md](06-mlx-vlm-mlx-lm.md) — reference libraries (mlx-lm 0.31.4.dev132, mlx-vlm 0.7.1).
7. [DESIGN-SPEC.md](DESIGN-SPEC.md) — the unified engine: model fixes, one step function, Meta-Automaton governor, acceptance tests, order of work.
8. [08-predictions.md](08-predictions.md) — predicted next implementations, with evidence and falsifiers.

Conventions: `file:line` anchors refer to the cited tree; `REF-*` aliases are defined in doc 01; `[read]` / `[vendor]` / `[unverified]` tags are defined in doc 03. No server was started and no port was opened while writing these documents.
