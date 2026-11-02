# go / rules — compiled predicates

Date: 2026-10-31 (RULE AGENT 3 of 6, Batch 2 Round 2). Status: PROPOSED. Scope: `plugins/web-development/backend/languages/go/`.

Normative source: `skills.md` (`## Rules` 1–10 + Pitfalls + Versions = 15 predicate rows below). This file is the predicate index (RETRIEVE: one canonical source per decision; no second copy of rule text).

Legend — severity: D = deny (block at pre-write), W = warn (allow at pre-write, veto at stop until archived/reported). Class: B = binding (machine-enforced via `pre_tool_guard.py` / `audit.py` / `stop_completion_gate.py`), A = advisory (human/reviewer-enforced). Judge: E = executable gate named in row; P = checker + pasted-output requirement. No-judge rows are CUT (none — 15/15 kept, 0 cut, proven by `kept()` dry-run below).

| # | Rule (skills.md) | Scope | S | Class | Consequence | Judge |
|---|---|---|---|---|---|---|
| 1 | Handle every error, wrap `%w`, test `errors.Is`/`As` | `**/*.go` | D | B | deny: swallowed/unwrapped error return blocked | E: `go vet ./...` + reviewer (Is/As test present) |
| 2 | No `panic` for expected failures | `**/*.go` | D | B | deny: `panic` on expected-failure path blocked | E: `grep -rn 'panic(' --include='*.go'` + reviewer sign-off |
| 3 | `context.Context` first param, never stored, `defer cancel()`, timeouts | `**/*.go` | D | B | deny: stored ctx / missing cancel blocked | E: `go vet ./...` + reviewer sign-off |
| 4 | Every goroutine has a known end (`WaitGroup`/`errgroup` + cancel; sender closes) | `**/*.go` | W | B | warn: unbounded goroutine recorded | E: `go test -race -count=1 ./...` |
| 5 | Parameterised queries (`database/sql`); check `rows.Err()` | `**/*.go` | D | B | deny: interpolated SQL blocked | E: `grep` for concat/`Sprintf` in `Query`/`Exec` + reviewer sign-off |
| 6 | Naming: no stutter, doc comment on exports, lowercase singular packages | `**/*.go` | W | B | warn: stutter/undocumented export flagged | E: `staticcheck ./...` + `gofmt -l .` clean |
| 7 | Table-driven tests (`t.Run`/`t.Helper`/`t.Cleanup`, `cmp.Diff`, exact values) | `**/*.go` (tests) | W | B | warn: non-table/inexact test recorded | P: `go test ./...` pasted exit 0 + table present |
| 8 | Slices/maps: preallocate, aliasing, nil-map panic, maps not concurrent-safe | `**/*.go` | W | B | warn: hot-path realloc / concurrent-map use flagged | E: reviewer sign-off |
| 9 | Secrets `crypto/rand`, `http.MaxBytesReader`, money as integers | `**/*.go` | D | B | deny: `math/rand` secret / float money / unlimited body blocked | E: `grep -rn 'math/rand' --include='*.go'` + reviewer sign-off |
| 10 | Modules: `go mod tidy`, `go mod verify`, `internal/`, one `main` per `cmd/<name>` | repo root | W | B | warn: untidy/unverified module recorded | E: `go mod tidy && go mod verify` |
| 11 | Accept interfaces, return concrete; small interfaces; no `init()` / package mutable state | `**/*.go` | W | B | warn: concrete-in / `init()` / mutable global flagged | E: `go vet ./...` + reviewer sign-off |
| 12 | Race gate: `go test -race -count=1 ./...` | `**/*.go` | W | B | warn: no `-race` run recorded | P: `go test -race -count=1 ./...` pasted exit 0 |
| 13 | Versions: run `go version`, honor project pin, add pin if missing | repo root | W | B | warn: unpinned project recorded | P: `go version` + `go.mod` pin pasted |
| 14 | Pitfall: nil pointer inside interface is not `== nil` | `**/*.go` | D | B | deny: bare `== nil` on error/interface blocked | E: `go vet ./...` + reviewer (explicit nil-interface test) |
| 15 | Pitfalls: shadowed `err`, copied `sync.Mutex`, range-over-copy, pre-1.22 loop capture | `**/*.go` | W | B | warn: recorded at audit | P: `go vet ./...` (copylocks) pasted exit 0 + reviewer |

