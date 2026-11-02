# javascript / rules

Date: 2026-10-31 (RULE AGENT 4 of 6, Round 2). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/javascript/`.

Normative source: `skills.md` (`## Rules` 1–10 + Pitfalls + Versions = 18 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 18/18 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | `const` by default, `let` when you must, never `var`; `===`; `??` for defaults (not `\|\|` which swallows 0 and `""`) | `**/*.js`, `**/*.jsx` | W | B | warn: `var` / loose-equality recorded | E: `eslint` (no-var, eqeqeq) + `prettier --check` |
| 2 | `async`/`await` with errors handled at a boundary; no floating promises; no `forEach` with async callback (`for...of` / `Promise.all`; `allSettled` for partial success) | `**/*.js`, `**/*.jsx` | W | B | warn: floating promise recorded | E: `node --test` exit 0 (unhandled rejection crashes the run) + `eslint` |
| 3 | Every `fetch`: check `res.ok`, set a timeout with `AbortController`, parse the body defensively | `**/*.js`, `**/*.jsx` | W | B | warn: unchecked-fetch recorded | P: boundary tests pasted (`node --test` / `vitest` exit 0) |
| 4 | Do not mutate arguments (spread / `structuredClone`); prefer `map`, `filter`, `reduce`, `Array.from` over index loops | `**/*.js`, `**/*.jsx` | W | B | warn: arg-mutation recorded | P: `eslint` (no-param-reassign) output pasted + behaviour test |
| 5 | Validate every external input (zod, ajv, hand-written guards); `JSON.parse` returns untrusted data | `**/*.js`, `**/*.jsx` | W | B | warn: unvalidated-boundary recorded | P: schema/guard file present + boundary tests pasted exit 0 |
| 6 | Security: no `eval` or `new Function` | `**/*.js`, `**/*.jsx` | D | B | deny: blocked at pre-write | E: `audit.py` (`no-eval`) + `pre_tool_guard.py` |
| 7 | Security: never put untrusted text in `innerHTML` (`textContent`); no string-built SQL or shell commands | `**/*.js`, `**/*.jsx` | D | B | deny: blocked at pre-write | E: `audit.py` (`no-innerhtml-sink`) + reviewer sign-off |
| 8 | Security + servers: `crypto.randomUUID` / `crypto.getRandomValues` (never `Math.random`) for secrets; anti-prototype-pollution (`Map`, `Object.create(null)`, never merge untrusted keys); no blocking sync I/O in request paths; remove listeners / clear timers | `**/*.js`, `**/*.jsx` | D | B | deny: blocked at pre-write | E: `audit.py` + `eslint` (no-sync) + reviewer sign-off |
| 9 | Numbers: `Number.isNaN`, `BigInt` above 2^53, integers (cents) for money. Dates: UTC ISO strings, format with `Intl` | `**/*.js`, `**/*.jsx` | W | B | warn: float-money / naive-date recorded | P: behaviour tests pasted exit 0 |
| 10 | Commit the lockfile, `npm ci`, `npm audit`; `// @ts-check` or TypeScript for anything non-trivial | repo root | W | B | warn: unpinned/unaudited recorded | E: `npm audit` + lockfile present |
| 11 | Tests deterministic: fake timers, no real network, one behaviour per test, exact assertions | `**/*.test.js`, `**/*.spec.js`, `**/*.jsx` (tests) | W | B | warn: non-deterministic test recorded | P: `node --test` / `vitest` / `jest` pasted exit 0 |
| 12 | Pitfall A: `this` binding in callbacks; `parseInt` without radix; `sort` lexicographic default; `==` coercion | `**/*.js`, `**/*.jsx` | W | B | warn: recorded at audit | P: `eslint` (eqeqeq, radix) output pasted |
| 13 | Pitfall B: floating-point sums; unhandled rejections crashing Node | `**/*.js`, `**/*.jsx` | W | B | warn: recorded at audit | E: `node --test` exit 0 (rejection crashes the run) |
| 14 | Versions: first `node --version` + read project pin (.nvmrc, .node-version, package.json engines); Node 24 LTS assumed, floor 20; follow older pin, add pin if missing | repo root | W | B | warn: unpinned project recorded | E: `node --version` + pin file pasted |
| 15 | Versions: toolchain-newer-than-pin or check-date > 12mo → re-verify skill vs release notes in same change | `skills.md` | W | B | warn: stale-pin recorded | E: `node --version` + pin comparison pasted with dated note |
| 16 | Sources (MDN, Node.js docs, ECMAScript spec, OWASP ASVS) cited, not duplicated | `skills.md` | W | B | warn: uncited normative claim recorded | P: source line present in change |
| 17 | Canonical-source rule: author here, legacy flat copy is reference-only | `skills.md` | W | B | warn: authoring in legacy copy recorded | P: diff shows write-set path only |
| 18 | Done-gate: `prettier --check .`, `eslint .`, and the tests (`node --test` / `vitest` / `jest`) all pass | changed files | W | B | warn: un-gated change recorded; stop vetoes until pasted | P: all three checker outputs pasted exit 0 |

No D-reads-W projection subtlety: the three D rows (6–8) enforce at pre-write/audit; all other rows warn at audit/stop. Binding is uniform (all 18 are B — see `binding_of`).

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows) + `rules-languages.json` (15 rows, VERIFIED 2026-10-31: none scopes `*.js`/`*.jsx`). Rows scoping `**/*.js` / `**/*.jsx`: `no-eval` (D), `no-innerhtml-sink` (D, js), `no-dangerous-html` (D, jsx), `no-div-button` (D, jsx), `no-fake-heading` (W, jsx), `no-hardcoded-password` (D), `no-todo-stub` (W), `no-console-log` (W), `no-debugger` (D), `no-select-star` (W).

