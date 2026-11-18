# tailwind / tools

Date: 2026-10-31. Status: SENT (tool-write strict chain pass). Scope: `plugins/web-development/frontend/languages/tailwind/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `tailwind` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

## Interface (human — purpose, when-to-use, signature)

- Purpose: route-first Tailwind checker dispatch with bounded output and a receipt per run. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — this language file supplies only its schemas, judges, and compilers underneath (per TOOLS-2026.md + staging TOOLS-AUDIT.md).
- When to use: Tailwind work (utility classes, `@tailwind` directives, `tailwind.config.js`, PostCSS pipeline). Dispatch goes through `route_tailwind_checker(checker)`; every run ends in a `TailwindToolReceipt`, including the why-receipt on failure.
- Trigger words: tailwind
- File extensions: `.css` (directives), `tailwind.config.js`
- Checkers (run before done; paste output): `tailwindcss`, `stylelint`, `prettier`, `postcss`
- Harness tools: `vm_exec`
- Type signature: `route_tailwind_checker(checker: string) -> string` (one of `stylelint | postcss | prettier | tailwindcss`); `is_over_budget(byte_len: number, max_bytes: number) -> boolean`; `gate_send(ts_strict, rust_port_supported, rust_compile: boolean) -> boolean`; `make_tailwind_receipt(...) -> TailwindToolReceipt`.

## Compiled tool (machine — TS source of truth, Rust runtime, compiler gate)

Authoring language: TypeScript 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions, no `any`, every arg+return typed). Type map applied: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
interface TailwindRouteRequest {
  lang_id: string;
  checker: string;
  command: string;
  max_bytes: number;
}

interface TailwindBoundedOutput {
  output: string;
  truncated: boolean;
  byte_len: number;
}

class TailwindToolReceipt {
  tool: string;
  source_hash: string;
  rust_hash: string;
  why: string | null;
  ts_strict: boolean;
  rust_port_supported: boolean;
  rust_compile: boolean;
  sent: boolean;
  constructor(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean, sent: boolean) {
    this.tool = tool;
    this.source_hash = source_hash;
    this.rust_hash = rust_hash;
    this.why = why;
    this.ts_strict = ts_strict;
    this.rust_port_supported = rust_port_supported;
    this.rust_compile = rust_compile;
    this.sent = sent;
  }
}

function route_tailwind_checker(checker: string): string {
  if (checker === "lint") {
    return "stylelint";
  }
  if (checker === "test") {
    return "postcss";
  }
  if (checker === "fmt") {
    return "prettier";
  }
  return "tailwindcss";
}

function is_over_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}

function gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}

function make_tailwind_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): TailwindToolReceipt {
  return new TailwindToolReceipt(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}

function passthrough_why(why: string | null): string | null {
  return why;
}
```

Rust runtime (ts-to-rust output; only change from transpiler emission: redundant parens around `if` conditions / `return` values removed — required by `rustc -D warnings`, semantics-preserving, documented here):

```rust
#[derive(Debug, Clone)]
pub struct TailwindRouteRequest {
  pub lang_id: String,
  pub checker: String,
  pub command: String,
  pub max_bytes: f64,
}
#[derive(Debug, Clone)]
pub struct TailwindBoundedOutput {
  pub output: String,
  pub truncated: bool,
  pub byte_len: f64,
}
#[derive(Debug, Clone)]
pub struct TailwindToolReceipt {
  pub tool: String,
  pub source_hash: String,
  pub rust_hash: String,
  pub why: Option<String>,
  pub ts_strict: bool,
  pub rust_port_supported: bool,
  pub rust_compile: bool,
  pub sent: bool,
}
impl TailwindToolReceipt {
  pub fn new(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool, sent: bool) -> Self {
    Self {
    tool: tool,
    source_hash: source_hash,
    rust_hash: rust_hash,
    why: why,
    ts_strict: ts_strict,
    rust_port_supported: rust_port_supported,
    rust_compile: rust_compile,
    sent: sent,
    }
  }
}
pub fn route_tailwind_checker(checker: String) -> String {
  if checker == "lint".to_string() {
    return "stylelint".to_string();
  }
  if checker == "test".to_string() {
    return "postcss".to_string();
  }
  if checker == "fmt".to_string() {
    return "prettier".to_string();
  }
  return "tailwindcss".to_string();
}
pub fn is_over_budget(byte_len: f64, max_bytes: f64) -> bool {
  return byte_len > max_bytes;
}
pub fn gate_send(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool {
  return ts_strict && rust_port_supported && rust_compile;
}
pub fn make_tailwind_receipt(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> TailwindToolReceipt {
  return TailwindToolReceipt::new(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}
pub fn passthrough_why(why: Option<String>) -> Option<String> {
  return why;
}
```

## Receipt (one per tool, every run)

- tool: `tailwind_checker_route`
- TS source hash (SHA-256): `26d5c722fa07881966238746f0fc7f7a68e32f53fe30695228eee0beb4490a73`
- Rust output hash (SHA-256, paren-normalized as documented): `55cba7f7321e6fe755128c037dcbf5bd002975a1e725cb58f8accb3bbd159e0a`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty; class+constructor from first draft — Tool-1 object-literal lesson applied)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- Attempt history (corrections, not edits): v1 passed first attempt, no rewrite needed. Paren normalization applied same as Tool-1 (redundant parens removed, semantics unchanged, recompile exit 0).
