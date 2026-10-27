# C07 — Memory State Management (AtomaDB-RAG Auto-Injection)

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

One canonical memory home for the whole swarm; every agent starts with collective memory and
deposits a structured session summary.

## Subcategories (broad topics)

- C07.1 Auto-injection at session/file-tab start (memory prefix from AtomaDB-RAG)
- C07.2 Mandatory structured session summary (6 sections: brainstorm, attempts, plan, executed, conclusions, coming plan)
- C07.3 Canonical home and collective reinvocation (no silos)
- C07.4 Enforcement and receipts (summary_missing receipt; mind record id + fingerprint)
- C07.5 Honest limits (file-tab hook TBD, mandatory gate not enforced, BM25 unless embed configured)
- C07.6 Cascade memory (Org Policy → User → Project Rules → Session Auto)
- C07.7 Prompt-injection hygiene for injected memory (sanitization/tagging, red-team ablation A10)

## Open questions / gaps

- File-tab-open injection point in `lab::start_turn` / `ide` open path (TBD).
- Memory injection necessity ablation (§16.1) required before "required" status.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 10. Memory State Management — AtomaDB-RAG auto-injection (PROPOSED)

**Concept.** One canonical memory home for the whole swarm. Every agent reads from the same
AtomaDB-RAG memory store on session start and writes its structured session summary back into
it. No agent is memoryless on reinvocation, and no agent holds a private memory island.

### 10.1 Auto-injection at session/file-tab start

- When an agent opens a new IDE file tab (or session boot), the backend builds the system prompt
  with an injected memory prefix pulled from AtomaDB-RAG (`memory` collection, scoped by
  project/lane, newest-first, bounded by the 200-line context budget).
- The injection is automatic and non-optional: the agent always starts with the canonical
  collective memory report, not just its own history.
- Source: `MindClient::tail` (existing) over the `mind` store, optionally augmented with RAG
  retrieval over `papers/research/knowledge` for evidence relevant to the task.
- Provenance: each injected block is tagged with lane + run_id + receipt fingerprint so the
  model knows which memories are its own vs. collective.

### 10.2 Mandatory session summary structure (agent-authored, appended to RAG memory)

Every agent, at session end (or pre-compaction), MUST deposit a structured 5W-derived summary
via `memory_deposit` / `POST /mind/submit` (and optionally `rag_ingest` into the `research`
collection) with these sections:

1. **Brainstorm summary** — what was considered, options, discarded ideas.
2. **Attempts report** — what was tried, parameters, tool calls, runtime, failures, root causes.
3. **Planning / preregistration** — predicted outcomes and judges (links to packet
   `preregistered_plan`), what would falsify the plan.
4. **Executed** — what actually ran, with notes on success and failures (receipt IDs, holes,
   verdicts).
5. **Conclusions** — what was learned, decisions made, invariants established.
6. **Coming plan** — next steps, open questions, hand-offs to other lanes.

These map onto the existing `FiveW` (who/what/where/when/why) plus `run_id`, `lane`,
`project` fields already in `MindLine`, so no new storage schema is required; the sections are
conventions in the `what`/`why` text plus `kind` tags.

### 10.3 Canonical home and collective reinvocation

- All summaries land in **one canonical home** (the AtomaDB-RAG memory store: `mind.jsonl` in
  the library-db root, plus RAG collections for research-paper-ingest artifacts). No per-agent
  silo, no duplicate ledgers.
- On reinvocation of any agent, the backend aggregates the **full memory report summary from
  collective ingestion** (latest live records across all lanes, filtered by project, ordered
  newest-first) and injects it into that agent's system prompt. Every agent re-enters with the
  team's combined experience, not a blank slate.
- Cross-lane visibility: unscoped records (project "") are visible to every frame; scoped records
  follow the project rule already in `MindClient::tail`.

### 10.4 Enforcement and receipts

- The summarize-before-compaction rule (60% fold) becomes a code-enforced gate: an agent that
  ends a session without depositing the structured summary records a `summary_missing` receipt
  and is flagged in `/metrics` / the swarm board. (Currently documented only; wiring PROPOSED.)
- Each summary deposit is a receipt (mind record id + receipt fingerprint) so the memory's
  provenance is auditable end to end, consistent with the existing hole/Merkle receipt pattern.
