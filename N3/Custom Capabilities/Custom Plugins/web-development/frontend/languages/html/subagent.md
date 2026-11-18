# html / subagent

Date: 2026-10-31 (SUBAGENT AGENT 6 of 6, N3 pilot Batch 4 Round 2). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/html/`.

Role: html-lane worker subagent. Spawned for one html task with a typed lifecycle; disposed after the run. No ambient authority — every capability comes from the injected scope below.

Lifecycle (normative): `created → injected → running → disposed`. Terminal: `disposed` only. A spawned agent starts `created`; it may act only after `inject` (`created → injected`) and `start` (`injected → running`); after `dispose` every handle is dead (is_disposed = 1, tool calls refused). Stale-handle use after dispose is a STOP.

Scope: html lane only — files under `plugins/web-development/frontend/languages/html/` plus the task dir named in the spawn packet. No writes outside scope (contract on disk before fan-out; cf. loops.md PARTITION).

Tool allowlist (explicit, least-privilege; everything else denied):
- `route` — route-first dispatch (TOOLS-2026.md contract; see tools.md)
- `read` — read-only inspection
- `write-with-receipt` — writes with receipt per run
- `test-with-judge` — judge runs (html-validate + eslint-recorded below)

No `execute`-arbitrary, no network tools, no direct `fs` writes outside the write-with-receipt path. Escalation never widens this list — strikes narrow it to zero.

Judges (recorded, never silent-pass; see hooks.md for gate + missing-judge policy):
- `html-validate` (PRESENT v11.17.0, VERIFIED 2026-10-31 per Hook-4/Loop-6) — hard gate
- `eslint` — recorded; when eslint is MISSING the verdict is `pass-recorded` (missing noted in receipt), never a clean `pass`
- post-run the receipt pastes checker output or records MISSING

Escalation (two-strike; see `N3/runbooks/02-two-strike-tactical-escalation.md`, loops.md):
- strike 1: re-plan within contract, one retry
- strike 2 (`strikes >= 2`): halt before stepping — `can_use_tool` returns 0 for every tool, `lifecycle_step` freezes state, parent owns the packet
- `MAX_STRIKES = 2`, `MAX_ITERS = 8` (hard bound; cf. loops.md)

Disposers / no-leak: `dispose_agent` maps any state to `disposed`; post-dispose, `is_disposed = 1` and all tool gates return deny. The compiled lifecycle below performs no I/O and holds no handles (pure state predicates over strings/numbers) — leak_grep clean (no `fs::`/`File::`/`write`/`Command`/`process::`), net_grep clean (no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::`), so a disposed agent retains nothing to leak.

## Compiled lifecycle (tool-write chain, SUBAGENT AGENT 6 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models create→inject→run→dispose transitions, tool allowlist (4 tools), judge gate (html-validate + eslint-recorded), two-strike escalation freeze, and MAX_ITERS/MAX_STRIKES bounds:

```ts
export function injectAgent(state: string): string {
  if (state === "created") {
    return "injected";
  } else {
    return state;
  }
}
export function startAgent(state: string): string {
  if (state === "injected") {
    return "running";
  } else {
    return state;
  }
}
export function disposeAgent(state: string): string {
  if (state === "disposed") {
    return "disposed";
  } else {
    return "disposed";
  }
}
export function addStrike(strikes: number): number {
  return strikes + 1;
}
export function maxStrikes(): number {
  return 2;
}
export function maxIters(): number {
  return 8;
}
export function isEscalated(strikes: number): number {
  if (strikes >= 2) {
    return 1;
  } else {
    return 0;
  }
}
export function isDisposed(state: string): number {
  if (state === "disposed") {
    return 1;
  } else {
    return 0;
  }
}
export function isToolAllowed(tool: string): number {
  if (tool === "route") {
    return 1;
  }
  if (tool === "read") {
    return 1;
  }
  if (tool === "write-with-receipt") {
    return 1;
  }
  if (tool === "test-with-judge") {
    return 1;
  }
  return 0;
}
export function canUseTool(tool: string, strikes: number): number {
  if (strikes >= 2) {
    return 0;
  } else {
    return isToolAllowed(tool);
  }
}
export function judgeGate(htmlValidatePass: number, eslintRecorded: number): string {
  if (htmlValidatePass === 0) {
    return "fail";
  } else {
    if (eslintRecorded === 0) {
      return "pass-recorded";
    } else {
      return "pass";
    }
  }
}
export function lifecycleStep(state: string, strikes: number): string {
  if (strikes >= 2) {
    return state;
  } else {
    if (state === "created") {
      return "injected";
    } else {
      if (state === "injected") {
        return "running";
      } else {
        return state;
      }
    }
  }
}
```

