---
title: "N³ whole-system traceability and observability SQLite foundation"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-29T20:54:29.675284+00:00"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: "README.md"
    lines: "1-98"
    role: "supporting"
  - path: "hypermind-nested-kv/README.md"
    lines: "1-120"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/.gitignore"
    lines: "1-2"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/DESIGN.md"
    lines: "1-85"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/README.md"
    lines: "1-331"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/SCHEMA.md"
    lines: "1-1890"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/FILE-MAP.md"
    lines: "1-39"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/build_schema.py"
    lines: "1-186"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/store.py"
    lines: "1-251"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/validate.py"
    lines: "1-51"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/VALIDATION.json"
    lines: "1-29"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/migrations/001_initial.sql"
    lines: "1-1798"
    role: "supporting"
  - path: "hypermind-nested-kv/observability/tests/test_store.py"
    lines: "1-415"
    role: "test"
related_issues: []
related_documents:
  - "DESIGN.md"
  - "../../N^3-Wiring-and-Fallbacks.md"
  - "../../MasterDesignDocument/C31-Observability-Telemetry/Design.md"
tags: [sqlite, traceability, observability, n3, swarm]
---

# Executive summary

This database can record a task from the first decision through execution,
validation, knowledge reuse, and recovery. It records what ran, what it cost,
and whether evidence supports success. The schema and local database are built;
live harness, MCP, and model telemetry adapters are not connected yet.

# Problem statement

N³ had receipts and a proposed SQLite log, but no installed relational schema
for tracing the whole system. A tool returning without an error cannot establish
that it completed the task. A brigade also needs a recorded cost justification
before any agent is activated.

# Solution overview

The store has 76 STRICT tables and five read-only summary views. A single event
ledger connects typed facts across domains. Every fact has an immutable ID and
an event foreign key; the event supplies run, span, actor and clock provenance.
There is no unrestricted payload JSON column. Sensitive content lives outside
this telemetry file, referenced through opaque IDs and hashes.

## The questions it can answer

- What objective and acceptance criteria authorized this run?
- Which meta-controller decision chose a model, capability, or swarm?
- Was an invocation requested, dispatched, observed starting, and completed?
- Did execution succeed, and did a separate evidence-backed check pass?
- Which retries changed strategy, and which alternate tool/model recovered?
- Why was each brigade member activated, for how long, and against what cap?
- How much was measured versus estimated, and which costs were unavailable?
- What was the planned DAG, what actually executed, and what blocked progress?
- Which policies, hooks, permissions, sandbox checks and approvals applied?
- Which atoms were retrieved, presented, cited, verified used, or added?
- Which trajectories were reranked, rejected, selected, or judged novel?
- Was application KV or model KV used? Did it hit, miss, reuse or evict?
- What context was compacted, which invariants were pinned, and what restored?
- Which knowledge graph revision informed a decision or received its output?
- What degraded, what was spooled, what replayed, and what was lost?
- Which experiment, baseline and evidence support each performance claim?

## System coverage and schema map

