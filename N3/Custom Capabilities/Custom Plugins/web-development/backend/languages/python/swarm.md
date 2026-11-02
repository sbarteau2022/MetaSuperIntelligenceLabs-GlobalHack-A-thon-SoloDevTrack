# python / swarm

Date: 2026-10-31 (SWARM AGENT 2 of 6, N3 pilot Round 3). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/python/`.

Swarm DAG for the python lane: five compiled nodes, four typed edges, one judge plus one receipt per node, typecheck-before-run. This file is the DAG index (RETRIEVE: one canonical source per decision). Node logic lives in each node's own file — referenced here, never duplicated.

Honest marking (per brief): `tools.md` compiled with `sent=false` (tool-write gate BLOCKED — Rust port unsupported construct, no `rustc` attempted). Every swarm node that depends on the route node is therefore a PROPOSED node with judge pending. Nothing below claims `sent`.

## DAG nodes (compiled nodes)

| Node | File | Status | Judge |
|---|---|---|---|
| `route` | tools.md | PROPOSED (`sent=false`) | route-first contract, 7 checkers, bounded output |
| `gate` | hooks.md | PROPOSED | `ruff check` + `ruff format --check` (+ `mypy --strict` recorded); E |
| `loop` | loops.md | VERIFIED | `LoopMachine` termination log, `rustc` exit 0 |
| `rule` | rules.md | PROPOSED | 21/21 rows kept, 0 cut, contradiction table |
| `worker` | subagent.md | PROPOSED | lifecycle + allowlist + two-strike freeze, `rustc` exit 0 |

## Typed edges (typecheck-before-run)

Linear chain, closed vocabulary — `edge_allowed` returns 1 only for these four pairs, 0 otherwise:

- `route → gate`
- `gate → loop`
- `loop → rule`
- `rule → worker`

No edge fires unless: edge is allowed AND downstream TS is strict-clean AND upstream verdict is not `fail` (`can_fire`: `edge_ok=1 ∧ ts_strict=1 ∧ upstream≠fail`). A `fail` upstream halts the chain before stepping. Bounds inherited from loops.md: `MAX_ITERS = 8`, `MAX_STRIKES = 2`.

## Compiled DAG (tool-write chain, SWARM AGENT 2 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`, no object literals) — node→file projection, node status, edge allowlist, per-node judge (`fail` / `pass-recorded` / `pass`), fire gate, inherited bounds:

```ts
export function nodeFile(node: string): string {
  if (node === "route") {
    return "tools.md";
  }
  if (node === "gate") {
    return "hooks.md";
  }
  if (node === "loop") {
    return "loops.md";
  }
  if (node === "rule") {
    return "rules.md";
  }
  if (node === "worker") {
    return "subagent.md";
  }
  return "unknown";
}
export function nodeStatus(node: string): string {
  if (node === "loop") {
    return "VERIFIED";
  } else {
    return "PROPOSED";
  }
}
export function edgeAllowed(from: string, to: string): number {
  if (from === "route") {
    if (to === "gate") {
      return 1;
    }
  }
  if (from === "gate") {
    if (to === "loop") {
      return 1;
    }
  }
  if (from === "loop") {
    if (to === "rule") {
      return 1;
    }
  }
  if (from === "rule") {
    if (to === "worker") {
      return 1;
    }
  }
  return 0;
}
export function judgeNode(checkPass: number, recorded: number): string {
  if (checkPass === 0) {
    return "fail";
  } else {
    if (recorded === 0) {
      return "pass-recorded";
    } else {
      return "pass";
    }
  }
}
export function canFire(edgeOk: number, tsStrict: number, upstream: string): number {
  if (edgeOk === 0) {
    return 0;
  } else {
    if (tsStrict === 0) {
      return 0;
    } else {
      if (upstream === "fail") {
        return 0;
      } else {
        return 1;
      }
    }
  }
}
export function maxIters(): number {
  return 8;
}
export function maxStrikes(): number {
  return 2;
}
```

Rust port: `ts_to_rust` output normalized (camelCase→snake_case, transpiler parens de-nested; semantics unchanged) and held at `/tmp/n3-swarm-py/py_swarm.rs` during the run. No object literals in TS source or Rust port.

## Checkers (pasted, not summarized)

- `ts_check`: `ok=true`, compiler `/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc`, `diagnostics=""`, `exitCode=0`
- `ts_to_rust`: `ok=true`, inner check `ok=true exitCode=0`, `unsupported=[]`
- `rustc --crate-type lib /tmp/n3-swarm-py/py_swarm.rs -D warnings`: `exit=0`, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`

## Dummy 2-node typecheck run (pasted log)

Driver `/tmp/n3-swarm-py/run/driver.rs` includes the compiled DAG and asserts: legal edge `route→gate` fires, illegal skip-edge `gate→worker` is refused (`edge=0 → fire=0`).

```
N1 route->gate edge=1 judge=pass-recorded fire=1 file=tools.md
N2 gate->worker edge=0 judge=pass-recorded fire=0 file=hooks.md
STATUS loop=VERIFIED worker=PROPOSED
BOUNDS iters=8 strikes=2
DUMMY_2NODE ok=true exit=0
run exit=0
```

Verdict reading: N1 fires (typed edge + strict-clean + upstream not fail); N2 correctly refused (no such typed edge — the DAG forces `gate→loop→rule→worker`, no shortcuts). `judge=pass-recorded` on both reflects the `tools.md` caveat: checkers pass but `sent=false` upstream, so downstream verdicts stay recorded, never clean `pass`.

## Receipt (tool-write, one per DAG)

- dag: `python/swarm` (SWARM AGENT 2 of 6, N3 pilot Round 3)
- source hash (sha256 of TS source `py_swarm.ts`): `192d2ad5125cc2a601a34432466512184f85b2269022eb47632959f7a185f84b`
- Rust output hash (sha256 of normalized `py_swarm.rs` as compiled): `3cf431d27dd3755ee9ab42ebd76984dac052974c6f6319272427bc29aacb11f8`
- `ts_strict`: true — diagnostics: none (exit 0)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib -D warnings`, exit 0, no output
- `sent`: false — gate requires all three true AND all five nodes VERIFIED; `route` node depends on `tools.md` (`sent=false`, judge pending). DAG engine itself compiles clean; the DAG as a dispatchable swarm stays PROPOSED until the route node clears its gate.
- why-receipt on the blocked node: `tools.md` TS source uses an unsupported construct (object literals); `ts_to_rust` refused the port, so per the STOP rule no `rustc` was attempted and `sent=false`. Recorded in tools.md receipt; not re-litigated here.

## References (canonical — linked, never duplicated)

- Route contract / checkers / blocked-node receipt: tools.md
- Hook gate, deny/warn rows, event envelopes: hooks.md
- Loop bounds, termination log, runbooks: loops.md
- Rule predicates, contradiction table: rules.md
- Worker lifecycle, allowlist, escalation: subagent.md
- Phase-0 bars (fixed, pre-registered): `N3/runbooks/swarm-phase0-prereg.md`
