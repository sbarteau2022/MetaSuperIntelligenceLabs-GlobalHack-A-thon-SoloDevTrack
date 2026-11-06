-- Schema 0009: identity and payload fields are READONLY after admission.
-- A second write with a different key_tensor or value_tensor must fail
-- closed. Do not UPDATE these columns. key_tensor and value_tensor stay bytes.

DEFINE FIELD OVERWRITE workspace ON transformer_kv_page TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE session ON transformer_kv_page TYPE record<transformer_kv_session> READONLY;
DEFINE FIELD OVERWRITE atom ON transformer_kv_page TYPE record<atom> READONLY;
DEFINE FIELD OVERWRITE address ON transformer_kv_page TYPE record<atom_address> READONLY;
DEFINE FIELD OVERWRITE model_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE dtype ON transformer_kv_page TYPE string READONLY
  ASSERT $value IN ['f32', 'bf16'];
DEFINE FIELD OVERWRITE layout ON transformer_kv_page TYPE string READONLY
  ASSERT $value = 'row-major';
DEFINE FIELD OVERWRITE shape ON transformer_kv_page TYPE array<int> READONLY;
DEFINE FIELD OVERWRITE toroidal_phase_encoder_version ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE address_contract_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE key_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE value_digest ON transformer_kv_page TYPE string READONLY;
DEFINE FIELD OVERWRITE geometry_space ON transformer_kv_page TYPE string READONLY
  ASSERT $value = 'h3_t2_r12_v1';
DEFINE FIELD OVERWRITE quotient_status ON transformer_kv_page TYPE string READONLY
  ASSERT $value = 'opaque';
DEFINE FIELD OVERWRITE contract_digest ON transformer_kv_page TYPE string READONLY
  ASSERT $value = 'UNSPECIFIED-R12-ACTION';
DEFINE FIELD OVERWRITE key_tensor ON transformer_kv_page TYPE bytes READONLY;
DEFINE FIELD OVERWRITE value_tensor ON transformer_kv_page TYPE bytes READONLY;

DEFINE FIELD OVERWRITE geometry_space ON transformer_kv_session TYPE string READONLY
  ASSERT $value = 'h3_t2_r12_v1';
DEFINE FIELD OVERWRITE quotient_status ON transformer_kv_session TYPE string READONLY
  ASSERT $value = 'opaque';
DEFINE FIELD OVERWRITE contract_digest ON transformer_kv_session TYPE string READONLY
  ASSERT $value = 'UNSPECIFIED-R12-ACTION';
DEFINE FIELD OVERWRITE model_digest ON transformer_kv_session TYPE string READONLY;
DEFINE FIELD OVERWRITE toroidal_phase_encoder_version ON transformer_kv_session TYPE string READONLY;

DEFINE FIELD OVERWRITE from_page ON transformer_kv_next TYPE record<transformer_kv_page> READONLY;
DEFINE FIELD OVERWRITE to_page ON transformer_kv_next TYPE record<transformer_kv_page> READONLY;
DEFINE FIELD OVERWRITE session ON transformer_kv_next TYPE record<transformer_kv_session> READONLY;
