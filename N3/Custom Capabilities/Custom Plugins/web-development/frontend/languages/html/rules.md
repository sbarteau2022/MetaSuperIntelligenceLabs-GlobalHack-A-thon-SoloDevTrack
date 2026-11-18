# html / rules

Date: 2026-10-31 (RULE AGENT 4 of 6, Round 2). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/html/`.

Normative source: `skills.md` (`## Rules` 1–9 + Pitfalls + Versions = 18 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 18/18 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | One `main`, one `h1` per page; headings descend without skipping levels; sections get `aria-labelledby` where the heading is not implicit | `**/*.html` (+ tsx/jsx/vue where authored) | W | B | warn: heading-order recorded | E: `html-validate` |
| 2 | Landmarks explicit: `header`, `nav`, `main`, `footer`, plus named `aside`/`section` when more than one of a kind exists | `**/*.html` (+ tsx/jsx/vue where authored) | W | B | warn: missing-landmark recorded | E: `html-validate` |
| 3 | First focusable element is a skip link (`<a href="#main">`) visible on focus; `#main` has `tabindex="-1"` | `**/*.html` | W | B | warn: missing-skip-link recorded | E: `html-validate` (presence) + reviewer (visibility) |
| 4 | Lists are `ul`/`ol`/`dl`, tables use `th scope`, `caption`, `thead/tbody`; never fake structure with divs and CSS | `**/*.html` (+ tsx/jsx/vue/svelte) | W | B | warn: fake-structure recorded | E: `audit.py` (`no-fake-heading`) + `html-validate` |
| 5 | Forms: every input has a `label`, errors use `aria-describedby` + `aria-invalid`, required marked with `required` (and visually, not by colour alone) | `**/*.html` | W | B | warn: unlabeled-input recorded | P: `html-validate` output + behaviour check pasted |
| 6 | Images carry `alt` (empty for decorative); video/audio ship captions or transcripts; autoplay muted-only and off by default | `**/*.html` | W | B | warn: missing-alt/caption recorded | P: checklist + axe output pasted |
| 7 | Buttons are `button`, links are `a[href]`; never a clickable div — keyboard, focus, and semantics come free with the right element | `**/*.html`, `**/*.tsx`, `**/*.jsx`, `**/*.vue`, `**/*.svelte` | D | B | deny: blocked at pre-write | E: `audit.py` (`no-div-button`) + `pre_tool_guard.py` + `html-validate` (`prefer-native-element`) |
| 8 | `lang` on `html`, unique descriptive `title` per page, correct `meta viewport`; `meta charset=utf-8` first in head | `**/*.html` | W | B | warn: missing-meta recorded | E: `html-validate` |
| 9 | Validate: run html-validate on every template change; zero errors before merge, warnings triaged not ignored | changed templates | W | B | warn: unvalidated-template recorded | E: `html-validate` exit 0 pasted |
| 10 | No duplicate `id`s (break skip links and `aria-*` references) | `**/*.html` | W | B | warn: duplicate-id recorded | E: `html-validate` |
| 11 | `placeholder` is not a label; `tabindex` above 0 hijacks natural order — neither ships | `**/*.html` | W | B | warn: recorded at audit | E: `html-validate` + `eslint` (jsx-a11y where applicable) |
| 12 | Well-formed: no unclosed tags, no stray `</div>` cascades | `**/*.html` | W | B | warn: malformed-markup recorded | E: `html-validate` |
| 13 | Versions: first read the project doctype and validator config (.htmlvalidate.json); HTML5 living standard is the floor; follow the project subset (XHTML, email-HTML) where one exists | repo root | W | B | warn: unchecked-doctype recorded | P: doctype + config pasted |
| 14 | Versions: check-date > 12mo → re-verify skill vs WHATWG standard and html-validate rule changes in same change | `skills.md` | W | B | warn: stale-pin recorded | P: dated verification note pasted |
| 15 | Sources (WHATWG HTML Standard, W3C WAI-ARIA practices, html-validate docs) cited, not duplicated | `skills.md` | W | B | warn: uncited normative claim recorded | P: source line present in change |
| 16 | Canonical-source rule: author here, legacy flat copy is reference-only | `skills.md` | W | B | warn: authoring in legacy copy recorded | P: diff shows write-set path only |
| 17 | Accessibility bar: axe/lighthouse with zero criticals on changed pages | changed pages | W | B | warn: un-gated a11y recorded | P: axe/lighthouse output pasted |
| 18 | Done-gate: `html-validate`, `prettier --check .`, `eslint` (jsx-a11y where applicable) all pass | changed files | W | B | warn: un-gated change recorded; stop vetoes until pasted | P: all three checker outputs pasted exit 0 |

