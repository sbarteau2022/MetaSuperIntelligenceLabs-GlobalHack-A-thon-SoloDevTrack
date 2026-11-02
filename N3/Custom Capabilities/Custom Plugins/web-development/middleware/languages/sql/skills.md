# sql: parameterised, constrained and index-aware
**Done means:** `sqlfluff lint` (with the dialect set) is clean, migrations run on a scratch database, and slow or new queries were checked with `EXPLAIN (ANALYZE, BUFFERS)` (PostgreSQL).
## Rules
1. Parameterise every value (`$1`, `?`, named parameters). Never build SQL by concatenating input.
2. List columns explicitly (no `SELECT *` in application code). Write joins as `JOIN ... ON`. Use `NOT EXISTS` instead of `NOT IN` when NULLs are possible.
3. NULL is three-valued: use `IS NULL`, `COALESCE`, and remember `= NULL` is never true. `UNION ALL` unless you need de-duplication. `COUNT(*)` and `COUNT(col)` differ.
4. Types: `numeric` for money, `timestamptz` stored in UTC, `text` with a `CHECK` where length matters, identity or UUID primary keys. Put integrity in the database: primary keys, foreign keys, `NOT NULL`, `UNIQUE`, `CHECK`.
5. Migrations are versioned, small and backwards-compatible (expand, migrate, contract). Create indexes `CONCURRENTLY` on busy tables. Never edit an applied migration.
6. Indexes: on foreign keys and common filters; composite order by use; partial and covering where it helps. Keep predicates sargable: no function on the indexed column, no leading wildcard.
7. Paginate large results with keyset pagination, not `OFFSET`. Avoid N+1 patterns from application code: join or batch.
8. Transactions are as small as correct. Touch rows in a consistent order to avoid deadlocks. Make writes idempotent (`ON CONFLICT`). Know your isolation level.
9. Never run `UPDATE` or `DELETE` without a `WHERE`; preview with `SELECT` and run inside a transaction.
10. Least privilege: separate roles for migrations and application, row-level security where it fits. Test backup and restore.
## Pitfalls
Implicit casts disabling an index; `BETWEEN` on timestamps; `DISTINCT` hiding a join bug; `GROUP BY` missing a non-aggregated column; time zones; `LIMIT` without `ORDER BY`.
## Versions
Pinned: SQL:2023, PostgreSQL 18, SQLite 3.54 (SQLite 3.54.0 and a PostgreSQL 17.11 client installed; PostgreSQL 18 is from knowledge; checked 2026-10-07, source: this machine; PostgreSQL 18 not verified online). This skill assumes at least PostgreSQL 14.
Rule: first run `sqlite3 --version` and read the project's pin (docker image tag, migration tool config, SELECT version();). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the PostgreSQL manual, SQL standard, "Use The Index, Luke".

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/sql.md).

Canonical source: `plugins/web-development/middleware/languages/sql/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/sql.md` (reference; do not author).
