"""Generate the reviewed, deterministic v1 SQLite migration. No database writes."""
from pathlib import Path

TABLES = {}
def table(name, columns):
    TABLES[name] = columns

def enum(name, values):
    return f"{name} TEXT NOT NULL CHECK({name} IN ({','.join(repr(v) for v in values.split())}))"
def hashcol(name, nullable=False):
    return f"{name} TEXT {' ' if nullable else 'NOT NULL'} CHECK({name} IS NULL OR (length({name})=64 AND {name} NOT GLOB '*[^0-9a-f]*'))"
def ref(name, target, nullable=False):
    return f"{name} TEXT {'' if nullable else 'NOT NULL'} REFERENCES {target}(id)"
def fact(name, columns):
    table(name, ['id TEXT PRIMARY KEY NOT NULL', 'event_seq INTEGER NOT NULL REFERENCES events(seq)', *columns])

table('projects', ['id TEXT PRIMARY KEY NOT NULL', hashcol('identity_hash')])
table('artifacts', ['id TEXT PRIMARY KEY NOT NULL', hashcol('content_hash'), 'byte_count INTEGER NOT NULL CHECK(byte_count>=0)', enum('classification','metadata restricted secret'), enum('availability','available withheld erased unknown')])
table('config_snapshots', ['id TEXT PRIMARY KEY NOT NULL', ref('artifact_id','artifacts'), hashcol('source_revision_hash')])
table('sessions', ['id TEXT PRIMARY KEY NOT NULL', ref('project_id','projects'), hashcol('identity_hash'), ref('config_id','config_snapshots')])
table('tasks', ['id TEXT PRIMARY KEY NOT NULL', ref('session_id','sessions'), ref('objective_artifact_id','artifacts'), ref('criteria_artifact_id','artifacts')])
table('runs', ['id TEXT PRIMARY KEY NOT NULL', ref('task_id','tasks'), ref('parent_run_id','runs',True), ref('config_id','config_snapshots'), 'trace_id TEXT NOT NULL UNIQUE'])
table('actors', ['id TEXT PRIMARY KEY NOT NULL', ref('project_id','projects'), enum('kind','meta swarm agent human system'), hashcol('identity_hash')])
table('model_instances', ['id TEXT PRIMARY KEY NOT NULL', ref('actor_id','actors'), hashcol('weights_hash'), hashcol('tokenizer_hash'), ref('config_id','config_snapshots'), hashcol('backend_hash'), hashcol('device_hash'), 'context_capacity INTEGER NOT NULL CHECK(context_capacity>0)'])
table('capabilities', ['id TEXT PRIMARY KEY NOT NULL', enum('kind','tool skill hook mcp_server mcp_method plugin resource prompt'), hashcol('definition_hash'), hashcol('version_hash'), ref('parent_id','capabilities',True)])
table('spans', ['id TEXT PRIMARY KEY NOT NULL', ref('run_id','runs'), ref('parent_span_id','spans',True), ref('actor_id','actors'), enum('kind','meta swarm model tool skill hook mcp plugin retrieval memory execution system')])
table('invocations', ['id TEXT PRIMARY KEY NOT NULL', ref('span_id','spans'), ref('capability_id','capabilities'), hashcol('request_hash'), 'max_attempts INTEGER NOT NULL CHECK(max_attempts BETWEEN 1 AND 1000)'])
table('attempts', ['id TEXT PRIMARY KEY NOT NULL', ref('invocation_id','invocations'), 'attempt_no INTEGER NOT NULL CHECK(attempt_no>0)', hashcol('strategy_hash'), 'UNIQUE(invocation_id,attempt_no)'])
table('events', ['seq INTEGER PRIMARY KEY', 'id TEXT NOT NULL UNIQUE', ref('run_id','runs'), ref('span_id','spans',True), ref('actor_id','actors',True), 'producer_id TEXT NOT NULL', 'producer_seq INTEGER NOT NULL CHECK(producer_seq>=0)', 'occurred_at_us INTEGER NOT NULL CHECK(occurred_at_us>=0)', 'boot_id TEXT NOT NULL', 'monotonic_ns INTEGER NOT NULL CHECK(monotonic_ns>=0)', 'ingested_at_us INTEGER NOT NULL CHECK(ingested_at_us>=0)', enum('event_type','lifecycle invocation check recovery decision lease resource inference graph knowledge kv context atom retrieval rerank novelty memory hunger security execution research service telemetry governance'), 'supersedes_seq INTEGER REFERENCES events(seq)', hashcol('body_hash'), hashcol('previous_hash'), hashcol('event_hash'), 'UNIQUE(producer_id,producer_seq)'])
fact('lifecycle_events', [enum('subject_kind','run span task session actor model'), 'subject_id TEXT NOT NULL', enum('state','requested queued started idle completed failed cancelled timed_out abandoned unknown'), ref('evidence_id','artifacts',True)])
fact('invocation_events', [ref('attempt_id','attempts'), enum('phase','requested dispatched started completed'), enum('status','pending running succeeded failed cancelled timed_out denied unknown'), hashcol('error_class_hash',True), ref('receipt_id','artifacts',True), 'duration_ns INTEGER CHECK(duration_ns>=0)'])
fact('acceptance_checks', [ref('attempt_id','attempts',True), ref('criteria_id','artifacts'), enum('result','pass fail error skipped unknown'), ref('evidence_id','artifacts',True), hashcol('checker_version_hash'), "CHECK(result!='pass' OR evidence_id IS NOT NULL)"])
fact('objective_outcomes', [ref('check_id','acceptance_checks'), enum('result','pass fail unknown')])
fact('recovery_edges', [ref('from_attempt_id','attempts'), ref('to_attempt_id','attempts'), enum('kind','retry alternate_tool alternate_model escalation rollback'), hashcol('reason_code_hash'), 'CHECK(from_attempt_id!=to_attempt_id)'])
fact('meta_decisions', [ref('meta_actor_id','actors'), enum('decision','activate deny defer release route compact distill'), ref('justification_id','artifacts'), 'estimated_cost_microusd INTEGER NOT NULL CHECK(estimated_cost_microusd>=0)', 'budget_microusd INTEGER NOT NULL CHECK(budget_microusd>=estimated_cost_microusd)', 'max_agents INTEGER NOT NULL CHECK(max_agents>=0)', 'expires_at_us INTEGER NOT NULL CHECK(expires_at_us>=0)', enum('approval','approved denied pending'), ref('policy_id','artifacts')])
fact('agent_leases', [ref('decision_id','meta_decisions'), ref('agent_id','actors'), ref('swarm_id','actors'), 'reserved_cost_microusd INTEGER NOT NULL CHECK(reserved_cost_microusd>=0)', 'expires_at_us INTEGER NOT NULL CHECK(expires_at_us>=0)', 'UNIQUE(decision_id,agent_id)'])
fact('lease_events', [ref('lease_id','agent_leases'), enum('state','activated working idle released expired failed'), 'idle_ns INTEGER CHECK(idle_ns>=0)', 'actual_cost_microusd INTEGER CHECK(actual_cost_microusd>=0)'])
fact('messages', [ref('sender_id','actors'), ref('receiver_id','actors'), hashcol('message_hash'), enum('state','sent delivered acknowledged rejected dropped'), 'byte_count INTEGER NOT NULL CHECK(byte_count>=0)'])
fact('capability_events', [ref('capability_id','capabilities'), enum('action','registered resolved enabled disabled denied uninstalled'), ref('evidence_id','artifacts',True)])
fact('hook_events', [ref('capability_id','capabilities'), ref('attempt_id','attempts'), enum('moment','before after error'), enum('effect','allow deny append_context skip error'), hashcol('context_hash',True)])
fact('mcp_exchanges', [ref('attempt_id','attempts'), ref('server_id','capabilities'), hashcol('request_id_hash'), enum('transport','stdio http sse'), enum('operation','initialize list call read subscribe cancel notify'), enum('status','sent received error timeout disconnected'), 'protocol_code INTEGER'])
table('allocations', ['id TEXT PRIMARY KEY NOT NULL', ref('owner_id','actors'), hashcol('device_hash'), enum('kind','weights kv process gpu_buffer host_buffer shared_pool'), enum('accounting','exclusive shared')])
fact('resource_samples', [ref('allocation_id','allocations',True), ref('model_id','model_instances',True), enum('metric','cpu_ns gpu_ns ane_ns rss_bytes allocated_bytes peak_bytes tokens_input tokens_output tokens_cached energy_uj cost_microusd io_read_bytes io_write_bytes network_bytes queue_ns idle_ns utilization_ppm thermal_state'), enum('sample_kind','delta counter gauge'), enum('quality','measured estimated unavailable'), 'value INTEGER CHECK(value>=0)', 'interval_start_us INTEGER NOT NULL CHECK(interval_start_us>=0)', 'interval_end_us INTEGER NOT NULL CHECK(interval_end_us>=interval_start_us)', hashcol('source_hash'), "CHECK((quality='unavailable' AND value IS NULL) OR (quality!='unavailable' AND value IS NOT NULL))"])
fact('inference_events', [ref('model_id','model_instances'), enum('phase','load prefill decode draft verify unload'), 'input_tokens INTEGER CHECK(input_tokens>=0)', 'output_tokens INTEGER CHECK(output_tokens>=0)', 'ttft_ns INTEGER CHECK(ttft_ns>=0)', 'duration_ns INTEGER CHECK(duration_ns>=0)', 'drafted_tokens INTEGER CHECK(drafted_tokens>=0)', 'accepted_tokens INTEGER CHECK(accepted_tokens>=0 AND accepted_tokens<=drafted_tokens)', ref('quality_check_id','acceptance_checks',True)])
table('graph_versions', ['id TEXT PRIMARY KEY NOT NULL', ref('run_id','runs'), enum('kind','dag knowledge capability'), hashcol('snapshot_hash'), ref('previous_id','graph_versions',True)])
table('graph_nodes', ['id TEXT PRIMARY KEY NOT NULL', ref('graph_id','graph_versions'), enum('kind','task capability note concept atom artifact decision checkpoint'), hashcol('external_id_hash'), ref('artifact_id','artifacts',True)])
fact('graph_edges', [ref('graph_id','graph_versions'), ref('source_id','graph_nodes'), ref('target_id','graph_nodes'), enum('relation','depends_on contains references derives_from contradicts supports supersedes links_to'), 'CHECK(source_id!=target_id)', 'UNIQUE(graph_id,source_id,target_id,relation)'])
fact('dag_transitions', [ref('node_id','graph_nodes'), enum('state','blocked ready queued running succeeded failed cancelled skipped'), ref('span_id','spans',True), hashcol('reason_hash',True)])
fact('scheduler_decisions', [ref('node_id','graph_nodes'), ref('actor_id','actors',True), enum('action','admit defer preempt isolate resume breaker_open breaker_close'), ref('policy_id','artifacts'), 'queue_depth INTEGER NOT NULL CHECK(queue_depth>=0)'])
fact('graph_crosslinks', [ref('source_id','graph_nodes'), ref('target_id','graph_nodes'), enum('relation','informed_by produced evidenced_by')])
fact('obsidian_sync', [ref('graph_id','graph_versions'), hashcol('vault_hash'), hashcol('manifest_hash'), enum('direction','import export reconcile'), enum('result','pass fail conflict unknown'), 'nodes_count INTEGER NOT NULL CHECK(nodes_count>=0)', 'edges_count INTEGER NOT NULL CHECK(edges_count>=0)', ref('evidence_id','artifacts',True)])
table('caches', ['id TEXT PRIMARY KEY NOT NULL', ref('owner_id','actors'), hashcol('identity_hash'), enum('kind','application_kv transformer_kv prefix radix retrieval'), 'capacity_bytes INTEGER NOT NULL CHECK(capacity_bytes>=0)', 'capacity_tokens INTEGER CHECK(capacity_tokens>=0)'])
fact('kv_operations', [ref('cache_id','caches'), hashcol('identity_hash'), hashcol('key_hash'), enum('operation','lookup read write reuse evict invalidate allocate release'), enum('result','hit miss stored rejected evicted error unknown'), 'bytes_count INTEGER CHECK(bytes_count>=0)', 'tokens_count INTEGER CHECK(tokens_count>=0)', ref('producer_span_id','spans',True)])
fact('kv_samples', [ref('cache_id','caches'), 'used_bytes INTEGER NOT NULL CHECK(used_bytes>=0)', 'used_tokens INTEGER CHECK(used_tokens>=0)', 'entries_count INTEGER NOT NULL CHECK(entries_count>=0)', enum('quality','measured estimated')])
fact('context_events', [ref('model_id','model_instances'), enum('action','assemble budget trim compact checkpoint restore pin unpin'), 'used_tokens INTEGER NOT NULL CHECK(used_tokens>=0)', 'limit_tokens INTEGER NOT NULL CHECK(limit_tokens>0 AND used_tokens<=limit_tokens)', ref('checkpoint_id','artifacts',True), ref('invariants_id','artifacts'), enum('result','pass fail deferred unknown')])
table('atoms', ['id TEXT PRIMARY KEY NOT NULL', ref('project_id','projects'), hashcol('source_atom_id_hash')])
fact('atom_revisions', [ref('atom_id','atoms'), ref('content_id','artifacts'), ref('prior_revision_id','atom_revisions',True), 'rank INTEGER NOT NULL CHECK(rank>=0)', hashcol('embedding_hash'), hashcol('method_version_hash'), ref('judge_check_id','acceptance_checks'), 'salience REAL NOT NULL CHECK(salience BETWEEN 0 AND 1)'])
fact('atom_operations', [ref('atom_id','atoms'), ref('revision_id','atom_revisions'), enum('operation','retrieved presented cited verified_used added reinforced updated merged rejected pinned decayed'), ref('evidence_id','artifacts',True), "CHECK(operation!='verified_used' OR evidence_id IS NOT NULL)"])
fact('ingest_stages', [ref('source_id','artifacts'), enum('stage','parse_5w condense judge mind_tail deposit embed index deduplicate quarantine'), enum('result','pass fail skipped unknown'), ref('output_id','artifacts',True), ref('check_id','acceptance_checks',True)])
fact('retrieval_queries', [hashcol('query_hash'), hashcol('method_version_hash'), enum('method','geometric lexical fused pami full_scan'), 'requested_k INTEGER NOT NULL CHECK(requested_k>0)', 'returned_count INTEGER NOT NULL CHECK(returned_count>=0)', 'duration_ns INTEGER NOT NULL CHECK(duration_ns>=0)', enum('mode','primary fallback degraded'), hashcol('degradation_hash',True), 'depth INTEGER CHECK(depth>=0)', 'participation_ratio REAL CHECK(participation_ratio>=0)'])
fact('retrieval_hits', [ref('query_id','retrieval_queries'), ref('revision_id','atom_revisions'), 'rank INTEGER NOT NULL CHECK(rank>0)', 'score REAL NOT NULL', enum('stage','candidate reranked selected presented'), 'UNIQUE(query_id,stage,rank)'])
table('trajectories', ['id TEXT PRIMARY KEY NOT NULL', ref('run_id','runs'), ref('parent_id','trajectories',True), hashcol('path_hash')])
fact('trajectory_steps', [ref('trajectory_id','trajectories'), 'step_no INTEGER NOT NULL CHECK(step_no>=0)', ref('span_id','spans'), ref('atom_id','atoms',True), 'UNIQUE(trajectory_id,step_no)'])
fact('rerank_runs', [ref('query_id','retrieval_queries',True), hashcol('ranker_version_hash'), ref('criteria_id','artifacts'), 'candidate_count INTEGER NOT NULL CHECK(candidate_count>=0)'])
fact('rerank_candidates', [ref('rerank_id','rerank_runs'), ref('trajectory_id','trajectories',True), ref('revision_id','atom_revisions',True), 'old_rank INTEGER NOT NULL CHECK(old_rank>0)', 'new_rank INTEGER NOT NULL CHECK(new_rank>0)', 'score REAL NOT NULL', 'selected INTEGER NOT NULL CHECK(selected IN(0,1))', 'CHECK((trajectory_id IS NULL)!=(revision_id IS NULL))', 'UNIQUE(rerank_id,new_rank)'])
fact('novelty_assessments', [ref('candidate_id','artifacts'), hashcol('comparison_set_hash'), hashcol('method_version_hash'), 'score REAL NOT NULL', 'threshold REAL NOT NULL', enum('decision','admit reinforce reject defer'), ref('admitted_atom_id','atoms',True), ref('evidence_id','artifacts'), "CHECK(decision!='admit' OR admitted_atom_id IS NOT NULL)"])
fact('memory_operations', [enum('operation','traverse basin_create carve dissolve consolidate checkpoint snapshot recover passport reweight'), ref('input_id','artifacts',True), ref('output_id','artifacts',True), hashcol('fingerprint_before'), hashcol('fingerprint_after'), enum('result','pass fail deferred unknown'), 'atom_count INTEGER CHECK(atom_count>=0)', 'invariants_preserved INTEGER CHECK(invariants_preserved IN(0,1))'])
fact('hunger_events', [hashcol('gap_hash'), enum('tap','traverse_refused lexical_thin query_thin judge_rejected uncovered_task'), enum('action','observed sated reopened'), 'depth INTEGER NOT NULL CHECK(depth>=0)', 'threshold REAL NOT NULL', ref('evidence_id','artifacts',True)])
fact('distillation_events', [hashcol('dataset_hash'), ref('teacher_id','model_instances',True), ref('student_id','model_instances',True), enum('stage','preregister carve generate judge consolidate admit reject'), ref('check_id','acceptance_checks',True), 'samples_count INTEGER NOT NULL CHECK(samples_count>=0)'])
fact('policy_events', [ref('actor_id','actors'), ref('policy_id','artifacts'), hashcol('capability_token_hash',True), enum('gate','auth role capability sandbox network filesystem approval egress'), enum('verdict','allow deny error unknown'), ref('evidence_id','artifacts',True)])
table('holes', ['id TEXT PRIMARY KEY NOT NULL', ref('run_id','runs'), hashcol('sandbox_profile_hash'), hashcol('base_revision_hash'), ref('policy_id','artifacts')])
fact('hole_events', [ref('hole_id','holes'), enum('action','create execute test diff review approve reject merge destroy'), ref('patch_id','artifacts',True), ref('check_id','acceptance_checks',True), ref('approver_id','actors',True), enum('result','pass fail blocked unknown')])
fact('approvals', [ref('subject_id','artifacts'), ref('actor_id','actors'), enum('decision','approve deny revoke expire'), ref('policy_id','artifacts'), 'expires_at_us INTEGER CHECK(expires_at_us>=0)'])
fact('receipts', [ref('attempt_id','attempts',True), hashcol('receipt_hash'), ref('artifact_id','artifacts'), hashcol('merkle_root',True), ref('proof_id','artifacts',True), enum('verification','verified failed unverified')])
table('experiments', ['id TEXT PRIMARY KEY NOT NULL', ref('project_id','projects'), ref('preregistration_id','artifacts'), hashcol('workload_hash'), hashcol('hardware_hash'), hashcol('baseline_hash'), ref('config_id','config_snapshots')])
fact('experiment_results', [ref('experiment_id','experiments'), ref('check_id','acceptance_checks'), ref('evidence_id','artifacts'), enum('arm','control candidate ablation pressure adversarial')])
fact('claims', [ref('subject_id','artifacts'), enum('label','VERIFIED REPORTED INFERRED PROPOSED'), ref('experiment_id','experiments',True), ref('evidence_id','artifacts',True), "CHECK(label!='VERIFIED' OR (experiment_id IS NOT NULL AND evidence_id IS NOT NULL))"])
fact('judge_results', [ref('check_id','acceptance_checks'), hashcol('judge_version_hash'), ref('model_id','model_instances',True), enum('kind','deterministic model human'), 'score REAL', ref('evidence_id','artifacts')])
fact('service_events', [hashcol('service_hash'), enum('state','starting healthy degraded unavailable stopped recovered'), hashcol('reason_hash'), 'pid INTEGER CHECK(pid>0)', 'queue_depth INTEGER CHECK(queue_depth>=0)', ref('evidence_id','artifacts',True)])
fact('degradation_events', [hashcol('component_hash'), hashcol('reason_hash'), hashcol('fallback_hash'), ref('receipt_id','artifacts'), enum('state','entered continued escalated recovered')])
fact('telemetry_delivery', [hashcol('batch_hash'), enum('state','queued replayed duplicate rejected dropped acknowledged'), 'count INTEGER NOT NULL CHECK(count>=0)', 'first_producer_seq INTEGER CHECK(first_producer_seq>=0)', 'last_producer_seq INTEGER CHECK(last_producer_seq>=first_producer_seq)', hashcol('reason_hash',True)])
fact('governance_events', [enum('operation','config release migration backup restore export retention erasure legal_hold access'), ref('subject_id','artifacts'), ref('policy_id','artifacts'), enum('result','requested pass fail denied deferred'), ref('evidence_id','artifacts',True)])
table('ledger_roots', ['id TEXT PRIMARY KEY NOT NULL', 'first_seq INTEGER NOT NULL REFERENCES events(seq)', 'last_seq INTEGER NOT NULL REFERENCES events(seq)', hashcol('root_hash'), "algorithm TEXT NOT NULL CHECK(algorithm='sha256_binary_duplicate_last')", 'CHECK(first_seq>0 AND last_seq>=first_seq)', 'UNIQUE(first_seq,last_seq)'])

