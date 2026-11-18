# typescript / loops

Date: 2026-10-31. Status: VERIFIED. Scope: `plugins/web-development/frontend/languages/typescript/`.

## Loop state machine (normative)

Enum: `Plan → Run → Judge → Receipt`. Terminal: `Receipt` only.

- `MAX_ITERS = 8` — hard bound; `run()` breaks when `iters >= MAX_ITERS`.
- Two-strike escalation — `MAX_STRIKES = 2`; a packet with `strikes >= 2` halts before stepping (`escalated` check precedes `step`).
- `iters` counts executed steps only; escalation-halted packet records `iters = 0`.

Reference implementation (TS subset, strict-checked): `LoopMachine` with `step` / `strike` / `escalated` / `run` over `LoopPacket { id, state, attempts, strikes }`. Design-loop lives in `ui-ux-flow`; component spec before code (GATE).

## Verification receipts (2026-10-31, Loop Agent 6)

- `ts_check`: `ok=true`, `exitCode=0`, zero diagnostics.
- `ts_to_rust`: `ok=true`, `unsupported=[]`.
- Mechanical fixes to generated Rust (semantics unchanged from `loop.ts`): stray `;` after `while` condition (empty-loop spin) → block body; borrowed signatures (`&LoopPacket`, `&LoopState`, `LoopState::` paths) for the borrow checker; `number` ports as `f64` per the type map.
- `rustc --deny warnings` lib + bin: `exit=0`, zero warnings (rustc 1.99.0 Homebrew).
- Termination log (`/tmp/n3-loop-test-ts/loop_test`, dummy packets `dummy-ts-001/002/003`):
  - `T1 final=Receipt attempts=3 strikes=0 iters=3`
  - `T2 final=Plan attempts=0 strikes=2 iters=0 escalated=true`
  - `T3 final=Receipt iters=3 bound_ok=true`
  - `ALL_TERMINATED iters_T1=3 iters_T2=0 iters_T3=3`, `run exit=0`

## Runbooks (canonical — linked, never duplicated)

- Delivery: `N3/runbooks/07-delivery-runbook.md`
- Research: `N3/runbooks/08-research-runbook.md`
- Pressure-test: `N3/runbooks/10-pressure-test-runbook.md`
- Release: `N3/runbooks/11-release-runbook.md`
- Plan & judges: `N3/runbooks/01-runbook-plan-and-judges.md`
- Two-strike escalation: `N3/runbooks/02-two-strike-tactical-escalation.md`

Contract on disk before fan-out (PARTITION); critic pass required (PROVE). Stop-gate veto is final within a session.

## Corrections

- 2026-10-31: replaces the 2026-10-30 PROPOSED stub (loop named but uncompiled, untested). This revision adds the strict-checked TS reference + Rust port + termination receipt; the stub's runbook sequence, design-loop GATE note, and stop-gate rule are retained, not withdrawn.
