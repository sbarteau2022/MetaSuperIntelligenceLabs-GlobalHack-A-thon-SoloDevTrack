-- Step 5: canonical text embedding for distilled-mind atoms.
-- Encoder: BGE-M3 (BAAI/bge-m3), revision pinned in docs/steps-5/.
-- Dimension: 1024 (BGE-M3 dense output; NOT the 250k sparse/colbert heads).
-- Index: HNSW cosine (SurrealDB 3.3.5 native; MTREE is obsolete/irrelevant).
-- Authority: exact Rust H3xT2 rerank + graph traversal. This table is a
-- candidate source only. Teacher/model identity lives in provenance fields
-- on atom/atom_edge — NEVER in the vector path (no teacher key, rank, gate).
-- Image/audio encoders are separately admitted later as sibling tables with
-- their own HNSW indexes; they do not block or alter this table.

DEFINE TABLE IF NOT EXISTS atom_embedding_text_v1 SCHEMAFULL TYPE NORMAL
  COMMENT 'Canonical 1024D BGE-M3 text vector per atom; candidate source only, Rust H3xT2 rerank is authoritative.';

DEFINE FIELD IF NOT EXISTS workspace ON atom_embedding_text_v1 TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS atom ON atom_embedding_text_v1 TYPE record<atom>;
DEFINE FIELD IF NOT EXISTS encoder ON atom_embedding_text_v1 TYPE string
  ASSERT $value = 'bge-m3';
DEFINE FIELD IF NOT EXISTS encoder_revision ON atom_embedding_text_v1 TYPE string
  ASSERT $value = '5617a9f61b028005a4858fdac845db406aefb181';
DEFINE FIELD IF NOT EXISTS dimensions ON atom_embedding_text_v1 TYPE int
  ASSERT $value = 1024;
DEFINE FIELD IF NOT EXISTS vector ON atom_embedding_text_v1 TYPE array<float, 1024>;
DEFINE FIELD IF NOT EXISTS normalized ON atom_embedding_text_v1 TYPE bool;
DEFINE FIELD IF NOT EXISTS source_digest ON atom_embedding_text_v1 TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON atom_embedding_text_v1 TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS atom_embedding_text_v1_atom_unique ON atom_embedding_text_v1
  FIELDS workspace, atom UNIQUE;

DEFINE INDEX IF NOT EXISTS atom_embedding_text_v1_hnsw ON atom_embedding_text_v1
  FIELDS vector HNSW DIMENSION 1024 DIST COSINE;

DEFINE TABLE IF NOT EXISTS schema_migration SCHEMAFULL TYPE NORMAL
  COMMENT 'Applied schema version and source digest.';
DEFINE FIELD IF NOT EXISTS version ON schema_migration TYPE string;
DEFINE FIELD IF NOT EXISTS digest ON schema_migration TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON schema_migration TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS schema_migration_version_unique ON schema_migration FIELDS version UNIQUE;