| Pair | Verdict |
|---|---|
| #6 (no `eval`/`new Function`) vs `no-eval` (D) | CONSISTENT — deny both sides; pre-write block. Binding wins. |
| #7 (no untrusted `innerHTML`) vs `no-innerhtml-sink` (D, js) / `no-dangerous-html` (D, jsx) | CONSISTENT — deny both sides; sink subset enforced twice. Binding wins. |
| #7 (no string-built SQL) vs `no-select-star` (W) | CONSISTENT — same direction; union warns the `SELECT *` subset. Binding wins. |
| #2 (no floating promises) vs `ts-floating-promise` (W, ts/tsx scope only) | CONSISTENT — same direction; union does not scope js, JS row extends the discipline. No conflict. |
| #8 (crypto randomness, no leaked secrets path) vs `no-hardcoded-password` (D) | CONSISTENT — same direction, union is stricter (D) on committed secrets. Binding wins. |
| skills.md silent on `debugger` vs `no-debugger` (D) | CONSISTENT — no permission granted; union deny governs. Binding wins. |
| skills.md silent on `console.log` vs `no-console-log` (W) | CONSISTENT — no permission granted; union warn governs. Binding wins. |
| #11 (deterministic tests, no stubs implied) vs `no-todo-stub` (W) | CONSISTENT — same direction. Binding wins. |
| #4 (structured transforms) vs `no-div-button` (D, jsx) / `no-fake-heading` (W, jsx) | CONSISTENT — union governs JSX-specific rendering subset. Binding wins. |
| #10 (`npm audit`) vs `no-hardcoded-password` (D) | CONSISTENT — same direction (dependency/secret hygiene). Binding wins. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0.

## Compiled predicates (tool-write chain, RULE AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–18), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no javascript):

```ts
export function sev_of(rule: number): number {
  if (rule === 6) { return 2; }
  if (rule === 7) { return 2; }
  if (rule === 8) { return 2; }
  if (rule < 1) { return 0; }
  if (rule > 18) { return 0; }
  return 1;
}
export function binding_of(rule: number): number {
  if (rule < 1) { return 0; }
  if (rule > 18) { return 0; }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 3) { return 3; }
  if (rule === 4) { return 3; }
  if (rule === 5) { return 3; }
  if (rule === 9) { return 3; }
  if (rule === 11) { return 3; }
  if (rule === 12) { return 3; }
  if (rule === 16) { return 3; }
  if (rule === 17) { return 3; }
  if (rule === 18) { return 3; }
  if (rule < 1) { return 0; }
  if (rule > 18) { return 0; }
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
export function file_gate(deny_hits: number, warn_hits: number, has_js: number): number {
  if (has_js === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-javascript.

Rust port (`js_rules.rs`, same `f64` convention as Hook-4 `js_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Hand-normalized from transpiler output (parens de-nested, `===` → `==`); semantics identical to the TS above.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 6f64 { return 2f64; }
  if rule == 7f64 { return 2f64; }
  if rule == 8f64 { return 2f64; }
  if rule < 1f64 { return 0f64; }
  if rule > 18f64 { return 0f64; }
  return 1f64;
}
pub fn binding_of(rule: f64) -> f64 {
  if rule < 1f64 { return 0f64; }
  if rule > 18f64 { return 0f64; }
  return 1f64;
}
pub fn judge_kind_of(rule: f64) -> f64 {
  if rule == 3f64 { return 3f64; }
  if rule == 4f64 { return 3f64; }
  if rule == 5f64 { return 3f64; }
  if rule == 9f64 { return 3f64; }
  if rule == 11f64 { return 3f64; }
  if rule == 12f64 { return 3f64; }
  if rule == 16f64 { return 3f64; }
  if rule == 17f64 { return 3f64; }
  if rule == 18f64 { return 3f64; }
  if rule < 1f64 { return 0f64; }
  if rule > 18f64 { return 0f64; }
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
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_js: f64) -> f64 {
  if has_js == 0f64 { return 0f64; }
  if deny_hits > 0f64 { return 2f64; }
  if warn_hits > 0f64 { return 1f64; }
  return 0f64;
}
```

## Receipt (tool-write, one per rules file)

- rules: `javascript/rules` (RULE AGENT 4 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-js/js_rules.ts`): `ef602ce5bc92fd4147f1ea936aba76b0f41178477a1dd944cd5f2e640f863487`
- Rust output hash (sha256 of `/tmp/n3-rule-js/js_rules.rs` as compiled): `a4b0a135459827a3946cf9259e96ba90f8ba6654ac979580f7ac92e90d041367`
- `ts_strict`: true — `tsc --noEmit --strict` (system `tsc 7.1.0-beta`), exit 0, no output; `ts_check` (tsrs TS 7.1) ok true, exit 0
- `rust_port_supported`: true — unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib js_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-js/dryrun`, `-D warnings`, exit 0): `kept=18/18 deny=3 warn=15 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0` (all 18 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/19 → 0; binding-beats-advisory both directions: `bw_vs_ad=1 ad_vs_bw=1`; file gate `deny=2 warn=1 clean=0 nojs=0`)
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`Command`/`process::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `javascript/rules.md`.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in predicate sources (grep exit 1 = no matches).
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
