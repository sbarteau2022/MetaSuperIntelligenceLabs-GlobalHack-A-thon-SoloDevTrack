-- N3-T&O hunger ledger — schema mirrors TAP-N-TRACE.md section 4 exactly.
-- One row per distinct gap. Hits count repeats. Nothing deleted ever.
CREATE TABLE IF NOT EXISTS hunger (
  gap_key     TEXT PRIMARY KEY,   -- deterministic from tap (see TAP-N-TRACE section 3)
  tap         TEXT NOT NULL,      -- which tap minted it
  terms_hash  TEXT NOT NULL,      -- sha256 of sorted query terms (H-3)
  atom_ids    TEXT NOT NULL,      -- JSON array of involved atom ids
  hits        INTEGER NOT NULL DEFAULT 1,
  depth       INTEGER NOT NULL DEFAULT 1,
  first_seen  INTEGER NOT NULL,   -- unix seconds
  last_seen   INTEGER NOT NULL,
  sated       INTEGER NOT NULL DEFAULT 0,  -- 1 when a later distill covers it
  sated_by    TEXT                -- distill receipt that sated it
);
CREATE INDEX IF NOT EXISTS hunger_priority ON hunger (sated, hits DESC, last_seen DESC);
