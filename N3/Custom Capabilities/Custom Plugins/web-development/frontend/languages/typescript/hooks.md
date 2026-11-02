# typescript / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 4/6). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/typescript/`.

- Gate: `tsc --noEmit` (strict + noUncheckedIndexedAccess) | Judge: E | This machine 2026-10-31: tsc PRESENT (7.1.0-beta, VERIFIED `tsc --version`; prior row said MISSING on 2026-10-30 — dated correction, not a silent edit)
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `ts-no-any` (warn, scope **/*.ts, **/*.tsx): Explicit any defeats the type gate. Fix: Use unknown and narrow, or a precise type; a cast needs a why-safe comment.
- `ts-floating-promise` (warn, scope **/*.ts, **/*.tsx): Un-awaited .then() chain (floating promise). Fix: await the chain or return it; enable @typescript-eslint/no-floating-promises.

## Compiled hook (tool-write chain, HOOK AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the typescript hook's gate/judge: identifier-byte classifier, space-byte classifier, word-boundary oracle for `: any` (`any_tail_ok`: tail bytes `a n y` + right-must-not-be-ident), `.then(` call oracle (`dot_then_call`: dot + `then` + open-paren), and the three gate decisions (`ts_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver; string methods are excluded from the TS subset (transpiler passes them through raw — see tools.md why-receipt pattern).

```ts
export function is_ident_byte(b: number): number {
  if (b === 95) {
    return 1;
  }
  if (b >= 48) {
    if (b <= 57) {
      return 1;
    }
  }
  if (b >= 65) {
    if (b <= 90) {
      return 1;
    }
  }
  if (b >= 97) {
    if (b <= 122) {
      return 1;
    }
  }
  return 0;
}
export function byte_is(b: number, want: number): number {
  if (b === want) {
    return 1;
  } else {
    return 0;
  }
}
export function is_space_byte(b: number): number {
  if (b === 32) {
    return 1;
  }
  if (b === 9) {
    return 1;
  }
  if (b === 10) {
    return 1;
  }
  if (b === 13) {
    return 1;
  }
  return 0;
}
export function any_tail_ok(a: number, n: number, y: number, after: number, has_after: number): number {
  if (a === 97) {
    if (n === 110) {
      if (y === 121) {
        if (has_after === 0) {
          return 1;
        } else {
          if (is_ident_byte(after) === 1) {
            return 0;
          } else {
            return 1;
          }
        }
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
export function then_tail_ok(t: number, h: number, e: number, n2: number, open: number): number {
  if (t === 116) {
    if (h === 104) {
      if (e === 101) {
        if (n2 === 110) {
          if (open === 40) {
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
    } else {
      return 0;
    }
  } else {
    return 0;
  }
}
export function dot_then_call(dot: number, t: number, h: number, e: number, n2: number, open: number): number {
  if (dot === 46) {
    return then_tail_ok(t, h, e, n2, open);
  } else {
    return 0;
  }
}
export function ts_gate(deny_hits: number, warn_hits: number, has_ts: number): number {
  if (has_ts === 0) {
    return 0;
  }
  if (deny_hits > 0) {
    return 2;
  }
  if (warn_hits > 0) {
    return 1;
  }
  return 0;
}
export function pre_guard_gate(deny_hit: number, schema_ok: number): string {
  if (schema_ok === 0) {
    return "deny";
  } else {
    if (deny_hit === 1) {
      return "deny";
    } else {
      return "allow";
    }
  }
}
export function stop_gate(pending_deny: number, pending_warn: number, archived: number): string {
  if (archived === 1) {
    return "allow";
  } else {
    if (pending_deny > 0) {
      return "continue";
    } else {
      if (pending_warn > 0) {
        return "continue";
      } else {
        return "allow";
      }
    }
  }
}
```

Rust port (`ts_to_rust`, unsupported: none): `ts_hook.rs` — 9 fns (`is_ident_byte`, `byte_is`, `is_space_byte`, `any_tail_ok`, `then_tail_ok`, `dot_then_call`, `ts_gate`, `pre_guard_gate`, `stop_gate`). Parens de-nested + `String` returns hand-normalized to idiomatic Rust; semantics unchanged from transpiler output.

