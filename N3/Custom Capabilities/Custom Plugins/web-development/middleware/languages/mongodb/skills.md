# mongodb: documents with a validator, not schemaless hope
**Done means these pass:** `mongosh` smoke (ping + server version recorded), schema-validation review on touched collections (`db.getCollectionInfos()` validator present), index audit via `db.<coll>.getIndexes()` covering every new query shape + sort.
## Rules
1. Schema validation on every collection (`$jsonSchema` validator, `validationLevel: "strict"` for new collections); schemaless writes need a validator — the app is not the schema.
2. An index for every query shape + sort: compound indexes ordered equality → sort → range; a sort with no supporting index spills (32MB in-memory cap) and then fails.
3. No unbounded arrays: cap embedded lists (`$slice` on push, fixed buckets) and paginate; overflow goes to a child collection, never to a document growing toward 16MB.
4. Transactions only where the topology supports them: multi-document transactions need a replica set (or sharded cluster with its limits); on standalone there are no multi-doc transactions — design single-document atomicity instead.
5. Connection pooling: one `MongoClient` per process with `minPoolSize`/`maxPoolSize` and `maxIdleTimeMS` tuned; never one connection per request, and cap against server `maxIncomingConnections`.
6. TTL indexes (`expireAfterSeconds`) for ephemeral data (sessions, OTPs, ephemeral jobs); TTL is cleanup, not correctness — never rely on it for invariants you must audit.
7. Aggregation pipelines tested on prod-shaped data: `$match` early, `explain` the stages, and run volume tests on realistic cardinality — a pipeline fast on fixtures can OOM on production.
8. Explicit write concern per path: `majority` for anything involving money or state transitions; `w: 1` only for regenerable data; reads that must not go stale use matching read concern, recorded next to the writer.
9. `_id` discipline: default `ObjectId`, never mix `ObjectId` and string ids in one collection — mixed types silently miss on equality match.
10. Backups via point-in-time (oplog / PIT restore window configured), tested by restore: schedule restore drills and record the last successful one — a snapshot that was never restored is not a backup.
## Pitfalls
Missing-index collection scans (COLLSCAN on a growing collection is a latency cliff); the 16MB document ceiling (hit via unbounded arrays or ever-wider embeddings); unbounded array growth moving documents and fragmenting storage; write-concern misunderstandings (`w: 1` acknowledged ≠ durable, `majority` ≠ all nodes); `ObjectId` vs string id mixing (queries return empty, no error); large-`skip` pagination getting slower per page (use range/`$gt` cursors); unsharded collections outgrowing one primary with no shard key chosen while it was still cheap.
## Versions
Pinned: none claimed (NEEDS-VERIFY 2026-10-30: `mongosh` MISSING on this machine, `mongosh --version` lookup failed — no version measured).
Floor: adopt the project's server pin as the floor and record it here; until then, no versioned commands beyond MongoDB 6-era basics without checking the server (`db.version()`).
Rule: first run `mongosh --version` and read the project's pin (infra config / docker image / Atlas tier note). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: MongoDB docs (schema validation, indexes, transactions, TTL, aggregation, write concern) — unopened on this machine, verify before citing specifics.
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/mongodb/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/mongodb.md` (reference; do not author).
