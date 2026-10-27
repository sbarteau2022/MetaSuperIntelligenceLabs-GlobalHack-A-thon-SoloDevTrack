# ToolingC06 — Tooling Certification for SKL-06 (Data & Knowledge Engineering)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-06` (`SkillsC1.md`) — Data & Knowledge Engineering (RAG/Ingest/Docs/Governance).
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS

**Necessity re-verified [VERIFIED — `C26/C33/C17/C18` read directly]:** ingest/round-trip (A8), retrieval parity (A3), doc governance (A26/A27), schema/migrations (C17), API contracts (C18) have no other operator. Gate G4 (contract) needs OpenAPI + MCP validation; G5/G6 need claim-lint + crosslink evidence. Necessity holds.

**Gate mapping correction (Scheme B → dual citation, per C15.9):**

| Day gate (SkillsC1) | Artifact gates (C15.x) operated |
| --- | --- |
| G3 Unit + Integration | C15.1 + C15.2 |
| G4 Integration + Contract | C15.2 + C15.4 |
| G5 Contract + Property | C15.5 |
| G6 Property + Chaos | C15.8 (property subset) |

**Ablations:** A3, A8, A26, A27 — no delta. (A16 LSP-stress: SKL-06 specifies the resource bound `max_lsp_servers`, SKL-03 operates the stress run — shared custody; bound without operation is a wish, operation without bound is unbounded.)

**Conditions:**

1. A8 round-trip runs PER TRUST TIER (C26.7): single-tier `test_default` pipeline evidences one tier and claims one. Tier-parameterize or qualify.
2. A3 epsilon 0.02 justified by power analysis in prereg (C26.8); per-collection deltas reported; query-set minimum N met. The 2-example YAML sketch in C26 is a SCHEMA example, not a query set.
3. The claim-linter grandfather rule (C33.7 item 2): legacy unlabeled text gets a dated labeling pass (Day 6–7 with C33), not a Day-1 build failure — else G1 blocks the sprint on day one over pre-existing prose.
4. `arrow`/`parquet` in the tool list: columnar formats are justified ONLY if a stated consumer (roll-up analytics? benchmark corpora?) requires them; otherwise YAGNI — remove or record the consumer. Unjustified deps violate Quality > Quantity (SkillsC1 principle).
5. MCP preset catalog (C18.4) definitions are missing (recorded C18 open item) — `openapi-generator`/MCP tooling below operates contracts, not presets; presets remain an open gap, not silently covered.

---

## Tool Specs

### TOOL-06.01 `bm25-retriever` — Deterministic baseline retrieval

1. **Tool Name:** `bm25-retriever` (k1=1.2, b=0.75 fixed; deterministic tie-break per C26.2)
2. **Tool Structure:** typed-JSON (query + k + collection in; ranked `[(doc_id, score)]` + `index_hash` out — index version bound in every result).
3. **Grammar Constraint Mask:** k1/b are CONSTANTS (tunable only via config migration with C09.8 re-validation — silent retuning invalidates A3 baseline); query length capped (DoS bound, C16 STRIDE).
4. **Tool Parser Required:** YES — `query-tokenizer` (deterministic tokenizer; same query string → same token stream, C03.5).
5. **Definition/Purpose:** The honest baseline everything dense must beat-or-tie (C26 verified-approximation bound); A3's control arm; NF6 graceful-degradation fallback.
6. **Draft Code:** (Rust — deterministic top-k)

```rust
fn retrieve(&self, q: &str, k: usize) -> Vec<ScoredDoc> {
    let terms = tokenize(q); // deterministic; no stemming surprises
    let mut s = score_terms(&terms);
    s.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0))); // tie-break = doc_id
    s.truncate(k); s
}
```

### TOOL-06.02 `ingest-pipeline` — Tiered CRDT ingest with receipts

1. **Tool Name:** `ingest-pipeline` (LWW-register per doc_id; tiered CANONICAL/VETTED/UNVETTED per C26.7)
2. **Tool Structure:** typed-JSON (document + tier + role in; `IngestReceipt` + `quarantine_flag` out; UNVETTED writes NEVER return a merge receipt — quarantine receipt only).
3. **Grammar Constraint Mask:** role gate pre-ingest (`Ingest` role required — C08 F4); UNVETTED→VETTED promotion requires operator approval + receipt (no silent promotion path exists in the tool).
4. **Tool Parser Required:** YES — `doc-validator` (schema + size caps + encoding normalization; oversized/malformed docs rejected pre-index, never half-indexed).
5. **Definition/Purpose:** Operates C26.3 ingest + C17.8 revocation counterpart (erase path calls the same tool with `op: revoke`); A8 round-trip operator per tier.
6. **Draft Code:** (Rust — tiered ingest)

