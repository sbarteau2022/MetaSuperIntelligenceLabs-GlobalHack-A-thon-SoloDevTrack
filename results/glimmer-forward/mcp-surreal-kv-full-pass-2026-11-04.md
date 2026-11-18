# Glimmer MCP + SurrealKV exact K/V component receipt — 2026-11-04

Command from `globalmind-runtime/`:

```sh
cargo build --bin globalmind-mcp
cargo test --release --test glimmer_native_full_pass glimmer_native_mcp_kv_full_pass -- --ignored --nocapture
```

Observed on the local Mac against frozen Muse Glimmer weights, a disposable
SurrealKV server on an ephemeral port, and the real `globalmind-mcp` child:

```text
[WeightManager] Metal GPU acceleration enabled
PASS Glimmer MCP prefix atom + disposable SurrealKV restart + exact 52-layer K/V continuation: pages=104 logits=202048 argmax=13796 authority=52485d2e3f96c1957fa1de8c8c007564406618ae96ea119b76ab382f729720cf recall=65330f2b4aa7512ac1b79fb97d346b3116b8d1d32c34eae579baa44b9840c62c kv=85c24da30002f4903efe19d45ae0f6de36925743f22ea361aa73a193caac3adf
test result: ok. 1 passed; 0 failed
```

The run rehashed all frozen model shards, tokenizer, config, and index. It
executed a real prefix through all 52 layers, persisted 104 exact head pages
under one model-bound session, killed and restarted SurrealKV, attested the
MCP child's authority, recalled the linked prefix atom at a TPPE-derived H3/T2
address, and checked that the atom ID, address, and content digest matched.
MCP's exact K/V digest matched the stored page. Typed payload hydration then
reassembled every layer/head. The resumed next-token logits matched the
uninterrupted run bit for bit. A changed model digest, changed page digest, or
missing head was rejected; perturbing an actual K/V value changed the logits.

**Boundary:** The atom here is a real Glimmer prefix witness created in the
disposable database, not a teacher-neutral distilled knowledge atom. It gates
selection of exact tensor pages; it does not demonstrate that Glimmer reasoned
over distilled cross-model knowledge or improved answer quality. The model I/O
is native token IDs and f32 tensors; MCP JSON-RPC is control transport only.
The proof uses local untracked `core_lm/` source, so the gate is not yet
reproducible from a clean checkout. Do not present it as a submission-ready
universal decoder or a full coherent-mind certification.

Targeted compiler gate `cargo check --tests` passed; runtime library gate
`cargo test --lib` passed 102 tests with 3 ignored. Strict Clippy is blocked
by 13 existing candidate `position_transform.rs`/`model_loader.rs` lints; the
mixed-bit loader itself is quarantined and fails before reading the weights.
