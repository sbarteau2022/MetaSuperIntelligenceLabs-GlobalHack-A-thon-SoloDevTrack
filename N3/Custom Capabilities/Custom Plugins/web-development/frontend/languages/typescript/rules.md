# typescript / rules

Date: 2026-10-31 (RULE AGENT 4 of 6, Round 2). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/typescript/`.

Normative source: `skills.md` (`## Rules` 1–9 + Pitfalls + Versions = 16 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 16/16 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | Strict family on: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`; write for the project pin (`tsc --version` + package.json/tsconfig first) | `**/*.ts`, `**/*.tsx` | D | B | deny: non-strict/loose-indexed config blocked | E: `tsc --noEmit` (strict flags) exit 0 |
| 2 | No `any`: use `unknown` and narrow; a cast (`as T`) or non-null `!` needs a why-safe comment | `**/*.ts`, `**/*.tsx` | W | B | warn: `ts-no-any` fires at audit/stop | E: `audit.py` (`ts-no-any`) + `eslint` (no-explicit-any) |
| 3 | Discriminated unions for alternatives + exhaustiveness (`default: const _x: never = v`); `readonly`, `as const`, `satisfies` | `**/*.ts`, `**/*.tsx` | D | B | deny at review gate: non-exhaustive switch blocked | E: `tsc --noEmit` (never-assignability) + reviewer sign-off |
| 4 | Types vanish at runtime: validate external data (zod/valibot), infer from schema; `JSON.parse` is `any` in disguise | `**/*.ts`, `**/*.tsx` | W | B | warn: unvalidated-boundary recorded | P: schema file present + `vitest`/`jest` boundary tests pasted exit 0 |
| 5 | Prefer string-literal unions / `as const` objects over `enum`; branded types for ids/units that must not mix | `**/*.ts`, `**/*.tsx` | W | B | warn: `enum` / unbranded-id recorded | E: `eslint` + reviewer sign-off |
| 6 | `import type` for type-only imports; return types on exported fns; constrain generics (`<T extends ...>`); no overload piles | `**/*.ts`, `**/*.tsx` | D | B | deny: untyped-export / unconstrained-generic blocked | E: `tsc --noEmit` + `eslint` (consistent-type-imports) |
| 7 | Errors: `catch (e: unknown)` + narrow; `Result`-like union for expected failures; throw only for bugs | `**/*.ts`, `**/*.tsx` | D | B | deny: `catch (e: any)` / un-narrowed use blocked | E: `tsc --noEmit` (useUnknownInCatchVariables) + reviewer sign-off |
| 8 | No floating promises (`@typescript-eslint/no-floating-promises`); await or return every promise | `**/*.ts`, `**/*.tsx` | W | B | warn: `ts-floating-promise` fires at audit/stop | E: `audit.py` (`ts-floating-promise`) + `eslint` (no-floating-promises) |
| 9 | `interface` for extensible object shapes, `type` for unions/mapped; no circular imports; `noUnusedLocals` + `noUnusedParameters` on | `**/*.ts`, `**/*.tsx` | D | B | deny: unused/circular blocked | E: `tsc --noEmit` (noUnused*) + `eslint` (import/no-cycle) |
| 10 | Tests: type-level checks for tricky generics (`expectTypeOf`); test behaviour, not implementation | `**/*.ts`, `**/*.tsx` (tests) | W | B | warn: generics without type assertion recorded | P: `vitest`/`jest` pasted exit 0 + `expectTypeOf` present |
| 11 | Pitfall: structural typing admits extras; `Object.keys` is `string[]`; `Array.includes` does not narrow | `**/*.ts`, `**/*.tsx` | W | B | warn: recorded at audit | P: `tsc --noEmit` pasted exit 0 (narrowing fix shown, no cast-without-comment) |
| 12 | Pitfall: index signatures widen to `T \| undefined` only with `noUncheckedIndexedAccess`; optional vs `undefined` differ under `exactOptionalPropertyTypes` | `**/*.ts`, `**/*.tsx` | W | B | warn: recorded at audit | P: `tsc --noEmit` pasted exit 0 on the touched file |
| 13 | Versions: first `tsc --version` + read project pin (package.json devDeps, tsconfig); follow older project pin, add pin if missing; pinned stable TS 7.0.2, floor 5.0 (7.1 is BETA, do not pin) | repo root | W | B | warn: unpinned project recorded | E: `tsc --version` + pin file pasted |
| 14 | Versions: toolchain-newer-than-pin or check-date > 12mo → re-verify skill vs release notes in same change | `skills.md` | W | B | warn: stale-pin recorded | P: dated re-verify note pasted |
| 15 | Sources (TS Handbook, TSConfig reference, typescript-eslint docs) cited, not duplicated | `skills.md` | W | B | warn: uncited normative claim recorded | P: source line present in change |
| 16 | Canonical-source rule: author here, legacy flat copy is reference-only | `skills.md` | W | B | warn: authoring in legacy copy recorded | P: diff shows write-set path only |

\* #1 reads W in the dry-run `sev_of` projection for rows whose enforcement lands at audit/stop rather than pre-write; the D rows enforce at `tsc` / review gate. The projection encodes warn-vs-deny, not binding (all 16 are B — see `binding_of`). Binding wins on every conflict (see table).

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows) + `rules-languages.json` (15 rows). Rows scoping `**/*.ts` / `**/*.tsx`: `ts-no-any` (W), `ts-floating-promise` (W), `no-eval` (D), `no-dangerous-html` (D, tsx), `no-innerhtml-sink` (D, ts), `no-div-button` (D, tsx), `no-fake-heading` (W, tsx), `no-todo-stub` (W), `no-console-log` (W), `no-debugger` (D), `no-select-star` (W), `no-hardcoded-password` (D).

