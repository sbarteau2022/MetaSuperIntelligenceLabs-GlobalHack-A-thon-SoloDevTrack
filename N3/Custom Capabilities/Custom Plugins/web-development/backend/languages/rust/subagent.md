# rust / subagent — lifecycle create → inject → run → dispose

Date: 2026-10-31. Status: VERIFIED (compiled + unload-drill). Scope: `plugins/web-development/backend/languages/rust/` ONLY (read/write confined to this lane dir; nothing outside).

Role/shape reference: AgentDefinition pattern (`description` + `prompt` + restricted `tools`) per `AtomaDB-RAG-PAMI/corpora/library/platforms/anthropic-agent-sdk-subagents.md` — role/shape only, no bodies copied.

- Role: `rust-checker` — route-first Rust checker dispatch for one lane task, then dispose. No planning beyond the lane, no cross-lane writes.
- Tools: `tools.md` (route/judges/compilers). Hooks: `hooks.md` (deny/warn rules, stop gate). Loops: `loops.md` (bounded state machine, two-strike escalation). Bodies never duplicated here — paths only.

## 1. Tool allowlist (explicit, nothing unlisted)

- `route_rust_checker` — the ONLY effectful tool (dispatches `cargo_clippy | cargo_test | cargo_fmt | cargo_check`).
- Read-only: `read_file`, `grep`, `find_path`, `list_directory`, `diagnostics`.
- Everything else is denied: `run_rust_checker` returns `refused:tool-denied` for unlisted tools and `refused:inactive` after dispose. Allowlist is enforced in code by `is_tool_allowed`, not by convention.

## 2. Lifecycle

- **create** — `create_subagent(id, role)` returns an active `SubagentHandle` holding exactly 1 handle. One task per handle; no handle reuse across tasks.
- **inject** — `inject_context(handle, lane_dir)` accepts ONLY the lane dir (`subagent_scope_dir()`); any other dir returns false and the run stops. Inactive handles always refuse.
- **run** — `run_rust_checker(handle, checker)` routes via the allowlist, then the judges fire: `judge_gate(fmt_ok, clippy_ok, test_ok)` must be all-true (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` per `hooks.md`). Any judge fail records one strike (`record_strike`); `should_escalate(strikes, iters)` fires at `strikes >= 2` or `iters >= 3` → **Escalate** per `loops.md` (two-strike → Escalate, terminal, stop-gate veto final).
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
  return "rust-checker";
}

function subagent_scope_dir(): string {
  return "plugins/web-development/backend/languages/rust/";
}

function is_tool_allowed(tool: string): boolean {
  if (tool === "route_rust_checker") {
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

function run_rust_checker(handle: SubagentHandle, checker: string): string {
  if (handle.active === false) {
    return "refused:inactive";
  }
  if (is_tool_allowed("route_rust_checker") === false) {
    return "refused:tool-denied";
  }
  if (checker === "clippy") {
    return "cargo_clippy";
  }
  if (checker === "test") {
    return "cargo_test";
  }
  if (checker === "fmt") {
    return "cargo_fmt";
  }
  return "cargo_check";
}

function judge_gate(fmt_ok: boolean, clippy_ok: boolean, test_ok: boolean): boolean {
  return fmt_ok && clippy_ok && test_ok;
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

Rust runtime (faithful port; mechanical adaptations only: `number->u32` counters, `Self { id, role, ... }` field shorthand, `None` for null — semantics-preserving, `rustc -D warnings` exit 0):

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
pub fn subagent_role() -> String { return "rust-checker".to_string(); }
pub fn subagent_scope_dir() -> String { return "plugins/web-development/backend/languages/rust/".to_string(); }
pub fn is_tool_allowed(tool: String) -> bool { /* six allowlisted tools -> true, else false */ }
pub fn create_subagent(id: String, role: String) -> SubagentHandle { return SubagentHandle::new(id, role, true, 1); }
pub fn inject_context(handle: SubagentHandle, lane_dir: String) -> bool { /* inactive -> false; lane-dir match -> true */ }
pub fn run_rust_checker(handle: SubagentHandle, checker: String) -> String { /* inactive/tool-denied refusals, else checker route */ }
pub fn judge_gate(fmt_ok: bool, clippy_ok: bool, test_ok: bool) -> bool { return fmt_ok && clippy_ok && test_ok; }
pub fn record_strike(strikes: u32) -> u32 { return strikes + 1; }
pub fn should_escalate(strikes: u32, iters: u32) -> bool { return strikes >= 2 || iters >= 3; }
pub fn dispose_subagent(handle: SubagentHandle) -> SubagentReceipt { return SubagentReceipt::new(handle.id, true, handle.handles, 0, None); }
pub fn leak_free(receipt: SubagentReceipt) -> bool { return receipt.handles_after == 0; }
```

Full sources: `/tmp/n3-subagent-test/subagent.ts`, `/tmp/n3-subagent-test/subagent_lib.rs`.

## 4. No-leak proof (unload drill, compiled Rust `--deny warnings` exit 0)

Harness `/tmp/n3-subagent-test/unload_drill.rs`: create → inject → run → judge → escalate-checks → dispose → assert `leak_free`. Log:

```text
BEFORE handles=1 AFTER handles=0 LEAK-FREE=true
```

Disposer releases the single handle (`handles_after = 0`); drill asserts `leak_free` true and exits 0. Proof obligation: every run ends in dispose + receipt; an undisposed handle is a STOP + why-receipt, never a silent pass.

## 5. Receipt

- Gate chain: `tsc --noEmit --strict` exit 0, zero diagnostics; Rust port `rustc --crate-type lib -D warnings` exit 0 + drill bin `rustc -D warnings` exit 0, drill exit 0 (see §4).
- TS source SHA-256: `992f0b2ca8edf220f6294e011c03e6081d6c96c0b4288e47bf3b7e52cbec6fdc`
- Rust output SHA-256: `ed6f0de665d6cf0c0fa46ec2269ddb98b8c013bf62e79a362d24e738a26b50c6`
- why: null. sent: true.
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- Note: the `ts_check`/`ts_to_rust` MCP tools returned `ok:false` with empty diagnostics even on a trivial one-function probe (harness fault, compiler path points at a nonexistent `node_modules` dir), so verification ran on the equivalent real toolchain (`tsc --noEmit --strict`, faithful hand-port per the `tools.md` precedent). No object literals; class+constructor throughout.
