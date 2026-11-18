# java / decomposition — typed route → check → judge → receipt DAG

Date: 2026-10-31. Status: VERIFIED (declare check + prove on dummy node list, `rustc -D warnings` exit 0). Scope: `plugins/web-development/backend/languages/java/`.

Race note (2026-10-31): `swarm.md` absent at read time (no such file in java/ listing; races with Swarm lane). This file is built from `tools.md` + `loops.md` + `rules.md` directly; cross-check against `swarm.md` once it lands.

## 1. Breakdown nodes (typed: input → output, judge, receipt)

Source of truth for each column lives in the lane file cited — signatures and bodies are NOT duplicated here.

| # | node | input type | output type | judge (executable gate) | receipt | lane source |
|---|------|------------|-------------|-------------------------|---------|-------------|
| 1 | route | `TaskSpec` | `CheckerId` | `route_java_checker(checker)` | `RouteReceipt` | `tools.md` (compiled tool: `route_java_checker`, duty `route_java_checker(checker)`; one of `java_maven \| java_gradle \| java_javac \| java_check`) |
| 2 | check | `CheckerId` | `CheckReport` | `mvn` / `mvnw` / `gradle` / `gradlew` / `javac` / `checkstyle` / `spotbugs` | `CheckReceipt` | `tools.md` (checkers) + `rules.md` (predicates #1–#12; denies #3, #4, #6, #9) |
| 3 | judge | `CheckReport` | `Verdict` | `judge_pass` (loops.md `Judge` state; strike on fail, 2-strike escalate) | `JudgeReceipt` | `loops.md` §1 (`Judge` → `Receipt` on pass, strike + `Run`/`Escalate` on fail) |
| 4 | receipt | `Verdict` | `JavaToolReceipt` | `gate_send(ts_strict, rust_port_supported, rust_compile)` | `JavaToolReceipt` | `tools.md` (`make_java_receipt`, `gate_send`; `sent` = all three true) |

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

- tool: `java_decomposition_prove`
- TS source hash (SHA-256): `b934bc8b1c90b1df27ded732b28eaa314102643373b0d056e10d016186911812`
- Rust lib hash (SHA-256, as compiled): `abb1515c5a22fc3e290a87f805fad68187c03ca3f48da6bdb10bc2c8e30653fe`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Dummy sources (not lane files): `/tmp/n3-decomp-test-java/decomp.ts`, `/tmp/n3-decomp-test-java/decomp_lib.rs`, `/tmp/n3-decomp-test-java/run_prove.rs`

## 4. Corrections (entries, not edits)

- None this round (v1 clean pass: snake_case from the start per go/decomposition.md precedent; paren-normalized + `.clone()` at by-value call sites before first compile, semantics-preserving per java/tools.md SENT precedent; no rewrite needed).
