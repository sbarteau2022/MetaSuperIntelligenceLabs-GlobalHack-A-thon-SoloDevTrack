-- Nested Hyperbolic KV and multi-model distillation provenance.
-- Additive migration for SurrealDB 3.3.5.
-- Runtime authority remains hypermind-mcp; direct harness DB writes are forbidden.

DEFINE TABLE IF NOT EXISTS geometry_space SCHEMAFULL TYPE NORMAL
  COMMENT 'Named address-space contract. A quotient is opaque until its executable action is admitted.';
DEFINE FIELD IF NOT EXISTS name ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS manifold ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS quotient_symbol ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS quotient_status ON geometry_space TYPE string
  ASSERT $value IN ['opaque', 'specified', 'certified'];
DEFINE FIELD IF NOT EXISTS coordinate_model ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS contract_version ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS contract_digest ON geometry_space TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON geometry_space TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS geometry_space_name_unique ON geometry_space FIELDS name UNIQUE;

-- Bind existing immutable address revisions to a named space without changing
-- the Lorentz or torus coordinate representation from migration 0001.
DEFINE FIELD IF NOT EXISTS space ON atom_address TYPE option<record<geometry_space>>;

DEFINE TABLE IF NOT EXISTS model_source SCHEMAFULL TYPE NORMAL
  COMMENT 'Exact contributing model identity admitted to a distillation run.';
DEFINE FIELD IF NOT EXISTS workspace ON model_source TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS provider ON model_source TYPE string;
DEFINE FIELD IF NOT EXISTS model ON model_source TYPE string;
DEFINE FIELD IF NOT EXISTS revision ON model_source TYPE string;
DEFINE FIELD IF NOT EXISTS modality ON model_source TYPE string
  ASSERT $value IN ['text', 'image', 'audio', 'video', 'code', 'multimodal'];
DEFINE FIELD IF NOT EXISTS artifact_digest ON model_source TYPE string;
DEFINE FIELD IF NOT EXISTS admitted_at ON model_source TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS model_source_identity_unique ON model_source
  FIELDS workspace, provider, model, revision UNIQUE;

DEFINE TABLE IF NOT EXISTS distillation_run SCHEMAFULL TYPE NORMAL
  COMMENT 'Receipted multi-model-to-HyperMind distillation event.';
DEFINE FIELD IF NOT EXISTS workspace ON distillation_run TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS sources ON distillation_run TYPE array<record<model_source>>;
DEFINE FIELD IF NOT EXISTS method ON distillation_run TYPE string;
DEFINE FIELD IF NOT EXISTS configuration_digest ON distillation_run TYPE string;
DEFINE FIELD IF NOT EXISTS input_digest ON distillation_run TYPE string;
DEFINE FIELD IF NOT EXISTS output_digest ON distillation_run TYPE string;
DEFINE FIELD IF NOT EXISTS receipt_digest ON distillation_run TYPE string;
DEFINE FIELD IF NOT EXISTS status ON distillation_run TYPE string
  ASSERT $value IN ['running', 'completed', 'failed', 'rejected'];
DEFINE FIELD IF NOT EXISTS started_at ON distillation_run TYPE datetime;
DEFINE FIELD IF NOT EXISTS completed_at ON distillation_run TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS distillation_run_receipt_unique ON distillation_run
  FIELDS workspace, receipt_digest UNIQUE;
DEFINE INDEX IF NOT EXISTS distillation_run_status ON distillation_run
  FIELDS workspace, status, started_at;

-- Atoms may cite the exact run that admitted them. Optional preserves all
-- migrated rows until their provenance is backfilled and verified.
DEFINE FIELD IF NOT EXISTS distillation_run ON atom TYPE option<record<distillation_run>>;

DEFINE TABLE IF NOT EXISTS kv_namespace SCHEMAFULL TYPE NORMAL
  COMMENT 'Tenant-local root of a nested Hyperbolic KV tree.';
DEFINE FIELD IF NOT EXISTS workspace ON kv_namespace TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS name ON kv_namespace TYPE string;
DEFINE FIELD IF NOT EXISTS description ON kv_namespace TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON kv_namespace TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS kv_namespace_name_unique ON kv_namespace
  FIELDS workspace, name UNIQUE;

DEFINE TABLE IF NOT EXISTS kv_key SCHEMAFULL TYPE NORMAL
  COMMENT 'Canonical nested key. Parent and path indexes provide hierarchical lookup.';
DEFINE FIELD IF NOT EXISTS workspace ON kv_key TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS namespace ON kv_key TYPE record<kv_namespace>;
DEFINE FIELD IF NOT EXISTS canonical_path ON kv_key TYPE string;
DEFINE FIELD IF NOT EXISTS segments ON kv_key TYPE array<string>;
DEFINE FIELD IF NOT EXISTS depth ON kv_key TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS parent ON kv_key TYPE option<record<kv_key>>;
DEFINE FIELD IF NOT EXISTS current_revision ON kv_key TYPE option<record<kv_value_revision>>;
DEFINE FIELD IF NOT EXISTS status ON kv_key TYPE string
  ASSERT $value IN ['active', 'stale', 'invalidated'];
