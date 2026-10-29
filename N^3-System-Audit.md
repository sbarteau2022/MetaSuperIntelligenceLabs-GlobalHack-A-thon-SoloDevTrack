# N³ System — Audit & Component Definitions

**Date.** 2026-10-28. **Auditor.** Zed agent (Stewart). **Scope.** The N³ ("Nested Neural Net") system as described in `MasterN^3DB-RAG - MultiModal.md` and implemented (partially) by `hypermind-nested-kv` (bin `hypermind-mcp`, v0.1.0). **Status.** Audit only; no code changed.

## 1. Bottom line (VERIFIED from repo)

- N³ is the **nested retrieval layer** inside Atoma, not an app and not a model loop. Substrate only: deterministic, model-independent. (REPORTED — design doc §1.)
- The single built, pressure-tested artifact is `hypermind-nested-kv` (`hypermind-nested-kv/Cargo.toml`, bin `hypermind-mcp`), certified WITH CORRECTIONS on 2026-10-27 (`HyperMindNestedKV-Certification.md`). PROPOSED gates NN-G1..G7 remain un-preregistered; retrieval is brute-force today, not VP-tree. [SUPERSEDED IN PART 2026-10-29 — see §9: VP-tree exact-prune kNN shipped on all query paths, NN-G prereg filed with measured run; PAMI rank tags implemented.]
- No telemetry SQLite DB exists in the built artifact; telemetry today = per-tool `receipt` objects returned by `handle_tool` (`src/main.rs::now_receipt` usage in every `map.*` branch). The SQLite aggregator described below is PROPOSED.

## 2. Part 1 — Vetted RAG (canonical source of truth)

**What it is.** The RAG side of the N³ tree: a vetted, compiled corpus (research preprints, autonomous runbooks, proofs/formalisms, implementations, compiled/vetted indexes) that serves as the canonical source of truth an AI builder attaches to. Records are data, never instructions; nothing is executed (MasterN³ doc §11, §4, security invariant). Ingest order is fixed and judge-gated: 5W parse → condense to invariants → judge pass → deposit full 5W to mind tail → ingest condensed invariant (rag only). Records without `why` fail validation and never enter (§5.2).

**Implementation surface (VERIFIED in `hypermind-nested-kv/src/main.rs`).**

- `map.ingest` — mints an atom (text payload + embedding-derived coordinate on H³×T²) only after a deterministic judge pass (`judge_text`); near-duplicates reinforce salience without a new atom; receipts on every result.
- `map.query` — k-NN recall by combined hyperbolic + toroidal score.
- `map.attend` — retrieve then apply depth gate k*.
- `map.lexical` — pure BM25 over all atoms with corpus IDF (document-heavy arm).
- `map.reweight` — recalibrate IDF on live texts and reembed all coordinates (O(N)).
- `map.traverse` — symplectic envelope (form/energy/frame checks) before dispatch; fail-explained, never fail-silent.

**Status.** Built and pressure-tested for exact-text retrieval at matched salience. Embedding is hashed-TF + random projection (corrected from 3-bucket collision); same-salience only (HyperMindNestedKV-Certification.md, Known gaps). VP-tree is not yet the actual index.

## 3. Part 2 — Hyperbolic Memory store (short-term memory)

**What it is.** The active-memory side of the N³ tree: session summaries, active production/task tracking, and telemetry "must-knows." Its job is context continuity: auto-detect the configured compaction threshold, gather the session's available window, summarize/condense/format, repack into the session context tree while pinning a short summary plus concise bullets at the back of the window; minimize lobotomy and context drift. At session sign-off the session is summarized as a whole and stored with model retrieval-only access, enabling later querying/expansion of the short-term mind (MasterN³ doc §5.1 modality "mind-tail receipts", §9 condensing, §10 active context).

**Implementation surface (VERIFIED).**

- `map.compact` — creates an anti-lobotomy continuity checkpoint (`objective`, `recent_state`) without destructive rewrite.
- `map.checkpoint` — read/write the continuity checkpoint.
- `map.carve` / `map.dissolve` — carve a task-scoped swarm view from the global map (weights shared, no duplication) and dissolve deltas back after fingerprint verification.
- `map.snapshot` / `map.recover` / `map.health` / `map.export_passport` — persistence, hash-verified recovery, status, portable passport.
- Push/sequence protocol and auto-trigger compaction are PROPOSED (MasterN³ §9–§10); the runtime currently exposes the primitives only. The auto compaction-percentage detector is NOT yet wired into a live session (INFERRED — design §10 says active usage is measured in the serving core; harness owns the trigger).

**Key invariants.** Anchors/locks are never overridden; no cross-identity KV sharing; no model self-summary in the record path (§9, §14).

## 4. Part 3 — DAG-style relational tool/skills tree (capability vault)

**What it is.** The coherence layer: a directed-acyclic, relational registry that nests capabilities by tool family, fed by MCP hooks and rules, and serving as the vault for capability. The shape (from the sketch in MasterN³ §19): two roots (vetted RAG + active memory) feed pinned invariants, tactical-reasoning skill, and tree-sitter, which converge on Capability. Hooks and skills are inputs; capability is the output (MasterN³ §19; C28/C29).

