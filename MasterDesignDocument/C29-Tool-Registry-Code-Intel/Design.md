# C29 — Typed Tool Registry & Code Intelligence

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Typed tool registry with JSON Schema params/results; code intelligence (Tree-sitter/LSP) gated by settings; risk levels as effect system; canonical tool surface with determinism guarantees.

## Subcategories (broad topics)

- C29.1 Built-in + code-intel + git write + web tools
- C29.2 Typed params/result schemas
- C29.3 Code intelligence (Tree-sitter/LSP) gated by settings
- C29.4 Risk levels and approval/hole binding
- C29.5 Telemetry (name/ok/latency/risk) and receipt hash

---

## Canonical Decisions (frozen for design)

| Decision                                                                                            | Rationale                                                           |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| **Tool = typed data** with JSON Schema for params/results; no dynamic `any`                         | Enables static verification, deterministic replay, schema evolution |
| **Risk levels** = effect system: `Pure` / `ReadOnly` / `WriteLocal` / `WriteRemote` / `Destructive` | Determines approval gate, hole binding, telemetry detail            |
| **Code intelligence** (Tree-sitter/LSP) opt-in via `configurations.json`; off by default            | No surprise resource usage; explicit user consent                   |
| **Tool resolution v2** validates refs deterministically; receipts hash-derived                      | Reproducible builds; audit trail                                    |
| **Canonical surface**: built-in tools registered in `tools.manifest.json` (compile-in, like skills) | Supply-chain integrity; single source of truth                      |

---

## Mathematical Specification

### 1. Tool as Typed Effect

A **tool** `t` is a tuple:

```
t = (id: ToolId, version: SemVer,
     params: JSONSchema, result: JSONSchema,
     effect: EffectLevel,
     caps: CapabilitySet,
     entry: WasmModule ⊕ NativeFn,
     provenance: ProvenanceToken)
```

**EffectLevel** ∈ { `Pure`, `ReadOnly`, `WriteLocal`, `WriteRemote`, `Destructive` }

Ordered by privilege: `Pure < ReadOnly < WriteLocal < WriteRemote < Destructive`.

**Effect semantics**:

- `Pure`: No side effects; deterministic; cacheable; no approval needed
- `ReadOnly`: Reads local state (FS, DB); no mutations; no approval
- `WriteLocal`: Mutates local state (files, DB); requires `WriteLocal` approval gate
- `WriteRemote`: Mutates remote state (API calls, git push); requires `WriteRemote` approval + hole
- `Destructive`: Irreversible (delete, format, deploy); requires `Destructive` approval + hole + human-in-loop

### 2. Tool Registry as Effect-Indexed Map

**Registry** `R: ToolId → Tool` with invariant:

```
∀t ∈ R: t.effect ≤ t.caps.max_effect
```

**Resolution v2** (deterministic):

```
resolve_v2(call: ToolCall) → (Tool ⊕ ResolutionError)
```

1. Lookup `call.tool_id` in `R` — exact match required (no fuzzy)
2. Validate `call.params` against `t.params` schema (JSON Schema Draft 2020-12)
3. Check `call.effect_requested ≤ t.effect` — caller cannot request lower effect than tool declares
4. Emit `ToolResolved` event (C31.1) with `tool_id`, `effect`, `params_hash`

**No dynamic tool registration** in ship builds. All tools in `tools.manifest.json`.

### 3. Code Intelligence as Gated Capability

**CodeIntelTool** = `{ "tree_sitter": TreeSitterTool, "lsp": LSPTool }`

**TreeSitterTool**:

- `parse(lang: Language, source: String) → CST`
- `query(lang: Language, query: String, cst: CST) → Matches`
- **Effect**: `ReadOnly` (reads source files)
- **Gated by**: `config.tools.code_intel.tree_sitter = true`

**LSPTool**:

- `hover(uri: URI, position: Position) → HoverInfo`
- `definition(uri: URI, position: Position) → Location[]`
- `references(uri: URI, position: Position) → Location[]`
- `completion(uri: URI, position: Position) → CompletionItem[]`
- **Effect**: `ReadOnly` (reads project, may spawn LSP server)
- **Gated by**: `config.tools.code_intel.lsp = true`

**LSP Server Management**:

- One server per language per workspace
- Spawned on first LSP tool call; shutdown on idle timeout (5 min)
- **Resource cap**: `max_lsp_servers = 4` (configurable)
- Telemetry: `lsp_server_spawn_total`, `lsp_request_duration_ms`