## Contradiction check vs merged union + skills.md (binding wins)

Merged union = `rules-table.json` (18 rows) + `rules-languages.json` (15 rows). Rows scoping `**/*.go`: `go-fmt-print` (W, `fmt\.Print`); globals `no-secrets-aws-key` (D), `no-private-key` (D).

| Pair | Verdict |
|---|---|
| `go-fmt-print` (W, union) vs this index (no `fmt.Print` row) | CONSISTENT — union-only coverage extension, enforced by the `hooks.md` gate (`go vet` + audit warn path). No predicate row needed here; no conflict. Binding wins. |
| `no-secrets-aws-key` (D, `**`) vs #9 (secrets, D) | CONSISTENT — same direction, union is the machine-enforced subset of #9. Binding wins. |
| `no-private-key` (D, `**`) vs #9 (secrets, D) | CONSISTENT — same direction. Binding wins. |
| #5 (param SQL, D) vs union (no SQL-concat row for `**/*.go`) | COVERAGE GAP, not contradiction — enforced by reviewer sign-off + `grep`, recorded here. No rule cut. |
| #2 (no panic, D) vs union (no panic row for `**/*.go`) | COVERAGE GAP, not contradiction — enforced by `grep` + reviewer sign-off, recorded here. No rule cut. |
| Normative text vs this index (RETRIEVE) | NO DUPLICATION — rule text lives in `skills.md`; this file holds predicates only. |

Hard contradictions: 0. Rows cut for no-judge: 0. Folded-not-dropped: skills.md R3 split into #4/#12, Pitfalls mutex/shadow/range/capture folded into #15, R5 `defer`-cleanup half enforced via `go vet` + reviewer (no single-regex row).

## Compiled predicates (tool-write chain, RULE AGENT 3 of 6, 2026-10-31)

TS 7.1 subset source — severity/class/judge projection over rule # (1–15), conflict-winner (binding wins), and file gate (deny > warn > clean; 0 when no go):

```ts
export function sev_of(rule: number): number {
  if (rule === 1) { return 2; }
  if (rule === 2) { return 2; }
  if (rule === 3) { return 2; }
  if (rule === 5) { return 2; }
  if (rule === 9) { return 2; }
  if (rule === 14) { return 2; }
  if (rule < 1) { return 0; }
  if (rule > 15) { return 0; }
  return 1;
}
export function binding_of(rule: number): number {
  if (rule < 1) { return 0; }
  if (rule > 15) { return 0; }
  return 1;
}
export function judge_kind_of(rule: number): number {
  if (rule === 7) { return 3; }
  if (rule === 12) { return 3; }
  if (rule === 13) { return 3; }
  if (rule === 15) { return 3; }
  if (rule < 1) { return 0; }
  if (rule > 15) { return 0; }
  return 1;
}
export function kept(has_judge: number): number {
  if (has_judge === 1) { return 1; } else { return 0; }
}
export function conflict_winner(sev_a: number, bind_a: number, sev_b: number, bind_b: number): number {
  if (bind_a > bind_b) { return sev_a; } else {
    if (bind_b > bind_a) { return sev_b; } else {
      if (sev_a >= sev_b) { return sev_a; } else { return sev_b; }
    }
  }
}
export function file_gate(deny_hits: number, warn_hits: number, has_go: number): number {
  if (has_go === 0) { return 0; }
  if (deny_hits > 0) { return 2; }
  if (warn_hits > 0) { return 1; }
  return 0;
}
```

Encodings: severity 2 = deny, 1 = warn, 0 = out-of-range; binding 1 = binding; judge 1 = executable (E), 3 = checker+pasted-output (P); gate 2 = deny, 1 = warn, 0 = clean/no-go.

