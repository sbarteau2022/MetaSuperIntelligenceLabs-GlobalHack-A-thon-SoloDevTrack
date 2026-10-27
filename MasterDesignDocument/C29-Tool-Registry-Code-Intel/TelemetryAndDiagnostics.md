# C29 — Telemetry & Diagnostics

**Category:** C29-Tool-Registry-Code-Intel  
**Purpose:** Observability contract for tool registry and code intelligence.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `ToolResolved` | `tool_id`, `effect`, `params_hash`, `resolution_latency_ms` | ids/latency/hash only |
| `ToolExecuted` | `tool_id`, `effect`, `ok`, `duration_ms`, `seatbelt_profile` | ids/latency/ok only |
| `ToolApproval` | `tool_id`, `effect`, `policy: auto/explicit/human`, `granted` | ids/policy/result only |
| `ToolReceiptEmitted` | `tool_id`, `receipt_hash`, `merkle_root` | ids/hashes only |
| `CodeIntelRequested` | `tool_id` (parse/query/hover/etc), `lang`, `latency_ms` | ids/latency only |
| `LSPServerSpawned` | `lang`, `server_pid`, `spawn_latency_ms` | ids/latency only |
| `LSPServerShutdown` | `lang`, `idle_duration_ms` | ids/latency only |

**Never logged**: tool arguments, tool results, source code content, LSP responses, file paths beyond hash.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `tool_resolution_total` | Counter | `tool_id`, `result` | Resolution attempts |
| `tool_execution_duration_ms` | Histogram | `tool_id`, `effect` | Execution latency |
| `tool_execution_total` | Counter | `tool_id`, `effect`, `ok` | Execution count |
| `tool_approval_total` | Counter | `tool_id`, `effect`, `policy`, `granted` | Approval decisions |
| `tool_receipt_verification_total` | Counter | `result` | Receipt verify pass/fail |
| `code_intel_request_duration_ms` | Histogram | `tool_id`, `lang` | Code intel latency |
| `lsp_server_spawn_total` | Counter | `lang` | LSP spawns |
| `lsp_server_active` | Gauge | `lang` | Active servers |
| `lsp_request_duration_ms` | Histogram | `lang`, `method` | LSP request latency |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `tool.resolve` | ToolResolved | `tool_id`, `effect`, `latency` |
| `tool.execute` | ToolExecuted | `tool_id`, `effect`, `ok`, `latency` |
| `tool.approval` | ToolApproval | `tool_id`, `effect`, `policy`, `granted` |
| `tool.receipt` | ToolReceiptEmitted | `tool_id`, `receipt_hash` |
| `code_intel.request` | CodeIntelRequested | `tool_id`, `lang`, `latency` |
| `lsp.spawn` | LSPServerSpawned | `lang`, `latency` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Tool execution (WriteLocal+) | `ToolReceipt` | `call_hash + result_hash + effect + caps_used + seatbelt_log_hash + timestamp` |
| Tool approval | `ApprovalReceipt` | `tool_id + policy + granted + timestamp` |
| LSP server spawn | `LSPReceipt` | `lang + server_hash + timestamp` |

---

## Neural Map Roll-Up (C31.4)

End-of-day aggregation:
- Tool execution frequency by effect level
- Approval grant rate by policy type
- Average latency by tool and effect
- Code intelligence usage by language
- LSP server spawn/shutdown rate

**No raw arguments, results, or source code** in roll-up.

---

## Diagnostics Commands

```bash
# Tool registry health
curl localhost:8088/api/tools/health

# Manifest validation
cargo xtask tools-manifest --check

# Tool execution trace
curl localhost:8088/api/tools/{id}/trace?limit=100

# LSP server status
curl localhost:8088/api/code-intel/lsp/status

# Receipt verification audit
curl localhost:8088/api/tools/receipts/verify?since=24h
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — enforced by telemetry process schema validation.
- No tool arguments, results, file contents, source code, or LSP responses in any metric, tap, receipt, or log.
- File paths hashed (SHA-256) if included in receipts.
- Violations caught by `scripts/compliance_smoke.sh` (C15).
