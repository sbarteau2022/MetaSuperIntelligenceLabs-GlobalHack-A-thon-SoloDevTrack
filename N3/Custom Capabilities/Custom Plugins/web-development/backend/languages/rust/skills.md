# rust / skills (interface-only)

Date: 2026-10-31. Status: PROPOSED (Round 2 interface shrink; chain STOP — see receipt).
Scope: `plugins/web-development/backend/languages/rust/`.
Canonical source: `plugins/web-development/backend/languages/rust/skills.md` (this file). Compiled logic: `plugins/web-development/backend/languages/rust/tools.md` (bodies live there — do not duplicate them here).

## Purpose

Route-first Rust checker dispatch with bounded output and a receipt per run.

## When to use

Rust work (`.rs`, cargo/rustc/clippy). Dispatch through `route_rust_checker(checker)`; every run ends in a `RustToolReceipt`, including the why-receipt on failure.

- Trigger words: rust, cargo, rustc, clippy
- File extensions: `.rs`

## Type signature (interface only — bodies in tools.md)

```text
route_rust_checker(checker: string) -> string
  // one of: cargo_clippy | cargo_test | cargo_fmt | cargo_check

is_over_budget(byte_len: number, max_bytes: number) -> boolean

gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> boolean

make_rust_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null,
                  ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> RustToolReceipt
```

Authoring contract: TS 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions; no `any`, every arg + return typed). Type map: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`. KNOWN transpiler gap: object literals `const x: Iface = {...}` are refused — use class+constructor instead.

## Chain receipt (this revision — STOP)

- `ts_strict`: false — REPORTED: `ts_check` returned `{"ok": false, "diagnostics": "", "exitCode": null}` (compiler path: `.../Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc`) on two attempts 2026-10-31, first a 7-line interface probe, then a trivial 5-line `route_rust_checker` that is valid strict TS by inspection. INFERRED (evidence ends at the two empty responses): tool-infra failure, not a type error — zero diagnostics plus null exit code on trivially valid source. NOT attempted a third time; NOT checked further.
- `rust_port_supported`: NOT RUN (gate STOP per tool-write Step 3 — no port without ts_check=true).
- `rust_compile`: NOT RUN (gate STOP per tool-write Step 4).
- `sent`: false — nothing new sent. Compiled logic remains covered by the existing SENT receipt in `tools.md` (`rust_checker_route`, 2026-10-31, ts_strict=true, rust_compile=true; hashes on disk in that file).
- Tool/server claim audit 2026-10-31: prior revision contained prose rules only, zero named tool/server claims — REMOVED under compile-and-list rule: nothing (nothing to remove).
- Prose removal 2026-10-31: prior 11 style rules + pitfalls + versions pin removed per Round 2 interface-only spec (this REPLACE). Checking behavior is enforced by the compiled checkers dispatched via `tools.md`.

## Compilers (version queries only — no artifact compiled this revision)

- `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `tsc Version 7.1.0-beta`
