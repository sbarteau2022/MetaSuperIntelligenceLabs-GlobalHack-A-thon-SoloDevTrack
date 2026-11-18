-- Complete the append-only trajectory cell bundle. A recallable trajectory
-- owns its rank-1 edge proofs, rank-2 loop proofs, and audit-only origins.

DEFINE TABLE IF NOT EXISTS edge_crystallization SCHEMAFULL TYPE NORMAL
  COMMENT 'Proof that one typed edge was minted from an admitted rank-1 packet.';
DEFINE FIELD IF NOT EXISTS workspace ON edge_crystallization TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS edge ON edge_crystallization TYPE record<atom_edge> READONLY;
DEFINE FIELD IF NOT EXISTS packet ON edge_crystallization TYPE record<distillation_packet> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON edge_crystallization TYPE record<distillation_admission> READONLY;
DEFINE FIELD IF NOT EXISTS transition_digest ON edge_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS replay_receipt_digest ON edge_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON edge_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON edge_crystallization TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS edge_crystallization_unique ON edge_crystallization
  FIELDS workspace, edge, packet UNIQUE;

DEFINE FIELD IF NOT EXISTS packet ON trajectory_step TYPE option<record<distillation_packet>> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON trajectory_step TYPE option<record<distillation_admission>> READONLY;
DEFINE FIELD IF NOT EXISTS edge_crystallization ON trajectory_step TYPE option<record<edge_crystallization>> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_radius_before ON trajectory_step TYPE option<float> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_radius_after ON trajectory_step TYPE option<float> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_revolutions_before ON trajectory_step TYPE option<int> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_revolutions_after ON trajectory_step TYPE option<int> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_boundary_margin_before ON trajectory_step TYPE option<float> READONLY;
DEFINE FIELD IF NOT EXISTS tppe_boundary_margin_after ON trajectory_step TYPE option<float> READONLY;

DEFINE FIELD IF NOT EXISTS packet ON trajectory_loop TYPE option<record<distillation_packet>> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON trajectory_loop TYPE option<record<distillation_admission>> READONLY;

DEFINE FIELD IF NOT EXISTS run ON local_jacobian_chart TYPE option<record<distillation_run>> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON local_jacobian_chart TYPE option<record<distillation_admission>> READONLY;
DEFINE FIELD IF NOT EXISTS tensor_digest ON local_jacobian_chart TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS numerical_receipt_digest ON local_jacobian_chart TYPE option<string> READONLY;

-- New trajectory edges are immutable. Runtime validity changes append a new
-- revision; the previously receipted edge is never rewritten.
DEFINE FIELD OVERWRITE workspace ON atom_edge TYPE record<workspace> READONLY;
DEFINE FIELD OVERWRITE kind ON atom_edge TYPE string
  ASSERT $value IN ['semantic', 'causal', 'temporal', 'supports', 'contradicts', 'part_of', 'derived_from', 'references', 'next'] READONLY;
DEFINE FIELD OVERWRITE weight ON atom_edge TYPE float ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD OVERWRITE confidence ON atom_edge TYPE float ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD OVERWRITE evidence_atoms ON atom_edge TYPE array<record<atom>> DEFAULT [] READONLY;
DEFINE FIELD OVERWRITE evidence_digest ON atom_edge TYPE string READONLY;
DEFINE FIELD OVERWRITE producer ON atom_edge TYPE string READONLY;
DEFINE FIELD OVERWRITE revision ON atom_edge TYPE int ASSERT $value >= 1 READONLY;
DEFINE FIELD OVERWRITE active ON atom_edge TYPE bool DEFAULT true READONLY;
DEFINE FIELD OVERWRITE valid_from ON atom_edge TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD OVERWRITE valid_to ON atom_edge TYPE option<datetime> READONLY;
DEFINE FIELD OVERWRITE created_at ON atom_edge TYPE datetime DEFAULT time::now() READONLY;