fact('budget_limits', [ref('decision_id','meta_decisions'), enum('resource','cost_microusd tokens cpu_ns gpu_ns memory_bytes energy_uj wall_ns idle_ns'), 'hard_limit INTEGER NOT NULL CHECK(hard_limit>=0)', 'soft_limit INTEGER NOT NULL CHECK(soft_limit>=0 AND soft_limit<=hard_limit)', enum('on_breach','deny stop release escalate')])
fact('budget_breaches', [ref('limit_id','budget_limits'), 'observed_value INTEGER NOT NULL CHECK(observed_value>=0)', enum('action','denied stopped released escalated unknown'), ref('evidence_id','artifacts')])
fact('model_routing', [ref('decision_id','meta_decisions'), ref('model_id','model_instances'), enum('action','selected rejected fallback loaded reused unloaded'), ref('evidence_id','artifacts',True)])
fact('slo_evaluations', [ref('policy_id','artifacts'), enum('metric','ttft_ns throughput_tok_s latency_ns error_rate memory_bytes cost_microusd queue_ns'), 'window_start_us INTEGER NOT NULL CHECK(window_start_us>=0)', 'window_end_us INTEGER NOT NULL CHECK(window_end_us>window_start_us)', 'sample_count INTEGER NOT NULL CHECK(sample_count>=0)', 'observed_value REAL', 'threshold REAL NOT NULL', enum('result','pass fail unknown'), ref('evidence_id','artifacts',True)])
fact('alert_events', [ref('evaluation_id','slo_evaluations'), enum('state','raised acknowledged resolved suppressed'), enum('severity','info warning critical'), ref('evidence_id','artifacts',True)])
fact('context_sources', [ref('context_event_id','context_events'), ref('source_id','artifacts'), enum('layer','org_policy user project session retrieved'), 'token_count INTEGER NOT NULL CHECK(token_count>=0)', 'pinned INTEGER NOT NULL CHECK(pinned IN(0,1))'])
fact('telemetry_sessions', [hashcol('monitor_hash'), hashcol('config_hash'), enum('state','started resumed healthy degraded stopped'), 'last_ack_seq INTEGER CHECK(last_ack_seq>=0)', 'pending_count INTEGER NOT NULL CHECK(pending_count>=0)', 'dropped_count INTEGER NOT NULL CHECK(dropped_count>=0)'])
fact('daily_rollups', ['day_utc INTEGER NOT NULL CHECK(day_utc>=19700101)', ref('aggregate_id','artifacts'), 'first_seq INTEGER NOT NULL REFERENCES events(seq)', 'last_seq INTEGER NOT NULL REFERENCES events(seq)', 'event_count INTEGER NOT NULL CHECK(event_count>=0)', 'CHECK(last_seq>=first_seq)'])