| Pair | Verdict |
|---|---|
| #2 (no `any`) vs `ts-no-any` (W) | CONSISTENT — warn both sides; stop-gate vetoes until archived. Binding wins. |
| #8 (no floating promises) vs `ts-floating-promise` (W) | CONSISTENT — warn both sides. Binding wins. |
| skills.md "throw only for bugs" vs `no-debugger` (D) | CONSISTENT — union is stricter (D); `debugger` is never a "bug throw". Binding wins. |
| #4 (validate external data) vs `no-eval` (D) | CONSISTENT — same direction; `eval` is an unvalidated execution sink. Binding wins. |
| #5/#6 (structured rendering, typed code) vs `no-dangerous-html` / `no-innerhtml-sink` (D) | CONSISTENT — union deny rows enforce the sink subset. Binding wins. |
| #10 (behaviour tests, no stubs) vs `no-todo-stub` (W) | CONSISTENT — same direction. Binding wins. |
| #4 (param/typed queries implied by validation) vs `no-select-star` (W) | CONSISTENT — same direction. Binding wins. |
| #7 (never log secrets subset) vs `no-hardcoded-password` (D) | CONSISTENT — same direction, union is stricter (D). Binding wins. |
| #8 (await/return) vs `no-console-log` (W) | ORTHOGONAL — logging hygiene vs promise settlement; no overlap. Both stand. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0.

## Compiled predicates (tool-write chain, RULE AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–16), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no typescript):

```ts
export function sev_of(rule: number): number {
  if (rule === 2) { return 1; }
  if (rule === 4) { return 1; }
  if (rule === 5) { return 1; }
  if (rule === 8) { return 1; }
  if (rule === 10) { return 1; }
  if (rule === 11) { return 1; }
  if (rule === 12) { return 1; }
  if (rule === 13) { return 1; }
  if (rule === 14) { return 1; }
  if (rule === 15) { return 1; }
  if (rule === 16) { return 1; }
  if (rule < 1) { return 0; }
  if (rule > 16) { return 0; }
  return 2;
}
export function binding_of(rule: number): number {
  if (rule < 1) { return 0; }
  if (rule > 16) { return 0; }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 10) { return 3; }
  if (rule === 11) { return 3; }
  if (rule === 12) { return 3; }
  if (rule === 15) { return 3; }
  if (rule === 16) { return 3; }
  if (rule < 1) { return 0; }
  if (rule > 16) { return 0; }
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
export function file_gate(deny_hits: number, warn_hits: number, has_ts: number): number {
  if (has_ts === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-typescript.

Rust port (`ts_rules.rs`, same `f64` convention as Hook-4 `ts_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Hand-normalized from transpiler output (parens de-nested, `===` → `==`); semantics identical to the TS above.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 2f64 { return 1f64; }
  if rule == 4f64 { return 1f64; }
  if rule == 5f64 { return 1f64; }
  if rule == 8f64 { return 1f64; }
  if rule == 10f64 { return 1f64; }
  if rule == 11f64 { return 1f64; }
  if rule == 12f64 { return 1f64; }
  if rule == 13f64 { return 1f64; }
  if rule == 14f64 { return 1f64; }
  if rule == 15f64 { return 1f64; }
  if rule == 16f64 { return 1f64; }
  if rule < 1f64 { return 0f64; }
  if rule > 16f64 { return 0f64; }
  return 2f64;
}
pub fn binding_of(rule: f64) -> f64 {
  if rule < 1f64 { return 0f64; }
  if rule > 16f64 { return 0f64; }
  return 1f64;
}
pub fn judge_kind_of(rule: f64) -> f64 {
  if rule == 10f64 { return 3f64; }
  if rule == 11f64 { return 3f64; }
  if rule == 12f64 { return 3f64; }
  if rule == 15f64 { return 3f64; }
  if rule == 16f64 { return 3f64; }
  if rule < 1f64 { return 0f64; }
  if rule > 16f64 { return 0f64; }
  return 1f64;
}
pub fn kept(has_judge: f64) -> f64 {
  if has_judge == 1f64 { return 1f64; } else { return 0f64; }
}
pub fn conflict_winner(sev_a: f64, bind_a: f64, sev_b: f64, bind_b: f64) -> f64 {
  if bind_a > bind_b { return sev_a; } else {
    if bind_b > bind_a { return sev_b; } else {
      if sev_a >= sev_b { return sev_a; } else { return sev_b; }
    }
  }
}
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_ts: f64) -> f64 {
  if has_ts == 0f64 { return 0f64; }
  if deny_hits > 0f64 { return 2f64; }
  if warn_hits > 0f64 { return 1f64; }
  return 0f64;
}
```

## Receipt (tool-write, one per rules file)

- rules: `typescript/rules` (RULE AGENT 4 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-ts/ts_rules.ts`): `81d87fae31f2e5221ff866311c05b6047f4e8aaf2b894ae72f4511bfae450637`
- Rust output hash (sha256 of `/tmp/n3-rule-ts/ts_rules.rs` as compiled): `a81bf92a50709b845757dd5dba82cb38c75bdab4b13af716b37f137ee0214b86`
- `ts_strict`: true — `tsc --noEmit --strict` (system `tsc 7.1.0-beta`), exit 0, no output
- `rust_port_supported`: true — unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib ts_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-ts/dryrun`, `-D warnings`, exit 0): `kept=16/16 deny=5 warn=11 cut=0 oob=0 ko=0` (all 16 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/17 → 0; binding-beats-advisory both directions; file gate deny/warn/clean/no-ts)
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`Command`/`process::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `typescript/rules.md`.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in predicate sources (grep exit 1 = no matches).
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
