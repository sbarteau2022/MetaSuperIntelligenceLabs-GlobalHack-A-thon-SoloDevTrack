# Research-Grade Agentic Harness

Rust implementation scaffold for the plan-driven, judge-gated harness in
`../Research-Grade-Agentic-Harness-Design.md`. The binding design points are:
plan-before-execution (§1), receipts as evidence (§3.3), the MadMind runbook
engine (§4 Layer 1), and real-work gating before claims (§13 Phase 0).

## Evidence boundary

Default local orchestration is fail-closed. `Engine::run_sequential` and
`Engine::run_parallel` create escalation/failure receipts because they have no
executor. They cannot certify work. Live execution-backed acceptance uses
`run_sequential_n3` or `run_parallel_n3`; a packet with no admitted `map.*`
output fails.

Synthetic benchmark code is retained only for historical shape comparison
under the `synthetic-scaffold` feature. It is not quality, energy, throughput,
or ship evidence. See `NO-SYNTHETIC-BENCHMARKS.md`.

## Verification

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

These default checks prove deterministic code contracts, not a live runtime.
The live N3 gate is separate and intentionally fails when its binary is absent:

```sh
HYPERMIND_SURREAL=off HYPERMIND_MCP_BIN=/absolute/path/to/hypermind-mcp \
  cargo test --features live-n3 --test engine_n3 -- --test-threads=1
```

The CLI `run` command validates and then fails closed until it is wired to a
live executor. `resume` chain-verifies the prior file (forged lines rejected, exit non-zero). Receipts carry prev/chain/ts/n3 fields. The `bench` command is disabled by default. To inspect the old
scaffold only, compile with `--features synthetic-scaffold`; never cite its
numbers as results.

## Current qualification

- Unit/integration checks: code-contract evidence only.
- Live N3 feature gate: required for runtime acceptance.
- Hardware energy calibration and real-model comparative benchmarks: not yet
  qualified; the design spec labels them proposed.

## HyperMind graph/KV plugin

`GlobalMindBridge` is now the read-only typed Rust entry point for the fresh
GlobalMind substrate. It verifies the isolated Surreal contract and returns
`GMRC` binary recall bundles from H3×T2 retrieval. It refuses quarantined
`hypermind/core` and exposes tensor bytes only through the full exact-model
session/page identity (never a bare page ID). See
`GLOBALMIND-BRIDGE-DESIGN.md`. It is not yet connected to a live model executor;
the older `MapPlugin` below remains a historical text/string MCP surface and
does not qualify the nested transformer K/V path.
The explicit `cargo test --test globalmind_bridge_live -- --ignored --nocapture`
gate passed against a disposable SurrealKV instance, decoded one real atom
in a GMRC bundle, fetched one typed K/V page by full identity, and rejected
a changed model digest. This is bridge read proof, not model inference.
The A14 capability gate reads the consolidated manifest from
`../N3/CapabilityTreeSitter/capability-tree-manifest.json`.

The canonical plugin source is `../hypermind-nested-kv`. Build its
`hypermind-mcp` executable, then set `HYPERBOLIC_MAP_PLUGIN_BIN` to that absolute
binary path. Plugin describes harness lifecycle; MCP describes transport. They
are one executable, not separate graph/KV implementations. `MapPlugin::load`
checks advertised map/KV tools and a receipted health call before registering
it; `unload` closes the process. That historical MCP path uses no direct
SurrealDB connection; the fresh typed `GlobalMindBridge` above does. The old `live-n3` path remains for historical compatibility and must
not be used as evidence of an independent plugin. The standalone `../hyperbolic-map-nested-kv-plugin` directory is absent.

```sh
HYPERMIND_SURREAL=off HYPERBOLIC_MAP_PLUGIN_BIN=/absolute/path/to/hypermind-nested-kv/target/debug/hypermind-mcp \
  cargo test --features live-map-plugin --test map_plugin_live
```

The default Native/Muse lane implementations now escalate with zero reported
tokens and energy until real model executors are provided. MCP tool errors,
missing receipts, unaffordable budgets, and telemetry write failures fail a
packet. A live map/KV smoke proves storage and retrieval, not a multi-model
mind or transformer KV acceleration.

Forward preparation: `GlobalMindBridge::prepare_forward_recall` joins binary GMRC neighborhood retrieval and exact-identity tensor K/V fetch. The model executor still must consume these bytes and produce activations before a full forward-pass smoke can be claimed.

The forward-preparation bridge now checks that the exact tensor page address is a member of the decoded recalled neighborhood. An isolated two-atom SurrealKV test rejects a valid K/V page from an unrelated recalled atom. Math and active-inference limits are recorded in `../results/forward-recall-math/report.tex`; this is still a read-boundary gate, not model execution.

Radix cache keys are now domain-separated and length-delimited across parent, token block, model, tokenizer, and quantization identity. The previous raw concatenation admitted deterministic tuple collisions. The in-memory key format is versioned; cache-hit performance remains unmeasured.

An explicit release-mode real Qwen3.6 Layer-3 smoke now consumes both K/V heads returned by `GlobalMindBridge` after an isolated SurrealKV restart. Hidden output and all 248,320 logits were bit-exact against the uninterrupted control on 2026-11-02. See `../results/tensor-kv/tensor-kv-harness-bridge-report.txt`. Run it with `cargo test --manifest-path ../globalmind-runtime/Cargo.toml --release --features live-forward-model --test tensor_kv_qualification -- --ignored --nocapture`. This qualifies one Layer-3 continuation, not the full 40-layer model.
# Bounded resource selection (component)

`active_inference::BoundedResourcePolicy` selects the least predicted Q32.31
resource cost among actions that meet a judged quality floor and a declared
budget. Equal-cost ties use information gain, quality, then stable ID. Unit
mismatches, invalid estimates, and empty feasible sets fail closed. See
`ACTIVE-INFERENCE-RESOURCE-POLICY-DESIGN.md` and
`tests/bounded_resource_policy.rs`. This selector is not yet connected to the
live executor: cost/quality forecasts need calibration and runtime metering.
The older floating-point `PolicyRanker` remains separate historical behavior;
neither component measures thermodynamic heat or proves an FEP optimum.