**Repo artifacts (VERIFIED paths).**

- `CapabilityTreeSitter/` — nested tool/skill families (`01-FormalMethods` … categories, each with `SKILL.md`, `AGENTS.md`, rules/schemas), the on-disk nesting by family.
- `MasterDesignDocument/MasterToolingIndex/` and `MasterSkillsIndex/` (symlinked as `MasterTooling`, `MasterSkills`) — tooling records (TOOL-XX.YY) and the seven necessary skills (SKL-01..07) with necessity proofs.
- `MasterDesignDocument/` C04/C28/C29 — architecture topology, skill registry/hooks, tool registry.
- MCP tool fabric responsibilities per MasterN³ §6 (definition schema, capability list per tool, role gate, sandbox, receipt) live in substrate; invocation timing/result handling live in the harness.

**Status.** Static/nested registries present on disk (VERIFIED). Runtime DAG execution and hook-driven coherence are PROPOSED; no runtime registry service beyond `map.*` is built (INFERRED — repo has no dispatcher DAG service).

## 5. Part 4 — Telemetry & Observability SQLite DB

**What it is (PROPOSED — design C31 / N³ open decisions).** A SQLite-backed observability store logging every decision: what, when, why, result, plus native aggregation of swarm telemetry from the SpecOps harness. It is the audit destination referenced by MasterN³ §4 (substrate holds telemetry) and §6 (receipts). It is also the integration point named in the user's N³ description.

**Current state (VERIFIED absence).**

- No SQLite dependency in `hypermind-nested-kv/Cargo.toml` (dependencies: atomadb, serde, serde_json, bincode, sha2, rand). No telemetry DB files present in the repo (search: none under `*.sqlite`/`*.db`).
- What exists instead: an in-band receipt per `map.*` call (`map.ingest`, `map.query`, …) carrying `receipt.receipt_sha256` (HyperMindNestedKV-Certification.md Step 3: every tool result carried a receipt). Also snapshot/WAL (`map.snapshot`, `map.recover`) and epoch bookkeeping.

**Open items (PROPOSED).** Define schema (decision id, tool, args hash, timestamp, verdict/reason, result hash, receipt sha), writer (outer shell only, per §2 security boundary), retention/migration, and the SpecOps harness swarm-aggregation adapter. Must remain read-only data to surface, never instructions (§4, §14).

## 6. Cross-part data flow (PROPOSED shape, per MasterN³ §3)

```mermaid
flowchart TD
    R[Request lane / user] --> O[Outer shell: auth, cap token, sandbox, role check, audit]
    O -->|FlatBuffers, data only| I[Inner core: map tree, basin, KV block, chunk]
    I --> M[Hyperbolic memory: compact/checkpoint, carved views]
    M --> RAG[Vetted RAG: judge-gated atoms, lexical + k-NN]
    RAG --> CAP[Capability tree: tool/skill families, hooks/rules]
    CAP --> TEL[Telemetry: per-call receipts (today); SQLite sink (proposed)]
```

## 7. Component inventory (file/path → part)

| Artifact                                                       | Part                                   | Status                            |
| -------------------------------------------------------------- | -------------------------------------- | --------------------------------- |
| `hypermind-nested-kv/` (bin `hypermind-mcp`)                   | RAG + Hyperbolic memory                | Built, certified with corrections |
| `MasterN^3DB-RAG - MultiModal.md`                              | Design of all four parts               | Pre-spec, draft                   |
| `CapabilityTreeSitter/`                                        | DAG capability vault (on-disk nesting) | Present on disk                   |
| `MasterDesignDocument/MasterToolingIndex`, `MasterSkillsIndex` | Tool/skill registries                  | Present on disk                   |
| Telemetry SQLite DB                                            | Observability                          | NOT BUILT (proposed)              |

## 7.5 Benchmark results (2026-10-28 run, VERIFIED)

Harness: `hypermind-nested-kv/pressure_test.py` against `target/debug/hypermind-mcp`, `HYPERMIND_ROOT` in a fresh temp dir. Stdio JSON-RPC, single process, no concurrency. Run outcome: PASS (all assertions green).

| Metric                          | Value                           | Notes                                    |
| ------------------------------- | ------------------------------- | ---------------------------------------- |
| Ingest 200 atoms (salience 0.5) | 640.2 ms → 312 atoms/s          | pressure harness, distinct texts         |
| Ingest 500 atoms (bulk)         | 5191.5 ms → 96 atoms/s          | bulk arm                                 |
| Query (exact text, k=5)         | 1.0 ms                          | top-1 = atom-42; hit #1 text exact match |
| Query under load (500 atoms)    | 1.9 ms                          | top-1 = atom-19 exact match              |
| Snapshot                        | status ok, digest `707c9aa7…`   | atoms=200                                |
| Recover                         | status ok, atoms=200, epoch=201 | hash round-trip verified                 |
| Compact                         | ok                              | receipt sha present                      |
| Passport export                 | sha256 recorded, atoms=200      | portable                                 |

