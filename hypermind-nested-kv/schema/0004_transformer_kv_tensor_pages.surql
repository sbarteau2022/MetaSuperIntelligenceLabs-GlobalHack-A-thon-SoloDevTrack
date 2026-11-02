-- Replace the empty legacy string KV schema with typed transformer K/V pages.
-- Tensor payloads are bytes, never text or JSON.

REMOVE TABLE IF EXISTS kv_grounded_in;
REMOVE TABLE IF EXISTS kv_depends_on;
REMOVE TABLE IF EXISTS kv_value_revision;
REMOVE TABLE IF EXISTS kv_key;
REMOVE TABLE IF EXISTS kv_namespace;

DEFINE TABLE IF NOT EXISTS transformer_kv_session SCHEMAFULL TYPE NORMAL
  COMMENT 'One model/request forward-pass KV cache identity.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_session TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS model_id ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS model_revision ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS model_digest ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS request_digest ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS dtype ON transformer_kv_session TYPE string
  ASSERT $value IN ['f16', 'bf16', 'f32', 'q8_0', 'q4_0'];
DEFINE FIELD IF NOT EXISTS layer_count ON transformer_kv_session TYPE int
  ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS kv_head_count ON transformer_kv_session TYPE int
  ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS head_dim ON transformer_kv_session TYPE int
  ASSERT $value > 0;
-- Positional identity is the native H3 x T2 contract: H3 address plus the
-- Toroidal Phase Positional Encoder. RoPE/Euclidean position is inadmissible.
REMOVE FIELD IF EXISTS position_encoding ON transformer_kv_session;
DEFINE FIELD IF NOT EXISTS geometry_space ON transformer_kv_session TYPE record<geometry_space>;
DEFINE FIELD IF NOT EXISTS toroidal_phase_encoder_version ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS address_contract_digest ON transformer_kv_session TYPE string;
DEFINE FIELD IF NOT EXISTS active ON transformer_kv_session TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_session TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS closed_at ON transformer_kv_session TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS transformer_kv_session_identity ON transformer_kv_session
  FIELDS workspace, model_digest, request_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS transformer_kv_page SCHEMAFULL TYPE NORMAL
  COMMENT 'Binary K and V tensor page for one layer, KV head, and token interval.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_page TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS session ON transformer_kv_page TYPE record<transformer_kv_session>;
DEFINE FIELD IF NOT EXISTS layer ON transformer_kv_page TYPE int ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS kv_head ON transformer_kv_page TYPE int ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS token_start ON transformer_kv_page TYPE int ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS token_end ON transformer_kv_page TYPE int ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS tensor_shape ON transformer_kv_page TYPE array<int, 2>;
DEFINE FIELD IF NOT EXISTS dtype ON transformer_kv_page TYPE string
  ASSERT $value IN ['f16', 'bf16', 'f32', 'q8_0', 'q4_0'];
DEFINE FIELD IF NOT EXISTS layout ON transformer_kv_page TYPE string
  ASSERT $value IN ['token_major', 'head_dim_major'];
DEFINE FIELD IF NOT EXISTS key_tensor ON transformer_kv_page TYPE bytes;
DEFINE FIELD IF NOT EXISTS value_tensor ON transformer_kv_page TYPE bytes;
DEFINE FIELD IF NOT EXISTS key_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS value_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS key_bytes ON transformer_kv_page TYPE int ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS value_bytes ON transformer_kv_page TYPE int ASSERT $value > 0;
DEFINE FIELD IF NOT EXISTS address ON transformer_kv_page TYPE record<atom_address>;
DEFINE FIELD IF NOT EXISTS trajectory_atoms ON transformer_kv_page TYPE array<record<atom>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS receipt_digest ON transformer_kv_page TYPE string;
DEFINE FIELD IF NOT EXISTS revision ON transformer_kv_page TYPE int ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS hits ON transformer_kv_page TYPE int DEFAULT 0 ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS active ON transformer_kv_page TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_page TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS last_accessed_at ON transformer_kv_page TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_identity ON transformer_kv_page
  FIELDS workspace, session, layer, kv_head, token_start, token_end, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_lookup ON transformer_kv_page
  FIELDS workspace, session, layer, kv_head, active, token_start;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_address ON transformer_kv_page
  FIELDS workspace, address, active;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_receipt ON transformer_kv_page
  FIELDS workspace, receipt_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS transformer_kv_next SCHEMAFULL
  TYPE RELATION IN transformer_kv_page OUT transformer_kv_page ENFORCED
  COMMENT 'Causal token-page trajectory for forward-pass continuation.';
DEFINE FIELD IF NOT EXISTS workspace ON transformer_kv_next TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS receipt_digest ON transformer_kv_next TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON transformer_kv_next TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS transformer_kv_next_unique ON transformer_kv_next
  FIELDS workspace, in, out UNIQUE;
