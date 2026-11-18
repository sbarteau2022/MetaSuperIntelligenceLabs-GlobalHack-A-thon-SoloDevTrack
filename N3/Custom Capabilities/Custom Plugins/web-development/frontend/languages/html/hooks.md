# html / hooks

Date: 2026-10-31 (converted, HOOK AGENT 4/6). Status: PROPOSED. Scope: `plugins/web-development/frontend/languages/html/`.

- Gate: html-validate + `audit.py` a11y D-rules (`no-div-button` etc.) | Judge: E/D | This machine 2026-10-31: html-validate v11.17.0 PRESENT (VERIFIED `html-validate --version` + `good.html` exit 0; prior row said NEEDS-VERIFY on 2026-10-30 — dated correction, not a silent edit)
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-table.json` scoping this language:
- `no-div-button` (deny, scope **/*.tsx, **/*.jsx, **/*.html, **/*.vue, **/*.svelte): `<div ... role="button">` breaks keyboard and screen-reader interaction. Fix: Use a native `<button>`.
- `no-fake-heading` (warn, scope **/*.tsx, **/*.jsx, **/*.html, **/*.vue, **/*.svelte): Possible fake heading (styled div/span instead of h1-h6). Fix: Use a real heading element with sequential level.

No `rules-languages.json` rule scopes `*.html` (VERIFIED 2026-10-31: all 15 rules read, none with an html/htm scope — matches the prior row's note).

## Compiled hook (tool-write chain, HOOK AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the html hook's gate/judge: identifier-byte classifier, space-byte classifier, quote-byte oracle (`"`/`'`), `<div` / `<span` head oracles, `role` head + `button` word oracles for the `role="button"` sink (optional whitespace around `=` handled in the scan driver, mirroring the `\s*=` regex; tag-end `>` bounds the search to one element), `class` head + `h1`-`h3` / `heading` / `title` word oracles for the fake-heading warn, and the three gate decisions (`html_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver; string methods are excluded from the TS subset (transpiler passes them through raw — same why-receipt pattern as the typescript/javascript hooks).

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
export function quote_byte_ok(q: number): number {
  if (q === 34) {
    return 1;
  }
  if (q === 39) {
    return 1;
  }
  return 0;
}
export function div_head_ok(lt: number, d: number, i: number, v: number): number {
  if (lt === 60) {
    if (d === 100) {
      if (i === 105) {
        if (v === 118) {
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
export function span_head_ok(lt: number, s: number, p: number, a: number, n: number): number {
  if (lt === 60) {
    if (s === 115) {
      if (p === 112) {
        if (a === 97) {
          if (n === 110) {
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
export function role_head_ok(r: number, o: number, l: number, e: number): number {
  if (r === 114) {
    if (o === 111) {
      if (l === 108) {
        if (e === 101) {
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
export function button_word_ok(b: number, u: number, t1: number, t2: number, o: number, n: number): number {
  if (b === 98) {
    if (u === 117) {
      if (t1 === 116) {
        if (t2 === 116) {
          if (o === 111) {
            if (n === 110) {
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
}
export function class_head_ok(c: number, l: number, a: number, s1: number, s2: number): number {
  if (c === 99) {
    if (l === 108) {
      if (a === 97) {
        if (s1 === 115) {
          if (s2 === 115) {
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
export function h_digit_ok(h: number, d: number): number {
  if (h === 104) {
    if (d === 49) {
      return 1;
    }
    if (d === 50) {
      return 1;
    }
    if (d === 51) {
      return 1;
    }
    return 0;
  } else {
    return 0;
  }
}
export function heading_head_ok(h: number, e: number, a: number, d: number): number {
  if (h === 104) {
    if (e === 101) {
      if (a === 97) {
        if (d === 100) {
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
export function title_word_ok(t: number, i: number, t2: number, l: number, e: number): number {
  if (t === 116) {
    if (i === 105) {
      if (t2 === 116) {
        if (l === 108) {
          if (e === 101) {
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
export function html_gate(deny_hits: number, warn_hits: number, has_html: number): number {
  if (has_html === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `html_hook.rs` — 15 fns (`is_ident_byte`, `byte_is`, `is_space_byte`, `quote_byte_ok`, `div_head_ok`, `span_head_ok`, `role_head_ok`, `button_word_ok`, `class_head_ok`, `h_digit_ok`, `heading_head_ok`, `title_word_ok`, `html_gate`, `pre_guard_gate`, `stop_gate`). Parens de-nested + `String` returns hand-normalized to idiomatic Rust; semantics unchanged from transpiler output.

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
pub fn quote_byte_ok(q: f64) -> f64 {
  if q == 34f64 {
    return 1f64;
  }
  if q == 39f64 {
    return 1f64;
  }
  return 0f64;
}
pub fn div_head_ok(lt: f64, d: f64, i: f64, v: f64) -> f64 {
  if lt == 60f64 {
    if d == 100f64 {
      if i == 105f64 {
        if v == 118f64 {
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
pub fn span_head_ok(lt: f64, s: f64, p: f64, a: f64, n: f64) -> f64 {
  if lt == 60f64 {
    if s == 115f64 {
      if p == 112f64 {
        if a == 97f64 {
          if n == 110f64 {
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
pub fn role_head_ok(r: f64, o: f64, l: f64, e: f64) -> f64 {
  if r == 114f64 {
    if o == 111f64 {
      if l == 108f64 {
        if e == 101f64 {
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
pub fn button_word_ok(b: f64, u: f64, t1: f64, t2: f64, o: f64, n: f64) -> f64 {
  if b == 98f64 {
    if u == 117f64 {
      if t1 == 116f64 {
        if t2 == 116f64 {
          if o == 111f64 {
            if n == 110f64 {
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
}
pub fn class_head_ok(c: f64, l: f64, a: f64, s1: f64, s2: f64) -> f64 {
  if c == 99f64 {
    if l == 108f64 {
      if a == 97f64 {
        if s1 == 115f64 {
          if s2 == 115f64 {
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
pub fn h_digit_ok(h: f64, d: f64) -> f64 {
  if h == 104f64 {
    if d == 49f64 {
      return 1f64;
    }
    if d == 50f64 {
      return 1f64;
    }
    if d == 51f64 {
      return 1f64;
    }
    return 0f64;
  } else {
    return 0f64;
  }
}
pub fn heading_head_ok(h: f64, e: f64, a: f64, d: f64) -> f64 {
  if h == 104f64 {
    if e == 101f64 {
      if a == 97f64 {
        if d == 100f64 {
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
pub fn title_word_ok(t: f64, i: f64, t2: f64, l: f64, e: f64) -> f64 {
  if t == 116f64 {
    if i == 105f64 {
      if t2 == 116f64 {
        if l == 108f64 {
          if e == 101f64 {
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
pub fn html_gate(deny_hits: f64, warn_hits: f64, has_html: f64) -> f64 {
  if has_html == 0f64 {
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

- hook: `html/hooks` (HOOK AGENT 4 of 6, N3 pilot Batch 4 Round 1)
- source hash (sha256 of TS source): `09c0e7b9076c078662404add54ea5446fd61fe81d1ac9ff83d65b52ddb8931da`
- Rust output hash (sha256 of `html_hook_norm.rs` as compiled): `a89c948e60c1826415398faf9c18d45dc64e64e2aee7ea7ba9290e04d64990d2`
- `ts_strict`: true — diagnostics: none (exit 0, TS 7.1 via ts_check; `tsc --version` 7.1.0-beta on this machine)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib rlib/html_hook_norm.rs -D warnings`, exit 0, no output; dry-run binary `rustc dryrun_main.rs -D warnings` exit 0; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-html/`, dummies `bad_div_button.html` violating `no-div-button`, `bad_fake_heading.html` violating `no-fake-heading` (warn-only), `bad_secret.html` violating universal `no-secrets-aws-key`, clean `good.html`):
  - `pre_tool_guard.py` on `bad_div_button.html` → `{"decision": "deny", ... "rule_id": "no-div-button", ...}` (exit 0, fail-CLOSED envelope)
  - `pre_tool_guard.py` on `bad_secret.html` → `{"decision": "deny", ... "rule_id": "no-secrets-aws-key", ...}`
  - `pre_tool_guard.py` on `bad_fake_heading.html` → `allow` (warn-only rules never deny at pre-guard — by contract)
  - `pre_tool_guard.py` on `good.html` → `{"decision": "allow", ...}`
  - `audit.py /tmp/n3-hook-test-html` → `audit: 4 files, 2 deny, 1 warnings` + `[DENY] bad_div_button.html (no-div-button)` + `[WARN] bad_fake_heading.html (no-fake-heading)` + `[DENY] bad_secret.html (no-secrets-aws-key)`, exit 2
  - `html-validate` v11.17.0 on `good.html` → exit 0, no findings; on `bad_div_button.html` → `1:19 error Prefer to use the native <button> element (prefer-native-element)` — independent E-gate confirmation of the deny
  - `tsc` 7.1.0-beta / node v26.9.0 present on this machine (toolchain context, not the HTML gate)
  - Rust dry-run binary (`dryrun_main.rs` incl. `html_hook_norm.rs`, `-D warnings`, exit 0): `ident_uscore=1 ident_digit=1 ident_upper=1 ident_lower=1 ident_dot=0` / `byte_is_eq=1 byte_is_ne=0` / `space_sp=1 space_nl=1 space_x=0` / `quote_dq=1 quote_sq=1 quote_x=0` / `div_hit=1 div_no=0` / `span_hit=1 span_no=0` / `role_hit=1 role_no=0` / `button_hit=1 button_no=0` / `class_hit=1 class_no=0` / `h1_hit=1 h2_hit=1 h3_hit=1 h4_no=0` (rule regex only covers h1-h3) / `heading_hit=1 title_hit=1` / `scan_btn_bad=1 scan_btn_clean=0 scan_btn_native=0` (`<button>` correctly not flagged) / `scan_head_bad=1 scan_head_clean=0 scan_head_card=0` (`class="card"` correctly not flagged) / `html_gate_deny=2 html_gate_warn=1 html_gate_clean=0 html_gate_nohtml=0` / `pre_deny=deny pre_allow=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow`
- no-out-of-root-writes proof: `html_hook_norm.rs` (the compiled lib) contains no `fs::`/`File::`/`Command`/`process::` (grep exit 1 = no matches). The only `fs::read` in the workspace is the dry-run driver reading its own `/tmp/n3-hook-test-html/` fixtures — outside the compiled lib, test-only by construction. No state writes: pure byte/predicate functions, no I/O imports in the lib.
- no-unlisted-network proof: no `fetch`/`socket`/`Tcp`/`Udp`/`net::` in sources or this file's code blocks (grep exit 1 = no matches); no `http` substring in the lib either. The only `http` in this file is the `https://html-validate.org` URL inside pasted `html-validate` checker output (evidence, not code).