```rust
pub fn is_ident_byte(b: f64) -> f64 {
  if b == 95f64 {
    return 1f64;
  }
  if b >= 48f64 {
    if b <= 57f64 {
      return 1f64;
    }
  }
  if b >= 65f64 {
    if b <= 90f64 {
      return 1f64;
    }
  }
  if b >= 97f64 {
    if b <= 122f64 {
      return 1f64;
    }
  }
  return 0f64;
}
pub fn byte_is(b: f64, want: f64) -> f64 {
  if b == want {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn is_space_byte(b: f64) -> f64 {
  if b == 32f64 {
    return 1f64;
  }
  if b == 9f64 {
    return 1f64;
  }
  if b == 10f64 {
    return 1f64;
  }
  if b == 13f64 {
    return 1f64;
  }
  return 0f64;
}
pub fn any_tail_ok(a: f64, n: f64, y: f64, after: f64, has_after: f64) -> f64 {
  if a == 97f64 {
    if n == 110f64 {
      if y == 121f64 {
        if has_after == 0f64 {
          return 1f64;
        } else {
          if is_ident_byte(after) == 1f64 {
            return 0f64;
          } else {
            return 1f64;
          }
        }
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
pub fn then_tail_ok(t: f64, h: f64, e: f64, n2: f64, open: f64) -> f64 {
  if t == 116f64 {
    if h == 104f64 {
      if e == 101f64 {
        if n2 == 110f64 {
          if open == 40f64 {
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
    } else {
      return 0f64;
    }
  } else {
    return 0f64;
  }
}
pub fn dot_then_call(dot: f64, t: f64, h: f64, e: f64, n2: f64, open: f64) -> f64 {
  if dot == 46f64 {
    return then_tail_ok(t, h, e, n2, open);
  } else {
    return 0f64;
  }
}
pub fn ts_gate(deny_hits: f64, warn_hits: f64, has_ts: f64) -> f64 {
  if has_ts == 0f64 {
    return 0f64;
  }
  if deny_hits > 0f64 {
    return 2f64;
  }
  if warn_hits > 0f64 {
    return 1f64;
  }
  return 0f64;
}
pub fn pre_guard_gate(deny_hit: f64, schema_ok: f64) -> String {
  if schema_ok == 0f64 {
    return "deny".to_string();
  } else {
    if deny_hit == 1f64 {
      return "deny".to_string();
    } else {
      return "allow".to_string();
    }
  }
}
pub fn stop_gate(pending_deny: f64, pending_warn: f64, archived: f64) -> String {
  if archived == 1f64 {
    return "allow".to_string();
  } else {
    if pending_deny > 0f64 {
      return "continue".to_string();
    } else {
      if pending_warn > 0f64 {
        return "continue".to_string();
      } else {
        return "allow".to_string();
      }
    }
  }
}
```

## Receipt (tool-write, one per hook)

- hook: `typescript/hooks` (HOOK AGENT 4 of 6, N3 pilot Batch 2 Round 1)
- source hash (sha256 of TS source): `0c722f4b74dc870277a2c79c1b9e894fff09f2475503cc2e1dafa69c0f2c2da6`
- Rust output hash (sha256 of `ts_hook.rs` as compiled): `3d7c4aabca516ac0e1046f92f004773f356b04cc68844476373ea1b3c58c83d3`
- `ts_strict`: true — diagnostics: none (exit 0, tsrs TS 7.1 via ts_check; `tsc --noEmit --strict` on same source also exit 0, no output)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib rlib/ts_hook.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-ts/`, dummy `bad_any.ts` violating `ts-no-any` with `export const label: any = ...`, dummy `floating.ts` violating `ts-floating-promise` with `p.then(...)` un-awaited, clean `good.ts`):
  - `pre_tool_guard.py` on `bad_any.ts` → `{"decision": "allow", "report_reason": null, "rule_id": null, "fix": null}` (warn-only rule: allows at pre-guard, fires at post/audit — same contract as python `warn.py`)
  - `pre_tool_guard.py` on `good.ts` → `{"decision": "allow", ...}`; `pre_tool_guard.py` on `floating.ts` → `{"decision": "allow", ...}` (warn-only, same reason)
  - `audit.py /tmp/n3-hook-test-ts` → `audit: 5 files, 0 deny, 2 warnings` + `[WARN] bad_any.ts (ts-no-any)` + `[WARN] floating.ts (ts-floating-promise)`, exit 1
  - `tsc --noEmit --strict` on test dir `good.ts` → exit 0, no output; `tsc --noEmit --strict` on hook TS source → exit 0, no output
  - Rust dry-run binary (`dryrun_main.rs` incl. `ts_hook.rs`, `-D warnings`, exit 0): `ident_uscore=1 ident_digit=1 ident_upper=1 ident_lower=1 ident_dot=0` / `byte_is_eq=1 byte_is_ne=0` / `space_sp=1 space_nl=1 space_x=0` / `any_tail_hit=1 expect=1` (`: any` + space) `any_tail_no=0 expect=0` (`: anys` rejected) / `then_tail_hit=1 then_tail_no=0` / `dot_then_hit=1 dot_then_no=0` / `ts_gate_warn=1 ts_gate_deny=2 ts_gate_clean=0 ts_gate_nots=0` / `pre_deny=deny pre_allow=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow`
- no-out-of-root-writes proof: `ts_hook.rs` + dry-run contain no `fs::`/`File::`/`write`/`Command`/`process::` (grep exit 1 = no matches). No state writes by construction: pure byte/predicate functions, no I/O imports.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in sources or this file's code blocks (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `aab20472b9532f2c8a16cfb367ca0492401c8d8b`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-ts/hooks.md.orig <path>` → shasum returns to `aab20472b9532f2c8a16cfb367ca0492401c8d8b` (backup at `/tmp/n3-hook-ts/hooks.md.orig`).
- callable schemas: none listed, none empty. This hook exposes no callable tool schemas — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 2 rule rows above. No listed-but-empty schemas exist to remove.