### 4. Approval and Hole Binding by Effect

| Effect        | Approval      | Hole Required | Seatbelt Profile                        |
| ------------- | ------------- | ------------- | --------------------------------------- |
| `Pure`        | None          | No            | `pure` (no syscalls)                    |
| `ReadOnly`    | None          | No            | `readonly` (read-only FS)               |
| `WriteLocal`  | Auto (policy) | Yes           | `write_local` (FS write, no net)        |
| `WriteRemote` | Explicit      | Yes           | `write_remote` (net allowed, allowlist) |
| `Destructive` | Human-in-loop | Yes           | `destructive` (full, audit logged)      |

**Approval policy** (per lane, in `configurations.json`):

```json
{
  "lane": "builder",
  "approvals": {
    "WriteLocal": "auto",
    "WriteRemote": "explicit",
    "Destructive": "human"
  }
}
```

**Hole execution** (C25): Tool runs in seatbelt sandbox; stdout/stderr captured; Merkle receipt emitted on exit.

### 5. Receipt Schema

**ToolReceipt** = `MerklePath(ToolCall, ToolResult, effect, caps_used, seatbelt_log_hash, timestamp)`

```json
{
  "tool_id": "git.commit",
  "call_hash": "sha256:...",
  "result_hash": "sha256:...",
  "effect": "WriteLocal",
  "caps_used": ["fs.write", "git"],
  "seatbelt_log_hash": "sha256:...",
  "timestamp": "2026-10-26T...",
  "merkle_root": "sha256:..."
}
```

**Verification**: `verify_receipt(receipt, tool_manifest) → bool` checks:

- `tool_id` exists in manifest with matching version
- `call_hash` = `sha256(call.params)`
- `result_hash` = `sha256(result)`
- `seatbelt_log_hash` matches hole execution log
- `merkle_root` = `merkle([call_hash, result_hash, effect, caps_used, seatbelt_log_hash, timestamp])`

---

## Built-In Tool Catalog (Frozen for Design)

| Tool ID           | Effect        | Params                                         | Result                                             | Caps             |
| ----------------- | ------------- | ---------------------------------------------- | -------------------------------------------------- | ---------------- |
| `fs.read`         | `ReadOnly`    | `{path: string}`                               | `{content: string}`                                | `fs.read`        |
| `fs.write`        | `WriteLocal`  | `{path: string, content: string}`              | `{ok: bool}`                                       | `fs.write`       |
| `fs.list`         | `ReadOnly`    | `{path: string}`                               | `{entries: string[]}`                              | `fs.read`        |
| `git.status`      | `ReadOnly`    | `{repo: string}`                               | `{status: string}`                                 | `git.read`       |
| `git.diff`        | `ReadOnly`    | `{repo: string}`                               | `{diff: string}`                                   | `git.read`       |
| `git.commit`      | `WriteLocal`  | `{repo: string, message: string}`              | `{hash: string}`                                   | `git.write`      |
| `git.push`        | `WriteRemote` | `{repo: string, remote: string}`               | `{ok: bool}`                                       | `git.write, net` |
| `web.fetch`       | `WriteRemote` | `{url: string, method: string, body?: string}` | `{status: int, body: string}`                      | `net`            |
| `web.search`      | `WriteRemote` | `{query: string, max_results: int}`            | `{results: SearchResult[]}`                        | `net, search`    |
| `code.parse`      | `ReadOnly`    | `{lang: string, source: string}`               | `{cst: CST}`                                       | `fs.read`        |
| `code.query`      | `ReadOnly`    | `{lang: string, query: string, cst: CST}`      | `{matches: Match[]}`                               | `fs.read`        |
| `code.hover`      | `ReadOnly`    | `{uri: string, position: Position}`            | `{hover: HoverInfo}`                               | `lsp`            |
| `code.definition` | `ReadOnly`    | `{uri: string, position: Position}`            | `{locations: Location[]}`                          | `lsp`            |
| `process.spawn`   | `Destructive` | `{cmd: string, args: string[], env: Map}`      | `{exit_code: int, stdout: string, stderr: string}` | `process`        |

---

## Verification Gates (C15)