No D-reads-W projection subtlety: the single D row (7) enforces at pre-write/audit; all other rows warn at audit/stop. Binding is uniform (all 18 are B — see `binding_of`).

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows) + `rules-languages.json` (15 rows, VERIFIED 2026-10-31: all 15 read, none scopes `*.html` — matches `hooks.md` Hook-4 row). Rows scoping `**/*.html`: `no-div-button` (D), `no-fake-heading` (W). Universal deny rows (`**`): `no-secrets-aws-key`, `no-private-key`.

| Pair | Verdict |
|---|---|
| #7 (never a clickable div) vs `no-div-button` (D) | CONSISTENT — deny both sides; pre-write block. Binding wins. |
| #1 + #4 (one `h1`, real headings, never fake structure) vs `no-fake-heading` (W) | CONSISTENT — same direction; union warns the styled-div/span subset. Binding wins. |
| #5 (form/error semantics) vs `no-fake-heading` (W) | CONSISTENT — same direction (native semantics over div soup). Binding wins. |
| #9 (zero validator errors pre-merge) vs `html-validate` `prefer-native-element` on div-button | CONSISTENT — independent E-gate confirms the deny (VERIFIED 2026-10-31: `bad_div_button.html` → `prefer-native-element` error, exit 1). Binding wins. |
| skills.md silent on secrets vs `no-secrets-aws-key` / `no-private-key` (D, universal) | CONSISTENT — no permission granted; union deny governs. Binding wins. |
| skills.md silent on `TODO`/stubs vs `no-todo-stub` (W, no html scope) | CONSISTENT — union does not scope html; no conflict either way. |
| `rules-languages.json` (15 rows) vs any html row | NO OVERLAP — none scopes `*.html`/`*.htm` (VERIFIED by scope read 2026-10-31). |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0.

## Compiled predicates (tool-write chain, RULE AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–18), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no html):

```ts
export function sev_of(rule: number): number {
  if (rule === 7) { return 2; }
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
  if (rule === 5) { return 3; }
  if (rule === 6) { return 3; }
  if (rule === 13) { return 3; }
  if (rule === 14) { return 3; }
  if (rule === 15) { return 3; }
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
export function file_gate(deny_hits: number, warn_hits: number, has_html: number): number {
  if (has_html === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-html.

Rust port (`html_rules.rs`, same `f64` convention as Hook-4 `html_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Hand-normalized from transpiler output (parens de-nested, `===` → `==`); semantics identical to the TS above.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 7f64 { return 2f64; }
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
  if rule == 5f64 { return 3f64; }
  if rule == 6f64 { return 3f64; }
  if rule == 13f64 { return 3f64; }
  if rule == 14f64 { return 3f64; }
  if rule == 15f64 { return 3f64; }
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
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_html: f64) -> f64 {
  if has_html == 0f64 { return 0f64; }
  if deny_hits > 0f64 { return 2f64; }
  if warn_hits > 0f64 { return 1f64; }
  return 0f64;
}
```

## Receipt (tool-write, one per rules file)

- rules: `html/rules` (RULE AGENT 4 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-html/html_rules.ts`): `ec3691cec5be914650566fe01243e99f36134083d698e7dc64458d6f0b2b1d4e`
- Rust output hash (sha256 of `/tmp/n3-rule-html/html_rules.rs` as compiled): `c981734131bd352a337693122ad50fd40b4f9d373fac506ac0df0e445b29e931`
- `ts_strict`: true — `tsc --noEmit --strict` (system `tsc 7.1.0-beta`), exit 0, no output; `ts_check` (tsrs TS 7.1) ok true, exit 0
- `rust_port_supported`: true — unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib html_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-html/dryrun`, `-D warnings`, exit 0): `kept=18/18 deny=1 warn=17 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0` (all 18 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/19 → 0; binding-beats-advisory both directions: `bw_vs_ad=1 ad_vs_bw=1`; file gate `deny=2 warn=1 clean=0 nohtml=0`)
- html gate log (`/tmp/n3-rule-html/dryrun/`, html-validate v11.17.0): `good.html` → exit 0, no findings; `bad_div_button.html` → `4:30 error Prefer to use the native <button> element (prefer-native-element)`, exit 1 (independent E-gate confirmation of D row 7); `audit.py /tmp/n3-rule-html/dryrun` → `audit: 3 files, 1 deny, 1 warnings` + `[DENY] bad_div_button.html (no-div-button)` + `[WARN] bad_fake_heading.html (no-fake-heading)`, exit 2
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`Command`/`process::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `html/rules.md`.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in predicate sources (grep exit 1 = no matches).
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
