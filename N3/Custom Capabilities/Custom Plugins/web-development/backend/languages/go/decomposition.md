# go / decomposition — typed route → check → judge → receipt DAG

Date: 2026-10-31. Status: VERIFIED (declare check + prove on dummy node list, `rustc -D warnings` exit 0). Scope: `plugins/web-development/backend/languages/go/`.

Race note (2026-10-31): `swarm.md` absent at read time (absent from go/ listing; races with Swarm-1). This file is built from `tools.md` + `loops.md` + `rules.md` directly; cross-check against `swarm.md` once it lands.

## 1. Breakdown nodes (typed: input → output, judge, receipt)

Source of truth for each column lives in the lane file cited — signatures and bodies are NOT duplicated here.

| # | node | input type | output type | judge (executable gate) | receipt | lane source |
|---|------|------------|-------------|-------------------------|---------|-------------|
| 1 | route | `TaskSpec` | `CheckerId` | `route_go_checker(checker)` | `RouteReceipt` | `tools.md` (compiled tool: `route_go_checker`, duty `route_go_checker(checker)`) |
| 2 | check | `CheckerId` | `CheckReport` | `go vet ./...` / `go test` / `gofmt -l` / `go test -race` | `CheckReceipt` | `tools.md` (checkers) + `rules.md` (predicates #1–#15; race row #12) |
| 3 | judge | `CheckReport` | `Verdict` | `judge_pass` (loops.md `Judge` state; strike on fail, 2-strike escalate) | `JudgeReceipt` | `loops.md` §1 (`Judge` → `Receipt` on pass, strike + `Run`/`Escalate` on fail) |
| 4 | receipt | `Verdict` | `GoToolReceipt` | `gate_send(ts_strict, rust_port_supported, rust_compile)` | `GoToolReceipt` | `tools.md` (`make_go_receipt`, `gate_send`; `sent` = all three true) |

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
  PROVE-OK: declare check + DAG type order hold on dummy node list (route->check->judge->receipt)
  NEGATIVE-OK: mismatched edge (WRONG vs CheckerId) rejected
run exit: 0
rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)
```

Positive case (`prove_chain`) typechecks the exact table above in DAG order; negative case (route output `WRONG`) is rejected — the link check is load-bearing, not vacuous.

## 3. Receipt

- tool: `go_decomposition_prove`
- TS source hash (SHA-256): `aab0af8d17f5ab72510aec1b6e5d4679df5a1de3091bc44696bee1e37ead25c8`
- Rust lib hash (SHA-256, as compiled): `11b5ca97edcea0e3cf67d7d6805c814c00b6daf355a55c5de88f5e6b92d6f20f`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Dummy sources (not lane files): `/tmp/n3-decomp-test-go/decomp.ts`, `/tmp/n3-decomp-test-go/decomp_lib.rs`, `/tmp/n3-decomp-test-go/run_prove.rs`

## 4. Corrections (entries, not edits)

- None this round (v1 clean pass: snake_case from the start per rust/decomposition.md precedent; paren-normalized + `.clone()` at by-value call sites before first compile, semantics-preserving; no rewrite needed).
