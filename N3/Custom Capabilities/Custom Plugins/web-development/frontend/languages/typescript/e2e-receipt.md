# typescript / e2e-receipt (E2E AGENT 2 of 2, Batch 2 Round 4)

Date: 2026-10-31. Status: E2E RUN COMPLETE (honest verdict table — 8/9 MIGRATED, 1/9 FOLLOW-UP).
Scope: dummy packet end-to-end through the whole typescript migrated chain.
Write discipline: CREATE only — no lane file (`tools.md`, `hooks.md`, `loops.md`, `skills.md`,
`rules.md`, `subagent.md`, `swarm.md`, `decomposition.md`, `preregistration.md`) was modified.
All nine lane files were read first, then the packet was driven through the chain in order.

## 1. Dummy packet

`/tmp/n3-e2e-ts/packet.json` (VERIFIED — created by this agent 2026-10-31; the expected
path did not exist at task start, so an equivalent packet was written: clean-TS dummy,
`checker=lint`, 100/1024 bytes, zero strikes; REPRODUCIBLE from this text):

    {
      "id": "dummy-ts-e2e-001",
      "checker": "lint",
      "byte_len": 100,
      "max_bytes": 1024,
      "strikes": 0,
      "iters": 1
    }

- sha256 (VERIFIED via `shasum -a 256`): `247a894b5f7ab66bbf01d4810bf02c1cc9069276a1679f80ec5676d82a0e7102`

## 2. Chain log (VERIFIED — compiled Rust driver, exit 0)

Driver: `/tmp/n3-e2e-ts/e2e_drive.rs` (sha256 `20758a8b72a1eb932fd16fb035c61e30d77480363ba060b11e9f19b31fa659d0`),
compiled `rustc --edition 2021` (dead-code lint allowed — driver exercises a subset of the
11+5+9+6+9+4 lane predicates per module; all called fns compiled `-D warnings`-clean in
their lane harnesses) against the lane's own compiled sources via `include!`
(`ts_prereg_norm.rs`, `ts_decomp.rs`, `swarm_lib.rs`, `loop_machine.rs`, `ts_hook.rs`,
`ts_rules.rs`, `ts_subagent.rs`, plus `tools_lane.rs` extracted byte-identical from the
`tools.md` Rust block). Run exit 0. Pasted output:

    [s1-prereg] version_chain_ok(1,0)=1 judge_pass(0,pasted)=1 bar_met(6/6)=1 gate_open=1 -> PASS
    [s2-decomp] typed_ok(1,2,3,4)=1 dag_ok()=1 gate_open=1 -> PASS
    [s3-swarm] declare order_ok=true run route=lint->eslint budget_over=false judge=true escalate=false sent=true -> PASS
    [s4-loop] packet=dummy-ts-e2e-001 final=Receipt attempts=3 strikes=0 iters=3 -> PASS
    [s5-hook] ts_gate(clean)=0 pre_guard=allow stop=allow rules:sev(2)=warn(1) file_gate(clean)=0 -> PASS
    [s6-tool] route=lint->eslint over_budget=false gate_send=true receipt.sent=true -> PASS
    [s7-subagent] created->injected->running allow_route=1 allow_execute=0 frozen(strikes=2)=0 judge(tsc=1,eslint-missing)=pass-recorded dispose->disposed is_disposed=1 -> PASS
    E2E_CHAIN_COMPLETE

Step mapping: prereg sign (S1) → decomposition typecheck (S2) → swarm DAG declare+run (S3) →
loop drive Plan→Run→Judge→Receipt (S4) → hook gate (S5) → tool route (S6) →
subagent lifecycle/allowlist (S7). Each step pasted pass/fail above.

