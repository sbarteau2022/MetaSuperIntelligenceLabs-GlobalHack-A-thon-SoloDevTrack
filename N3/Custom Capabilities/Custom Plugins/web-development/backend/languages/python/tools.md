# python / tools

Date: 2026-10-30. Status: PROPOSED (tool-write gate: BLOCKED — see receipt). Scope: `plugins/web-development/backend/languages/python/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `python` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

- File extensions: `.py`, `.pyi`
- Checkers (run before done; paste output): `ruff`, `mypy`, `pyright`, `pytest`, `flake8`, `pylint`, `unittest`
- Harness tools: `run_python`, `vm_exec`
- Trigger words: python, pytest, pyproject

Tool contract (per TOOLS-2026.md + staging TOOLS-AUDIT.md): route-first, bounded output, receipt per run, why-receipt on failure. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — a language supplies only its schemas, judges, and compilers underneath.

## Compiled tool (tool-write chain, TOOL AGENT 2 of 6, 2026-10-30)

TS 7.1beta subset source (fully typed, no `any`) — models the python language route-first contract: route request/response schemas, check receipt, send-gate receipt, checker enum (7 checkers), `PythonTool` class (route entry + bounded-output clamp):

```ts
interface PythonRouteRequest {
  filePath: string;
  checker: string;
  boundedOutputLimit: number;
}
interface PythonCheckReceipt {
  checker: string;
  passed: boolean;
  output: string;
}
interface PythonToolReceipt {
  toolName: string;
  sourceHash: string;
  rustHash: string;
  tsStrict: boolean;
  rustSupported: boolean;
  rustCompile: boolean;
  sent: boolean;
}
enum PythonChecker {
  Ruff = "ruff",
  Mypy = "mypy",
  Pyright = "pyright",
  Pytest = "pytest",
  Flake8 = "flake8",
  Pylint = "pylint",
  Unittest = "unittest"
}
class PythonTool {
  name: string;
  routeEntry: string;
  constructor(name: string, routeEntry: string) {
    this.name = name;
    this.routeEntry = routeEntry;
  }
  routeChecker(filePath: string): string {
    if (filePath == "") {
      return "ruff";
    } else {
      return "mypy";
    }
  }
  boundedLimit(limit: number): number {
    if (limit > 4000) {
      return 4000;
    } else {
      return limit;
    }
  }
}
function makeRouteRequest(filePath: string, checker: string, boundedOutputLimit: number): PythonRouteRequest {
  const req: PythonRouteRequest = { filePath: filePath, checker: checker, boundedOutputLimit: boundedOutputLimit };
  return req;
}
function makeCheckReceipt(checker: string, passed: boolean, output: string): PythonCheckReceipt {
  const receipt: PythonCheckReceipt = { checker: checker, passed: passed, output: output };
  return receipt;
}
function isGateOpen(tsStrict: boolean, rustSupported: boolean, rustCompile: boolean): boolean {
  if (tsStrict) {
    if (rustSupported) {
      if (rustCompile) {
        return true;
      } else {
        return false;
      }
    } else {
      return false;
    }
  } else {
    return false;
  }
}
function makeReceipt(toolName: string, sourceHash: string, rustHash: string, tsStrict: boolean, rustSupported: boolean, rustCompile: boolean, sent: boolean): PythonToolReceipt {
  const receipt: PythonToolReceipt = { toolName: toolName, sourceHash: sourceHash, rustHash: rustHash, tsStrict: tsStrict, rustSupported: rustSupported, rustCompile: rustCompile, sent: sent };
  return receipt;
}
function defaultLimit(): number {
  return 4000;
}
```

Rust artifact: NONE — port blocked (unsupported construct), no `rustc` compile attempted per STOP rule.

## Receipt (tool-write, one per tool)

- tool name: `python/tools`
- source hash (sha256 of TS source): `d34baba87994d729baf76f18595f493bb1b0341493d919af9613d8df0bba6f09`
- Rust output hash: `none (no artifact — port blocked)`
- `ts_strict`: true — diagnostics: none (exit 0, tsrs)
- `rust_port_supported`: false — unsupported list: `["object literal (no target struct known in expression)"]` (from `makeRouteRequest` / `makeCheckReceipt` / `makeReceipt` struct-literal constructors)
- `rust_compile`: false — compiler output: none (not attempted; chain STOP after unsupported port)
- `sent`: false — gate requires all three true; NOT sent, NOT registered, NOT executed
- why-receipt: `ts_to_rust` refuses struct-literal constructors (`const x: Iface = { ... }`); rewrite constructors without object literals (field-by-field function or Default pattern expressible in the subset) and re-run the chain.
- compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)` / `tsc Version 7.1.0-beta`
