-- Additive contract: shared atoms/trajectories are teacher invariant.
-- Model lineage is audit-only and never a semantic retrieval or rank key.
-- All admitted records are append-only; a correction gets a new record id.

DEFINE TABLE IF NOT EXISTS distillation_admission SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable, judge-passed source artifact admission for a teacher.';
DEFINE FIELD IF NOT EXISTS workspace ON distillation_admission TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS model_source ON distillation_admission TYPE record<model_source> READONLY;
DEFINE FIELD IF NOT EXISTS source_artifact_digest ON distillation_admission TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS source_artifact_uri ON distillation_admission TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS roster_receipt_digest ON distillation_admission TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS judge_receipt_digest ON distillation_admission TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS eligibility_receipt_digest ON distillation_admission TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS admitted_at ON distillation_admission TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS distillation_admission_unique ON distillation_admission
  FIELDS workspace, model_source, source_artifact_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS atom_origin SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable audit lineage; never joined into recall or rank.';
DEFINE FIELD IF NOT EXISTS workspace ON atom_origin TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS atom ON atom_origin TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON atom_origin TYPE record<distillation_admission> READONLY;
DEFINE FIELD IF NOT EXISTS source_step_digest ON atom_origin TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS mint_receipt_digest ON atom_origin TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS recorded_at ON atom_origin TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS atom_origin_unique ON atom_origin
  FIELDS workspace, atom, admission, source_step_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS distilled_trajectory SCHEMAFULL TYPE NORMAL
  COMMENT 'Teacher-neutral ordered atom walk; links shared atoms only.';
DEFINE FIELD IF NOT EXISTS workspace ON distilled_trajectory TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS atoms ON distilled_trajectory TYPE array<record<atom>> READONLY;
DEFINE FIELD IF NOT EXISTS trajectory_digest ON distilled_trajectory TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS judge_receipt_digest ON distilled_trajectory TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS minted_at ON distilled_trajectory TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS distilled_trajectory_unique ON distilled_trajectory
  FIELDS workspace, trajectory_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS trajectory_origin SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable audit join between teacher-neutral walk and admitted source.';
DEFINE FIELD IF NOT EXISTS workspace ON trajectory_origin TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS trajectory ON trajectory_origin TYPE record<distilled_trajectory> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON trajectory_origin TYPE record<distillation_admission> READONLY;
DEFINE FIELD IF NOT EXISTS source_walk_digest ON trajectory_origin TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS receipt_digest ON trajectory_origin TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS recorded_at ON trajectory_origin TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS trajectory_origin_unique ON trajectory_origin
  FIELDS workspace, trajectory, admission, source_walk_digest UNIQUE;

-- Preserve migration history across later versions. Corrections append a new
-- version row; no version may silently rewrite an earlier version or digest.
DEFINE FIELD OVERWRITE version ON schema_migration TYPE string READONLY;
DEFINE FIELD OVERWRITE digest ON schema_migration TYPE string READONLY;
DEFINE FIELD OVERWRITE applied_at ON schema_migration TYPE datetime DEFAULT time::now() READONLY;