| Domain | Tables | Captured facts / boundaries |
| --- | --- | --- |
| Ownership and task | projects, sessions, tasks, runs, config_snapshots | Project identity; session isolation; objective/criteria artifact references; parent runs; trace IDs; source/config hashes |
| Actors and models | actors, model_instances | Meta, swarm, agent, human, system; weights/tokenizer/backend/device/config identity; context capacity |
| Execution hierarchy | spans, events, lifecycle_events | Parent spans; requested/queued/started/idle/terminal states; producer replay IDs; occurrence and ingestion clocks; correction links |
| Capability registry | capabilities, capability_events | Tool, skill, hook, MCP server/method, plugin, resource, prompt; immutable definition/version; nesting; registration/resolve/deny/uninstall |
| Invocation and validation | invocations, attempts, invocation_events, acceptance_checks, objective_outcomes | Logical call; numbered attempts; strategy hashes; separate dispatch/start/execution/acceptance; evidence-backed verdicts |
| Fallbacks | recovery_edges, degradation_events | Retry, alternate tool/model, escalation, rollback; changed strategy; audited degraded path and recovery |
| Meta and brigade | meta_decisions, agent_leases, lease_events, model_routing | Meta justification artifact; approval; estimated/capped cost; max agents; expiry; reservation; activation/working/idle/release; selected/rejected models |
| Budget enforcement evidence | budget_limits, budget_breaches | Hard/soft cost, token, CPU/GPU, memory, energy, wall and idle limits; required breach action; evidence of actual response |
| Coordination | messages, scheduler_decisions | Sender/receiver and message hash; delivery/ack/rejection; admission, preemption, isolation, breaker state, queue depth |
| Skills/hooks/plugins/MCP | hook_events, mcp_exchanges | Hook before/after/error and allow/deny/append/skip; server and request identity; protocol/transport errors; initialize/list/call/read/subscribe/cancel/notify |
| Resources | allocations, resource_samples | Shared/exclusive physical allocation; actor/model attribution; tokens, CPU/GPU/ANE, memory, I/O/network, money, energy, idle, queue and utilization |
| Inference | inference_events | Load/prefill/decode/draft/verify/unload; input/output tokens; TTFT; duration; drafted/accepted counts; quality-check reference |
| Service objectives | slo_evaluations, alert_events | Latency, throughput, cost, memory and error-rate windows; threshold, sample count, result; alert lifecycle |
| DAG and capability graphs | graph_versions, graph_nodes, graph_edges, dag_transitions | Immutable graph revisions; dependencies; cycle refusal; readiness/blocking/execution transitions |
| Knowledge / Obsidian | graph_crosslinks, obsidian_sync | Notes/concepts/artifacts/atoms; supports/contradicts/derives/links; vault and manifest hashes; import/export/reconcile counts and conflicts |
| KV/cache | caches, kv_operations, kv_samples | Application KV versus transformer/prefix/radix/retrieval caches; identity owner; capacity; hit/miss/store/reuse/eviction; byte/token occupancy |
| Context continuity | context_events, context_sources | Assembly/trim/compact/checkpoint/restore/pin; usage/limit; org/user/project/session/retrieved source layers; invariant references |
| Canonical atoms | atoms, atom_revisions, atom_operations | Stable atom and revision identity; source/hash/rank/embedding/method/salience; passed judge; added/reinforced/merged/retrieved/presented/cited/verified-used |
| Ingest/retrieval | ingest_stages, retrieval_queries, retrieval_hits | 5W/condense/judge/mind-tail/deposit/embed/index/dedup/quarantine; lexical/geometric/fused/PAMI/full-scan; k/depth/latency/degradation; hit scores and ranks |
| Trajectories/reranking | trajectories, trajectory_steps, rerank_runs, rerank_candidates | Parent trajectories and steps; ranker/criteria identity; candidates; rank before/after; score and selected flag |
| Novelty | novelty_assessments | Candidate and comparison-set hashes; method/version; score/threshold; admit/reinforce/reject/defer; admitted atom and evidence |
| Memory geometry | memory_operations | Traverse/basin/carve/dissolve/consolidate/checkpoint/snapshot/recover/passport/reweight; fingerprints before/after; invariant preservation and counts |
| Hunger and learning | hunger_events, distillation_events | Counted gap observations/sating/reopening; deterministic taps and thresholds; teacher/student/dataset and preregister/generate/judge/admit stages |
| Security and execution | policy_events, holes, hole_events, approvals | Auth/role/capability/sandbox/network/filesystem/egress gates; hashed token references; sandbox/base revision; patch/test/review/merge evidence; approval/revocation/expiry |
| Evidence and research | artifacts, receipts, experiments, experiment_results, claims, judge_results | Evidence hashes; receipt/Merkle proof verification state; preregistration/workload/hardware/baseline; control/candidate/ablation arms; claim labels; deterministic/model/human judges |
| Store operations | service_events, telemetry_sessions, telemetry_delivery, daily_rollups, governance_events | Monitor health; resume/ack/backlog/drop counts; spool/replay/duplicates; bounded rollup ranges; release/config/migration/backup/restore/export/retention/erasure/legal hold |
| Ledger integrity | schema_migrations, ledger_roots | Version/checksum; immutable schema and facts; event hash chain; 1000-event Merkle blocks |

A table's presence establishes storage coverage, not that its subsystem exists
or emits events. For example, PAMI, Obsidian synchronization, energy measurement,
external Merkle anchoring and automatic model release are not implemented here.

# Implementation details

## Files modified

The requested documentation-standard file was absent from `Desktop/OPS_BOARD`;
the available `Desktop/Special Ops/DOCUMENTATION_STANDARDS.md` supplied the
implementation structure. Review status stays draft; no approval is invented.

See `FILE-MAP.md` for generated exact line ranges and `SCHEMA.md` for every table,
column, type and constraint. These are source line counts; new files have no
`git blame` history until committed. Nothing has been merged or deployed.

## Languages chosen and why

