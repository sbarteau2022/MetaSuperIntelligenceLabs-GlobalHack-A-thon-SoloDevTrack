# ToolingC04 — Tooling Certification for SKL-04 (Observability & Telemetry Engineering)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-04` (`SkillsC1.md`) — Observability & Telemetry Engineering.
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS

**Necessity re-verified [VERIFIED — `C31` deepened spec read directly]:** ledger + telemetry process + metrics + taps + receipts + roll-up + privacy schema is the evidence substrate for every gate. Without it, G5/G6/G7/G8 have no inputs (metric-evidence gate C15.5 reads `CLAIMS_REGISTER.md`, which telemetry populates). A22–A25 are telemetry measurements. Necessity holds.

**Gate mapping correction (Scheme B → dual citation, per C15.9):**

| Day gate (SkillsC1) | Artifact gates (C15.x) operated |
| --- | --- |
| G5 Contract + Property | C15.5 (claims-evidence check) |
| G6 Property + Chaos | C15.8 (property subset) |
| G7 Chaos/Recovery + Performance | C15.8 (chaos subset) |
| G8 Performance + Build Receipt | C15.7 (receipt) + C15.8 (bench subset) |

**Ablations:** A22–A25 — no delta.

**Conditions:**

1. Ledger payload is a RESTRICTED store (P-CORR-12): TOOL-04.01 readers limited to telemetry-process / Admin-export / crash-replay. Any fourth reader is a privacy incident (A23 class), not a feature.
2. Ledger encryption-at-rest decision REQUIRED before P5 (C31.8 item 3 residual risk): unencrypted SQLite holding A1/A2 payloads fails the C16.1 DPIA row. Record the decision (FileVault/APFS assumption + validation, or column-level encryption with secret-slot key).
3. RingBuffer labeled LOSSY in every UI surface consuming it (C31.9); alerting taps ledger-first (P-CORR-13-class ordering: ledger write precedes alert emit — same failure class as C18.9 TTFT-before-tokens).
4. `grafana`/`vector`/`fluent-bit` are PRESENTATION/SHIPPING, not evidence: dashboards never cited in claims; only ledger + receipts + roll-up JSON are citable. Operators trained on this boundary or A25 comparisons get dashboard-screen-shotted instead of ledger-compared.

---

## Tool Specs

### TOOL-04.01 `ledger-append` — Append-only event writer + Merkle sealer

1. **Tool Name:** `ledger-append` (SQLite WAL writer; Merkle root every 1000 entries per CORR-10)
2. **Tool Structure:** typed-JSON (event_type + payload_hash + clock in — PAYLOAD NEVER ON THIS WIRE; `seq` + `prev_hash` + receipt out).
3. **Grammar Constraint Mask:** event_type enum-closed (unknown type = 400); payload_hash format `sha256:[0-9a-f]{64}` enforced pre-write; UPDATE/DELETE statements absent from the tool binary by construction (audited at Gate 2 — grep for `UPDATE|DELETE` in tool source = build fail).
4. **Tool Parser Required:** NO (hashes in, hashes out; the tool never parses content — the privacy boundary IS the architecture).
5. **Definition/Purpose:** Sole writer to `events.sqlite`; maintains hash chain + Merkle schedule; emits `TelemetryPrivacyViolation` (compliant, ids-only) and drops any emit failing the C31.7 privacy schema.
6. **Draft Code:** (Rust — append with chain)

```rust
fn append(ev: &LedgerEmit) -> Result<LedgerReceipt> {
    validate_privacy_schema(ev)?;                    // fail closed on content leak
    let seq = next_seq()?; let prev = head_hash()?;
    let h = sha256(&(seq, &ev.type_id, &ev.payload_hash, &prev));
    insert(seq, ev, h)?; if seq % 1000 == 0 { seal_merkle_root()?; }
    Ok(LedgerReceipt { seq, hash: h })
}
```

### TOOL-04.02 `telemetry-proc` — Separate-PID health + session supervisor

1. **Tool Name:** `telemetry-proc` (independent process: honest availability, session persistence)
2. **Tool Structure:** typed-JSON (health-check envelope: `ledger_writable`, `metrics_scrapable`, `oom_60s`, `merkle_ok` in; `healthy: bool` + `/healthz` + `/readyz` out).
3. **Grammar Constraint Mask:** read-only SQLite handle (open with `mode=ro` — write attempt from telemetry process = process bug, fail loudly); session resume keyed by `config_hash` (config changed → new session, no silent carry-over).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** ADR-16 honest availability; survives agent restarts with persisted session; its OWN OOM/health is reported (self-monitoring, not self-exemption).
6. **Draft Code:** (Bash — supervision contract)

