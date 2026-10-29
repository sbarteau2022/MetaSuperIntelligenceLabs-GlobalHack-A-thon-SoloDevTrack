---
title: "N³ SQL data dictionary"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-29T20:54:29.675284+00:00"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files: []
related_issues: []
related_documents: ["README.md"]
tags: [n3, observability, sqlite]
---


# Schema dictionary

Generated from the initialized SQLite schema. See `README.md` for domain
semantics, units, evidence requirements and integration limits.

All tables are STRICT and append-only. Every foreign key is indexed.
IDs are opaque; artifact content and secrets remain outside this database.
Schema migrations are the authoritative executable specification.

## `acceptance_checks`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `attempt_id` | TEXT | no | attempts.id |
| `criteria_id` | TEXT | yes | artifacts.id |
| `result` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |
| `checker_version_hash` | TEXT | yes | — |

```sql
CREATE TABLE acceptance_checks (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  attempt_id TEXT  REFERENCES attempts(id),
  criteria_id TEXT NOT NULL REFERENCES artifacts(id),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','error','skipped','unknown')),
  evidence_id TEXT  REFERENCES artifacts(id),
  checker_version_hash TEXT NOT NULL CHECK(checker_version_hash IS NULL OR (length(checker_version_hash)=64 AND checker_version_hash NOT GLOB '*[^0-9a-f]*')),
  CHECK(result!='pass' OR evidence_id IS NOT NULL)
) STRICT;
```

## `actors`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `project_id` | TEXT | yes | projects.id |
| `kind` | TEXT | yes | — |
| `identity_hash` | TEXT | yes | — |

