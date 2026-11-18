# html / e2e-receipt (E2E AGENT 2 of 2, Batch 4 Round 4)

Date: 2026-10-31. Status: E2E RUN COMPLETE (honest verdict table — 9/9 MIGRATED, 1 FOLLOW-UP).
Scope: dummy packet end-to-end through the whole html migrated chain.
Write discipline: CREATE only — no lane file (`tools.md`, `hooks.md`, `loops.md`, `skills.md`,
`rules.md`, `subagent.md`, `swarm.md`, `decomposition.md`, `preregistration.md`) was modified.
All nine lane files were read first, then the packet was driven through the chain in order.

## 1. Dummy packet

`/tmp/n3-e2e-html/packet.json` (VERIFIED — created by this agent 2026-10-31; the expected
path did not exist at task start, so an equivalent packet was written: clean-HTML dummy,
`checker=lint`, 100/1024 bytes, zero strikes; REPRODUCIBLE from this text):

    {
      "id": "dummy-html-e2e-001",
      "checker": "lint",
      "byte_len": 100,
      "max_bytes": 1024,
      "strikes": 0,
      "iters": 1
    }

- sha256 (VERIFIED via `shasum -a 256`): `563d80af7f5da32d10e0032a739d88508e100c46b82042e028f67daafbb57fb2`

## 2. Chain log (VERIFIED — compiled Rust driver, exit 0)

Driver: `/tmp/n3-e2e-html/e2e_drive.rs` (compiled `rustc --edition 2021` against the lane's
own compiled sources via `include!`: `html_prereg_norm.rs`, `html_decomp.rs`, `swarm_lib.rs`,
`loop_machine.rs`, `html_hook_norm.rs`, `html_rules.rs`, `html_subagent.rs`, plus `tools_lane.rs`
extracted byte-identical from the `tools.md` code blocks). Compile exit 0 (only `dead_code`
warnings — the driver exercises a subset of each lane module's fns; every called fn
compiled `-D warnings`-clean in its lane harness). Run exit 0. Pasted output:

    [s1-prereg] version_chain_ok(1,0)=1 judge_pass(0,pasted)=1 bar_met(6/6)=1 gate_open=1 -> PASS
    [s2-decomp] typed_ok(1,2,3,4)=1 dag_ok()=1 gate_open=1 -> PASS
    [s3-swarm] declare order_ok=true run route=lint->html-validate budget_over=false judge=true escalate=false sent=true -> PASS
    [s4-loop] packet=dummy-html-e2e-001 final=Receipt attempts=3 strikes=0 iters=3 -> PASS
    [s5-hook] html_gate(clean)=0 pre_guard=allow stop=allow rules:sev(7)=deny(2) sev(1)=warn(1) file_gate(clean)=0 -> PASS
    [s6-tool] route=lint->html-validate over_budget=false gate_send=true receipt.sent=true -> PASS
    [s7-subagent] created->injected->running allow_route=1 allow_network=0 frozen(strikes=2)=0 judge(html-validate=1,eslint-missing)=pass-recorded dispose->disposed is_disposed=1 -> PASS
    E2E_CHAIN_COMPLETE

Step mapping: prereg sign (S1) → decomposition typecheck (S2) → swarm DAG declare+run (S3) →
loop drive Plan→Run→Judge→Receipt (S4) → hook gate (S5) → tool route (S6) →
subagent lifecycle/allowlist (S7). Each step pasted pass/fail above.

