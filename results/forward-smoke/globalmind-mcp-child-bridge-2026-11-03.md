# GlobalMind MCP child and bridge live gate

Command: `cargo test --test globalmind_bridge_live bridge_reads_one_real_atom_as_binary_recall -- --ignored --nocapture` from `research-grade-harness/`, after `cargo build --bin globalmind-mcp` in `globalmind-runtime/`.

Observed: 1 passed, 0 failed, 6.86 s. The test started a disposable SurrealKV instance, applied the graph and K/V schemas, launched the active `globalmind-mcp` child with process-local `GLOBALMIND_SURREAL_*` settings, required its receipted `mind.db_status` to match the same endpoint/namespace/database with `contract=verified`, decoded a GMRC atom neighborhood, fetched an exact tensor K/V page, and checked mismatch behavior. The child did not create `.hypermind` in its isolated root. The live `hypermind/core` database was not used.

Earlier observed failures: numeric atom IDs did not decode as the bridge's canonical string `RecordId`; using string IDs against retired `hypermind-mcp` failed hydration because that MCP only accepts numeric `external_id`. The test now uses the active model-neutral MCP. This is a real MCP/read boundary proof, not a model forward or benchmark.
