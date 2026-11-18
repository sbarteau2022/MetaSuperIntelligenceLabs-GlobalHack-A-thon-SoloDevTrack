# tailwind / hooks

Date: 2026-10-31 (converted, HOOK AGENT 4 of 6). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/tailwind/`.

- Gate: v3: config content-scan; v4: `@theme` in CSS (D-rules `no-hardcoded-color`) | Judge: D | This machine 2026-10-31: tailwindcss binary MISSING (VERIFIED `npx --no-install tailwindcss --version` fails, no local install; prior row said only "n/a (content gate)" on 2026-10-30 — dated correction, not a silent edit). Content gate VERIFIED via `audit.py` (see receipt). prettier 3.9.10 PRESENT; prettier-plugin-tailwindcss NOT installed (RECORD item); eslint 10.14.0 PRESENT (binary; no `eslint.config.*` in fixture dir — flat config required since v9, so no-config run is not a gate signal).
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-table.json` scoping this language (Tailwind v4 source-of-truth is CSS; `**/*.css` rows fire on `@theme`/utility CSS):
- `no-hardcoded-color` (warn, scope **/*.css, excl. tokens/tokens.css): Literal color in CSS bypasses the token system. Fix: Use a semantic token from frontend-design-language/tokens/.
- `no-global-reset` (warn, scope **/*.css): Global * reset cannot be overridden by components or lower layers. Fix: Target specific elements/conditions instead.
- `no-important` (warn, scope **/*.css): `!important` outside the utilities layer hides the real conflict. Fix: Fix layer order or scope the rule; keep `!important` in utilities only.
- `no-transition-all` (warn, scope **/*.css): `transition: all` animates unintended properties and hurts performance. Fix: List the animated properties explicitly (prefer transform/opacity).
- `no-linear-ease` (warn, scope **/*.css): linear/ease-in-out reads as default/generic motion. Fix: Use a token easing curve (see frontend-design-language/tokens/tokens.css).
- `no-todo-stub` (warn, scope incl. **/*.css): Stub marker will ship if the stop-gate is bypassed. Fix: Implement it now or file a tracked issue and reference its id.