```bash
sqlite3 "file:$LEDGER?mode=ro" "SELECT verify_chain();" || healthy=false
telemetry-proc --healthz --expect healthy="$healthy" --session "$SESSION_ID"
# healthy=false => orchestrator pages; telemetry never silently degrades
```

### TOOL-04.03 `prom-export` — Prometheus text exporter with cardinality guard

1. **Tool Name:** `prom-export` (`/metrics` endpoint; <10,000 active series enforced at registration)
2. **Tool Structure:** typed-JSON registration envelope per metric (`name`, `type`, `labels[]` — label VALUES constrained: lowercase alnum+underscore, ≤64 chars, no raw IDs/hashes/paths per C31.3).
3. **Grammar Constraint Mask:** registration-time cardinality estimator (projected series > 10k → registration REJECTED with `cardinality_blocked` receipt — A24's enforcement arm).
4. **Tool Parser Required:** YES — `prom-text-parser` (emitted exposition format validated as Prometheus text pre-serve; malformed exposition never served).
5. **Definition/Purpose:** Only metrics path the UI/scraper may read; label discipline is the A23 privacy boundary for metrics (labels are the leak vector, not values).
6. **Draft Code:** (Rust — guarded registration)

```rust
fn register(m: &MetricDef) -> Result<()> {
    for v in &m.label_values { validate_label(v)?; }   // no IDs/hashes/paths
    if projected_series() + m.series() > 10_000 { return Err(CardinalityBlocked); }
    REGISTRY.insert(m)
}
```

### TOOL-04.04 `rollup-daily` — Neural-map aggregation writer

1. **Tool Name:** `rollup-daily` (02:00 UTC aggregation → `neural_map_YYYYMMDD.json`)
2. **Tool Structure:** typed-JSON (date window in; aggregates-only JSON out — counts/rates/percentiles/unique-counts; NO raw payloads/hashes/IDs per C31.6).
3. **Grammar Constraint Mask:** output schema validated pre-write (any `payload`/`text`/`content` key present → write REJECTED + `RollupPrivacyViolation` receipt — A23 covers the roll-up path too).
4. **Tool Parser Required:** NO (reads ledger via TOOL-04.01 read path; writes JSON against schema).
5. **Definition/Purpose:** Produces the A25 comparison artifact; `restart_seam` marker handling on post-crash backfill (C31.9 item 2).
6. **Draft Code:** (Python — aggregates only)

```python
def rollup(day: str) -> Path:
    rows = ledger_read(day)  # hashes only, never payload
    out = {"date": day, "event_counts": count(rows), "latency_p50_ms": p50(rows),
           "latency_p99_ms": p99(rows), "restart_seams": seams(rows)}
    validate(out, ROLLUP_SCHEMA)  # content key => reject
    return write(f"neural_map_{day}.json", out)
```

### TOOL-04.05 `privacy-fuzz` — Schema-violation fuzzer (A23 operator)

1. **Tool Name:** `privacy-fuzz` (10,000 fuzzed emits against the C31.7 privacy schema)
2. **Tool Structure:** typed-JSON (fuzz config: `count`, `mutators[]`, `seed` — seed REQUIRED per reproducibility rule in; `violations[]` + pass/fail out).
3. **Grammar Constraint Mask:** mutators drawn from a CLOSED set (arg-injection, result-injection, text-injection, path-injection, hash-format-break); unknown mutator = 400.
4. **Tool Parser Required:** YES — `violation-classifier` (maps schema failures to `leak_class` for RB-TELEM-04 triage).
5. **Definition/Purpose:** Executes the A23 judge; every clean run re-certifies the ADR-06 boundary after each C09.8 trigger event (new skill/tool/lane = re-fuzz).
6. **Draft Code:** (Bash — judge run)

```bash
privacy-fuzz --count 10000 --seed "$SEED" --mutators args,results,text,paths \
  | violation-classifier --out a23_result.json
test "$(jq .violations a23_result.json)" = "[]" || exit 1  # any violation => A23 FAIL
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-04.01–04.05 | G5→C15.5, G6/G7→C15.8, G8→C15.7/C15.8 | A22–A25 | C31.7–31.9, CORR-10, P-CORR-12 |