DEFINE FIELD IF NOT EXISTS created_at ON kv_key TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON kv_key TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS kv_key_path_unique ON kv_key
  FIELDS workspace, namespace, canonical_path UNIQUE;
DEFINE INDEX IF NOT EXISTS kv_key_parent ON kv_key
  FIELDS workspace, namespace, parent, status;

DEFINE TABLE IF NOT EXISTS kv_value_revision SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable-by-runtime value revision with provenance, atom anchors, and receipt.';
DEFINE FIELD IF NOT EXISTS workspace ON kv_value_revision TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS key ON kv_value_revision TYPE record<kv_key>;
DEFINE FIELD IF NOT EXISTS revision ON kv_value_revision TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS value_kind ON kv_value_revision TYPE string
  ASSERT $value IN ['text', 'content_uri', 'atom_set', 'binary_digest'];
DEFINE FIELD IF NOT EXISTS value_text ON kv_value_revision TYPE option<string>;
DEFINE FIELD IF NOT EXISTS value_uri ON kv_value_revision TYPE option<string>;
DEFINE FIELD IF NOT EXISTS value_digest ON kv_value_revision TYPE string;
DEFINE FIELD IF NOT EXISTS produced_by ON kv_value_revision TYPE array<record<atom>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS distillation_run ON kv_value_revision TYPE option<record<distillation_run>>;
DEFINE FIELD IF NOT EXISTS source_digest ON kv_value_revision TYPE string;
DEFINE FIELD IF NOT EXISTS receipt_digest ON kv_value_revision TYPE string;
DEFINE FIELD IF NOT EXISTS stored_epoch ON kv_value_revision TYPE int ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS stale ON kv_value_revision TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS created_at ON kv_value_revision TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS kv_value_revision_unique ON kv_value_revision
  FIELDS workspace, key, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS kv_value_digest ON kv_value_revision
  FIELDS workspace, value_digest;
DEFINE INDEX IF NOT EXISTS kv_value_receipt_unique ON kv_value_revision
  FIELDS workspace, receipt_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS kv_depends_on SCHEMAFULL TYPE RELATION IN kv_key OUT kv_key ENFORCED
  COMMENT 'Directed derivation dependency. Invalidation walks reverse adjacency.';
DEFINE FIELD IF NOT EXISTS workspace ON kv_depends_on TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS revision ON kv_depends_on TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS receipt_digest ON kv_depends_on TYPE string;
DEFINE FIELD IF NOT EXISTS active ON kv_depends_on TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON kv_depends_on TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS kv_depends_on_unique ON kv_depends_on
  FIELDS workspace, in, out, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS kv_depends_on_forward ON kv_depends_on
  FIELDS workspace, in, active;
DEFINE INDEX IF NOT EXISTS kv_depends_on_reverse ON kv_depends_on
  FIELDS workspace, out, active;

DEFINE TABLE IF NOT EXISTS kv_grounded_in SCHEMAFULL TYPE RELATION IN kv_value_revision OUT atom ENFORCED
  COMMENT 'Evidence edge from an immutable KV value revision to a HyperMind atom.';
DEFINE FIELD IF NOT EXISTS workspace ON kv_grounded_in TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS evidence_digest ON kv_grounded_in TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON kv_grounded_in TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS kv_grounded_in_unique ON kv_grounded_in
  FIELDS workspace, in, out UNIQUE;
DEFINE INDEX IF NOT EXISTS kv_grounded_in_revision ON kv_grounded_in
  FIELDS workspace, in;
DEFINE INDEX IF NOT EXISTS kv_grounded_in_atom ON kv_grounded_in
  FIELDS workspace, out;

-- Seed only stable roots. The quotient remains explicitly opaque.
UPSERT geometry_space:h3_t2_r12_v1 CONTENT {
  name: 'H3xT2/r_12',
  manifold: 'H3xT2',
  quotient_symbol: 'r_12',
  quotient_status: 'opaque',
  coordinate_model: 'lorentz4-plus-torus2',
  contract_version: 'v1',
  contract_digest: 'UNSPECIFIED-R12-ACTION',
  created_at: time::now()
};

UPDATE atom_address
  SET space = geometry_space:h3_t2_r12_v1
  WHERE space = NONE
  RETURN NONE;

UPSERT kv_namespace:hypermind CONTENT {
  workspace: workspace:hypermind,
  name: 'hypermind',
  description: 'Nested KV inside the distilled multi-model HyperMind graph',
  created_at: time::now()
};
