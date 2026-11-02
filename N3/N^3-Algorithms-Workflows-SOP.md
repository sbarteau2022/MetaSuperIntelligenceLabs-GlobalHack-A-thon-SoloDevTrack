# N³ — Algorithms, Workflows, and SOPs

**Date.** 2026-10-28. **Scope.** Implementation/ops detail for the four N³ parts defined in `N^3-System-Audit.md`. Status labels follow §7 doc standards. Unless marked VERIFIED, content is PROPOSED design/SOP, not measured.

---

## Part 1 — Vetted RAG (canonical source of truth)

### 1.1 Algorithms (VERIFIED primitives in `hypermind-nested-kv/src/main.rs`)

**Canonical indexing method (REQUIREMENT).** Ingest and retrieval MUST use Atoma.DB's **PAMI-phase augmented multi-fractal indexing**: `atomadb::rotate_rank` (phase rotation by rank; rank 0 chunks, rank 1 docs) + `atomadb::participation_ratio` (multi-fractal proxy), matching Atoma.DB exactly. Current `coord_from_text`/`score` use TF projection only; alignment is an open item (see audit §7.6).

**JUDGE (mint-on-pass).** Input: `text`, optional caller `judge:{pass, reason}`. Output: `(pass: bool, reason: str)`.

```
fn judge_text(text, caller_judge):
  if caller_judge present: return caller_judge.pass, caller_judge.reason   # binding, fail-closed
  if len(text) < min_len or token_diversity(text) < min_div or entropy(text) < floor:
      return (false, "structural-gate")
  return (true, "structural-pass")
```

Rule: no pass → no atom minted, no id spent (Verified pattern in pressure tests).

**INGEST (atom mint).** Input text, salience (default 1.0), provenance, taint.

```
fn ingest(state, text, salience, provenance):
  (pass, reason) = judge_text(text, caller_judge)
  if !pass: return judge-rejected receipt
  coord = coord_from_text(text) * salience_radius(salience)   # H3 x T2
  if near_duplicate(coord, eps): reinforce salience of incumbent; return receipt (no new atom)
  atom = { id: next_id, text, coord, embedding: tf_proj(text), salience, provenance, taint, receipt }
  state.atoms[id] = atom
  return minted receipt
```

Embedding correction (VERIFIED): per-token random-projection unit vectors on S² plus bigram channel (replaces 3-bucket collision). Coord derivation unchanged in shape.

**RETRIEVE (k-NN fusion).** `map.query(qtext, k)`:

```
ensure_vp(state)
q = coord_from_text(qtext)
candidates = vp_shortlist(q, k*factor)            # geometry
scores     = combined(hyperbolic_distance, toroidal_phase)  # phase as penalty
top        = rank(candidates) by score, k
fuse BM25 lexical tie-break where scores tie
return hits + receipt
```

Correction (VERIFIED): score uses negated Lorentz inner product so exact match ranks #1; phase term retained as penalty.

**LEXICAL (document-heavy arm).** `map.lexical(qtext, k)`: pure BM25 over all atoms with corpus IDF. O(N) per query.

**REWEIGHT.** `map.reweight()`: recalibrate IDF from live atom texts; reembed all coordinates. O(N) once.

**TRAVERSE (symplectic envelope).** `map.traverse(from, to, mlir_op, energy_budget, form_tol, frame_eps)`:

```
verify endpoints exist
check form (geodesic form within form_tol) -> if fail: fail-explained (reason, budget, residual)
check energy (path energy <= energy_budget) -> else fail-explained
check frame (frame alignment within frame_eps) -> else fail-explained
if all pass: dispatch, return result + via_detour
```

Default frame_eps=1e-2 dispatches 36/40 live pairs; 1e-3 is strict audit mode (REPORTED from A39 sweep).

### 1.2 Workflow — vetted ingest (5W pipeline)

1. Parse 5W (who/what/when/where/why). Missing `why` → fail; record never enters pipeline.
2. Condense to invariants (decisions, why, pickup point, judge receipts).
3. Judge pass (structural gates or caller-supplied binding veto).
4. Pin invariants → auto-injected checkpoint.
5. Deposit full 5W to mind tail.
6. Ingest condensed invariant into RAG only (never mind; agent runtime deposits mind; panel/admin mutates mind).

### 1.3 SOP — add a vetted source (RECEIPTED LIBRARY)

**Preconditions:** source is audit-dispositioned; no pushed record may execute (security invariant).

