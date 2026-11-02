-- Model Home cognitive map — Slice 1 schema
-- Postgres 17, schema `cogmap`. Policy: atlas-evidence-v1.1 (Blueprint v1.1 §§A-I,K).
-- Design rule: deterministic engine owns state; model proposes, never certifies (I08).
-- Apply: psql "postgres://127.0.0.1:5433/<db>" -f schema.sql   (Stewart runs; see README)
BEGIN;
CREATE SCHEMA IF NOT EXISTS cogmap;

-- ---------- enrollments: S = (V,P,A,G,M,U,X), pinned versions (§B) ----------
CREATE TABLE IF NOT EXISTS cogmap.enrollments (
  id                 uuid PRIMARY KEY,
  learner_id         text NOT NULL,
  curriculum_version text NOT NULL,
  curriculum_hash    text NOT NULL,            -- pinned; change requires migration event (§B)
  rubric_hash        text NOT NULL,
  policy_version     text NOT NULL DEFAULT 'atlas-evidence-v1.1',
  revision           bigint NOT NULL DEFAULT 0, -- domain projection version (§B)
  deletion_epoch     bigint NOT NULL DEFAULT 0, -- I10: jobs recheck before read/write
  deleted_at         timestamptz,
  created_at         timestamptz NOT NULL DEFAULT now(),
  UNIQUE (learner_id, curriculum_hash, policy_version)
);