```sql
CREATE TABLE actors (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id),
  kind TEXT NOT NULL CHECK(kind IN ('meta','swarm','agent','human','system')),
  identity_hash TEXT NOT NULL CHECK(identity_hash IS NULL OR (length(identity_hash)=64 AND identity_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `agent_leases`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `decision_id` | TEXT | yes | meta_decisions.id |
| `agent_id` | TEXT | yes | actors.id |
| `swarm_id` | TEXT | yes | actors.id |
| `reserved_cost_microusd` | INTEGER | yes | — |
| `expires_at_us` | INTEGER | yes | — |

```sql
CREATE TABLE agent_leases (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  decision_id TEXT NOT NULL REFERENCES meta_decisions(id),
  agent_id TEXT NOT NULL REFERENCES actors(id),
  swarm_id TEXT NOT NULL REFERENCES actors(id),
  reserved_cost_microusd INTEGER NOT NULL CHECK(reserved_cost_microusd>=0),
  expires_at_us INTEGER NOT NULL CHECK(expires_at_us>=0),
  UNIQUE(decision_id,agent_id)
) STRICT;
```

## `alert_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `evaluation_id` | TEXT | yes | slo_evaluations.id |
| `state` | TEXT | yes | — |
| `severity` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE alert_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  evaluation_id TEXT NOT NULL REFERENCES slo_evaluations(id),
  state TEXT NOT NULL CHECK(state IN ('raised','acknowledged','resolved','suppressed')),
  severity TEXT NOT NULL CHECK(severity IN ('info','warning','critical')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `allocations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `owner_id` | TEXT | yes | actors.id |
| `device_hash` | TEXT | yes | — |
| `kind` | TEXT | yes | — |
| `accounting` | TEXT | yes | — |

```sql
CREATE TABLE allocations (
  id TEXT PRIMARY KEY NOT NULL,
  owner_id TEXT NOT NULL REFERENCES actors(id),
  device_hash TEXT NOT NULL CHECK(device_hash IS NULL OR (length(device_hash)=64 AND device_hash NOT GLOB '*[^0-9a-f]*')),
  kind TEXT NOT NULL CHECK(kind IN ('weights','kv','process','gpu_buffer','host_buffer','shared_pool')),
  accounting TEXT NOT NULL CHECK(accounting IN ('exclusive','shared'))
) STRICT;
```

## `approvals`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `subject_id` | TEXT | yes | artifacts.id |
| `actor_id` | TEXT | yes | actors.id |
| `decision` | TEXT | yes | — |
| `policy_id` | TEXT | yes | artifacts.id |
| `expires_at_us` | INTEGER | no | — |

```sql
CREATE TABLE approvals (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  subject_id TEXT NOT NULL REFERENCES artifacts(id),
  actor_id TEXT NOT NULL REFERENCES actors(id),
  decision TEXT NOT NULL CHECK(decision IN ('approve','deny','revoke','expire')),
  policy_id TEXT NOT NULL REFERENCES artifacts(id),
  expires_at_us INTEGER CHECK(expires_at_us>=0)
) STRICT;
```

## `artifacts`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `content_hash` | TEXT | yes | — |
| `byte_count` | INTEGER | yes | — |
| `classification` | TEXT | yes | — |
| `availability` | TEXT | yes | — |

```sql
CREATE TABLE artifacts (
  id TEXT PRIMARY KEY NOT NULL,
  content_hash TEXT NOT NULL CHECK(content_hash IS NULL OR (length(content_hash)=64 AND content_hash NOT GLOB '*[^0-9a-f]*')),
  byte_count INTEGER NOT NULL CHECK(byte_count>=0),
  classification TEXT NOT NULL CHECK(classification IN ('metadata','restricted','secret')),
  availability TEXT NOT NULL CHECK(availability IN ('available','withheld','erased','unknown'))
) STRICT;
```

## `atom_operations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `atom_id` | TEXT | yes | atoms.id |
| `revision_id` | TEXT | yes | atom_revisions.id |
| `operation` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE atom_operations (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  atom_id TEXT NOT NULL REFERENCES atoms(id),
  revision_id TEXT NOT NULL REFERENCES atom_revisions(id),
  operation TEXT NOT NULL CHECK(operation IN ('retrieved','presented','cited','verified_used','added','reinforced','updated','merged','rejected','pinned','decayed')),
  evidence_id TEXT  REFERENCES artifacts(id),
  CHECK(operation!='verified_used' OR evidence_id IS NOT NULL)
) STRICT;
```

## `atom_revisions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `atom_id` | TEXT | yes | atoms.id |
| `content_id` | TEXT | yes | artifacts.id |
| `prior_revision_id` | TEXT | no | atom_revisions.id |
| `rank` | INTEGER | yes | — |
| `embedding_hash` | TEXT | yes | — |
| `method_version_hash` | TEXT | yes | — |
| `judge_check_id` | TEXT | yes | acceptance_checks.id |
| `salience` | REAL | yes | — |

```sql
CREATE TABLE atom_revisions (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  atom_id TEXT NOT NULL REFERENCES atoms(id),
  content_id TEXT NOT NULL REFERENCES artifacts(id),
  prior_revision_id TEXT  REFERENCES atom_revisions(id),
  rank INTEGER NOT NULL CHECK(rank>=0),
  embedding_hash TEXT NOT NULL CHECK(embedding_hash IS NULL OR (length(embedding_hash)=64 AND embedding_hash NOT GLOB '*[^0-9a-f]*')),
  method_version_hash TEXT NOT NULL CHECK(method_version_hash IS NULL OR (length(method_version_hash)=64 AND method_version_hash NOT GLOB '*[^0-9a-f]*')),
  judge_check_id TEXT NOT NULL REFERENCES acceptance_checks(id),
  salience REAL NOT NULL CHECK(salience BETWEEN 0 AND 1)
) STRICT;
```

## `atoms`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `project_id` | TEXT | yes | projects.id |
| `source_atom_id_hash` | TEXT | yes | — |

```sql
CREATE TABLE atoms (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id),
  source_atom_id_hash TEXT NOT NULL CHECK(source_atom_id_hash IS NULL OR (length(source_atom_id_hash)=64 AND source_atom_id_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `attempts`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `invocation_id` | TEXT | yes | invocations.id |
| `attempt_no` | INTEGER | yes | — |
| `strategy_hash` | TEXT | yes | — |

```sql
CREATE TABLE attempts (
  id TEXT PRIMARY KEY NOT NULL,
  invocation_id TEXT NOT NULL REFERENCES invocations(id),
  attempt_no INTEGER NOT NULL CHECK(attempt_no>0),
  strategy_hash TEXT NOT NULL CHECK(strategy_hash IS NULL OR (length(strategy_hash)=64 AND strategy_hash NOT GLOB '*[^0-9a-f]*')),
  UNIQUE(invocation_id,attempt_no)
) STRICT;
```

## `budget_breaches`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `limit_id` | TEXT | yes | budget_limits.id |
| `observed_value` | INTEGER | yes | — |
| `action` | TEXT | yes | — |
| `evidence_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE budget_breaches (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  limit_id TEXT NOT NULL REFERENCES budget_limits(id),
  observed_value INTEGER NOT NULL CHECK(observed_value>=0),
  action TEXT NOT NULL CHECK(action IN ('denied','stopped','released','escalated','unknown')),
  evidence_id TEXT NOT NULL REFERENCES artifacts(id)
) STRICT;
```

## `budget_limits`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `decision_id` | TEXT | yes | meta_decisions.id |
| `resource` | TEXT | yes | — |
| `hard_limit` | INTEGER | yes | — |
| `soft_limit` | INTEGER | yes | — |
| `on_breach` | TEXT | yes | — |

```sql
CREATE TABLE budget_limits (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  decision_id TEXT NOT NULL REFERENCES meta_decisions(id),
  resource TEXT NOT NULL CHECK(resource IN ('cost_microusd','tokens','cpu_ns','gpu_ns','memory_bytes','energy_uj','wall_ns','idle_ns')),
  hard_limit INTEGER NOT NULL CHECK(hard_limit>=0),
  soft_limit INTEGER NOT NULL CHECK(soft_limit>=0 AND soft_limit<=hard_limit),
  on_breach TEXT NOT NULL CHECK(on_breach IN ('deny','stop','release','escalate'))
) STRICT;
```

## `caches`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `owner_id` | TEXT | yes | actors.id |
| `identity_hash` | TEXT | yes | — |
| `kind` | TEXT | yes | — |
| `capacity_bytes` | INTEGER | yes | — |
| `capacity_tokens` | INTEGER | no | — |

```sql
CREATE TABLE caches (
  id TEXT PRIMARY KEY NOT NULL,
  owner_id TEXT NOT NULL REFERENCES actors(id),
  identity_hash TEXT NOT NULL CHECK(identity_hash IS NULL OR (length(identity_hash)=64 AND identity_hash NOT GLOB '*[^0-9a-f]*')),
  kind TEXT NOT NULL CHECK(kind IN ('application_kv','transformer_kv','prefix','radix','retrieval')),
  capacity_bytes INTEGER NOT NULL CHECK(capacity_bytes>=0),
  capacity_tokens INTEGER CHECK(capacity_tokens>=0)
) STRICT;
```

## `capabilities`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `kind` | TEXT | yes | — |
| `definition_hash` | TEXT | yes | — |
| `version_hash` | TEXT | yes | — |
| `parent_id` | TEXT | no | capabilities.id |

```sql
CREATE TABLE capabilities (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('tool','skill','hook','mcp_server','mcp_method','plugin','resource','prompt')),
  definition_hash TEXT NOT NULL CHECK(definition_hash IS NULL OR (length(definition_hash)=64 AND definition_hash NOT GLOB '*[^0-9a-f]*')),
  version_hash TEXT NOT NULL CHECK(version_hash IS NULL OR (length(version_hash)=64 AND version_hash NOT GLOB '*[^0-9a-f]*')),
  parent_id TEXT  REFERENCES capabilities(id)
) STRICT;
```

## `capability_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `capability_id` | TEXT | yes | capabilities.id |
| `action` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE capability_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  capability_id TEXT NOT NULL REFERENCES capabilities(id),
  action TEXT NOT NULL CHECK(action IN ('registered','resolved','enabled','disabled','denied','uninstalled')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `claims`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `subject_id` | TEXT | yes | artifacts.id |
| `label` | TEXT | yes | — |
| `experiment_id` | TEXT | no | experiments.id |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE claims (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  subject_id TEXT NOT NULL REFERENCES artifacts(id),
  label TEXT NOT NULL CHECK(label IN ('VERIFIED','REPORTED','INFERRED','PROPOSED')),
  experiment_id TEXT  REFERENCES experiments(id),
  evidence_id TEXT  REFERENCES artifacts(id),
  CHECK(label!='VERIFIED' OR (experiment_id IS NOT NULL AND evidence_id IS NOT NULL))
) STRICT;
```

## `config_snapshots`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `artifact_id` | TEXT | yes | artifacts.id |
| `source_revision_hash` | TEXT | yes | — |

```sql
CREATE TABLE config_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  artifact_id TEXT NOT NULL REFERENCES artifacts(id),
  source_revision_hash TEXT NOT NULL CHECK(source_revision_hash IS NULL OR (length(source_revision_hash)=64 AND source_revision_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `context_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `model_id` | TEXT | yes | model_instances.id |
| `action` | TEXT | yes | — |
| `used_tokens` | INTEGER | yes | — |
| `limit_tokens` | INTEGER | yes | — |
| `checkpoint_id` | TEXT | no | artifacts.id |
| `invariants_id` | TEXT | yes | artifacts.id |
| `result` | TEXT | yes | — |

```sql
CREATE TABLE context_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  model_id TEXT NOT NULL REFERENCES model_instances(id),
  action TEXT NOT NULL CHECK(action IN ('assemble','budget','trim','compact','checkpoint','restore','pin','unpin')),
  used_tokens INTEGER NOT NULL CHECK(used_tokens>=0),
  limit_tokens INTEGER NOT NULL CHECK(limit_tokens>0 AND used_tokens<=limit_tokens),
  checkpoint_id TEXT  REFERENCES artifacts(id),
  invariants_id TEXT NOT NULL REFERENCES artifacts(id),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','deferred','unknown'))
) STRICT;
```

## `context_sources`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `context_event_id` | TEXT | yes | context_events.id |
| `source_id` | TEXT | yes | artifacts.id |
| `layer` | TEXT | yes | — |
| `token_count` | INTEGER | yes | — |
| `pinned` | INTEGER | yes | — |

```sql
CREATE TABLE context_sources (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  context_event_id TEXT NOT NULL REFERENCES context_events(id),
  source_id TEXT NOT NULL REFERENCES artifacts(id),
  layer TEXT NOT NULL CHECK(layer IN ('org_policy','user','project','session','retrieved')),
  token_count INTEGER NOT NULL CHECK(token_count>=0),
  pinned INTEGER NOT NULL CHECK(pinned IN(0,1))
) STRICT;
```

## `dag_transitions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `node_id` | TEXT | yes | graph_nodes.id |
| `state` | TEXT | yes | — |
| `span_id` | TEXT | no | spans.id |
| `reason_hash` | TEXT | no | — |

```sql
CREATE TABLE dag_transitions (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  node_id TEXT NOT NULL REFERENCES graph_nodes(id),
  state TEXT NOT NULL CHECK(state IN ('blocked','ready','queued','running','succeeded','failed','cancelled','skipped')),
  span_id TEXT  REFERENCES spans(id),
  reason_hash TEXT   CHECK(reason_hash IS NULL OR (length(reason_hash)=64 AND reason_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `daily_rollups`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `day_utc` | INTEGER | yes | — |
| `aggregate_id` | TEXT | yes | artifacts.id |
| `first_seq` | INTEGER | yes | events.seq |
| `last_seq` | INTEGER | yes | events.seq |
| `event_count` | INTEGER | yes | — |

```sql
CREATE TABLE daily_rollups (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  day_utc INTEGER NOT NULL CHECK(day_utc>=19700101),
  aggregate_id TEXT NOT NULL REFERENCES artifacts(id),
  first_seq INTEGER NOT NULL REFERENCES events(seq),
  last_seq INTEGER NOT NULL REFERENCES events(seq),
  event_count INTEGER NOT NULL CHECK(event_count>=0),
  CHECK(last_seq>=first_seq)
) STRICT;
```

## `degradation_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `component_hash` | TEXT | yes | — |
| `reason_hash` | TEXT | yes | — |
| `fallback_hash` | TEXT | yes | — |
| `receipt_id` | TEXT | yes | artifacts.id |
| `state` | TEXT | yes | — |

```sql
CREATE TABLE degradation_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  component_hash TEXT NOT NULL CHECK(component_hash IS NULL OR (length(component_hash)=64 AND component_hash NOT GLOB '*[^0-9a-f]*')),
  reason_hash TEXT NOT NULL CHECK(reason_hash IS NULL OR (length(reason_hash)=64 AND reason_hash NOT GLOB '*[^0-9a-f]*')),
  fallback_hash TEXT NOT NULL CHECK(fallback_hash IS NULL OR (length(fallback_hash)=64 AND fallback_hash NOT GLOB '*[^0-9a-f]*')),
  receipt_id TEXT NOT NULL REFERENCES artifacts(id),
  state TEXT NOT NULL CHECK(state IN ('entered','continued','escalated','recovered'))
) STRICT;
```

## `distillation_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `dataset_hash` | TEXT | yes | — |
| `teacher_id` | TEXT | no | model_instances.id |
| `student_id` | TEXT | no | model_instances.id |
| `stage` | TEXT | yes | — |
| `check_id` | TEXT | no | acceptance_checks.id |
| `samples_count` | INTEGER | yes | — |

```sql
CREATE TABLE distillation_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  dataset_hash TEXT NOT NULL CHECK(dataset_hash IS NULL OR (length(dataset_hash)=64 AND dataset_hash NOT GLOB '*[^0-9a-f]*')),
  teacher_id TEXT  REFERENCES model_instances(id),
  student_id TEXT  REFERENCES model_instances(id),
  stage TEXT NOT NULL CHECK(stage IN ('preregister','carve','generate','judge','consolidate','admit','reject')),
  check_id TEXT  REFERENCES acceptance_checks(id),
  samples_count INTEGER NOT NULL CHECK(samples_count>=0)
) STRICT;
```

## `events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `seq` | INTEGER | yes | — |
| `id` | TEXT | yes | — |
| `run_id` | TEXT | yes | runs.id |
| `span_id` | TEXT | no | spans.id |
| `actor_id` | TEXT | no | actors.id |
| `producer_id` | TEXT | yes | — |
| `producer_seq` | INTEGER | yes | — |
| `occurred_at_us` | INTEGER | yes | — |
| `boot_id` | TEXT | yes | — |
| `monotonic_ns` | INTEGER | yes | — |
| `ingested_at_us` | INTEGER | yes | — |
| `event_type` | TEXT | yes | — |
| `supersedes_seq` | INTEGER | no | events.seq |
| `body_hash` | TEXT | yes | — |
| `previous_hash` | TEXT | yes | — |
| `event_hash` | TEXT | yes | — |

```sql
CREATE TABLE events (
  seq INTEGER PRIMARY KEY,
  id TEXT NOT NULL UNIQUE,
  run_id TEXT NOT NULL REFERENCES runs(id),
  span_id TEXT  REFERENCES spans(id),
  actor_id TEXT  REFERENCES actors(id),
  producer_id TEXT NOT NULL,
  producer_seq INTEGER NOT NULL CHECK(producer_seq>=0),
  occurred_at_us INTEGER NOT NULL CHECK(occurred_at_us>=0),
  boot_id TEXT NOT NULL,
  monotonic_ns INTEGER NOT NULL CHECK(monotonic_ns>=0),
  ingested_at_us INTEGER NOT NULL CHECK(ingested_at_us>=0),
  event_type TEXT NOT NULL CHECK(event_type IN ('lifecycle','invocation','check','recovery','decision','lease','resource','inference','graph','knowledge','kv','context','atom','retrieval','rerank','novelty','memory','hunger','security','execution','research','service','telemetry','governance')),
  supersedes_seq INTEGER REFERENCES events(seq),
  body_hash TEXT NOT NULL CHECK(body_hash IS NULL OR (length(body_hash)=64 AND body_hash NOT GLOB '*[^0-9a-f]*')),
  previous_hash TEXT NOT NULL CHECK(previous_hash IS NULL OR (length(previous_hash)=64 AND previous_hash NOT GLOB '*[^0-9a-f]*')),
  event_hash TEXT NOT NULL CHECK(event_hash IS NULL OR (length(event_hash)=64 AND event_hash NOT GLOB '*[^0-9a-f]*')),
  UNIQUE(producer_id,producer_seq)
) STRICT;
```

## `experiment_results`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `experiment_id` | TEXT | yes | experiments.id |
| `check_id` | TEXT | yes | acceptance_checks.id |
| `evidence_id` | TEXT | yes | artifacts.id |
| `arm` | TEXT | yes | — |

```sql
CREATE TABLE experiment_results (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  experiment_id TEXT NOT NULL REFERENCES experiments(id),
  check_id TEXT NOT NULL REFERENCES acceptance_checks(id),
  evidence_id TEXT NOT NULL REFERENCES artifacts(id),
  arm TEXT NOT NULL CHECK(arm IN ('control','candidate','ablation','pressure','adversarial'))
) STRICT;
```

## `experiments`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `project_id` | TEXT | yes | projects.id |
| `preregistration_id` | TEXT | yes | artifacts.id |
| `workload_hash` | TEXT | yes | — |
| `hardware_hash` | TEXT | yes | — |
| `baseline_hash` | TEXT | yes | — |
| `config_id` | TEXT | yes | config_snapshots.id |

```sql
CREATE TABLE experiments (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id),
  preregistration_id TEXT NOT NULL REFERENCES artifacts(id),
  workload_hash TEXT NOT NULL CHECK(workload_hash IS NULL OR (length(workload_hash)=64 AND workload_hash NOT GLOB '*[^0-9a-f]*')),
  hardware_hash TEXT NOT NULL CHECK(hardware_hash IS NULL OR (length(hardware_hash)=64 AND hardware_hash NOT GLOB '*[^0-9a-f]*')),
  baseline_hash TEXT NOT NULL CHECK(baseline_hash IS NULL OR (length(baseline_hash)=64 AND baseline_hash NOT GLOB '*[^0-9a-f]*')),
  config_id TEXT NOT NULL REFERENCES config_snapshots(id)
) STRICT;
```

## `governance_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `operation` | TEXT | yes | — |
| `subject_id` | TEXT | yes | artifacts.id |
| `policy_id` | TEXT | yes | artifacts.id |
| `result` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE governance_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  operation TEXT NOT NULL CHECK(operation IN ('config','release','migration','backup','restore','export','retention','erasure','legal_hold','access')),
  subject_id TEXT NOT NULL REFERENCES artifacts(id),
  policy_id TEXT NOT NULL REFERENCES artifacts(id),
  result TEXT NOT NULL CHECK(result IN ('requested','pass','fail','denied','deferred')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `graph_crosslinks`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `source_id` | TEXT | yes | graph_nodes.id |
| `target_id` | TEXT | yes | graph_nodes.id |
| `relation` | TEXT | yes | — |

```sql
CREATE TABLE graph_crosslinks (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  source_id TEXT NOT NULL REFERENCES graph_nodes(id),
  target_id TEXT NOT NULL REFERENCES graph_nodes(id),
  relation TEXT NOT NULL CHECK(relation IN ('informed_by','produced','evidenced_by'))
) STRICT;
```

## `graph_edges`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `graph_id` | TEXT | yes | graph_versions.id |
| `source_id` | TEXT | yes | graph_nodes.id |
| `target_id` | TEXT | yes | graph_nodes.id |
| `relation` | TEXT | yes | — |

```sql
CREATE TABLE graph_edges (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  graph_id TEXT NOT NULL REFERENCES graph_versions(id),
  source_id TEXT NOT NULL REFERENCES graph_nodes(id),
  target_id TEXT NOT NULL REFERENCES graph_nodes(id),
  relation TEXT NOT NULL CHECK(relation IN ('depends_on','contains','references','derives_from','contradicts','supports','supersedes','links_to')),
  CHECK(source_id!=target_id),
  UNIQUE(graph_id,source_id,target_id,relation)
) STRICT;
```

## `graph_nodes`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `graph_id` | TEXT | yes | graph_versions.id |
| `kind` | TEXT | yes | — |
| `external_id_hash` | TEXT | yes | — |
| `artifact_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE graph_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  graph_id TEXT NOT NULL REFERENCES graph_versions(id),
  kind TEXT NOT NULL CHECK(kind IN ('task','capability','note','concept','atom','artifact','decision','checkpoint')),
  external_id_hash TEXT NOT NULL CHECK(external_id_hash IS NULL OR (length(external_id_hash)=64 AND external_id_hash NOT GLOB '*[^0-9a-f]*')),
  artifact_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `graph_versions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `run_id` | TEXT | yes | runs.id |
| `kind` | TEXT | yes | — |
| `snapshot_hash` | TEXT | yes | — |
| `previous_id` | TEXT | no | graph_versions.id |

```sql
CREATE TABLE graph_versions (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES runs(id),
  kind TEXT NOT NULL CHECK(kind IN ('dag','knowledge','capability')),
  snapshot_hash TEXT NOT NULL CHECK(snapshot_hash IS NULL OR (length(snapshot_hash)=64 AND snapshot_hash NOT GLOB '*[^0-9a-f]*')),
  previous_id TEXT  REFERENCES graph_versions(id)
) STRICT;
```

## `hole_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `hole_id` | TEXT | yes | holes.id |
| `action` | TEXT | yes | — |
| `patch_id` | TEXT | no | artifacts.id |
| `check_id` | TEXT | no | acceptance_checks.id |
| `approver_id` | TEXT | no | actors.id |
| `result` | TEXT | yes | — |

```sql
CREATE TABLE hole_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  hole_id TEXT NOT NULL REFERENCES holes(id),
  action TEXT NOT NULL CHECK(action IN ('create','execute','test','diff','review','approve','reject','merge','destroy')),
  patch_id TEXT  REFERENCES artifacts(id),
  check_id TEXT  REFERENCES acceptance_checks(id),
  approver_id TEXT  REFERENCES actors(id),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','blocked','unknown'))
) STRICT;
```

## `holes`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `run_id` | TEXT | yes | runs.id |
| `sandbox_profile_hash` | TEXT | yes | — |
| `base_revision_hash` | TEXT | yes | — |
| `policy_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE holes (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES runs(id),
  sandbox_profile_hash TEXT NOT NULL CHECK(sandbox_profile_hash IS NULL OR (length(sandbox_profile_hash)=64 AND sandbox_profile_hash NOT GLOB '*[^0-9a-f]*')),
  base_revision_hash TEXT NOT NULL CHECK(base_revision_hash IS NULL OR (length(base_revision_hash)=64 AND base_revision_hash NOT GLOB '*[^0-9a-f]*')),
  policy_id TEXT NOT NULL REFERENCES artifacts(id)
) STRICT;
```

## `hook_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `capability_id` | TEXT | yes | capabilities.id |
| `attempt_id` | TEXT | yes | attempts.id |
| `moment` | TEXT | yes | — |
| `effect` | TEXT | yes | — |
| `context_hash` | TEXT | no | — |

```sql
CREATE TABLE hook_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  capability_id TEXT NOT NULL REFERENCES capabilities(id),
  attempt_id TEXT NOT NULL REFERENCES attempts(id),
  moment TEXT NOT NULL CHECK(moment IN ('before','after','error')),
  effect TEXT NOT NULL CHECK(effect IN ('allow','deny','append_context','skip','error')),
  context_hash TEXT   CHECK(context_hash IS NULL OR (length(context_hash)=64 AND context_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `hunger_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `gap_hash` | TEXT | yes | — |
| `tap` | TEXT | yes | — |
| `action` | TEXT | yes | — |
| `depth` | INTEGER | yes | — |
| `threshold` | REAL | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE hunger_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  gap_hash TEXT NOT NULL CHECK(gap_hash IS NULL OR (length(gap_hash)=64 AND gap_hash NOT GLOB '*[^0-9a-f]*')),
  tap TEXT NOT NULL CHECK(tap IN ('traverse_refused','lexical_thin','query_thin','judge_rejected','uncovered_task')),
  action TEXT NOT NULL CHECK(action IN ('observed','sated','reopened')),
  depth INTEGER NOT NULL CHECK(depth>=0),
  threshold REAL NOT NULL,
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `inference_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `model_id` | TEXT | yes | model_instances.id |
| `phase` | TEXT | yes | — |
| `input_tokens` | INTEGER | no | — |
| `output_tokens` | INTEGER | no | — |
| `ttft_ns` | INTEGER | no | — |
| `duration_ns` | INTEGER | no | — |
| `drafted_tokens` | INTEGER | no | — |
| `accepted_tokens` | INTEGER | no | — |
| `quality_check_id` | TEXT | no | acceptance_checks.id |

```sql
CREATE TABLE inference_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  model_id TEXT NOT NULL REFERENCES model_instances(id),
  phase TEXT NOT NULL CHECK(phase IN ('load','prefill','decode','draft','verify','unload')),
  input_tokens INTEGER CHECK(input_tokens>=0),
  output_tokens INTEGER CHECK(output_tokens>=0),
  ttft_ns INTEGER CHECK(ttft_ns>=0),
  duration_ns INTEGER CHECK(duration_ns>=0),
  drafted_tokens INTEGER CHECK(drafted_tokens>=0),
  accepted_tokens INTEGER CHECK(accepted_tokens>=0 AND accepted_tokens<=drafted_tokens),
  quality_check_id TEXT  REFERENCES acceptance_checks(id)
) STRICT;
```

## `ingest_stages`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `source_id` | TEXT | yes | artifacts.id |
| `stage` | TEXT | yes | — |
| `result` | TEXT | yes | — |
| `output_id` | TEXT | no | artifacts.id |
| `check_id` | TEXT | no | acceptance_checks.id |

```sql
CREATE TABLE ingest_stages (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  source_id TEXT NOT NULL REFERENCES artifacts(id),
  stage TEXT NOT NULL CHECK(stage IN ('parse_5w','condense','judge','mind_tail','deposit','embed','index','deduplicate','quarantine')),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','skipped','unknown')),
  output_id TEXT  REFERENCES artifacts(id),
  check_id TEXT  REFERENCES acceptance_checks(id)
) STRICT;
```

## `invocation_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `attempt_id` | TEXT | yes | attempts.id |
| `phase` | TEXT | yes | — |
| `status` | TEXT | yes | — |
| `error_class_hash` | TEXT | no | — |
| `receipt_id` | TEXT | no | artifacts.id |
| `duration_ns` | INTEGER | no | — |

```sql
CREATE TABLE invocation_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  attempt_id TEXT NOT NULL REFERENCES attempts(id),
  phase TEXT NOT NULL CHECK(phase IN ('requested','dispatched','started','completed')),
  status TEXT NOT NULL CHECK(status IN ('pending','running','succeeded','failed','cancelled','timed_out','denied','unknown')),
  error_class_hash TEXT   CHECK(error_class_hash IS NULL OR (length(error_class_hash)=64 AND error_class_hash NOT GLOB '*[^0-9a-f]*')),
  receipt_id TEXT  REFERENCES artifacts(id),
  duration_ns INTEGER CHECK(duration_ns>=0)
) STRICT;
```

## `invocations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `span_id` | TEXT | yes | spans.id |
| `capability_id` | TEXT | yes | capabilities.id |
| `request_hash` | TEXT | yes | — |
| `max_attempts` | INTEGER | yes | — |

```sql
CREATE TABLE invocations (
  id TEXT PRIMARY KEY NOT NULL,
  span_id TEXT NOT NULL REFERENCES spans(id),
  capability_id TEXT NOT NULL REFERENCES capabilities(id),
  request_hash TEXT NOT NULL CHECK(request_hash IS NULL OR (length(request_hash)=64 AND request_hash NOT GLOB '*[^0-9a-f]*')),
  max_attempts INTEGER NOT NULL CHECK(max_attempts BETWEEN 1 AND 1000)
) STRICT;
```

## `judge_results`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `check_id` | TEXT | yes | acceptance_checks.id |
| `judge_version_hash` | TEXT | yes | — |
| `model_id` | TEXT | no | model_instances.id |
| `kind` | TEXT | yes | — |
| `score` | REAL | no | — |
| `evidence_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE judge_results (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  check_id TEXT NOT NULL REFERENCES acceptance_checks(id),
  judge_version_hash TEXT NOT NULL CHECK(judge_version_hash IS NULL OR (length(judge_version_hash)=64 AND judge_version_hash NOT GLOB '*[^0-9a-f]*')),
  model_id TEXT  REFERENCES model_instances(id),
  kind TEXT NOT NULL CHECK(kind IN ('deterministic','model','human')),
  score REAL,
  evidence_id TEXT NOT NULL REFERENCES artifacts(id)
) STRICT;
```

## `kv_operations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `cache_id` | TEXT | yes | caches.id |
| `identity_hash` | TEXT | yes | — |
| `key_hash` | TEXT | yes | — |
| `operation` | TEXT | yes | — |
| `result` | TEXT | yes | — |
| `bytes_count` | INTEGER | no | — |
| `tokens_count` | INTEGER | no | — |
| `producer_span_id` | TEXT | no | spans.id |

```sql
CREATE TABLE kv_operations (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  cache_id TEXT NOT NULL REFERENCES caches(id),
  identity_hash TEXT NOT NULL CHECK(identity_hash IS NULL OR (length(identity_hash)=64 AND identity_hash NOT GLOB '*[^0-9a-f]*')),
  key_hash TEXT NOT NULL CHECK(key_hash IS NULL OR (length(key_hash)=64 AND key_hash NOT GLOB '*[^0-9a-f]*')),
  operation TEXT NOT NULL CHECK(operation IN ('lookup','read','write','reuse','evict','invalidate','allocate','release')),
  result TEXT NOT NULL CHECK(result IN ('hit','miss','stored','rejected','evicted','error','unknown')),
  bytes_count INTEGER CHECK(bytes_count>=0),
  tokens_count INTEGER CHECK(tokens_count>=0),
  producer_span_id TEXT  REFERENCES spans(id)
) STRICT;
```

## `kv_samples`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `cache_id` | TEXT | yes | caches.id |
| `used_bytes` | INTEGER | yes | — |
| `used_tokens` | INTEGER | no | — |
| `entries_count` | INTEGER | yes | — |
| `quality` | TEXT | yes | — |

```sql
CREATE TABLE kv_samples (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  cache_id TEXT NOT NULL REFERENCES caches(id),
  used_bytes INTEGER NOT NULL CHECK(used_bytes>=0),
  used_tokens INTEGER CHECK(used_tokens>=0),
  entries_count INTEGER NOT NULL CHECK(entries_count>=0),
  quality TEXT NOT NULL CHECK(quality IN ('measured','estimated'))
) STRICT;
```

## `lease_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `lease_id` | TEXT | yes | agent_leases.id |
| `state` | TEXT | yes | — |
| `idle_ns` | INTEGER | no | — |
| `actual_cost_microusd` | INTEGER | no | — |

```sql
CREATE TABLE lease_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  lease_id TEXT NOT NULL REFERENCES agent_leases(id),
  state TEXT NOT NULL CHECK(state IN ('activated','working','idle','released','expired','failed')),
  idle_ns INTEGER CHECK(idle_ns>=0),
  actual_cost_microusd INTEGER CHECK(actual_cost_microusd>=0)
) STRICT;
```

## `ledger_roots`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `first_seq` | INTEGER | yes | events.seq |
| `last_seq` | INTEGER | yes | events.seq |
| `root_hash` | TEXT | yes | — |
| `algorithm` | TEXT | yes | — |

```sql
CREATE TABLE ledger_roots (
  id TEXT PRIMARY KEY NOT NULL,
  first_seq INTEGER NOT NULL REFERENCES events(seq),
  last_seq INTEGER NOT NULL REFERENCES events(seq),
  root_hash TEXT NOT NULL CHECK(root_hash IS NULL OR (length(root_hash)=64 AND root_hash NOT GLOB '*[^0-9a-f]*')),
  algorithm TEXT NOT NULL CHECK(algorithm='sha256_binary_duplicate_last'),
  CHECK(first_seq>0 AND last_seq>=first_seq),
  UNIQUE(first_seq,last_seq)
) STRICT;
```

## `lifecycle_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `subject_kind` | TEXT | yes | — |
| `subject_id` | TEXT | yes | — |
| `state` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE lifecycle_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  subject_kind TEXT NOT NULL CHECK(subject_kind IN ('run','span','task','session','actor','model')),
  subject_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('requested','queued','started','idle','completed','failed','cancelled','timed_out','abandoned','unknown')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `mcp_exchanges`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `attempt_id` | TEXT | yes | attempts.id |
| `server_id` | TEXT | yes | capabilities.id |
| `request_id_hash` | TEXT | yes | — |
| `transport` | TEXT | yes | — |
| `operation` | TEXT | yes | — |
| `status` | TEXT | yes | — |
| `protocol_code` | INTEGER | no | — |

```sql
CREATE TABLE mcp_exchanges (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  attempt_id TEXT NOT NULL REFERENCES attempts(id),
  server_id TEXT NOT NULL REFERENCES capabilities(id),
  request_id_hash TEXT NOT NULL CHECK(request_id_hash IS NULL OR (length(request_id_hash)=64 AND request_id_hash NOT GLOB '*[^0-9a-f]*')),
  transport TEXT NOT NULL CHECK(transport IN ('stdio','http','sse')),
  operation TEXT NOT NULL CHECK(operation IN ('initialize','list','call','read','subscribe','cancel','notify')),
  status TEXT NOT NULL CHECK(status IN ('sent','received','error','timeout','disconnected')),
  protocol_code INTEGER
) STRICT;
```

## `memory_operations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `operation` | TEXT | yes | — |
| `input_id` | TEXT | no | artifacts.id |
| `output_id` | TEXT | no | artifacts.id |
| `fingerprint_before` | TEXT | yes | — |
| `fingerprint_after` | TEXT | yes | — |
| `result` | TEXT | yes | — |
| `atom_count` | INTEGER | no | — |
| `invariants_preserved` | INTEGER | no | — |

```sql
CREATE TABLE memory_operations (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  operation TEXT NOT NULL CHECK(operation IN ('traverse','basin_create','carve','dissolve','consolidate','checkpoint','snapshot','recover','passport','reweight')),
  input_id TEXT  REFERENCES artifacts(id),
  output_id TEXT  REFERENCES artifacts(id),
  fingerprint_before TEXT NOT NULL CHECK(fingerprint_before IS NULL OR (length(fingerprint_before)=64 AND fingerprint_before NOT GLOB '*[^0-9a-f]*')),
  fingerprint_after TEXT NOT NULL CHECK(fingerprint_after IS NULL OR (length(fingerprint_after)=64 AND fingerprint_after NOT GLOB '*[^0-9a-f]*')),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','deferred','unknown')),
  atom_count INTEGER CHECK(atom_count>=0),
  invariants_preserved INTEGER CHECK(invariants_preserved IN(0,1))
) STRICT;
```

## `messages`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `sender_id` | TEXT | yes | actors.id |
| `receiver_id` | TEXT | yes | actors.id |
| `message_hash` | TEXT | yes | — |
| `state` | TEXT | yes | — |
| `byte_count` | INTEGER | yes | — |

```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  sender_id TEXT NOT NULL REFERENCES actors(id),
  receiver_id TEXT NOT NULL REFERENCES actors(id),
  message_hash TEXT NOT NULL CHECK(message_hash IS NULL OR (length(message_hash)=64 AND message_hash NOT GLOB '*[^0-9a-f]*')),
  state TEXT NOT NULL CHECK(state IN ('sent','delivered','acknowledged','rejected','dropped')),
  byte_count INTEGER NOT NULL CHECK(byte_count>=0)
) STRICT;
```

## `meta_decisions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `meta_actor_id` | TEXT | yes | actors.id |
| `decision` | TEXT | yes | — |
| `justification_id` | TEXT | yes | artifacts.id |
| `estimated_cost_microusd` | INTEGER | yes | — |
| `budget_microusd` | INTEGER | yes | — |
| `max_agents` | INTEGER | yes | — |
| `expires_at_us` | INTEGER | yes | — |
| `approval` | TEXT | yes | — |
| `policy_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE meta_decisions (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  meta_actor_id TEXT NOT NULL REFERENCES actors(id),
  decision TEXT NOT NULL CHECK(decision IN ('activate','deny','defer','release','route','compact','distill')),
  justification_id TEXT NOT NULL REFERENCES artifacts(id),
  estimated_cost_microusd INTEGER NOT NULL CHECK(estimated_cost_microusd>=0),
  budget_microusd INTEGER NOT NULL CHECK(budget_microusd>=estimated_cost_microusd),
  max_agents INTEGER NOT NULL CHECK(max_agents>=0),
  expires_at_us INTEGER NOT NULL CHECK(expires_at_us>=0),
  approval TEXT NOT NULL CHECK(approval IN ('approved','denied','pending')),
  policy_id TEXT NOT NULL REFERENCES artifacts(id)
) STRICT;
```

## `model_instances`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `actor_id` | TEXT | yes | actors.id |
| `weights_hash` | TEXT | yes | — |
| `tokenizer_hash` | TEXT | yes | — |
| `config_id` | TEXT | yes | config_snapshots.id |
| `backend_hash` | TEXT | yes | — |
| `device_hash` | TEXT | yes | — |
| `context_capacity` | INTEGER | yes | — |

```sql
CREATE TABLE model_instances (
  id TEXT PRIMARY KEY NOT NULL,
  actor_id TEXT NOT NULL REFERENCES actors(id),
  weights_hash TEXT NOT NULL CHECK(weights_hash IS NULL OR (length(weights_hash)=64 AND weights_hash NOT GLOB '*[^0-9a-f]*')),
  tokenizer_hash TEXT NOT NULL CHECK(tokenizer_hash IS NULL OR (length(tokenizer_hash)=64 AND tokenizer_hash NOT GLOB '*[^0-9a-f]*')),
  config_id TEXT NOT NULL REFERENCES config_snapshots(id),
  backend_hash TEXT NOT NULL CHECK(backend_hash IS NULL OR (length(backend_hash)=64 AND backend_hash NOT GLOB '*[^0-9a-f]*')),
  device_hash TEXT NOT NULL CHECK(device_hash IS NULL OR (length(device_hash)=64 AND device_hash NOT GLOB '*[^0-9a-f]*')),
  context_capacity INTEGER NOT NULL CHECK(context_capacity>0)
) STRICT;
```

## `model_routing`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `decision_id` | TEXT | yes | meta_decisions.id |
| `model_id` | TEXT | yes | model_instances.id |
| `action` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE model_routing (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  decision_id TEXT NOT NULL REFERENCES meta_decisions(id),
  model_id TEXT NOT NULL REFERENCES model_instances(id),
  action TEXT NOT NULL CHECK(action IN ('selected','rejected','fallback','loaded','reused','unloaded')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `novelty_assessments`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `candidate_id` | TEXT | yes | artifacts.id |
| `comparison_set_hash` | TEXT | yes | — |
| `method_version_hash` | TEXT | yes | — |
| `score` | REAL | yes | — |
| `threshold` | REAL | yes | — |
| `decision` | TEXT | yes | — |
| `admitted_atom_id` | TEXT | no | atoms.id |
| `evidence_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE novelty_assessments (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  candidate_id TEXT NOT NULL REFERENCES artifacts(id),
  comparison_set_hash TEXT NOT NULL CHECK(comparison_set_hash IS NULL OR (length(comparison_set_hash)=64 AND comparison_set_hash NOT GLOB '*[^0-9a-f]*')),
  method_version_hash TEXT NOT NULL CHECK(method_version_hash IS NULL OR (length(method_version_hash)=64 AND method_version_hash NOT GLOB '*[^0-9a-f]*')),
  score REAL NOT NULL,
  threshold REAL NOT NULL,
  decision TEXT NOT NULL CHECK(decision IN ('admit','reinforce','reject','defer')),
  admitted_atom_id TEXT  REFERENCES atoms(id),
  evidence_id TEXT NOT NULL REFERENCES artifacts(id),
  CHECK(decision!='admit' OR admitted_atom_id IS NOT NULL)
) STRICT;
```

## `objective_outcomes`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `check_id` | TEXT | yes | acceptance_checks.id |
| `result` | TEXT | yes | — |

```sql
CREATE TABLE objective_outcomes (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  check_id TEXT NOT NULL REFERENCES acceptance_checks(id),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','unknown'))
) STRICT;
```

## `obsidian_sync`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `graph_id` | TEXT | yes | graph_versions.id |
| `vault_hash` | TEXT | yes | — |
| `manifest_hash` | TEXT | yes | — |
| `direction` | TEXT | yes | — |
| `result` | TEXT | yes | — |
| `nodes_count` | INTEGER | yes | — |
| `edges_count` | INTEGER | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE obsidian_sync (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  graph_id TEXT NOT NULL REFERENCES graph_versions(id),
  vault_hash TEXT NOT NULL CHECK(vault_hash IS NULL OR (length(vault_hash)=64 AND vault_hash NOT GLOB '*[^0-9a-f]*')),
  manifest_hash TEXT NOT NULL CHECK(manifest_hash IS NULL OR (length(manifest_hash)=64 AND manifest_hash NOT GLOB '*[^0-9a-f]*')),
  direction TEXT NOT NULL CHECK(direction IN ('import','export','reconcile')),
  result TEXT NOT NULL CHECK(result IN ('pass','fail','conflict','unknown')),
  nodes_count INTEGER NOT NULL CHECK(nodes_count>=0),
  edges_count INTEGER NOT NULL CHECK(edges_count>=0),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `policy_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `actor_id` | TEXT | yes | actors.id |
| `policy_id` | TEXT | yes | artifacts.id |
| `capability_token_hash` | TEXT | no | — |
| `gate` | TEXT | yes | — |
| `verdict` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE policy_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  actor_id TEXT NOT NULL REFERENCES actors(id),
  policy_id TEXT NOT NULL REFERENCES artifacts(id),
  capability_token_hash TEXT   CHECK(capability_token_hash IS NULL OR (length(capability_token_hash)=64 AND capability_token_hash NOT GLOB '*[^0-9a-f]*')),
  gate TEXT NOT NULL CHECK(gate IN ('auth','role','capability','sandbox','network','filesystem','approval','egress')),
  verdict TEXT NOT NULL CHECK(verdict IN ('allow','deny','error','unknown')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `projects`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `identity_hash` | TEXT | yes | — |

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY NOT NULL,
  identity_hash TEXT NOT NULL CHECK(identity_hash IS NULL OR (length(identity_hash)=64 AND identity_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `receipts`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `attempt_id` | TEXT | no | attempts.id |
| `receipt_hash` | TEXT | yes | — |
| `artifact_id` | TEXT | yes | artifacts.id |
| `merkle_root` | TEXT | no | — |
| `proof_id` | TEXT | no | artifacts.id |
| `verification` | TEXT | yes | — |

```sql
CREATE TABLE receipts (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  attempt_id TEXT  REFERENCES attempts(id),
  receipt_hash TEXT NOT NULL CHECK(receipt_hash IS NULL OR (length(receipt_hash)=64 AND receipt_hash NOT GLOB '*[^0-9a-f]*')),
  artifact_id TEXT NOT NULL REFERENCES artifacts(id),
  merkle_root TEXT   CHECK(merkle_root IS NULL OR (length(merkle_root)=64 AND merkle_root NOT GLOB '*[^0-9a-f]*')),
  proof_id TEXT  REFERENCES artifacts(id),
  verification TEXT NOT NULL CHECK(verification IN ('verified','failed','unverified'))
) STRICT;
```

## `recovery_edges`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `from_attempt_id` | TEXT | yes | attempts.id |
| `to_attempt_id` | TEXT | yes | attempts.id |
| `kind` | TEXT | yes | — |
| `reason_code_hash` | TEXT | yes | — |

```sql
CREATE TABLE recovery_edges (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  from_attempt_id TEXT NOT NULL REFERENCES attempts(id),
  to_attempt_id TEXT NOT NULL REFERENCES attempts(id),
  kind TEXT NOT NULL CHECK(kind IN ('retry','alternate_tool','alternate_model','escalation','rollback')),
  reason_code_hash TEXT NOT NULL CHECK(reason_code_hash IS NULL OR (length(reason_code_hash)=64 AND reason_code_hash NOT GLOB '*[^0-9a-f]*')),
  CHECK(from_attempt_id!=to_attempt_id)
) STRICT;
```

## `rerank_candidates`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `rerank_id` | TEXT | yes | rerank_runs.id |
| `trajectory_id` | TEXT | no | trajectories.id |
| `revision_id` | TEXT | no | atom_revisions.id |
| `old_rank` | INTEGER | yes | — |
| `new_rank` | INTEGER | yes | — |
| `score` | REAL | yes | — |
| `selected` | INTEGER | yes | — |

```sql
CREATE TABLE rerank_candidates (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  rerank_id TEXT NOT NULL REFERENCES rerank_runs(id),
  trajectory_id TEXT  REFERENCES trajectories(id),
  revision_id TEXT  REFERENCES atom_revisions(id),
  old_rank INTEGER NOT NULL CHECK(old_rank>0),
  new_rank INTEGER NOT NULL CHECK(new_rank>0),
  score REAL NOT NULL,
  selected INTEGER NOT NULL CHECK(selected IN(0,1)),
  CHECK((trajectory_id IS NULL)!=(revision_id IS NULL)),
  UNIQUE(rerank_id,new_rank)
) STRICT;
```

## `rerank_runs`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `query_id` | TEXT | no | retrieval_queries.id |
| `ranker_version_hash` | TEXT | yes | — |
| `criteria_id` | TEXT | yes | artifacts.id |
| `candidate_count` | INTEGER | yes | — |

```sql
CREATE TABLE rerank_runs (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  query_id TEXT  REFERENCES retrieval_queries(id),
  ranker_version_hash TEXT NOT NULL CHECK(ranker_version_hash IS NULL OR (length(ranker_version_hash)=64 AND ranker_version_hash NOT GLOB '*[^0-9a-f]*')),
  criteria_id TEXT NOT NULL REFERENCES artifacts(id),
  candidate_count INTEGER NOT NULL CHECK(candidate_count>=0)
) STRICT;
```

## `resource_samples`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `allocation_id` | TEXT | no | allocations.id |
| `model_id` | TEXT | no | model_instances.id |
| `metric` | TEXT | yes | — |
| `sample_kind` | TEXT | yes | — |
| `quality` | TEXT | yes | — |
| `value` | INTEGER | no | — |
| `interval_start_us` | INTEGER | yes | — |
| `interval_end_us` | INTEGER | yes | — |
| `source_hash` | TEXT | yes | — |

```sql
CREATE TABLE resource_samples (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  allocation_id TEXT  REFERENCES allocations(id),
  model_id TEXT  REFERENCES model_instances(id),
  metric TEXT NOT NULL CHECK(metric IN ('cpu_ns','gpu_ns','ane_ns','rss_bytes','allocated_bytes','peak_bytes','tokens_input','tokens_output','tokens_cached','energy_uj','cost_microusd','io_read_bytes','io_write_bytes','network_bytes','queue_ns','idle_ns','utilization_ppm','thermal_state')),
  sample_kind TEXT NOT NULL CHECK(sample_kind IN ('delta','counter','gauge')),
  quality TEXT NOT NULL CHECK(quality IN ('measured','estimated','unavailable')),
  value INTEGER CHECK(value>=0),
  interval_start_us INTEGER NOT NULL CHECK(interval_start_us>=0),
  interval_end_us INTEGER NOT NULL CHECK(interval_end_us>=interval_start_us),
  source_hash TEXT NOT NULL CHECK(source_hash IS NULL OR (length(source_hash)=64 AND source_hash NOT GLOB '*[^0-9a-f]*')),
  CHECK((quality='unavailable' AND value IS NULL) OR (quality!='unavailable' AND value IS NOT NULL))
) STRICT;
```

## `retrieval_hits`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `query_id` | TEXT | yes | retrieval_queries.id |
| `revision_id` | TEXT | yes | atom_revisions.id |
| `rank` | INTEGER | yes | — |
| `score` | REAL | yes | — |
| `stage` | TEXT | yes | — |

```sql
CREATE TABLE retrieval_hits (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  query_id TEXT NOT NULL REFERENCES retrieval_queries(id),
  revision_id TEXT NOT NULL REFERENCES atom_revisions(id),
  rank INTEGER NOT NULL CHECK(rank>0),
  score REAL NOT NULL,
  stage TEXT NOT NULL CHECK(stage IN ('candidate','reranked','selected','presented')),
  UNIQUE(query_id,stage,rank)
) STRICT;
```

## `retrieval_queries`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `query_hash` | TEXT | yes | — |
| `method_version_hash` | TEXT | yes | — |
| `method` | TEXT | yes | — |
| `requested_k` | INTEGER | yes | — |
| `returned_count` | INTEGER | yes | — |
| `duration_ns` | INTEGER | yes | — |
| `mode` | TEXT | yes | — |
| `degradation_hash` | TEXT | no | — |
| `depth` | INTEGER | no | — |
| `participation_ratio` | REAL | no | — |

```sql
CREATE TABLE retrieval_queries (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  query_hash TEXT NOT NULL CHECK(query_hash IS NULL OR (length(query_hash)=64 AND query_hash NOT GLOB '*[^0-9a-f]*')),
  method_version_hash TEXT NOT NULL CHECK(method_version_hash IS NULL OR (length(method_version_hash)=64 AND method_version_hash NOT GLOB '*[^0-9a-f]*')),
  method TEXT NOT NULL CHECK(method IN ('geometric','lexical','fused','pami','full_scan')),
  requested_k INTEGER NOT NULL CHECK(requested_k>0),
  returned_count INTEGER NOT NULL CHECK(returned_count>=0),
  duration_ns INTEGER NOT NULL CHECK(duration_ns>=0),
  mode TEXT NOT NULL CHECK(mode IN ('primary','fallback','degraded')),
  degradation_hash TEXT   CHECK(degradation_hash IS NULL OR (length(degradation_hash)=64 AND degradation_hash NOT GLOB '*[^0-9a-f]*')),
  depth INTEGER CHECK(depth>=0),
  participation_ratio REAL CHECK(participation_ratio>=0)
) STRICT;
```

## `runs`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `task_id` | TEXT | yes | tasks.id |
| `parent_run_id` | TEXT | no | runs.id |
| `config_id` | TEXT | yes | config_snapshots.id |
| `trace_id` | TEXT | yes | — |

```sql
CREATE TABLE runs (
  id TEXT PRIMARY KEY NOT NULL,
  task_id TEXT NOT NULL REFERENCES tasks(id),
  parent_run_id TEXT  REFERENCES runs(id),
  config_id TEXT NOT NULL REFERENCES config_snapshots(id),
  trace_id TEXT NOT NULL UNIQUE
) STRICT;
```

## `scheduler_decisions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `node_id` | TEXT | yes | graph_nodes.id |
| `actor_id` | TEXT | no | actors.id |
| `action` | TEXT | yes | — |
| `policy_id` | TEXT | yes | artifacts.id |
| `queue_depth` | INTEGER | yes | — |

```sql
CREATE TABLE scheduler_decisions (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  node_id TEXT NOT NULL REFERENCES graph_nodes(id),
  actor_id TEXT  REFERENCES actors(id),
  action TEXT NOT NULL CHECK(action IN ('admit','defer','preempt','isolate','resume','breaker_open','breaker_close')),
  policy_id TEXT NOT NULL REFERENCES artifacts(id),
  queue_depth INTEGER NOT NULL CHECK(queue_depth>=0)
) STRICT;
```

## `schema_migrations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `version` | INTEGER | yes | — |
| `checksum` | TEXT | yes | — |
| `applied_at_us` | INTEGER | yes | — |

```sql
CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at_us INTEGER NOT NULL) STRICT;
```

## `service_events`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `service_hash` | TEXT | yes | — |
| `state` | TEXT | yes | — |
| `reason_hash` | TEXT | yes | — |
| `pid` | INTEGER | no | — |
| `queue_depth` | INTEGER | no | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE service_events (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  service_hash TEXT NOT NULL CHECK(service_hash IS NULL OR (length(service_hash)=64 AND service_hash NOT GLOB '*[^0-9a-f]*')),
  state TEXT NOT NULL CHECK(state IN ('starting','healthy','degraded','unavailable','stopped','recovered')),
  reason_hash TEXT NOT NULL CHECK(reason_hash IS NULL OR (length(reason_hash)=64 AND reason_hash NOT GLOB '*[^0-9a-f]*')),
  pid INTEGER CHECK(pid>0),
  queue_depth INTEGER CHECK(queue_depth>=0),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `sessions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `project_id` | TEXT | yes | projects.id |
| `identity_hash` | TEXT | yes | — |
| `config_id` | TEXT | yes | config_snapshots.id |

```sql
CREATE TABLE sessions (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id),
  identity_hash TEXT NOT NULL CHECK(identity_hash IS NULL OR (length(identity_hash)=64 AND identity_hash NOT GLOB '*[^0-9a-f]*')),
  config_id TEXT NOT NULL REFERENCES config_snapshots(id)
) STRICT;
```

## `slo_evaluations`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `policy_id` | TEXT | yes | artifacts.id |
| `metric` | TEXT | yes | — |
| `window_start_us` | INTEGER | yes | — |
| `window_end_us` | INTEGER | yes | — |
| `sample_count` | INTEGER | yes | — |
| `observed_value` | REAL | no | — |
| `threshold` | REAL | yes | — |
| `result` | TEXT | yes | — |
| `evidence_id` | TEXT | no | artifacts.id |

```sql
CREATE TABLE slo_evaluations (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  policy_id TEXT NOT NULL REFERENCES artifacts(id),
  metric TEXT NOT NULL CHECK(metric IN ('ttft_ns','throughput_tok_s','latency_ns','error_rate','memory_bytes','cost_microusd','queue_ns')),
  window_start_us INTEGER NOT NULL CHECK(window_start_us>=0),
  window_end_us INTEGER NOT NULL CHECK(window_end_us>window_start_us),
  sample_count INTEGER NOT NULL CHECK(sample_count>=0),
  observed_value REAL,
  threshold REAL NOT NULL,
  result TEXT NOT NULL CHECK(result IN ('pass','fail','unknown')),
  evidence_id TEXT  REFERENCES artifacts(id)
) STRICT;
```

## `spans`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `run_id` | TEXT | yes | runs.id |
| `parent_span_id` | TEXT | no | spans.id |
| `actor_id` | TEXT | yes | actors.id |
| `kind` | TEXT | yes | — |

```sql
CREATE TABLE spans (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES runs(id),
  parent_span_id TEXT  REFERENCES spans(id),
  actor_id TEXT NOT NULL REFERENCES actors(id),
  kind TEXT NOT NULL CHECK(kind IN ('meta','swarm','model','tool','skill','hook','mcp','plugin','retrieval','memory','execution','system'))
) STRICT;
```

## `tasks`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `session_id` | TEXT | yes | sessions.id |
| `objective_artifact_id` | TEXT | yes | artifacts.id |
| `criteria_artifact_id` | TEXT | yes | artifacts.id |

```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY NOT NULL,
  session_id TEXT NOT NULL REFERENCES sessions(id),
  objective_artifact_id TEXT NOT NULL REFERENCES artifacts(id),
  criteria_artifact_id TEXT NOT NULL REFERENCES artifacts(id)
) STRICT;
```

## `telemetry_delivery`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `batch_hash` | TEXT | yes | — |
| `state` | TEXT | yes | — |
| `count` | INTEGER | yes | — |
| `first_producer_seq` | INTEGER | no | — |
| `last_producer_seq` | INTEGER | no | — |
| `reason_hash` | TEXT | no | — |

```sql
CREATE TABLE telemetry_delivery (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  batch_hash TEXT NOT NULL CHECK(batch_hash IS NULL OR (length(batch_hash)=64 AND batch_hash NOT GLOB '*[^0-9a-f]*')),
  state TEXT NOT NULL CHECK(state IN ('queued','replayed','duplicate','rejected','dropped','acknowledged')),
  count INTEGER NOT NULL CHECK(count>=0),
  first_producer_seq INTEGER CHECK(first_producer_seq>=0),
  last_producer_seq INTEGER CHECK(last_producer_seq>=first_producer_seq),
  reason_hash TEXT   CHECK(reason_hash IS NULL OR (length(reason_hash)=64 AND reason_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `telemetry_sessions`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `monitor_hash` | TEXT | yes | — |
| `config_hash` | TEXT | yes | — |
| `state` | TEXT | yes | — |
| `last_ack_seq` | INTEGER | no | — |
| `pending_count` | INTEGER | yes | — |
| `dropped_count` | INTEGER | yes | — |

```sql
CREATE TABLE telemetry_sessions (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  monitor_hash TEXT NOT NULL CHECK(monitor_hash IS NULL OR (length(monitor_hash)=64 AND monitor_hash NOT GLOB '*[^0-9a-f]*')),
  config_hash TEXT NOT NULL CHECK(config_hash IS NULL OR (length(config_hash)=64 AND config_hash NOT GLOB '*[^0-9a-f]*')),
  state TEXT NOT NULL CHECK(state IN ('started','resumed','healthy','degraded','stopped')),
  last_ack_seq INTEGER CHECK(last_ack_seq>=0),
  pending_count INTEGER NOT NULL CHECK(pending_count>=0),
  dropped_count INTEGER NOT NULL CHECK(dropped_count>=0)
) STRICT;
```

## `trajectories`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `run_id` | TEXT | yes | runs.id |
| `parent_id` | TEXT | no | trajectories.id |
| `path_hash` | TEXT | yes | — |

```sql
CREATE TABLE trajectories (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES runs(id),
  parent_id TEXT  REFERENCES trajectories(id),
  path_hash TEXT NOT NULL CHECK(path_hash IS NULL OR (length(path_hash)=64 AND path_hash NOT GLOB '*[^0-9a-f]*'))
) STRICT;
```

## `trajectory_steps`

| Column | SQLite type | Required | Reference |
| --- | --- | --- | --- |
| `id` | TEXT | yes | — |
| `event_seq` | INTEGER | yes | events.seq |
| `trajectory_id` | TEXT | yes | trajectories.id |
| `step_no` | INTEGER | yes | — |
| `span_id` | TEXT | yes | spans.id |
| `atom_id` | TEXT | no | atoms.id |

```sql
CREATE TABLE trajectory_steps (
  id TEXT PRIMARY KEY NOT NULL,
  event_seq INTEGER NOT NULL REFERENCES events(seq),
  trajectory_id TEXT NOT NULL REFERENCES trajectories(id),
  step_no INTEGER NOT NULL CHECK(step_no>=0),
  span_id TEXT NOT NULL REFERENCES spans(id),
  atom_id TEXT  REFERENCES atoms(id),
  UNIQUE(trajectory_id,step_no)
) STRICT;
```

## Views

### `atom_activity`

```sql
CREATE VIEW atom_activity AS SELECT e.run_id,o.operation,COUNT(*) AS operations,COUNT(DISTINCT o.atom_id) AS distinct_atoms
 FROM atom_operations o JOIN events e ON e.seq=o.event_seq GROUP BY e.run_id,o.operation;
```

### `hunger_summary`

```sql
CREATE VIEW hunger_summary AS SELECT gap_hash,SUM(action='observed') AS hits,
 MAX(depth) AS max_depth,MIN(e.occurred_at_us) AS first_seen_us,MAX(e.occurred_at_us) AS last_seen_us,
 (SELECT action FROM hunger_events h2 WHERE h2.gap_hash=h.gap_hash ORDER BY event_seq DESC,id DESC LIMIT 1) AS latest_action
 FROM hunger_events h JOIN events e ON e.seq=h.event_seq GROUP BY gap_hash;
```

### `invocation_summary`

```sql
CREATE VIEW invocation_summary AS SELECT a.id AS attempt_id,a.invocation_id,a.attempt_no,
 EXISTS(SELECT 1 FROM invocation_events i WHERE i.attempt_id=a.id AND i.phase='dispatched') AS did_dispatch,
 EXISTS(SELECT 1 FROM invocation_events i WHERE i.attempt_id=a.id AND i.phase='started') AS did_start,
 (SELECT status FROM invocation_events i WHERE i.attempt_id=a.id ORDER BY event_seq DESC,id DESC LIMIT 1) AS execution_status,
 (SELECT result FROM acceptance_checks c WHERE c.attempt_id=a.id ORDER BY event_seq DESC,id DESC LIMIT 1) AS latest_check
 FROM attempts a;
```

### `kv_usage_extrema`

```sql
CREATE VIEW kv_usage_extrema AS SELECT k.cache_id,MIN(k.used_bytes) AS sampled_min_bytes,MAX(k.used_bytes) AS sampled_max_bytes,
 MIN(k.used_tokens) AS sampled_min_tokens,MAX(k.used_tokens) AS sampled_max_tokens,COUNT(*) AS samples,
 MIN(e.occurred_at_us) AS first_sample_us,MAX(e.occurred_at_us) AS last_sample_us
 FROM kv_samples k JOIN events e ON e.seq=k.event_seq GROUP BY k.cache_id;
```

### `resource_by_actor`

```sql
CREATE VIEW resource_by_actor AS SELECT e.run_id,e.actor_id,r.metric,r.quality,SUM(r.value) AS delta_total,COUNT(*) AS samples
 FROM resource_samples r JOIN events e ON e.seq=r.event_seq WHERE r.sample_kind='delta' AND r.allocation_id IS NULL
 GROUP BY e.run_id,e.actor_id,r.metric,r.quality;
```

## Changelog

1.0.0 — 2026-10-29: generated data dictionary for schema version 1.
