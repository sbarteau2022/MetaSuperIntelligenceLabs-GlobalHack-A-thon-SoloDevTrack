# Competition provenance boundary

`toroidal_attention-vault-20261030.rs` is retained only as historical prior
work. It is not an oracle, implementation source, or admissible competition
evidence.

The eligible gain contract originates in `hypermind-nested-kv/src/main.rs` and
is mirrored in `src/hyperbolic.rs` for Rust-to-Rust verification:

`exp(g * ln(max(s + 2, 0.001)) - peak)`, normalized across candidates, with
the preregistered initial gain `g = 1.5`.
