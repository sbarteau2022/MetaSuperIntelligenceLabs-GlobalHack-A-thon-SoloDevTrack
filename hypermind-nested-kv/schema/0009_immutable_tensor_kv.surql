-- Exact transformer K/V identity and payload are immutable after admission.
-- Runtime counters, access time, closure, and active state remain mutable.

DEFINE FIELD OVERWRITE workspace ON transformer_kv_session TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE model_id ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE model_revision ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE model_digest ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE request_digest ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE dtype ON transformer_kv_session TYPE string
  ASSERT $value IN ['f16', 'bf16', 'f32', 'q8_0', 'q4_0'] READONLY;
DEFINE FIELD OVERWRITE layer_count ON transformer_kv_session TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE kv_head_count ON transformer_kv_session TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE head_dim ON transformer_kv_session TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE geometry_space ON transformer_kv_session TYPE record<geometry_space> READONLY;
DEFINE FIELD OVERWRITE toroidal_phase_encoder_version ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE address_contract_digest ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE prefix_digest ON transformer_kv_session TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE tensor_contract_digest ON transformer_kv_session TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE tppe_configuration_digest ON transformer_kv_session TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE created_at ON transformer_kv_session TYPE datetime DEFAULT time::now() READONLY;

DEFINE FIELD OVERWRITE workspace ON transformer_kv_page TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE session ON transformer_kv_page TYPE record<transformer_kv_session> READONLY;
DEFINE FIELD OVERWRITE layer ON transformer_kv_page TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD OVERWRITE kv_head ON transformer_kv_page TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD OVERWRITE token_start ON transformer_kv_page TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD OVERWRITE token_end ON transformer_kv_page TYPE int ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE tensor_shape ON transformer_kv_page TYPE array<int, 2> READONLY;
DEFINE FIELD OVERWRITE dtype ON transformer_kv_page TYPE string
  ASSERT $value IN ['f16', 'bf16', 'f32', 'q8_0', 'q4_0'] READONLY;
DEFINE FIELD OVERWRITE layout ON transformer_kv_page TYPE string
  ASSERT $value IN ['token_major', 'head_dim_major'] READONLY;
DEFINE FIELD OVERWRITE key_tensor ON transformer_kv_page TYPE bytes READONLY;
DEFINE FIELD OVERWRITE value_tensor ON transformer_kv_page TYPE bytes READONLY;
DEFINE FIELD OVERWRITE key_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE value_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE key_bytes ON transformer_kv_page TYPE int ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE value_bytes ON transformer_kv_page TYPE int ASSERT $value > 0 READONLY;
DEFINE FIELD OVERWRITE address ON transformer_kv_page TYPE record<atom_address> READONLY;
DEFINE FIELD OVERWRITE trajectory_atoms ON transformer_kv_page TYPE array<record<atom>> DEFAULT [] READONLY;
DEFINE FIELD OVERWRITE receipt_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE revision ON transformer_kv_page TYPE int ASSERT $value >= 1 READONLY;
DEFINE FIELD OVERWRITE created_at ON transformer_kv_page TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD OVERWRITE well ON transformer_kv_page TYPE option<record<cognitive_well>> READONLY;
DEFINE FIELD OVERWRITE trajectory ON transformer_kv_page TYPE option<record<distilled_trajectory>> READONLY;
DEFINE FIELD OVERWRITE phase_state_digest ON transformer_kv_page TYPE option<string> READONLY;
