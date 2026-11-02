-- Version updates may append new address revisions, but may not alter a
-- previously admitted atom's content or an existing H3 x T2 address record.
-- current_address remains mutable only to point to a newly created revision.

DEFINE FIELD OVERWRITE workspace ON atom TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE external_id ON atom TYPE string READONLY;
DEFINE FIELD OVERWRITE content_kind ON atom TYPE string
  ASSERT $value IN ['text', 'image', 'audio', 'video', 'code', 'mixed'] READONLY;
DEFINE FIELD OVERWRITE content_text ON atom TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE content_uri ON atom TYPE option<string> READONLY;
DEFINE FIELD OVERWRITE content_digest ON atom TYPE string READONLY;
DEFINE FIELD OVERWRITE provenance ON atom TYPE string READONLY;
DEFINE FIELD OVERWRITE created_at ON atom TYPE datetime DEFAULT time::now() READONLY;

DEFINE FIELD OVERWRITE workspace ON atom_address TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE atom ON atom_address TYPE record<atom> READONLY;
DEFINE FIELD OVERWRITE h3_lorentz ON atom_address TYPE array<float, 4> READONLY;
DEFINE FIELD OVERWRITE t2_phase ON atom_address TYPE array<float, 2> READONLY;
DEFINE FIELD OVERWRITE geometry_model ON atom_address TYPE string READONLY;
DEFINE FIELD OVERWRITE geometry_version ON atom_address TYPE string READONLY;
DEFINE FIELD OVERWRITE revision ON atom_address TYPE int ASSERT $value >= 1 READONLY;
DEFINE FIELD OVERWRITE created_at ON atom_address TYPE datetime DEFAULT time::now() READONLY;
