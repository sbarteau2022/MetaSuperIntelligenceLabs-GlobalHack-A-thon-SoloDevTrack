# Scoped Glimmer recall design (2026-11-03)

One Glimmer weight owner may issue many worker handles. Each handle carries an authenticated governor permit and one or more cognitive-well IDs. The harness first validates its MCP Surreal target against the sole GlobalMind bridge. The read adapter then returns only atoms whose persisted `address_well_membership` lies in those wells, and active edges whose two endpoints are visible. It never writes SurrealDB and never returns an unrestricted recall bundle to a worker.

Preconditions: caller has validated the governor permit, role, expiry, and weight owner. This module cannot manufacture or verify that permit. Until the harness supplies that check, workers must not call it. Well IDs are parsed `RecordId`s. Empty wells, empty memberships, absent rows, malformed rows, and candidate-budget saturation fail closed. The existing workspace-wide candidate scan remains a scale limit, so this is a correctness/scope seam, not a latency claim.

Adversarial gate: out-of-well atoms and edges cannot appear; an edge with one hidden endpoint is suppressed; same well with different ordering yields deterministic results; a budget hit rejects rather than truncates. Live isolated SurrealKV integration is required before forward-smoke promotion. No semantic model quality or live Glimmer proof follows from these checks.
