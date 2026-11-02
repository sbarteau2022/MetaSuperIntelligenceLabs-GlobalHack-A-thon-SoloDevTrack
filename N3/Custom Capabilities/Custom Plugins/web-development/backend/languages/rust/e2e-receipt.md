# rust / e2e-receipt (E2E AGENT 1 of 2, Round 4)

Date: 2026-10-31. Status: E2E RUN COMPLETE (mixed verdict — see table).
Scope: dummy packet end-to-end through the whole rust migrated chain.
Write discipline: CREATE only — no lane file (`tools.md`, `hooks.md`, `loops.md`, `skills.md`,
`rules.md`, `subagent.md`, `swarm.md`, `decomposition.md`, `preregistration.md`) was modified.
Lane files were read first (all nine), then the packet was driven through the chain in order.

## 1. Dummy packet

`/tmp/n3-e2e-rust/packet.json` (fake route/handler contract, no real code):

    {
      "route": "POST /api/widgets",
      "handler": "create_widget",
      "contract": "runbooks/07-delivery-runbook.md#route-handler-contract",
      "checker": "clippy",
      "byte_len": 100,
      "max_bytes": 1024
    }

- sha256 (VERIFIED via `shasum -a 256`): `445fed5e215bb562e71bd115238f603b93e1ed3e3c6582bbfe5cbdfbf47ecd9a`

## 2. Chain log (VERIFIED — compiled Rust driver, exit 0)

Driver: `/tmp/n3-e2e-rust/e2e_drive.rs`, compiled `rustc --edition 2021 -D warnings`
(zero warnings) against all six lane libs re-compiled from their canonical sources
(`prereg_lib.rs` ← `rs_prereg_norm.rs`, `decomp_lib.rs`, `swarm_lib.rs`, `looplib_lib.rs`,
`subagent_lib.rs`, `toolslib_lib.rs` extracted from `tools.md`), run exit 0. Pasted output:

    [s1-prereg] version_chain_ok(1,0)=1 sign_prereg(1,1)=1 gate_open=1 -> PASS
    [s2-decomp] prove_chain(route->check->judge->receipt)=true -> PASS
    [s3-swarm] declare order_ok=true run route=cargo_clippy budget_over=false judge=true escalate=false sent=true -> PASS
    [s4-loop] packet_ready=true drive verdicts=[pass,*,*] final=Done iters=1 strikes=0 terminated=true -> PASS
    [s5-hook] rs-unwrap deny absent=true fmt=true clippy=true test=true (STUB: no compiled harness, evaluated by inspection) -> PASS-BY-INSPECTION (stub, sent=false)
    [s6-tool] route=cargo_clippy over_budget=false gate_send=true receipt.sent=true -> PASS
    [s7-subagent] role=rust-checker inject_lane=true inject_other=false allow_route=true allow_unlisted=false run=cargo_clippy judge=true escalate=false leak_free=true -> PASS

Step mapping: prereg sign (S1) → decomposition typecheck (S2) → swarm DAG declare+run (S3) →
loop drive plan→run→judge→receipt (S4) → hook gate (S5) → tool route (S6) →
subagent execute/allowlist check (S7). Each step pasted pass/fail above.

Compilers (VERIFIED): `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `tsc Version 7.1.0-beta`.

## 3. Per-artifact verdicts (artifact × all-gates-true? → MIGRATED or BLOCKED)

Lane receipts below are REPORTED (read from the lane files, hashes on disk there);
the MIGRATED/BLOCKED verdicts are INFERRED by applying the all-gates-true rule.

| artifact | gates (per lane receipt) | verdict |
|---|---|---|
| tools.md | ts_strict=true, rust_port_supported=true, rust_compile=true, sent=true | MIGRATED |
| decomposition.md | ts_strict=true, rust_port_supported=true (unsupported empty), rust_compile=true, sent=true | MIGRATED |
| swarm.md | ts_check=true, ts_to_rust=true (unsupported []), rust_compile=true, sent=true | MIGRATED |
| loops.md | ts_check=true, ts_to_rust=true (1 unsupported item, fixed), rustc exit 0, termination proof exit 0 | MIGRATED |
| subagent.md | tsc strict exit 0, rustc lib+drill exit 0, drill exit 0, leak_free=true, sent=true | MIGRATED |
| rules.md | ts_strict=true, rust_port_supported=true, rust_compile=true, sent=true (local toolchain; MCP wrappers unrunnable, recorded) | MIGRATED (local-toolchain gates; MCP harness gap recorded in lane file) |
| preregistration.md | ts_strict=true, rust_port_supported=true, rust_compile=true, gate_open=1, sent=false BY DESIGN (record, not send) | MIGRATED (gates all true; sent=false is the specified prereg semantic) |
| hooks.md | UNCHANGED stub, sent=false, no compiled harness | BLOCKED (blocker: no compiled hook gate; stub only, sent=false) |
| skills.md | ts_strict=false (STOP), rust_port_supported NOT RUN, rust_compile NOT RUN, sent=false | BLOCKED (blocker: ts_check ok:false infra failure; chain STOP per tool-write Step 3) |

## 4. Honest accounting

- `rust/hooks.md` (UNCHANGED stub, sent=false) reads BLOCKED — no compiled harness exists,
  so S5 passed by inspection only and cannot count as a migrated gate. Blocker named above.
- `rust/skills.md` (STOP) reads BLOCKED — `ts_check` returned `ok:false` with empty
  diagnostics on trivially valid source, so the chain stopped per tool-write Step 3 and the
  port/compile gates never ran. Blocker named above.
- Full E2E chain PASS (all 7 driver steps green, exit 0) does not override per-artifact
  BLOCKED verdicts: the chain exercised the compiled artifacts where they exist and exposes
  exactly the two lane gaps above.
- Lane files modified by this agent: NONE. This file is the sole lane-dir CREATE.
  Scratch + driver sources live in `/tmp/n3-e2e-rust/` (outside the lane, per write set).
