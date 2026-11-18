-- Capability Atlas Contract (Migration 0012)
-- Model-neutral relational map of problems, solution families, capabilities,
-- runbooks, MCP servers, plugins, tools, skills, hooks, policy rules, evidence
-- receipts, and failure patterns inside the single GlobalMind semantic graph.
--
-- Authority boundaries:
-- 1. One GlobalMind graph: every capability profile attaches 1-to-1 to an
--    immutable semantic `atom` and `atom_address`, never to `transformer_kv_page`.
-- 2. All 12 atom kinds and 13 edge kinds are strictly schema-enforced and READONLY.
-- 3. Untyped "related" edges, missing receipts, secret/credential values, and
--    in-place overwrites fail closed atomically.

DEFINE TABLE IF NOT EXISTS capability_profile SCHEMAFULL TYPE NORMAL
  COMMENT 'Immutable model-neutral capability profile bound 1-to-1 to one semantic GlobalMind atom.';

DEFINE FIELD IF NOT EXISTS workspace ON capability_profile TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS atom ON capability_profile TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS address ON capability_profile TYPE record<atom_address> READONLY;
DEFINE FIELD IF NOT EXISTS well ON capability_profile TYPE record<cognitive_well> READONLY;
DEFINE FIELD IF NOT EXISTS kind ON capability_profile TYPE string
  ASSERT $value IN [
    'problem_pattern',
    'solution_family',
    'capability',
    'runbook',
    'mcp_server',
    'plugin',
    'tool',
    'skill',
    'hook',
    'policy_rule',
    'evidence_receipt',
    'failure_pattern'
  ] READONLY;
DEFINE FIELD IF NOT EXISTS canonical_name ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS summary ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS version ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS content_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS source_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS configuration_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS lifecycle ON capability_profile TYPE string
  ASSERT $value IN ['candidate', 'qualified', 'benched', 'retired'] READONLY;
DEFINE FIELD IF NOT EXISTS admission_receipt_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS judge_receipt_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS replay_receipt_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;

-- Native scaffold fields (binary envelope + strongly typed operational bounds).
DEFINE FIELD IF NOT EXISTS argument_schema_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS result_schema_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS effect_class ON capability_profile TYPE string
  ASSERT $value IN ['read_only', 'local_write', 'external_write', 'destructive'] READONLY;
DEFINE FIELD IF NOT EXISTS permission_classes ON capability_profile TYPE array<string> DEFAULT [] READONLY;
DEFINE FIELD IF NOT EXISTS credential_classes ON capability_profile TYPE array<string> DEFAULT [] READONLY;
DEFINE FIELD IF NOT EXISTS platform_requirements ON capability_profile TYPE array<string> DEFAULT [] READONLY;
DEFINE FIELD IF NOT EXISTS runtime_requirements ON capability_profile TYPE array<string> DEFAULT [] READONLY;
DEFINE FIELD IF NOT EXISTS version_requirement ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS timeout_ms ON capability_profile TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD IF NOT EXISTS max_retries ON capability_profile TYPE int
  ASSERT $value >= 0 READONLY;
DEFINE FIELD IF NOT EXISTS idempotent ON capability_profile TYPE bool READONLY;
DEFINE FIELD IF NOT EXISTS rollback_policy ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS max_memory_bytes ON capability_profile TYPE int
  ASSERT $value > 0 READONLY;
DEFINE FIELD IF NOT EXISTS health_check_id ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS live_registry_id ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS component_receipt_digest ON capability_profile TYPE option<string>
  ASSERT $value = NONE OR string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS integration_receipt_digest ON capability_profile TYPE option<string>
  ASSERT $value = NONE OR string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS live_receipt_digest ON capability_profile TYPE option<string>
  ASSERT $value = NONE OR string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS incompatible_profiles ON capability_profile TYPE array<record<capability_profile>> DEFAULT [] READONLY;
DEFINE FIELD IF NOT EXISTS benched_reason ON capability_profile TYPE option<string> READONLY;
DEFINE FIELD IF NOT EXISTS expected_utility ON capability_profile TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS resource_cost ON capability_profile TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS prediction_error ON capability_profile TYPE float
  ASSERT $value >= 0.0 READONLY;

-- Kind-specific required payload slots (non-empty and matched to `kind`).
DEFINE FIELD IF NOT EXISTS payload_schema_tag ON capability_profile TYPE string
  ASSERT $value = $this.kind READONLY;
DEFINE FIELD IF NOT EXISTS payload_slot_1 ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS payload_slot_2 ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS payload_slot_3 ON capability_profile TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS payload_slot_4 ON capability_profile TYPE option<string> READONLY;

