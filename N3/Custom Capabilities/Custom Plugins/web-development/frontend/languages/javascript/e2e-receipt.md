# javascript / e2e-receipt (E2E AGENT 2 of 2, Batch 3 Round 4)

Date: 2026-10-31. Status: E2E RUN COMPLETE (honest verdict table — 9/9 MIGRATED, 1 FOLLOW-UP).
Scope: dummy packet end-to-end through the whole javascript migrated chain.
Write discipline: CREATE only — no lane file (`tools.md`, `hooks.md`, `loops.md`, `skills.md`,
`rules.md`, `subagent.md`, `swarm.md`, `decomposition.md`, `preregistration.md`) was modified.
All nine lane files were read first, then the packet was driven through the chain in order.

## 1. Dummy packet

`/tmp/n3-e2e-js/packet.json` (VERIFIED — created by this agent 2026-10-31; the expected
path did not exist at task start, so an equivalent packet was written: clean-JS dummy,
`checker=lint`, 100/1024 bytes, zero strikes; REPRODUCIBLE from this text):

    {
      "id": "dummy-js-e2e-001",
      "checker": "lint",
      "byte_len": 100,
      "max_bytes": 1024,
      "strikes": 0,
      "iters": 1
    }

- sha256 (VERIFIED via `shasum -a 256`): `ee3150cccb6fd7a8a1f1472ed3ce27055c24803540b658da68f139ac245859b6`

## 2. Chain log (VERIFIED — compiled Rust driver, exit 0)

Driver: `/tmp/n3-e2e-js/e2e_drive.rs` (compiled `rustc --edition 2021` against the lane's
own compiled sources via `include!`: `js_prereg_norm.rs`, `js_decomp.rs`, `swarm_lib.rs`,
`loop_machine.rs`, `js_hook_norm.rs`, `js_rules.rs`, `js_subagent.rs`, plus `tools_lane.rs`
extracted byte-identical from the `tools.md` code blocks). Compile exit 0 (only `dead_code`
warnings — the driver exercises a subset of each lane module's fns; every called fn
compiled `-D warnings`-clean in its lane harness). Run exit 0. Pasted output:

    [s1-prereg] version_chain_ok(1,0)=1 judge_pass(0,pasted)=1 bar_met(6/6)=1 gate_open=1 -> PASS
    [s2-decomp] typed_ok(1,2,3,4)=1 dag_ok()=1 gate_open=1 -> PASS
    [s3-swarm] declare order_ok=true run route=lint->eslint budget_over=false judge=true escalate=false sent=true -> PASS
    [s4-loop] packet=dummy-js-e2e-001 final=Receipt attempts=3 strikes=0 iters=3 -> PASS
    [s5-hook] js_gate(clean)=0 pre_guard=allow stop=allow rules:sev(6)=deny(2) sev(1)=warn(1) file_gate(clean)=0 -> PASS
    [s6-tool] route=lint->eslint over_budget=false gate_send=true receipt.sent=true -> PASS
    [s7-subagent] created->injected->running allow_route=1 allow_execute=0 frozen(strikes=2)=0 judge(node-test=1,eslint-missing)=pass-recorded dispose->disposed is_disposed=1 -> PASS
    E2E_CHAIN_COMPLETE

Step mapping: prereg sign (S1) → decomposition typecheck (S2) → swarm DAG declare+run (S3) →
loop drive Plan→Run→Judge→Receipt (S4) → hook gate (S5) → tool route (S6) →
subagent lifecycle/allowlist (S7). Each step pasted pass/fail above.

