# tailwind / rules

Date: 2026-10-31 (RULE AGENT 4 of 6, Round 2). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/tailwind/`.

Normative source: `skills.md` (`## Rules` 1–10 + Pitfalls + Versions + Done = 18 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`, or via named executable/pasted-output stop veto), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 18/18 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | v3 projects: all design tokens live in `tailwind.config.{js,ts}` (`theme.extend` only — never edit `theme` core scales directly) | v3 projects (`tailwind.config.{js,ts}`) | W | B | warn: core-scale edit recorded | E: `npx tailwindcss --version` + config read |
| 2 | v4 projects: CSS-first config via `@import "tailwindcss"` plus `@theme { --color-brand: ...; }`; no `tailwind.config` file unless interop demands it | v4 projects (`**/*.css`) | W | B | warn: config-file-in-v4 recorded | E: `npx tailwindcss --version` + build |
| 3 | Never mix v3 config-file and v4 CSS-first in one project; pick by installed version and follow it throughout | repo | W | B | warn: mixed-config recorded | E: `npx tailwindcss --version` |
| 4 | Extract repeated utility clusters into components (`@layer components` / framework components), not into new CSS files with magic values; utilities stay the source of truth | `**/*.css` | W | B | warn: magic-value recorded | E: `audit.py` (`no-hardcoded-color`) |
| 5 | Dark mode, breakpoints, and state variants use Tailwind variants (`dark:`, `md:`, `hover:`), never hand-written media queries duplicating the scale | `**/*.css` (+ templates) | W | B | warn: hand-media recorded | P: grep pasted showing no scale-duplicating `@media` |
| 6 | Arbitrary values (`w-[13px]`) are escape hatches: two uses of the same value means promote it to a token | templates + `**/*.css` / `@theme` | W | B | warn: repeated-arbitrary recorded | P: grep of `-[` pasted + token promotion |
| 7 | Keep class order deterministic with prettier-plugin-tailwindcss so diffs stay reviewable; never hand-sort | templates | W | B | warn: unordered-classes recorded | E: `prettier --check .` |
| 8 | Purge-safety: dynamically constructed class names (`bg-${c}`) are invisible to the scanner — safelist them or map them explicitly | templates + config content globs | W | B | warn: unscanned-class recorded | P: build log + safelist pasted |
| 9 | v4 upgrades: run the official upgrade tool, then hand-check `@apply` sites and custom plugins (the two most common breakages) | upgraded repos | W | B | warn: unchecked-apply/plugin recorded | E: upgrade tool + `vite build` / `tailwindcss` build |
| 10 | Ship only used CSS: verify output size in the build log; a full-reset or `@import` of preflight twice means the pipeline is miswired | build output | W | B | warn: bloated-output recorded | E: build log (size + single preflight) |
| 11 | Content globs must cover every template path or classes silently vanish in production (dev-works/prod-empty mystery) | v3 config content globs | W | B | warn: missing-content-path recorded | E: `tailwindcss` build (content gate) |
| 12 | `@apply` of complex variants fails at build time — keep `@apply` simple, verify at build | `**/*.css` | W | B | warn: complex-apply recorded | E: `vite build` / `tailwindcss` build |
| 13 | v4 `@theme` tokens need the `--*: initial` reset pattern for narrowed scales | `**/*.css` (`@theme`) | W | B | warn: unreset-scale recorded | E: build (scale resolves, no fallback leak) |
| 14 | Preflight resets fight legacy global CSS — isolate or remove the legacy; no global `*` reset | `**/*.css` | W | B | warn: global-reset recorded | E: `audit.py` (`no-global-reset`) |
| 15 | `!important` modifiers (`!mt-2` v3 / trailing `!` v4) signal a specificity war to refactor, not a fix | `**/*.css` + templates | W | B | warn: important-modifier recorded | P: `audit.py` (`no-important`) output + refactor note pasted |
| 16 | Versions: first run `npx tailwindcss --version` and read the project pin (package.json); write only for that version; if the project pins nothing, add a pin | repo root + `skills.md` | W | B | warn: unchecked-pin recorded | P: version + pin pasted |
| 17 | Versions: check-date over 12 months old → re-verify this skill against the release notes and update in the same change | `skills.md` | W | B | warn: stale-pin recorded | P: dated verification note pasted |
| 18 | Done-gate: `vite build` (or `tailwindcss -i input.css -o output.css --minify`), `prettier --check .`, `eslint`, visual review at 360px + 1280px all pass | changed files | W | B | warn: un-gated change recorded; stop vetoes until pasted | P: all checker outputs pasted |

No D-reads-W projection subtlety: all 18 rows are W (skills.md grants no deny; union deny rows are universal and govern silently — see contradiction table). Binding is uniform (all 18 are B — see `binding_of`).

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows, VERIFIED 2026-10-31) + `rules-languages.json` (15 rows, VERIFIED 2026-10-31: all 15 read, none scopes `*.css` — matches `hooks.md` Hook-4 row). Rows scoping `**/*.css`: `no-hardcoded-color`, `no-global-reset`, `no-important`, `no-transition-all`, `no-linear-ease` (all W) + `no-todo-stub` (W, scope incl. `**/*.css`). Universal deny rows (`**`): `no-secrets-aws-key`, `no-private-key`.