Independent corroboration (VERIFIED — lane's own artifacts re-run 2026-10-31, plus real harness):
- decomp dryrun exit 0: `DAG_ORDER input(0)->output(1)->judge(2)->receipt(3) dag=1 typed=1 gate=1`, `ALL_DECOMP_PASS`
- swarm dag_run exit 0: `route(lint)->html-validate`, `budget(100,1024)->over=false`, `judge(true,true)->true`, `escalate(0,1)->false`, `sent=true`, `DAG-TYPECHECK-OK`
- loop_test exit 0: T1 `final=Receipt attempts=3 strikes=0 iters=3`, T2 escalated-halt (`final=Plan attempts=0 strikes=2 iters=0`), T3 bound ok, `ALL_TERMINATED`
- rules dryrun exit 0: `kept=18/18 deny=1 warn=17 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0`, `bw_vs_ad=1 ad_vs_bw=1`, file gate `deny=2 warn=1 clean=0 nohtml=0`
- hook dryrun exit 0: `html_gate_deny=2 html_gate_warn=1 html_gate_clean=0 html_gate_nohtml=0`, `pre_deny=deny pre_allow=allow pre_schema_fail=deny`, `stop_hit=continue stop_archived=allow stop_clean=allow`, `scan_btn_bad=1 scan_btn_clean=0 scan_btn_native=0`, `scan_head_bad=1 scan_head_clean=0 scan_head_card=0`
- subagent dryrun exit 0: `ALL_TERMINATED ok`
- `rustc --crate-type lib -D warnings` over `html_prereg_norm.rs` exit 0 and over the `tools.md`-extracted Rust block exit 0 (hash matches lane receipt as header+block: see §3)
- `pre_tool_guard.py` on the lane's own fixtures (real harness): `bad_div_button.html` → `deny` (`no-div-button`), `bad_secret.html` → `deny` (`no-secrets-aws-key`), `bad_fake_heading.html` → `allow` (warn-only never denies at pre-guard), `good.html` → `allow`; `audit.py /tmp/n3-hook-test-html` → `audit: 4 files, 2 deny, 1 warnings`, exit 2
- html-validate 11.17.0 (PRESENT — VERIFIED via `html-validate --version` 2026-10-31) on the lane's own fixtures: `good.html` → exit 0, no findings; `bad_div_button.html` (`<div class="card" role="button" tabindex="0">`) → `1:19 error Prefer to use the native <button> element (prefer-native-element)`, exit 1 — independent E-gate confirmation of deny row 7
- eslint standing: config-missing corroborated by loops.md (`eslint.config.*` not found, exit 2) → recorded as MISSING per hooks.md stop-gate contract; S7 shows `judge_gate(1,0)=pass-recorded`, never a clean pass

Compilers (VERIFIED): `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `node v26.9.0`, `tsc Version 7.1.0-beta`, `prettier 3.9.10`, `html-validate 11.17.0`.

## 3. Per-artifact verdicts (artifact × all-gates-true? → MIGRATED or BLOCKED/FOLLOW-UP)

Lane receipts below are REPORTED (read from the lane files); the MIGRATED verdicts are
INFERRED by applying the all-gates-true rule. All full-length source/Rust hashes named in
the lane receipts were additionally VERIFIED (recomputed via `shasum -a 256` 2026-10-31 —
every one matches the lane file, including the prereg raw `a2351bb8…11a74ba` + norm
`448924b9…5d84cb` + TS `dc46b73b…9533220` + mutant `138f0c0a…09dd368a`, the subagent TS
`024854d1…b3a62d`, and the `tools.md` Rust block under the header noted below). Loops.md
hashes are truncated (`b3952d81…38f73e81`, `1bef2a59…012c6916`, `7a1c4bef…f6394d56`) so
they are corroborated (prefix-match against recomputed full hashes) rather than fully
re-verified — stated, not silently passed.

| artifact | gates (per lane receipt) | verdict |
|---|---|---|
| tools.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (TS hash re-verified; Rust hash re-verified as transpiler header + verbatim code block, matching the prereg-lane header convention) | MIGRATED |
| decomposition.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (both hashes re-verified) | MIGRATED |
| swarm.md | ts_check=true, ts_to_rust=true (unsupported []), rust_compile=true, sent=true (both hashes re-verified) | MIGRATED |
| loops.md | ts_check=true (strict exit 0), ts_to_rust=true, `tsc --allowJs --checkJs` exit 0, `node --test` pass 1/1, prettier exit 0, `rustc --deny warnings` lib+bin exit 0, termination proof exit 0, html-validate 11.17.0 probe exit 0 | MIGRATED |
| subagent.md | ts_strict=true (ts_check exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), dryrun exit 0, sent=true (both hashes re-verified) | MIGRATED |
| rules.md | ts_strict=true (exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), sent=true (both hashes re-verified) | MIGRATED |
| preregistration.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true (raw + norm exit 0), gate_open=1, sent=false BY DESIGN (record, not send) | MIGRATED (gates all true; sent=false is the specified prereg semantic) |
| hooks.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, dryrun exit 0, sent=true (both hashes re-verified) | MIGRATED |
| skills.md | ts_strict=true (interface probe), rust CI compiled logic covered by tools.md SENT receipt, sent=false BY DESIGN (interface-only, bodies live in tools.md) | MIGRATED (interface-only by Round-2 spec; no compiled gate of its own to fail) |

FOLLOW-UP (not a blocker, recorded per documentation standards — the lane's own `skills.md`
names it): `skills.md` notes `rules.md` text still states "Normative rules live in `skills.md`
(`## Rules` section)" — that section no longer exists after the interface shrink; the
rules-file owner must repoint it (to `tools.md` checkers or the new canonical rule home).
Out of this agent's write set — NOT fixed here.

## 4. Honest accounting

- Full E2E chain PASS (all 7 driver steps green, exit 0) is consistent with the per-artifact
  table: every compiled artifact the chain exercises reports all-gates-true in its lane
  receipt, and the S5/S7 steps corroborate the warn-vs-deny, route, and pass-recorded
  semantics against the real harness (not just the compiled predicates).
- Nothing was overridden: unlike the rust-lane E2E (2 BLOCKED: stub hook, STOPped skills),
  the html lane ships compiled hook predicates + dryrun, a hard html-validate gate that is
  PRESENT (not NEEDS-VERIFY — corrected in-file with the measured version), and an
  interface-only skills file whose compiled logic is covered by the tools.md SENT receipt —
  all VERIFIED above.
- `sent=false` appears twice (preregistration, skills) and both are BY DESIGN with the
  reason pasted in the table — neither is a silent non-send.
- Lane files modified by this agent: NONE. This file is the sole lane-dir CREATE.
  Scratch + driver sources live in `/tmp/n3-e2e-html/` (outside the lane, per write set).
- NOT checked (stated per §7.5): `axe`/`lighthouse`/`vnu`/`tidy` runs (not exercised for this
  probe); checker coverage for them rests on the lane's `tools.md` receipt, not on this run.
