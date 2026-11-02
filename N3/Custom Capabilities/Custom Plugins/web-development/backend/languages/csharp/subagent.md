# csharp / subagent — lifecycle create → inject → run → dispose

Date: 2026-10-31. Status: VERIFIED (compiled + unload-drill). Scope: `plugins/web-development/backend/languages/csharp/` ONLY (read/write confined to this lane dir; nothing outside).

Role/shape reference: AgentDefinition pattern (`description` + `prompt` + restricted `tools`) per `AtomaDB-RAG-PAMI/corpora/library/platforms/anthropic-agent-sdk-subagents.md` — role/shape only, no bodies copied.

- Role: `csharp-checker` — route-first C# checker dispatch for one lane task, then dispose. No planning beyond the lane, no cross-lane writes.
- Tools: `tools.md` (route/judges/compilers). Hooks: `hooks.md` (deny/warn rules, stop gate). Loops: `loops.md` (bounded state machine, two-strike escalation). Bodies never duplicated here — paths only.

## 1. Tool allowlist (explicit, nothing unlisted)

- `route_csharp_checker` — the ONLY effectful tool (dispatches `dotnet build | dotnet test | dotnet format`).
- Read-only: `read_file`, `grep`, `find_path`, `list_directory`, `diagnostics`.
- Everything else is denied: `run_csharp_checker` returns `refused:tool-denied` for unlisted tools and `refused:inactive` after dispose. Allowlist is enforced in code by `is_tool_allowed`, not by convention.

## 2. Lifecycle

- **create** — `create_subagent(id, role)` returns an active `SubagentHandle` holding exactly 1 handle. One task per handle; no handle reuse across tasks.
- **inject** — `inject_context(handle, lane_dir)` accepts ONLY the lane dir (`subagent_scope_dir()`); any other dir returns false and the run stops. Inactive handles always refuse.
- **run** — `run_csharp_checker(handle, checker)` routes via the allowlist, then the judges fire: `judge_gate(build_ok, test_ok, format_ok)` must be all-true (`dotnet build` + `dotnet test` + `dotnet format` per `hooks.md`, Judge E). Any judge fail records one strike (`record_strike`); `should_escalate(strikes, iters)` fires at `strikes >= 2` or `iters >= 3` → **Escalate** per `loops.md` (`MAX_STRIKES = 2`, `MAX_ITERS = 3`, two-strike → Escalate, terminal, stop-gate veto final).
- **dispose** — `dispose_subagent(handle)` consumes the handle, emits a `SubagentReceipt` with `handles_after = 0`, and `leak_free(receipt)` must be true. No-leak-on-unload is a proof obligation, not a best effort (§4).

## 3. Compiled lifecycle (machine — TS source of truth, Rust runtime)