sql = ['-- N³ traceability v1. Generated by build_schema.py; do not hand-edit.', '-- Connection policy: foreign_keys=ON, journal_mode=WAL, synchronous=FULL.', 'CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at_us INTEGER NOT NULL) STRICT;']
for name, cols in TABLES.items():
    sql.append(f'CREATE TABLE {name} (\n  ' + ',\n  '.join(cols) + '\n) STRICT;')
for name in ['schema_migrations', *TABLES]:
    for action in ['UPDATE','DELETE']:
        sql.append(f"CREATE TRIGGER {name}_no_{action.lower()} BEFORE {action} ON {name} BEGIN SELECT RAISE(ABORT,'append-only'); END;")
    sql.append(f"CREATE TRIGGER {name}_no_replace BEFORE INSERT ON {name} WHEN EXISTS(SELECT 1 FROM {name} WHERE {'version=NEW.version' if name=='schema_migrations' else 'seq=NEW.seq' if name=='events' else 'id=NEW.id'}) BEGIN SELECT RAISE(ABORT,'append-only replacement'); END;")
    if any('event_seq ' in c for c in TABLES.get(name, [])):
        sql.append(f'CREATE INDEX {name}_event_idx ON {name}(event_seq);')
# Guard every unique key before INSERT OR REPLACE, even with recursive_triggers OFF.
for name, cols in TABLES.items():
    keys=[]
    for col in cols:
        if ' UNIQUE' in col and not col.startswith('UNIQUE('): keys.append([col.split()[0]])
        if col.startswith('UNIQUE('): keys.append(col[7:-1].split(','))
    for i, key in enumerate(keys):
        cond=' AND '.join(f'{field}=NEW.{field}' for field in key)
        sql.append(f"CREATE TRIGGER {name}_unique_{i} BEFORE INSERT ON {name} WHEN EXISTS(SELECT 1 FROM {name} WHERE {cond}) BEGIN SELECT RAISE(ABORT,'append-only unique replacement'); END;")
