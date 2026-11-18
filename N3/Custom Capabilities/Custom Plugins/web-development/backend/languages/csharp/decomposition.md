# csharp / decomposition — typed route → check → judge → receipt DAG

Date: 2026-10-31. Status: VERIFIED (declare check + prove on dummy node list, `rustc -D warnings` exit 0). Scope: `plugins/web-development/backend/languages/csharp/`.

Race note (2026-10-31): `swarm.md` absent at read time (no such file in csharp/ listing; races with Swarm lane). This file is built from `tools.md` + `loops.md` + `rules.md` directly; cross-check against `swarm.md` once it lands.

## 1. Breakdown nodes (typed: input → output, judge, receipt)

Source of truth for each column lives in the lane file cited — signatures and bodies are NOT duplicated here.

| # | node | input type | output type | judge (executable gate) | receipt | lane source |
|---|------|------------|-------------|-------------------------|---------|-------------|
| 1 | route | `TaskSpec` | `CheckerId` | `route_csharp_checker(checker)` | `RouteReceipt` | `tools.md` (compiled tool: `route_csharp_checker`, duty `route_csharp_checker(checker)`; one of `dotnet_build \| dotnet_test \| dotnet_format \| csharp_check`) |
| 2 | check | `CheckerId` | `CheckReport` | `dotnet build` / `dotnet test` / `dotnet format` | `CheckReceipt` | `tools.md` (checkers) + `rules.md` (predicates #1–#12; denies #2, #3, #4, #6, #8, #9) |
| 3 | judge | `CheckReport` | `Verdict` | `judge_pass` (loops.md `Judge` state; strike on fail, 2-strike escalate) | `JudgeReceipt` | `loops.md` §1 (`Judge` → `Receipt` on pass, strike + `Run`/`Escalate` on fail) |
| 4 | receipt | `Verdict` | `CsharpToolReceipt` | `gate_send(ts_strict, rust_port_supported, rust_compile)` | `CsharpToolReceipt` | `tools.md` (`make_csharp_receipt`, `gate_send`; `sent` = all three true) |

DAG order constraint (declared, then proven below): `route.output_type == check.input_type == CheckerId`, `check.output_type == judge.input_type == CheckReport`, `judge.output_type == receipt.input_type == Verdict`. No node runs until `typecheck_chain` over the declared list returns true.

Loop binding: nodes 1–4 execute inside the `loops.md` state machine (`Plan` → `Run` → `Judge` → `Receipt` → `Done`/`Escalate`; `MAX_ITERS = 3`, `MAX_STRIKES = 2`). Hooks, skills prose, and subagent contract: see `hooks.md`, `skills.md`, `subagent.md` (linked, not duplicated).

## 2. Declare check + prove (dummy node list, pasted log)

TS 7.1beta strict subset source: class `BreakdownNode` + constructor, typed functions, no `any`, no object literals (snake_case fields — required by `rustc -D warnings` `non-snake-case` lint; object-literal grep `{[^{}]*:[^{}]*}` → 0 matches).

```text
ts_check:  ok:true, exit 0, diagnostics "" (tsc Version 7.1.0-beta)
ts_to_rust: ok:true, check ok:true exit 0, unsupported: [] (empty — full port, no object literals)
rustc --crate-type lib -D warnings decomp_lib.rs: exit 0, zero warnings
rustc -D warnings run_prove.rs: exit 0, zero warnings
run_prove output:
  node: route
  node: check
  node: judge
  node: receipt
  PROVE-OK: declare check + DAG type order hold on dummy node list (route->check->judge->receipt)
  NEGATIVE-OK: mismatched edge (WRONG vs CheckerId) rejected
run exit: 0
rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)
```

Positive case (`prove_chain`) typechecks the exact table above in DAG order; negative case (route output `WRONG`) is rejected — the link check is load-bearing, not vacuous. Only changes from transpiler emission: redundant parens removed, `.clone()` at by-value call sites, `let mut` → `let` for immutable bindings — required by `rustc -D warnings`, semantics-preserving per csharp/tools.md SENT precedent.

## 3. Receipt

- tool: `csharp_decomposition_prove`
- TS source hash (SHA-256): `34554dca8ecea7329db0b902f0479d332e4cd05a6779bcd57419d7c824d80400`
- Rust lib hash (SHA-256, as compiled): `55854f08b8c5db8a39fbb5973749e352f40e5bdf0ceb00369142ee5b6a875eaf`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Dummy sources (not lane files): `/tmp/n3-decomp-test-csharp/decomp.ts`, `/tmp/n3-decomp-test-csharp/decomp_lib.rs`, `/tmp/n3-decomp-test-csharp/run_prove.rs`

## 4. Corrections (entries, not edits)

- None this round beyond the harness note: first `run_prove.rs` compile hit `-D warnings` dead-code lint on unread field `node_name` (lib itself was clean exit 0); harness revised to read each node name before proving. Lib semantics unchanged.