Accuracy (VERIFIED, debug binary, same-salience): exact-text query returns the ingested atom as top-1 for tested ids (42 → 42, 19 → 19); `atom-42 in top-3` assertion passed. Claim restricted to same-salience or radius-normalized queries; cross-salience exact text does not always rank #1 (HyperMindNestedKV-Certification.md, Known gaps).

Caveats: debug binary, single-threaded, N≤500; no flat-baseline comparison yet (NN-G1/G2 remain PROPOSED); `exact_attention_measured=false` (built-in oracle full scan) per `map.health`.

## 7.6 Canonical ingest/retrieval method (REQUIREMENT, 2026-10-28)

Per Stewart: the N³ ingest and retrieval method **must be identical to Atoma.DB's**, i.e. **PAMI-phase augmented multi-fractal indexing** (phase rotation by rank, partition-and-aggregate, participation-ratio multi-fractal statistic). No bespoke embedder or ad-hoc geometry may diverge from it.

Binding implications (PROPOSED enforcement):

- `hypermind-nested-kv` must use `atomadb::rotate_rank` (RoPE-family phase rotation; rank 0 chunks, rank 1 docs; same-rank inner products preserved, cross-rank phase-suppressed) and `atomadb::participation_ratio` (multi-fractal proxy) instead of the current TF-projection-only direction in `coord_from_text`/`score`.
- Rank tagging: every atom tagged rank=0 (chunk) at mint; document-level views aggregate child chunks (partition-and-aggregate). Cross-rank retrieval applies phase rotation before scoring.
- Current state (VERIFIED): `src/main.rs` imports `atomadb::{Db, Role}` and uses `atomadb::tf_embed`, but does NOT call `rotate_rank`/`participation_ratio`. Alignment is an open implementation item (gap, not cosmetic).
- The `PAMI SMOKE` receipts in `vendor/atomadb-engine` (`pami-receipt.json`) are the reference evidence; any change must reproduce its checks.

## 7.7 Test/build status + deferred benchmarks (2026-10-28, VERIFIED/REPORTED)

- `cargo build` (release): PASS with warnings (HyperMindNestedKV-Certification.md, 2026-10-27). VERIFIED historically.
- `cargo test` (2026-10-28): FAILS TO COMPILE — 5 pre-existing errors (`snap` scope, `HungerRow` trait conflict, `StateSnapshot` serde bounds). Not caused by this audit; crate has no unit/integration tests beyond compile-time. REPORTED.
- Pressure harness `pressure_test.py`: PASS (all assertions) against debug binary, 2026-10-28. VERIFIED (§7.5).
- Full-system benchmark: **deferred**. Per-user note, per-component numbers are valid only for the built MCP substrate; end-to-end throughput/accuracy is to be (re)benchmarked when the RAG, hyperbolic memory, capability tree, and telemetry sink are wired together. Do not quote per-component numbers as full-system numbers until then.

## 8. What was NOT verified in this audit

- No build/test run was re-executed in this pass (last certified 2026-10-27); treat the certification receipt as REPORTED.
- No code path was traced beyond tool names in `tools_list`/`handle_tool` dispatch; open the crate (`src/main.rs`) for byte-level review.
- Gate status (NN-G1..G7) is PROPOSED and not preregistered; retrieval-speed claims are not yet validated against a flat baseline.

## 9. Dated corrections — 2026-10-29 (Zed agent; corrections are entries, originals preserved above)

- **§7.6 SUPERSEDED in part.** `rotate_rank` IS now called in `hypermind-nested-kv/src/main.rs` (`embed_direction_idf`, rank-parameterized since this pass): chunk embeds at rank 0 (identity — same-rank products exact), doc embeds at rank 1 (phase rotation, partition-and-aggregate over `children`). Rank tags (`rank: u8`, serde-default 0) ride the `Atom` struct, receipts, and all query/hit/euclid surfaces; `map.pami_stats` reports `participation_ratio` per rank group. What REMAINS open from §7.6: cross-rank retrieval at scale is exercised only on small fixtures (rank-1 query surfacing asserted in `ingest_doc_aggregates_children_and_tags_rank`); no large mixed-rank recall benchmark yet. The claim "does NOT call rotate_rank" is WITHDRAWN.
- **§7.7 compile-break note WITHDRAWN.** The "FAILS TO COMPILE — 5 pre-existing errors" report does not reproduce at HEAD: `cargo test --locked` green (3 lib + 42 bin + 8 transducer, VERIFIED this session). The errors were against a tree that predates the `rank`/PAMI fields (missing-field class). Standing rule unchanged: a crate with zero tests beyond compile-time is still REPORTED as test-thin, but it COMPILES.
- **§5 telemetry note PARTIALLY closed.** The SQLite sink exists OFFLINE (`hypermind-nested-kv/observability/`, 54/54 suite green, REPORTED) with a demonstrated replay path (`wire-proof.sqlite3`, 10 events, idempotent). The live in-server writer remains PROPOSED. `map.health` now surfaces `swarm_run_id` echo per receipt + `nng_status` (NN-G1/G3/G4/G5/G6 single-machine PASS; G2 speed + G7 leak audit OPEN).
