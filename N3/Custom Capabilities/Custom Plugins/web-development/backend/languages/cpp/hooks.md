# cpp / hooks

Date: 2026-10-30 (converted 2026-10-31, HOOK AGENT 3 of 6). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/cpp/`.

- Gate: `clang++ -std=c++20 -Wall` + `clang-tidy` | Judge: E/D | This machine 2026-10-31: clang 21 INSTALLED (`Apple clang version 21.0.0`); `clang-tidy` MISSING (`which clang-tidy` exit 1 — RECORD, not silent pass); `cpp-raw-new` = D-warn
- Hook events: `pre_tool` (route first), `post_edit` (re-verify), `stop` (completion gate: checkers pasted or MISSING recorded — never a silent pass).
- Harness scripts (canonical, no drift): `web-development/hooks/scripts/pre_tool_guard.py` (deny-list on write content, fail CLOSED), `post_edit_verify.py` (append warnings to `.agent-state/pending-issues.jsonl`), `stop_completion_gate.py` (veto on pending issues, archive-on-report). Event envelopes: `web-development/hooks/HOOK_EVENT_SCHEMA.md`.

Deny/warn rules from `web-development/hooks/rules-languages.json` scoping this language:
- `cpp-raw-new` (warn, scope **/*.cpp, **/*.hpp, **/*.cc, **/*.hh): Raw new/new[] needs a matching delete; prefer ownership types. Fix: Use std::unique_ptr / std::vector / std::string.

## Compiled hook (tool-write chain, HOOK AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source (fully typed, no `any`) — models the cpp hook's gate/judge: identifier-byte classifier, blank-byte classifier (`is_blank`: space/tab/LF/CR), 3-byte `new` matcher (n=110 e=101 w=110... n=110 e=101 w=119), right-boundary oracle (`right_is_ident`, for the `\b` in the table pattern `=\s*new\b`), left-side `=`-scan expressed in the Rust dry-run driver (backwards blank-skip to `=`), and the three gate decisions (`cpp_gate`: audit exit 0/1/2; `pre_guard_gate`: deny-on-hit, deny-on-schema-error / fail CLOSED; `stop_gate`: continue-on-pending, allow-when-archived-or-clean). Byte-vector scanning loops are expressed in the Rust dry-run driver. Cpp scope has zero deny rules (`cpp-raw-new` is warn-only), so `pre_guard_gate` allow-on-content is the live path (verified in dry-run below).

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
export function is_blank(b: number): number {
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
export function is_new_at(n: number, e: number, w: number): number {
  if (n === 110) {
    if (e === 101) {
      if (w === 119) {
        return 1;
      }
    }
  }
  return 0;
}
export function right_is_ident(next: number, has_next: number): number {
  if (has_next === 0) {
    return 0;
  } else {
    return is_ident_byte(next);
  }
}
export function cpp_gate(deny_hits: number, warn_hits: number, has_cpp: number): number {
  if (has_cpp === 0) {
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

Rust port (`ts_to_rust`, unsupported: none): `cpp_hook.rs` — 8 fns (`is_ident_byte`, `byte_is`, `is_blank`, `is_new_at`, `right_is_ident`, `cpp_gate`, `pre_guard_gate`, `stop_gate`). Paren normalization applied (mechanical `sed -E 's/if \((.*)\) \{/if \1 {/'` — transpiler emits `if (cond) {`, rustc `-D warnings` rejects `unused_parens`; go/csharp SENT precedent compiled clean as emitted, cpp did not — RECORD, not silent): compiled clean under `-D warnings` after normalization; pasted block below is byte-identical to the compiled file (recompile-verified).

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
pub fn is_blank(b: f64) -> f64 {
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
pub fn is_new_at(n: f64, e: f64, w: f64) -> f64 {
  if n == 110f64 {
    if e == 101f64 {
      if w == 119f64 {
        return 1f64;
      }
    }
  }
  return 0f64;
}
pub fn right_is_ident(next: f64, has_next: f64) -> f64 {
  if has_next == 0f64 {
    return 0f64;
  } else {
    return is_ident_byte(next);
  }
}
pub fn cpp_gate(deny_hits: f64, warn_hits: f64, has_cpp: f64) -> f64 {
  if has_cpp == 0f64 {
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

- hook: `cpp/hooks` (HOOK AGENT 3 of 6, N3 pilot Batch 5 Round 1)
- source hash (sha256 of TS source): `a48b14bb61be8b77a83cfa429e4329d9c887bff713a23a79d7ef27f0a670e3d6`
- Rust output hash (sha256 of `cpp_hook.rs` as compiled): `ccad3109536c105bdd069927c35aa7980d7ad0b61bf8a14cf781f577db160857`
- `ts_strict`: true — diagnostics: none (exit 0, tsc 7.1.0-beta via tsrs)
- `rust_port_supported`: true — unsupported list: [] (empty)
- `rust_compile`: true — `rustc --crate-type lib cpp_hook.rs -D warnings`, exit 0, no output (after mechanical paren normalization, see above); `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-hook-test-cpp/`, dummy `bad.cpp` violating `cpp-raw-new` with `int* p = new int(42);`, clean `good.cpp` with `std::make_unique<int>`):
  - `pre_tool_guard.py` on `bad.cpp` → `{"decision": "allow", "report_reason": null, "rule_id": null, "fix": null}` (warn-only rule: allows at pre-guard by design — pre-guard is a deny-list; warns fire at post/audit, not pre — matches go/java precedent where warn-only `bad.go`/`Bad.java` also allowed)
  - `pre_tool_guard.py` on clean `good.cpp` → `{"decision": "allow", ...}`; malformed stdin (`{"bogus":1}`, missing path) → `deny` with `rule_id: "schema-error"` (fail CLOSED verified)
  - `audit.py /tmp/n3-hook-test-cpp` → `audit: 2 files, 0 deny, 1 warnings` + `[WARN] bad.cpp (cpp-raw-new)`, exit 1
  - `clang++ -std=c++20 -Wall` on both files → exit 0, no warnings (raw `new` is not a `-Wall` diagnostic — the D-gate `audit.py` is what fires `cpp-raw-new`; E/D split as headered)
  - `clang-tidy` → cannot run: `which clang-tidy` exit 1 — MISSING recorded, not silent pass (install ticket or waiver before done is claimed on real cpp work)
  - Rust dry-run binary (`dryrun_main.rs` incl. `cpp_hook.rs` copy, `-D warnings`, exit 0): `=`-anchored `new\b` scanner: `bad_new_hits=1 expect=1` / `good_new_hits=0 expect=0` (asserts passed) / `gate_bad=1 gate_good=0 gate_nocpp=0` / `pre_bad=allow pre_good=allow pre_schema_fail=deny pre_deny_path=deny` / `stop_hit=continue stop_archived=allow stop_clean=allow` — all asserts passed (`DRYRUN-OK`)
- no-out-of-root-writes proof: `cpp_hook.rs` (shipped artifact) contains no `fs::`/`File::`/`Command`/`process::`/`std::fs`/`std::io`/`std::process`/`OpenOptions`/`create_dir`/`remove_file` and no write-call forms `fs::write|File::create|write!|writeln!|.write(` (both greps exit 1 = no matches). Pure byte/predicate functions, no I/O imports. (Dry-run driver `dryrun_main.rs` reads the two test files via `std::fs::read` and never writes — same write-call grep exit 1 = no matches.)
- no-unlisted-network proof: no `fetch`/`socket`/`http`/`Tcp`/`Udp`/`net::` in artifact or driver (grep exit 1 = no matches).
- unload proof (no loader state changed; file restored byte-identical on revert): `shasum` before `dcb7af5b6eab499b3601f6d45ac88bd97b92c8cd`, working copy at hand-off differs only by this conversion; revert = `cp /tmp/n3-hook-cpp-hooks.md.orig <path>` → shasum returns to `dcb7af5b6eab499b3601f6d45ac88bd97b92c8cd` (backup at `/tmp/n3-hook-cpp-hooks.md.orig`).
- callable schemas: hook exposes no callable tool schemas of its own — its contract surface is the 4 hook events (`session_start`/`pre_write`/`post_write`/`stop`) in `HOOK_EVENT_SCHEMA.md` plus the 1 rule row above. Sibling `cpp/tools.md` lists `CppRouteRequest`, `CppBoundedOutput`, `CppToolReceipt` — all non-empty, all populated. No listed-but-empty schemas exist to remove.
