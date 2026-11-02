# redis: a cache with an eviction story, not a database with hope
**Done means these pass:** `redis-cli PING` smoke, `SLOWLOG GET` reviewed for new access paths, keyspace audit (`INFO keyspace` + key count / memory per namespace sane). See also: postgres.md (system of record when Redis is cache, not store).
## Rules
1. TTL on everything: every key gets an expiry at write time; no immortal keys without a reason recorded next to the writer.
2. Namespace keys as `{service}:{entity}:{id}` (e.g. `billing:session:abc123`); never bare ids, never cross-service prefixes.
3. Eviction policy explicit and recorded (`maxmemory-policy`): `allkeys-lru` for pure caches, `volatile-*` only when every key has a TTL; the default `noeviction` is a pager, not a policy.
4. No Redis as primary store without a persistence story: AOF and/or RDB configured, restores drilled; otherwise Redis is cache and the database of record lives in postgres.md.
5. Check-and-set goes through Lua scripts (`EVAL`) or native atomics (`SET NX`, `INCR`); never read-modify-write across a round trip.
6. Connection pooling in front of app servers; never one connection per request, and cap pool size against `maxclients`.
7. Big-key discipline: no MB-scale values — chunk lists/hashes (`HSCAN`/`SSCAN` pagination) or externalize blobs to object storage and keep only pointers.
8. `KEYS *` is forbidden outside local dev; iterate with `SCAN` (backoff on `busy`) in every other environment.
9. `MONITOR` only in dev: it halves throughput and leaks key data into logs; use `SLOWLOG` and `INFO stats` for prod diagnosis.
10. Fail closed on cache errors: a down cache degrades to the database with backpressure, never to unbounded retries or a thundering reconnect storm.
## Pitfalls
Cache stampede on hot-key expiry — use locking (`SET NX` mutex) or singleflight so one worker recomputes; cold-start thundering after a flush/restart warms nothing because every miss queues behind the same query; `KEYS *` in prod blocks the event loop on large keyspaces; unbounded pub/sub buffers (`client-output-buffer-limit` unset) OOM the server under a slow subscriber; clock-sensitive TTL drift across replicas/containers silently shortens or stretches expiries.
## Versions
Pinned: none claimed (NEEDS-VERIFY 2026-10-30: `redis-cli` MISSING on this machine, `redis-cli --version` exit 127 — no version measured).
Floor: adopt the project's server pin as the floor and record it here; until then, no versioned commands beyond Redis 6-era basics without checking the server (`INFO server`).
Rule: first run `redis-cli --version` and read the project's pin (infra config / docker image / Valkey fork note). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change. Valkey compatibility: treat Valkey as a separate pin, recorded explicitly, never assumed from a Redis version.
Sources: Redis docs (EXPIRE, eviction, persistence, EVAL, SCAN) — unopened on this machine, verify before citing specifics.
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/redis/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/redis.md` (reference; do not author).
