# Glimmer forward preflight receipt — 2026-11-03

Command from `globalmind-runtime/`:

`cargo run --release --bin glimmer_forward_preflight -- manifests/muse-glimmer-candidate.manifest`

Observed result: exit 1 **as required**. All frozen artifact bytes rehashed
successfully (`roster_digest=187597bb49578ac31e250da2240b6960d0af5487399c0df77d46ee19771b5c04`,
`input_digest=5ec5680c736f3d4aaee64827e67e8ef037a34d4e5f9ce570f66385a563b6c465`).
The gate then refused `tppe-reference-v1`: “Glimmer TPPE contract is
unqualified; reference-v1 cannot enter forward smoke.” No database connection
or model forward execution occurred.

Source audit: the external Rust server's `native_qwen` accepts Qwen3.5 MoE
only; `native_nemotron` is an admission-only Nemotron-H path. Neither executes
Muse Glimmer. The installed Glimmer implementation is Python MLX/SGLang;
SGLang port 30000 was not listening at inspection. Do not relabel that path
as a native model I/O smoke.

Gates after source change: `cargo check --tests`, `cargo fmt --check`, and
`cargo clippy --all-targets -- -D warnings` passed. `cargo test --all-targets`
passed the library, binaries, capability atlas, distillation Surreal, and
Surreal integration before reaching the debug-profile tensor K/V hardware
test. It was stopped after more than a minute of debug-profile hashing;
the parent integration lane separately reran that hardware test in release
and recorded its pass. This receipt proves checkpoint integrity and an honest
stop condition, not Glimmer model inference.
