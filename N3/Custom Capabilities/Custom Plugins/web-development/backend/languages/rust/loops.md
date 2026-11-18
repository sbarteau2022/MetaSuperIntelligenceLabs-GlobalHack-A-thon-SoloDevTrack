# rust / loops

Date: 2026-10-31. Status: VERIFIED (compiled + termination-tested). Scope: `plugins/web-development/backend/languages/rust/`.

## 1. Interface

The loop is a bounded state machine over exactly these states:

```rust
pub const MAX_ITERS: u32 = 3;    // hard bound on judge rounds
pub const MAX_STRIKES: u32 = 2;  // two-strike escalation

pub enum State { Plan, Run, Judge, Receipt, Escalate, Done }
```

- `Plan` — route/handler/service/validator contract on disk before fan-out (PARTITION).
- `Run` — delivery work per runbook [07](../.../../../../runbooks/07-delivery-runbook.md) (link only; bodies never duplicated here).
- `Judge` — critic pass required (PROVE); each fail records one strike.
- `Receipt` — write the tool receipt (tool-write chain: `ts_check`, `rustc`, termination log).
- `Escalate` — terminal: 2 judge fails (`strikes >= MAX_STRIKES`) or `iters >= MAX_ITERS`.
- `Done` — terminal: judge passed AND receipt written and green.
- Stop-gate veto is final within a session: pending checker/contract issues are fixed or each recorded with a dated reason before `Done` is claimed.

Sequence (links only, never runbook bodies): delivery [07](../../../../../runbooks/07-delivery-runbook.md) → research [08](../../../../../runbooks/08-research-runbook.md) → pressure-test [10](../../../../../runbooks/10-pressure-test-runbook.md) → release [11](../../../../../runbooks/11-release-runbook.md).

## 2. Compiled state machine (Rust, `rustc --deny warnings` exit 0)

Authored in the TS strict subset (`ts_check`: `ok:true`, exit 0, zero diagnostics), ported via `ts_to_rust` (`ok:true`, 1 unsupported item: object literal; raw output also emitted `State.Plan` instead of `State::Plan` — fixed mechanically, counters narrowed `f64`→`u32` with bounds unchanged). Full sources: `/tmp/n3-loop-test/loop.ts`, `/tmp/n3-loop-test/loop_lib.rs`.

```rust
pub fn packet_ready(p: &Packet) -> bool {
    !(p.route.is_empty() || p.handler.is_empty() || p.contract.is_empty())
}

impl LoopMachine {
    pub fn new() -> Self;                    // state = Plan, iters = 0, strikes = 0
    pub fn step(&mut self, judge_pass: bool) -> State;
    //   pass      -> Receipt
    //   fail      -> strikes += 1;
    //                strikes >= 2  -> Escalate
    //                iters += 1; iters >= MAX_ITERS -> Escalate
    //                else -> Run
    pub fn finish(&mut self, receipt_ok: bool) -> State;
    //   Receipt + ok -> Done | Receipt + !ok -> Escalate | else unchanged
}

/// At most MAX_ITERS judge rounds; every path returns Done or Escalate.
pub fn drive_loop(verdicts: [bool; 3]) -> LoopResult;
```

Gates (per tool-write chain; any `false` = STOP + why-receipt):

| Gate | Result (2026-10-31) |
|---|---|
| `ts_check` strict | `ok:true`, exit 0, zero diagnostics |
| `ts_to_rust` port | `ok:true`, unsupported: `[object literal]` (+ `::` path fix applied) |
| `rustc --deny warnings` lib + bin | exit 0, zero warnings |

## 3. Test receipt (termination, dummy packet)

Packet `/tmp/n3-loop-test/packet.json` (fake route/handler contract). Harness `/tmp/n3-loop-test/run_loop.rs` (compiled Rust, `--deny warnings` exit 0). Log:

```text
[plan] packet ready: POST /api/widgets -> create_widget
[run] scenario A verdicts=[pass,*,*] -> Done iters=1 strikes=0 terminated=true
[judge] scenario B verdicts=[fail,fail,*] -> Escalate iters=2 strikes=2 terminated=true
[plan] machine start: Plan
[judge#1] fail -> Run iters=1 strikes=1
[judge#2] pass -> Receipt iters=1 strikes=1
[receipt] ok -> Done (total judge rounds=2)
[bound] scenario A iterations=1 <= MAX_ITERS=3 OK
[bound] scenario B iterations=2 <= MAX_ITERS=3 OK
TERMINATION-OK: all paths terminated in Done or Escalate within bound 3
```

VERIFIED (measured 2026-10-31, `run_loop` exit 0): all paths terminate within the bound — max observed iterations 2 ≤ `MAX_ITERS` 3; terminals are only `Done` or `Escalate`. Two-strike escalation fires on consecutive judge fails (scenario B: 2 strikes → `Escalate`).

## 4. Corrections

- 2026-10-31: replaces the 2026-10-30 PROPOSED stub (loop named but uncompiled, untested). This revision adds the compiled state machine + termination receipt; the stub's runbook sequence and stop-gate rule are retained, not withdrawn.