No `rules-languages.json` rule scopes tailwind artifacts (VERIFIED 2026-10-31: all 15 rules read — scopes cover py/ts/tsx/rs/sh/yml/yaml/c/h/cpp/hpp/cc/hh/go/java only; tailwind declares no file extensions in `tools.md` — matches the prior row's note, which stays).

Universal `**`-scoped deny rows (`no-secrets-aws-key`, `no-private-key`) apply to every path including these (VERIFIED: `bad_secret.css` denied at pre-guard — see receipt).

v3 `tailwind.config.{js,ts}` files are `*.js`/`*.ts` by extension: the js/ts-scoped rows apply to them and are documented once in `javascript/hooks.md` / `typescript/hooks.md` (RETRIEVE — not duplicated here).

## Compiled hook (tool-write chain, HOOK AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the tailwind hook's gate/judge: identifier-byte classifier, space-byte classifier, hex-digit oracle + `#` head oracle for the `no-hardcoded-color` literal-color sink, `@` head + `theme` word oracle for the v4 `@theme` block, `!` head + `important` word oracle for the `no-important` warn (bare trailing `!` v4 modifier also counts), line-anchored `*` head + `{` oracle for the `no-global-reset` warn, and the three gate decisions (`tw_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver; string methods are excluded from the TS subset (transpiler passes them through raw — same why-receipt pattern as the html/javascript hooks).

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
export function hex_digit_ok(h: number): number {
  if (h >= 48) {
    if (h <= 57) {
      return 1;
    }
  }
  if (h >= 65) {
    if (h <= 70) {
      return 1;
    }
  }
  if (h >= 97) {
    if (h <= 102) {
      return 1;
    }
  }
  return 0;
}
export function hash_head_ok(h: number): number {
  if (h === 35) {
    return 1;
  } else {
    return 0;
  }
}
export function at_head_ok(a: number): number {
  if (a === 64) {
    return 1;
  } else {
    return 0;
  }
}
export function theme_head_ok(t: number, h: number, e: number, m: number, e2: number): number {
  if (t === 116) {
    if (h === 104) {
      if (e === 101) {
        if (m === 109) {
          if (e2 === 101) {
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
export function bang_head_ok(b: number): number {
  if (b === 33) {
    return 1;
  } else {
    return 0;
  }
}
export function important_head_ok(i: number, m: number, p: number, o: number, r: number, t: number, a: number, n: number, t2: number): number {
  if (i === 105) {
    if (m === 109) {
      if (p === 112) {
        if (o === 111) {
          if (r === 114) {
            if (t === 116) {
              if (a === 97) {
                if (n === 110) {
                  if (t2 === 116) {
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
export function star_head_ok(s: number): number {
  if (s === 42) {
    return 1;
  } else {
    return 0;
  }
}
export function brace_ok(b: number): number {
  if (b === 123) {
    return 1;
  } else {
    return 0;
  }
}
export function tw_gate(deny_hits: number, warn_hits: number, has_css: number): number {
  if (has_css === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `tw_hook.rs` — 14 fns (`is_ident_byte`, `byte_is`, `is_space_byte`, `hex_digit_ok`, `hash_head_ok`, `at_head_ok`, `theme_head_ok`, `bang_head_ok`, `important_head_ok`, `star_head_ok`, `brace_ok`, `tw_gate`, `pre_guard_gate`, `stop_gate`). Parens de-nested + `String` returns hand-normalized to idiomatic Rust; semantics unchanged from transpiler output.

```rust
// Generated by ts-to-rust v1 (subset). Unsupported items are listed, not hidden.
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
pub fn hex_digit_ok(h: f64) -> f64 {
  if h >= 48f64 {
    if h <= 57f64 {
      return 1f64;
    }
  }
  if h >= 65f64 {
    if h <= 70f64 {
      return 1f64;
    }
  }
  if h >= 97f64 {
    if h <= 102f64 {
      return 1f64;
    }
  }
  return 0f64;
}
pub fn hash_head_ok(h: f64) -> f64 {
  if h == 35f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn at_head_ok(a: f64) -> f64 {
  if a == 64f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn theme_head_ok(t: f64, h: f64, e: f64, m: f64, e2: f64) -> f64 {
  if t == 116f64 {
    if h == 104f64 {
      if e == 101f64 {
        if m == 109f64 {
          if e2 == 101f64 {
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
pub fn bang_head_ok(b: f64) -> f64 {
  if b == 33f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn important_head_ok(i: f64, m: f64, p: f64, o: f64, r: f64, t: f64, a: f64, n: f64, t2: f64) -> f64 {
  if i == 105f64 {
    if m == 109f64 {
      if p == 112f64 {
        if o == 111f64 {
          if r == 114f64 {
            if t == 116f64 {
              if a == 97f64 {
                if n == 110f64 {
                  if t2 == 116f64 {
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
pub fn star_head_ok(s: f64) -> f64 {
  if s == 42f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn brace_ok(b: f64) -> f64 {
  if b == 123f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn tw_gate(deny_hits: f64, warn_hits: f64, has_css: f64) -> f64 {
  if has_css == 0f64 {
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

- hook: `tailwind/hooks` (HOOK AGENT 4 of 6, N3 pilot Batch 5 Round 1)
- source hash (sha256 of TS source): `6407bb9fd422ae74609ef3b717817ecea30db0a49fa97d5fe08666a2da6faf98`
- Rust output hash (sha256 of `tw_hook_norm.rs` as compiled): `1519a4e6e6e5762468217430add76d5cf4223a76d65137d4bd1ee47c851c61e0`
- `ts_strict`: true — diagnostics: none (exit 0, TS 7.1 via ts_check; `tsc --version` 7.1.0-beta on this machine)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib tw_hook_norm.rs -D warnings`, exit 0, no output; dry-run binary `rustc dryrun_main.rs -D warnings` exit 0; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-tailwind/`, dummies `bad_color.css` violating `no-hardcoded-color`, `bad_global.css` violating `no-global-reset`, `bad_important.css` violating `no-important`, universal `bad_secret.css` violating `no-secrets-aws-key`, clean `good.css` with `@theme` + token var):
  - `pre_tool_guard.py` on `bad_secret.css` → `{"decision": "deny", ... "rule_id": "no-secrets-aws-key", "report_reason": "Blocked bad_secret.css: Possible AWS access key in file content. Matched: 'AKIAIOSFODNN7EXAMPLE'."}` (exit 0, fail-CLOSED envelope)
  - `pre_tool_guard.py` on `bad_color.css` → `{"decision": "allow", ...}` (warn-only rules never deny at pre-guard — by contract)
  - `pre_tool_guard.py` on `good.css` → `{"decision": "allow", ...}`
  - `post_edit_verify.py` on `bad_color.css` → `{}` (exit 0) + appended `{"path": "bad_color.css", "rule_id": "no-hardcoded-color", "message": "Literal color in CSS bypasses the token system.", ...}` to pending-issues
  - `stop_completion_gate.py` with that pending warning → `{"decision": "continue", "report_reason": "Completion vetoed: 1 pending warning(s). bad_color.css: [no-hardcoded-color] ..."}` (veto confirmed)
  - `audit.py /tmp/n3-hook-test-tailwind` → `audit: 6 files, 1 deny, 3 warnings` + `[WARN] bad_color.css (no-hardcoded-color)` + `[WARN] bad_global.css (no-global-reset)` + `[WARN] bad_important.css (no-important)` + `[DENY] bad_secret.css (no-secrets-aws-key)`, exit 2
  - `prettier --check good.css` → `All matched files use Prettier code style!` (prettier 3.9.10 PRESENT); prettier-plugin-tailwindcss NOT installed (RECORD item, no silent pass). `tailwindcss` binary MISSING (`npx --no-install tailwindcss --version` fails — RECORD item, content gate carries the signal). `eslint` 10.14.0 PRESENT (binary; no `eslint.config.*` in fixture dir — flat config required since v9, so no-config run is not a gate signal; same why-receipt pattern as the javascript hook). `node --check tailwind.config.js` → OK (node v26.9.0).
  - `tsc` 7.1.0-beta / `ts_check` exit 0 on the hook TS source (toolchain context, not the tailwind gate)
  - Rust dry-run binary (`dryrun_main.rs` incl. `tw_hook_norm.rs`, `-D warnings`, exit 0): `ident_uscore=1 ident_digit=1 ident_upper=1 ident_lower=1 ident_dot=0` / `byte_is_eq=1 byte_is_ne=0` / `space_sp=1 space_nl=1 space_x=0` / `hex_0=1 hex_F=1 hex_f=1 hex_g=0` / `hash_hit=1 hash_no=0` / `at_hit=1 at_no=0` / `theme_hit=1 theme_no=0` / `bang_hit=1 bang_no=0` / `important_hit=1 important_no=0` / `star_hit=1 star_no=0 brace_hit=1 brace_no=0` / `scan_color_bad=1 scan_color_clean=0` / `scan_global_bad=1 scan_global_clean=0` (`good.css` correctly not flagged) / `scan_important_bad=1 scan_important_clean=0` / `scan_theme_good=1 scan_theme_bad=0` (v4 `@theme` detected in `good.css` only) / `tw_gate_deny=2 tw_gate_warn=1 tw_gate_clean=0 tw_gate_nocss=0` / `pre_deny=deny pre_allow=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow`
- no-out-of-root-writes proof: `tw_hook_norm.rs` (the compiled lib) contains no `fs::`/`File::`/`Command`/`process::` (grep count 0, exit 1 = no matches). The only `std::fs::read` in the workspace is the dry-run driver reading its own `/tmp/n3-hook-test-tailwind/` fixtures — outside the compiled lib, test-only by construction. No state writes: pure byte/predicate functions, no I/O imports in the lib.
- no-unlisted-network proof: no `fetch`/`socket`/`Tcp`/`Udp`/`net::` in sources or this file's code blocks (grep count 0, exit 1 = no matches); no `http` substring in the lib either. No URLs appear in this file at all.
