# html / loops

Date: 2026-10-31. Status: VERIFIED. Scope: `plugins/web-development/frontend/languages/html/`.

## Loop state machine (normative)

Enum: `Plan → Run → Judge → Receipt`. Terminal: `Receipt` only.

- `MAX_ITERS = 8` — hard bound; `run()` breaks when `iters >= MAX_ITERS`.
- Two-strike escalation — `MAX_STRIKES = 2`; a packet with `strikes >= 2` halts before stepping (`escalated` check precedes `step`).
- `iters` counts executed steps only; escalation-halted packet records `iters = 0`.

Reference implementation (ESM, JSDoc-typed, checkJs-strict): `LoopMachine` with
`step` / `strike` / `escalated` / `run` over `LoopPacket { id, state, attempts, strikes }`.
States are string constants (`LoopState.Plan/Run/Judge/Receipt`); no TS-only syntax in the
shipped module, so the same logic runs under plain Node. Design-loop lives in `ui-ux-flow`;
component spec before code (GATE).

## Verification receipts (2026-10-31, Loop Agent 6)

- `ts_check` (TS twin of the reference, strict): `ok=true`, `exitCode=0`, zero diagnostics.
- `ts_to_rust` (TS twin): `ok=true`, `unsupported=[]`.
- `tsc --allowJs --checkJs --noEmit --strict` over `loop.mjs`: exit 0, no output.
- `node --test` over `loop_test.mjs`: pass 1, fail 0, exit 0.
- `prettier --check` over `loop.mjs` + `loop_test.mjs`: all files use Prettier code style, exit 0.
- `eslint` over the probe dir: config-missing (`eslint.config.*` not found, exit 2) → recorded
  as MISSING per `hooks.md` stop-gate contract (checkers pasted or MISSING recorded — never
  a silent pass). Same standing as the typescript and javascript lanes' eslint notices.
- `rustc --deny warnings` lib + bin (Rust port of the same state machine): `exit=0`,
  zero warnings (rustc 1.99.0 Homebrew).
- Mechanical fixes to generated Rust (semantics unchanged from the TS twin, identical to the
  typescript lane's `loop_machine.rs` modulo packet ids — VERIFIED via `diff` clean):
  stray `;` after `while` condition (empty-loop spin) → block body; borrowed signatures
  (`&LoopPacket`, `&LoopState`, `LoopState::` paths) for the borrow checker; `number` ports
  as `f64` per the type map.
- Lane gate — `html-validate 11.17.0` over a probe `probe.html` (HTML5 doctype, `lang`,
  `meta charset`, single `main` landmark): exit 0, no findings. Recorded because this lane's
  `hooks.md` names `html-validate` as gate and lists it NEEDS-VERIFY on this machine.
- Termination log (`/tmp/n3-loop-test-html/loop_test`, dummy packets `dummy-html-001/002/003`):
  - `T1 final=Receipt attempts=3 strikes=0 iters=3`
  - `T2 final=Plan attempts=0 strikes=2 iters=0 escalated=true`
  - `T3 final=Receipt iters=3 bound_ok=true`
  - `ALL_TERMINATED iters_T1=3 iters_T2=0 iters_T3=3`, `run exit=0`
- Artifact hashes (sha256, VERIFIED via `shasum -a 256`):
  - `packet.json` (`dummy-html-001`, Plan, 0/0): `7a1c4bef…f6394d56`
  - `loop.mjs`: `b3952d81…38f73e81`
  - `loop_test.mjs`: `1bef2a59…012c6916`

Compilers (VERIFIED): `node v26.9.0`, `tsc Version 7.1.0-beta`, `rustc 1.99.0`, `prettier 3.9.10`, `html-validate 11.17.0`.

## Runbooks (canonical — linked, never duplicated)

- Delivery: `N3/runbooks/07-delivery-runbook.md`
- Research: `N3/runbooks/08-research-runbook.md`
- Pressure-test: `N3/runbooks/10-pressure-test-runbook.md`
- Release: `N3/runbooks/11-release-runbook.md`
- Plan & judges: `N3/runbooks/01-runbook-plan-and-judges.md`
- Two-strike escalation: `N3/runbooks/02-two-strike-tactical-escalation.md`

Contract on disk before fan-out (PARTITION); critic pass required (PROVE). Stop-gate veto is final within a session.

## Corrections

- 2026-10-31: replaces the 2026-10-30 PROPOSED stub (loop named but uncompiled, untested). This revision adds the checkJs-strict ESM reference + `node --test` + prettier + Rust port + html-validate gate + termination receipt; the stub's runbook sequence, design-loop GATE note, and stop-gate rule are retained, not withdrawn.
