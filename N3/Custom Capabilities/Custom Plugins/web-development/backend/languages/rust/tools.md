# rust / tools

Date: 2026-10-31. Status: SENT (tool-write strict chain pass). Scope: `plugins/web-development/backend/languages/rust/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `rust` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

## Interface (human — purpose, when-to-use, signature)

- Purpose: route-first Rust checker dispatch with bounded output and a receipt per run. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — this language file supplies only its schemas, judges, and compilers underneath (per TOOLS-2026.md + staging TOOLS-AUDIT.md).
- When to use: Rust work (`.rs`, cargo/rustc/clippy). Dispatch goes through `route_rust_checker(checker)`; every run ends in a `RustToolReceipt`, including the why-receipt on failure.
- Trigger words: rust, cargo, rustc, clippy
- File extensions: `.rs`
- Checkers (run before done; paste output): `cargo clippy`, `cargo test`, `cargo check`, `cargo fmt`, `rustfmt`, `cargo nextest`
- Harness tools: `vm_exec`
- Type signature: `route_rust_checker(checker: string) -> string` (one of `cargo_clippy | cargo_test | cargo_fmt | cargo_check`); `is_over_budget(byte_len: number, max_bytes: number) -> boolean`; `gate_send(ts_strict, rust_port_supported, rust_compile: boolean) -> boolean`; `make_rust_receipt(...) -> RustToolReceipt`.

## Compiled tool (machine — TS source of truth, Rust runtime, compiler gate)

Authoring language: TypeScript 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions, no `any`, every arg+return typed). Type map applied: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
interface RustRouteRequest {
  lang_id: string;
  checker: string;
  command: string;
  max_bytes: number;
}

interface RustBoundedOutput {
  output: string;
  truncated: boolean;
  byte_len: number;
}

class RustToolReceipt {
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

function route_rust_checker(checker: string): string {
  if (checker === "clippy") {
    return "cargo_clippy";
  }
  if (checker === "test") {
    return "cargo_test";
  }
  if (checker === "fmt") {
    return "cargo_fmt";
  }
  return "cargo_check";
}

function is_over_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}

function gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}

function make_rust_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): RustToolReceipt {
  return new RustToolReceipt(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}

function passthrough_why(why: string | null): string | null {
  return why;
}
```

Rust runtime (ts-to-rust output; only change from transpiler emission: redundant parens around `if` conditions / `return` values removed — required by `rustc -D warnings`, semantics-preserving, documented here):

```rust
#[derive(Debug, Clone)]
pub struct RustRouteRequest {
  pub lang_id: String,
  pub checker: String,
  pub command: String,
  pub max_bytes: f64,
}
#[derive(Debug, Clone)]
pub struct RustBoundedOutput {
  pub output: String,
  pub truncated: bool,
  pub byte_len: f64,
}
#[derive(Debug, Clone)]
pub struct RustToolReceipt {
  pub tool: String,
  pub source_hash: String,
  pub rust_hash: String,
  pub why: Option<String>,
  pub ts_strict: bool,
  pub rust_port_supported: bool,
  pub rust_compile: bool,
  pub sent: bool,
}
impl RustToolReceipt {
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
pub fn route_rust_checker(checker: String) -> String {
  if checker == "clippy".to_string() {
    return "cargo_clippy".to_string();
  }
  if checker == "test".to_string() {
    return "cargo_test".to_string();
  }
  if checker == "fmt".to_string() {
    return "cargo_fmt".to_string();
  }
  return "cargo_check".to_string();
}
pub fn is_over_budget(byte_len: f64, max_bytes: f64) -> bool {
  return byte_len > max_bytes;
}
pub fn gate_send(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool {
  return ts_strict && rust_port_supported && rust_compile;
}
pub fn make_rust_receipt(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> RustToolReceipt {
  return RustToolReceipt::new(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}
pub fn passthrough_why(why: Option<String>) -> Option<String> {
  return why;
}
```

## Receipt (one per tool, every run)

- tool: `rust_checker_route`
- TS source hash (SHA-256): `db3c1bb2ff652a8000a6425640599af7fd08777acfb69afbe27197ed692b718e`
- Rust output hash (SHA-256, paren-normalized as documented): `ec20772ee9386e460d06da6bae8b419d2e4fcaa163da5b798f8bed7c29503895`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty; first draft failed on `object literal` and was rewritten to class+constructor — see Failures note)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)` — note: NOT 1.98.1; `tsc Version 7.1.0-beta` (both match expected versions).
- Attempt history (corrections, not edits): v1 TS used an object literal in `make_rust_receipt` → `ts_to_rust` verdict false (`unsupported: ["object literal (no target struct known in expression)"]`), STOP per Step 3, rewritten to class+constructor → chain re-run from Step 2, all true. Raw transpiler Rust failed `rustc -D warnings` on 5 redundant-paren lints → parens removed, semantics unchanged, recompile exit 0.
