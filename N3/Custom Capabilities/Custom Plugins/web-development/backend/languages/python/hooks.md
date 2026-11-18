# python / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 4/6). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/python/`.

- Gate: `ruff check` + `ruff format --check` (+ `mypy --strict` for typed code) | Judge: E | This machine 2026-10-30: ruff INSTALLED (0.17.0); mypy MISSING (RECORD, not silent pass)
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `py-exec-eval` (deny, scope **/*.py): exec()/eval() in Python is a code-injection surface. Fix: Replace with ast.literal_eval, a dispatch dict, or a real parser.
- `py-sql-concat` (deny, scope **/*.py): f-string SQL passed to execute() is an injection hole. Fix: Use parameterized queries (%s / ? / named placeholders).
- `py-bare-except` (warn, scope **/*.py): Bare except: swallows KeyboardInterrupt and hides the real error. Fix: Catch Exception (or narrower) and log before handling.
- `py-print-prod` (warn, scope **/*.py, excludes **/hooks/**): print() left in code outside hooks/ (hook scripts print JSON to stdout by contract). Fix: Remove or route through the logging middleware.

## Compiled hook (tool-write chain, HOOK AGENT 4 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the python hook's gate/judge: identifier-byte classifier, word-boundary oracles for `exec(`/`eval(` (left-must-not-be-ident, right-must-be-`(`), and the three gate decisions (`py_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver; string methods are excluded from the TS subset (transpiler passes them through raw — see tools.md why-receipt pattern).

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
export function right_is_ident(next: number, has_next: number): number {
  if (has_next === 0) {
    return 0;
  } else {
    if (next === 40) {
      return 2;
    } else {
      return is_ident_byte(next);
    }
  }
}
export function py_gate(deny_hits: number, warn_hits: number, has_py: number): number {
  if (has_py === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `py_hook.rs` — 7 fns (`is_ident_byte`, `byte_is`, `left_is_ident`, `right_is_ident`, `py_gate`, `pre_guard_gate`, `stop_gate`). Parens de-nested + `String` returns hand-normalized to idiomatic Rust; semantics unchanged from transpiler output.

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
pub fn right_is_ident(next: f64, has_next: f64) -> f64 {
  if has_next == 0f64 {
    return 0f64;
  } else {
    if next == 40f64 {
      return 2f64;
    } else {
      return is_ident_byte(next);
    }
  }
}
pub fn py_gate(deny_hits: f64, warn_hits: f64, has_py: f64) -> f64 {
  if has_py == 0f64 {
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

- hook: `python/hooks` (HOOK AGENT 4 of 6, N3 pilot Batch 1)
- source hash (sha256 of TS source): `42ee025ed31e1f0a63308a5f120352a7e183e92c3efedf31073d2d26acc65950`
- Rust output hash (sha256 of `py_hook.rs` as compiled): `c3cda6906db082f90bc9ded630e4b346f1c98828`
- `ts_strict`: true — diagnostics: none (exit 0, tsrs TS 7.1)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib rlib/py_hook.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-py/`, dummy `bad.py` violating `py-exec-eval` with `result = eval(cmd)`):
  - `pre_tool_guard.py` on `bad.py` → `{"decision": "deny", "report_reason": "Blocked bad.py: exec()/eval() in Python is a code-injection surface. Matched: 'eval('.", "rule_id": "py-exec-eval", "fix": "Replace with ast.literal_eval, a dispatch dict, or a real parser."}`
  - `pre_tool_guard.py` on clean `good.py` → `{"decision": "allow", ...}`; warn-only `warn.py` (bare except) → `allow` at pre-guard (warns fire at post/audit, not pre)
  - `audit.py /tmp/n3-hook-test-py` → `audit: 4 files, 1 deny, 1 warnings` + `[DENY] bad.py (py-exec-eval)` + `[WARN] warn.py (py-bare-except)`, exit 2
  - `ruff check` on test dir → flags E722 on `warn.py` (bare except), exit 1
  - Rust dry-run binary (`dryrun_main.rs` incl. `py_hook.rs`, `-D warnings`, exit 0): `bad_eval_bytes_deny=1 gate=2 expect_deny=1 gate_expect=2` / `good_gate=0 expect=0 nopy_gate=0 expect=0` / `pre_deny=deny pre_allow=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow`
- no-out-of-root-writes proof: `py_hook.rs` + dry-run contain no `fs::`/`File::`/`write`/`Command`/`process::` (grep exit 1 = no matches). No state writes by construction: pure byte/predicate functions, no I/O imports.
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in sources or this file's code blocks (grep exit 1 = no matches); only network-adjacent token is the prose word "surface" in the deny message, not a call.
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `00d0ac1d22f491b1f888f9d6dc3737d581331dd3`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-py/hooks.md.orig <path>` → shasum returns to `00d0ac1d22f491b1f888f9d6dc3737d581331dd3` (backup at `/tmp/n3-hook-py/hooks.md.orig`).
- callable schemas: none listed, none empty. This hook exposes no callable tool schemas — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 4 rule rows above. No listed-but-empty schemas exist to remove.