- Telemetry for memory events carries ids and scopes only, never summary text (unchanged privacy
  rule from `mind.rs`).

### 10.5 Honest limits

- The auto-injection wiring at file-tab open is PROPOSED (backend currently has no file-tab-open
  hook; injection point is in `lab::start_turn` / `ide` open path, TBD).
- The canonical-home ingest path (`rag_ingest` + `memory_deposit`) exists as of this pass; the
  _mandatory_ summary gate is not yet enforced.
- Dense retrieval over memory is BM25 today unless an embed endpoint is configured (honest).

## Formal Memory Consistency Model

### C07.1 Auto-Injection as Causal Broadcast

The memory injection is a **causal broadcast** over the agent swarm. Let $\mathcal{A} = \{a_1, \dots, a_N\}$ be the set of agents. The **memory state** at agent $a_i$ is a vector clock $VC_i \in \mathbb{N}^N$.

**Injection Protocol**:

1. Agent $a_i$ opens tab/session → sends `INJECT_REQ(VC_i)` to memory coordinator
2. Coordinator responds with `INJECT_RESP(M, VC_{mem})` where $M$ is memory prefix, $VC_{mem}$ is memory vector clock
3. Agent updates $VC_i \leftarrow \max(VC_i, VC_{mem})$ (element-wise max)
4. Agent builds system prompt with $M$ tagged by lane/run_id/receipt

**Causal Consistency Guarantee**: If $M_1$ causally precedes $M_2$ (i.e., $VC_1 \prec VC_2$), then any agent that sees $M_2$ must have seen $M_1$ (or a superset).

### C07.2 Session Summary as CRDT

The **session summary** is a **Conflict-free Replicated Data Type (CRDT)** over the 5W structure:

$$\text{Summary} = \text{G-Counter}(\text{brainstorm}) \oplus \text{LWW-Register}(\text{attempts}) \oplus \text{OR-Set}(\text{conclusions}) \oplus \dots$$

- **Brainstorm**: G-Counter (monotonic, append-only)
- **Attempts**: LWW-Register per attempt (timestamped by logical clock)
- **Plan/Executed/Conclusions/Coming**: OR-Sets with tombstones

**Merge**: Element-wise CRDT merge preserves all non-conflicting information. Conflicts (same attempt ID, different outcomes) are flagged for human review.

### C07.3 Canonical Home: Linearizable Log

The canonical home (`mind.jsonl` + RAG collections) implements a **linearizable log** via:

- **Single-writer**: Memory coordinator serializes all deposits
- **Logical clock**: Monotonic `LogicalClock` (C17) provides total order
- **Receipt chain**: Each entry $e_n$ contains $\text{hash}(e_{n-1})$ forming a hash chain

**Reinvocation Protocol**:

1. Agent $a_i$ requests `MEMORY_REPORT(project, VC_i)`
2. Coordinator returns all entries with $VC_{entry} \succ VC_i$ (causally newer)
3. Agent injects returned entries in logical clock order

### C07.4 Enforcement as Typed Effect

The **summarize-before-compaction** rule is a **typed effect** in the agent's type system:

```rust
// Agent effect signature
type AgentEffect =
    [Read(Memory), Write(Tool), ...,
     DepositSummary(SessionSummary) @ CompactionPoint] -> Result
```

The type checker rejects any agent implementation that does not include `DepositSummary` before `CompactionPoint`. The `summary_missing` receipt is a **type error at runtime**.

### C07.5 Prompt-Injection Hygiene: Sanitization as Information Flow Control

Injected memory is **untrusted data** flowing into the **system prompt** (trusted context). We enforce **non-interference**:

$$\text{Low}(M_{injected}) \not\leadsto \text{High}(\text{tool\_calls})$$

**Sanitization Pipeline**:

1. **Tagging**: Each injected block tagged with `provenance = (lane, run_id, receipt_hash, trust_level)`
2. **Classification**: ML-based prompt-injection classifier (trained on red-team corpus) outputs `risk_score \in [0,1]`
3. **Redaction**: If `risk_score > \tau_{inject}`, block is either:
   - **Quarantined**: Moved to separate `quarantine` collection, not injected
   - **Redacted**: Suspicious spans masked, safe remainder injected with `trust_level = LOW`
4. **Audit**: Every redaction/quarantine emits receipt with classifier version and confidence

