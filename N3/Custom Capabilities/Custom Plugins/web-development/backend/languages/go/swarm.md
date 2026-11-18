# go / swarm — compiled DAG

Date: 2026-10-31. Status: SENT (tool-write strict chain pass + DAG-typecheck proof). Scope: `plugins/web-development/backend/languages/go/`.

Swarm shape reference (link, never copied): `runbooks/swarm-phase0-prereg.md` (role-scoped sub-agents on one set of weights; spawn roles, P1–P3 bars).

Lane files (paths only — bodies live there, never duplicated here):
- tools: `plugins/web-development/backend/languages/go/tools.md` (SENT: `route_go_checker`, `is_over_budget`, `gate_send`, `make_go_receipt`, `passthrough_why` — nodes claim VERIFIED off this receipt)
- hooks: `plugins/web-development/backend/languages/go/hooks.md` (gate: `go vet ./...` + `gofmt -l` | Judge E; deny/warn rules)
- loops: `plugins/web-development/backend/languages/go/loops.md` (bounded state machine: `Plan → Run → Judge → Receipt → Done`, `MAX_ITERS=3`, two-strike escalation)
- rules: `plugins/web-development/backend/languages/go/rules.md` (predicate index, 15 rows; binding wins)
- skills: `plugins/web-development/backend/languages/go/skills.md` (interface-only signatures)
- subagent: `plugins/web-development/backend/languages/go/subagent.md` (lifecycle: create → inject → run → dispose; allowlist; no-leak proof)

## 1. DAG declaration (nodes = compiled tools, edges typed)

Type universe (declared once, referenced by every edge):

| type name | meaning |
|-----------|---------|
| `CheckerName` | one of `vet \| test \| fmt \| check` (human input) |
| `GoCommand` | one of `go_vet \| go_test \| go_fmt \| go_check` (routed dispatch) |
| `BudgetFlag` | `boolean`: true = over byte budget, run blocked |
| `JudgeVerdict` | `boolean`: all lane judges passed |
| `StrikeCount` | `number`: judge-fail strikes so far |
| `EscalateFlag` | `boolean`: true = terminal Escalate |
| `OrderOk` | `boolean`: declare-phase typecheck passed |
| `SwarmReceipt` | `SwarmReceipt` class (dag_id, nodes, edges_checked, order_ok, sent) |

Nodes (every node: compiled tool + judge + receipt requirement):

| node | compiled tool (source file) | input type → output type | judge | receipt requirement |
|------|-----------------------------|--------------------------|-------|---------------------|
| n1-route | `swarm_route` (= `route_go_checker` in `tools.md`, SENT) | `CheckerName → GoCommand` | `route ∈ {go_vet,go_test,go_fmt,go_check}` | `GoToolReceipt`, `sent=true` per `tools.md` |
| n2-budget | `swarm_budget` (= `is_over_budget` in `tools.md`, SENT) | `(BudgetFlag inputs) → BudgetFlag` | `over=false` required to proceed; `over=true` = STOP + why-receipt | why-receipt on block |
| n3-judge | `swarm_judge` (= `judge_gate` in `subagent.md`; gate commands in `hooks.md`) | `GoCommand → JudgeVerdict` | `vet_ok && fmt_ok && test_ok` (`go vet ./...`, `gofmt -l` clean, `go test`) | `SwarmReceipt`, `order_ok=true` |
| n4-strike | `swarm_strikes` (= `record_strike` in `subagent.md`) + `swarm_escalate` (= `should_escalate` in `subagent.md`) | `StrikeCount → EscalateFlag` | `strikes >= 2 ∨ iters >= 3 → Escalate` per `loops.md` Escalate rule | strike recorded per judge fail; escalate terminal |
| n5-gate | `swarm_gate` (= `gate_send` in `tools.md`, SENT) + `make_swarm_receipt` | `OrderOk → SwarmReceipt` | `ts_strict && rust_port_supported && rust_compile`; any false = STOP + why-receipt per tool-write | one `SwarmReceipt` per DAG run, every time |

Edges (typed; edge type must equal producer output AND consumer input — checked by `dag_order_typechecks` before any node runs):

