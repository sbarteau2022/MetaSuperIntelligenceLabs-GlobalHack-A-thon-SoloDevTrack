# cpp / rules — compiled predicates

Date: 2026-10-31 (RULE AGENT 3 of 6, Batch 5 Round 2). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/cpp/`.

Normative source: `skills.md` (`## Rules` 1–11 + Pitfalls + Versions = 13 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 13/13 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | RAII: `unique_ptr` default via `make_unique`/`make_shared`; no raw `new`/`delete`; rule of zero (or five) | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | D | B | deny: raw `new`/`delete` blocked | E: `grep -rnE '=\\s*new\\b|\\bdelete\\b'` + `audit.py` pasted |
| 2 | Pass cheaply: `const T&` / `string_view` / `span`; by value to copy; return by value, never moved return-local | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | W | B | warn: flagged at audit | E: `clang++ -std=c++20 -Wall -Wextra -Wpedantic -Werror` + reviewer sign-off |
| 3 | `const`/`constexpr`; `noexcept`/`override`/`final`; `enum class`, `nullptr`, `[[nodiscard]]`, `static_cast` (no C-style casts) | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | D | B | deny: C-style cast blocked | E: `grep -rnE '\\([a-zA-Z_][a-zA-Z_0-9:<>]*\\*\\)'` + build `-Werror` pasted |
| 4 | Absence/errors: `optional`/`variant`/`expected`; exceptions exceptional, never from destructors | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | W | B | warn: flagged at audit | E: reviewer sign-off |
| 5 | Algorithms/ranges over hand loops; `array`/`vector` over C arrays; concepts over unconstrained templates | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | W | B | warn: flagged at audit | E: reviewer sign-off |
| 6 | Lifetimes: no `string_view`/`span`/ref to temporary; iterator invalidation; no stored refs across growth | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | W | B | warn: ASan/UBSan lapse recorded | E: `-fsanitize=address,undefined` build pasted + reviewer sign-off |
| 7 | Headers self-contained (IWYU); no `using namespace` in headers; ODR; avoid macros | `**/*.hpp`, `**/*.hh` | D | B | deny: `using namespace` in header blocked | E: `grep -rn 'using namespace' --include=*.hpp` + reviewer sign-off |
| 8 | Concurrency: `jthread`/`scoped_lock`; explicit memory orders only when justified; no data races (TSan) | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | D | B | deny: TSan race blocks | E: `-fsanitize=thread` run pasted + reviewer sign-off |
| 9 | Initialise with `{}`; no narrowing; no signed/unsigned mixing | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | D | B | deny: `-Werror` failure blocks | E: full-flag build pasted exit 0 |
| 10 | Tests (GoogleTest/Catch2) under sanitizers in CI; parsers fuzzed | `**/*.cpp` (tests) | W | B | warn: unsanitized test run recorded | P: sanitizer test log pasted exit 0 |
| 11 | `-Werror` hygiene: unnamed/`[[maybe_unused]]` params, no dead helpers, exact-type calls; compile after every edit | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | D | B | deny: build failure blocks | E: full-flag build pasted exit 0 |
| 12 | Pitfalls: slicing; range-for over temporary; `vector<bool>`; `map::operator[]` inserts; arg evaluation order | `**/*.cpp`, `**/*.hpp`, `**/*.cc`, `**/*.hh` | W | B | warn: recorded at audit | P: build + sanitizer run pasted exit 0 + reviewer |
| 13 | Versions: run `clang++ --version`, honor project pin (`CMAKE_CXX_STANDARD`), add pin if missing | repo root | W | B | warn: unpinned project recorded | P: `clang++ --version` + pin pasted |

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` + `rules-languages.json` (15 + 18 rules, verified 2026-10-31). Rows scoping cpp: `cpp-raw-new` only (W, `**/*.cpp, **/*.hpp, **/*.cc, **/*.hh`). Global `**` rows touching cpp files: `no-secrets-aws-key` (D), `no-private-key` (D). `c-malloc-check` scope (`**/*.c`, `**/*.h`) does not touch cpp scopes (`*.h` ≠ `*.hpp` — no overlap).

| Pair | Verdict |
|---|---|
| `cpp-raw-new` (W, scoped) vs #1 (RAII no-new ban, D) | SEVERITY GAP, same direction — union machine-enforces the warn subset at audit; local D enforced via `grep` judge + reviewer sign-off. Binding wins (union floor applies; local deny governs writes). No rule cut. |
| `no-secrets-aws-key` (D, `**`) vs skills.md (no secrets rule) | NO OVERLAP — union row applies silently to cpp files; no local counterpart to contradict. |
| `no-private-key` (D, `**`) vs skills.md (no key rule) | NO OVERLAP — same as above. |
| `c-malloc-check` (W, `*.c`/`*.h`) vs cpp scopes | NO OVERLAP — disjoint scopes. |
| #11 (`-Werror` hygiene, D) vs union (no cpp build row) | COVERAGE GAP, not contradiction — enforced by full-flag build pasted exit 0, recorded here. No rule cut. |
| #8 (TSan, D) / #6 (ASan/UBSan, W) vs union (no sanitizer row) | COVERAGE GAP, not contradiction — enforced by sanitizer runs pasted, recorded here. No rule cut. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0. Folded-not-dropped: skills.md Pitfalls folded into #12; R3 C-cast half has no union row (gap, enforced by `grep` + build); R7 `using namespace` half has no union row (gap, enforced by `grep` + reviewer).

## Compiled predicates (tool-write chain, RULE AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–13), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no cpp):

```ts
export function sev_of(rule: number): number {
  if (rule === 1) {
    return 2;
  }
  if (rule === 3) {
    return 2;
  }
  if (rule === 7) {
    return 2;
  }
  if (rule === 8) {
    return 2;
  }
  if (rule === 9) {
    return 2;
  }
  if (rule === 11) {
    return 2;
  }
  if (rule < 1) {
    return 0;
  }
  if (rule > 13) {
    return 0;
  }
  return 1;
}
export function binding_of(rule: number): number {
  if (rule < 1) {
    return 0;
  }
  if (rule > 13) {
    return 0;
  }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 10) {
    return 3;
  }
  if (rule === 12) {
    return 3;
  }
  if (rule === 13) {
    return 3;
  }
  if (rule < 1) {
    return 0;
  }
  if (rule > 13) {
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
export function file_gate(deny_hits: number, warn_hits: number, has_cpp: number): number {
  if (has_cpp === 0) {
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

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-cpp.

Rust port (`cpp_rules.rs`, same `f64` convention as csharp `csharp_rules.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Paren-normalized from transpiler output (`if rule == 2f64`); semantics identical.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 1f64 {
    return 2f64;
  }
  if rule == 3f64 {
    return 2f64;
  }
  if rule == 7f64 {
    return 2f64;
  }
  if rule == 8f64 {
    return 2f64;
  }
  if rule == 9f64 {
    return 2f64;
  }
  if rule == 11f64 {
    return 2f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 13f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn binding_of(rule: f64) -> f64 {
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 13f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn judge_kind_of(rule: f64) -> f64 {
  if rule == 10f64 {
    return 3f64;
  }
  if rule == 12f64 {
    return 3f64;
  }
  if rule == 13f64 {
    return 3f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 13f64 {
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
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_cpp: f64) -> f64 {
  if has_cpp == 0f64 {
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

- rules: `cpp/rules` (RULE AGENT 3 of 6, Batch 5 Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-cpp/cpp_rules.ts`): `3446965396e825045693f0bcec475c71a1c3a0afb92b403abdb1ea375b739d4e`
- Rust output hash (sha256 of `/tmp/n3-rule-cpp/cpp_rules.rs` as compiled, paren-normalized per csharp/java precedent): `c776102d2c2231d7febcb4aba690b685ff491a7f7dfb7c7b77ea68c657365509`
- `ts_strict`: true — `ts_check` MCP tool, exit 0, empty diagnostics (final 6-fn source)
- `rust_port_supported`: true — `ts_to_rust` MCP tool, unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib cpp_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-cpp/dryrun`, `-D warnings`, exit 0): `kept=13/13 deny=6 warn=7 cut=0 gate_ok=1 winner_ok=1` (all 13 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/14 → 0; deny set {1,3,7,8,9,11} asserted; warn set {2,4,5,6,10,12,13} asserted; binding-beats-advisory both directions; file gate deny/warn/clean/no-cpp)
- Live-judge probes this machine 2026-10-31 (`Apple clang version 21.0.0 (clang-2100.4.1.1)`): good.cpp (`make_unique`/`vector`) → `clang++ -std=c++20 -Wall -Wextra -Wpedantic -Werror` exit 0, runs rc=0; bad.cpp (`int* p = new int(42)`) → E-judge `grep -rnE '=\s*new\b|\bdelete\b'` HIT (#1-deny fires), good.cpp grep CLEAN; `clang-tidy` → `which clang-tidy` exit 1 — MISSING recorded, not silent pass (cf. `hooks.md` cpp-raw-new row; install ticket or waiver before done is claimed on real cpp work).
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io` and no `fetch`/`socket`/`http`/`Tcp`/`net::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `cpp/rules.md`.
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
