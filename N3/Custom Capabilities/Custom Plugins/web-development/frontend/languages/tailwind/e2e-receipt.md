# tailwind / e2e-receipt (E2E AGENT 2 of 2, Batch 5 Round 4)

Date: 2026-10-31. Status: E2E RUN COMPLETE (honest verdict table — 9/9 MIGRATED, 1 FOLLOW-UP).
Scope: dummy packet end-to-end through the whole tailwind migrated chain.
Write discipline: CREATE only — no lane file (`tools.md`, `hooks.md`, `loops.md`, `skills.md`,
`rules.md`, `subagent.md`, `swarm.md`, `decomposition.md`, `preregistration.md`) was modified
(lane-dir mtimes all predate this run — VERIFIED via `ls -lT` 2026-10-31).
All nine lane files were read first, then the packet was driven through the chain in order.

## 1. Dummy packet

`/tmp/n3-e2e-tailwind/packet.json` (VERIFIED — created by this agent 2026-10-31; the expected
path did not exist at task start, so an equivalent packet was written: clean-CSS dummy,
`checker=lint`, 100/1024 bytes, zero strikes; REPRODUCIBLE from this text):

    {
      "id": "dummy-tailwind-e2e-001",
      "checker": "lint",
      "byte_len": 100,
      "max_bytes": 1024,
      "strikes": 0,
      "iters": 1
    }

- sha256 (VERIFIED via `shasum -a 256`): `2fafc10230eb70bbe7f865b8f6fbc90c9de78cafbdcb68f4fb4d1a299425a2f3`

## 2. Chain log (VERIFIED — compiled Rust driver, run exit 0)

Driver: `/tmp/n3-e2e-tailwind/e2e_drive.rs` (compiled `rustc --edition 2021` against the lane's
own compiled sources via `include!`: `tail_prereg_norm.rs`, `tw_decomp.rs`, `swarm_lib.rs`,
`loop_machine.rs`, `tw_hook_norm.rs`, `tw_rules.rs`, `tw_subagent.rs`, plus `tools_lane.rs`
extracted byte-identical from the `tools.md` code blocks). Compile exit 0 (only `dead_code`
warnings — the driver exercises a subset of each lane module's fns; every called fn
compiled `-D warnings`-clean in its lane harness). Run exit 0. Pasted output:

    [s1-prereg] version_chain_ok(1,0)=1 judge_pass(0,pasted)=1 bar_met(6/6)=1 gate_open=1 -> PASS
    [s2-decomp] typed_ok(1,2,3,4)=1 dag_ok()=1 gate_open=1 -> PASS
    [s3-swarm] declare order_ok=true run route=lint->stylelint budget_over=false judge=true escalate=false sent=true -> PASS
    [s4-loop] packet=dummy-tailwind-e2e-001 final=Receipt attempts=3 strikes=0 iters=3 -> PASS
    [s5-hook] tw_gate(clean)=0 pre_guard=allow stop=allow rules:sev(4)=1 file_gate(clean)=0 -> PASS
    [s6-tool] route=lint->stylelint over_budget=false gate_send=true receipt.sent=true -> PASS
    [s7-subagent] created->injected->running allow_route=1 allow_network=0 frozen(strikes=2)=0 judge(tw=1,prettier-missing)=pass-recorded dispose->disposed is_disposed=1 -> PASS
    E2E_CHAIN_COMPLETE

Step mapping: prereg sign (S1) → decomposition typecheck (S2) → swarm DAG declare+run (S3) →
loop drive Plan→Run→Judge→Receipt (S4) → hook gate (S5) → tool route (S6) →
subagent lifecycle/allowlist (S7). Each step pasted pass/fail above.

