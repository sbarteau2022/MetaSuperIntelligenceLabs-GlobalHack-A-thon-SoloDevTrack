# Native GTK2/GTO2 tokenize route — hardware smoke, 2026-11-04

Isolated `serve_glimmer` on 127.0.0.1:18111 (Metal GPU enabled, model dir
Muse-Glimmer-q4km-gs128-MLX); ports 8000/8011/4810 untouched. Probe:
`core_lm/target/release/smoke_token_wire`.

PASS native tokenize: token_ids=[200000, 954, 7963, 323, 11698, 373]
tokenizer=c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d
receipt=c45a93fabc3275974bcb0fd08d3ecdf5eddfc7e647af8e0ddc7049b448ce39eb bos=200000
Malformed frames each got HTTP 400: truncated GTK2, wrong magic, invalid add_bos,
empty text length, header-only.

Limits: one request, one checkpoint; no backend consumer yet (handoff item 2).
Unit tests: `cargo test --lib native_token_wire` 2 passed.