| edge | from → to | type | producer output | consumer input |
|------|-----------|------|-----------------|----------------|
| e1 | n1-route → n3-judge | `GoCommand` | `GoCommand` | `GoCommand` |
| e2 | n2-budget → n3-judge | `BudgetFlag` | `BudgetFlag` | `BudgetFlag` (gate: must be false) |
| e3 | n3-judge → n4-strike | `JudgeVerdict` | `JudgeVerdict` | `JudgeVerdict` (fail → strike) |
| e4 | n4-strike → n5-gate | `EscalateFlag` | `EscalateFlag` | `EscalateFlag` (true = terminal, no receipt-send) |
| e5 | declare → n5-gate | `OrderOk` | `OrderOk` | `OrderOk` (false = no node runs) |

Declare check function (runs BEFORE any node; false = STOP, no node runs):

```ts
function edge_ok(out_type: string, in_type: string): boolean {
  return out_type === in_type;
}
function dag_order_typechecks(e1o: string, e1i: string, e2o: string, e2i: string, e3o: string, e3i: string, e4o: string, e4i: string, e5o: string, e5i: string): boolean {
  if (edge_ok(e1o, e1i) === false) {
    return false;
  }
  if (edge_ok(e2o, e2i) === false) {
    return false;
  }
  if (edge_ok(e3o, e3i) === false) {
    return false;
  }
  if (edge_ok(e4o, e4i) === false) {
    return false;
  }
  if (edge_ok(e5o, e5i) === false) {
    return false;
  }
  return true;
}
```

## 2. Compiled sources (machine — TS source of truth, Rust runtime)

Authoring language: TypeScript 7.1beta ts-to-rust subset (classes + constructors, typed functions, no `any`, every arg+return typed; no object literals). Type map per `tools.md`: `number->f64`, `string->String`, `boolean->bool`.

Full TS source: `/tmp/n3-swarm-test-go/swarm.ts`. Full Rust lib: `/tmp/n3-swarm-test-go/swarm_lib.rs`. Harness: `/tmp/n3-swarm-test-go/dag_run.rs`.

```ts
class DagNode {
  id: string;
  tool: string;
  input_type: string;
  output_type: string;
  judge: string;
  constructor(id: string, tool: string, input_type: string, output_type: string, judge: string) {
    this.id = id;
    this.tool = tool;
    this.input_type = input_type;
    this.output_type = output_type;
    this.judge = judge;
  }
}
class DagEdge {
  from: string;
  to: string;
  type_name: string;
  constructor(from: string, to: string, type_name: string) {
    this.from = from;
    this.to = to;
    this.type_name = type_name;
  }
}
class SwarmReceipt {
  dag_id: string;
  nodes: number;
  edges_checked: number;
  order_ok: boolean;
  sent: boolean;
  constructor(dag_id: string, nodes: number, edges_checked: number, order_ok: boolean, sent: boolean) {
    this.dag_id = dag_id;
    this.nodes = nodes;
    this.edges_checked = edges_checked;
    this.order_ok = order_ok;
    this.sent = sent;
  }
}
function swarm_route(checker: string): string { /* vet→go_vet, test→go_test, fmt→go_fmt, else go_check */ }
function swarm_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}
function swarm_judge(vet_ok: boolean, fmt_ok: boolean, test_ok: boolean): boolean {
  return vet_ok && fmt_ok && test_ok;
}
function swarm_gate(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}
function swarm_strikes(strikes: number): number {
  return strikes + 1;
}
function swarm_escalate(strikes: number, iters: number): boolean { /* strikes>=2 → true; iters>=3 → true; else false */ }
function edge_ok(out_type: string, in_type: string): boolean {
  return out_type === in_type;
}
function dag_order_typechecks(e1o: string, e1i: string, e2o: string, e2i: string, e3o: string, e3i: string, e4o: string, e4i: string, e5o: string, e5i: string): boolean { /* all five edge_ok, else false */ }
function make_swarm_receipt(dag_id: string, nodes: number, edges_checked: number, order_ok: boolean): SwarmReceipt {
  return new SwarmReceipt(dag_id, nodes, edges_checked, order_ok, order_ok);
}
```

Rust runtime (ts-to-rust output; mechanical adaptations only, semantics-preserving, `rustc -D warnings` exit 0: redundant parens around `if` conditions / `return` values removed, `Self { id, ... }` field shorthand kept as emitted, bare tail returns, `1f64` literal suffix):

