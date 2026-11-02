-- GlobalMind distillation, cognitive-well, trajectory, and tensor-KV binding.
-- Additive migration for SurrealDB 3.3.5. Existing/quarantined rows remain
-- untouched: every new admission field is optional on the legacy tables.

DEFINE TABLE IF NOT EXISTS cognitive_hub SCHEMAFULL TYPE NORMAL
  COMMENT 'The one central GlobalMind reference state surrounded by twelve peer wells.';
DEFINE FIELD IF NOT EXISTS workspace ON cognitive_hub TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS geometry_space ON cognitive_hub TYPE record<geometry_space> READONLY;
DEFINE FIELD IF NOT EXISTS label ON cognitive_hub TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS h3_center_lorentz ON cognitive_hub TYPE array<float, 4> READONLY;
DEFINE FIELD IF NOT EXISTS t2_phase ON cognitive_hub TYPE array<float, 2> READONLY;
DEFINE FIELD IF NOT EXISTS reference_contract_digest ON cognitive_hub TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS status ON cognitive_hub TYPE string
  ASSERT $value IN ['specified', 'certified', 'retired'] READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON cognitive_hub TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS cognitive_hub_space_unique ON cognitive_hub
  FIELDS workspace, geometry_space UNIQUE;

DEFINE TABLE IF NOT EXISTS cognitive_well SCHEMAFULL TYPE NORMAL
  COMMENT 'One of twelve peer routing basins surrounding one cognitive hub.';
DEFINE FIELD IF NOT EXISTS workspace ON cognitive_well TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS geometry_space ON cognitive_well TYPE record<geometry_space> READONLY;
DEFINE FIELD IF NOT EXISTS ordinal ON cognitive_well TYPE int
  ASSERT $value >= 0 AND $value < 12 READONLY;
DEFINE FIELD IF NOT EXISTS label ON cognitive_well TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS h3_center_lorentz ON cognitive_well TYPE array<float, 4> READONLY;
DEFINE FIELD IF NOT EXISTS t2_sector ON cognitive_well TYPE int
  ASSERT $value >= 0 AND $value < 12 READONLY;
DEFINE FIELD IF NOT EXISTS boundary_contract_digest ON cognitive_well TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS status ON cognitive_well TYPE string
  ASSERT $value IN ['specified', 'certified', 'retired'] READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON cognitive_well TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS cognitive_well_ordinal_unique ON cognitive_well
  FIELDS workspace, geometry_space, ordinal UNIQUE;
DEFINE INDEX IF NOT EXISTS cognitive_well_sector_unique ON cognitive_well
  FIELDS workspace, geometry_space, t2_sector UNIQUE;

DEFINE TABLE IF NOT EXISTS hub_has_well SCHEMAFULL
  TYPE RELATION IN cognitive_hub OUT cognitive_well ENFORCED
  COMMENT 'One-to-twelve topology edge; runtime certification requires exactly twelve unique outgoing wells.';
DEFINE FIELD IF NOT EXISTS workspace ON hub_has_well TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS topology_contract_digest ON hub_has_well TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON hub_has_well TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS hub_has_well_unique ON hub_has_well
  FIELDS workspace, in, out UNIQUE;
DEFINE INDEX IF NOT EXISTS well_has_one_hub ON hub_has_well
  FIELDS workspace, out UNIQUE;

DEFINE TABLE IF NOT EXISTS address_well_membership SCHEMAFULL TYPE NORMAL
  COMMENT 'Receipted classification of one immutable address into one cognitive well.';
DEFINE FIELD IF NOT EXISTS workspace ON address_well_membership TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS address ON address_well_membership TYPE record<atom_address> READONLY;
DEFINE FIELD IF NOT EXISTS well ON address_well_membership TYPE record<cognitive_well> READONLY;
DEFINE FIELD IF NOT EXISTS classifier_version ON address_well_membership TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS classifier_digest ON address_well_membership TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS boundary_receipt_digest ON address_well_membership TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS classified_at ON address_well_membership TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS address_well_membership_unique ON address_well_membership
  FIELDS workspace, address, classifier_digest UNIQUE;
