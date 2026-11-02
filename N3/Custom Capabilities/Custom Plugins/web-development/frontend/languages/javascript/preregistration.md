# javascript / preregistration

Date: 2026-10-31 (PREREG AGENT 6 of 6, Batch 3 Round 3). Status: PROPOSED.
Scope: `plugins/web-development/frontend/languages/javascript/preregistration.md` (CREATE canonical; sole file in this agent's write set).

- Version: `1`, supersedes: `0`. Chain check: `version_chain_ok(1, 0) = 1` (VERIFIED — computed from the compiled predicate below).
- Signer: PREREG AGENT 6 of 6, Batch 3 Round 3. Timestamp: `2026-10-31T05:50:33Z` (UTC, at chain completion; file written after).
- Normative parent: `N3/runbooks/swarm-phase0-prereg.md` (phase-0 prereg pattern: fixed bars before the run, never moved after results).
- Lane sources read first: `skills.md`, `tools.md`, `rules.md`, `hooks.md`, `loops.md`, `subagent.md` (all in this directory).
- Authoring law: `tool-write` skill chain (TS subset → `ts_check` → `ts_to_rust` → `rustc` → gate). No object literals in TS source (subset forbids struct-literal constructors — cf. `python/tools` why-receipt, which STOPped on exactly this). STOP on false at every gate.

## Prereg struct

### Task set (6 tasks, fixed before the run)

| # | Task | Done predicate |
|---|---|---|
| T1 | Write TS 7.1-subset source for this prereg (version/chain, scope, judges, bars, gate fns); zero object literals | `grep -c "= {"` = 0 and `grep -c "return {"` = 0 |
| T2 | `ts_check` strict pass | `judge_pass(exit, pasted) = 1` with `exit = 0`, pasted output below |
| T3 | `ts_to_rust` port, unsupported list empty | `unsupported = []`, pasted output below |
| T4 | `rustc` compile of ported Rust | exit `0`; `-D warnings` exit `0` on the normalized artifact (paren-only de-nest, semantics unchanged — same precedent as lane `hooks.md` and pilot `python/preregistration.md` and `typescript/preregistration.md`) |
| T5 | Mutation proof: single-token mutant → recompute sha256 → hashes differ | both hashes pasted, `diff` shows the one line |
| T6 | CREATE this file with struct + hashes + version/supersedes + signer/timestamp + mutation proof + TS chain, no object literals | file exists at the exact write-set path |

### DAG (edges must satisfy `dag_edge_ok(before, after) = 1`, i.e. both in scope 1–6 and `before < after`; linear chain, acyclic by construction)

`T1 → T2 → T3 → T4 → T5 → T6`. No edge skipped, no edge reversed, no edge added. (VERIFIED — the run log follows exactly this order; no out-of-order events.)

### Judges (6, each `judge_pass(exit_code, pasted)` requires exit 0 AND pasted output)

| # | Judge | Command / check | Result |
|---|---|---|---|
| J1 | TS strict | `tsc --noEmit --strict` (system tsc 7.1.0-beta) + `ts_check` wrapper | exit 0, zero diagnostics, both pasted below |
| J2 | No object literals | `grep -c "= {"`, `grep -c "return {"` over TS source | 0 / 0 |
| J3 | Port supported | `ts_to_rust`, `unsupported` field | `[]` |
| J4 | Rust compiles | `rustc --crate-type lib` raw exit 0; norm `-D warnings` exit 0 | 0 / 0 |
| J5 | Hash integrity | `shasum -a 256` recompute over the exact sources | matches below |
| J6 | Write-set discipline | file created at the exact path, no other path written | this file only |

### Criteria / bars (5, each evaluated by `bar_met(passed, total, bar_num, bar_den)`; miss = reported as missed, bars never moved)

| # | Criterion | Bar | Result |
|---|---|---|---|
| C1 | Version chain well-formed | `version_chain_ok(1, 0) = 1` | 1 (VERIFIED) |
| C2 | STOP-on-false honored | every false stops the chain; re-run from the failing step; no silent pass | honored once (see Corrections) |
| C3 | No object literals anywhere in the chain source | J2 count 0 | 0 (VERIFIED) |
| C4 | Mutation changes the hash | mutant sha256 ≠ canonical sha256, one-line diff | differ (VERIFIED) |
| C5 | Full TS chain pasted in-file | TS source + Rust port + all checker outputs present below | present |

## Compiled prereg (tool-write chain, PREREG AGENT 6 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`, zero object literals) — version/supersedes constants, scope/size oracles, `version_chain_ok`, `task_in_scope`, `dag_edge_ok`, `judge_pass`, `bar_met` (cross-multiplied proportion: `passed * bar_den >= bar_num * total`), `gate_open` (send gate: all three true):

```ts
export function prereg_version(): number {
  return 1;
}
export function supersedes_version(): number {
  return 0;
}
export function task_count(): number {
  return 6;
}
export function judge_count(): number {
  return 6;
}
export function criterion_count(): number {
  return 5;
}
export function version_chain_ok(version: number, supersedes: number): number {
  if (version === 1) {
    if (supersedes === 0) {
      return 1;
    } else {
      return 0;
    }
  } else {
    if (version > 1) {
      if (supersedes === version - 1) {
        return 1;
      } else {
        return 0;
      }
    } else {
      return 0;
    }
  }
}
export function task_in_scope(task: number): number {
  if (task >= 1) {
    if (task <= 6) {
      return 1;
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
export function dag_edge_ok(before: number, after: number): number {
  if (task_in_scope(before) === 0) {
    return 0;
  } else {
    if (task_in_scope(after) === 0) {
      return 0;
    } else {
      if (before < after) {
        return 1;
      } else {
        return 0;
      }
    }
  }
}
export function judge_pass(exit_code: number, pasted: number): number {
  if (exit_code === 0) {
    if (pasted === 1) {
      return 1;
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
export function bar_met(passed: number, total: number, bar_num: number, bar_den: number): number {
  if (total === 0) {
    return 0;
  } else {
    if (passed * bar_den >= bar_num * total) {
      return 1;
    } else {
      return 0;
    }
  }
}
export function gate_open(ts_strict: number, rust_supported: number, rust_compile: number): number {
  if (ts_strict === 1) {
    if (rust_supported === 1) {
      if (rust_compile === 1) {
        return 1;
      } else {
        return 0;
      }
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
```

Rust port (`ts_to_rust`, `ok:true`, `unsupported:[]`): `js_prereg.rs` (raw transpiler output, sha256 `a2351bb86d016a89f2f5625fbddf8519116bcfd6e326aa96aacfc16ef11a74ba`), normalized to `js_prereg_norm.rs` (paren-only de-nest, 16 lines, semantics unchanged; sha256 `448924b936642ce2c71fa51d69b44b92b9fc0048cde99c00f7fcaa4545d84cb4`) — 11 fns (`prereg_version`, `supersedes_version`, `task_count`, `judge_count`, `criterion_count`, `version_chain_ok`, `task_in_scope`, `dag_edge_ok`, `judge_pass`, `bar_met`, `gate_open`). Full normalized source:

```rust
pub fn prereg_version() -> f64 {
  return 1f64;
}
pub fn supersedes_version() -> f64 {
  return 0f64;
}
pub fn task_count() -> f64 {
  return 6f64;
}
pub fn judge_count() -> f64 {
  return 6f64;
}
pub fn criterion_count() -> f64 {
  return 5f64;
}
pub fn version_chain_ok(version: f64, supersedes: f64) -> f64 {
  if version == 1f64 {
    if supersedes == 0f64 {
      return 1f64;
    } else {
      return 0f64;
    }
  } else {
    if version > 1f64 {
      if supersedes == (version - 1f64) {
        return 1f64;
      } else {
        return 0f64;
      }
    } else {
      return 0f64;
    }
  }
}
pub fn task_in_scope(task: f64) -> f64 {
  if task >= 1f64 {
    if task <= 6f64 {
      return 1f64;
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
pub fn dag_edge_ok(before: f64, after: f64) -> f64 {
  if task_in_scope(before) == 0f64 {
    return 0f64;
  } else {
    if task_in_scope(after) == 0f64 {
      return 0f64;
    } else {
      if before < after {
        return 1f64;
      } else {
        return 0f64;
      }
    }
  }
}
pub fn judge_pass(exit_code: f64, pasted: f64) -> f64 {
  if exit_code == 0f64 {
    if pasted == 1f64 {
      return 1f64;
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
pub fn bar_met(passed: f64, total: f64, bar_num: f64, bar_den: f64) -> f64 {
  if total == 0f64 {
    return 0f64;
  } else {
    if (passed * bar_den) >= (bar_num * total) {
      return 1f64;
    } else {
      return 0f64;
    }
  }
}
pub fn gate_open(ts_strict: f64, rust_supported: f64, rust_compile: f64) -> f64 {
  if ts_strict == 1f64 {
    if rust_supported == 1f64 {
      if rust_compile == 1f64 {
        return 1f64;
      } else {
        return 0f64;
      }
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
```

## Pasted checkers (evidence, not summaries)

- J1a system tsc: `tsc --noEmit --strict /tmp/n3-prereg-js/js_prereg.ts` → `TSC_EXIT=0`, `Version 7.1.0-beta`, no output. (VERIFIED)
- J1b wrapper: `ts_check` → `{"ok": true, "diagnostics": "", "exitCode": 0}`. (VERIFIED — pasted verbatim modulo the compiler path)
- J2: `grep -c "= {"` → `0`; `grep -c "return {"` → `0`. (VERIFIED)
- J3: `ts_to_rust` → `{"ok": true, "check": {"ok": true, "diagnostics": "", "exitCode": 0}, "unsupported": []}` (full rust in section above). (VERIFIED)
- J4a raw: `rustc --crate-type lib js_prereg.rs` → exit `0` with 16 `unused_parens` warnings (`warning: 16 warnings emitted`); `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`. (VERIFIED)
- J4b norm: `rustc --crate-type lib -D warnings js_prereg_norm.rs` → `NORM_EXIT=0`, zero output. A first normalization attempt used an over-broad `if (` regex that broke nesting (`NORM_EXIT=1`, `unexpected closing delimiter`); per STOP it was not claimed — line-based paren-only de-nest applied instead (16 lines, semantics unchanged) and `NORM_EXIT=0` recorded. (VERIFIED)
- J5: hashes below recomputed via `shasum -a 256`; TS source cross-checked against the terminal-computed value at T1. (VERIFIED)
- J6: this file is the only path written by this agent. (VERIFIED)

## Hashes (sha256, pasted)

- TS source (`js_prereg.ts`, 97 lines): `dc46b73b3fdfe394b85140d8d30862798477ac74312a4061330ab4e5b9533220`
- Rust raw (`js_prereg.rs` as transpiled): `a2351bb86d016a89f2f5625fbddf8519116bcfd6e326aa96aacfc16ef11a74ba`
- Rust norm (`js_prereg_norm.rs`, canonical compiled artifact): `448924b936642ce2c71fa51d69b44b92b9fc0048cde99c00f7fcaa4545d84cb4`
- Mutant (`js_prereg_MUTANT.ts`, one-token mutation `bar_den` → `bar_num` in `bar_met`, line 76): `138f0c0a6de268005aba5bb55df6bd5b33cd767b7ebaa3678e8d22bd09dd368a`
- `diff` canonical vs mutant: single line — `< if (passed * bar_den >= bar_num * total) {` / `> if (passed * bar_num >= bar_num * total) {`. Hashes differ ⇒ C4 met. (VERIFIED)
- Hash-collision note (INFERRED, disclosed per §7): TS-source and both Rust hashes are byte-identical to the `typescript/preregistration.md` (and pilot `python/preregistration.md`) values. Cause: the prereg predicate struct is lane-independent by design (same 11 fns, same bytes) — the javascript lane identity lives in the header/scope/task paths, not in the compiled predicates. No silent reuse: this run re-executed every gate on local bytes (`TSC_EXIT=0`, `ts_check ok:true`, `ts_to_rust unsupported:[]`, raw exit 0, `NORM_EXIT=0` all computed 2026-10-31 in `/tmp/n3-prereg-js/`).

## Receipt (tool-write, one per prereg)

- prereg: `javascript/preregistration` (PREREG AGENT 6 of 6, Batch 3 Round 3, N3 pilot)
- source hash (sha256 of TS source): `dc46b73b3fdfe394b85140d8d30862798477ac74312a4061330ab4e5b9533220`
- Rust output hash (sha256 of `js_prereg_norm.rs` as compiled): `448924b936642ce2c71fa51d69b44b92b9fc0048cde99c00f7fcaa4545d84cb4` (raw transpiler bytes: `a2351bb8…`; delta is paren-only, diff-reviewed)
- `ts_strict`: true — wrapper `ok:true`, exit 0, zero diagnostics; system tsc exit 0
- `rust_port_supported`: true — `unsupported:[]`
- `rust_compile`: true — norm artifact `rustc -D warnings` exit 0 (`rustc 1.99.0 Homebrew`); raw exit 0 with 16 style warnings
- `sent`: false — a preregistration is a record, not a tool send; nothing registered, loaded, or executed. `gate_open(1, 1, 1) = 1` is computed and reported, not acted on.
- compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)` / `tsc Version 7.1.0-beta`

## Corrections (entries, not edits — §7.3)

1. 2026-10-31: first normalization attempt used regex `s/if \(/if /` which stripped the opening paren but left the closing paren, breaking `version_chain_ok` (`unexpected closing delimiter`, `NORM_EXIT=1`). Per STOP, the broken artifact was not claimed clean; replaced with a line-based paren-only de-nest (16 lines, semantics unchanged) and `NORM_EXIT=0` recorded. Both hashes kept; nothing rewritten silently.