# Index every foreign key for bounded joins and integrity checks.
import re
for name, cols in TABLES.items():
    for col in cols:
        if ' REFERENCES ' in col and not col.startswith('event_seq '):
            field=col.split()[0]
            sql.append(f'CREATE INDEX {name}_{field}_idx ON {name}({field});')

def trigger(name, target, when, message):
    sql.append(f"CREATE TRIGGER {name} BEFORE INSERT ON {target} WHEN {when} BEGIN SELECT RAISE(ABORT,'{message}'); END;")
trigger('events_order','events', "NEW.seq!=(SELECT COALESCE(MAX(seq),0)+1 FROM events) OR NEW.ingested_at_us<(SELECT COALESCE(MAX(ingested_at_us),0) FROM events) OR NEW.previous_hash!=COALESCE((SELECT event_hash FROM events ORDER BY seq DESC LIMIT 1),printf('%064d',0))",'ledger order or chain')
trigger('event_span_scope','events', 'NEW.span_id IS NOT NULL AND NEW.run_id!=(SELECT run_id FROM spans WHERE id=NEW.span_id)', 'cross-run span')
trigger('span_parent_scope','spans', 'NEW.parent_span_id IS NOT NULL AND (NEW.id=NEW.parent_span_id OR NEW.run_id!=(SELECT run_id FROM spans WHERE id=NEW.parent_span_id))', 'invalid span parent')
trigger('attempt_limit','attempts','NEW.attempt_no>(SELECT max_attempts FROM invocations WHERE id=NEW.invocation_id)', 'attempt limit')
trigger('objective_pass','objective_outcomes', "NEW.result='pass' AND COALESCE((SELECT result FROM acceptance_checks WHERE id=NEW.check_id),'unknown')!='pass'",'objective requires passed check')
trigger('recovery_material_change','recovery_edges', '(SELECT strategy_hash FROM attempts WHERE id=NEW.from_attempt_id)=(SELECT strategy_hash FROM attempts WHERE id=NEW.to_attempt_id)', 'recovery requires changed strategy')
trigger('meta_role','meta_decisions', "(SELECT kind FROM actors WHERE id=NEW.meta_actor_id)!='meta'", 'meta decision requires meta actor')
trigger('lease_gate','agent_leases', "NOT EXISTS(SELECT 1 FROM meta_decisions d JOIN events e ON e.seq=NEW.event_seq WHERE d.id=NEW.decision_id AND d.approval='approved' AND d.decision='activate' AND NEW.expires_at_us<=d.expires_at_us AND e.occurred_at_us<NEW.expires_at_us AND e.occurred_at_us<d.expires_at_us AND (SELECT COUNT(*) FROM agent_leases WHERE decision_id=d.id)<d.max_agents AND (SELECT COALESCE(SUM(reserved_cost_microusd),0) FROM agent_leases WHERE decision_id=d.id)+NEW.reserved_cost_microusd<=d.budget_microusd)", 'unapproved expired or over-budget lease')
trigger('lease_roles','agent_leases', "(SELECT kind FROM actors WHERE id=NEW.agent_id)!='agent' OR (SELECT kind FROM actors WHERE id=NEW.swarm_id)!='swarm'", 'invalid lease actors')
trigger('kv_identity','kv_operations','NEW.identity_hash!=(SELECT identity_hash FROM caches WHERE id=NEW.cache_id)', 'cross-identity KV')
trigger('kv_capacity','kv_samples','NEW.used_bytes>(SELECT capacity_bytes FROM caches WHERE id=NEW.cache_id) OR NEW.used_tokens>(SELECT capacity_tokens FROM caches WHERE id=NEW.cache_id)', 'KV exceeds capacity')
trigger('atom_revision_scope','atom_operations','NEW.atom_id!=(SELECT atom_id FROM atom_revisions WHERE id=NEW.revision_id)', 'atom revision mismatch')
trigger('atom_judge_gate','atom_revisions', "COALESCE((SELECT result FROM acceptance_checks WHERE id=NEW.judge_check_id),'unknown')!='pass'", 'atom requires passed judge')
trigger('edge_graph_scope','graph_edges','NEW.graph_id!=(SELECT graph_id FROM graph_nodes WHERE id=NEW.source_id) OR NEW.graph_id!=(SELECT graph_id FROM graph_nodes WHERE id=NEW.target_id)', 'cross-graph edge')
trigger('edge_dag_cycle','graph_edges', "(SELECT kind FROM graph_versions WHERE id=NEW.graph_id) IN('dag','capability') AND EXISTS(WITH RECURSIVE reachable(id) AS (SELECT NEW.target_id UNION SELECT e.target_id FROM graph_edges e JOIN reachable r ON e.source_id=r.id WHERE e.graph_id=NEW.graph_id) SELECT 1 FROM reachable WHERE id=NEW.source_id)", 'DAG cycle')
trigger('invocation_phase_status','invocation_events', "NOT ((NEW.phase='requested' AND NEW.status IN('pending','denied','unknown')) OR (NEW.phase='dispatched' AND NEW.status IN('pending','running','unknown')) OR (NEW.phase='started' AND NEW.status='running') OR (NEW.phase='completed' AND NEW.status IN('succeeded','failed','cancelled','timed_out','denied','unknown')))", 'phase status mismatch')
trigger('invocation_success_started','invocation_events', "NEW.status='succeeded' AND NOT EXISTS(SELECT 1 FROM invocation_events WHERE attempt_id=NEW.attempt_id AND phase='started')", 'success requires observed execution')
trigger('invocation_scope','invocation_events', '(SELECT run_id FROM events WHERE seq=NEW.event_seq)!=(SELECT s.run_id FROM attempts a JOIN invocations i ON i.id=a.invocation_id JOIN spans s ON s.id=i.span_id WHERE a.id=NEW.attempt_id)', 'cross-run invocation')
trigger('check_attempt_scope','acceptance_checks', 'NEW.attempt_id IS NOT NULL AND (SELECT run_id FROM events WHERE seq=NEW.event_seq)!=(SELECT s.run_id FROM attempts a JOIN invocations i ON i.id=a.invocation_id JOIN spans s ON s.id=i.span_id WHERE a.id=NEW.attempt_id)', 'cross-run check')
trigger('recovery_run_scope','recovery_edges', '(SELECT s.run_id FROM attempts a JOIN invocations i ON i.id=a.invocation_id JOIN spans s ON s.id=i.span_id WHERE a.id=NEW.to_attempt_id)!=(SELECT s.run_id FROM attempts a JOIN invocations i ON i.id=a.invocation_id JOIN spans s ON s.id=i.span_id WHERE a.id=NEW.from_attempt_id)', 'cross-run recovery')
trigger('actor_span_scope','spans', '(SELECT project_id FROM actors WHERE id=NEW.actor_id)!=(SELECT se.project_id FROM runs r JOIN tasks t ON t.id=r.task_id JOIN sessions se ON se.id=t.session_id WHERE r.id=NEW.run_id)', 'cross-project actor')
trigger('event_actor_scope','events', 'NEW.actor_id IS NOT NULL AND (SELECT project_id FROM actors WHERE id=NEW.actor_id)!=(SELECT se.project_id FROM runs r JOIN tasks t ON t.id=r.task_id JOIN sessions se ON se.id=t.session_id WHERE r.id=NEW.run_id)', 'cross-project event actor')
trigger('graph_version_scope','graph_versions', 'NEW.previous_id IS NOT NULL AND (NEW.run_id!=(SELECT run_id FROM graph_versions WHERE id=NEW.previous_id) OR NEW.kind!=(SELECT kind FROM graph_versions WHERE id=NEW.previous_id))', 'graph revision scope')
trigger('supersedes_scope','events', 'NEW.supersedes_seq IS NOT NULL AND (NEW.supersedes_seq>=NEW.seq OR NEW.run_id!=(SELECT run_id FROM events WHERE seq=NEW.supersedes_seq))', 'invalid correction scope')
# Bind direct run-bearing entity references to the fact event run.
for name, cols in TABLES.items():
    if not any(c.startswith('event_seq ') for c in cols): continue
    for col in cols:
        match=re.search(r'^(\w+) TEXT .*REFERENCES (\w+)\(id\)',col)
        if match and any(c.startswith('run_id ') for c in TABLES[match[2]]):
            field,target=match.groups()
            trigger(f'{name}_{field}_entity_run',name,f'NEW.{field} IS NOT NULL AND (SELECT run_id FROM events WHERE seq=NEW.event_seq)!=(SELECT run_id FROM {target} WHERE id=NEW.{field})','cross-run entity')
