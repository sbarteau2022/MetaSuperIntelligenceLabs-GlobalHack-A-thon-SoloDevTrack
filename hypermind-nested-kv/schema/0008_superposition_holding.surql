-- GlobalMind superposition-holding receipts.
-- Runtime candidates are held without external emission, iterated, and then
-- collapsed onto one replayable branch or failed closed.

DEFINE TABLE IF NOT EXISTS superposition_hold SCHEMAFULL TYPE NORMAL
  COMMENT 'One immutable reasoning hold opened by prediction error, ambiguity, cycle, or boundary.';
DEFINE FIELD IF NOT EXISTS workspace ON superposition_hold TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS decision ON superposition_hold TYPE record<active_inference_decision> READONLY;
DEFINE FIELD IF NOT EXISTS trigger_observation ON superposition_hold TYPE option<record<surprise_observation>> READONLY;
DEFINE FIELD IF NOT EXISTS hub ON superposition_hold TYPE record<cognitive_hub> READONLY;
DEFINE FIELD IF NOT EXISTS well ON superposition_hold TYPE option<record<cognitive_well>> READONLY;
DEFINE FIELD IF NOT EXISTS request_digest ON superposition_hold TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS prefix_digest ON superposition_hold TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS trigger ON superposition_hold TYPE string
  ASSERT $value IN ['no_direct_trajectory', 'fixed_point_or_cycle', 'branch_ambiguity', 'high_prediction_error', 'low_readiness', 'compliance_boundary'] READONLY;
DEFINE FIELD IF NOT EXISTS controller_version ON superposition_hold TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS controller_digest ON superposition_hold TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS branch_limit ON superposition_hold TYPE int
  ASSERT $value > 1 AND $value <= 12 READONLY;
DEFINE FIELD IF NOT EXISTS iteration_limit ON superposition_hold TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD IF NOT EXISTS opened_at ON superposition_hold TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS superposition_hold_unique ON superposition_hold
  FIELDS workspace, request_digest, prefix_digest, controller_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS superposition_branch SCHEMAFULL TYPE NORMAL
  COMMENT 'One candidate trajectory held under the same scorer and compliance gate.';
DEFINE FIELD IF NOT EXISTS workspace ON superposition_branch TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS hold ON superposition_branch TYPE record<superposition_hold> READONLY;
DEFINE FIELD IF NOT EXISTS ordinal ON superposition_branch TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS source ON superposition_branch TYPE string
  ASSERT $value IN ['direct_trajectory', 'phase_neighbor', 'operator_bridge', 'cross_well_bridge', 'fresh_derivation'] READONLY;
DEFINE FIELD IF NOT EXISTS source_trajectory ON superposition_branch TYPE option<record<distilled_trajectory>> READONLY;
DEFINE FIELD IF NOT EXISTS from_atom ON superposition_branch TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS candidate_atom ON superposition_branch TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS prediction_digest ON superposition_branch TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS scorer_version ON superposition_branch TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS scorer_digest ON superposition_branch TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS free_energy_value ON superposition_branch TYPE float READONLY;
DEFINE FIELD IF NOT EXISTS precision_weighted_error ON superposition_branch TYPE float READONLY;
DEFINE FIELD IF NOT EXISTS readiness ON superposition_branch TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS compliance_permitted ON superposition_branch TYPE bool READONLY;
DEFINE FIELD IF NOT EXISTS score_receipt_digest ON superposition_branch TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS compliance_receipt_digest ON superposition_branch TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON superposition_branch TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS superposition_branch_ordinal_unique ON superposition_branch
  FIELDS workspace, hold, ordinal UNIQUE;
DEFINE INDEX IF NOT EXISTS superposition_branch_prediction_unique ON superposition_branch
  FIELDS workspace, hold, prediction_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS superposition_iteration SCHEMAFULL TYPE NORMAL
  COMMENT 'One counterfactual prune/reweight iteration with no externally emitted token.';
DEFINE FIELD IF NOT EXISTS workspace ON superposition_iteration TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS hold ON superposition_iteration TYPE record<superposition_hold> READONLY;
DEFINE FIELD IF NOT EXISTS ordinal ON superposition_iteration TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS input_branch_digest ON superposition_iteration TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS surviving_branches ON superposition_iteration TYPE array<record<superposition_branch>> READONLY;
DEFINE FIELD IF NOT EXISTS rollout_receipt_digest ON superposition_iteration TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS loop_receipt_digest ON superposition_iteration TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS suppression_receipt_digest ON superposition_iteration TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS output_branch_digest ON superposition_iteration TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS external_emissions ON superposition_iteration TYPE int
  ASSERT $value = 0 READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON superposition_iteration TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS superposition_iteration_ordinal_unique ON superposition_iteration
  FIELDS workspace, hold, ordinal UNIQUE;

DEFINE TABLE IF NOT EXISTS superposition_resolution SCHEMAFULL TYPE NORMAL
  COMMENT 'Terminal collapse, mandated iteration, or fail-closed receipt for one hold.';
DEFINE FIELD IF NOT EXISTS workspace ON superposition_resolution TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS hold ON superposition_resolution TYPE record<superposition_hold> READONLY;
DEFINE FIELD IF NOT EXISTS outcome ON superposition_resolution TYPE string
  ASSERT $value IN ['collapsed', 'iterate', 'fail_closed'] READONLY;
DEFINE FIELD IF NOT EXISTS selected_branch ON superposition_resolution TYPE option<record<superposition_branch>> READONLY;
DEFINE FIELD IF NOT EXISTS resulting_trajectory ON superposition_resolution TYPE option<record<distilled_trajectory>> READONLY;
DEFINE FIELD IF NOT EXISTS resulting_atom ON superposition_resolution TYPE option<record<atom>> READONLY;
DEFINE FIELD IF NOT EXISTS judge_receipt_digest ON superposition_resolution TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS replay_receipt_digest ON superposition_resolution TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON superposition_resolution TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS resolution_receipt_digest ON superposition_resolution TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS resolved_at ON superposition_resolution TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS superposition_resolution_hold_unique ON superposition_resolution
  FIELDS workspace, hold UNIQUE;