Rust port (`go_rules.rs`, same `f64` convention as Hook-3 `go_hook.rs`): 6 fns (`sev_of`, `binding_of`, `judge_kind_of`, `kept`, `conflict_winner`, `file_gate`). Direct transpiler output, paren-normalized (`if rule == 1f64`); semantics identical.

```rust
pub fn sev_of(rule: f64) -> f64 {
  if rule == 1f64 {
    return 2f64;
  }
  if rule == 2f64 {
    return 2f64;
  }
  if rule == 3f64 {
    return 2f64;
  }
  if rule == 5f64 {
    return 2f64;
  }
  if rule == 9f64 {
    return 2f64;
  }
  if rule == 14f64 {
    return 2f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 15f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn binding_of(rule: f64) -> f64 {
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 15f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn judge_kind_of(rule: f64) -> f64 {
  if rule == 7f64 {
    return 3f64;
  }
  if rule == 12f64 {
    return 3f64;
  }
  if rule == 13f64 {
    return 3f64;
  }
  if rule == 15f64 {
    return 3f64;
  }
  if rule < 1f64 {
    return 0f64;
  }
  if rule > 15f64 {
    return 0f64;
  }
  return 1f64;
}
pub fn kept(has_judge: f64) -> f64 {
  if has_judge == 1f64 {
    return 1f64;
  } else {
    return 0f64;
  }
}
pub fn conflict_winner(sev_a: f64, bind_a: f64, sev_b: f64, bind_b: f64) -> f64 {
  if bind_a > bind_b {
    return sev_a;
  } else {
    if bind_b > bind_a {
      return sev_b;
    } else {
      if sev_a >= sev_b {
        return sev_a;
      } else {
        return sev_b;
      }
    }
  }
}
pub fn file_gate(deny_hits: f64, warn_hits: f64, has_go: f64) -> f64 {
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
```

## Receipt (tool-write, one per rules file)

- rules: `go/rules` (RULE AGENT 3 of 6, Round 2, N3 pilot)
- source hash (sha256 of TS source `/tmp/n3-rule-go/go_rules.ts`): `56f6d5b68135a4e89480993dc3f7ac896bfafe613ccd5cb01190dea468854e1a`
- Rust output hash (sha256 of `/tmp/n3-rule-go/go_rules.rs` as compiled): `f455c48a8bfac9aa0b82ecb9fce0b434b1e334e3ee892075377d99fb0dd3392e`
- `ts_strict`: true — `ts_check` MCP tool, exit 0, empty diagnostics (full 6-fn source; system `tsc 7.1.0-beta --strict --noEmit` agrees, exit 0)
- `rust_port_supported`: true — `ts_to_rust` MCP tool, unsupported list: [] (no strings, no methods, no object literals; `{[^{}]*:}` grep → 0 matches)
- `rust_compile`: true — `rustc --crate-type lib go_rules.rs -D warnings`, exit 0, no output; `rustc 1.99.0 (b940084d7 2026-09-28) (Homebrew)`
- `sent`: true — gate requires all three true; ALL TRUE
- dry-run log (`/tmp/n3-rule-go/dryrun`, `-D warnings`, exit 0): `kept=15/15 deny=6 warn=9 cut=0 gate_ok=1 winner_ok=1` (all 15 in range, binding=1, judge>=1, kept(1)=1/kept(0)=0; out-of-range 0/16 → 0; binding-beats-advisory both directions; file gate deny/warn/clean/no-go)
- no-out-of-root-writes proof: predicate sources contain no `fs::`/`File::`/`write`/`Command`/`process::`/`std::fs`/`std::io` and no `fetch`/`socket`/`http`/`Tcp`/`net::` (pure `f64` functions, no imports; grep exit 1 = no matches). This receipt's only write is the in-write-set REPLACE of `go/rules.md`.
- callable schemas: none listed, none empty. Rules files expose no callable tool schemas.
