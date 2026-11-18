# java / skills (interface-only)

Date: 2026-10-31. Status: PROPOSED (Round 2 interface shrink; chain PASS — see receipt).
Scope: `plugins/web-development/backend/languages/java/`.
Canonical source: `plugins/web-development/backend/languages/java/skills.md` (this file). Compiled logic: `plugins/web-development/backend/languages/java/tools.md` (bodies live there — do not duplicate them here).

## Purpose

Route-first Java checker dispatch with bounded output and a receipt per run.

## When to use

Java work (`.java`, maven/gradle). Dispatch through `route_java_checker(checker)`; every run ends in a `JavaToolReceipt`, including the why-receipt on failure.

- Trigger words: java, maven, gradle, spring boot
- File extensions: `.java`

## Type signature (interface only — bodies in tools.md)

```text
route_java_checker(checker: string) -> string
  // one of: java_maven | java_gradle | java_javac | java_check

is_over_budget(byte_len: number, max_bytes: number) -> boolean

gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> boolean

make_java_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null,
                ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> JavaToolReceipt
```

Authoring contract: TS 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions; no `any`, every arg + return typed). Type map: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`. KNOWN transpiler gap: object literals `const x: Iface = {...}` are refused — use class+constructor instead.

## Chain receipt (this revision — PASS)

- `ts_strict`: true — VERIFIED: `ts_check` exit 0, zero diagnostics, compiler `.../Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc`, 2026-10-31.
- `rust_port_supported`: true — VERIFIED: `ts_to_rust` ok, unsupported list empty (class+constructor from the start), 2026-10-31.
- `rust_compile`: true — VERIFIED: `rustc --crate-type lib -D warnings` exit 0, no warnings (semantics-preserving paren normalization: redundant parens around `if` conditions / `return` values removed, same as SENT precedent in `tools.md`), 2026-10-31.
- `sent`: true — all three true. Compiled bodies remain covered by the existing SENT receipt in `tools.md` (`java_checker_route`, 2026-10-31); this revision re-verified the identical TS subset against the live compilers.
- Tool/server claim audit 2026-10-31: prior revision contained prose rules only, zero named tool/server claims — REMOVED under compile-and-list rule: nothing (nothing to remove).
- Prose removal 2026-10-31: prior 10 style rules + pitfalls + versions pin removed per Round 2 interface-only spec (this REPLACE). Checking behavior is enforced by the compiled checkers dispatched via `tools.md`.

## Compilers (version queries — full chain compiled this revision, exit 0)

- `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `tsc Version 7.1.0-beta`
