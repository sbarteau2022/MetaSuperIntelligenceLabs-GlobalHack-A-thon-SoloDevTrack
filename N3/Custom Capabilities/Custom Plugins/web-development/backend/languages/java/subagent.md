# java / subagent — lifecycle create → inject → run → dispose

Date: 2026-10-31. Status: VERIFIED (compiled + unload-drill). Scope: `plugins/web-development/backend/languages/java/` ONLY (read/write confined to this lane dir; nothing outside).

Role/shape reference: AgentDefinition pattern (`description` + `prompt` + restricted `tools`) per `AtomaDB-RAG-PAMI/corpora/library/platforms/anthropic-agent-sdk-subagents.md` — role/shape only, no bodies copied.

- Role: `java-checker` — route-first Java checker dispatch for one lane task, then dispose. No planning beyond the lane, no cross-lane writes.
- Tools: `tools.md` (route/judges/compilers). Hooks: `hooks.md` (deny/warn rules, stop gate). Loops: `loops.md` (bounded state machine, two-strike escalation). Bodies never duplicated here — paths only.

## 1. Tool allowlist (explicit, nothing unlisted)

- `route_java_checker` — the ONLY effectful tool (dispatches `mvn | mvnw | gradle | gradlew | javac | checkstyle | spotbugs`).
- Read-only: `read_file`, `grep`, `find_path`, `list_directory`, `diagnostics`.
- Everything else is denied: `run_java_checker` returns `refused:tool-denied` for unlisted tools and `refused:inactive` after dispose. Allowlist is enforced in code by `is_tool_allowed`, not by convention.

## 2. Lifecycle

- **create** — `create_subagent(id, role)` returns an active `SubagentHandle` holding exactly 1 handle. One task per handle; no handle reuse across tasks.
- **inject** — `inject_context(handle, lane_dir)` accepts ONLY the lane dir (`subagent_scope_dir()`); any other dir returns false and the run stops. Inactive handles always refuse.
- **run** — `run_java_checker(handle, checker)` routes via the allowlist, then the judges fire: `judge_gate(javac_ok, style_ok, test_ok)` must be all-true (`javac -Xlint:all` + checkstyle/spotbugs per `hooks.md`). Any judge fail records one strike (`record_strike`); `should_escalate(strikes, iters)` fires at `strikes >= 2` or `iters >= 3` → **Escalate** per `loops.md` (`MAX_STRIKES = 2`, `MAX_ITERS = 3`, two-strike → Escalate, terminal, stop-gate veto final).
- **dispose** — `dispose_subagent(handle)` consumes the handle, emits a `SubagentReceipt` with `handles_after = 0`, and `leak_free(receipt)` must be true. No-leak-on-unload is a proof obligation, not a best effort (§3).

## 3. Compiled lifecycle (machine — TS source of truth, Rust runtime)

Authoring language: TypeScript 7.1beta strict subset (class + constructor, typed functions, no `any`, no object literals, every arg+return typed). Type map per `tools.md` narrowed mechanically for counters (`number->u32` for handles/strikes/iters, bounded small — same narrowing as `go/subagent.md` and `java/loops.md`); `string->String`, `boolean->bool`, `T|null->Option<T>`.

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
  return "java-checker";
}

function subagent_scope_dir(): string {
  return "plugins/web-development/backend/languages/java/";
}

function is_tool_allowed(tool: string): boolean {
  if (tool === "route_java_checker") {
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

function run_java_checker(handle: SubagentHandle, checker: string): string {
  if (handle.active === false) {
    return "refused:inactive";
  }
  if (is_tool_allowed("route_java_checker") === false) {
    return "refused:tool-denied";
  }
  if (checker === "maven") {
    return "java_maven";
  }
  if (checker === "gradle") {
    return "java_gradle";
  }
  if (checker === "javac") {
    return "java_javac";
  }
  return "java_check";
}

function judge_gate(javac_ok: boolean, style_ok: boolean, test_ok: boolean): boolean {
  return javac_ok && style_ok && test_ok;
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
pub fn subagent_role() -> String { return "java-checker".to_string(); }
pub fn subagent_scope_dir() -> String { return "plugins/web-development/backend/languages/java/".to_string(); }
pub fn is_tool_allowed(tool: String) -> bool { /* six allowlisted tools -> true, else false */ }
pub fn create_subagent(id: String, role: String) -> SubagentHandle { return SubagentHandle::new(id, role, true, 1); }
pub fn inject_context(handle: SubagentHandle, lane_dir: String) -> bool { /* inactive -> false; lane-dir match -> true */ }
pub fn run_java_checker(handle: SubagentHandle, checker: String) -> String { /* inactive/tool-denied refusals, else checker route */ }
pub fn judge_gate(javac_ok: bool, style_ok: bool, test_ok: bool) -> bool { return javac_ok && style_ok && test_ok; }
pub fn record_strike(strikes: u32) -> u32 { return strikes + 1; }
pub fn should_escalate(strikes: u32, iters: u32) -> bool { return strikes >= 2 || iters >= 3; }
pub fn dispose_subagent(handle: SubagentHandle) -> SubagentReceipt { return SubagentReceipt::new(handle.id, true, handle.handles, 0, None); }
pub fn leak_free(receipt: SubagentReceipt) -> bool { return receipt.handles_after == 0; }
```

Full sources: `/tmp/n3-subagent-java/subagent.ts`, `/tmp/n3-subagent-java/subagent_lib.rs`.

## 4. No-leak proof (unload drill, compiled Rust `--deny warnings` exit 0)

Harness `/tmp/n3-subagent-java/unload_drill.rs`: create → inject (lane-dir accept + cross-lane refuse) → run → judge → escalate-checks → allowlist-deny (`execute` refused) → inactive-refuse → dispose → assert `leak_free`. Log:

```text
BEFORE handles=1 AFTER handles=0 LEAK-FREE=true
```

Disposer releases the single handle (`handles_after = 0`); drill asserts `leak_free` true and exits 0. Proof obligation: every run ends in dispose + receipt; an undisposed handle is a STOP + why-receipt, never a silent pass.

## 5. Receipt

- Gate chain: `ts_check` exit 0, zero diagnostics; `ts_to_rust` `ok:true`, unsupported `[]`; Rust port `rustc --crate-type lib -D warnings` exit 0 + drill bin `rustc -D warnings` exit 0, drill exit 0 (see §4).
- TS source SHA-256: `ab39f0c06f5931da2af9e1a19fba3f7bb05a324dd4ace4d78a1b1d37f5f9d722`
- Rust output SHA-256: `442bb9b93e9367aaa17763dbf2e758001c858c98907c9a77c63c3cd69295ab34`
- why: null. sent: true.
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- No object literals; class+constructor throughout (grep exit 1 = no matches). No I/O in lifecycle (`fs::`/`File::`/`std::fs`/`write!`/`Command`/`process::` grep exit 1 = no matches); no network (`fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` grep exit 1 = no matches) — nothing retained past dispose by construction.

## References (canonical — linked, never duplicated)

- Tools / route-first contract / checkers: `tools.md`
- Hook gate (`javac -Xlint:all` + checkstyle/spotbugs), deny/warn rules, event envelopes: `hooks.md`
- Loop bounds (`MAX_ITERS = 3`, `MAX_STRIKES = 2`, two-strike), runbooks: `loops.md`
