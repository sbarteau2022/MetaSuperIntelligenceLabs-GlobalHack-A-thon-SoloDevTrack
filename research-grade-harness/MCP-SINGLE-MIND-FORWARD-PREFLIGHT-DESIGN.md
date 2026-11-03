# MCP to harness single-mind preflight

Date: 2026-11-03. Scope: read-only forward smoke admission.

The typed `GlobalMindBridge` already reads binary GMRC neighborhoods and exact-model K/V from SurrealDB. The older `MapPlugin` can launch a separately rooted `hypermind-mcp`, and that MCP defaults to `hypermind/core`. Connecting a harness bridge to one database while MCP points to another would split recall authority.

Before any forward smoke with MCP, require the MCP authority declaration to be `on` and match the bridge endpoint, namespace and database byte-for-byte. Reject `core`, `off`, `shadow`, empty fields and mismatches before connecting. No `.hypermind` directory creation, schema creation, atom minting or database write is performed by this preflight. It does not certify the MCP process or its advertised tools; a live MCP round-trip remains a separate gate. The control case runs against a disposable SurrealKV database with real atom and K/V rows, then verifies address-gated binary recall and exact K/V. Mutating one authority field must reject before database access.
