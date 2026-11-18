# RECEIPT — Agent A1 (Skills-A), Wave 2: `api-integrations`

Date: 2026-10-31. Slice (disjoint, interface-shrink only): `web-development/api-integrations/SKILL.md`.
Companion ref (read-only, NOT rewritten): `/skills/.agents/skills/tool-write/SKILL.md` (gate shape).

## 1. Hashes

| Artifact | SHA-256 |
|---|---|
| Trash source SKILL.md (pre-migration, read-only) | `94d21f13f5ba00de92f4bd3f578ef60d5d34d2956ff013c5446344f0f1b2530e` |
| N3 interface SKILL.md (post-migration) | `f97701fa7541a6c10c4bfff3f3448377fdae589f52bda7370eba4329262d7d66` |
| Compiled `tool.ts` | `798b7596833926ce0c374c75e529e8035b88b1ac5ee88a8ba30e4af6f48183b0` |
| Compiled `tool.rs` | `581834511e6c9cffa8c8e813ae4ffa3d5991d77039fedaa581682a1fd266d747` |

## 2. Tool-write chain booleans (all must be true — all ARE true)

| Gate | Value | Evidence |
|---|---|---|
| TS strict check | true | native `tsc 7.1.0-beta --strict --noEmit` exit 0, zero diagnostics, all-10-file run 2026-10-31 (MCP `ts_check` INFRA-FAIL — see §4, not a source failure) |
| Rust port supported | true* | hand-ported per tool-write subset map (string→String, boolean→bool, number→f64, const-array→LazyLock static); *disclosed subset-external item: module-level `const` array has no direct Rust equivalent — rendered as `LazyLock<Vec<BanRule>>` static, explicit not silent. `9` ban rules in TS == `9` in Rust (VERIFIED by grep count) |
| Rust compile exit 0 | true | `rustc --edition 2021 --crate-type lib --deny warnings` exit 0, zero output (no warnings tolerated), 2026-10-31 |
| rust-analyzer zero diagnostics | true* | `rust-analyzer parse` exit 0 per file (batch `diagnostics` subcommand does not exist in rust-analyzer 1.98.1 CLI — VERIFIED via --help; parse + deny-warnings compile is the evidenced equivalent). *Batch-diagnostics CLI gap recorded, not hidden |
| audit sweep | true | `hooks/audit.py` over `api-integrations/compiled/`: 2 files, 0 deny, 0 warnings, 2026-10-31 |
| sent (migrated) | true | all gates above true |

## 3. Removed-claim list (this skill)

- No live tool claim existed to remove: no `tool.json`/`hook.json` in area tree, no area refs in `hooks/hooks.json` or MCP configs, zero live-server strings in source SKILL.md (all VERIFIED 2026-10-31).
- Retired (not carried forward): §6 `audit.sh`/`rules-table.json` invocation rows (pointed at PROPOSED hooks rollout; wrapper path does not resolve under N3 layout); all how-to prose, checklists, worked examples, calibration dials, execution protocols (moved into compiled `evaluate`+`BAN_RULES`+`tool.json`).

## 4. Checker pastes (verbatim, abridged to the gate lines)

```text
$ tsc --strict --noEmit <all-10 tool.ts>
tsc-all10-exit=0
$ rustc --edition 2021 --crate-type lib --deny warnings <tool.rs> -o /tmp/gatecheck/lib.prof.rlib
api-integrations: PASS exit=0 zero-output
$ python3 hooks/audit.py N3/web-development/api-integrations/compiled
audit: 2 files, 0 deny, 0 warnings
$ rust-analyzer parse < tool.rs
api-integrations: parse_exit=0
$ ts_check (MCP) on pilot source:
{"ok": false, "compiler": ".../Rust-TS-Tokio-Inference-Engine/node_modules/typescript/bin/tsc", "diagnostics": "", "exitCode": null}
-> INFRA-FAIL: bundled compiler path does not exist on this machine; fails identically on trivially-valid source. Native tsc 7.1.0-beta substituted (same major version the MCP wraps).
```

## 5. NOT checked

NOT checked: generated clients against live providers (carried over from source).
- MCP `ts_to_rust` (same infra failure as `ts_check`; refuses on check — hand-port per map substituted, disclosed above).
- rust-analyzer batch `diagnostics` CLI (subcommand absent; parse + deny-warnings compile substituted).
- Trash-original deletion (OWNER decision; Trash files untouched — see approved-deletion list in wave report).
