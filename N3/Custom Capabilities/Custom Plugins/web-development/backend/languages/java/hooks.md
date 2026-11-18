# java / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 3 of 6). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/java/`.

- Gate: `javac -Xlint:all` + checkstyle/spotbugs | Judge: E | This machine 2026-10-30: MISSING (2026-10-31 re-check: `javac` present at `/usr/bin/javac` but JRE absent — `java` fails "Unable to locate a Java Runtime"; RECORD, not silent pass)
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `java-sysout` (warn, scope **/*.java): System.out/err printing is not a logging path. Fix: Remove or route through the logging framework.

## Compiled hook (tool-write chain, HOOK AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the java hook's gate/judge: identifier-byte classifier, `.`-separator oracle (`dot_follows`), 6-byte `System` matcher (S=83 y=121 s=115 t=116 e=101 m=109), 3-byte `out`/`err` matcher, 5-byte `print` matcher (lowercase p=112 — `println`/`printf`/`printStackTrace` share the `print` prefix; byte/numeric predicates only, string methods excluded from the TS subset per SENT precedent), and the three gate decisions (`java_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver. Java scope has zero deny rules, so `pre_guard_gate` allow-on-warn-only is the live path (verified in dry-run below).

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
export function left_is_ident(prev: number, has_prev: number): number {
  if (has_prev === 0) {
    return 0;
  } else {
    return is_ident_byte(prev);
  }
}
export function dot_follows(dot: number, has_dot: number): number {
  if (has_dot === 0) {
    return 0;
  } else {
    if (dot === 46) {
      return 1;
    } else {
      return 0;
    }
  }
}
export function is_system_at(s: number, y: number, s2: number, t: number, e: number, m: number): number {
  if (s === 83) {
    if (y === 121) {
      if (s2 === 115) {
        if (t === 116) {
          if (e === 101) {
            if (m === 109) {
              return 1;
            }
          }
        }
      }
    }
  }
  return 0;
}
export function is_outerr_at(a: number, b: number, c: number): number {
  if (a === 111) {
    if (b === 117) {
      if (c === 116) {
        return 1;
      }
    }
  }
  if (a === 101) {
    if (b === 114) {
      if (c === 114) {
        return 1;
      }
    }
  }
  return 0;
}
export function is_print_at(p: number, r: number, i: number, n: number, t: number): number {
  if (p === 112) {
    if (r === 114) {
      if (i === 105) {
        if (n === 110) {
          if (t === 116) {
            return 1;
          }
        }
      }
    }
  }
  return 0;
}
export function java_gate(deny_hits: number, warn_hits: number, has_java: number): number {
  if (has_java === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `java_hook.rs` — 10 fns (`is_ident_byte`, `byte_is`, `left_is_ident`, `dot_follows`, `is_system_at`, `is_outerr_at`, `is_print_at`, `java_gate`, `pre_guard_gate`, `stop_gate`). No paren normalization needed: transpiler output compiled clean under `-D warnings` as emitted; pasted block below is byte-identical to the compiled file (diff-verified).

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
pub fn left_is_ident(prev: f64, has_prev: f64) -> f64 {
  if has_prev == 0f64 {
    return 0f64;
  } else {
    return is_ident_byte(prev);
  }
}
pub fn dot_follows(dot: f64, has_dot: f64) -> f64 {
  if has_dot == 0f64 {
    return 0f64;
  } else {
    if dot == 46f64 {
      return 1f64;
    } else {
      return 0f64;
    }
  }
}
pub fn is_system_at(s: f64, y: f64, s2: f64, t: f64, e: f64, m: f64) -> f64 {
  if s == 83f64 {
    if y == 121f64 {
      if s2 == 115f64 {
        if t == 116f64 {
          if e == 101f64 {
            if m == 109f64 {
              return 1f64;
            }
          }
        }
      }
    }
  }
  return 0f64;
}
pub fn is_outerr_at(a: f64, b: f64, c: f64) -> f64 {
  if a == 111f64 {
    if b == 117f64 {
      if c == 116f64 {
        return 1f64;
      }
    }
  }
  if a == 101f64 {
    if b == 114f64 {
      if c == 114f64 {
        return 1f64;
      }
    }
  }
  return 0f64;
}
pub fn is_print_at(p: f64, r: f64, i: f64, n: f64, t: f64) -> f64 {
  if p == 112f64 {
    if r == 114f64 {
      if i == 105f64 {
        if n == 110f64 {
          if t == 116f64 {
            return 1f64;
          }
        }
      }
    }
  }
  return 0f64;
}
pub fn java_gate(deny_hits: f64, warn_hits: f64, has_java: f64) -> f64 {
  if has_java == 0f64 {
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

- hook: `java/hooks` (HOOK AGENT 3 of 6, N3 pilot Batch 3 Round 1)
- source hash (sha256 of TS source): `918ed059039b575edf2109a2c08ddc37e5e4f646cc554561fff224930d90d695`
- Rust output hash (sha256 of `java_hook.rs` as compiled): `92c1b21e89dc994ce7e485fabe51c211cb8a3f58d0b86fceb03a3f8bc2188ae2`
- `ts_strict`: true — diagnostics: none (exit 0, tsc 7.1.0-beta via tsrs)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib java_hook.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-java/`, dummy `Bad.java` violating `java-sysout` with `System.out.println("debug noise")`, clean `Good.java` with `java.util.logging.Logger`):
  - `pre_tool_guard.py` on `Bad.java` → `{"decision": "allow", "report_reason": null, "rule_id": null, "fix": null}` (warn-only rule: allows at pre-guard by design; warns fire at post/audit, not pre — matches go precedent where warn-only `bad.go` also allowed)
  - `pre_tool_guard.py` on clean `Good.java` → `{"decision": "allow", ...}`; malformed stdin (`{"bogus":1}`, missing path) → `deny` with `rule_id: "schema-error"` (fail CLOSED verified)
  - `audit.py /tmp/n3-hook-test-java` → `audit: 2 files, 0 deny, 1 warnings` + `[WARN] Bad.java (java-sysout)`, exit 1
  - `javac -Xlint:all` on test dir → cannot run: JRE absent (`java` fails "Unable to locate a Java Runtime"; `javac` present but unusable without runtime) — MISSING recorded, not silent pass
  - Rust dry-run binary (`dryrun_main.rs` incl. `java_hook.rs`, `-D warnings`, exit 0): `bad_warn_hits=1 gate=1 expect_warn=1 gate_expect=1` / `good_warn_hits=0 gate=0 expect=0` / `nojava_gate=0 expect=0` / `pre_warn_only=allow pre_clean=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow` — all asserts passed (`DRYRUN-OK`)
- no-out-of-root-writes proof: `java_hook.rs` (shipped artifact) contains no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io`/`std::process` (grep exit 1 = no matches). Pure byte/predicate functions, no I/O imports. (Dry-run driver `dryrun_main.rs` reads the two test files via `std::fs::read` and never writes — `fs::write|File::create|write!|writeln!` grep exit 1 = no matches.)
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in artifact or driver (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `01f7810e04e5f6872e1aeb5cda73bb90fdc734c1`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-java/hooks.md.orig <path>` → shasum returns to `01f7810e04e5f6872e1aeb5cda73bb90fdc734c1` (backup at `/tmp/n3-hook-java/hooks.md.orig`).
- callable schemas: hook exposes no callable tool schemas of its own — its contract surface is the hook events (`pre_tool`/`post_edit`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 1 rule row above. Sibling `java/tools.md` lists `JavaRouteRequest`, `JavaBoundedOutput`, `JavaToolReceipt` — all non-empty, all populated. No listed-but-empty schemas exist to remove.
