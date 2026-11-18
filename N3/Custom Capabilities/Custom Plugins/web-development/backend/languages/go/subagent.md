# go / subagent — lifecycle create → inject → run → dispose

Date: 2026-10-31. Status: VERIFIED (compiled + unload-drill). Scope: `plugins/web-development/backend/languages/go/` ONLY (read/write confined to this lane dir; nothing outside).

Role/shape reference: AgentDefinition pattern (`description` + `prompt` + restricted `tools`) per `AtomaDB-RAG-PAMI/corpora/library/platforms/anthropic-agent-sdk-subagents.md` — role/shape only, no bodies copied.

- Role: `go-checker` — route-first Go checker dispatch for one lane task, then dispose. No planning beyond the lane, no cross-lane writes.
- Tools: `tools.md` (route/judges/compilers). Hooks: `hooks.md` (deny/warn rules, stop gate). Loops: `loops.md` (bounded state machine, two-strike escalation). Bodies never duplicated here — paths only.

## 1. Tool allowlist (explicit, nothing unlisted)

- `route_go_checker` — the ONLY effectful tool (dispatches `go vet | go test | gofmt | go check`).
- Read-only: `read_file`, `grep`, `find_path`, `list_directory`, `diagnostics`.
- Everything else is denied: `run_go_checker` returns `refused:tool-denied` for unlisted tools and `refused:inactive` after dispose. Allowlist is enforced in code by `is_tool_allowed`, not by convention.

## 2. Lifecycle

- **create** — `create_subagent(id, role)` returns an active `SubagentHandle` holding exactly 1 handle. One task per handle; no handle reuse across tasks.
- **inject** — `inject_context(handle, lane_dir)` accepts ONLY the lane dir (`subagent_scope_dir()`); any other dir returns false and the run stops. Inactive handles always refuse.
- **run** — `run_go_checker(handle, checker)` routes via the allowlist, then the judges fire: `judge_gate(vet_ok, fmt_ok, test_ok)` must be all-true (`go vet ./...`, `gofmt -l` clean, `go test` per `hooks.md`). Any judge fail records one strike (`record_strike`); `should_escalate(strikes, iters)` fires at `strikes >= 2` or `iters >= 3` → **Escalate** per `loops.md` (`MAX_STRIKES = 2`, `MAX_ITERS = 3`, two-strike → Escalate, terminal, stop-gate veto final).
- **dispose** — `dispose_subagent(handle)` consumes the handle, emits a `SubagentReceipt` with `handles_after = 0`, and `leak_free(receipt)` must be true. No-leak-on-unload is a proof obligation, not a best effort (§3).

## 3. Compiled lifecycle (machine — TS source of truth, Rust runtime)

Authoring language: TypeScript 7.1beta strict subset (class + constructor, typed functions, no `any`, no object literals, every arg+return typed). Type map per `tools.md`: `number->u32` (counters: handles/strikes/iters, bounded small), `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
class SubagentHandle {
  id: string;
  role: string;
  active: boolean;
  handles: number;
  constructor(id: string, role: string, active: boolean, handles: number) {
    this.id = id;
    this.role = role;
    this.active = active;
    this.handles = handles;
  }
}

class SubagentReceipt {
  subagent_id: string;
  disposed: boolean;
  handles_before: number;
  handles_after: number;
  why: string | null;
  constructor(subagent_id: string, disposed: boolean, handles_before: number, handles_after: number, why: string | null) {
    this.subagent_id = subagent_id;
    this.disposed = disposed;
    this.handles_before = handles_before;
    this.handles_after = handles_after;
    this.why = why;
  }
}

function subagent_role(): string {
  return "go-checker";
}

function subagent_scope_dir(): string {
  return "plugins/web-development/backend/languages/go/";
}

function is_tool_allowed(tool: string): boolean {
  if (tool === "route_go_checker") {
    return true;
  }
  if (tool === "read_file") {
    return true;
  }
  if (tool === "grep") {
    return true;
  }
  if (tool === "find_path") {
    return true;
  }
  if (tool === "list_directory") {
    return true;
  }
  if (tool === "diagnostics") {
    return true;
  }
  return false;
}

function create_subagent(id: string, role: string): SubagentHandle {
  return new SubagentHandle(id, role, true, 1);
}

function inject_context(handle: SubagentHandle, lane_dir: string): boolean {
  if (handle.active === false) {
    return false;
  }
  if (lane_dir === subagent_scope_dir()) {
    return true;
  }
  return false;
}

function run_go_checker(handle: SubagentHandle, checker: string): string {
  if (handle.active === false) {
    return "refused:inactive";
  }
  if (is_tool_allowed("route_go_checker") === false) {
    return "refused:tool-denied";
  }
  if (checker === "vet") {
    return "go_vet";
  }
  if (checker === "test") {
    return "go_test";
  }
  if (checker === "fmt") {
    return "go_fmt";
  }
  return "go_check";
}

function judge_gate(vet_ok: boolean, fmt_ok: boolean, test_ok: boolean): boolean {
  return vet_ok && fmt_ok && test_ok;
}

function record_strike(strikes: number): number {
  return strikes + 1;
}

function should_escalate(strikes: number, iters: number): boolean {
  if (strikes >= 2) {
    return true;
  }
  if (iters >= 3) {
    return true;
  }
  return false;
}

function dispose_subagent(handle: SubagentHandle): SubagentReceipt {
  return new SubagentReceipt(handle.id, true, handle.handles, 0, null);
}

function leak_free(receipt: SubagentReceipt): boolean {
  return receipt.handles_after === 0;
}
```

