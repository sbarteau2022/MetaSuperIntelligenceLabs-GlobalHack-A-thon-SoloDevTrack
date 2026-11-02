-- Native, append-only ingress receipts for fresh GlobalMind distillation.
-- The run envelope never changes state; ordered events carry lifecycle state.

DEFINE FIELD OVERWRITE workspace ON model_source TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE provider ON model_source TYPE string READONLY;
DEFINE FIELD OVERWRITE model ON model_source TYPE string READONLY;
DEFINE FIELD OVERWRITE revision ON model_source TYPE string READONLY;
DEFINE FIELD OVERWRITE modality ON model_source TYPE string
  ASSERT $value IN ['text', 'image', 'audio', 'video', 'code', 'multimodal'] READONLY;
DEFINE FIELD OVERWRITE artifact_digest ON model_source TYPE string READONLY;
DEFINE FIELD OVERWRITE admitted_at ON model_source TYPE datetime DEFAULT time::now() READONLY;

DEFINE TABLE IF NOT EXISTS model_artifact_receipt SCHEMAFULL TYPE NORMAL
  COMMENT 'One rehashed frozen artifact belonging to an immutable model roster.';
DEFINE FIELD IF NOT EXISTS workspace ON model_artifact_receipt TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS model_source ON model_artifact_receipt TYPE record<model_source> READONLY;
DEFINE FIELD IF NOT EXISTS role ON model_artifact_receipt TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS absolute_path ON model_artifact_receipt TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS sha256 ON model_artifact_receipt TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS byte_count ON model_artifact_receipt TYPE int ASSERT $value > 0 READONLY;
DEFINE FIELD IF NOT EXISTS verified_at ON model_artifact_receipt TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS model_artifact_receipt_unique ON model_artifact_receipt
  FIELDS model_source, role, absolute_path UNIQUE;

DEFINE FIELD OVERWRITE workspace ON distillation_run TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE sources ON distillation_run TYPE array<record<model_source>> READONLY;
DEFINE FIELD OVERWRITE method ON distillation_run TYPE string READONLY;
DEFINE FIELD OVERWRITE configuration_digest ON distillation_run TYPE string READONLY;
DEFINE FIELD OVERWRITE input_digest ON distillation_run TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS tpe_version ON distillation_run TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS tpe_configuration_digest ON distillation_run TYPE string READONLY;
DEFINE FIELD OVERWRITE output_digest ON distillation_run TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE receipt_digest ON distillation_run TYPE string READONLY;
DEFINE FIELD OVERWRITE status ON distillation_run TYPE string
  ASSERT $value IN ['opened', 'running', 'completed', 'failed', 'rejected'] READONLY;
DEFINE FIELD OVERWRITE started_at ON distillation_run TYPE datetime READONLY;
DEFINE FIELD OVERWRITE completed_at ON distillation_run TYPE option<datetime> READONLY;

DEFINE TABLE IF NOT EXISTS distillation_run_event SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable ordered lifecycle event; completion never rewrites the run envelope.';
DEFINE FIELD IF NOT EXISTS workspace ON distillation_run_event TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS run ON distillation_run_event TYPE record<distillation_run> READONLY;
DEFINE FIELD IF NOT EXISTS ordinal ON distillation_run_event TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS phase ON distillation_run_event TYPE string
  ASSERT $value IN ['opened', 'extracting', 'judging', 'replaying', 'completed', 'failed', 'rejected'] READONLY;
DEFINE FIELD IF NOT EXISTS input_digest ON distillation_run_event TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS tpe_version ON distillation_run_event TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS tpe_configuration_digest ON distillation_run_event TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS output_digest ON distillation_run_event TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS receipt_digest ON distillation_run_event TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS recorded_at ON distillation_run_event TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS distillation_run_event_ordinal_unique ON distillation_run_event
  FIELDS run, ordinal UNIQUE;
DEFINE INDEX IF NOT EXISTS distillation_run_event_receipt_unique ON distillation_run_event
  FIELDS run, receipt_digest UNIQUE;

-- A replayable trajectory is TPPE-bound. The receipts prove expanding-radius
-- spiral behavior and boundary safety; the database does not infer them.
DEFINE FIELD IF NOT EXISTS tpe_configuration_digest ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS tpe_radial_law_digest ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS tpe_boundary_receipt_digest ON distilled_trajectory TYPE option<string> READONLY;
