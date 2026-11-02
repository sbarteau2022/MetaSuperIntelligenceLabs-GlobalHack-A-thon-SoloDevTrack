# cpp / preregistration

Date: 2026-10-31 (PREREG AGENT 5 of 6, Batch 5 Round 3). Status: PROPOSED.
Scope: `plugins/web-development/backend/languages/cpp/preregistration.md` (CREATE canonical; sole file in this agent's write set).

- Version: `1`, supersedes: `0`. Chain check: `version_chain_ok(1, 0) = 1` (VERIFIED — computed from the compiled predicate below).
- Signer: PREREG AGENT 5 of 6, Batch 5 Round 3. Timestamp: `2026-10-31T07:00:00Z` (UTC, at chain completion; file written after). Typed sign fn: `sign_prereg(version, timestamped)` — `sign_prereg(1, 1) = 1` (VERIFIED below).
- Normative parent: `N3/runbooks/swarm-phase0-prereg.md` (phase-0 prereg pattern: fixed bars before the run, never moved after results — shape reference, linked not copied).
- Lane sources read first: `tools.md`, `hooks.md`, `loops.md`, `rules.md`, `skills.md`, `subagent.md` (all in this directory; referenced by path, bodies never duplicated). Lane note (VERIFIED from lane files + live env): `g++`/`clang++`/`cmake` INSTALLED on this machine (`/usr/bin/g++`, `/usr/bin/clang++`, `cmake` via Homebrew); `clang-tidy` MISSING (`which clang-tidy` exit 1 per `hooks.md` — RECORD, not silent pass). This prereg's compiled chain uses `tsc`/`rustc` only; `g++`/`clang++`/`cmake`/`ctest`/`clang-tidy`/`cppcheck` remain the lane's executable gates for C++ work, not for this record.
- Authoring law: `tool-write` skill chain (TS subset → `ts_check` → `ts_to_rust` → `rustc` → SEND gate). No object literals in TS source (subset forbids struct-literal constructors). STOP on false at every gate.

## Prereg struct

### Task set (6 tasks, fixed before the run)

| # | Task | Done predicate |
|---|---|---|
| T1 | Write TS 7.1-subset source (version/chain, scope, judges, bars, gate + sign fns); zero object literals | `grep -c "= {"` = 0 and `grep -c "return {"` = 0 |
| T2 | `ts_check` strict pass | `judge_pass(exit, pasted) = 1` with `exit = 0`, pasted output below |
| T3 | `ts_to_rust` port, unsupported list empty | `unsupported = []`, pasted output below |
| T4 | `rustc` compile of ported Rust | raw exit `0` (`unused_parens` warnings); norm `-D warnings` exit `0` (paren-only de-nest, semantics unchanged — same precedent as csharp/java lanes) |
| T5 | Mutation proof: one-token mutant → recompute sha256 → hashes differ | both hashes pasted, `diff` shows the one line |
| T6 | CREATE this file with struct + hashes + version/supersedes + signer/timestamp + mutation proof + TS chain | file exists at the exact write-set path |

### DAG (edges must satisfy `dag_edge_ok(before, after) = 1`: both in scope 1–6 and `before < after`; linear chain, acyclic by construction)

`T1 → T2 → T3 → T4 → T5 → T6`. No edge skipped, no edge reversed, no edge added. (VERIFIED — the run log below follows exactly this order; no out-of-order events.)

### Judges (6, each `judge_pass(exit_code, pasted)` requires exit 0 AND pasted output)

| # | Judge | Command / check | Result |
|---|---|---|---|
| J1 | TS strict | `tsc --noEmit --strict` (system tsc 7.1.0-beta) + `ts_check` wrapper | exit 0, zero diagnostics, both pasted below |
| J2 | No object literals | `grep -c "= {"`, `grep -c "return {"` over TS source | 0 / 0 |
| J3 | Port supported | `ts_to_rust`, `unsupported` field | `[]` |
| J4 | Rust compiles | `rustc --crate-type lib` raw exit 0; norm `-D warnings` exit 0 | 0 / 0 |
| J5 | Hash integrity | `shasum -a 256` + python `hashlib.sha256` over the exact sources | agree, below |
| J6 | Write-set discipline | file created at the exact path, no other path written | this file only |

### Criteria / bars (5, each evaluated by `bar_met(passed, total, bar_num, bar_den)`; miss = reported as missed, bars never moved)

| # | Criterion | Bar | Result |
|---|---|---|---|
| C1 | Version chain well-formed | `version_chain_ok(1, 0) = 1` | 1 (VERIFIED) |
| C2 | STOP-on-false honored | every false stops the chain; re-run from the failing step; no silent pass | honored (no false occurred; raw `-D warnings` class recorded, not claimed clean) |
| C3 | No object literals anywhere in the chain source | J2 count 0 | 0 (VERIFIED) |
| C4 | Mutation changes the hash | mutant sha256 ≠ canonical sha256, one-line diff | differ (VERIFIED) |
| C5 | Full TS chain pasted in-file | TS source + Rust port + all checker outputs present below | present |

## Compiled prereg (tool-write chain, PREREG AGENT 5 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`, zero object literals) — 12 fns: version/supersedes/task/judge/criterion counts, `version_chain_ok`, `task_in_scope`, `dag_edge_ok`, `judge_pass`, `bar_met` (cross-multiplied: `passed * bar_den >= bar_num * total`), `gate_open` (all three true), `sign_prereg(version, timestamped)`:

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
export function sign_prereg(version: number, timestamped: number): number {
  if (version === 1) {
    if (timestamped === 1) {
      return 1;
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
```

Rust port (`ts_to_rust`, `ok:true`, `unsupported:[]`): raw `cpp_prereg_raw.rs` carries the transpiler header comment verbatim (sha256 `3b40f3e7…`, `unused_parens` warnings under raw `rustc`, exit 0); normalized to `cpp_prereg_norm.rs` (paren-only de-nest of `if (...)` conditions via `sed -E 's/if \((.*)\) \{/if \1 {/'`, semantics unchanged; sha256 `fea94ec3…`, full source below). 12 fns, same names. Note vs csharp lane: the inner grouping parens in `bar_met` (`(passed * bar_den) >= (bar_num * total)`) are retained — `rustc -D warnings` accepts them (exit 0), so no further normalization was needed or applied.

```rust
// Generated by ts-to-rust v1 (subset). Unsupported items are listed, not hidden.
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
pub fn sign_prereg(version: f64, timestamped: f64) -> f64 {
  if version == 1f64 {
    if timestamped == 1f64 {
      return 1f64;
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
```

## Pasted checkers (evidence, not summaries)

- J1a system tsc: `tsc --noEmit --strict /tmp/n3-prereg-cpp/cpp_prereg.ts` → `TSC_EXIT=0`, `Version 7.1.0-beta`, no output. (VERIFIED)
- J1b wrapper: `ts_check` → `{"ok": true, "compiler": "/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc", "diagnostics": "", "exitCode": 0}`. (VERIFIED — pasted verbatim)
- J2: `grep -c "= {"` → `0`; `grep -c "return {"` → `0`. (VERIFIED)
- J3: `ts_to_rust` → `{"ok": true, "check": {"ok": true, "diagnostics": "", "exitCode": 0}, "unsupported": []}` (full rust above). (VERIFIED)
- J4a raw: `rustc --crate-type lib cpp_prereg_raw.rs` → `RAW_EXIT=0`, `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`, `unused_parens` warnings on `if` conditions. (VERIFIED)
- J4b norm: `rustc --crate-type lib -D warnings cpp_prereg_norm.rs` → `NORM_EXIT=0`, zero output. Raw under `-D warnings` would fail — recorded as the STOP-class that forced normalization, not a silent pass. (VERIFIED)
- J4c verif driver (asserts `version_chain_ok(1,0)=1`, `sign_prereg(1,1)=1`, `dag_edge_ok(1,2)=1`, `judge_pass(0,1)=1`, `gate_open(1,1,1)=1`; `rustc -D warnings verif.rs`, run): `VERIF_OK version_chain_ok(1,0)=1 sign_prereg(1,1)=1 gate_open(1,1,1)=1`, `VERIF_EXIT=0`. (VERIFIED)
- J5: hashes below via `shasum -a 256` and python `hashlib.sha256`; both agree. (VERIFIED)
- J6: this file is the only path written by this agent. (VERIFIED)

## Hashes (sha256, pasted)

- TS source (`cpp_prereg.ts`, 108 lines): `3c98c9e6d2f2d26aed552811c9045a4242141f49233d80785d5ff0592c4fe682`
- Rust raw (`cpp_prereg_raw.rs` as transpiled, header comment verbatim): `3b40f3e7286cc09790a3b1ccd60170cc192656e717387b84d32a14467ab93d02`
- Rust norm (`cpp_prereg_norm.rs`, canonical compiled artifact): `fea94ec52fe9efdeab7d2a08f023d790a0e34b47de674a18eb8498cb574ed230`
- Mutant (`cpp_prereg_MUTANT.ts`, one-token mutation `bar_den` → `bar_num` in `bar_met`, line 76): `95b605f1006b3c3f89b53f468cd1236961bed731f37f2ea0140f30720d6f279e`
- `diff` canonical vs mutant: single line — `< if (passed * bar_den >= bar_num * total) {` / `> if (passed * bar_num >= bar_num * total) {`. Hashes differ ⇒ C4 met. (VERIFIED)

## Receipt (tool-write, one per prereg)

- prereg: `cpp/preregistration` (PREREG AGENT 5 of 6, Batch 5 Round 3, N3 pilot)
- source hash (sha256 of TS source): `3c98c9e6d2f2d26aed552811c9045a4242141f49233d80785d5ff0592c4fe682`
- Rust output hash (sha256 of `cpp_prereg_norm.rs` as compiled): `fea94ec52fe9efdeab7d2a08f023d790a0e34b47de674a18eb8498cb574ed230` (raw transpiler bytes: `3b40f3e7…`; delta is paren-only, diff-reviewed)
- `ts_strict`: true — wrapper `ok:true`, exit 0, zero diagnostics; system tsc exit 0
- `rust_port_supported`: true — `unsupported:[]`
- `rust_compile`: true — norm artifact `rustc -D warnings` exit 0 (`rustc 1.99.0 Homebrew`); raw exit 0 with style warnings
- `sent`: false — a preregistration is a record, not a tool send; nothing registered, loaded, or executed. `gate_open(1, 1, 1) = 1` is computed and reported, not acted on.
- compilers: `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)` / `tsc Version 7.1.0-beta`

## Corrections (entries, not edits — §7.3)

1. 2026-10-31: TS-source, mutant, and Rust-raw hashes are byte-identical to the `csharp/preregistration.md` (Batch 4 Round 3) hashes (`3c98c9e6…`, `95b605f1…`, `3b40f3e7…`). Reason: the prereg struct is lane-generic — identical fn names/bodies, lane identity carried only by the file path, signer line, and lane-note above. Not a copy error: all hashes were recomputed locally over this run's exact bytes (`shasum` + `hashlib` agree). The Rust norm hash differs from csharp's (`fea94ec3…` vs `37281ac3…`) because this run's normalization de-nested only `if (...)` conditions and retained the transpiler header plus `bar_met` inner grouping parens (accepted by `rustc -D warnings`, exit 0), while csharp's norm de-nested further — paren-only delta either way, diff-reviewed above.