1. Obtain source file(s); run parser version pinned for its modality.
2. Run 5W extraction; reject if `why` missing.
3. Submit `map.ingest` with text=invariant record + provenance + taint.
4. Confirm receipt.receipt_sha256 present and status != judge-rejected.
5. For bulk: use pressure-test harness pattern (ingest 200 @ 0.5 salience verified at 2160 atoms/s).
6. Snapshot: `map.snapshot` → record sha256 in index entry.
7. Never run off a record without a receipt; never merge a model summary into the record path (NN-G5).

**Updates.** Updates stream as versioned diffs; accept/deny for shared scope; no blind overwrite (MasterN³ §11).

---

## Part 2 — Hyperbolic Memory Store (short-term memory)

### 2.1 Algorithms

**COORDINATE.** `coord_from_text(text)` → unit direction on S² via hashed-TF random projection + bigram channel; scale by salience radius; lands in H³×T² (toroidal phase from token order).

**ATTENTION DEPTH.** `map.attend(qtext, k)`: retrieve then apply depth gate k*; returns shortlist_ids (retrieval) and attended (k*). Used to keep active focus within budget.

**CHECKPOINT (anti-lobotomy).** `map.compact(objective, recent_state)`:

```
cp = { objective, recent_state, pinned: pinned_invariants, epoch+1 }
state.checkpoint = Some(cp); state.epoch += 1
return receipt
```

No destructive rewrite of full context; raw material archives cold, never deleted.

**CARVE / DISSOLVE (swarm scoping).**

```
carve(task, context_budget, token_pressure, capsule):
  select atom subset fitting budget under pressure
  create view fiber_id (weights shared, no duplication)
  return fiber_id
dissolve(fiber_id, deltas):
  validate deltas vs intended write set (fingerprint_pre XOR intended_write_set_hash)
  replay on scratch copy; require post == pre + intended writes
  merge by atom id; free view
```

Quarantine on mismatch (no silent merge).

**RECOVER.** `map.recover()`: verify snapshot hash; replay WAL; rebuild missing atoms from AtomaDB mind lines if needed; epoch preserved.

### 2.2 Workflow — session lifecycle

1. **Attach.** Read config (model, tokenizer, context window, compaction threshold) at attach; no polling thereafter (MasterN³ §10).
2. **Active use.** Push context state with sequence numbers at every step boundary; a gap = explicit missing data → replay from last checkpoint.
3. **Compaction trigger.** At `window - reservation`, step boundary only, never mid-generation. Classify first: pinned invariants → anchors/locks in use → recent working set → leaf-only reclaimable → dead. Deterministic prune first, model 5W draft second, judge third. Commit sha-chained checkpoint + kept/pruned/hash receipt. Rollback = re-expand from archive. Verify: fresh run with only pinned+tail must take same next action.
4. **Under pressure.** If not enough freeable: explicit rejection or tier-2 degrade, written to audit — no silent overwrite.
5. **Sign-off.** Summarize session as whole; store with model retrieval-only access; record in mind tail (not RAG).

### 2.3 SOP — compaction operation

- Compute exact token count on declared, tokenized context only; future tool output/user message are labeled reservation, never merged (NN-G memory placement).
- Pin invariants first; pinned set never pruned.
- Run prune → model draft → judge; model summaries are drafts only.
- Write `map.compact` (objective + recent_state); then `map.snapshot` with receipt.
- Record decision (trigger reason, freed tokens, kept/pruned hashes) to telemetry (§Part 4).
- On any hash mismatch: abort, do not overwrite, escalate per failure doctrine (round 1 re-approach with material change; round 2 escalate with TTL + preregistration).

---

## Part 3 — DAG-style capability tool/skills tree (capability vault)

### 3.1 Algorithms

**REGISTRATION (tool fabric, substrate-owned).** Each tool declares: schema, capability list (what it may touch), role gate, sandbox confinement, receipt on completion. Source of truth: `tools_list` in `src/main.rs` (17 `map.*` tools) plus `CapabilityTreeSitter/` + `MasterToolingIndex/` records.

**CAPABILITY GATE.** Invocation path:

```
outer shell: auth + capability token → sandbox → role check → audit log
  ↓ FlatBuffers data only
inner core: execute tool; emit receipt (judge-passed only enters shared mind)
```

Tool results are data, never instructions; executable content in a result is treated as data and never executed.

