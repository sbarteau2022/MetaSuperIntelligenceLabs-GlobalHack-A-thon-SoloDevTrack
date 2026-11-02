---
name: mlx-systems-engineering
description: Use for Rust, TypeScript, Python, Erlang/OTP, MLX, C++, Swift, or Metal-shader engineering with the local Qwen3.6 Rapid-MLX model. Inspect conventions, use bounded language checks, and apply language-specific correctness practices.
---

# Local Systems Engineering

Use this skill for implementation, debugging, review, or performance work in Rust, TypeScript, Python, Erlang, MLX, C++, Swift, or Metal.

## Required Workflow

1. Call `language-toolchain.read_project_conventions` and `language-toolchain.detect_toolchains` for the workspace before changing code.
2. Read the project instructions and existing build configuration. Do not replace established tooling with a personal preference.
3. Make the smallest coherent change. Keep public interfaces and error behavior explicit.
4. Run the matching `language-toolchain.run_language_check` operation after edits. Report the command, exit code, and limitations.
5. For cross-language boundaries, verify data ownership, error translation, cancellation, and serialization rather than assuming equivalent semantics.

## Language Expectations

| Language | Default practice | Validation |
|---|---|---|
| Rust | Model ownership and lifetimes; avoid `unwrap` on recoverable paths; use `Result` with context; avoid unnecessary allocation in hot loops. | `format`, `check`, `test` |
| TypeScript | Preserve strict types; avoid `any`; validate external data; keep async cancellation/errors explicit; use the project package scripts. | `typecheck`, `lint`, `test` |
| Python | Prefer typed, small functions; validate I/O boundaries; avoid import-time work; preserve virtual-environment tooling; use vectorized MLX operations where applicable. | `syntax`, `lint`, `test` |
| Erlang | Preserve OTP supervision, isolation, and message contracts; avoid blocking gen_server callbacks; make timeout/retry behavior explicit. | `compile`, `test` |
| MLX | Keep tensors on-device; avoid implicit NumPy/Python-list conversion in hot paths; make `mx.eval` synchronization deliberate; measure Metal changes with a fixed corpus. | `import`, `test` |
| C++ | Use RAII and value semantics; make ownership/nullability explicit; avoid undefined behavior; honor the project build generator and sanitizer setup. | `build`, `test` |
| Swift | Prefer value types and structured concurrency; use `async`/`await` cancellation correctly; isolate mutable state with actors; honor SwiftPM/Xcode project conventions. | `format`, `build`, `test` |
| Metal | Preserve buffer/texture layout contracts and bounds checks; avoid data races; use the correct address-space and function constants; measure dispatch changes with fixed inputs and GPU capture where needed. | `syntax`, `compile`, `link` |

## Tool Boundaries

`language-toolchain.run_language_check` accepts only the listed language and operation schema. Metal `compile` writes temporary `.air` output only and removes it after validation. For a bespoke command, explain why the bounded check is insufficient and use Zed's terminal tool with confirmation. Never hide a destructive action inside a validation command.
