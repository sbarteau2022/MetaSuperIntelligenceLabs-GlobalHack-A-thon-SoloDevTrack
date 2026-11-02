# java / rules — compiled predicates

Date: 2026-10-31 (RULE AGENT 3 of 6, Batch 3 Round 2). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/java/`.

Normative source: `skills.md` (`## Rules` 1–10 + Pitfalls + Versions = 12 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 12/12 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | LTS target (25, assume >=17); records, sealed + pattern-matching `switch`, `var`, text blocks | `**/*.java` | W | B | warn: non-idiom recorded | E: `javac -Xlint:all` + reviewer sign-off |
| 2 | Immutability: `final` fields, `List.of`/`Map.of`, defensive copies at boundaries | `**/*.java` | W | B | warn: mutable exposure flagged | E: reviewer sign-off |
| 3 | `Optional` return-only; never unguarded `.get()`; `orElseThrow`; `requireNonNull` | `**/*.java` | D | B | deny: unguarded `.get()` / nullable field blocked | E: `grep -rn '\.get()' --include='*.java'` + reviewer sign-off |
| 4 | Checked=recoverable, unchecked=programmer error; never swallow; no broad `Throwable`/`Exception` catch; try-with-resources | `**/*.java` | D | B | deny: swallowed/broad-caught exception blocked | E: `grep -rn 'catch\s*(\(Throwable\|Exception\))'` + reviewer sign-off |
| 5 | `equals`+`hashCode` together; `compareTo` consistent with `equals` | `**/*.java` | W | B | warn: unpaired override flagged | E: reviewer sign-off |
| 6 | Money `BigDecimal`-from-`String`; `java.time` only, never `Date`/`Calendar` | `**/*.java` | D | B | deny: float money / `Date` use blocked | E: `grep -rn 'new BigDecimal([0-9]\|java\.util\.Date\|Calendar\.getInstance'` + reviewer sign-off |
| 7 | `java.util.concurrent`, executor shutdown, virtual threads (21+), no `synchronized` on public, safe publication | `**/*.java` | W | B | warn: unsafe publication recorded | E: reviewer sign-off |
| 8 | Constructor injection; no static mutable state; SLF4J parameterised (printing half -> union row, binding wins) | `**/*.java` | W | B | warn: flagged at audit; printing via union `java-sysout` | E: `grep` mutable static + `audit.py` `java-sysout` + reviewer |
| 9 | `PreparedStatement` always; no untrusted deserialisation; `SecureRandom`; validate+encode I/O | `**/*.java` | D | B | deny: concat SQL / `readObject` / `Random` token blocked | E: `grep -rn 'createStatement\|readObject\|new Random('` + reviewer sign-off |
| 10 | JUnit 5 + AssertJ, parameterised edge tables, fakes over Mockito, one behaviour per test | `**/*.java` (tests) | W | B | warn: non-table test recorded | P: `./mvnw -q verify` or `./gradlew check` pasted exit 0 + table present |
| 11 | Pitfalls: `==` on strings/boxed, overflow, `Arrays.asList` fixed-size, CME, mutating keys, `SimpleDateFormat` | `**/*.java` | W | B | warn: recorded at audit | P: `javac -Xlint:all` + checkstyle/spotbugs pasted exit 0 + reviewer |
| 12 | Versions: run `java -version`, honor project pin, add pin if missing | repo root | W | B | warn: unpinned project recorded | P: `java -version` + pin (`pom.xml`/`toolchain`) pasted |

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` + `rules-languages.json` (D8-verified). Rows scoping `**/*.java`: `java-sysout` (W, `System\.(out|err)\.print`); globals `no-secrets-aws-key` (D), `no-private-key` (D).

| Pair | Verdict |
|---|---|
| `java-sysout` (W, union) vs #8 (W) | CONSISTENT — same direction; union is the machine-enforced printing subset of #8. Binding wins. |
| `no-secrets-aws-key` (D, `**`) vs #9 (secrets/token half, D) | CONSISTENT — same direction, union is the machine-enforced subset of #9. Binding wins. |
| `no-private-key` (D, `**`) vs #9 | CONSISTENT — same direction. Binding wins. |
| #4 (broad-catch ban, D) vs union (no `catch (Throwable` row for `**/*.java`) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #9 (SQL-concat/deserialisation ban, D) vs union (no java SQL row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| #3 (unguarded `.get()` ban, D) vs union (no Optional row) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0. Folded-not-dropped: skills.md Pitfalls folded into #11; R1 version-pin half enforced via #12; R8 printing half defers to union `java-sysout` (W, binding wins); R10 one-behaviour-per-test enforced via reviewer sign-off.

## Compiled predicates (tool-write chain, RULE AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–12), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no java):

```ts
export function sev_of(rule: number): number {
  if (rule === 3) {
    return 2;
  }
  if (rule === 4) {
    return 2;
  }
  if (rule === 6) {
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
export function file_gate(deny_hits: number, warn_hits: number, has_java: number): number {
  if (has_java === 0) {
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

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-java.

Rust port (`java_rules.rs`, same `f64` convention as Hook-3 `java_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Paren-normalized from transpiler output (`if rule == 3f64`); semantics identical.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 3f64 {
    return 2f64;
  }
  if rule == 4f64 {
    return 2f64;
  }
  if rule == 6f64 {
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
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_java: f64) -> f64 {
  if has_java == 0f64 {
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

- rules: `java/rules` (RULE AGENT 3 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-java/java_rules.ts`): `3d488ad7c1e7bf5515a1c62f041cfaa38201a79aa24f1d4dc91ac7bb2e348dfe`
- Rust output hash (sha256 of `/tmp/n3-rule-java/java_rules.rs` as compiled, paren-normalized per go precedent): `b2416dfcedaf7106896d33fdbeb1587065f4e6ddcdab3d708c1e28c9f0fc2d95`
- `ts_strict`: true — `ts_check` MCP tool, exit 0, empty diagnostics (final 6-fn source)
- `rust_port_supported`: true — `ts_to_rust` MCP tool, unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib java_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-java/dryrun`, `-D warnings`, exit 0): `kept=12/12 deny=4 warn=8 cut=0 gate_ok=1 winner_ok=1` (all 12 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/13 → 0; binding-beats-advisory both directions; file gate deny/warn/clean/no-java)
- Correction 2026-10-31: first TS draft encoded deny={4,6,9}, omitting #3 (unguarded `.get()`); caught on review before any file write, corrected to deny={3,4,6,9}; `ts_check`/`ts_to_rust`/`rustc`/dry-run above all ran on the corrected source. Nothing shipped with the wrong set.
- Runtime judges NOT verified on this machine: `javac` present but JRE absent (`java` fails "Unable to locate a Java Runtime" — REPORTED from `hooks.md` 2026-10-31 re-check), so E-judges needing a runtime and P-judges needing `./mvnw verify` are Molar-form here (named + pasted-output-required, execution deferred to a JDK host). Static `grep` halves and all predicate logic verified by dry-run above.
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io` and no `fetch`/`socket`/`http`/`Tcp`/`net::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `java/rules.md`.
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
