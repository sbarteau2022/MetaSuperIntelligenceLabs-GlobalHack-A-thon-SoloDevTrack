---
title: "GlobalMind Prior Evidence Transfer Matrix"
document_type: "qualification"
version: "1.0.0"
date: "2026-11-01"
status: "active"
---

# Purpose

Prevent duplicate work while preserving the fresh-implementation boundary.
Before a GlobalMind runtime component is designed, inspect the current Rust
server and the trashed Monster server evidence side-by-side. Reuse invariants,
fixtures, failure modes, and measured baselines. Do not copy Monster/CoreLM
implementation or promote synthetic/component evidence as fresh certification.

# Transfer matrix

| Concern | Current Rust server evidence | Monster evidence | GlobalMind ruling |
| --- | --- | --- | --- |
| Exact prefix/KV identity | `tests/cache_control.rs` varies model, tokenizer, quantization, namespace, token path; tests immutable forks, leases, deduplicated inflight fills, bounded queues, and producer failure release. | Map K/V tests exercised layer/position identity and newest-slot selection. | Reuse the identity and concurrency invariants. Extend identity with TPPE version/config/radial law, tensor shape, address contract, and content digests. Existing tests use fixtures and do not certify real tensor reuse. |
| Prefix resource control | `src/cache/meta_automaton.rs` covers hot/warm/cold prefix tiers, sink/tail retention, predictive prefetch, and correction without prefix copying. | No stronger transferable evidence found. | Use as the baseline design before inventing another cache controller. Rerun with real model K/V and GlobalMind addresses. |
| Native model boundary | `src/serving/stream_wire.rs` implements bounded size-prefixed FlatBuffers with sequence, version, terminal, mutation, and poison checks. | Monster qualifier wrote JSON evidence but that was outside its model computation. | Reuse the binary framing pattern for runtime I/O. JSON remains forbidden inside model I/O and may not carry tensors, TPPE state, tokens, or activations. |
| Toroidal structural controls | No qualified TPPE implementation found in the current Rust server. | Periodic phase equivalence, deterministic streaming, unknown-atom rejection, and finite long-run state were measured. | Reuse these as required controls, then add expanding-radius, ideal-boundary, Lorentz-sheet, and TPPE replay checks. Structural transfer only. |
| Radial coordinate semantics | No qualified TPPE radial mapping found. | The discarded field accumulated an unbounded radius and used separate Poincare/Lorentz projections; its language gate still failed 0/24. | The fresh law operates on hyperbolic distance `u`; Poincare radius is `tanh(u/2)`. Directly treating the unbounded law as ball radius is invalid. No Monster implementation is imported. |
| Toroidal language quality | No current-server proof found. | `NATIVE_TOROIDAL_REPORT.md` recorded **FAILED 0/24** continuation quality; fixed-point feedback and no EOS/learned objective. | Bench that untrained recurrence/readout architecture. Do not reconstruct it. TPPE must run against the full real-weight reference and matched quality baseline. |
| Map attention | No direct TPPE proof found. | Four map-toroidal integration tests covered causal masks, failure closure, slot replacement, and vault round-trip. | Reuse test questions, not code or promotion claims. Old map, phase source, and model identities are incompatible with fresh GlobalMind. |
| Evidence discipline | Current server has adversarial, native execution, device, cache, and production-pipeline suites. | Qualifier separated component evidence from language quality and returned a distinct quality-failure exit. | Preserve tiered evidence and fail-closed exits. Fresh runs need exact model/build/hardware/TPPE receipts. |

# Search order for every next component

1. Current Rust server source, tests, and measured evidence.
2. Monster reports/tests as read-only failure and test-design evidence.
3. Fresh implementation only for an uncovered requirement.
4. Matched rerun when any identity field or contract changed.

# Non-transferable facts

- Monster/CoreLM source is excluded from the competition implementation.
- Monster’s successful structural tests do not override its failed language
  continuation gate.
- Current Rust cache tests use fixtures and JSON configuration; they establish
  useful invariants but not the native real-model boundary or hardware proof.
- A prior test transfers as certification only when model digest, tensor shape,
  TPPE version/configuration/radial law, address contract, runtime digest,
  workload, and hardware identity all match.