| Language | Component | Reason |
| --- | --- | --- |
| SQL | Versioned migration, constraints, indexes, views | SQLite is the specified durable sink; compatible with future Rust bindings |
| Python | Deterministic schema generator, offline admin, tests | Standard library only; no package installation; not an inference worker or runtime transport |
| Markdown | Design, coverage, operations, review evidence | Reviewable beside implementation |

Push further: connect the native Rust outer-shell writer and receipt adapters;
keep database work outside the retrieval and decode hot paths.

## Grounding in verifiable literature

This implementation was not directly grounded in published literature. It applies
established relational integrity, write-ahead logging, hash-chain and Merkle-tree
patterns to the local design sources named in `DESIGN.md`. It makes no new
research or throughput claim.

## Configuration and location

- Default database: `observability/store/traceability.sqlite3` relative to the N³ crate.
- Explicit path override: `store.py --db PATH` (argument follows the command).
- SQLite 3.37+ with JSON1 capability for future queries; the current migration
  itself does not depend on JSON functions. All tables use STRICT typing.
- Writer connections: foreign keys ON, recursive triggers ON, WAL, synchronous
  FULL, busy timeout 5000 ms. Reapply connection-local settings in every adapter.
- Database and backups use file mode 0600. The enclosing filesystem and host
  permissions remain part of access control. WAL/SHM inherit SQLite protections.
- Runtime database, WAL/SHM and test scratch are ignored by Git. SQL, tests and
  documentation are the reproducible source; there are no seeded fake live events.
- The existing `live-map` and Rust sources/dependencies are unchanged by this work.

## Outcome semantics

`invocation_summary` exposes observed dispatch/start, latest execution status,
and latest check separately. The latest check is not automatically the task's
acceptance verdict. `objective_outcomes` records that separate judgment.
A success requires a recorded start, but this does not prove physical execution
against a dishonest producer: the trusted boundary must verify receipts.
A passed check requires evidence; checking its contents remains the checker’s job.
Retries have distinct attempts; `recovery_edges` refuses identical strategy hashes.

## Brigade economics

Lease admission requires an approved activation by a meta actor, valid expiry,
agent/swarm roles, free agent count and cumulative reservation within budget.
Reservations are cumulative per decision; release does not silently refund them.
Reactivation requires a fresh decision. The recorded times are producer evidence;
the scheduler must enforce real-time expiry and actual resource limits itself.
Idle time and actual cost are separate facts. A schema cannot shut down a model.

## Measurement semantics

Costs use integer micro-USD; energy uses microjoules; memory uses bytes; durations
use nanoseconds; wall-clock timestamps use UTC microseconds. Numeric samples
carry measured/estimated/unavailable quality, interval, source hash and
counter/delta/gauge semantics. Missing readings are NULL, never zero.
Do not sum cumulative counters or memory gauges. Do not add inclusive swarm
metrics to constituent agent totals. `resource_by_actor` sums only unallocated
actor deltas, deliberately excluding shared physical allocation readings.
`kv_usage_extrema` reports observed min/max, sample count and interval; it cannot
prove unobserved continuous peaks. Restrict queries by run/window for comparisons.
Atoms retrieved, presented and verified-used are distinct operations; evidence
is mandatory for the last. Novelty scores do not prove correctness.

## Ledger, clocks and privacy

The offline writer atomically commits event and typed facts, rejects conflicting
producer-sequence replays and computes canonical SHA256 hashes. Immutable catalog
rows are protected by constraints but are not themselves part of event body hashes;
producers should reference definition hashes in their externally verified receipts.
The verifier checks the schema against the migration, foreign keys, SQLite
integrity, sequential ingestion order, fact hashes, event chain and Merkle blocks.
The hash chain is not a signature or external trust anchor. A filesystem owner
can rebuild it; external signing/anchoring is a future requirement.

Occurrence UTC may arrive out of order. Monotonic nanoseconds only compare within
a boot ID. Global sequence is ingestion order, not proof of causal execution order.
Ingestion UTC is clamped against clock rollback. C31 sketches TAI64N; this
foundation explicitly uses UTC microseconds plus boot-relative monotonic time.
TAI64N conversion is not implemented and must be reconciled at runtime integration.

No raw prompts, arguments, results, source code, message bodies, note text,
secrets or filesystem paths belong in telemetry. Raw content and vault paths stay
in the separate governed artifact/content store; telemetry holds opaque IDs/hashes.
Unknown fields are rejected. Typed IDs are not protection against a malicious
producer encoding secrets inside an ID; the trusted outer-shell adapter must
validate identifier provenance and authorization.

