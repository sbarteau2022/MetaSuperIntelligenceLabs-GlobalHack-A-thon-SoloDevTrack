# Glimmer native forward smoke preflight

Date: 2026-11-03. Status: preregistered setup gate, not forward qualification.

The shared swarm seat is the frozen Muse Glimmer checkpoint on the WD Drive.
The existing Qwen Layer-3 test is a real weight-backed SurrealKV bridge smoke,
but its 40-layer Qwen implementation cannot decode Glimmer's 52-layer, 6656
hidden, mixed 4/5/6-bit affine, gated-Q, sandwich-norm, NoPE/sliding/full
attention architecture. The separate Rust server's `native_qwen` rejects any
non-Qwen3.5 MoE config; `native_nemotron` is admission-only and rejects
Glimmer. The installed Glimmer implementation is Python MLX/SGLang, so it
cannot serve as the Rust-native model I/O boundary. Port 30000 was not
listening at inspection. No Glimmer forward output or tensor K/V in the one
GlobalMind can be claimed yet.

Before calling a Glimmer forward run, preflight must rehash every frozen
artifact, bind Glimmer config/tokenizer identities and architecture, reject
the retired `tppe-reference-v1` manifest, require a native executable plus
its pinned SHA256, and refuse writes to `hypermind/core`. A valid native
executor must load the exact four mixed-quantized shards, execute all 52
layers on the same tokenizer IDs as the frozen Glimmer tokenizer, emit
receipted hidden/logit and exact K/V identities, and prove continuation
parity with both uninterrupted and Surreal-recalled K/V. The executable must
use native typed/binary I/O; JSON is allowed only for bounded on-disk model
metadata and MCP transport, never between harness and model compute.

The existing Qwen test remains the current integration baseline. The
Glimmer preflight intentionally fails until a new candidate TPPE contract
and native executable are qualified. Do not change the manifest's identity
to make a gate green. Keep the live `hypermind/core` quarantined and write
only to disposable SurrealKV for the smoke.

Promotion gates: native full-model logits vs an independent frozen reference
on a held-out tape, exact-model K/V round-trip and identity-mismatch misses,
same shared weight buffer across two carved roles, one receipted harness
forward through the MCP-to-harness automaton, and no scope escape. A partial
layer probe is a component, not the promotion gate.
