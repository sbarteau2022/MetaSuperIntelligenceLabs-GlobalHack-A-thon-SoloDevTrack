# javascript / hooks

Date: 2026-10-31 (converted, HOOK AGENT 4/6). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/javascript/`.

- Gate: `node --test` / eslint + biome | Judge: E | This machine 2026-10-31: node v26.9.0 PRESENT; eslint v10.14.0 PRESENT (binary; no `eslint.config.*` in fixture dir — flat config required since v9, so no-config run is not a gate signal); biome 2.6.4 PRESENT (binary; only a formatting nit on the single-line fixture, not a gate failure). Dated correction: the 2026-10-30 row said "eslint/biome MISSING" — binaries are present as of the 2026-10-31 check; the row stays, this note corrects it, nothing silently edited.
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-table.json` scoping this language:
- `no-eval` (deny, scope **/*.ts, **/*.tsx, **/*.js, **/*.jsx): `eval()` is a code-injection surface. Fix: Replace with JSON.parse, a lookup table, or a proper parser.
- `no-innerhtml-sink` (deny, scope **/*.ts, **/*.js): `innerHTML` assignment is an XSS sink. Fix: Use textContent / DOM construction or a sanitizer.
- `no-debugger` (deny, scope **/*.ts, **/*.tsx, **/*.js, **/*.jsx): `debugger` statement halts execution for every user. Fix: Remove it.
- `no-hardcoded-password` (deny, shared scope incl. **/*.js, **/*.jsx): Hardcoded password/secret assignment. Fix: Read from environment or secret manager; keep only a placeholder in .env.example.
- `no-console-log` (warn, scope **/*.ts, **/*.tsx, **/*.js, **/*.jsx): `console.(log|debug)` left in code. Fix: Remove or route through the logging middleware.
- `no-todo-stub` (warn, scope incl. **/*.js, **/*.jsx): TODO/FIXME/stub marker. Fix: Resolve or file the tracked issue.
- `no-select-star` (warn, scope incl. **/*.ts, **/*.js): `SELECT *` in code. Fix: Name the columns.

