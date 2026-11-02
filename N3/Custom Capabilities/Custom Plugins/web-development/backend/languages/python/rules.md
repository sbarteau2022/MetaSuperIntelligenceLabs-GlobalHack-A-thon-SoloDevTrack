# python / rules

Date: 2026-10-31 (RULE AGENT 4 of 6, Round 2). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/python/`.

Normative source: `skills.md` (`## Rules` 1–12 + Pitfalls + Versions = 21 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 21/21 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | Python 3.10+, annotate public fns, `X \| None`, `list[int]`, `collections.abc` | `**/*.py` | D | B | deny: untyped public fn blocked | E: `mypy --strict` (or `pyright`) exit 0 |
| 2 | Structured data = frozen dataclass/NamedTuple/TypedDict; Enum/Literal, no magic strings | `**/*.py` | D | B | deny: bare-dict-shaped data blocked at review gate | E: `mypy --strict` + reviewer sign-off |
| 3 | Narrowest except + context (`raise ... from e`); never bare `except:` | `**/*.py` | W | B | warn: `py-bare-except` fires at audit/stop | E: `audit.py` (`py-bare-except`), `ruff check` (E722) |
| 4 | No mutable defaults (`None` + build inside / `field(default_factory=...)`) | `**/*.py` | W | B | warn: recorded at audit | E: `ruff check` (B006) |
| 5 | `with` + `pathlib.Path`, `encoding="utf-8"` | `**/*.py` | D | B | deny: unclosed-resource pattern blocked | E: `ruff check` + reviewer sign-off |
| 6 | Pure core, I/O at edges, no import-time work, `__main__` guard | `**/*.py` | D | B | deny: import-time work blocked | E: reviewer sign-off + `pytest -q` exit 0 |
| 7 | `logging` not `print` in libraries, lazy args, never log secrets | `**/*.py` | D | B | deny at union level for secrets; warn for print | E: `audit.py` (`py-print-prod`, `no-hardcoded-password`) |
| 8 | Security: no eval/exec, no pickle-untrusted, no shell=True-interp, safe_load, param SQL, secrets+compare_digest | `**/*.py` | W* | B | deny for eval/SQL-fstring (`py-exec-eval`, `py-sql-concat`); warn-recorded for the rest | E: `audit.py` + `ruff check` (S) |
| 9 | tz-aware datetimes, Decimal money, `math.isclose` floats | `**/*.py` | W | B | warn: naive-datetime/float-== flagged | E: `ruff check` (DTZ/PLR) + `pytest -q` |
| 10 | asyncio I/O (TaskGroup, timeouts), threads=blocking, processes=CPU | `**/*.py` | D | B | deny: loop-blocking pattern blocked | E: reviewer sign-off + `pytest -q` |
| 11 | pytest: tmp_path, parametrize edge tables, raises(match=), seeded RNG, exact asserts, no sleeps | `**/*.py` (tests) | W | B | warn: weak-test pattern recorded | P: `pytest -q` pasted exit 0 + edge-table present |
| 12 | `pyproject.toml`, locked deps, virtualenv, absolute imports | repo root | W | B | warn: missing pin/lock recorded | E: `pip check` / lockfile presence |
| 13 | Pitfall: late-binding loop closures | `**/*.py` | W | B | warn: recorded at audit | P: `ruff check` (B023) pasted exit 0 |
| 14 | Pitfall: `is` for value equality | `**/*.py` | W | B | warn: recorded at audit | P: `ruff check` (E711/E712) pasted exit 0 |
| 15 | Pitfall: mutating list while iterating; `zip` truncation (`strict=True`) | `**/*.py` | W | B | warn: recorded at audit | P: `ruff check` (B909/C418) pasted exit 0 |
| 16 | Pitfall: `==` on floats | `**/*.py` | W | B | warn: recorded (see #9) | E: `ruff check` + `pytest -q` |
| 17 | Pitfall: mixing `str` and `bytes` | `**/*.py` | D | B | deny: type error surface | E: `mypy --strict` exit 0 |
| 18 | Versions: run `python3 --version`, honor project pin, add pin if missing | repo root | W | B | warn: unpinned project recorded | P: `python3 --version` + pin file pasted |
| 19 | Versions: toolchain-newer-than-pin or check-date > 12mo → re-verify skill vs release notes in same change | `skills.md` | W | B | warn: stale-pin recorded | P: dated re-verify note pasted |
| 20 | Sources (PEP 8/20/484, PyPUG, OWASP ASVS) cited, not duplicated | `skills.md` | D | B | deny: uncited normative claim blocked | E: reviewer sign-off |
| 21 | Canonical-source rule: author here, legacy flat copy is reference-only | `skills.md` | W | B | warn: authoring in legacy copy recorded | P: diff shows write-set path only |

\* #8 is W at row level because only its eval/SQL-fstring subset has a deny row in the merged union; the deny subset enforces as D via `py-exec-eval` / `py-sql-concat`. Binding wins on the subset (see contradiction table).

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows) + `rules-languages.json` (15 rows). Rows scoping `**/*.py`: `py-exec-eval` (D), `py-sql-concat` (D), `py-bare-except` (W), `py-print-prod` (W, excludes `**/hooks/**`), `no-hardcoded-password` (D), `no-todo-stub` (W), `no-select-star` (W).

