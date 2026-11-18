# csharp / rules — compiled predicates

Date: 2026-10-31 (RULE AGENT 3 of 6, Batch 4 Round 2). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/csharp/`.

Normative source: `skills.md` (`## Rules` 1–10 + Pitfalls + Versions = 12 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 12/12 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | Nullable on (`<Nullable>enable</Nullable>`); `ArgumentNullException.ThrowIfNull`, guard clauses, `required`; `record` for immutable data; pattern matching / switch expressions | `**/*.cs` | W | B | warn: nullable/annotation lapse flagged | E: `dotnet build -warnaserror` analyzers + reviewer sign-off |
| 2 | Async all the way: never `.Result`/`.Wait()`, no `async void` except handlers, `CancellationToken`, `ConfigureAwait(false)` in library, `Async` suffix | `**/*.cs` | D | B | deny: blocking call / `async void` blocked | E: `grep -rn '\.Result\|\.Wait()\|async void'` + reviewer sign-off |
| 3 | Dispose via `using`/`await using`; `HttpClient` only through `IHttpClientFactory`, never per-request | `**/*.cs` | D | B | deny: `new HttpClient(` per-request blocked | E: `grep -rn 'new HttpClient('` + reviewer sign-off |
| 4 | Specific exception types; `throw;` never `throw ex;`; no bare `catch (Exception)` without rethrow/log; no exceptions for control flow | `**/*.cs` | D | B | deny: `throw ex` / swallowing catch blocked | E: `grep -rn 'throw ex;\|catch\s*(Exception)'` + reviewer sign-off |
| 5 | LINQ for queries; single `ToList` vs deferred/multiple enumeration; `StringBuilder`/spans in hot paths | `**/*.cs` | W | B | warn: multiple-enumeration recorded | E: reviewer sign-off |
| 6 | Money is `decimal`; time is `DateTimeOffset` UTC; `StringComparison.Ordinal` unless culture matters | `**/*.cs` | D | B | deny: `double` money / `DateTime.Now` / ordinal-less compare blocked | E: `grep -rn 'double .*price\|DateTime\.Now\|Compare('` + reviewer sign-off |
| 7 | Constructor DI; options pattern; no service locator; `ILogger` templates (`"Loaded {Count} items"`), not interpolation | `**/*.cs` | W | B | warn: flagged at audit | E: reviewer sign-off |
| 8 | EF Core: no N+1 (`Include`/projections), `AsNoTracking` read-only, never concat into `FromSqlRaw`, review migrations | `**/*.cs` | D | B | deny: interpolated `FromSqlRaw` blocked | E: `grep -rn 'FromSqlRaw(\$"'` + reviewer sign-off |
| 9 | Validate every input; `RandomNumberGenerator` for secrets; slow-KDF password hashing; never `BinaryFormatter` | `**/*.cs` | D | B | deny: `BinaryFormatter` / `new Random(` token blocked | E: `grep -rn 'BinaryFormatter\|new Random('` + reviewer sign-off |
| 10 | xUnit `[Theory]` edge tables, fakes over mocks, no `Thread.Sleep` | `**/*.cs` (tests) | W | B | warn: non-table test recorded | P: `dotnet test` pasted exit 0 + table present |
| 11 | Pitfalls: closure over loop var (old C#), mutating struct copy, deferred LINQ re-run, `lock` on `this`/public, culture-compare surprise | `**/*.cs` | W | B | warn: recorded at audit | P: `dotnet format --verify-no-changes` + `dotnet build -warnaserror` pasted exit 0 + reviewer |
| 12 | Versions: run `dotnet --version`, honor project pin (`global.json`/`TargetFramework`), add pin if missing | repo root | W | B | warn: unpinned project recorded | P: `dotnet --version` + pin (`global.json`/`TargetFramework`) pasted |

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` + `rules-languages.json` (D8-verified). Rows scoping `**/*.cs`: none (verified 2026-10-31: 15 language rules, zero match `csharp`/`c#`/`*.cs` — cf. `hooks.md` Hook-3 finding). Global `**` rows touching `.cs`: `no-secrets-aws-key` (D), `no-private-key` (D).

| Pair | Verdict |
|---|---|
| `no-secrets-aws-key` (D, `**`) vs #9 (secrets/KDF half, D) | CONSISTENT — same direction, union is the machine-enforced subset of #9. Binding wins. |
| `no-private-key` (D, `**`) vs #9 | CONSISTENT — same direction. Binding wins. |
| #2 (blocking-call ban, D) vs union (no cs async row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #3 (`new HttpClient(` ban, D) vs union (no cs row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #4 (`throw ex` / broad-catch ban, D) vs union (no cs row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #6 (float-money / `DateTime.Now` ban, D) vs union (no cs row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #8 (interpolated `FromSqlRaw` ban, D) vs union (no cs SQL row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #9 (`BinaryFormatter` / `Random` token ban, D) vs union (no cs serialisation row beyond `**` key rows) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0. Folded-not-dropped: skills.md Pitfalls folded into #11; R7 `ILogger`-interpolation half has no union `cs` row (gap, enforced by reviewer); R10 one-behaviour-per-test enforced via reviewer sign-off.

## Compiled predicates (tool-write chain, RULE AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–12), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no csharp):

```ts
export function sev_of(rule: number): number {
  if (rule === 2) {
    return 2;
  }
  if (rule === 3) {
    return 2;
  }
  if (rule === 4) {
    return 2;
  }
  if (rule === 6) {
    return 2;
  }
  if (rule === 8) {
    return 2;
  }
  if (rule === 9) {
    return 2;
  }
  if (rule < 1) {
    return 0;
  }
  if (rule > 12) {
    return 0;
  }
  return 1;
}
export function binding_of(rule: number): number {
  if (rule < 1) {
    return 0;
  }
  if (rule > 12) {
    return 0;
  }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 10) {
    return 3;
  }
  if (rule === 11) {
    return 3;
  }
  if (rule === 12) {
    return 3;
  }
  if (rule < 1) {
    return 0;
  }
  if (rule > 12) {
    return 0;
  }
  return 1;
}
export function kept(has_judge: number): number {
  if (has_judge === 1) {
    return 1;
  } else {
    return 0;
  }
}
export function conflict_winner(sev_a: number, bind_a: number, sev_b: number, bind_b: number): number {
  if (bind_a > bind_b) {
    return sev_a;
  } else {
    if (bind_b > bind_a) {
      return sev_b;
    } else {
      if (sev_a >= sev_b) {
        return sev_a;
      } else {
        return sev_b;
      }
    }
  }
}
export function file_gate(deny_hits: number, warn_hits: number, has_cs: number): number {
  if (has_cs === 0) {
    return 0;
  }
  if (deny_hits > 0) {
    return 2;
  }
  if (warn_hits > 0) {
    return 1;
  }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-csharp.

Rust port (`csharp_rules.rs`, same `f64` convention as java `java_rules.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Paren-normalized from transpiler output (`if rule == 2f64`); semantics identical.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 2f64 {
    return 2f64;
  }
  if rule == 3f64 {
    return 2f64;
  }
  if rule == 4f64 {
    return 2f64;
  }
  if rule == 6f64 {
    return 2f64;
  }
  if rule == 8f64 {
    return 2f64;
  }
  if rule == 9f64 {
    return 2f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 12f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn binding_of(rule: f64) -> f64 {
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 12f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn judge_kind_of(rule: f64) -> f64 {
  if rule == 10f64 {
    return 3f64;
  }
  if rule == 11f64 {
    return 3f64;
  }
  if rule == 12f64 {
    return 3f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 12f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn kept(has_judge: f64) -> f64 {
  if has_judge == 1f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn conflict_winner(sev_a: f64, bind_a: f64, sev_b: f64, bind_b: f64) -> f64 {
  if bind_a > bind_b {
    return sev_a;
  } else {
    if bind_b > bind_a {
      return sev_b;
    } else {
      if sev_a >= sev_b {
        return sev_a;
      } else {
        return sev_b;
      }
    }
  }
}
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_cs: f64) -> f64 {
  if has_cs == 0f64 {
    return 0f64;
  }
  if deny_hits > 0f64 {
    return 2f64;
  }
  if warn_hits > 0f64 {
    return 1f64;
  }
  return 0f64;
}
```


## Receipt (tool-write, one per rules file)

- rules: `csharp/rules` (RULE AGENT 3 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-csharp/csharp_rules.ts`): `23c3605321eeca2260ec0aaeb819795dcdba58ad7034974073c003f7f907d245`
- Rust output hash (sha256 of `/tmp/n3-rule-csharp/csharp_rules.rs` as compiled, paren-normalized per java precedent): `742856a9d5ac7fb5024c955c0e48fa319e82b1af5910a9889325ee2b4`
- `ts_strict`: true — `ts_check` MCP tool, exit 0, empty diagnostics (final 6-fn source)
- `rust_port_supported`: true — `ts_to_rust` MCP tool, unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib csharp_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-csharp/dryrun`, `-D warnings`, exit 0): `kept=12/12 deny=6 warn=6 cut=0 gate_ok=1 winner_ok=1` (all 12 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/13 → 0; deny set {2,3,4,6,8,9} asserted; binding-beats-advisory both directions; file gate deny/warn/clean/no-cs)
- Runtime judges NOT verified on this machine: `dotnet` absent (`dotnet: command not found`, exit 127 — cf. `hooks.md` 2026-10-31 re-check), so E-judges needing `dotnet build/test/format` and P-judges needing `dotnet test` pasted output are Molar-form here (named + pasted-output-required, execution deferred to a .NET host). Static `grep` halves and all predicate logic verified by dry-run above.
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io` and no `fetch`/`socket`/`http`/`Tcp`/`net::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `csharp/rules.md`.
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
