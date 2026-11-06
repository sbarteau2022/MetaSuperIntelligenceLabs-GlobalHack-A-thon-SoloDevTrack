-- Nested hyperbolic KV tensor plane (SurrealDB 3.3.5).
-- Namespace hypermind, database core. These three tables are the tensor
-- retrieval plane. They link to atom / atom_address and are not embeddings.
-- geometry_space h3_t2_r12_v1 is an engineering label.
-- quotient_status stays opaque. contract_digest stays UNSPECIFIED-R12-ACTION.
-- H3 x T2 / r_12 is not certified here.

DEFINE TABLE IF NOT EXISTS transformer_kv_session SCHEMAFULL TYPE NORMAL
  COMMENT 'One model + TPPE session. r_12 quotient is opaque.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_session TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS geometry_space ON transformer_kv_session TYPE string
  ASSERT $value = 'h3_t2_r12_v1';
DEFINE FIELD IF NOT EXISTS quotient_status ON transformer_kv_session TYPE string
  ASSERT $value = 'opaque';
DEFINE FIELD IF NOT EXISTS contract_digest ON transformer_kv_session TYPE string
  ASSERT $value = 'UNSPECIFIED-R12-ACTION';
DEFINE FIELD IF NOT EXISTS model_digest ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS toroidal_phase_encoder_version ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_session TYPE datetime DEFAULT time::now();

DEFINE TABLE IF NOT EXISTS transformer_kv_page SCHEMAFULL TYPE NORMAL
  COMMENT 'Binary K/V page. key_tensor and value_tensor are bytes, never text or JSON. Separate from semantic atom recall.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_page TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS session ON transformer_kv_page TYPE record<transformer_kv_session>;
DEFINE FIELD IF NOT EXISTS atom ON transformer_kv_page TYPE record<atom>;
DEFINE FIELD IF NOT EXISTS address ON transformer_kv_page TYPE record<atom_address>;
DEFINE FIELD IF NOT EXISTS model_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS dtype ON transformer_kv_page TYPE string
  ASSERT $value IN ['f32', 'bf16'];
DEFINE FIELD IF NOT EXISTS layout ON transformer_kv_page TYPE string
  ASSERT $value = 'row-major';
DEFINE FIELD IF NOT EXISTS shape ON transformer_kv_page TYPE array<int>;
DEFINE FIELD IF NOT EXISTS toroidal_phase_encoder_version ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS address_contract_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS key_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS value_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS geometry_space ON transformer_kv_page TYPE string
  ASSERT $value = 'h3_t2_r12_v1';
DEFINE FIELD IF NOT EXISTS quotient_status ON transformer_kv_page TYPE string
  ASSERT $value = 'opaque';
DEFINE FIELD IF NOT EXISTS contract_digest ON transformer_kv_page TYPE string
  ASSERT $value = 'UNSPECIFIED-R12-ACTION';
DEFINE FIELD IF NOT EXISTS key_tensor ON transformer_kv_page TYPE bytes;
DEFINE FIELD IF NOT EXISTS value_tensor ON transformer_kv_page TYPE bytes;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_page TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS transformer_kv_page_atom ON transformer_kv_page FIELDS atom;

DEFINE TABLE IF NOT EXISTS transformer_kv_next SCHEMAFULL TYPE NORMAL
  COMMENT 'Ordered link between tensor pages inside one session. Not a semantic edge.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_next TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS session ON transformer_kv_next TYPE record<transformer_kv_session>;
DEFINE FIELD IF NOT EXISTS from_page ON transformer_kv_next TYPE record<transformer_kv_page>;
DEFINE FIELD IF NOT EXISTS to_page ON transformer_kv_next TYPE record<transformer_kv_page>;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_next TYPE datetime DEFAULT time::now();