**Red-Team Ablation (A10)**: Adversarial memory poison must not change tool calls without judge flag. Measured as:
$$\text{ASR} = \frac{\#\text{successful injections without flag}}{\#\text{injection attempts}} < 0.01$$

### C07.6 Cascade Memory as Precision Hierarchy

The cascade (Org Policy → User → Project Rules → Session Auto) is a **precision hierarchy** in the active inference sense:

| Level         | Precision $\pi$  | Content                       | Update Rate       |
| ------------- | ---------------- | ----------------------------- | ----------------- |
| Org Policy    | $\infty$ (fixed) | Compliance, security, erasure | Never (immutable) |
| User          | High             | Preferences, workflows        | Session           |
| Project Rules | Medium           | Lane configs, RAG scopes      | Project           |
| Session Auto  | Low (adaptive)   | Context budget, compaction    | Turn              |

**Precision-weighted prediction error** at level $l$:
$$\epsilon_l = \pi_l (o_l - \hat{o}_l)$$
Higher precision = slower updates = stronger prior.

### Verification Gates

| Gate                | Test                         | Threshold                                |
| ------------------- | ---------------------------- | ---------------------------------------- |
| Causal broadcast    | Concurrent inject requests   | No causal violations                     |
| CRDT merge          | Concurrent summaries         | Merge preserves all non-conflicting info |
| Linearizable log    | Concurrent deposits          | Total order = logical clock order        |
| Type enforcement    | Agent without DepositSummary | Compile-time rejection                   |
| Sanitization        | Red-team corpus (A10)        | ASR < 1%                                 |
| Precision hierarchy | Policy update propagation    | Org Policy never overwritten             |

## Open questions / gaps

- File-tab-open injection point in `lab::start_turn` / `ide` open path (TBD).
- Memory injection necessity ablation (§16.1) required before "required" status.
- Select ML classifier for prompt-injection detection; red-team corpus curation.

---

## Contributor Deepening — Muse (2026-10-26)

### C07.7 Memory capacity accounting (depth: the leak nobody budgeted)

The canonical home (`mind.jsonl` + RAG collections) is append-only (NF2) with no stated bound. Append-only without a bound is a disk leak with a receipt. Frozen accounting rule [PROPOSED — numbers are placeholders until C21 measures; the accounting _structure_ is the contribution]:

1. Every deposit carries `bytes` + `logical_clock`; the coordinator maintains `total_bytes` + `bytes_per_project` + `bytes_per_lane` as ledger counters (counts only — ADR-06 clean).
2. A configured `mind_budget_bytes` (default PROPOSED: 1 GiB per project) triggers **roll-up, not deletion**: oldest entries are compacted into a dated roll-up summary (same 6-section schema, `kind = rollup`) whose receipt chains to the compacted range; the roll-up preserves hash-chain continuity (entry `e_n` still contains `hash(e_{n-1})` via the roll-up's `covers_range` field).
3. Telemetry: `mind_bytes_total`, `mind_rollups_total`, `mind_oldest_clock` — counts/clocks only, never text.
4. Judge (ties to A9 necessity): recall of a rolled-up objective must equal recall of the raw entries on the A9 test set; if roll-up loses information, the budget is too aggressive and the item FAILS back to unbounded-with-alert.

[INFERRED — roll-up preserves C07.3 linearizable-log guarantees only if `covers_range` chains hashes; ends at the C07.3 spec above.]

### C07.8 Injection budget interaction (depth: F2 vs the 200-line ceiling)

F2 requires auto-injection "bounded by the context budget" (C08) but no arbitration rule exists when memory prefix + task + pinned messages exceed budget. Frozen arbitration order [PROPOSED — precedence is my proposal]:

1. Pinned messages (system + original user) — never truncated (C24.3).
2. Current task context — truncated last.
3. Memory prefix — truncated first, newest-first, with a `prefix_truncated` receipt recording `entries_dropped` + `oldest_clock_kept` (counts only).

A run where the memory prefix is truncated to zero MUST still satisfy A9's cross-lane recall test on the _kept_ prefix — if A9 needs entries that arbitration drops, the budget (C24 three tiers) is undersized for that workload and the failure is recorded against C24, not C07.

What was NOT checked here: `MindClient::tail` current truncation behavior (implementation not in this workspace); ML classifier selection for C07.5 (still open per above).
