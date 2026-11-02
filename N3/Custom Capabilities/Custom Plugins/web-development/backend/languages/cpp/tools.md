# cpp / tools

Date: 2026-10-31. Status: SENT (tool-write strict chain pass). Scope: `plugins/web-development/backend/languages/cpp/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `cpp` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

## Interface (human — purpose, when-to-use, signature)

- Purpose: route-first C++ checker dispatch with bounded output and a receipt per run. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — this language file supplies only its schemas, judges, and compilers underneath (per TOOLS-2026.md + staging TOOLS-AUDIT.md).
- When to use: C++ work (`.cc`, `.cpp`, `.cxx`, `.hpp`, `.hh`, `.hxx`, g++/clang/cmake). Dispatch goes through `route_cpp_checker(checker)`; every run ends in a `CppToolReceipt`, including the why-receipt on failure.
- Trigger words: c++, cpp
- File extensions: `.cc`, `.cpp`, `.cxx`, `.hpp`, `.hh`, `.hxx`
- Checkers (run before done; paste output): `g++`, `clang++`, `cmake`, `make`, `ctest`, `clang-tidy`, `cppcheck`
- Harness tools: `vm_exec`
- Type signature: `route_cpp_checker(checker: string) -> string` (one of `cpp_gcc | cpp_clang | cpp_cmake | cpp_tidy | cpp_check`); `is_over_budget(byte_len: number, max_bytes: number) -> boolean`; `gate_send(ts_strict, rust_port_supported, rust_compile: boolean) -> boolean`; `make_cpp_receipt(...) -> CppToolReceipt`.

## Compiled tool (machine — TS source of truth, Rust runtime, compiler gate)

Authoring language: TypeScript 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions, no `any`, every arg+return typed). Type map applied: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
interface CppRouteRequest {
  lang_id: string;
  checker: string;
  command: string;
  max_bytes: number;
}

interface CppBoundedOutput {
  output: string;
  truncated: boolean;
  byte_len: number;
}

class CppToolReceipt {
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

function route_cpp_checker(checker: string): string {
  if (checker === "gcc") {
    return "cpp_gcc";
  }
  if (checker === "clang") {
    return "cpp_clang";
  }
  if (checker === "cmake") {
    return "cpp_cmake";
  }
  if (checker === "tidy") {
    return "cpp_tidy";
  }
  return "cpp_check";
}

function is_over_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}

function gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}

function make_cpp_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): CppToolReceipt {
  return new CppToolReceipt(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}

function passthrough_why(why: string | null): string | null {
  return why;
}
```

Rust runtime (ts-to-rust output; only change from transpiler emission: redundant parens around `if` conditions / `return` values removed — required by `rustc -D warnings`, semantics-preserving, documented here):

```rust
#[derive(Debug, Clone)]
pub struct CppRouteRequest {
  pub lang_id: String,
  pub checker: String,
  pub command: String,
  pub max_bytes: f64,
}
#[derive(Debug, Clone)]
pub struct CppBoundedOutput {
  pub output: String,
  pub truncated: bool,
  pub byte_len: f64,
}
#[derive(Debug, Clone)]
pub struct CppToolReceipt {
  pub tool: String,
  pub source_hash: String,
  pub rust_hash: String,
  pub why: Option<String>,
  pub ts_strict: bool,
  pub rust_port_supported: bool,
  pub rust_compile: bool,
  pub sent: bool,
}
impl CppToolReceipt {
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
pub fn route_cpp_checker(checker: String) -> String {
  if checker == "gcc".to_string() {
    return "cpp_gcc".to_string();
  }
  if checker == "clang".to_string() {
    return "cpp_clang".to_string();
  }
  if checker == "cmake".to_string() {
    return "cpp_cmake".to_string();
  }
  if checker == "tidy".to_string() {
    return "cpp_tidy".to_string();
  }
  return "cpp_check".to_string();
}
pub fn is_over_budget(byte_len: f64, max_bytes: f64) -> bool {
  return byte_len > max_bytes;
}
pub fn gate_send(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool {
  return ts_strict && rust_port_supported && rust_compile;
}
pub fn make_cpp_receipt(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> CppToolReceipt {
  return CppToolReceipt::new(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}
pub fn passthrough_why(why: Option<String>) -> Option<String> {
  return why;
}
```

## Receipt (one per tool, every run)

- tool: `cpp_checker_route`
- TS source hash (SHA-256): `559c3ee7f561fdf154728962471e3deddfd12f62ce72ebce68aaffc7520d7a3f`
- Rust output hash (SHA-256, paren-normalized as documented): `0dbfa9d92c4b84182272afb7743b6761acdce5c3c4e7f539dc562eb303877c3c`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty; class+constructor from the start — SENT precedent followed)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- Attempt history: v1 clean pass, no rewrites. Raw transpiler Rust had redundant-paren lints → parens removed, semantics unchanged, recompile exit 0 (same normalization as go/tools.md SENT precedent).
