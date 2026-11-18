# go / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 3 of 6). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/go/`.

- Gate: `go vet ./...` + `gofmt -l` | Judge: E | This machine 2026-10-30: INSTALLED 1.27.3
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `go-fmt-print` (warn, scope **/*.go): fmt.Print* in non-test Go is stdout noise, not logging. Fix: Remove or route through the structured logger.

## Compiled hook (tool-write chain, HOOK AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the go hook's gate/judge: identifier-byte classifier, `.`-separator oracle (`dot_follows`), 5-byte `Print` matcher (byte/numeric predicates: P=80 r=114 i=105 n=110 t=116, `.`=46 — `fmt.` prefix plus `Print` name; string methods excluded from the TS subset per SENT precedent), and the three gate decisions (`go_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver. Go scope has zero deny rules, so `pre_guard_gate` allow-on-warn-only is the live path (verified in dry-run below).

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
export function is_print_at(p: number, r: number, i: number, n: number, t: number): number {
  if (p === 80) {
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
export function go_gate(deny_hits: number, warn_hits: number, has_go: number): number {
  if (has_go === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `go_hook.rs` — 8 fns (`is_ident_byte`, `byte_is`, `left_is_ident`, `dot_follows`, `is_print_at`, `go_gate`, `pre_guard_gate`, `stop_gate`). No paren normalization needed: transpiler output compiled clean under `-D warnings` as emitted; pasted block below is byte-identical to the compiled file (diff-verified).

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
pub fn is_print_at(p: f64, r: f64, i: f64, n: f64, t: f64) -> f64 {
  if p == 80f64 {
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
pub fn go_gate(deny_hits: f64, warn_hits: f64, has_go: f64) -> f64 {
  if has_go == 0f64 {
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

- hook: `go/hooks` (HOOK AGENT 3 of 6, N3 pilot Batch 2 Round 1)
- source hash (sha256 of TS source): `0f86c4d16ace88e69a89ef5986065e351aa018400885faa658a365a22072ba4c`
- Rust output hash (sha256 of `go_hook.rs` as compiled): `9cf4a9235fa0d10c8c44d3d281899a39236b19166d0585d4f4c31807f463591c`
- `ts_strict`: true — diagnostics: none (exit 0, tsc 7.1.0-beta via tsrs)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib go_hook.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-go/`, dummy `bad.go` violating `go-fmt-print` with `fmt.Println("debug noise")`, clean `good.go` with `log.Println`, `go.mod`):
  - `pre_tool_guard.py` on `bad.go` → `{"decision": "allow", "report_reason": null, "rule_id": null, "fix": null}` (warn-only rule: allows at pre-guard by design; warns fire at post/audit, not pre — matches python precedent where warn-only `warn.py` also allowed)
  - `pre_tool_guard.py` on clean `good.go` → `{"decision": "allow", ...}`; malformed stdin (hand-built JSON, missing envelope) → `deny` with `rule_id: "schema-error"` (fail CLOSED verified)
  - `audit.py /tmp/n3-hook-test-go` → `audit: 3 files, 0 deny, 1 warnings` + `[WARN] bad.go (go-fmt-print)`, exit 1
  - `go vet ./...` on test dir → exit 1 on test-harness artifact only (two `main` funcs in one dir: `main redeclared`), no vet finding on `fmt.Println` itself; `gofmt -l .` → clean, no output, exit 0
  - Rust dry-run binary (`dryrun_main.rs` incl. `go_hook.rs`, `-D warnings`, exit 0): `bad_warn_hits=1 gate=1 expect_warn=1 gate_expect=1` / `good_warn_hits=0 gate=0 expect=0` / `nogo_gate=0 expect=0` / `pre_warn_only=allow pre_clean=allow pre_schema_fail=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow` — all asserts passed (`DRYRUN-OK`)
- no-out-of-root-writes proof: `go_hook.rs` (shipped artifact) contains no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io`/`std::process` (grep exit 1 = no matches). Pure byte/predicate functions, no I/O imports. (Dry-run driver `dryrun_main.rs` reads the two test files via `std::fs::read` and never writes — `fs::write|File::create|write!|writeln!` grep exit 1 = no matches.)
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in artifact or driver (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `702fbce3a836064d28b60eede1b6e29b96cf39acf3b443cea40179c1ca89ea31`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-go/hooks.md.orig <path>` → shasum returns to `702fbce3a836064d28b60eede1b6e29b96cf39acf3b443cea40179c1ca89ea31` (backup at `/tmp/n3-hook-go/hooks.md.orig`).
- callable schemas: hook exposes no callable tool schemas of its own — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 1 rule row above. Sibling `go/tools.md` lists `GoRouteRequest`, `GoBoundedOutput`, `GoToolReceipt` — all non-empty, all populated. No listed-but-empty schemas exist to remove.
