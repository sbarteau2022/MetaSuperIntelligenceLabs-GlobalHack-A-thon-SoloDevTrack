# java / tools

Date: 2026-10-31. Status: SENT (tool-write strict chain pass). Scope: `plugins/web-development/backend/languages/java/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `java` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

## Interface (human — purpose, when-to-use, signature)

- Purpose: route-first Java checker dispatch with bounded output and a receipt per run. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — this language file supplies only its schemas, judges, and compilers underneath (per TOOLS-2026.md + staging TOOLS-AUDIT.md).
- When to use: Java work (`.java`, maven/gradle). Dispatch goes through `route_java_checker(checker)`; every run ends in a `JavaToolReceipt`, including the why-receipt on failure.
- Trigger words: java, maven, gradle, spring boot
- File extensions: `.java`
- Checkers (run before done; paste output): `mvn`, `mvnw`, `gradle`, `gradlew`, `javac`, `checkstyle`, `spotbugs`
- Harness tools: `vm_exec`
- Type signature: `route_java_checker(checker: string) -> string` (one of `java_maven | java_gradle | java_javac | java_check`); `is_over_budget(byte_len: number, max_bytes: number) -> boolean`; `gate_send(ts_strict, rust_port_supported, rust_compile: boolean) -> boolean`; `make_java_receipt(...) -> JavaToolReceipt`.

## Compiled tool (machine — TS source of truth, Rust runtime, compiler gate)

Authoring language: TypeScript 7.1beta ts-to-rust subset (interfaces, class + constructor, typed functions, no `any`, every arg+return typed). Type map applied: `number->f64`, `string->String`, `boolean->bool`, `T|null->Option<T>`.

```ts
interface JavaRouteRequest {
  lang_id: string;
  checker: string;
  command: string;
  max_bytes: number;
}

interface JavaBoundedOutput {
  output: string;
  truncated: boolean;
  byte_len: number;
}

class JavaToolReceipt {
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

function route_java_checker(checker: string): string {
  if (checker === "maven") {
    return "java_maven";
  }
  if (checker === "gradle") {
    return "java_gradle";
  }
  if (checker === "javac") {
    return "java_javac";
  }
  return "java_check";
}

function is_over_budget(byte_len: number, max_bytes: number): boolean {
  return byte_len > max_bytes;
}

function gate_send(ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): boolean {
  return ts_strict && rust_port_supported && rust_compile;
}

function make_java_receipt(tool: string, source_hash: string, rust_hash: string, why: string | null, ts_strict: boolean, rust_port_supported: boolean, rust_compile: boolean): JavaToolReceipt {
  return new JavaToolReceipt(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}

function passthrough_why(why: string | null): string | null {
  return why;
}
```

Rust runtime (ts-to-rust output; only change from transpiler emission: redundant parens around `if` conditions / `return` values removed — required by `rustc -D warnings`, semantics-preserving, documented here):

```rust
#[derive(Debug, Clone)]
pub struct JavaRouteRequest {
  pub lang_id: String,
  pub checker: String,
  pub command: String,
  pub max_bytes: f64,
}
#[derive(Debug, Clone)]
pub struct JavaBoundedOutput {
  pub output: String,
  pub truncated: bool,
  pub byte_len: f64,
}
#[derive(Debug, Clone)]
pub struct JavaToolReceipt {
  pub tool: String,
  pub source_hash: String,
  pub rust_hash: String,
  pub why: Option<String>,
  pub ts_strict: bool,
  pub rust_port_supported: bool,
  pub rust_compile: bool,
  pub sent: bool,
}
impl JavaToolReceipt {
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
pub fn route_java_checker(checker: String) -> String {
  if checker == "maven".to_string() {
    return "java_maven".to_string();
  }
  if checker == "gradle".to_string() {
    return "java_gradle".to_string();
  }
  if checker == "javac".to_string() {
    return "java_javac".to_string();
  }
  return "java_check".to_string();
}
pub fn is_over_budget(byte_len: f64, max_bytes: f64) -> bool {
  return byte_len > max_bytes;
}
pub fn gate_send(ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> bool {
  return ts_strict && rust_port_supported && rust_compile;
}
pub fn make_java_receipt(tool: String, source_hash: String, rust_hash: String, why: Option<String>, ts_strict: bool, rust_port_supported: bool, rust_compile: bool) -> JavaToolReceipt {
  return JavaToolReceipt::new(tool, source_hash, rust_hash, why, ts_strict, rust_port_supported, rust_compile, gate_send(ts_strict, rust_port_supported, rust_compile));
}
pub fn passthrough_why(why: Option<String>) -> Option<String> {
  return why;
}
```

## Receipt (one per tool, every run)

- tool: `java_checker_route`
- TS source hash (SHA-256): `fb9fc29e8859e1776115bb62012c08caa8dd06083da0d9197b8af65fb527bf2f`
- Rust output hash (SHA-256, paren-normalized as documented): `2181ab808b276d91d584da2f204a930016b0cb08d3bda2d567a4827ee34e0e31`
- `ts_strict`: true (zero diagnostics, exit 0)
- `rust_port_supported`: true (unsupported list empty; class+constructor from the start — SENT precedent followed)
- `rust_compile`: true (`rustc --crate-type lib -D warnings`, exit 0, no warnings)
- `sent`: true (all three true)
- why: null
- Compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`; `tsc Version 7.1.0-beta`.
- Attempt history: v1 clean pass, no rewrites. Raw transpiler Rust had redundant-paren lints → parens removed, semantics unchanged, recompile exit 0 (same normalization as go/tools.md SENT precedent).