```rust
fn ingest(&mut self, d: Doc, role: Role) -> Result<IngestReceipt> {
    role.can_ingest()?; validate_doc(&d)?;            // fail closed, pre-index
    let tier = classify_tier(&d)?;                    // CANONICAL|VETTED|UNVETTED
    if tier == UNVETTED { return quarantine_only(d); } // never merges silently
    self.lww_write(d)
}
```

### TOOL-06.03 `eval-harness-A3` — Labeled-query retrieval evaluation

1. **Tool Name:** `eval-harness-A3` (Top-1/MRR/nDCG@10 over the labeled set; per-collection deltas)
2. **Tool Structure:** typed-JSON (query-set version hash + retrieval config in; per-collection `Precision@k` deltas + global verdict out).
3. **Grammar Constraint Mask:** query-set MINIMUM N enforced (C26.8 item 3 — set below power-analysis N refuses to run with `insufficient_power` receipt, not a weak PASS).
4. **Tool Parser Required:** YES — `qrel-parser` (labeled YAML → typed relevance judgments; malformed qrel fails the harness, not the retrieval).
5. **Definition/Purpose:** Executes A3 including the T2-weight sensitivity arm (0.0/0.25/0.5/1.0 per C35.12); ANY collection below −epsilon fails A3 even if the mean passes.
6. **Draft Code:** (Python — per-collection gate)

```python
def judge_a3(results: dict) -> str:  # results: collection -> delta vs BM25
    for coll, d in results.items():
        if d < -EPSILON: return f"FAIL:{coll}"  # no mean-hiding (C26.8)
    return "PASS" if power_ok(QUERY_SET_N) else "INSUFFICIENT_POWER"
```

### TOOL-06.04 `claim-lint` — Claim-label + evidence linter (C33.2 type system)

1. **Tool Name:** `claim-lint` (`scripts/claim_linter.py`; regex `[(LABEL)]` + evidence-pointer resolution)
2. **Tool Structure:** typed-JSON (file set + grandfather-cutoff date in; `violations[]` with `pre|post_cutoff` classification out).
3. **Grammar Constraint Mask:** label enum closed (VERIFIED/REPORTED/INFERRED/PROPOSED); evidence-pointer URI must resolve (dangling pointer = violation, ADR-07).
4. **Tool Parser Required:** YES — `claim-extract-parser` (markdown → typed claim records; must ALSO recognize my bracketed long-form `[VERIFIED — ...]` / `[PROPOSED — ...]` / `[INFERRED — ...]` / `[REPORTED — ...]` labels used across this pass — a linter that misses the pass's own label format fails its own gate).
5. **Definition/Purpose:** Operates C33.2 + A26 (inject-50-unlabeled catch test); enforces grandfather rule (pre-cutoff violations → dated labeling backlog, not build failure).
6. **Draft Code:** (Python — labeling gate with grandfathering)

```python
def lint(files: list[str], cutoff: str) -> LintResult:
    claims = claim_extract_parser(files)  # must match [LABEL] and [LABEL — ...] forms
    for c in claims:
        if c.label is None: violation(c, "unlabeled", grandfather=c.date < cutoff)
        elif not resolves(c.evidence): violation(c, "dangling-evidence", grandfather=False)
```

### TOOL-06.05 `migration-audit` — Missing-doc + crosslink + five-file enforcer

1. **Tool Name:** `migration-audit` (`migration_status.json` tracker + crosslink validator + five-file checker)
2. **Tool Structure:** typed-JSON (workspace root in; `unresolved_refs[]`, `crosslink_breaks[]`, `missing_five_file[]` out; GAP-C05-01 registered as a tracked absent-source entry per C33.7 item 3).
3. **Grammar Constraint Mask:** reference URI patterns closed (workspace-relative + `~/.agents/skills` global per tool-use rules); absolute user paths (`/Users/...`) = violation (NF4/C10 portability).
4. **Tool Parser Required:** NO (graph walk + JSON validation, not language parsing).
5. **Definition/Purpose:** Operates A27 + C33.4/33.5 migration tracking; the single registry for absent sources (no side-channel gap lists).
6. **Draft Code:** (Bash — audit gate)

```bash
migration-audit --root MasterDesignDocument \
  --register-gap "GAP-C05-01:on-disk systems proof:pending" \
  | jq -e '.unresolved_refs == [] and .crosslink_breaks == []' || exit 1
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-06.01–06.05 | G3→C15.1/C15.2, G4→C15.2/C15.4, G5→C15.5, G6→C15.8 | A3, A8, A26, A27 (A16 shared) | C17.8, C18.4/18.10, C26.7/26.8, C33.7, C35.12 |
