# typescript / skills (interface-only)

Date: 2026-10-31. Status: PROPOSED (Round 2 interface shrink; chain-gate pass on interface probe — see receipt).
Scope: `plugins/web-development/frontend/languages/typescript/`.
Canonical source: `plugins/web-development/frontend/languages/typescript/skills.md` (this file). Compiled logic: `plugins/web-development/frontend/languages/typescript/tools.md` (bodies live there — do not duplicate them here).

## Purpose

Route-first TypeScript checker dispatch with bounded output and a receipt per run.

## When to use

TypeScript work (`.ts`, `.tsx`, `.mts`, `.cts`, tsc/eslint/prettier). Dispatch through `route_typescript_checker(checker)`; every run ends in a `TypescriptToolReceipt`, including the why-receipt on failure.

- Trigger words: typescript
- File extensions: `.ts`, `.tsx`, `.mts`, `.cts`

## Type signature (interface only — bodies in tools.md)

```text
route_typescript_checker(checker: string) -> string
  // one of: eslint | vitest | prettier | tsc

is_over_budget(byte_len: number, max_bytes: number) -> boolean

gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> boolean

make_typescript_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null,
  ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> TypescriptToolReceipt
```

Authoring contract: TS 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions; no `any`, every arg + return typed). Type map: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`. KNOWN transpiler gap: object literals `const x: Iface = {...}` are refused — use class+constructor instead (applied in tools.md `TypescriptToolReceipt`).

## Chain receipt (this revision — interface-signature probe, no new tool)

- `ts_strict`: true — VERIFIED: `ts_check` returned `{"ok": true, "diagnostics": "", "exitCode": 0}` 2026-10-31 on the three interface functions (`route_typescript_checker`, `is_over_budget`, `gate_send`).
- `rust_port_supported`: true — VERIFIED: `ts_to_rust` returned `ok: true` with `unsupported: []` on the same probe 2026-10-31.
- `rust_compile`: true — VERIFIED: `rustc --crate-type=lib -D warnings` on the transpiled probe, exit 0, no warnings, 2026-10-31.
- `sent`: false — nothing new sent from this file. Compiled logic remains covered by the existing SENT receipt in `tools.md` (`typescript_checker_route`, 2026-10-31, ts_strict=true, rust_compile=true; hashes on disk in that file).
- Tool/server claim audit 2026-10-31: checkers named in the prior revision (tsc/eslint/prettier/vitest/jest) are retained as the dispatched set in `tools.md` (Checkers + SENT receipt) — not duplicated here. Runtime-validation library mentions (zod, valibot) were prose guidance with no compiled artifact — REMOVED under interface-only rule: nothing (nothing to remove).
- Prose removal 2026-10-31: prior 9 style rules + pitfalls + versions pin removed per Round 2 interface-only spec (this REPLACE). Checking behavior is enforced by the compiled checkers dispatched via `tools.md`.
- FOLLOW-UP (out of scope, not fixed): sibling `rules.md` states "Normative rules live in `skills.md` (`## Rules` section)" — that section no longer exists after this REPLACE. The rules-file owner must repoint it (to `tools.md` checkers or the new canonical rule home).

## Compilers (version queries only — no artifact compiled this revision)

- `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `tsc Version 7.1.0-beta`
