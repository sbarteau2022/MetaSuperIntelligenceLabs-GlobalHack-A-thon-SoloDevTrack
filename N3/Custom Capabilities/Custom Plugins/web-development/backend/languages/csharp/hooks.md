# csharp / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 3 of 6). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/csharp/`.

- Gate: `dotnet build` + `dotnet test` + `dotnet format` | Judge: E | This machine 2026-10-31: MISSING (`dotnet: command not found`, exit 127 — RECORD, not silent pass; cf. java JRE-absent precedent)
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
(none — no `rules-languages.json` rule scopes this language's extensions; verified 2026-10-31: 15 rules, zero match `csharp`/`c#`/`*.cs`; nearest analogues are warn-only stdout-noise rules `go-fmt-print` and `java-sysout` in sibling languages)

## Compiled hook (tool-write chain, HOOK AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the csharp hook's gate/judge: identifier-byte classifier, `.`-separator oracle (`dot_follows`), 7-byte `Console` matcher (C=67 o=111 n=110 s=115 o=111 l=108 e=101), 5-byte `Write` matcher (W=87 r=114 i=105 t=116 e=101 — prospective `Console.Write*` stdout-noise detector, same shape as go `fmt.Print*` / java `System.out.print*`; no table rule consumes it yet, so the live gate path is allow/deny-on-schema-error), and the three gate decisions (`csharp_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver. Csharp scope has zero deny rules, so `pre_guard_gate` allow-on-content is the live path (verified in dry-run below).

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
export function is_console_at(c: number, o: number, n: number, s: number, o2: number, l: number, e: number): number {
  if (c === 67) {
    if (o === 111) {
      if (n === 110) {
        if (s === 115) {
          if (o2 === 111) {
            if (l === 108) {
              if (e === 101) {
                return 1;
              }
            }
          }
        }
      }
    }
  }
  return 0;
}
export function is_write_at(w: number, r: number, i: number, t: number, e: number): number {
  if (w === 87) {
    if (r === 114) {
      if (i === 105) {
        if (t === 116) {
          if (e === 101) {
            return 1;
          }
        }
      }
    }
  }
  return 0;
}
export function csharp_gate(deny_hits: number, warn_hits: number, has_cs: number): number {
  if (has_cs === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `csharp_hook.rs` — 9 fns (`is_ident_byte`, `byte_is`, `left_is_ident`, `dot_follows`, `is_console_at`, `is_write_at`, `csharp_gate`, `pre_guard_gate`, `stop_gate`). No paren normalization needed: transpiler output compiled clean under `-D warnings` as emitted; pasted block below is byte-identical to the compiled file (diff-verified).

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
pub fn is_console_at(c: f64, o: f64, n: f64, s: f64, o2: f64, l: f64, e: f64) -> f64 {
  if c == 67f64 {
    if o == 111f64 {
      if n == 110f64 {
        if s == 115f64 {
          if o2 == 111f64 {
            if l == 108f64 {
              if e == 101f64 {
                return 1f64;
              }
            }
          }
        }
      }
    }
  }
  return 0f64;
}
pub fn is_write_at(w: f64, r: f64, i: f64, t: f64, e: f64) -> f64 {
  if w == 87f64 {
    if r == 114f64 {
      if i == 105f64 {
        if t == 116f64 {
          if e == 101f64 {
            return 1f64;
          }
        }
      }
    }
  }
  return 0f64;
}
pub fn csharp_gate(deny_hits: f64, warn_hits: f64, has_cs: f64) -> f64 {
  if has_cs == 0f64 {
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

- hook: `csharp/hooks` (HOOK AGENT 3 of 6, N3 pilot Batch 4 Round 1)
- source hash (sha256 of TS source): `007ec0763afd69cc28653029ca78f65e5f6960dfdbfb207199d38bee571a1910`
- Rust output hash (sha256 of `csharp_hook.rs` as compiled): `e643fc5ac40bce90684474bcc9e7d17c1f0cf9f6592d2b19a45e3935c37499b4`
- `ts_strict`: true — diagnostics: none (exit 0, via tsrs)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib csharp_hook.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-csharp/`, dummy `Bad.cs` with `Console.WriteLine("debug noise")`, clean `Good.cs` with `Logger.Log`):
  - `pre_tool_guard.py` on `Bad.cs` → `{"decision": "allow", "report_reason": null, "rule_id": null, "fix": null}` (zero table rules scope `**/*.cs`: allows at pre-guard by design; warns fire at post/audit, not pre — matches go/java precedent where warn-only `bad.go`/`Bad.java` also allowed)
  - `pre_tool_guard.py` on clean `Good.cs` → `{"decision": "allow", ...}`; malformed stdin (`{"bogus":1}`, missing path) → `deny` with `rule_id: "schema-error"` (fail CLOSED verified)
  - `audit.py /tmp/n3-hook-test-csharp` → `audit: 2 files, 0 deny, 0 warnings`, exit 0 (no table rule fires on `.cs` — consistent with zero scoped rows)
  - `dotnet` gate checkers (`build`/`test`/`format`) → cannot run: `dotnet: command not found`, exit 127 — MISSING recorded, not silent pass (cf. java JRE-absent handling)
  - Rust dry-run binary (`dryrun_main.rs` incl. `csharp_hook.rs`, `-D warnings`, exit 0): prospective `Console.Write` detector: `bad_console_write_hits=1 expect=1` / `good_console_write_hits=0 expect=0` (asserts passed) / `gate_bad=1 gate_good=0 gate_nocs=0` / `pre_content=allow pre_schema_fail=deny pre_deny_path=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow` — all asserts passed (`DRYRUN-OK`)
- no-out-of-root-writes proof: `csharp_hook.rs` (shipped artifact) contains no `fs::`/`File::`/`Command`/`process::`/`std::fs`/`std::io`/`std::process`/`OpenOptions`/`create_dir`/`remove_file` (grep exit 1 = no matches; the lone `write` substring in the file is the detector fn name `is_write_at`, not an I/O call) and no write-call forms `fs::write|File::create|write!|writeln!|.write(` (grep exit 1 = no matches). Pure byte/predicate functions, no I/O imports. (Dry-run driver `dryrun_main.rs` reads the two test files via `std::fs::read` and never writes — same write-call grep exit 1 = no matches.)
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in artifact or driver (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `6ed5d1c431b14c0cc810bb0c680b53cac107c242`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-csharp-hooks.md.orig <path>` → shasum returns to `6ed5d1c431b14c0cc810bb0c680b53cac107c242` (backup at `/tmp/n3-hook-csharp-hooks.md.orig`).
- callable schemas: hook exposes no callable tool schemas of its own — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus zero rule rows (no rule scopes `**/*.cs`). Sibling `csharp/tools.md` names no `XxxRequest`/`XxxOutput`/`XxxReceipt` schemas (only extensions, checkers, harness, trigger words) — all populated, nothing listed-but-empty. No listed-but-empty schemas exist to remove.