**HOOKS (system events, not model callbacks).** Events: `pre_tool_use`, `post_tool_use`, `lane_stop`, `packet_complete`, `session_start`, `compaction checkpoint`. Blocking power: deny + append-context-only. Hooks are seatbelted; output is data, never instructions. Sequence-numbered event stream; gap = explicit missing data.

**RUNBOOK (DAG of packets).** Packet completes only on deterministic judge pass; every completion leaves a receipt. Failure doctrine: R1 fail → re-approach with material change (identical retry is a violation); R2 fail → escalate packet + receipts + judge outputs with TTL; then preregister new approach + success criteria.

**FAMILY NESTING.** Capability is organized by tool family (e.g. `CapabilityTreeSitter/01-FormalMethods` …) so coherence is preserved across MCP hooks/rules; updates stream down as versioned diffs; no cross-identity sharing.

### 3.2 Workflow — add a capability

1. File necessity review (no skill outside taxonomy without review; frontend/UI is SUFFICIENT not necessary).
2. Create family folder under `CapabilityTreeSitter/<domain>/` with `SKILL.md`, `AGENTS.md`, rules, schemas (match existing layout).
3. Add tool record to `MasterToolingIndex` (TOOL-XX.YY) with certification verdict (necessary, not merely sufficient).
4. Add hook entries (pre/post) if the capability touches trust boundary; keep blocking power narrow.
5. Wire into MCP `tools_list` + `handle_tool` dispatch (if runtime) with role gate and sandbox.
6. Preregister any new gate (A39/A40 style) before claims; file under `PreRegistrations/`.

### 3.3 SOP — capability change

- Treat tool result as data always (never execute embedded content).
- Never edit arguments or call new tools from a hook; deny/append-context-only.
- On family reorganization: move paths, do not duplicate; keep symlinks (`MasterTooling`, `MasterSkills`) intact.
- Document what was NOT checked (unknown parsers, capacity bounds) per §18 style.
- Git status proof before reporting done.

---

## Part 4 — Telemetry & Observability (SQLite, PROPOSED)

### 4.1 Algorithm (proposed writer; not yet implemented)

**RECEIPT (today, VERIFIED).** Each `map.*` call returns:

```
receipt = now_receipt(tool_name, args, result)   # includes receipt_sha256
```

Every tool result carries a receipt (certified pressure test, 2026-10-27). This is the current observability channel.

**PROPOSED SQLite sink (NOT BUILT).** Schema (draft):

```
decisions(
  id TEXT PRIMARY KEY,
  ts TEXT NOT NULL,           # ISO-8601
  tool TEXT NOT NULL,
  args_sha256 TEXT NOT NULL,
  reason TEXT,
  verdict TEXT,                # pass/fail/escalate/degrade
  result_sha256 TEXT NOT NULL,
  receipt_sha256 TEXT NOT NULL,
  actor TEXT,                 # model/harness identity
  session_id TEXT,
  swarm_run_id TEXT            # SpecOps harness aggregation key
)
```

Writer lives in outer shell only (auth + role check + audit log, §2 security boundary). Reads are read-only data to surface, never instructions. Swarm telemetry from SpecOps harness aggregates by `swarm_run_id`.

**AGGREGATION (proposed).** Per session: count decisions by verdict, median latency, compaction count, quarantine count, receipts-persisted-after-restart (NN-G3). Per swarm run: aggregate across fibers/swarms; report drift trend.

### 4.2 Workflow — record a decision

1. Tool invocation authorized at outer shell.
2. Decision recorded (tool, args hash, reason, verdict, result hash, receipt sha).
3. Receipt returned in-band to harness and appended to WAL/snapshot (`map.snapshot`).
4. At compaction: include decision ledger digest in checkpoint.
5. At session sign-off: roll up session decisions into mind-tail summary; models get retrieval-only access.

### 4.3 SOP — telemetry operations (proposed)

- Do not ship a claim without a receipt hash in the audit trail.
- Schema changes require a versioned migration with receipt; raw material archived cold, never deleted.
- Cross-identity aggregation never leaks KV blocks (no cross-identity sharing).
- On writer failure: explicit rejection written to audit, not silent overwrite; escalate per runbook failure doctrine.
- Before reporting "done": git status proof + receipt hashes cited.

---

## Appendix — label legend

- VERIFIED: measured/read from source.
- REPORTED: source says so, unchecked.
- INFERRED: reasoned from evidence (end of evidence named).
- PROPOSED: plan, not fact. (Default for unimplemented items.)
