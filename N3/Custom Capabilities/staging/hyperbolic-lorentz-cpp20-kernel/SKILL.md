---
name: hyperbolic-lorentz-cpp20-kernel
description: C++20 constexpr Q32.31 Minkowski hyperbolic inner product and geodesic cursor kernel.
---

# hyperbolic-lorentz-cpp20-kernel (interface)

## Purpose
Constexpr Q32.31 Minkowski bilinear form `<u,v>_L = u0*v0 − u1*v1 − u2*v2 − u3*v3`
over `__int128_t` with compile-time `static_assert` proof. ConvergentLogic well,
MlirOpId 202565.

## When to use
Verifying/invoking the Lorentz kernel: `cpp20::synthesize::hyperbolic_lorentz_kernel`
trigger or macro-opcode `MlirOpId(202565)`. Standalone verification:
`clang++ -std=c++20 -fsyntax-only hyperbolic_lorentz_kernel_cpp20.cpp` (VERIFIED
clean 2026-10-31 — see RECEIPT).

## Out of scope
Runtime-evaluated inner products where constexpr is required. Non-`__int128_t`
accumulation (overflow). Anything the `static_assert` proof does not cover.

## Type signature
Compiled gate: `tool.rs` (`lorentz-kernel-gate`, schema: `tool.json`).
- `lorentz_gate(cfg: LorentzCheck) -> bool` — constexpr_ok AND syntax_ok
- `make_lorentz(constexpr_ok, syntax_ok) -> LorentzCheck`
Native artifact stays: `hyperbolic_lorentz_kernel_cpp20.cpp` + `skill.atoma_skill`.

## Source
Already in N3 (`N3/design/autonomous_skills/hyperbolic-lorentz-cpp20-kernel/`);
interface shrunk 2026-10-31 (Agent A2); `tool.ts`/`tool.rs`/`tool.json` added.
No tool/server claims beyond the `clang++ -fsyntax-only` verifier (re-ran VERIFIED).
See `RECEIPT.md`.