-- Native binary envelopes (no JSON/JSONL/Python/prompt-schema across planning/execution).
DEFINE FIELD IF NOT EXISTS context_projection_bytes ON capability_profile TYPE bytes READONLY;
DEFINE FIELD IF NOT EXISTS context_projection_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS scaffold_envelope_bytes ON capability_profile TYPE bytes READONLY;
DEFINE FIELD IF NOT EXISTS scaffold_envelope_digest ON capability_profile TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON capability_profile TYPE datetime DEFAULT time::now() READONLY;

DEFINE INDEX IF NOT EXISTS capability_profile_atom_unique ON capability_profile
  FIELDS workspace, atom UNIQUE;
DEFINE INDEX IF NOT EXISTS capability_profile_name_version_unique ON capability_profile
  FIELDS workspace, kind, canonical_name, version UNIQUE;
DEFINE INDEX IF NOT EXISTS capability_profile_well_lookup ON capability_profile
  FIELDS workspace, well, kind, lifecycle;

DEFINE TABLE IF NOT EXISTS capability_edge SCHEMAFULL
  TYPE RELATION IN capability_profile OUT capability_profile ENFORCED
  COMMENT 'Immutable, evidence-bearing directional edge in the Capability Atlas.';

DEFINE FIELD IF NOT EXISTS workspace ON capability_edge TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS from_atom ON capability_edge TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS to_atom ON capability_edge TYPE record<atom> READONLY;
DEFINE FIELD IF NOT EXISTS kind ON capability_edge TYPE string
  ASSERT $value IN [
    'solves',
    'specializes',
    'implements',
    'exposes',
    'requires',
    'composes_with',
    'conflicts_with',
    'fallback_for',
    'enforced_by',
    'triggered_by',
    'verified_by',
    'failed_on',
    'supersedes'
  ] READONLY;
DEFINE FIELD IF NOT EXISTS confidence ON capability_edge TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS evidence_receipt ON capability_edge TYPE option<record<capability_profile>> READONLY;
DEFINE FIELD IF NOT EXISTS evidence_receipt_digest ON capability_edge TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS producer ON capability_edge TYPE string
  ASSERT string::len(string::trim($value)) > 0 READONLY;
DEFINE FIELD IF NOT EXISTS revision ON capability_edge TYPE int
  ASSERT $value >= 1 READONLY;
DEFINE FIELD IF NOT EXISTS valid_from ON capability_edge TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS valid_to ON capability_edge TYPE option<datetime>
  ASSERT $value = NONE OR $value > $this.valid_from READONLY;
DEFINE FIELD IF NOT EXISTS compatibility_digest ON capability_edge TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS edge_envelope_bytes ON capability_edge TYPE bytes READONLY;
DEFINE FIELD IF NOT EXISTS crystallization_receipt_digest ON capability_edge TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON capability_edge TYPE datetime DEFAULT time::now() READONLY;

DEFINE INDEX IF NOT EXISTS capability_edge_revision_unique ON capability_edge
  FIELDS workspace, in, out, kind, revision UNIQUE;
DEFINE INDEX IF NOT EXISTS capability_edge_outgoing ON capability_edge
  FIELDS workspace, in, kind;
DEFINE INDEX IF NOT EXISTS capability_edge_incoming ON capability_edge
  FIELDS workspace, out, kind;

DEFINE TABLE IF NOT EXISTS capability_utility_receipt SCHEMAFULL TYPE NORMAL
  COMMENT 'Append-only outcome and prediction-error receipt for a materialized execution DAG; never mutates semantic coordinates directly.';

DEFINE FIELD IF NOT EXISTS workspace ON capability_utility_receipt TYPE record<workspace> READONLY;
DEFINE FIELD IF NOT EXISTS problem_profile ON capability_utility_receipt TYPE record<capability_profile> READONLY;
DEFINE FIELD IF NOT EXISTS selected_profiles ON capability_utility_receipt TYPE array<record<capability_profile>> READONLY;
DEFINE FIELD IF NOT EXISTS dag_digest ON capability_utility_receipt TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS execution_receipt_digest ON capability_utility_receipt TYPE string
  ASSERT string::len($value) = 64 READONLY;
DEFINE FIELD IF NOT EXISTS observed_utility ON capability_utility_receipt TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS prediction_error ON capability_utility_receipt TYPE float
  ASSERT $value >= 0.0 READONLY;
DEFINE FIELD IF NOT EXISTS resource_cost ON capability_utility_receipt TYPE float
  ASSERT $value >= 0.0 AND $value <= 1.0 READONLY;
DEFINE FIELD IF NOT EXISTS outcome ON capability_utility_receipt TYPE string
  ASSERT $value IN ['succeeded', 'degraded', 'failed', 'blocked_by_policy'] READONLY;
DEFINE FIELD IF NOT EXISTS recorded_at ON capability_utility_receipt TYPE datetime DEFAULT time::now() READONLY;

DEFINE INDEX IF NOT EXISTS capability_utility_receipt_unique ON capability_utility_receipt
  FIELDS workspace, dag_digest, execution_receipt_digest UNIQUE;