No `rules-languages.json` rule scopes `*.js`/`*.jsx` (VERIFIED 2026-10-31: all 15 rules read, none with a js/jsx scope — matches the prior row's note).

## Compiled hook (tool-write chain, HOOK AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the javascript hook's gate/judge: identifier-byte classifier, space-byte classifier, `eval(` call oracle, `.inner` head + `HTML` tail oracles for the `.innerHTML =` sink (optional space before `=` handled in the scan driver, mirroring the `\s*=` regex), `debugger` head/tail word-boundary oracle (`debugger_tail_ok`: tail bytes `g g e r` + right-must-not-be-ident, EOL or `;` both hit), and the three gate decisions (`js_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver; string methods are excluded from the TS subset (transpiler passes them through raw — same why-receipt pattern as the typescript hook).

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
export function eval_tail_ok(e: number, v: number, a: number, l: number, open: number): number {
  if (e === 101) {
    if (v === 118) {
      if (a === 97) {
        if (l === 108) {
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
export function inner_head_ok(i: number, n1: number, n2: number, e: number, r: number): number {
  if (i === 105) {
    if (n1 === 110) {
      if (n2 === 110) {
        if (e === 101) {
          if (r === 114) {
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
export function html_tail_ok(h: number, t: number, m: number, l: number, eq: number): number {
  if (h === 72) {
    if (t === 84) {
      if (m === 77) {
        if (l === 76) {
          if (eq === 61) {
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
export function dot_inner_sink(dot: number, i: number, n1: number, n2: number, e: number, r: number): number {
  if (dot === 46) {
    return inner_head_ok(i, n1, n2, e, r);
  } else {
    return 0;
  }
}
export function debugger_head_ok(d: number, e: number, b: number, u: number): number {
  if (d === 100) {
    if (e === 101) {
      if (b === 98) {
        if (u === 117) {
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
}
export function debugger_tail_ok(g1: number, g2: number, e2: number, r: number, after: number, has_after: number): number {
  if (g1 === 103) {
    if (g2 === 103) {
      if (e2 === 101) {
        if (r === 114) {
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
  } else {
    return 0;
  }
}
export function js_gate(deny_hits: number, warn_hits: number, has_js: number): number {
  if (has_js === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `js_hook.rs` — 12 fns (`is_ident_byte`, `byte_is`, `is_space_byte`, `eval_tail_ok`, `inner_head_ok`, `html_tail_ok`, `dot_inner_sink`, `debugger_head_ok`, `debugger_tail_ok`, `js_gate`, `pre_guard_gate`, `stop_gate`). Parens de-nested + `String` returns hand-normalized to idiomatic Rust; semantics unchanged from transpiler output.

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
pub fn eval_tail_ok(e: f64, v: f64, a: f64, l: f64, open: f64) -> f64 {
  if e == 101f64 {
    if v == 118f64 {
      if a == 97f64 {
        if l == 108f64 {
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
pub fn inner_head_ok(i: f64, n1: f64, n2: f64, e: f64, r: f64) -> f64 {
  if i == 105f64 {
    if n1 == 110f64 {
      if n2 == 110f64 {
        if e == 101f64 {
          if r == 114f64 {
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
pub fn html_tail_ok(h: f64, t: f64, m: f64, l: f64, eq: f64) -> f64 {
  if h == 72f64 {
    if t == 84f64 {
      if m == 77f64 {
        if l == 76f64 {
          if eq == 61f64 {
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
pub fn dot_inner_sink(dot: f64, i: f64, n1: f64, n2: f64, e: f64, r: f64) -> f64 {
  if dot == 46f64 {
    return inner_head_ok(i, n1, n2, e, r);
  } else {
    return 0f64;
  }
}
pub fn debugger_head_ok(d: f64, e: f64, b: f64, u: f64) -> f64 {
  if d == 100f64 {
    if e == 101f64 {
      if b == 98f64 {
        if u == 117f64 {
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
}
pub fn debugger_tail_ok(g1: f64, g2: f64, e2: f64, r: f64, after: f64, has_after: f64) -> f64 {
  if g1 == 103f64 {
    if g2 == 103f64 {
      if e2 == 101f64 {
        if r == 114f64 {
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
  } else {
    return 0f64;
  }
}
pub fn js_gate(deny_hits: f64, warn_hits: f64, has_js: f64) -> f64 {
  if has_js == 0f64 {
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

- hook: `javascript/hooks` (HOOK AGENT 4 of 6, N3 pilot Batch 3 Round 1)
- source hash (sha256 of TS source): `bc044fc94a301e7b31b69893e9da9a20d40d1c7f1f2183f15b11b73c84264856`
- Rust output hash (sha256 of `js_hook.rs` as compiled): `6633ec9598548b006443bdd2221ccd15dcf9b7036b0b48c419325e2deb611c97`
- `ts_strict`: true — diagnostics: none (exit 0, tsrs TS 7.1 via ts_check; `tsc --noEmit --strict` on same source also exit 0, no output)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib rlib/js_hook_norm.rs -D warnings`, exit 0, no output; dry-run binary `rustc dryrun_main.rs -D warnings` exit 0; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-js/`, dummies `bad_eval.js` violating `no-eval` with `eval(expr)`, `bad_sink.js` violating `no-innerhtml-sink` with `el.innerHTML = html`, `bad_debugger.js` violating `no-debugger`, clean `good.js`):
  - `pre_tool_guard.py` on `bad_eval.js` → `{"decision": "deny", ... "rule_id": "no-eval", ...}` (exit 0, fail-CLOSED envelope)
  - `pre_tool_guard.py` on `bad_sink.js` → `{"decision": "deny", ... "rule_id": "no-innerhtml-sink", ...}`
  - `pre_tool_guard.py` on `bad_debugger.js` → `{"decision": "deny", ... "rule_id": "no-debugger", ...}`
  - `pre_tool_guard.py` on `good.js` → `{"decision": "allow", ...}`
  - `audit.py /tmp/n3-hook-test-js` → `audit: 4 files, 3 deny, 0 warnings` + `[DENY] bad_debugger.js (no-debugger)` + `[DENY] bad_eval.js (no-eval)` + `[DENY] bad_sink.js (no-innerhtml-sink)`, exit 2
  - `node --test` on test dir `good.js` → pass 1, fail 0, exit 0
  - `tsc --noEmit --strict` on hook TS source → exit 0, no output
  - `eslint` v10.14.0 binary PRESENT (no `eslint.config.*` in fixture dir — flat-config run not applicable there); `biome` 2.6.4 PRESENT (flags only single-line formatting on the hand-made fixture, not a rule violation)
  - Rust dry-run binary (`dryrun_main.rs` incl. `js_hook_norm.rs`, `-D warnings`, exit 0): `ident_uscore=1 ident_digit=1 ident_upper=1 ident_lower=1 ident_dot=0` / `byte_is_eq=1 byte_is_ne=0` / `space_sp=1 space_nl=1 space_x=0` / `eval_tail_hit=1 eval_tail_no=0` / `inner_head_hit=1 inner_head_no=0` / `html_tail_hit=1 html_tail_no=0` (`HTML=` checked as `H T M L` + optional-space + `=`) / `dot_inner_hit=1 dot_inner_no=0` / `dbg_head_hit=1 dbg_head_no=0` / `dbg_tail_eol=1 dbg_tail_semi=1 dbg_tail_x=0` / `scan_eval_bad=1 scan_eval_clean=0 scan_eval_prefixword=0` (`retrieval(` correctly rejected) / `scan_sink_bad=1 scan_sink_clean=0` / `scan_dbg_bad=1 scan_dbg_clean=0` / `js_gate_deny=2 js_gate_warn=1 js_gate_clean=0 js_gate_nojs=0` / `pre_deny=deny pre_allow=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow`
- dated correction 2026-10-31 (recorded, not silently edited): v1 TS had `html_tail_ok` checking bytes `H M L L` (`t==77, m==76` — dropped the `T`); the Rust dry-run caught it (`scan_sink_bad=0` against the `innerHTML` fixture). Fixed to `H T M L` (`t==84, m==77`) in TS, re-ran the full chain (`ts_check` exit 0 → `ts_to_rust` unsupported [] → `rustc -D warnings` exit 0 → dry-run `scan_sink_bad=1`). The scan driver also skips optional whitespace before `=`, mirroring the `\s*=` in the `no-innerhtml-sink` regex. Buggy v1 preserved at `/tmp/n3-hook-js/rlib/js_hook_v1_buggy.rs`.
- no-out-of-root-writes proof: `js_hook_norm.rs` (the compiled lib) contains no `fs::`/`File::`/`Command`/`process::` (grep exit 1 = no matches). The only `fs::read` in the workspace is the dry-run driver reading its own `/tmp/n3-hook-test-js/` fixtures — outside the compiled lib, test-only by construction. No state writes: pure byte/predicate functions, no I/O imports in the lib.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in sources or this file's code blocks (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `5cb170ad247d409370aed43e6cd6a592f71374d4` (backup at `/tmp/n3-hook-js/hooks.md.orig`); working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-js/hooks.md.orig <path>` → shasum returns to `5cb170ad…`.
- callable schemas: none listed, none empty. This hook exposes no callable tool schemas — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 7 rule rows above. No listed-but-empty schemas exist to remove.
