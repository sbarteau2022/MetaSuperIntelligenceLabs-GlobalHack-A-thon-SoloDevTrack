# MCP to harness single-mind preflight

Date: 2026-11-03. Scope: read-only forward smoke admission.

The typed `GlobalMindBridge` already reads binary GMRC neighborhoods and exact-model K/V from SurrealDB. The older `MapPlugin` can launch a separately rooted `hypermind-mcp`, and that MCP defaults to `hypermind/core`. Connecting a harness bridge to one database while MCP points to another would split recall authority.

Before any forward smoke with MCP, require the MCP authority declaration to be `on` and match the bridge endpoint, namespace and database byte-for-byte. Reject `core`, `off`, `shadow`, empty fields and mismatches before connecting. No `.hypermind` directory creation, schema creation, atom minting or database write is performed by this preflight. It does not certify the MCP process or its advertised tools; a live MCP round-trip remains a separate gate. The control case runs against a disposable SurrealKV database with real atom and K/V rows, then verifies address-gated binary recall and exact K/V. Mutating one authority field must reject before database access.

Live process gate: spawn the active MCP child with explicit process-local Surreal env, call receipted `mind.db_status`, require `mode=On`, exact endpoint/namespace/database, `contract=verified`, and a full receipt digest. Fail if any field is absent or the tool is unadvertised. This attests the child-reported target; it does not independently prove that the child cannot mutate SurrealDB through other tools. That remains a role/tool authorization gate.

Correction 2026-11-03: the numeric-ID legacy `hypermind-mcp` cannot hydrate the
new model-neutral string atom IDs; a disposable live test exposed this failure.
The active child is `globalmind-mcp`, which advertises `mind.db_status` and
attests `mode=On`, exact endpoint/namespace/database, and
`contract=verified`. The child also gives a full bound receipt. The older
`parity_atoms` field applies only to the retired MCP and is not an admission
condition for the active path. The harness must pass `GLOBALMIND_SURREAL_*`
process-local variables and reject `core`.