# Facts that refer to other event facts must belong to the same run.
for name, cols in TABLES.items():
    if not any(c.startswith('event_seq ') for c in cols): continue
    for col in cols:
        match=re.search(r'^(\w+) TEXT .*REFERENCES (\w+)\(id\)',col)
        if match and any(c.startswith('event_seq ') for c in TABLES[match[2]]):
            field,target=match.groups()
            trigger(f'{name}_{field}_run',name,f'NEW.{field} IS NOT NULL AND (SELECT run_id FROM events WHERE seq=NEW.event_seq)!=(SELECT e.run_id FROM {target} t JOIN events e ON e.seq=t.event_seq WHERE t.id=NEW.{field})','cross-run fact')
sql += [
'CREATE INDEX events_run_order ON events(run_id,seq);',
'CREATE INDEX events_type_time ON events(event_type,occurred_at_us);',
'CREATE INDEX hunger_gap ON hunger_events(gap_hash,event_seq);',
'''CREATE VIEW invocation_summary AS SELECT a.id AS attempt_id,a.invocation_id,a.attempt_no,
 EXISTS(SELECT 1 FROM invocation_events i WHERE i.attempt_id=a.id AND i.phase='dispatched') AS did_dispatch,
 EXISTS(SELECT 1 FROM invocation_events i WHERE i.attempt_id=a.id AND i.phase='started') AS did_start,
 (SELECT status FROM invocation_events i WHERE i.attempt_id=a.id ORDER BY event_seq DESC,id DESC LIMIT 1) AS execution_status,
 (SELECT result FROM acceptance_checks c WHERE c.attempt_id=a.id ORDER BY event_seq DESC,id DESC LIMIT 1) AS latest_check
 FROM attempts a;''',
'''CREATE VIEW kv_usage_extrema AS SELECT k.cache_id,MIN(k.used_bytes) AS sampled_min_bytes,MAX(k.used_bytes) AS sampled_max_bytes,
 MIN(k.used_tokens) AS sampled_min_tokens,MAX(k.used_tokens) AS sampled_max_tokens,COUNT(*) AS samples,
 MIN(e.occurred_at_us) AS first_sample_us,MAX(e.occurred_at_us) AS last_sample_us
 FROM kv_samples k JOIN events e ON e.seq=k.event_seq GROUP BY k.cache_id;''',
'''CREATE VIEW hunger_summary AS SELECT gap_hash,SUM(action='observed') AS hits,
 MAX(depth) AS max_depth,MIN(e.occurred_at_us) AS first_seen_us,MAX(e.occurred_at_us) AS last_seen_us,
 (SELECT action FROM hunger_events h2 WHERE h2.gap_hash=h.gap_hash ORDER BY event_seq DESC,id DESC LIMIT 1) AS latest_action
 FROM hunger_events h JOIN events e ON e.seq=h.event_seq GROUP BY gap_hash;''',
'''CREATE VIEW resource_by_actor AS SELECT e.run_id,e.actor_id,r.metric,r.quality,SUM(r.value) AS delta_total,COUNT(*) AS samples
 FROM resource_samples r JOIN events e ON e.seq=r.event_seq WHERE r.sample_kind='delta' AND r.allocation_id IS NULL
 GROUP BY e.run_id,e.actor_id,r.metric,r.quality;''',
'''CREATE VIEW atom_activity AS SELECT e.run_id,o.operation,COUNT(*) AS operations,COUNT(DISTINCT o.atom_id) AS distinct_atoms
 FROM atom_operations o JOIN events e ON e.seq=o.event_seq GROUP BY e.run_id,o.operation;''']
Path(__file__).with_name('migrations').joinpath('001_initial.sql').write_text('\n\n'.join(sql)+'\n')
print(f'Generated {len(TABLES)+1} tables')