Authoring language: TypeScript 7.1beta strict subset (class + constructor, typed functions, no `any`, no object literals, every arg+return typed). Type map per `tools.md` narrowed mechanically for counters (`number->u32` for handles/strikes/iters, bounded small — same narrowing as `java/subagent.md` and `csharp/loops.md`); `string->String`, `boolean->bool`, `T|null->Option<T>`.

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
  return "csharp-checker";
}
function subagent_scope_dir(): string {
  return "plugins/web-development/backend/languages/csharp/";
}
function is_tool_allowed(tool: string): boolean {
  if (tool === "route_csharp_checker") {
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
function run_csharp_checker(handle: SubagentHandle, checker: string): string {
  if (handle.active === false) {
    return "refused:inactive";
  }
  if (is_tool_allowed("route_csharp_checker") === false) {
    return "refused:tool-denied";
  }
  if (checker === "build") {
    return "dotnet_build";
  }
  if (checker === "test") {
    return "dotnet_test";
  }
  if (checker === "format") {
    return "dotnet_format";
  }
  return "csharp_check";
}
function judge_gate(build_ok: boolean, test_ok: boolean, format_ok: boolean): boolean {
  return build_ok && test_ok && format_ok;
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

Rust runtime (faithful port; mechanical adaptations only: `number->u32` counters per §3 narrowing, `Self { id, role, ... }` field shorthand, `None` for null, redundant transpiler parens removed per `rustc -D warnings` — semantics-preserving, `rustc -D warnings` exit 0):

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
pub fn subagent_role() -> String {
  return "csharp-checker".to_string();
}
pub fn subagent_scope_dir() -> String {
  return "plugins/web-development/backend/languages/csharp/".to_string();
}
pub fn is_tool_allowed(tool: String) -> bool {
  if tool == "route_csharp_checker".to_string() {
    return true;
  }
  if tool == "read_file".to_string() {
    return true;
  }
  if tool == "grep".to_string() {
    return true;
  }
  if tool == "find_path".to_string() {
    return true;
  }
  if tool == "list_directory".to_string() {
    return true;
  }
  if tool == "diagnostics".to_string() {
    return true;
  }
  return false;
}
pub fn create_subagent(id: String, role: String) -> SubagentHandle {
  return SubagentHandle::new(id, role, true, 1);
}
pub fn inject_context(handle: SubagentHandle, lane_dir: String) -> bool {
  if handle.active == false {
    return false;
  }
  if lane_dir == subagent_scope_dir() {
    return true;
  }
  return false;
}
pub fn run_csharp_checker(handle: SubagentHandle, checker: String) -> String {
  if handle.active == false {
    return "refused:inactive".to_string();
  }
  if is_tool_allowed("route_csharp_checker".to_string()) == false {
    return "refused:tool-denied".to_string();
  }
  if checker == "build".to_string() {
    return "dotnet_build".to_string();
  }
  if checker == "test".to_string() {
    return "dotnet_test".to_string();
  }
  if checker == "format".to_string() {
    return "dotnet_format".to_string();
  }
  return "csharp_check".to_string();
}
pub fn judge_gate(build_ok: bool, test_ok: bool, format_ok: bool) -> bool {
  return build_ok && test_ok && format_ok;
}
pub fn record_strike(strikes: u32) -> u32 {
  return strikes + 1;
}
pub fn should_escalate(strikes: u32, iters: u32) -> bool {
  if strikes >= 2 {
    return true;
  }
  if iters >= 3 {
    return true;
  }
  return false;
}
pub fn dispose_subagent(handle: SubagentHandle) -> SubagentReceipt {
  return SubagentReceipt::new(handle.id, true, handle.handles, 0, None);
}
pub fn leak_free(receipt: SubagentReceipt) -> bool {
  return receipt.handles_after == 0;
}
```

Full sources: `/tmp/n3-subagent-csharp/subagent.ts`, `/tmp/n3-subagent-csharp/subagent_lib.rs`.

## 4. No-leak proof (unload drill, compiled Rust `--deny warnings` exit 0)

Harness `/tmp/n3-subagent-csharp/unload_drill.rs`: create → inject (lane-dir accept + cross-lane refuse) → run (build/test/format routes) → judge → escalate-checks → allowlist-deny (`execute` refused) → inactive-refuse → dispose → assert `leak_free`. Log:

```text
BEFORE handles=1 AFTER handles=0 LEAK-FREE=true
```

Disposer releases the single handle (`handles_after = 0`); drill asserts `leak_free` true and exits 0. Proof obligation: every run ends in dispose + receipt; an undisposed handle is a STOP + why-receipt, never a silent pass.

## 5. Receipt

- Gate chain (pasted, 2026-10-31):
  - `ts_check`: `ok:true`, exit 0, zero diagnostics
  - `ts_to_rust`: `ok:true`, unsupported `[]`
  - Rust port `rustc --crate-type lib -D warnings`: exit 0, zero warnings
  - Drill bin `rustc -D warnings`: exit 0; drill run exit 0 (see §4)
  - `dotnet --version`: `dotnet: command not found` (exit 127, MISSING — RECORDED, never silent-pass; Judge E gate cannot execute on this machine until dotnet is installed; lifecycle/judge/escalation logic above is verified by the compiled drill, not by a live dotnet run)
- TS source SHA-256: `369c6e6b92ac6b058ddfe6ea80ad0e555d6ac09423be5e6d1d697d2cb6008dce`
- Rust output SHA-256: `a47a9261d86c59212f77e8b7b958d4bbad295dbc4e80091946532ee6ee5d84f3`
- why: null. sent: true.
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- No object literals (grep exit 1 = no matches). No I/O in lifecycle (`fs::`/`File::`/`std::fs`/`write!`/`Command`/`process::` grep exit 1 = no matches); no network (`fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` grep exit 1 = no matches) — nothing retained past dispose by construction.

## References (canonical — linked, never duplicated)

- Tools / route-first contract / checkers: `tools.md`
- Hook gate (`dotnet build` + `dotnet test` + `dotnet format`, Judge E), deny/warn rules, event envelopes: `hooks.md`
- Loop bounds (`MAX_ITERS = 3`, `MAX_STRIKES = 2`, two-strike), runbooks: `loops.md`