Independent corroboration (VERIFIED — lane's own artifacts re-run 2026-10-31, plus real harness):
- decomp dryrun exit 0: `DAG_ORDER input(0)->output(1)->judge(2)->receipt(3) dag=1 typed=1 gate=1`, `ALL_DECOMP_PASS`
- swarm dag_run exit 0: `route(lint)->stylelint`, `budget(100,1024)->over=false`, `judge(true,true)->true`, `escalate(0,1)->false`, `sent=true`, `DAG-TYPECHECK-OK`
- loop_test exit 0: T1 `final=Receipt attempts=3 strikes=0 iters=3`, T2 escalated-halt (`final=Plan attempts=0 strikes=2 iters=0`), T3 bound ok, `ALL_TERMINATED`
- rules dryrun exit 0: `kept=18/18 deny=0 warn=18 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0 e=11 p=7 bw_vs_ad=1 ad_vs_bw=1`, file gate `deny=2 warn=1 clean=0 nocss=0`
- hook dryrun exit 0: `tw_gate_deny=2 tw_gate_warn=1 tw_gate_clean=0 tw_gate_nocss=0`, `pre_deny=deny pre_allow=allow pre_schema_fail=deny`, `stop_hit=continue stop_archived=allow stop_clean=allow`, `scan_theme_good=1 scan_theme_bad=0`
- subagent dryrun exit 0: `ALL_TERMINATED ok`
- `pre_tool_guard.py` with canonical `HOOK_EVENT_SCHEMA.md` envelope on the lane's own hook fixtures (real harness, re-run 2026-10-31): `bad_secret.css` → `deny` (`no-secrets-aws-key`, fail-CLOSED envelope `Blocked bad_secret.css ... Matched: 'AKIAIOSFODNN7EXAMPLE'`), `good.css` → `allow`, `bad_color.css` → `allow` (warn-only never denies at pre-guard — by contract)
- `audit.py /tmp/n3-rule-tw/dryrun` → `audit: 10 files, 1 deny, 7 warnings` + all 5 css-scoped union warns + `no-todo-stub` + universal `[DENY] bad_secret.css (no-secrets-aws-key)`; `good.css` `@theme`-token-definition hex also flags `no-hardcoded-color` (definition-site triage per rules.md — consumption sites governed, not the token definition)
- `prettier --check` over lane fixtures (`good.css` hook + rules): `All matched files use Prettier code style!`, exit 0
- tailwindcss standing: binary MISSING on machine PATH (VERIFIED `npx --no-install tailwindcss --version` fails, 2026-10-31 — matches hooks.md Hook-4 and subagent.md judge policy); S7 shows `judge_gate(1,0)=pass-recorded`, never a clean pass. The loops.md `tailwindcss v4.3.3` content-scan gate ran from the sandbox install `/tmp/n3-tw-install` (still on disk; `probe.out.css` hash `139d31f1…cebfdee` prefix-matches the lane receipt) — carried signal, honestly labeled, not a machine-PATH pass
- `rustc --crate-type lib` over the `tools.md`-extracted Rust block exit 0 (hash matches lane receipt as emission — see §3)

Compilers (VERIFIED): `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `node v26.9.0`, `tsc Version 7.1.0-beta`, `prettier 3.9.10`.

## 3. Per-artifact verdicts (artifact × all-gates-true? → MIGRATED or BLOCKED/FOLLOW-UP)

Lane receipts below are REPORTED (read from the lane files); the MIGRATED verdicts are
INFERRED by applying the all-gates-true rule. All full-length source/Rust hashes named in
the lane receipts were additionally VERIFIED (recomputed via `shasum -a 256` 2026-10-31 —
every one matches the lane file, including the prereg raw `a2351bb8…11a74ba` + norm
`448924b9…5d84cb` + TS `dc46b73b…9533220` + mutant `138f0c0a…09dd368a`, the subagent TS
`56ebbdd3…07a4765a` + Rust `a924de2b…c86ed31`, the hooks TS `6407bb9f…a6faf98` + Rust
`1519a4e6…851c61e0`, the tools TS `26d5c722…4490a73` + Rust `55cba7f7…d159e0a`
(re-verified against the byte-identical extraction in `/tmp/n3-e2e-tailwind/`), the
decomp TS `986d52ba…5aef993a` + Rust `4532482c…59b70da89e`, the rules TS
`dd07b6a0…5cbf0363` + Rust `fc9624fa…b148a34944`, and the swarm TS `9c0418e3…72755eb1`
+ Rust `fd966488…0f800e272f`). Loops.md hashes are truncated (`4539008a…6591b109`,
`7d83c64b…d8366e3b`, `84adc326…b5b87e51`, `a3c85d5d…aba5d67`, `139d31f1…cebfdee`) so
they are corroborated (prefix-match against recomputed full hashes) rather than fully
re-verified — stated, not silently passed.

| artifact | gates (per lane receipt) | verdict |
|---|---|---|
| tools.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (both hashes re-verified against extraction) | MIGRATED |
| decomposition.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (both hashes re-verified) | MIGRATED |
| swarm.md | ts_check=true, ts_to_rust=true (unsupported []), rust_compile=true, sent=true (both hashes re-verified) | MIGRATED |
| loops.md | ts_check=true (strict exit 0), ts_to_rust=true, `tsc --allowJs --checkJs` exit 0, `node --test` pass 1/1, prettier exit 0, `rustc --deny warnings` lib+bin exit 0, termination proof exit 0, tailwindcss v4.3.3 content-scan exit 0 (sandbox install), html-validate 11.17.0 probe exit 0 | MIGRATED |
| subagent.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, dryrun exit 0, sent=true (both hashes re-verified) | MIGRATED |
| rules.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, sent=true (both hashes re-verified) | MIGRATED |
| preregistration.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true (raw + norm exit 0), gate_open=1, sent=false BY DESIGN (record, not send) | MIGRATED (gates all true; sent=false is the specified prereg semantic) |
| hooks.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, dryrun exit 0, sent=true (both hashes re-verified) | MIGRATED |
| skills.md | ts_strict=true (interface probe), rust CI compiled logic covered by tools.md SENT receipt, sent=false BY DESIGN (interface-only, bodies in tools.md) | MIGRATED (interface-only by Round-2 spec; no compiled gate of its own to fail) |

FOLLOW-UP (not a blocker, recorded per documentation standards — the lane's own `skills.md`
names it): `rules.md` states "Normative rules live in `skills.md` (`## Rules` section)" — that
section no longer exists after the Round-2 interface shrink; the rules-file owner must repoint
it (to `tools.md` checkers or the new canonical rule home). Out of this agent's write set —
NOT fixed here.

## 4. Honest accounting

- Full E2E chain PASS (all 7 driver steps green, run exit 0) is consistent with the per-artifact
  table: every compiled artifact the chain exercises reports all-gates-true in its lane
  receipt, and the S5/S7 steps corroborate the warn-vs-deny, tailwindcss-MISSING, and
  pass-recorded semantics against the real harness (not just the compiled predicates).
- Nothing was overridden: unlike lanes that ship stub hooks or STOPped skills, the tailwind
  lane ships compiled hook predicates + dry-run, a hard prettier gate that is PRESENT
  (not NEEDS-VERIFY), a tailwindcss-MISSING row that is VERIFIED on this machine with the
  `pass-recorded` verdict (never a clean pass), and an interface-only skills file whose
  compiled logic is covered by the tools.md SENT receipt — all VERIFIED above.
- `sent=false` appears twice (preregistration, skills) and both are BY DESIGN with the
  reason pasted in the table — neither is a silent non-send.
- Lane files modified by this agent: NONE. This file is the sole lane-dir CREATE.
  Scratch + driver sources live in `/tmp/n3-e2e-tailwind/` (outside the lane, per write set).
- NOT checked (stated per §7.5): `stylelint`/`postcss` runs (not exercised for this probe;
  route dispatch to `stylelint` is proven by the compiled predicate, execution coverage rests
  on the lane's `tools.md` receipt, not on this run); the loops.md `probe.out.css` build
  was corroborated by hash only, not re-executed (tailwindcss binary MISSING on machine PATH).