All tables reject update/delete and replacement through primary/unique keys.
Corrections append `supersedes_seq`; consumers must explicitly resolve correction
chains. Summary views report recorded latest facts, not a universal corrected
materialization. Retention/erasure tables record decisions, not physical deletion.
Sensitive external content deletion and segment retirement need a governed
procedure; append-only history must not be advertised as an implemented erasure UX.

# Testing evidence

The database suite covers normal paths, adversarial integrity failures, and
four concurrent writers producing 1040 events with a Merkle checkpoint.
`VALIDATION.json` records the final result, environment and evidence scope.
Final result: **54 tests passed**; 76 tables, 296 indexes, 292 triggers and five
views. SQL also loads successfully in the system SQLite CLI. No synthetic
telemetry is left in the initialized local database.
No Rust/runtime integration, full-system workload, sustained soak or real-model
benchmark is claimed. The source generator must reproduce byte-identical SQL.

From the repository root:

```sh
# Reproduce the schema and run the database suite.
python3 hypermind-nested-kv/observability/build_schema.py
python3 -m unittest discover -s hypermind-nested-kv/observability/tests -v
# Persist a fresh report and check the initialized base and system SQLite.
python3 hypermind-nested-kv/observability/validate.py
```

Necessity: the N³ audit documents the missing SQLite sink; these tests establish
the implemented database invariants, not a measured benefit to model throughput.

# Deployment / rollout

```sh
# Initialize or checksum-check the local store; no demo rows are inserted.
python3 hypermind-nested-kv/observability/store.py init
# Read-only integrity verification.
python3 hypermind-nested-kv/observability/store.py verify
# Online SQLite backup, including committed WAL contents; never overwrite.
python3 hypermind-nested-kv/observability/store.py backup --destination hypermind-nested-kv/observability/store/backup.sqlite3
```

The local base is initialized. No server, daemon, scraper, runtime adapter or
model was started. Before runtime attachment: back up the existing store, verify
the migration checksum, configure bounded spool/replay and authorize the writer.
Rollback: disable the future adapter and retain the verified database/backup;
never delete a live store or downgrade its schema in place. Migration failure
rolls back atomically; unrelated databases and checksum drift are rejected.

# Adversarial review

Review was performed in this run without subagents; independent human review
remains pending. Primary attacks and the resulting defenses:

- Unique-key replacement could delete old rows with recursive triggers off:
  explicit pre-insert guards now cover every unique key.
- An execution success could precede a start: phase/status and observed-start
  guards now reject that false completion.
- Shared allocations could inflate swarm totals: explicit allocation ownership
  and a restricted delta aggregation avoid automatic double counting.
- A passed tool result could be mistaken for objective success: separate
  evidence-backed checks/outcomes preserve the distinction.
- Replays, cross-run joins, KV identity reuse, DAG cycles, unapproved leases,
  changed schemas, fact tampering and partial writes are tested directly.

# Next stage of implementation

| Next step | Dependency | Status |
| --- | --- | --- |
| Rust outer-shell writer and receipt adapter | This schema and permission boundary | PROPOSED |
| Brigade scheduler cost/idle/expiry enforcement | Approved decision and budget records | PROPOSED |
| MCP/skill/plugin and native inference emitters | Atomic event/fact contract | PROPOSED |
| Obsidian and knowledge-revision adapter | Graph/content ownership contract | PROPOSED |
| Bounded spool/replay and separate monitor | Delivery/session/health schema | PROPOSED |
| Prometheus cardinality/privacy enforcement | Bounded projections and metric catalog | PROPOSED |
| Retention, erasure and signed external anchors | Content-store policy and segment procedure | PROPOSED |
| End-to-end acceptance, soak, workload measurements | Connected runtime | PROPOSED |

This implementation sets up whole-system traceability because all domains can
share one causal ledger. It does not establish that the entire system is wired.

# What we learned

| Category | Finding |
| --- | --- |
| Technical | SQLite REPLACE needs unique-key guards, not just UPDATE/DELETE triggers |
| Process | The canonical repo and current source differ from the stale chat working directory |
| Tooling | Standard-library SQLite supports deterministic, isolated failure tests |
| Estimation | No prior estimate or runtime throughput claim was made |
| Collaboration | Existing unrelated changes were preserved; no subagents were used |

## Changelog

| Version | Date | Author | Change |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-29 | Codex | SQLite foundation, coverage, admin, tests and local initialization |