-- ---------- events: ordered, append-only source of truth (§§B,H,I) ----------
CREATE TABLE IF NOT EXISTS cogmap.events (
  seq           bigserial PRIMARY KEY,          -- global order; gaps in a filtered stream are normal (§B)
  enrollment_id uuid NOT NULL REFERENCES cogmap.enrollments(id),
  revision      bigint NOT NULL,                -- enrollment revision after this event batch
  command_id    text,                           -- client command identity for retries (I03)
  event_type    text NOT NULL,
  payload       jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS events_enrollment_seq ON cogmap.events(enrollment_id, seq);

-- ---------- receipts: I03 idempotency ----------
-- Retrying same (enrollment_id, command_id) + same canonical_hash returns same receipt, no state change.
CREATE TABLE IF NOT EXISTS cogmap.receipts (
  enrollment_id  uuid NOT NULL REFERENCES cogmap.enrollments(id),
  command_id     text NOT NULL,
  canonical_hash text NOT NULL,                 -- sha256 of canonical command bytes
  first_seq      bigint NOT NULL,
  last_seq       bigint NOT NULL,
  revision       bigint NOT NULL,
  response       jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at     timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (enrollment_id, command_id)
);

-- ---------- attempts: state machine §C ----------
-- States: assigned -> prediction_committed -> revealed -> assessment_pending -> assessed
--         assessed -> disputed -> (assessed|invalid); any active -> invalid|cancelled
CREATE TABLE IF NOT EXISTS cogmap.attempts (
  id                 uuid PRIMARY KEY,
  enrollment_id      uuid NOT NULL REFERENCES cogmap.enrollments(id),
  task_template_id   text NOT NULL,
  task_variant       text NOT NULL,             -- distinct authored variant (§C,F)
  task_kind          text NOT NULL DEFAULT 'scored' CHECK (task_kind IN ('scored','practice','exploration')),
  concept_id         text NOT NULL,
  dimension          text NOT NULL,             -- Coding-101 dimension (one of nine; manifest-owned)
  difficulty         smallint NOT NULL CHECK (difficulty BETWEEN 0 AND 5),  -- D0-D5
  subtlety           smallint CHECK (subtlety BETWEEN 1 AND 5),            -- S1-S5
  state              text NOT NULL DEFAULT 'assigned' CHECK (state IN
    ('assigned','prediction_committed','revealed','assessment_pending','assessed','disputed','invalid','cancelled')),
  prediction_hash    text,                      -- sealed at commit; reveal requires it (§C)
  prediction_seq     bigint,                    -- seq of prediction commit (promotion gate: precedes reveal)
  reveal_seq         bigint,
  hint_tier          smallint NOT NULL DEFAULT 0 CHECK (hint_tier >= 0),
  independent        boolean NOT NULL DEFAULT false, -- distinct task, core answer not supplied (§D)
  submission_id      uuid,                      -- set once at submit (§C: one scored submission)
  created_at         timestamptz NOT NULL DEFAULT now(),
  UNIQUE (enrollment_id, id)
);

-- ---------- submissions & artifacts: immutable, finalized only (§§C,D,H) ----------
CREATE TABLE IF NOT EXISTS cogmap.submissions (
  id             uuid PRIMARY KEY,
  enrollment_id  uuid NOT NULL REFERENCES cogmap.enrollments(id),
  attempt_id     uuid NOT NULL,
  artifact_hash  text NOT NULL,                 -- sha256 of finalized artifact bytes
  submission_hash text NOT NULL,
  finalized      boolean NOT NULL DEFAULT false, -- only finalized enters a command (§H)
  created_at     timestamptz NOT NULL DEFAULT now(),
  UNIQUE (attempt_id),                          -- §J: one scored submission per attempt
  FOREIGN KEY (enrollment_id, attempt_id) REFERENCES cogmap.attempts(enrollment_id, id)
);

-- ---------- assessment heads: I04 one active head per submission ----------
CREATE TABLE IF NOT EXISTS cogmap.assessments (
  id                      uuid PRIMARY KEY,
  enrollment_id           uuid NOT NULL REFERENCES cogmap.enrollments(id),
  attempt_id              uuid NOT NULL,
  submission_id           uuid NOT NULL REFERENCES cogmap.submissions(id),
  supersedes_id           uuid REFERENCES cogmap.assessments(id), -- regrade: new head, no new attempt (I04)
  is_head                 boolean NOT NULL DEFAULT true,
  active                  boolean NOT NULL DEFAULT true,  -- false when disputed/invalid (§§C,F)
  grader_id               text NOT NULL,          -- deterministic runner id or reviewer id (§D)
  grader_version          text NOT NULL,
  admission_policy_version text NOT NULL DEFAULT 'atlas-evidence-v1.1',
  reviewer_id             text,                   -- required for semantic criteria at launch (§D.5)
  verdict                 smallint NOT NULL CHECK (verdict IN (0,1)),        -- v
  justification           smallint NOT NULL CHECK (justification BETWEEN 0 AND 3), -- j
  evidence_quality_x4     smallint NOT NULL CHECK (evidence_quality_x4 BETWEEN 0 AND 4), -- q = 4*quality
  hint_percent            smallint NOT NULL CHECK (hint_percent IN (100,90,75,55,35)),   -- h (§E)
  raw_units               integer GENERATED ALWAYS AS (96*verdict + 28*justification + 15*evidence_quality_x4) STORED,
  score_units             integer GENERATED ALWAYS AS ((96*verdict + 28*justification + 15*evidence_quality_x4) * hint_percent) STORED, -- 0..24000 (§E)
  false_negative          boolean NOT NULL DEFAULT false,
  false_positive          boolean NOT NULL DEFAULT false,
  is_fully_clean          boolean NOT NULL DEFAULT false, -- for promotion coverage (§F)
  is_defective_task       boolean NOT NULL DEFAULT false,
  artifact_hash           text NOT NULL,
  created_at              timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (enrollment_id, attempt_id) REFERENCES cogmap.attempts(enrollment_id, id)
);
-- I04: at most one head per submission
CREATE UNIQUE INDEX IF NOT EXISTS assessments_one_head_per_submission
  ON cogmap.assessments(submission_id) WHERE is_head;

-- ---------- criteria: per-criterion, per-concept scoring (§§D,F) ----------
-- Concept score ONLY from criteria explicitly mapped to that concept. No dimension-score copying (§F).
CREATE TABLE IF NOT EXISTS cogmap.criteria (
  id              uuid PRIMARY KEY,
  assessment_id   uuid NOT NULL REFERENCES cogmap.assessments(id),
  criterion_id    text NOT NULL,
  concept_id      text,                          -- null = dimension-only evidence
  dimension       text NOT NULL,
  normalized_value integer NOT NULL,
  evidence_refs   jsonb NOT NULL DEFAULT '[]'::jsonb,
  grader_id       text NOT NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  UNIQUE (assessment_id, criterion_id)
);

-- ---------- dimension observations: I05 one per attempt/dimension ----------
CREATE TABLE IF NOT EXISTS cogmap.dimension_observations (
  enrollment_id uuid NOT NULL REFERENCES cogmap.enrollments(id),
  attempt_id    uuid NOT NULL,
  dimension     text NOT NULL,
  difficulty    smallint NOT NULL CHECK (difficulty BETWEEN 0 AND 5),
  assessment_id uuid NOT NULL REFERENCES cogmap.assessments(id),
  submission_seq bigint NOT NULL,                -- ordering for window (§F)
  score_units   integer NOT NULL CHECK (score_units BETWEEN 0 AND 24000),
  independent   boolean NOT NULL DEFAULT false,
  hint_tier     smallint NOT NULL DEFAULT 0,
  is_fully_clean boolean NOT NULL DEFAULT false,
  is_defective_task boolean NOT NULL DEFAULT false,
  false_negative boolean NOT NULL DEFAULT false,
  false_positive boolean NOT NULL DEFAULT false,
  PRIMARY KEY (enrollment_id, attempt_id, dimension)  -- I05 regardless of retries/regrades (§F dedup)
);

-- ---------- windows: materialized last-8 per dimension (§F) ----------
CREATE TABLE IF NOT EXISTS cogmap.windows (
  enrollment_id uuid NOT NULL REFERENCES cogmap.enrollments(id),
  dimension     text NOT NULL,
  difficulty    smallint NOT NULL CHECK (difficulty BETWEEN 0 AND 5),
  n             integer NOT NULL DEFAULT 0,
  sum_units     bigint NOT NULL DEFAULT 0,       -- gate: mean>=.80 iff sum>=19200*n (§E)
  defective_count bigint NOT NULL DEFAULT 0,
  miss_count    bigint NOT NULL DEFAULT 0,       -- FNR<=.20 iff 5*miss<=defective (§E)
  clean_opp_count bigint NOT NULL DEFAULT 0,
  false_alarm_count bigint NOT NULL DEFAULT 0,   -- FPR<=.25 iff 4*fa<=clean_opp (§E)
  attempt_ids   jsonb NOT NULL DEFAULT '[]'::jsonb, -- last 8, seq ascending (§F)
  basis_revision bigint NOT NULL DEFAULT 0,
  updated_at    timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (enrollment_id, dimension, difficulty)
);

-- ---------- decisions: derived, keyed, rechecked on accept (§§B,G) ----------
CREATE TABLE IF NOT EXISTS cogmap.decisions (
  id             uuid PRIMARY KEY,
  enrollment_id  uuid NOT NULL REFERENCES cogmap.enrollments(id),
  basis_revision bigint NOT NULL,
  goal_hash      text NOT NULL,
  policy_version text NOT NULL DEFAULT 'atlas-evidence-v1.1',
  kind           text NOT NULL CHECK (kind IN
    ('hold','explain','ask_evidence','human_review','diagnostic','offer_prereq','offer_advance','next_practice')),
  evidence_refs  jsonb NOT NULL DEFAULT '[]'::jsonb, -- I07: non-unknown claims cite evidence+policy
  payload        jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at     timestamptz NOT NULL DEFAULT now(),
  UNIQUE (enrollment_id, basis_revision, goal_hash, policy_version)
);

-- ---------- placements: offers only, no auto-demotion (§F) ----------
CREATE TABLE IF NOT EXISTS cogmap.placements (
  id            uuid PRIMARY KEY,
  enrollment_id uuid NOT NULL REFERENCES cogmap.enrollments(id),
  dimension     text NOT NULL,
  from_difficulty smallint NOT NULL,
  to_difficulty   smallint NOT NULL,
  decision_id   uuid REFERENCES cogmap.decisions(id),
  status        text NOT NULL DEFAULT 'offered' CHECK (status IN ('offered','accepted','declined','needs_reassessment')),
  created_at    timestamptz NOT NULL DEFAULT now()
);

-- ---------- outbox: leases, retries, dead-letter (§H) ----------
CREATE TABLE IF NOT EXISTS cogmap.outbox (
  id                uuid PRIMARY KEY,
  enrollment_id     uuid NOT NULL REFERENCES cogmap.enrollments(id),
  kind              text NOT NULL,               -- e.g. assessment_job, delivery
  payload           jsonb NOT NULL DEFAULT '{}'::jsonb,
  lease_owner       text,
  lease_until       timestamptz,
  delivery_attempts integer NOT NULL DEFAULT 0,
  next_attempt_at   timestamptz,
  terminal_receipt  jsonb,
  dead_letter       boolean NOT NULL DEFAULT false,
  created_at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS outbox_ready ON cogmap.outbox(next_attempt_at) WHERE terminal_receipt IS NULL AND NOT dead_letter;
COMMIT;
-- Invariants not expressible as constraints (reducer/test-enforced, §K): I01,I02,I06-I12.
-- I07: API layer must reject map claims lacking evidence_refs+policy_version. I08: no client-writable grade path.