DEFINE INDEX IF NOT EXISTS address_well_lookup ON address_well_membership
  FIELDS workspace, well, address;

DEFINE TABLE IF NOT EXISTS distillation_packet SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable candidate or verdict for frozen-weight and walked-trajectory distillation.';
DEFINE FIELD IF NOT EXISTS workspace ON distillation_packet TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS run ON distillation_packet TYPE record<distillation_run> READONLY;
DEFINE FIELD IF NOT EXISTS admission ON distillation_packet TYPE record<distillation_admission> READONLY;
DEFINE FIELD IF NOT EXISTS source_kind ON distillation_packet TYPE string
  ASSERT $value IN ['frozen_weight', 'walked_trajectory'] READONLY;
DEFINE FIELD IF NOT EXISTS cell_kind ON distillation_packet TYPE string
  ASSERT $value IN ['rank0_atom', 'rank1_transition', 'rank2_loop', 'rank8_jacobian'] READONLY;
DEFINE FIELD IF NOT EXISTS source_fragment_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS teacher_neutral_input_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS projector_version ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS projector_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS configuration_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS candidate_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS novelty_receipt_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS judge_receipt_digest ON distillation_packet TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS verdict ON distillation_packet TYPE string
  ASSERT $value IN ['admitted', 'rejected'] READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON distillation_packet TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS distillation_packet_candidate_unique ON distillation_packet
  FIELDS workspace, run, candidate_digest UNIQUE;
DEFINE INDEX IF NOT EXISTS distillation_packet_input ON distillation_packet
  FIELDS workspace, teacher_neutral_input_digest, cell_kind, verdict;

DEFINE TABLE IF NOT EXISTS atom_crystallization SCHEMAFULL TYPE NORMAL
  COMMENT 'Proof that an admitted rank-0 atom has a reconstructable payload contract.';
DEFINE FIELD IF NOT EXISTS workspace ON atom_crystallization TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS atom ON atom_crystallization TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS packet ON atom_crystallization TYPE record<distillation_packet> READONLY;
DEFINE FIELD IF NOT EXISTS payload_digest ON atom_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS reconstruction_contract_digest ON atom_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS reconstruction_receipt_digest ON atom_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON atom_crystallization TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON atom_crystallization TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS atom_crystallization_unique ON atom_crystallization
  FIELDS workspace, atom, packet UNIQUE;
DEFINE INDEX IF NOT EXISTS atom_crystallization_payload ON atom_crystallization
  FIELDS workspace, payload_digest;

DEFINE TABLE IF NOT EXISTS local_jacobian_chart SCHEMAFULL TYPE NORMAL
  COMMENT 'Teacher-neutral local tangent chart with admitted rank at most eight.';
DEFINE FIELD IF NOT EXISTS workspace ON local_jacobian_chart TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS anchor ON local_jacobian_chart TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS packet ON local_jacobian_chart TYPE record<distillation_packet> READONLY;
DEFINE FIELD IF NOT EXISTS chart_rank ON local_jacobian_chart TYPE int
  ASSERT $value >= 1 AND $value <= 8 READONLY;
DEFINE FIELD IF NOT EXISTS dtype ON local_jacobian_chart TYPE string
  ASSERT $value IN ['f16', 'bf16', 'f32'] READONLY;
DEFINE FIELD IF NOT EXISTS tensor_shape ON local_jacobian_chart TYPE array<int> READONLY;
DEFINE FIELD IF NOT EXISTS tangent_basis ON local_jacobian_chart TYPE bytes READONLY;
DEFINE FIELD IF NOT EXISTS chart_digest ON local_jacobian_chart TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS marginal_gain_receipt_digest ON local_jacobian_chart TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON local_jacobian_chart TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS local_jacobian_chart_unique ON local_jacobian_chart
  FIELDS workspace, anchor, chart_digest UNIQUE;

