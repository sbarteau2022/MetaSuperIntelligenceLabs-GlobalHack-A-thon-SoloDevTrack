# python: write Python a reviewer would merge

**Purpose.** Authoring contract for Python code: style, typing, safety, and the done-gate.
This file is interface-only. Executable logic lives in the compiled tool section of
`plugins/web-development/backend/languages/python/tools.md` (by reference, not duplicated here).

**When to use.** Any task whose topic is Python (`.py`, `.pyi`, pytest, pyproject):
route here first, apply the rules below, then close the done-gate.

**Done means these pass:** `ruff format --check . && ruff check .`, `pyright` (or `mypy --strict`), `pytest -q`.

## Type signature (skill contract)

```ts
interface PythonSkillRequest {
  topic: string;
  filePath: string;
}
interface PythonSkillPolicy {
  rule: string;
  enforced: boolean;
}
enum PythonSkillChecker {
  Ruff = "ruff",
  Mypy = "mypy",
  Pyright = "pyright",
  Pytest = "pytest"
}
class PythonSkill {
  name: string;
  routeEntry: string;
  constructor(name: string, routeEntry: string) {
    this.name = name;
    this.routeEntry = routeEntry;
  }
  appliesTo(topic: string): boolean {
    if (topic == "python") {
      return true;
    } else {
      return false;
    }
  }
  checkerFor(filePath: string): string {
    if (filePath == "") {
      return "ruff";
    } else {
      return "mypy";
    }
  }
  doneGateCount(): number {
    return 3;
  }
}
function skillName(): string {
  return "python/skills";
}
function whenToUse(topic: string): boolean {
  if (topic == "python") {
    return true;
  } else {
    return false;
  }
}
function defaultChecker(): string {
  return "ruff";
}
function makeSkill(name: string, routeEntry: string): PythonSkill {
  return new PythonSkill(name, routeEntry);
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
```

Compiled tool section: see `tools.md` ("Compiled tool (tool-write chain...)").
No SENT claims are made here: `python/tools` reports `sent=false` (port BLOCKED),
so every tool claim in this file stays PROPOSED.

## Rules
1. Python 3.10+. Annotate every public function (parameters and return). Use `X | None`, `list[int]`, and `collections.abc` types for inputs.
2. Structured data is a `@dataclass(frozen=True, slots=True)`, `NamedTuple` or `TypedDict`, not a bare dict or tuple. Use `Enum` or `Literal` instead of magic strings.
3. Catch the narrowest exception and add context (`raise Err("...") from e`). Never a bare `except:` and never `except Exception: pass`.
4. No mutable default arguments: default to `None` and build inside, or use `field(default_factory=...)`.
5. Use `with` for files, locks, connections and temp dirs. Use `pathlib.Path`. Open text files with `encoding="utf-8"`.
6. Keep the core pure and put I/O at the edges. No work at import time; guard scripts with `if __name__ == "__main__":`.
7. Use `logging`, not `print`, in libraries, with lazy args (`log.info("x %s", x)`). Never log secrets.
8. Security: no `eval`/`exec`, no `pickle` on untrusted data, no `subprocess(..., shell=True)` with interpolated input (pass a list), `yaml.safe_load`, parameterised SQL, `secrets` for tokens, `hmac.compare_digest` to compare them.
9. Time and money: timezone-aware datetimes (`datetime.now(timezone.utc)`), `Decimal` for money, `math.isclose` for floats.
10. Concurrency: `asyncio` for I/O (`TaskGroup`, timeouts, never block the loop), threads only for blocking calls, processes for CPU work.
11. Tests use pytest: `tmp_path`, `parametrize` for edge tables, `pytest.raises(match=...)`, seeded randomness, exact assertions, no sleeps.
12. Packaging: `pyproject.toml`, locked dependencies, a virtualenv, absolute imports.
## Pitfalls
Late-binding closures in loops; `is` used for value equality; editing a list while iterating; `==` on floats; mixing `str` and `bytes`; `zip` silently truncating (use `strict=True`).
## Versions
Pinned: Python 3.14 (3.14.7 installed and current; checked 2026-10-07, source: this machine). This skill assumes at least 3.10.
Rule: first run `python3 --version` and read the project's pin (.python-version, pyproject.toml requires-python). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Note 2026-10-30: `python3 --version` on this machine reports Python 3.15.2 (newer than the 3.14 pin above). Re-verification against 3.15 release notes is outstanding.
Sources: PEP 8, PEP 20, PEP 484, Python Packaging User Guide, OWASP ASVS.

## Receipt (tool-write, skill interface, SKILL AGENT 2 of 6, 2026-10-30)

- tool name: `python/skills`
- source hash (sha256 of TS source): `fe268879c771b8d7adfed27c6f21b1c85dcc3824c74f0cb905ebfbdd6cc09064`
- Rust output hash: `none (no artifact — chain STOP before port)`
- `ts_strict`: true — VERIFIED via real `tsc --noEmit --strict` exit 0 (tsc Version 7.1.0-beta, /opt/homebrew/bin/tsc). NOTE: the `ts_check` wrapper returned `ok:false` with empty diagnostics and null exit (3 calls, incl. a trivial one-function probe) — wrapper infra failure, not source diagnostics; its pinned compiler path does not exist on this machine.
- `rust_port_supported`: unknown — `ts_to_rust` refused to run because it gates on the `ts_check` wrapper (`"ts_check failed, refusing to emit"`); no unsupported-construct list was produced.
- `rust_compile`: false — not attempted per STOP rule (no artifact to compile).
- `sent`: false — gate requires all three true; NOT sent, NOT registered, NOT executed.
- why-receipt: chain STOP at port step. `ts_to_rust` hard-gates on the `ts_check` wrapper, which errors on missing infra (`node_modules/typescript/bin/tsc` absent under `~/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/`). Source itself is strict-clean under the real tsc 7.1.0-beta. Unblock by fixing the wrapper's compiler path, then re-run the chain from `ts_to_rust`.
- compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)` / `tsc Version 7.1.0-beta`

Date: 2026-10-30. Status: PROPOSED (rewritten 2026-10-30 as interface-only; chain BLOCKED — see receipt).

Canonical source: `plugins/web-development/backend/languages/python/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/python.md` (reference; do not author).