Independent corroboration (VERIFIED — lane's own binaries re-run 2026-10-31, plus real judges):
- decomp dryrun exit 0: `DAG_ORDER ... dag=1 typed=1 gate=1`, `ALL_DECOMP_PASS`
- hook dryrun exit 0: all 22 asserts pass (`any_tail_hit=1`, `dot_then_hit=1`, `pre_deny=deny`, `stop_hit=continue`, …)
- loop_test exit 0: T1 `final=Receipt attempts=3 strikes=0 iters=3`, T2 escalated-halt, `ALL_TERMINATED`
- subagent dryrun exit 0: `ALL_TERMINATED ok`
- swarm dag_run exit 0: `dag_order_typechecks=true`, `route(lint)->eslint`, `judge(true,true)->true`, `DAG-TYPECHECK-OK`
- rules dryrun exit 0: `kept=16/16 deny=5 warn=11 cut=0 oob=0 ko=0`
- prereg asserts (fresh compile, `-D warnings` minus dead-code): `PREG_OK chain=1 dag=1 judge=1 bar=1 gate=1`
- `tsc --noEmit --strict --noUncheckedIndexedAccess` over tools/decomp/prereg/swarm/loop/hook TS sources: exit 0, no output
- `rustc --crate-type lib -D warnings` over the `tools.md`-extracted Rust block: exit 0 (hash matches lane receipt: see §3)
- `pre_tool_guard.py` (real harness): clean `good.ts` → `allow`; `label: any` → `allow` (warn-class rule fires at audit/stop, not pre-guard — matches hooks.md contract)
- `audit.py` (real harness) on probe dir with `label: any`: `0 deny, 1 warnings`, `[WARN] audit_probe.ts (ts-no-any)` — warn fires exactly where the lane says it does
- eslint on the dummy packet TS: config-missing (exit 0 with `eslint.config.*` notice) → eslint-recorded as MISSING per subagent.md `pass-recorded` semantic; corroborated by S7 `judge_gate(1,0)=pass-recorded`

Compilers (VERIFIED): `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `tsc Version 7.1.0-beta`.

## 3. Per-artifact verdicts (artifact × all-gates-true? → MIGRATED or BLOCKED/FOLLOW-UP)

Lane receipts below are REPORTED (read from the lane files, hashes on disk there);
the MIGRATED verdicts are INFERRED by applying the all-gates-true rule. The tools.md
hashes were additionally VERIFIED (recomputed from the lane file 2026-10-31: TS block
`6e0e4eec…f8633ef`, Rust block `dc41217b…f5f8c15` — both match the lane receipt exactly).

| artifact | gates (per lane receipt) | verdict |
|---|---|---|
| tools.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true (hashes re-verified this run) | MIGRATED |
| decomposition.md | ts_strict=true, rust_port_supported=true (unsupported []), rust_compile=true, sent=true | MIGRATED |
| swarm.md | ts_check=true, ts_to_rust=true (unsupported []), rust_compile=true, sent=true | MIGRATED |
| loops.md | ts_check=true (exit 0), ts_to_rust=true, rustc lib+bin `-D warnings` exit 0, termination proof exit 0 | MIGRATED |
| subagent.md | ts_strict=true (ts_check exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), dryrun exit 0, sent=true | MIGRATED |
| rules.md | ts_strict=true (exit 0), rust_port_supported=true ([]), rust_compile=true (exit 0), sent=true | MIGRATED |
| preregistration.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, gate_open=1, sent=false BY DESIGN (record, not send) | MIGRATED (gates all true; sent=false is the specified prereg semantic) |
| hooks.md | ts_strict=true, rust_port_supported=true ([]), rust_compile=true, dryrun exit 0, sent=true | MIGRATED |
| skills.md | ts_strict=true (interface probe), rust CI compiled logic covered by tools.md SENT receipt, sent=false BY DESIGN (interface-only, bodies live in tools.md) | MIGRATED (interface-only by Round-2 spec; no compiled gate of its own to fail) |

FOLLOW-UP (not a blocker, recorded per documentation standards — the lane's own `skills.md`
names it): `skills.md` notes `rules.md` text still points at a `skills.md` `## Rules`
section removed by the interface shrink; the rules-file owner must repoint it (to `tools.md`
checkers or the new canonical rule home). Out of this agent's write set — NOT fixed here.

## 4. Honest accounting

- Full E2E chain PASS (all 7 driver steps green, exit 0) is consistent with the per-artifact
  table: every compiled artifact the chain exercised reports all-gates-true in its lane
  receipt, and the S5/S6/S7 steps corroborate the warn-vs-deny, route, and pass-recorded
  semantics against the real harnesses (not just the compiled predicates).
- Nothing was overridden: unlike the rust-lane E2E (2 BLOCKED: stub hook, STOPped skills),
  the typescript lane ships compiled hook predicates + dryrun and an interface-only skills
  file whose compiled logic is covered by the tools.md SENT receipt — both VERIFIED above.
- `sent=false` appears twice (preregistration, skills) and both are BY DESIGN with the
  reason pasted in the table — neither is a silent non-send.
- Lane files modified by this agent: NONE. This file is the sole lane-dir CREATE.
  Scratch + driver sources live in `/tmp/n3-e2e-ts/` (outside the lane, per write set).
- NOT checked (stated per §7.5): `vitest`/`jest`/`biome` runs (not installed for this probe);
  checker coverage for them rests on the lane's `tools.md` receipt, not on this run.
