# go / tools

Date: 2026-10-31. Status: SENT (tool-write strict chain pass). Scope: `plugins/web-development/backend/languages/go/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `go` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

## Interface (human — purpose, when-to-use, signature)

- Purpose: route-first Go checker dispatch with bounded output and a receipt per run. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — this language file supplies only its schemas, judges, and compilers underneath (per TOOLS-2026.md + staging TOOLS-AUDIT.md).
- When to use: Go work (`.go`, go mod/go vet). Dispatch goes through `route_go_checker(checker)`; every run ends in a `GoToolReceipt`, including the why-receipt on failure.
- Trigger words: golang, go mod, go vet
- File extensions: `.go`
- Checkers (run before done; paste output): `go vet`, `go test`, `gofmt`, `staticcheck`, `golangci-lint`
- Harness tools: `vm_exec`
- Type signature: `route_go_checker(checker: string) -> string` (one of `go_vet | go_test | go_fmt | go_check`); `is_over_budget(byte_len: number, max_bytes: number) -> boolean`; `gate_send(ts_strict, rust_port_supported, rust_compile: boolean) -> boolean`; `make_go_receipt(...) -> GoToolReceipt`.

## Compiled tool (machine — TS source of truth, Rust runtime, compiler gate)

Authoring language: TypeScript 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions, no `any`, every arg+return typed). Type map applied: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
interface GoRouteRequest {
  lang_id: string;
  checker: string;
  command: string;
  max_bytes: number;
}

interface GoBoundedOutput {
  output: string;
  truncated: boolean;
  byte_len: number;
}

class GoToolReceipt {
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

function route_go_checker(checker: string): string {
  if (checker === "vet") {
    return "go_vet";
  }
  if (checker === "test") {
    return "go_test";
  }
  if (checker === "fmt") {
    return "go_fmt";
  }
  return "go_check";
}

function is_over_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}

function gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}

function make_go_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): GoToolReceipt {
  return new GoToolReceipt(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}

function passthrough_why(why: string | null): string | null {
  return why;
}
```

Rust runtime (ts-to-rust output; only change from transpiler emission: redundant parens around `if` conditions / `return` values removed — required by `rustc -D warnings`, semantics-preserving, documented here):

```rust
#[derive(Debug, Clone)]
pub struct GoRouteRequest {
  pub lang_id: String,
  pub checker: String,
  pub command: String,
  pub max_bytes: f64,
}
#[derive(Debug, Clone)]
pub struct GoBoundedOutput {
  pub output: String,
  pub truncated: bool,
  pub byte_len: f64,
}
#[derive(Debug, Clone)]
pub struct GoToolReceipt {
  pub tool: String,
  pub source_hash: String,
  pub rust_hash: String,
  pub why: Option<String>,
  pub ts_strict: bool,
  pub rust_port_supported: bool,
  pub rust_compile: bool,
  pub sent: bool,
}
impl GoToolReceipt {
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
pub fn route_go_checker(checker: String) -> String {
  if checker == "vet".to_string() {
    return "go_vet".to_string();
  }
  if checker == "test".to_string() {
    return "go_test".to_string();
  }
  if checker == "fmt".to_string() {
    return "go_fmt".to_string();
  }
  return "go_check".to_string();
}
pub fn is_over_budget(byte_len: f64, max_bytes: f64) -> bool {
  return byte_len > max_bytes;
}
pub fn gate_send(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool {
  return ts_strict && rust_port_supported && rust_compile;
}
pub fn make_go_receipt(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> GoToolReceipt {
  return GoToolReceipt::new(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}
pub fn passthrough_why(why: Option<String>) -> Option<String> {
  return why;
}
```

## Receipt (one per tool, every run)

- tool: `go_checker_route`
- TS source hash (SHA-256): `fd2587f2d39a1b1410b02df80994a3f8086c4a242696dfcd993864c83340f48e`
- Rust output hash (SHA-256, paren-normalized as documented): `29ad5ecd35bbac02a79b3b043e8037332ac91ad41aa5b4caecdcd0370e006eac`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty; class+constructor from the start — SENT precedent followed)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- Attempt history: v1 clean pass, no rewrites. Raw transpiler Rust had redundant-paren lints → parens removed, semantics unchanged, recompile exit 0 (same normalization as rust/tools.md SENT precedent).
