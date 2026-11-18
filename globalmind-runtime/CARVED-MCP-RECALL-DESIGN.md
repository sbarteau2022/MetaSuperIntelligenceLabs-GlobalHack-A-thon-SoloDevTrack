# Carved MCP recall contract

Date: 2026-11-03. This extends the one GlobalMind MCP child; it does not create a worker map.

## Scope

`mind.recall_scoped` accepts the same H3×T2 query as `mind.recall` plus an explicit bounded set of cognitive well IDs. It calls the existing `retrieval::scoped::retrieve_scoped` against persisted address-well memberships and only returns atoms in those wells and edges whose endpoints are both visible. It returns the scope digest, query/ranking digest, members, and internal edges. Empty, duplicate, missing-membership or budget-saturated scopes fail closed. The result is read-only.

The child does not authorize who may choose a well. The backend governor must supply well IDs from a server-held, role-bound, expiring capability; it must never forward a caller-selected scope unvalidated. Until that handle exists, `/api/harness/mcp/execute` allows only L0 to use unscoped recall and denies worker recall. Exposing the scoped tool in the child is a read-surface component, not carved swarm completion.

## Backend capability binding

L0's harness process holds a server-configured issuer secret outside model text. A new loopback route checks that secret, obtains a coordinator compliance permit, and mints an unpredictable 256-bit handle kept only in the backend process. The registry binds worker role, canonical mind target, prefix/model contract identity, well IDs, and expiry. Scope sets contain 1..12 distinct valid record IDs. The worker submits only the handle and query; the backend replaces any caller-provided `well_ids` with the registered set and switches the tool to `mind.recall_scoped`. A missing, expired, role/target/prefix-mismatched handle fails before child contact. Worker exact-K/V remains denied until it has a separate exact tensor entitlement. Restart invalidates handles; reissue requires L0 authorization. This is a local smoke capability, not a distributed multi-node token protocol.

## Proof gates

Use the existing isolated Surreal scoped retrieval fixture for membership and cross-edge exclusion. Add an MCP child test that calls the scoped tool against the same fixture and checks the returned scope digest, visible atoms and internal edge. Falsifiers: a hidden atom appears, a cross-scope edge appears, a missing well succeeds, or child authority differs from the configured mind. Do not run against `hypermind/core`.
