-- Exact-match admission fields for tensor pages.
-- A page may be reused only when model_digest, dtype, layout, shape,
-- toroidal_phase_encoder_version, address_contract_digest, key_digest,
-- and value_digest all match. RoPE and Euclidean positional encodings
-- are not representable: layout is row-major and dtype is f32 or bf16.
-- r_12 remains opaque (UNSPECIFIED-R12-ACTION). Not a geometry certificate.

DEFINE FIELD OVERWRITE model_digest ON transformer_kv_page TYPE string;
DEFINE FIELD OVERWRITE dtype ON transformer_kv_page TYPE string
  ASSERT $value IN ['f32', 'bf16'];
DEFINE FIELD OVERWRITE layout ON transformer_kv_page TYPE string
  ASSERT $value = 'row-major';
DEFINE FIELD OVERWRITE shape ON transformer_kv_page TYPE array<int>;
DEFINE FIELD OVERWRITE toroidal_phase_encoder_version ON transformer_kv_page TYPE string;
DEFINE FIELD OVERWRITE address_contract_digest ON transformer_kv_page TYPE string;
DEFINE FIELD OVERWRITE key_digest ON transformer_kv_page TYPE string;
DEFINE FIELD OVERWRITE value_digest ON transformer_kv_page TYPE string;
DEFINE FIELD OVERWRITE geometry_space ON transformer_kv_page TYPE string
  ASSERT $value = 'h3_t2_r12_v1';
DEFINE FIELD OVERWRITE quotient_status ON transformer_kv_page TYPE string
  ASSERT $value = 'opaque';
DEFINE FIELD OVERWRITE contract_digest ON transformer_kv_page TYPE string
  ASSERT $value = 'UNSPECIFIED-R12-ACTION';

DEFINE INDEX OVERWRITE transformer_kv_page_identity ON transformer_kv_page
  FIELDS workspace, atom, model_digest, dtype, layout, toroidal_phase_encoder_version, address_contract_digest UNIQUE;
