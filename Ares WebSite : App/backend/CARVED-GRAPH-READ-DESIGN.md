# Carved graph read seam for the forward smoke

Date: 2026-11-03. Status: component contract, not live swarm certification.

The backend already stores one global atom table and twelve carved `SiloView`
membership tables. A worker must receive only a scoped read handle, never the
`CarvedSwarmMap` itself. `CarvedSwarmMap::scoped_read(role)` rejects L0 and
returns a borrowed handle whose fields are private. The handle exposes atom
payload and coordinate only when the atom ID is in that role's membership;
transition admission requires both endpoints visible. It borrows the existing
atom bytes and `Arc<FrozenBaseWeights>`, creating no second map or weight copy.

The handle is not a SurrealDB capability token. The MCP adapter must later
enforce the same role and membership against the authoritative SurrealDB
relational graph, bind a permit and map revision digest, and reject a stale
handle after recarve/erasure. Until that adapter and a real Glimmer worker run
exist, this is a backend component gate only. Never claim this wrapper is an
MCP server, full graph traversal, or a live carved swarm.

Falsifiers: L12 cannot read a planted L1 atom when overlap is zero; L1 can
read it; a cross-view transition fails even if one endpoint is visible; two
handles borrow the same frozen weight `Arc`; L0 cannot obtain a worker handle.
Any read path that hands the full map to a worker voids this gate.