Independent corroboration (VERIFIED — lane's own artifacts re-run 2026-10-31, plus real harness):
- decomp dryrun exit 0: `DAG_ORDER input(0)->output(1)->judge(2)->receipt(3) dag=1 typed=1 gate=1`, `ALL_DECOMP_PASS`
- rules dryrun exit 0: `kept=18/18 deny=3 warn=15 cut=0 oob_bad=0 bind_bad=0 judge_bad=0 ko=0`, `file_gate: deny=2 warn=1 clean=0 nojs=0`
- loop_test exit 0: T1 `final=Receipt attempts=3 strikes=0 iters=3`, T2 escalated-halt (`final=Plan attempts=0 strikes=2 iters=0`), T3 bound ok, `ALL_TERMINATED`
- hook dryrun exit 0: `eval_tail_hit=1`, `scan_eval_bad=1 scan_eval_clean=0 scan_eval_prefixword=0`, `scan_sink_bad=1`, `js_gate_deny=2 js_gate_warn=1 js_gate_clean=0`, `pre_deny=deny pre_allow=allow pre_schema_fail=deny`, `stop_hit=continue stop_archived=allow stop_clean=allow`
- subagent dryrun exit 0: `ALL_TERMINATED ok`
- swarm dag_run exit 0: `dag_order_typechecks=true`, `route(lint)->eslint`, `judge(true,true)->true`, `escalate(0,1)->false`, `sent=true`, `DAG-TYPECHECK-OK` (note: the lane harness log line prints the edge type label as `TscCommand` verbatim — REPORTED quirk of the lane's harness string; the typecheck itself compares producer output == consumer input and passed)
- `rustc --crate-type lib -D warnings` over the `tools.md`-extracted Rust block: exit 0 (hash matches lane receipt: see §3)
- `pre_tool_guard.py` (real harness, JSON event on stdin): clean `probe/good.js` → `allow`; `eval(x);` in `probe/bad.js` → `deny` (`no-eval`) — deny-class fires exactly where the lane says it does
- eslint on the dummy packet JS: config-missing standing (no `eslint.config.*` in probe dir, exit 2 per loops.md) → eslint-recorded as MISSING per hooks.md stop-gate contract; corroborated by S7 `judge_gate(1,0)=pass-recorded`

Compilers (VERIFIED): `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `node v26.9.0`, `tsc Version 7.1.0-beta`.

## 3. Per-artifact verdicts (artifact × all-gates-true? → MIGRATED or BLOCKED/FOLLOW-UP)

Lane receipts below are REPORTED (read from the lane files); the MIGRATED verdicts are
INFERRED by applying the all-gates-true rule. All source/Rust hashes named in the lane
receipts were additionally VERIFIED (recomputed via `shasum -a 256` 2026-10-31 — every
one matches the lane file, including the `tools.md` TS block `46d37003…f56f648e` and Rust
block `7545cdd8…eb59` re-extracted byte-identical for the S6 driver step).

| artifact | gates (per lane receipt) | verdict |
|---|---|---|
| tools.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (hashes re-verified this run) | MIGRATED |
| decomposition.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true | MIGRATED |
| swarm.md | ts_check=true, ts_to_rust=true (unsupported []), rust_compile=true, sent=true | MIGRATED |
| loops.md | ts_check=true (strict exit 0), ts_to_rust=true, `tsc --allowJs --checkJs` exit 0, `node --test` pass 1/1, prettier exit 0, `rustc --deny warnings` lib+bin exit 0, termination proof exit 0 | MIGRATED |
| subagent.md | ts_strict=true (ts_check exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), dryrun exit 0, sent=true | MIGRATED |
| rules.md | ts_strict=true (exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), sent=true | MIGRATED |
| preregistration.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, gate_open=1, sent=false BY DESIGN (record, not send) | MIGRATED (gates all true; sent=false is the specified prereg semantic) |
| hooks.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, dryrun exit 0, sent=true | MIGRATED |
| skills.md | ts_strict=true (interface probe), rust CI compiled logic covered by tools.md SENT receipt, sent=false BY DESIGN (interface-only, bodies live in tools.md) | MIGRATED (interface-only by Round-2 spec; no compiled gate of its own to fail) |

FOLLOW-UP (not a blocker, recorded per documentation standards — the lane's own `skills.md`
names it): `skills.md` notes `rules.md` text still states "Normative rules live in `skills.md`
(`## Rules` section)" — that section no longer exists after the interface shrink; the
rules-file owner must repoint it (to `tools.md` checkers or the new canonical rule home).
Out of this agent's write set — NOT fixed here.

## 4. Honest accounting

- Full E2E chain PASS (all 7 driver steps green, exit 0) is consistent with the per-artifact
  table: every compiled artifact the chain exercised reports all-gates-true in its lane
  receipt, and the S5/S7 steps corroborate the warn-vs-deny, route, and pass-recorded
  semantics against the real harness (not just the compiled predicates).
- Nothing was overridden: unlike the rust-lane E2E (2 BLOCKED: stub hook, STOPped skills),
  the javascript lane ships compiled hook predicates + dryrun and an interface-only skills
  file whose compiled logic is covered by the tools.md SENT receipt — both VERIFIED above.
- `sent=false` appears twice (preregistration, skills) and both are BY DESIGN with the
  reason pasted in the table — neither is a silent non-send.
- Lane files modified by this agent: NONE. This file is the sole lane-dir CREATE.
  Scratch + driver sources live in `/tmp/n3-e2e-js/` (outside the lane, per write set).
- NOT checked (stated per §7.5): `vitest`/`jest`/`biome` runs (not installed for this probe);
  checker coverage for them rests on the lane's `tools.md` receipt, not on this run.