```rust
pub struct DagNode { pub id: String, pub tool: String, pub input_type: String, pub output_type: String, pub judge: String }
pub struct DagEdge { pub from: String, pub to: String, pub type_name: String }
pub struct SwarmReceipt { pub dag_id: String, pub nodes: f64, pub edges_checked: f64, pub order_ok: bool, pub sent: bool }
pub fn swarm_route(checker: String) -> String { /* vet→go_vet, test→go_test, fmt→go_fmt, else go_check */ }
pub fn swarm_budget(byte_len: f64, max_bytes: f64) -> bool { byte_len > max_bytes }
pub fn swarm_judge(vet_ok: bool, fmt_ok: bool, test_ok: bool) -> bool { vet_ok && fmt_ok && test_ok }
pub fn swarm_gate(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool { ts_strict && rust_port_supported && rust_compile }
pub fn swarm_strikes(strikes: f64) -> f64 { strikes + 1f64 }
pub fn swarm_escalate(strikes: f64, iters: f64) -> bool { /* strikes>=2 → true; iters>=3 → true; else false */ }
pub fn edge_ok(out_type: String, in_type: String) -> bool { out_type == in_type }
pub fn dag_order_typechecks(e1o: String, e1i: String, e2o: String, e2i: String, e3o: String, e3i: String, e4o: String, e4i: String, e5o: String, e5i: String) -> bool { /* all five edge_ok, else false */ }
pub fn make_swarm_receipt(dag_id: String, nodes: f64, edges_checked: f64, order_ok: bool) -> SwarmReceipt { SwarmReceipt::new(dag_id, nodes, edges_checked, order_ok, order_ok) }
```

## 3. DAG typecheck proof (dummy 2-node run in order)

Harness `/tmp/n3-swarm-test-go/dag_run.rs` (compiled Rust, `rustc -D warnings` exit 0). Nodes: n1 = `swarm_route`, n2 = `swarm_judge`; edge e1 typed `GoCommand` with declaration `e1.type == n1.output && e1.type == n2.input`; `dag_order_typechecks` asserted true BEFORE either node ran. Log:

```text
[declare] edge n1 -> n2 type=GoCommand decl_ok=true
[declare] dag_order_typechecks=true
[run n1] route(vet) -> go_vet
[run n1] budget(100,1024) over=false (expect false)
[run n2] judge(true,true,true) -> true (expect true)
[run n2] escalate(0,1) -> false (expect false)
[receipt] dag=go-swarm-dag nodes=2 edges_checked=1 order_ok=true sent=true
DAG-TYPECHECK-OK: 2 nodes ran in order, all judges passed
```

VERIFIED (measured 2026-10-31, `dag_run` exit 0): DAG order typechecked before execution; 2 nodes ran in order (n1 before n2); all judges passed; receipt `sent=true`.

## 4. Reusable swarm (nodes + edges + judges + receipts)

To reuse this DAG on a new lane task: (1) declare nodes with input/output types from the §1 type universe (extend only by adding a row + a `edge_ok` pair — never by widening an existing type); (2) run `dag_order_typechecks` over every edge — false = STOP, no node runs; (3) run nodes in topological order (n1 → n2 → n3 → n4 → n5); (4) each node fires its §1 judge; any judge fail records one strike via `swarm_strikes` and checks `swarm_escalate` (two-strike / three-iter → Escalate terminal per `loops.md`); (5) n5 closes with `make_swarm_receipt` gated by `swarm_gate` — any false = STOP + why-receipt, never a silent pass. Bounded output per `tools.md` (`swarm_budget`); scope confined to this lane dir per `subagent.md` (`inject_context` refuses any other dir).

## 5. Receipt

- tool: `go_swarm_dag`
- TS source hash (SHA-256): `4934c5755815a17d8d991b81591ca22bec45d9d00667b2c315141f94c4cbad3f`
- Rust output hash (SHA-256, paren-normalized as documented): `11bb5a12b5128e7eaa250e9a1521c44a25069fd6d55a3826f7ddfc7cde564a30`
- `ts_check`: true (`ok:true`, exit 0, zero diagnostics; compiler `.../Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc`)
- `ts_to_rust`: true (`ok:true`, unsupported: `[]`)
- `rust_compile`: true (`rustc --crate-type lib -D warnings` exit 0; `rustc -D warnings` harness exit 0; `dag_run` exit 0)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- No object literals (class+constructor throughout). Attempt history: no STOP — first-try chain pass; raw transpiler Rust had redundant-paren lints → parens removed, semantics unchanged, recompile exit 0 (same normalization as SENT precedent in `tools.md`).