Rust runtime (faithful port; mechanical adaptations only: `number->u32` counters per `tools.md` type map, `Self { id, role, ... }` field shorthand, `None` for null, redundant transpiler parens removed per `rustc -D warnings` — semantics-preserving, `rustc -D warnings` exit 0):

```rust
#[derive(Debug, Clone)]
pub struct SubagentHandle {
  pub id: String,
  pub role: String,
  pub active: bool,
  pub handles: u32,
}
#[derive(Debug, Clone)]
pub struct SubagentReceipt {
  pub subagent_id: String,
  pub disposed: bool,
  pub handles_before: u32,
  pub handles_after: u32,
  pub why: Option<String>,
}
impl SubagentHandle {
  pub fn new(id: String, role: String, active: bool, handles: u32) -> Self {
    Self { id, role, active, handles }
  }
}
impl SubagentReceipt {
  pub fn new(subagent_id: String, disposed: bool, handles_before: u32, handles_after: u32, why: Option<String>) -> Self {
    Self { subagent_id, disposed, handles_before, handles_after, why }
  }
}
pub fn subagent_role() -> String { return "go-checker".to_string(); }
pub fn subagent_scope_dir() -> String { return "plugins/web-development/backend/languages/go/".to_string(); }
pub fn is_tool_allowed(tool: String) -> bool { /* six allowlisted tools -> true, else false */ }
pub fn create_subagent(id: String, role: String) -> SubagentHandle { return SubagentHandle::new(id, role, true, 1); }
pub fn inject_context(handle: SubagentHandle, lane_dir: String) -> bool { /* inactive -> false; lane-dir match -> true */ }
pub fn run_go_checker(handle: SubagentHandle, checker: String) -> String { /* inactive/tool-denied refusals, else checker route */ }
pub fn judge_gate(vet_ok: bool, fmt_ok: bool, test_ok: bool) -> bool { return vet_ok && fmt_ok && test_ok; }
pub fn record_strike(strikes: u32) -> u32 { return strikes + 1; }
pub fn should_escalate(strikes: u32, iters: u32) -> bool { return strikes >= 2 || iters >= 3; }
pub fn dispose_subagent(handle: SubagentHandle) -> SubagentReceipt { return SubagentReceipt::new(handle.id, true, handle.handles, 0, None); }
pub fn leak_free(receipt: SubagentReceipt) -> bool { return receipt.handles_after == 0; }
```

Full sources: `/tmp/n3-subagent-go/subagent.ts`, `/tmp/n3-subagent-go/subagent_lib.rs`.

## 4. No-leak proof (unload drill, compiled Rust `--deny warnings` exit 0)

Harness `/tmp/n3-subagent-go/unload_drill.rs`: create → inject (lane-dir accept + cross-lane refuse) → run → judge → escalate-checks → allowlist-deny (`execute` refused) → dispose → assert `leak_free`. Log:

```text
BEFORE handles=1 AFTER handles=0 LEAK-FREE=true
```

Disposer releases the single handle (`handles_after = 0`); drill asserts `leak_free` true and exits 0. Proof obligation: every run ends in dispose + receipt; an undisposed handle is a STOP + why-receipt, never a silent pass.

## 5. Receipt

- Gate chain: `ts_check` exit 0, zero diagnostics; `ts_to_rust` `ok:true`, unsupported `[]`; `tsc --noEmit --strict` exit 0; Rust port `rustc --crate-type lib -D warnings` exit 0 + drill bin `rustc -D warnings` exit 0, drill exit 0 (see §4).
- TS source SHA-256: `a792bcef106b066691b945e93cb26d540bd5158d35fb7b423de68bfcd80ddc79`
- Rust output SHA-256: `4b809ce3b5de338ff66aa249f593f084a1624054972acb0b83e3f8883bd88e40`
- why: null. sent: true.
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- No object literals; class+constructor throughout (grep exit 1 = no matches). No I/O in lifecycle (`fs::`/`File::`/`std::fs`/`write!`/`Command`/`process::` grep exit 1 = no matches); no network (`fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` grep exit 1 = no matches) — nothing retained past dispose by construction.

## References (canonical — linked, never duplicated)

- Tools / route-first contract / checkers: `tools.md`
- Hook gate (`go vet` + `gofmt -l`), deny/warn rules, event envelopes: `hooks.md`
- Loop bounds (`MAX_ITERS = 3`, `MAX_STRIKES = 2`, two-strike), runbooks: `loops.md`
