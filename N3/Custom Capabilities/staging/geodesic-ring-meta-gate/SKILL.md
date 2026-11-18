---
name: geodesic-ring-meta-gate
description: Safe Rust Q32.31 Thalamic Geodesic Ring Buffer and Kojukhov-Bovshover Meta-Cognitive Readiness Gate.
---

# geodesic-ring-meta-gate (interface)

## Purpose
Integer-only Q32.31 thalamic ring buffer with Kojukhov-Bovshover Omega readiness
gate: ConvergentLogic carved well, MlirOpId 202563, EpistemicMark Verified.
Hot push/pop with zero heap allocation on the hot path.

## When to use
Verifying/invoking the geodesic ring: `rust::synthesize::geodesic_ring_and_meta_gate`
trigger or macro-opcode `MlirOpId(202563)`. Standalone verification:
`rustc --edition=2021 --test geodesic_ring_and_meta_gate.rs` (VERIFIED green
2026-10-31, 2 passed — see RECEIPT).

## Out of scope
Floating-point arithmetic (integer-only by construction). Unbounded capacity
(power-of-two bound). Anything but `#![forbid(unsafe_code)]`.

## Type signature
Compiled gate: `tool.rs` (`geodesic-readiness-gate`, schema: `tool.json`).
- `geodesic_gate(cfg: GeodesicGate) -> bool` — q_fixed AND bounded_cap AND tests_pass
- `opcode_valid(opcode: number) -> bool` — 202563..202565
- `make_geodesic(q_fixed, bounded_cap, tests_pass) -> GeodesicGate`
Native artifact stays: `geodesic_ring_and_meta_gate.rs` + `skill.atoma_skill`.

## Source
Already in N3 (`N3/design/autonomous_skills/geodesic-ring-meta-gate/`); interface
shrunk 2026-10-31 (Agent A2); `tool.ts`/`tool.rs`/`tool.json` added. No tool/server
claims in source beyond the `rustc --test` verifier command (re-ran VERIFIED).
See `RECEIPT.md`.
