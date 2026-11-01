-- HyperMind canonical graph schema for SurrealDB 3.3.5.
-- JSONL migration is intentionally outside this migration.

DEFINE TABLE IF NOT EXISTS workspace SCHEMAFULL TYPE NORMAL
  COMMENT 'Tenant boundary for every HyperMind record.';
DEFINE FIELD IF NOT EXISTS slug ON workspace TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON workspace TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS workspace_slug_unique ON workspace FIELDS slug UNIQUE;

DEFINE TABLE IF NOT EXISTS atom SCHEMAFULL TYPE NORMAL
  COMMENT 'Stable graph node; addresses and embeddings are separately versioned.';
DEFINE FIELD IF NOT EXISTS workspace ON atom TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS external_id ON atom TYPE string;
DEFINE FIELD IF NOT EXISTS content_kind ON atom TYPE string
  ASSERT $value IN ['text', 'image', 'audio', 'video', 'code', 'mixed'];
DEFINE FIELD IF NOT EXISTS content_text ON atom TYPE option<string>;
DEFINE FIELD IF NOT EXISTS content_uri ON atom TYPE option<string>;
DEFINE FIELD IF NOT EXISTS content_digest ON atom TYPE string;
DEFINE FIELD IF NOT EXISTS domain ON atom TYPE string;
DEFINE FIELD IF NOT EXISTS rank ON atom TYPE int ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS salience ON atom TYPE float ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD IF NOT EXISTS provenance ON atom TYPE string;
DEFINE FIELD IF NOT EXISTS taint ON atom TYPE string;
DEFINE FIELD IF NOT EXISTS pinned ON atom TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS current_address ON atom TYPE option<record<atom_address>>;
DEFINE FIELD IF NOT EXISTS created_at ON atom TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON atom TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS atom_external_unique ON atom FIELDS workspace, external_id UNIQUE;
DEFINE INDEX IF NOT EXISTS atom_digest ON atom FIELDS workspace, content_digest;
DEFINE INDEX IF NOT EXISTS atom_domain ON atom FIELDS workspace, domain;

DEFINE TABLE IF NOT EXISTS atom_address SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable versioned intrinsic H3 x T2 address.';
DEFINE FIELD IF NOT EXISTS workspace ON atom_address TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS atom ON atom_address TYPE record<atom>;
DEFINE FIELD IF NOT EXISTS h3_lorentz ON atom_address TYPE array<float, 4>;
DEFINE FIELD IF NOT EXISTS t2_phase ON atom_address TYPE array<float, 2>;
DEFINE FIELD IF NOT EXISTS geometry_model ON atom_address TYPE string;
DEFINE FIELD IF NOT EXISTS geometry_version ON atom_address TYPE string;
DEFINE FIELD IF NOT EXISTS revision ON atom_address TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS active ON atom_address TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON atom_address TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS atom_address_revision_unique ON atom_address
  FIELDS workspace, atom, geometry_version, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS atom_address_active ON atom_address FIELDS workspace, atom, active;

DEFINE TABLE IF NOT EXISTS atom_embedding SCHEMAFULL TYPE NORMAL
  COMMENT 'Versioned modality embedding; ANN indexes are admitted per fixed model dimension.';
DEFINE FIELD IF NOT EXISTS workspace ON atom_embedding TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS atom ON atom_embedding TYPE record<atom>;
DEFINE FIELD IF NOT EXISTS modality ON atom_embedding TYPE string
  ASSERT $value IN ['text', 'image', 'audio', 'video', 'code', 'joint'];
DEFINE FIELD IF NOT EXISTS model ON atom_embedding TYPE string;
DEFINE FIELD IF NOT EXISTS model_revision ON atom_embedding TYPE string;
DEFINE FIELD IF NOT EXISTS dimensions ON atom_embedding TYPE int ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS vector ON atom_embedding TYPE array<float>;
DEFINE FIELD IF NOT EXISTS normalized ON atom_embedding TYPE bool;
DEFINE FIELD IF NOT EXISTS source_digest ON atom_embedding TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON atom_embedding TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS atom_embedding_version_unique ON atom_embedding
  FIELDS workspace, atom, modality, model, model_revision UNIQUE;
DEFINE INDEX IF NOT EXISTS atom_embedding_lookup ON atom_embedding
  FIELDS workspace, modality, model, model_revision;

DEFINE TABLE IF NOT EXISTS atom_edge SCHEMAFULL TYPE RELATION IN atom OUT atom ENFORCED
  COMMENT 'Directed typed evidenced atom relation; native adjacency is the traversal substrate.';
DEFINE FIELD IF NOT EXISTS workspace ON atom_edge TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS kind ON atom_edge TYPE string
  ASSERT $value IN ['semantic', 'causal', 'temporal', 'supports', 'contradicts', 'part_of', 'derived_from', 'references', 'next'];
DEFINE FIELD IF NOT EXISTS weight ON atom_edge TYPE float ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD IF NOT EXISTS confidence ON atom_edge TYPE float ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD IF NOT EXISTS evidence_atoms ON atom_edge TYPE array<record<atom>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS evidence_digest ON atom_edge TYPE string;
DEFINE FIELD IF NOT EXISTS producer ON atom_edge TYPE string;
DEFINE FIELD IF NOT EXISTS revision ON atom_edge TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS active ON atom_edge TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS valid_from ON atom_edge TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS valid_to ON atom_edge TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON atom_edge TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS atom_edge_revision_unique ON atom_edge
  FIELDS workspace, in, out, kind, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS atom_edge_outgoing ON atom_edge FIELDS workspace, in, active, kind;
DEFINE INDEX IF NOT EXISTS atom_edge_incoming ON atom_edge FIELDS workspace, out, active, kind;

DEFINE TABLE IF NOT EXISTS schema_migration SCHEMAFULL TYPE NORMAL
  COMMENT 'Applied schema version and source digest.';
DEFINE FIELD IF NOT EXISTS version ON schema_migration TYPE string;
DEFINE FIELD IF NOT EXISTS digest ON schema_migration TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON schema_migration TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS schema_migration_version_unique ON schema_migration FIELDS version UNIQUE;
