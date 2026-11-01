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
HYPERMIND_MCP_BIN=/absolute/path/to/hypermind-mcp \
  cargo test --features live-n3 --test engine_n3 -- --test-threads=1
```

The CLI `run` command validates and then fails closed until it is wired to a
live executor. The `bench` command is disabled by default. To inspect the old
scaffold only, compile with `--features synthetic-scaffold`; never cite its
numbers as results.

## Current qualification

- Unit/integration checks: code-contract evidence only.
- Live N3 feature gate: required for runtime acceptance.
- Hardware energy calibration and real-model comparative benchmarks: not yet
  qualified; the design spec labels them proposed.

## Standalone HyperbolicMap and KV plugin

The external plugin source lives at `../hyperbolic-map-nested-kv-plugin` (a
sibling folder, independent of N3). Build its `hypermind-mcp` executable, then
set `HYPERBOLIC_MAP_PLUGIN_BIN` to the absolute binary path. `MapPlugin::load`
checks advertised map/KV tools and a receipted health call before registering
it; `unload` closes the process. The old `live-n3` path remains for historical
compatibility and must not be used as evidence of an independent plugin.

```sh
HYPERBOLIC_MAP_PLUGIN_BIN=/absolute/path/to/hyperbolic-map-nested-kv-plugin/target/debug/hypermind-mcp \
  cargo test --features live-map-plugin --test map_plugin_live
```

The default Native/Muse lane implementations now escalate with zero reported
tokens and energy until real model executors are provided. MCP tool errors,
missing receipts, unaffordable budgets, and telemetry write failures fail a
packet. A live map/KV smoke proves storage and retrieval, not a multi-model
mind or transformer KV acceleration.
