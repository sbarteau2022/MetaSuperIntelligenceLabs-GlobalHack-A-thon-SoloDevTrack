-- Model Home cognitive map — INTEGRATION migration: activation persistence
-- Companion to slice1/schema.sql (apply schema.sql FIRST, then this file).
-- Postgres 17, schema `cogmap`. Policy: atlas-evidence-v1.1 (Blueprint §K gates 4-5).
-- Apply: psql "postgres://home@127.0.0.1:5433/home" -f activation.sql   (Stewart runs)
--
-- Why a new table: slice1 schema persists placements/offers (cogmap.placements)
-- and async jobs (cogmap.outbox), but the Slice 4 Activation state machine
-- (Shadow -> OptedIn -> Active, Disabled = kill switch) has no home there.
-- This migration adds it WITHOUT altering any Slice 1 table.
BEGIN;
CREATE SCHEMA IF NOT EXISTS cogmap;

-- ---------- activation: one row per enrollment (Slice 4 Activation.phase) ----------
CREATE TABLE IF NOT EXISTS cogmap.activation (
  enrollment_id text PRIMARY KEY,               -- matches Activation.enrollment_id (text id space, see README)
  phase         text NOT NULL DEFAULT 'shadow'
                CHECK (phase IN ('shadow','opted_in','active','disabled')),
  updated_at    timestamptz NOT NULL DEFAULT now()
);

-- ---------- activation audit: every transition, append-only (Slice 4 AuditEntry) ----------
CREATE TABLE IF NOT EXISTS cogmap.activation_audit (
  seq           bigserial PRIMARY KEY,
  enrollment_id text NOT NULL REFERENCES cogmap.activation(enrollment_id),
  actor         text NOT NULL,                  -- human actor only for transitions (I08)
  action        text NOT NULL,                  -- opt_in | activate | disable | placement_offered | placement_blocked
  phase_after   text NOT NULL CHECK (phase_after IN ('shadow','opted_in','active','disabled')),
  created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS activation_audit_enrollment_seq
  ON cogmap.activation_audit(enrollment_id, seq);
COMMIT;
-- Fence kept: phase transitions are human commands; nothing in this file (or the
-- tools layer) lets a model actor write 'active'. Kill switch = phase 'disabled'.
