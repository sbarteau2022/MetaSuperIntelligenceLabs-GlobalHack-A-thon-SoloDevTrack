# typescript / decomposition

Date: 2026-10-31 (DECOMP AGENT 4 of 6, Round 3, N3 pilot). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/typescript/`.

Normative task contract: typed nodes (input / output / judge / receipt), DAG-order
typecheck proof pasted, TS → `ts_check` → `ts_to_rust` → `rustc` chain, no object
literals, STOP on false. This file is canonical for the typescript lane (CREATE).

Reference, don't duplicate:
- State machine + bounds: `loops.md` (`Plan → Run → Judge → Receipt`, terminal
  `Receipt` only, `MAX_ITERS = 8`, `MAX_STRIKES = 2`, escalation halts before step).
- Worker lifecycle + allowlist: `subagent.md` (`created → injected → running → disposed`).
- Predicates + judges: `rules.md` (16 rows, severity/class/judge projection).
- Gates + deny rows: `hooks.md` (`ts_gate` / `pre_guard_gate` / `stop_gate`; warn rows
  `ts-no-any`, `ts-floating-promise`).
- Route-first contract + checkers: `tools.md` (`route_typescript_checker`,
  `is_over_budget`, `gate_send`, `make_typescript_receipt`; checkers `tsc`,
  `eslint`, `prettier`, `vitest`, `jest`, `biome`).

## Typed nodes (normative)

Node ids `0..3`; kinds encoded `1 = input`, `2 = output`, `3 = judge`, `4 = receipt`
(`node_kind`; id `>= 4` → `0` = untyped, rejected). A decomposition packet is well-typed
iff `typed_ok(input, output, judge, receipt) = 1`, i.e. kinds arrive in DAG order
`1 → 2 → 3 → 4`. Any other kind tuple returns `0` = STOP (re-plan, cf. loops.md strike 1).

DAG edges (`edge_ok`): allowed iff `0 <= from`, `to <= 3`, and `from < to` — strictly
forward, so no cycle is expressible. Canonical chain plus the read-only shortcut:

- `0 → 1` (input → output), `1 → 2` (output → judge), `2 → 3` (judge → receipt)
- `0 → 3` (input → receipt: receipt cites the input it closes)

`dag_ok` proves all four edges open; it returns `1` only on the full chain.

## STOP on false (tool-write chain, per-skill `tool-write` SKILL.md)

`gate_open(ts_strict, rust_supported, rust_compile)` = AND of the three booleans
(encoded `1`/`0`, no boolean/String mixing — subset keeps the port total). Any `0`
→ `0` = STOP: do not port, do not compile, do not send; return diagnostics as the
why-receipt. Same gate as `tools.md` `gate_send` and `subagent.md` judge gate.

No object literals: the subset source below constructs nothing via struct literals
(`grep -cE '\{[^{}]*:'` → `0` matches) — the `make*` constructor pattern BLOCKED
elsewhere (`ts_to_rust` refuses struct-literal constructors) is not used here. Pure
`f64` predicates only, no I/O imports (`fs::`/`File::`/`write`/`Command`/`process::`
→ `0` matches), no network (`fetch`/`socket`/`http`/`net::` → `0` matches).

## Compiled decomposition (tool-write chain, DECOMP AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — node-kind classifier, typed-packet
proof, forward-only edge predicate, DAG proof over the canonical chain, send gate:

```ts
export function node_kind(id: number): number {
  if (id === 0) { return 1; }
  if (id === 1) { return 2; }
  if (id === 2) { return 3; }
  if (id === 3) { return 4; }
  return 0;
}
export function typed_ok(input: number, output: number, judge: number, receipt: number): number {
  if (input === 1) {
    if (output === 2) {
      if (judge === 3) {
        if (receipt === 4) {
          return 1;
        } else {
          return 0;
        }
      } else {
        return 0;
      }
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
export function edge_ok(from: number, to: number): number {
  if (from < 0) { return 0; }
  if (to > 3) { return 0; }
  if (from >= to) { return 0; }
  return 1;
}
export function dag_ok(): number {
  if (edge_ok(0, 1) === 0) { return 0; }
  if (edge_ok(1, 2) === 0) { return 0; }
  if (edge_ok(2, 3) === 0) { return 0; }
  if (edge_ok(0, 3) === 0) { return 0; }
  return 1;
}
export function gate_open(ts_strict: number, rust_supported: number, rust_compile: number): number {
  if (ts_strict === 0) { return 0; }
  if (rust_supported === 0) { return 0; }
  if (rust_compile === 0) { return 0; }
  return 1;
}
```

Rust port (`ts_to_rust`, unsupported: none — hand-normalized from `(x)` parens and
camelCase params to idiomatic Rust; semantics unchanged from transpiler output):

```rust
pub fn node_kind(id: f64) -> f64 {
  if id == 0f64 {
    return 1f64;
  }
  if id == 1f64 {
    return 2f64;
  }
  if id == 2f64 {
    return 3f64;
  }
  if id == 3f64 {
    return 4f64;
  }
  return 0f64;
}
pub fn typed_ok(input: f64, output: f64, judge: f64, receipt: f64) -> f64 {
  if input == 1f64 {
    if output == 2f64 {
      if judge == 3f64 {
        if receipt == 4f64 {
          return 1f64;
        } else {
          return 0f64;
        }
      } else {
        return 0f64;
      }
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
pub fn edge_ok(from: f64, to: f64) -> f64 {
  if from < 0f64 {
    return 0f64;
  }
  if to > 3f64 {
    return 0f64;
  }
  if from >= to {
    return 0f64;
  }
  return 1f64;
}
pub fn dag_ok() -> f64 {
  if edge_ok(0f64, 1f64) == 0f64 {
    return 0f64;
  }
  if edge_ok(1f64, 2f64) == 0f64 {
    return 0f64;
  }
  if edge_ok(2f64, 3f64) == 0f64 {
    return 0f64;
  }
  if edge_ok(0f64, 3f64) == 0f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn gate_open(ts_strict: f64, rust_supported: f64, rust_compile: f64) -> f64 {
  if ts_strict == 0f64 {
    return 0f64;
  }
  if rust_supported == 0f64 {
    return 0f64;
  }
  if rust_compile == 0f64 {
    return 0f64;
  }
  return 1f64;
}
```

## Typecheck log (pasted, 2026-10-31)

- `ts_check`: `ok=true`, `exitCode=0`, zero diagnostics (strict).
- `ts_to_rust`: `ok=true`, `check.ok=true`, `unsupported=[]` (empty).
- `rustc --crate-type lib ts_decomp.rs -D warnings`: `exit=0`, no output;
  `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`.
- dry-run (`dryrun_main.rs` incl. `ts_decomp.rs`, `-D warnings`, build exit 0, run exit 0):
  - `DAG_ORDER input(0)->output(1)->judge(2)->receipt(3) dag=1 typed=1 gate=1`
  - `ALL_DECOMP_PASS`
  - asserts: `node_kind(0..3)=1..4`, `node_kind(9)=0`; `typed_ok(1,2,3,4)=1`,
    mistyped triples `=0`; `edge_ok(0,1)=1`, backward `edge_ok(1,0)=0`,
    self-loop `edge_ok(2,2)=0`; `dag_ok()=1`; `gate_open(1,1,1)=1`,
    `gate_open(1,0,1)=0`.
- no-object-literal proof: `grep -cE '\{[^{}]*:' ts_decomp.ts` → `0`.
- no-I/O proof: `grep -cE 'fs::|File::|write|Command|process::' ts_decomp.rs` → `0`.
- no-network proof: `grep -cE 'fetch|socket|http|net::' ts_decomp.rs` → `0`.

## Receipt (tool-write, one per decomposition file)

- tool name: `typescript/decomposition` (DECOMP AGENT 4 of 6, Round 3, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-decomp-ts/ts_decomp.ts`): `986d52ba147c038a955d77d8e63026abbb2767bb9506c392f1d3dc2b5aef993a`
- Rust output hash (sha256 of `/tmp/n3-decomp-ts/ts_decomp.rs` as compiled): `4d9b0fa70e1ee0baa13f09b3cca27a5e7011c4eaf866a15aaf7db41180d7156e`
- `ts_strict`: true — diagnostics: none (exit 0)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — compiler output: none (exit 0, `-D warnings`)
- `sent`: true — gate requires all three true; ALL TRUE
- no-out-of-root-writes proof: predicates are pure `f64` functions, no imports; this
  receipt's only write is the in-write-set CREATE of `typescript/decomposition.md`.
- callable schemas: none listed, none empty. This file exposes no callable tool
  schemas — its contract surface is the 5 compiled predicates above. No
  listed-but-empty schemas exist to remove.
