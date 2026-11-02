# tailwind / skills (interface-only)

Date: 2026-10-31. Status: PROPOSED (Round 2 interface shrink; chain-gate pass on interface probe — see receipt).
Scope: `plugins/web-development/frontend/languages/tailwind/`.
Canonical source: `plugins/web-development/frontend/languages/tailwind/skills.md` (this file). Compiled logic: `plugins/web-development/frontend/languages/tailwind/tools.md` (bodies live there — do not duplicate them here).

## Purpose

Route-first Tailwind checker dispatch with bounded output and a receipt per run.

## When to use

Tailwind work (`.css` directives, `tailwind.config.js`, PostCSS pipeline). Dispatch through `route_tailwind_checker(checker)`; every run ends in a `TailwindToolReceipt`, including the why-receipt on failure.

- Trigger words: tailwind
- File extensions: `.css` (directives), `tailwind.config.js`

## Type signature (interface only — bodies in tools.md)

```text
route_tailwind_checker(checker: string) -> string
  // one of: stylelint | postcss | prettier | tailwindcss

is_over_budget(byte_len: number, max_bytes: number) -> boolean

gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> boolean

make_tailwind_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null,
  ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean) -> TailwindToolReceipt
```

Authoring contract: TS 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions; no `any`, every arg + return typed). Type map: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`. KNOWN transpiler gap: object literals `const x: Iface = {...}` are refused — use class+constructor instead (applied in tools.md `TailwindToolReceipt`).

## Chain receipt (this revision — interface-signature probe, no new tool)

- `ts_strict`: true — VERIFIED: `ts_check` returned `{"ok": true, "diagnostics": "", "exitCode": 0}` 2026-10-31 on the three interface functions (`route_tailwind_checker`, `is_over_budget`, `gate_send`).
- `rust_port_supported`: true — VERIFIED: `ts_to_rust` returned `ok: true` with `unsupported: []` on the same probe 2026-10-31.
- `rust_compile`: true — VERIFIED: `rustc --crate-type=lib -D warnings` on the transpiled probe, exit 0, no warnings, 2026-10-31.
- `sent`: false — nothing new sent from this file. Compiled logic remains covered by the existing SENT receipt in `tools.md` (`tailwind_checker_route`, 2026-10-31, ts_strict=true, rust_compile=true; hashes on disk in that file).
- Tool/server claim audit 2026-10-31: checkers named in the prior revision (vite build / tailwindcss CLI, prettier, eslint, visual review at 360px + 1280px) versus the dispatched set in `tools.md` (stylelint, postcss, prettier, tailwindcss) — retained as the dispatched set in `tools.md`, not duplicated here. The eslint / vite-build / visual-review mentions were prose guidance with no compiled artifact — REMOVED under interface-only rule: nothing (nothing to remove). stylelint/postcss dispatch-only additions live in `tools.md`.
- Prose removal 2026-10-31: prior 10 utility-first/token rules + pitfalls + versions pin removed per Round 2 interface-only spec (this REPLACE). Checking behavior is enforced by the compiled checkers dispatched via `tools.md`.
- FOLLOW-UP (out of scope, not fixed): sibling `rules.md` states "Normative rules live in `skills.md` (`## Rules` section)" — that section no longer exists after this REPLACE. The rules-file owner must repoint it (to `tools.md` checkers or the new canonical rule home).

## Compilers (version queries only — no artifact compiled this revision)

- `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `tsc Version 7.1.0-beta`