| Pair | Verdict |
|---|---|
| #4 + #6 (tokens as contract, no magic values, arbitrary→token) vs `no-hardcoded-color` (W, `**/*.css`) | CONSISTENT — same direction; union warns literal colors bypassing tokens. Binding wins. |
| #14 (isolate legacy, no global `*` reset) vs `no-global-reset` (W) | CONSISTENT — deny neither side; warn both sides. Binding wins. |
| #15 (`!important` = refactor, not fix) vs `no-important` (W) | CONSISTENT — same direction; union warns outside utilities layer. Binding wins. |
| skills.md silent on transitions/easing vs `no-transition-all` + `no-linear-ease` (W) | CONSISTENT — no permission granted; union warns. Binding wins. |
| skills.md silent on secrets vs `no-secrets-aws-key` / `no-private-key` (D, universal) | CONSISTENT — no permission granted; union deny governs. Binding wins. |
| skills.md silent on `TODO`/stubs vs `no-todo-stub` (W, scope incl. `**/*.css`) | CONSISTENT — union warns incl. css; no conflict either way. Binding wins. |
| `rules-languages.json` (15 rows) vs any tailwind row | NO OVERLAP — none scopes `*.css` (VERIFIED by scope read 2026-10-31). v3 `tailwind.config.{js,ts}` files are `*.js`/`*.ts` by extension: js/ts-scoped rows apply and are documented once in `javascript/hooks.md` / `typescript/hooks.md` (RETRIEVE — not duplicated here). |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0.

## Compiled predicates (tool-write chain, RULE AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–18), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no css):

```ts
export function sev_of(rule: number): number {
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
  if (rule === 8) { return 3; }
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
export function file_gate(deny_hits: number, warn_hits: number, has_css: number): number {
  if (has_css === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 1 = warn, 0 = out-of-range (no deny rows — skills.md grants no deny); binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-css.

Rust port (`tw_rules.rs`, same `f64` convention as Hook-4 `tw_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Hand-normalized from transpiler output (parens de-nested, `===` → `==`); semantics identical to the TS above.

```rust
// Generated by ts-to-rust v1 (subset). Unsupported items are listed, not hidden.
pub fn sev_of(rule: f64) -> f64 {
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
  if rule == 8f64 { return 3f64; }
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
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_css: f64) -> f64 {
  if has_css == 0f64 { return 0f64; }
  if deny_hits > 0f64 { return 2f64; }
  if warn_hits > 0f64 { return 1f64; }
  return 0f64;
}
```

## Receipt (tool-write, one per rules file)

- rules: `tailwind/rules` (RULE AGENT 4 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-tw/tw_rules.ts`): `dd07b6a05f2e78c2c9c7cc8c88ff835dd68db81468782bf546e2751a5cbf0363`
- Rust output hash (sha256 of `/tmp/n3-rule-tw/tw_rules.rs` as compiled): `fc9624fa2197e8d4e1d668227a781b4f13efe8bcabe614f74e5230b148a34944`
- `ts_strict`: true — `tsc --noEmit --strict` (system `tsc 7.1.0-beta`), exit 0, no output
- `rust_port_supported`: true — unsupported list: [] (no strings, no methods, no object literals)
- `rust_compile`: true — `rustc --crate-type lib tw_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-tw/dryrun/`, `-D warnings`, exit 0): `kept=18/18 deny=0 warn=18 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0 e=11 p=7 bw_vs_ad=1 ad_vs_bw=1` (all 18 in range, severity=warn, binding=1, judge>=1 — 11 E + 7 P; out-of-range 0/19 → 0; kept(1)=1/kept(0)=0; binding-beats-advisory both directions: `bw_vs_ad=1 ad_vs_bw=1`; file gate `deny=2 warn=1 clean=0 nocss=0`)
- css gate log (`/tmp/n3-rule-tw/dryrun/`, fixtures `good.css` with `@theme` + token var, `bad_color.css`, `bad_global.css`, `bad_important.css`, `bad_trans.css`, `bad_ease.css`, `bad_todo.css`, universal `bad_secret.css`): `audit.py` → `audit: 10 files, 1 deny, 7 warnings` + `[WARN] bad_color.css (no-hardcoded-color)` + `[WARN] bad_global.css (no-global-reset)` + `[WARN] bad_important.css (no-important)` + `[WARN] bad_trans.css (no-transition-all)` + `[WARN] bad_ease.css (no-linear-ease)` + `[WARN] bad_todo.css (no-todo-stub)` + `[DENY] bad_secret.css (no-secrets-aws-key)`, exit 2 (all 5 css-scoped union warns + `no-todo-stub` fire; universal deny fires)
- triage note (dated 2026-10-31, not a silent pass): `good.css` `@theme { --color-brand: #1a56db; }` token-DEFINITION hex also flags `no-hardcoded-color` — definition-site accepted (union excl. covers the design-language tokens path; project `@theme` definitions triaged the same way); the rule governs consumption sites (must use `var(--color-brand)`). `prettier --check good.css` → `All matched files use Prettier code style!`, exit 0 (prettier 3.9.10 PRESENT). `tailwindcss` binary MISSING (`npx --no-install tailwindcss --version` fails, exit 1 — VERIFIED, matches `hooks.md` Hook-4 row; content/audit gate carries the signal). `eslint` 10.14.0 PRESENT (binary; no `eslint.config.*` in fixture dir — flat config required since v9, so no-config run is not a gate signal). `vite` 8.3.5 PRESENT (`npx --no-install vite --version` exit 0; no project fixture to build — build gate applies to real projects). `node` v26.9.0.
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`Command`/`process::` (pure `f64` functions, no imports; grep count 0, exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `tailwind/rules.md`.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in predicate sources (grep count 0, exit 1 = no matches).
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