-- Extend the immutable trajectory header. Legacy rows remain quarantined
-- because these fields are absent; only fully bound rows are recall-admissible.
DEFINE FIELD IF NOT EXISTS run ON distilled_trajectory TYPE option<record<distillation_run>> READONLY;
DEFINE FIELD IF NOT EXISTS tpe_version ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS address_contract_digest ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS step_count ON distilled_trajectory TYPE option<int>
  ASSERT $value > 0 READONLY;
DEFINE FIELD IF NOT EXISTS replay_receipt_digest ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON distilled_trajectory TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS state ON distilled_trajectory TYPE option<string>
  ASSERT $value IN ['candidate', 'replay_passed', 'crystallized', 'rejected'] READONLY;

DEFINE TABLE IF NOT EXISTS trajectory_step SCHEMAFULL TYPE NORMAL
  COMMENT 'One ordered, typed, replayable transition in a teacher-neutral trajectory.';
DEFINE FIELD IF NOT EXISTS workspace ON trajectory_step TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS trajectory ON trajectory_step TYPE record<distilled_trajectory> READONLY;
DEFINE FIELD IF NOT EXISTS ordinal ON trajectory_step TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS from_atom ON trajectory_step TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS to_atom ON trajectory_step TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS edge ON trajectory_step TYPE record<atom_edge> READONLY;
DEFINE FIELD IF NOT EXISTS transition_kind ON trajectory_step TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS t2_phase_before ON trajectory_step TYPE array<float, 2> READONLY;
DEFINE FIELD IF NOT EXISTS t2_phase_after ON trajectory_step TYPE array<float, 2> READONLY;
DEFINE FIELD IF NOT EXISTS action_digest ON trajectory_step TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS observation_digest ON trajectory_step TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS transition_digest ON trajectory_step TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS replay_receipt_digest ON trajectory_step TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON trajectory_step TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS trajectory_step_ordinal_unique ON trajectory_step
  FIELDS workspace, trajectory, ordinal UNIQUE;
DEFINE INDEX IF NOT EXISTS trajectory_step_transition_unique ON trajectory_step
  FIELDS workspace, trajectory, transition_digest UNIQUE;
DEFINE INDEX IF NOT EXISTS trajectory_step_from_lookup ON trajectory_step
  FIELDS workspace, from_atom, trajectory, ordinal;

DEFINE TABLE IF NOT EXISTS trajectory_loop SCHEMAFULL TYPE NORMAL
  COMMENT 'Rank-2 closed walk with an explicit, independently checked loop invariant.';
DEFINE FIELD IF NOT EXISTS workspace ON trajectory_loop TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS trajectory ON trajectory_loop TYPE record<distilled_trajectory> READONLY;
DEFINE FIELD IF NOT EXISTS start_atom ON trajectory_loop TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS end_atom ON trajectory_loop TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS first_step ON trajectory_loop TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS last_step ON trajectory_loop TYPE int ASSERT $value >= $this.first_step READONLY;
DEFINE FIELD IF NOT EXISTS loop_invariant_kind ON trajectory_loop TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS loop_invariant_digest ON trajectory_loop TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS closure_receipt_digest ON trajectory_loop TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON trajectory_loop TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS trajectory_loop_unique ON trajectory_loop
  FIELDS workspace, trajectory, first_step, last_step UNIQUE;

DEFINE TABLE IF NOT EXISTS active_inference_decision SCHEMAFULL TYPE NORMAL
  COMMENT 'Receipt for adaptive retrieval/forward depth and measured resource outcome.';
DEFINE FIELD IF NOT EXISTS workspace ON active_inference_decision TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS request_digest ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS objective_digest ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS selected_wells ON active_inference_decision TYPE array<record<cognitive_well>> READONLY;
DEFINE FIELD IF NOT EXISTS selected_trajectories ON active_inference_decision TYPE array<record<distilled_trajectory>> READONLY;
DEFINE FIELD IF NOT EXISTS retrieval_depth ON active_inference_decision TYPE int ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS forward_mode ON active_inference_decision TYPE string
  ASSERT $value IN ['recall_only', 'adaptive_forward', 'full_forward'] READONLY;
