# coreLM source

## Exact Glimmer K/V candidate

`serve_glimmer` now has a separate `POST /v1/forward-exact` component route.
It accepts bounded binary `GFX2` token-ID frames and returns binary `GFO2`
frames with actual generated IDs, final-logits digest, and a deterministic
digest of every exact layer/head K/V page, plus streamed hashes of the frozen
weight roster, tokenizer, config, and their composite. This route uses
`forward_with_backend(ExactKvCache)` and never falls back to geometric memory.
The existing `/v1/forward` JSON route remains a different, non-exact path.
No harness integration or full-model certification follows from this
component route alone. See
`globalmind-runtime/EXACT-GLIMMER-KV-DESIGN.md`.
`POST /v1/tokenize-native` accepts bounded GTK2 text frames and returns GTO2
token IDs from the model-owned tokenizer, optional BOS from `config.bos_token_id`,
the pinned tokenizer digest, and a request-bound receipt. Malformed, truncated,
empty, and invalid-flag frames are HTTP 400. Probe with `smoke_token_wire`.
The JSON `/v1/tokenize` route remains a different, non-receipted path.

`exact_kv::ExactKvCache` is a native, full-context K/V backend for Glimmer's
`AttentionBackend` seam. It stores post-position-transform K/V in token order,
applies sliding eligibility before softmax, and rejects position/shape
mismatches. `glimmer_exact_smoke` uses real frozen weights and compares an
uninterrupted next-token forward with an owned cache snapshot continuation.
This only checks in-memory continuation. SurrealKV restart parity, independent
MLX logits, mind recall, and governed mission integration remain separate
gates; see `globalmind-runtime/EXACT-GLIMMER-KV-DESIGN.md`.

The isolated `globalmind-runtime/tests/glimmer_native_full_pass.rs` component
gate now proves disposable SurrealKV restart and MCP-selected prefix-witness
K/V hydration with full 52-layer bit-exact continuation. The prefix witness
does not certify distilled knowledge use or model-neutral serving.
For an offline independent oracle, `GLIMMER_LOGITS_OUT=<path>` writes the
smoke's last-token logits as little-endian f32 bytes after the bit-parity gate.
`GLIMMER_EMBED_OUT=<path>` writes its first-token post-normalization embedding
for a stage-local MLX comparison. The frozen MLX reference keeps this tensor
in bfloat16 while this Rust candidate retains f32; the observed full logits
do not match exactly. See
`../../results/glimmer-forward/independent-mlx-oracle-2026-11-04.md`.
GLIMMER_EMBED_RAW_OUT writes the same token's raw row after out-scale for a
local precision diagnosis. This diagnostic is outside production model I/O.
GLIMMER_BF16_EMBED_ABLATION=1 opts into a lab-only bfloat16 embedding profile.
It made the real BOS embedding exact against MLX but slightly worsened the
full-logit error, so it is benched and off by default. GLIMMER_QPROJ0_OUT and
GLIMMER_PRENORM0_OUT emit layer-0 diagnostics for the same offline comparison.

## Purpose

Contains the Tier 0 scheduler, backend seam, parity gate, and executable health check.

## Inputs/Outputs

The library accepts tokenized `GenerateRequest` values and returns ordered tokenized `GenerateResponse` values; the executable prints one deterministic health-check response.

## Dependencies

The parent `core_lm` crate and Rust standard library.

## Qualification

Run `cargo test --manifest-path ../Cargo.toml` from this directory.
