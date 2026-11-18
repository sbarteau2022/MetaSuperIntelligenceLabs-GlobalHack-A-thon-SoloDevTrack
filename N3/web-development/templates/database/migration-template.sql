-- migration-template.sql — forward-only numbered migration skeleton.
-- Status: PROPOSED. Date: 2026-10-30.
-- RULES.md rules 45/46/48: versioned, forward-only, dual-compat (safe while
-- old code still serves), rollback documented. Naming: NNNN_slug.sql.
-- Replace <table>/<column> placeholders in the WORKED EXAMPLE only; keep the
-- skeleton additive (ADD COLUMN nullable / WITH DEFAULT-safe / index).

-- ============ UP (forward, additive, dual-compat) ============
-- Rule: additive changes only here. New columns MUST be NULLABLE (or carry a
-- safe DEFAULT) so rows written by the OLD code still insert cleanly. Ship
-- code that reads the new column only AFTER this migration is deployed
-- everywhere; removal of the old shape is a LATER migration, never this one.
-- Index note: on large tables prefer CONCURRENTLY-style (non-blocking) index
-- builds; plain CREATE INDEX locks writes. Adjust per engine below.

-- ---- WORKED EXAMPLE: add nullable column + index ----
-- ALTER TABLE widgets ADD COLUMN IF NOT EXISTS subtitle TEXT NULL;
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_widgets_subtitle ON widgets (subtitle);

-- ============ DOWN (rollback) ============
-- Rule: every UP has a tested DOWN. Rollback drops only what UP added and is
-- itself safe while either code version serves (dropping a nullable,
-- unread-by-old-code column qualifies; dropping a column old code writes
-- does NOT — that needs the two-step removal migration instead).

-- ---- WORKED EXAMPLE ROLLBACK ----
-- DROP INDEX IF EXISTS idx_widgets_subtitle;
-- ALTER TABLE widgets DROP COLUMN IF EXISTS subtitle;