DEFINE FIELD IF NOT EXISTS controller_version ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS controller_digest ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS quality_receipt_digest ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS resource_receipt_digest ON active_inference_decision TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON active_inference_decision TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS active_inference_decision_unique ON active_inference_decision
  FIELDS workspace, request_digest, controller_digest UNIQUE;

DEFINE TABLE IF NOT EXISTS surprise_observation SCHEMAFULL TYPE NORMAL
  COMMENT 'Prediction-error measurement attached to a routing decision; never a tensor-KV value.';
DEFINE FIELD IF NOT EXISTS workspace ON surprise_observation TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS decision ON surprise_observation TYPE record<active_inference_decision> READONLY;
DEFINE FIELD IF NOT EXISTS hub ON surprise_observation TYPE record<cognitive_hub> READONLY;
DEFINE FIELD IF NOT EXISTS well ON surprise_observation TYPE option<record<cognitive_well>> READONLY;
DEFINE FIELD IF NOT EXISTS trajectory ON surprise_observation TYPE option<record<distilled_trajectory>> READONLY;
DEFINE FIELD IF NOT EXISTS prior_digest ON surprise_observation TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS observation_digest ON surprise_observation TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS estimator_version ON surprise_observation TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS estimator_digest ON surprise_observation TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS prediction_error ON surprise_observation TYPE float READONLY;
DEFINE FIELD IF NOT EXISTS precision ON surprise_observation TYPE float
  ASSERT $value > 0.0 READONLY;
DEFINE FIELD IF NOT EXISTS precision_weighted_error ON surprise_observation TYPE float READONLY;
DEFINE FIELD IF NOT EXISTS surprisal ON surprise_observation TYPE float
  ASSERT $value >= 0.0 READONLY;
DEFINE FIELD IF NOT EXISTS unit ON surprise_observation TYPE string
  ASSERT $value IN ['nats', 'bits'] READONLY;
DEFINE FIELD IF NOT EXISTS bayesian_surprise ON surprise_observation TYPE option<float>
  ASSERT $value = NONE OR $value >= 0.0 READONLY;
DEFINE FIELD IF NOT EXISTS resolution ON surprise_observation TYPE string
  ASSERT $value IN ['reuse', 'deeper_retrieval', 'trajectory_extend', 'full_forward', 'quarantine'] READONLY;
DEFINE FIELD IF NOT EXISTS receipt_digest ON surprise_observation TYPE string READONLY;
DEFINE FIELD IF NOT EXISTS observed_at ON surprise_observation TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS surprise_observation_unique ON surprise_observation
  FIELDS workspace, decision, observation_digest, estimator_digest UNIQUE;
DEFINE INDEX IF NOT EXISTS surprise_observation_route ON surprise_observation
  FIELDS workspace, hub, well, trajectory, observed_at;

-- Bind exact-model tensor pages into the same geometry without allowing them
-- into semantic recall. These remain optional for quarantined historical rows.
DEFINE FIELD IF NOT EXISTS prefix_digest ON transformer_kv_session TYPE option<string>;
DEFINE FIELD IF NOT EXISTS tensor_contract_digest ON transformer_kv_session TYPE option<string>;
DEFINE FIELD IF NOT EXISTS tpe_configuration_digest ON transformer_kv_session TYPE option<string>;
DEFINE FIELD IF NOT EXISTS well ON transformer_kv_page TYPE option<record<cognitive_well>>;
DEFINE FIELD IF NOT EXISTS trajectory ON transformer_kv_page TYPE option<record<distilled_trajectory>>;
DEFINE FIELD IF NOT EXISTS phase_state_digest ON transformer_kv_page TYPE option<string>;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_well ON transformer_kv_page
  FIELDS workspace, well, session, active;
DEFINE INDEX IF NOT EXISTS transformer_kv_page_trajectory ON transformer_kv_page
  FIELDS workspace, trajectory, session, active;