| Pair | Verdict |
|---|---|
| #8 (no eval) vs `py-exec-eval` (D) | CONSISTENT — binding union row enforces the deny subset of #8. Binding wins. |
| #8 (param SQL) vs `py-sql-concat` (D) | CONSISTENT — same direction. Binding wins. |
| #3 (no bare except) vs `py-bare-except` (W) | CONSISTENT — warn both sides; stop-gate vetoes until archived. Binding wins. |
| #7 (logging not print) vs `py-print-prod` (W, hooks-excluded) | CONSISTENT — hooks/ exclusion is a documented contract exception (hook scripts print JSON to stdout), not a conflict. Binding wins. |
| #7 (never log secrets) vs `no-hardcoded-password` (D) | CONSISTENT — same direction, union is stricter (D). Binding wins. |
| #11 (exact asserts, no stubs) vs `no-todo-stub` (W) | CONSISTENT — same direction. Binding wins. |
| #8 (param SQL) vs `no-select-star` (W) | CONSISTENT — same direction. Binding wins. |
| #8 pickle/shell=True/yaml/secrets-subset vs union (no rows) | COVERAGE GAP, not contradiction — enforced by reviewer sign-off + `ruff check` (S-band), recorded here per §7.4. No rule cut. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0.

## Compiled predicates (tool-write chain, RULE AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–21), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no python):

```ts
export function sev_of(rule: number): number {
  if (rule === 3) { return 1; }
  if (rule === 4) { return 1; }
  if (rule === 8) { return 1; }
  if (rule === 9) { return 1; }
  if (rule === 11) { return 1; }
  if (rule === 12) { return 1; }
  if (rule === 13) { return 1; }
  if (rule === 14) { return 1; }
  if (rule === 15) { return 1; }
  if (rule === 16) { return 1; }
  if (rule === 18) { return 1; }
  if (rule === 19) { return 1; }
  if (rule === 21) { return 1; }
  if (rule < 1) { return 0; }
  if (rule > 21) { return 0; }
  return 2;
}
export function binding_of(rule: number): number {
  if (rule < 1) { return 0; }
  if (rule > 21) { return 0; }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 11) { return 3; }
  if (rule === 13) { return 3; }
  if (rule === 14) { return 3; }
  if (rule === 15) { return 3; }
  if (rule === 18) { return 3; }
  if (rule === 19) { return 3; }
  if (rule === 21) { return 3; }
  if (rule < 1) { return 0; }
  if (rule > 21) { return 0; }
  return 1;
}
export function kept(has_judge: number): number {
  if (has_judge === 1) { return 1; } else { return 0; }
}
export function conflict_winner(sev_a: number, bind_a: number, sev_b: number, bind_b: number): number {
  if (bind_a > bind_b) { return sev_a; } else {
    if (bind_b > bind_a) { return sev_b; } else {
      if (sev_a >= sev_b) { return sev_a; } else { return sev_b; }
    }
  }
}
export function file_gate(deny_hits: number, warn_hits: number, has_py: number): number {
  if (has_py === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-python.

Rust port (`py_rules.rs`, same `f64` convention as Hook-4 `py_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Direct hand-port of the TS above; semantics identical.

## Receipt (tool-write, one per rules file)

- rules: `python/rules` (RULE AGENT 4 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-py/py_rules.ts`): `7c0cfac7e8ed940eda82fd8309c8e711f0eae7db3dc7e2ba985e9c3cae245604`
- Rust output hash (sha256 of `/tmp/n3-rule-py/py_rules.rs` as compiled): `a8747963c53fa2879fdf83e6f44f2139fc0d946b54c642c5f2132c2cec16254e`
- `ts_strict`: true — `tsc --noEmit --strict` (system `tsc 7.1.0-beta`), exit 0, no output. NOTE (RECORD per §7, not silent): the `ts_check`/`ts_to_rust` MCP tools both fail on ANY input (empty diagnostics, null exit — backend points at a nonexistent `node_modules/typescript/bin/tsc`); `ts_to_rust` refuses on that basis. TS verified via system tsc instead; Rust is a direct hand-port, diff-reviewed against the TS.
- `rust_port_supported`: true — unsupported list: [] (no strings, no methods, no object literals; grep `{[^{}]*:` → 0 matches)
- `rust_compile`: true — `rustc --crate-type lib py_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-py/dryrun`, `-D warnings`, exit 0): `kept=21/21 deny=8 warn=13 cut=0 gate_ok=1 winner_ok=1` (all 21 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/22 → 0; binding-beats-advisory both directions; file gate deny/warn/clean)
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`write`/`Command`/`process::` (pure `f64` functions, no imports). This receipt's only write is the in-write-set REPLACE of `python/rules.md`.
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
