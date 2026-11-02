# rust / decomposition — typed route → check → judge → receipt DAG

Date: 2026-10-31. Status: VERIFIED (declare check + prove on dummy node list, `rustc -D warnings` exit 0). Scope: `plugins/web-development/backend/languages/rust/`.

Race note (2026-10-31): `swarm.md` did not exist at read time (races with Swarm-1). This file is built from `tools.md` + `loops.md` directly; cross-check against `swarm.md` once it lands.

## 1. Breakdown nodes (typed: input → output, judge, receipt)

Source of truth for each column lives in the lane file cited — signatures and bodies are NOT duplicated here.

| # | node | input type | output type | judge (executable gate) | receipt | lane source |
|---|------|------------|-------------|-------------------------|---------|-------------|
| 1 | route | `TaskSpec` | `CheckerId` | `route_rust_checker(checker)` | `RouteReceipt` | `tools.md` (compiled tool: `route_rust_checker`, duty `route_rust_checker(checker)`) |
| 2 | check | `CheckerId` | `CheckReport` | `cargo check` / `cargo clippy` / `cargo test` / `cargo fmt --check` | `CheckReceipt` | `tools.md` (checkers) + `rules.md` (predicates `rs-*` enforced by clippy) |
| 3 | judge | `CheckReport` | `Verdict` | `judge_pass` (loops.md `Judge` state; strikes on fail) | `JudgeReceipt` | `loops.md` §1 (`Judge` → `Receipt` on pass, strike + `Run`/`Escalate` on fail) |
| 4 | receipt | `Verdict` | `RustToolReceipt` | `gate_send(ts_strict, rust_port_supported, rust_compile)` | `RustToolReceipt` | `tools.md` (`make_rust_receipt`, `gate_send`; `sent` = all three true) |

DAG order constraint (declared, then proven below): `route.output_type == check.input_type == CheckerId`, `check.output_type == judge.input_type == CheckReport`, `judge.output_type == receipt.input_type == Verdict`. No node runs until `typecheck_chain` over the declared list returns true.

Loop binding: nodes 1–4 execute inside the `loops.md` state machine (`Plan` → `Run` → `Judge` → `Receipt` → `Done`/`Escalate`; `MAX_ITERS = 3`, `MAX_STRIKES = 2`). Hooks, skills prose, and subagent contract: see `hooks.md`, `skills.md`, `subagent.md` (linked, not duplicated).

## 2. Declare check + prove (dummy node list, pasted log)

TS 7.1beta strict subset source: class `BreakdownNode` + constructor, typed functions, no `any`, no object literals (snake_case fields — required by `rustc -D warnings` `non-snake-case` lint; see Corrections).

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

- tool: `rust_decomposition_prove`
- TS source hash (SHA-256): `5a2db0da93db9e753be49b07214564fb8859747ffc9f668cdbd8529c370c6177`
- Rust lib hash (SHA-256, as compiled): `a40a1986266e8d325e3093b59b78957689960dc353ebecb293659b2ec784848f`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Dummy sources (not lane files): `/tmp/n3-decomp-test/decomp.ts`, `/tmp/n3-decomp-test/decomp_lib.rs`, `/tmp/n3-decomp-test/run_prove.rs`

## 4. Corrections (entries, not edits)

- 2026-10-31 v1: TS used camelCase fields (`inputType`, `outputType`, `receiptKind`) — `ts_check`/`ts_to_rust` passed but `rustc -D warnings` failed with 6 `non-snake-case` errors. Rewrote TS source to snake_case, re-ran the full chain from Step 2 (not a patch on the Rust side — the TS source is the source of truth).
- 2026-10-31 v1 Rust hand-normalization: by-value `declare_check`/`nodes_link` signatures consumed nodes before reuse (`E0382` use of moved value in `typecheck_chain`). Fixed with `.clone()` at call sites; transpiler signatures unchanged (transpiler output itself carries redundant parens and by-value moves — normalization is mechanical, semantics-preserving).
