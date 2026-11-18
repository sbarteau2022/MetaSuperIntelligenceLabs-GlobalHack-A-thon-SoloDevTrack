# sqlite: small, strict, single-writer — respect the shape
**Done means these pass:** `sqlite3` smoke + `PRAGMA integrity_check`, app tests green, migration up+down tested. See also: sql.md (shared SQL conventions).
## Rules
1. WAL mode (`PRAGMA journal_mode=WAL`) for concurrent readers; default rollback-journal serialises everything.
2. `PRAGMA foreign_keys=ON` per connection (it is off by default); enforce integrity in the engine, not in hope.
3. Parameterize with `?` placeholders; never interpolate values into SQL strings.
4. Migrations are versioned and forward-only (`PRAGMA user_version` or a migrations table); test down-migrations on a copy.
5. `SYNCHRONOUS=FULL` for data you cannot lose; NORMAL only for rebuildable caches, decided per database and recorded.
6. One writer at a time: serialize writes in the app (queue/mutex); `SQLITE_BUSY` is a design signal, not a retry loop.
7. Store UTC text (`YYYY-MM-DDTHH:MM:SSZ`) or integer epoch; never local-time strings.
8. Know the ceiling: single-file, single-writer, ~TB scale. If you need concurrent writers or row-level security, that is postgres.md, decided explicitly.
## Pitfalls
Type affinity (a TEXT column happily stores integers); `AUTOINCREMENT` has costs plain `ROWID` doesn't; WAL files left behind by unclean shutdowns; in-memory DBs vanish between connections unless shared-cache is deliberate; `LIKE` is case-insensitive for ASCII only.
## Versions
Pinned: SQLite 3.54.0 verified on this machine 2026-10-30. Floor: SQLite 3.40.
Rule: first run `sqlite3 --version` and read the project's pin (bundled version / system lib). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: SQLite docs (WAL, pragmas, datatypes, limits).
Date: 2026-10-30
Status: active
See also: mongodb.md (document profile) vs this relational profile — pick per data shape, record the decision.

Canonical source: `plugins/web-development/middleware/languages/sqlite/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/sqlite.md` (reference; do not author).
