# Failures.md

Date: 2026-10-30. Append-only. A recorded failure beats an asserted success.

## 2026-10-30 — `no-todo-stub` false positive on docstring ellipsis

- What: `audit.py` flagged `stop_completion_gate.py` — the `...` inside a docstring's example JSON (`"report_reason": ...`) matched the stub pattern.
- Cause: rule regex treats prose ellipsis the same as a code stub. INFERRED: acceptable for a warn-level rule (cheap to reword), not worth a context-aware parser.
- Disposition: reworded the docstring; rule unchanged. Per-item, no batch change.
- Lesson: warn-level rules SHOULD stay dumb and cheap; deny-level rules MUST stay precise. Recorded so the next false positive is judged against this bar, not as a surprise.

## 2026-10-30 — audit flagged the token file's own palette

- What: `no-hardcoded-color` fired on `tokens/tokens.css` — the one file where literals are canonical by design (Tier 1).
- Cause: rule had no exclusion mechanism.
- Disposition: added `exclude` support to `_engine.py` + excluded `**/tokens/tokens.css` in the rule. Engine fix, not a content fix.
- Lesson: every deny/warn system needs an escape hatch from day one; added before it bit twice.

## 2026-10-30 — open items (not failures, tracked risks)

1. `openapi.yaml` never parser-verified (no PyYAML in env). See Pre-Registration #5 for the exact settling command.
2. Hook event payloads (`pre_write`, `post_write`, `stop`) never fired by a live harness here — schema is PROPOSED until a harness round-trips it.
3. Kit's inherited risks adopted as open: critic-holds-the-pen, no a11y specialist role, directory-partitioning as the only concurrency control. See GLOBAL-LOGIC.md §6.
4. `modern-web-guidance` skill reported itself out of date (2026_05_16 vs 2026_10_16 available). Guidance retrieved under the old version; re-pull `css`/`design-token-reactivity` after upgrading.

## 2026-10-30 — PARTITION B pressure-test: two engine bugs caught by synthetic tests

- What: (1) `fnmatch("b.py", "**/*.py")` is False — extension-scoped rules never fired on directory-less paths. Every `**/*.ext` rule was silently dead for root-level files. (2) First fix attempt used `lstrip("**/")` which chars-strips (`"**/*.py"` → `".py"`); required `removeprefix`. (3) Exclude `**/hooks/**` failed on leading-component paths (`hooks/audit.py`); required directory-component matching.
- Cause: fnmatch `**` is not globstar; lstrip is not removeprefix; leading `**/` needs a slash the path may not have. All three INFERRED from direct REPL verification, then fixed and re-verified.
- Disposition: engine `_scope_match` now matches full path + basename + directory components. Synthetic suite (AKIA→deny, exec(→deny, clean→allow, print→append, stop→veto) all pass. Audit exit 0 at 126 files.
- Lesson: gate code needs its own hostile tests — the gates were green while half the rules were dead. Synthetic stdin cases are now part of the PARTITION B acceptance ritual.

## 2026-10-30 — Admitted-content warnings: recorded, not edited (batch disposition)

- Context: post-ADMIT audit showed 2 DENY + 33 warnings in third-party admitted scripts. DENY skills (gcp-spark-troubleshooting, super-pm) REMOVED per owner (unnecessary, covered elsewhere) — DENY count now 0 (VERIFIED).
- Remaining 24 warnings: `py-print-prod` + `no-todo-stub` inside admitted GCP/understand/skill-miner/toolchain example scripts. Per-item review: all are print/logging or TODO markers in third-party example/reference code, none on a prod path, none in N3-owned files.
- Disposition (dated, per WAIVE-OR-OBEY — this is a recorded reason, not a silent pass): NOT editing third-party example scripts to satisfy our linter; rewriting code we don't own adds risk without value. Revisit if any file becomes load-bearing (then it gets fixed or vendored-pin-excluded like the kit).
- Mechanical fix applied: commented the single `console.log` in skills/building-data-apps/examples/express_chat.ts (1 file, additive).

## 2026-10-30 — open items from languages reshape (not failures, tracked risks)

1. D9 in `discipline.py` still reads `N3/languages/*.md` + `plugin.json` (both gone in the concurrent reshuffle). Decision needed from Stewart: repoint D9 at `plugins/web-development/{frontend,backend,middleware}/languages/*/skills.md` + `languages-plugin.json`, or restore the flat dir. Recorded, not silently fixed — gate text change needs owner sign-off.
2. `audit.sh` path assumption: computes ROOT from its own location then appends `skills/web-development/...`, so it only runs from a checkout where `skills/` is a child dir (fails inside N3 with `can't open file .../N3/skills/web-development/hooks/audit.py`). Pre-existing; `audit.py` invoked directly works. Fix PROPOSED: derive ROOT as script-dir parent instead.
3. The other scaffold process's writes (TREE.md, placeholders, middleware split incl. HLSL-under-backend and swift-under-frontend quirks) were adopted as-is; taxonomy disputes are out of scope for this reshape — recorded for the next vetting pass.
