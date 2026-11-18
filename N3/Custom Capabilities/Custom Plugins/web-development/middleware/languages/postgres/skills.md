# postgres: the database is the last line of defence
**Done means these pass:** `psql` smoke, `EXPLAIN (ANALYZE, BUFFERS)` reviewed for new queries, migration up+down tested, `pg_prove` where harnessed. See also: sql.md (shared SQL conventions).
## Rules
1. Migrations are forward-only and additive; destructive changes go through expand (add) -> migrate -> contract (drop), never in one deploy.
2. Use `CREATE INDEX CONCURRENTLY` outside transactions on live tables; never lock writes for an index build.
3. Parameterize everything (`$1`, `$2`); string-interpolated SQL is an injection hole — the audit deny-rule `py-sql-concat` exists for this.
4. `ON CONFLICT` for idempotent writes; unique constraints are correctness, not just performance.
5. Set explicit `lock_timeout` + `statement_timeout` per role; a runaway query must die, not cascade.
6. Connection pooling (PgBouncer/pooler in transaction mode) in front of app servers; never one connection per request.
7. `timestamptz` everywhere, UTC storage; format in the user's timezone at the edge.
8. Backups tested by restore, not by existence: schedule restore drills and record the last successful one.
## Pitfalls
`SELECT *` couples code to column order; `SERIAL` without sequence ownership breaks dumps; long transactions bloat (VACUUM can't keep up); `NOTIFY` payloads over 8KB silently truncate; timezone-naive comparisons shift rows across DST.
## Versions
Pinned: PostgreSQL 18-era SQL with 17.11 client verified on this machine (NEEDS-VERIFY 2026-10-30: PG18 server vs installed 17.11 client). Floor: PostgreSQL 14.
Rule: first run `psql --version` and read the project's pin (infra config / docker image). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: PostgreSQL docs (MVCC, EXPLAIN, CONCURRENTLY, config).
Date: 2026-10-30
Status: active
See also: mongodb.md (document profile) vs this relational profile — pick per data shape, record the decision.

Canonical source: `plugins/web-development/middleware/languages/postgres/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/postgres.md` (reference; do not author).
