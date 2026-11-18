# Exact Glimmer K/V component receipt — 2026-11-04

Command: `cd core_lm && cargo run --release --bin glimmer_exact_smoke`

Observed on the local Mac with the frozen Muse Glimmer checkpoint and Metal
weight path:

```text
[WeightManager] Metal GPU acceleration enabled
PASS real Glimmer 52-layer exact K/V in-memory continuation: prefix_tokens=5 logits=202048 argmax=13796 logit=17.3479
```

The binary ran the actual frozen-weight prefix with an ordered full-context
attention backend, cloned its typed K/V state, and ran the same next token
through both cache instances. Every logit matched bit for bit. The top token
and printed logit also match the previously recorded short-prompt baseline.
The two continuations share the same model implementation and weights, so
this is an internal parity gate, not independent MLX/reference parity.

This cache was **not** persisted to or hydrated from SurrealKV. No atom was
recalled or consumed in this run. It does not certify a full GlobalMind pass.
The `core_lm/` source remains untracked reference work, not promoted as the
production model-neutral decoder. See `globalmind-runtime/EXACT-GLIMMER-KV-DESIGN.md`.

The separate `globalmind-runtime/src/model_loader.rs` candidate is incompatible
with this frozen mixed-bit checkpoint. It now rejects the checkpoint before
reading four shards. The `glimmer_coherent_mind` candidate's persistence and
hydration placeholders now return errors, so they cannot silently pass.