| Gate                     | Test                                                    | Threshold |
| ------------------------ | ------------------------------------------------------- | --------- |
| Schema validation        | All tool params/results validate against JSON Schema    | 100%      |
| Effect monotonicity      | Caller effect ≤ tool effect for all resolutions         | 100%      |
| Deterministic resolution | Same call → same tool_id + effect (1000 iterations)     | 100%      |
| Receipt verification     | All receipts verify against manifest                    | 100%      |
| Hole isolation           | WriteRemote tool cannot escape allowlist                | 0 escapes |
| Code intel gating        | LSP/Tree-sitter off by default; on only when config set | 100%      |
| Manifest determinism     | Same source → same `tools.manifest.json` hash           | 100%      |

---

## Preregistration Entries (C09)

### A14 — Tool Registry Determinism

- Hypothesis: `resolve_v2` is deterministic; same call always resolves to same tool/effect.
- Baseline: Ad-hoc string-based tool dispatch.
- Judge: 10,000 random calls; 0 non-deterministic resolutions.
- Status: TBD.

### A15 — Effect System Soundness

- Hypothesis: No tool executes with effect exceeding its declared level.
- Judge: Static analysis + dynamic hole audit; 0 violations.
- Status: TBD.

### A16 — Code Intelligence Resource Bounds

- Hypothesis: LSP servers respect `max_lsp_servers` cap; no resource exhaustion.
- Judge: Stress test with 50 concurrent LSP requests; max 4 servers spawned.
- Status: TBD.

---

## Open Questions / Gaps

- Tool version negotiation for backward-compatible schema changes (deferred to P6).
- WebAssembly tool sandboxing vs native fn performance tradeoff (benchmarked in C21).
- Cross-lane tool capability inheritance (tied to C30/C34).

---

## Contributor Deepening — Muse (2026-10-26)

### C29.6 web.fetch as WriteRemote (depth: a read classified as a write)

The Built-In Tool Catalog classifies `web.fetch` AND `web.search` as WriteRemote — but fetch with method=GET mutates no remote state. The conservative classification is defensible (egress = exfiltration vector) but it has a cost: every RAG-adjacent web read (C26 UNVETTED tier) requires Explicit approval + hole, which prices honest research out of the tool loop. Frozen refinement [PROPOSED — refinement is my proposal; the effect ordering is frozen]:

1. Split `web.fetch` by method: GET/HEAD = ReadOnly (egress-logged, response-hashed, no approval beyond lane allowlist); POST/PUT/DELETE = WriteRemote (Explicit + hole, unchanged).
2. The exfiltration concern is met by the egress log (URL host + byte count + response hash in the receipt — counts/hashes only, ADR-06 clean), not by the approval gate. Approval gates stop accidents; egress logs catch malice. Conflating them prices out the legitimate use.
3. `web.search` stays WriteRemote (third-party query = data disclosure to the search provider by construction — the query text leaves the trust boundary).
4. This split changes the C26 UNVETTED tier economics: quarantined web reads become cheap (ReadOnly) while quarantined web writes stay gated. Record the reclassification in C36; A15 (effect soundness) gains a case: no GET-fetch executes with WriteRemote, no POST-fetch executes with ReadOnly.

### C29.7 process.spawn audit paradox (depth: the most dangerous tool has the thinnest receipt)

`process.spawn` is Destructive (human-in-loop + hole + audit) but its receipt schema carries only exit_code/stdout/stderr hashes — no command-line record. A Destructive tool whose receipt cannot answer "what command ran" fails C03.1 verifiable execution for the highest-risk action in the catalog. Frozen rule [PROPOSED — rule is my proposal; C03.1 is the authority]:

1. The `process.spawn` receipt MUST include the command + args hash (already have param_hash via call_hash — VERIFIED present in receipt schema) AND the resolved-binary identity (realpath + code-signature status on macOS). A hash of unknown provenance is not provenance.
2. The seatbelt profile for `destructive` (audit logged) must log the exec argv, not just allow/deny. An allow-decision without the argv is an alibi without a suspect.
3. Human-in-loop approval for process.spawn shows the human the FULL argv + working directory + env diff vs parent — approving a hash is not informed consent.

What was NOT checked here: current tool implementations (outside workspace); whether web.fetch method-splitting breaks existing callers (verify at implementation).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Typed Tool Registry (Original)

- **Built-in tools**: fs, git, web, code-intel, process.
- **Code intelligence** (Tree-sitter/LSP) gated by settings.
- **Risk levels**: Pure/ReadOnly/WriteLocal/WriteRemote/Destructive map to approval/hole.
- **Telemetry**: name/ok/latency/risk; receipt hash for mutating tools.
- **Resolve v2** validates tool refs deterministically; receipts hash-derived.