Rust port (`ts_to_rust`, unsupported: none): `html_subagent.rs` — 12 fns. Parens de-nested + camelCase hand-normalized to snake_case (cf. typescript hooks.md conversion precedent); semantics unchanged from transpiler output.

```rust
pub fn inject_agent(state: String) -> String {
  if state == "created".to_string() {
    return "injected".to_string();
  } else {
    return state;
  }
}
pub fn start_agent(state: String) -> String {
  if state == "injected".to_string() {
    return "running".to_string();
  } else {
    return state;
  }
}
pub fn dispose_agent(state: String) -> String {
  if state == "disposed".to_string() {
    return "disposed".to_string();
  } else {
    return "disposed".to_string();
  }
}
pub fn add_strike(strikes: f64) -> f64 {
  return strikes + 1f64;
}
pub fn max_strikes() -> f64 {
  return 2f64;
}
pub fn max_iters() -> f64 {
  return 8f64;
}
pub fn is_escalated(strikes: f64) -> f64 {
  if strikes >= 2f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn is_disposed(state: String) -> f64 {
  if state == "disposed".to_string() {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn is_tool_allowed(tool: String) -> f64 {
  if tool == "route".to_string() {
    return 1f64;
  }
  if tool == "read".to_string() {
    return 1f64;
  }
  if tool == "write-with-receipt".to_string() {
    return 1f64;
  }
  if tool == "test-with-judge".to_string() {
    return 1f64;
  }
  return 0f64;
}
pub fn can_use_tool(tool: String, strikes: f64) -> f64 {
  if strikes >= 2f64 {
    return 0f64;
  } else {
    return is_tool_allowed(tool);
  }
}
pub fn judge_gate(html_validate_pass: f64, eslint_recorded: f64) -> String {
  if html_validate_pass == 0f64 {
    return "fail".to_string();
  } else {
    if eslint_recorded == 0f64 {
      return "pass-recorded".to_string();
    } else {
      return "pass".to_string();
    }
  }
}
pub fn lifecycle_step(state: String, strikes: f64) -> String {
  if strikes >= 2f64 {
    return state;
  } else {
    if state == "created".to_string() {
      return "injected".to_string();
    } else {
      if state == "injected".to_string() {
        return "running".to_string();
      } else {
        return state;
      }
    }
  }
}
```

## Receipt (tool-write, one per subagent)

- subagent: `html/subagent` (SUBAGENT AGENT 6 of 6, N3 pilot Batch 4 Round 2)
- source hash (sha256 of TS source): `024854d1dd6bc25fa727e25c1b1cffba9184b60f5d85ec89730a99dabdb3a62d`
- Rust output hash (sha256 of normalized `html_subagent.rs` as compiled): `61b261e3f0a6e7b6422554b17ea184fbb6721fed86a6e70afb97f70e3913ae00`
- `ts_strict`: true — diagnostics: none (exit 0, tsrs TS 7.1.0-beta via ts_check)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib html_subagent.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-subagent-html/dryrun`, asserts over `html_subagent.rs`): inject `created→injected`, start `injected→running`, dispose `any→disposed`, `is_escalated(2)=1`, `is_disposed(disposed)=1`, allowlist `route=1` / `read=1` / `write-with-receipt=1` / `test-with-judge=1` / `execute=0`, `can_use_tool(read, strikes=2)=0` (freeze), `judge_gate(html-validate=1, eslint-missing=0)=pass-recorded`, `judge_gate(html-validate=0)=fail`, `judge_gate(html-validate=1, eslint=1)=pass`, `lifecycle_step` freeze at strikes=2 → `ALL_TERMINATED ok`, exit 0
- lane gate: `html-validate 11.17.0` over probe `good.html` (HTML5 doctype, `lang`, `meta charset`, single `main` landmark) → exit 0, no findings (VERIFIED 2026-10-31; matches hooks.md Hook-4 + loops.md Loop-6 gate)
- no-out-of-root-writes proof: no `fs::`/`File::`/`std::fs`/`write!`/`Command`/`process::` in TS source or Rust port (grep exit 1 = no matches; the lone `write-with-receipt` substring in the port is the allowlisted tool NAME, not a call). Pure state predicates, no I/O imports — nothing retained past dispose by construction.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in TS source or Rust port (grep exit 1 = no matches).
- callable schemas: none listed, none empty. This subagent exposes no callable tool schemas — its surface is the 4 lifecycle transitions plus the 4-tool allowlist above. Tool contracts live in tools.md, hook events in hooks.md, loop bounds/runbooks in loops.md — referenced, not duplicated.

## References (canonical — linked, never duplicated)

- Tools / route-first contract / checkers: tools.md
- Hook gate (`html-validate` + `audit.py` a11y D-rules), deny/warn rules, event envelopes: hooks.md
- Loop bounds (`MAX_ITERS = 8`, two-strike), runbooks: loops.md
