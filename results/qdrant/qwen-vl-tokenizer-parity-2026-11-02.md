# Qwen VL tokenizer parity receipt

The Rust-native tokenizer loads the pinned 11 MB tokenizer file, rejects a
digest mismatch, and encodes plain text with special tokens enabled.
`hyperbolic memory` yields `[68192, 65, 7762, 4938, 151643]`, matching the
offline MLX ingest oracle. Empty and over-cap queries fail closed.

On 2026-11-02: local installed-checkpoint test 1/1 passed; backend
`cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, and
`cargo fmt --check` passed. This proves tokenizer parity for one text, not
28-layer hidden-state parity or Qdrant dense retrieval.
