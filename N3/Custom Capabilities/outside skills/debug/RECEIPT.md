# RECEIPT — debug

Date: 2026-10-31. Agent: Wave 2 Agent A2 (Skills-B). Tool-write chain, boolean gate.

- Tool: debug-gate
- Source: `TRASH skills-desktop-plugins-20261030/engineering/skills/debug.md` (sha256 `cae4a7b052b466104dcffeae941e1349c4a4d` — VERIFIED)
- N3 dest: `/Users/stewartbarteau/Desktop/N3/skills/debug/` (`tool.ts`, `tool.rs`, `tool.json`, `SKILL.md`, `RECEIPT.md`)
- N3 tool.ts sha256: `92f84c2473a248073b092a010a04b954d624b7a4ffb82bb33916d468ace0af88`
- N3 tool.rs sha256 (as compiled): `d9b2122c5c245020f3ebf7b04aee8d6275bc4dc6822e0003bfcc74447a4b63fe`
- N3 tool.json: present (typed JSON schema for the gate inputs)
- ts_strict: true — `ts_check` exit 0, zero diagnostics (TS 7.1 via tsrs, `--noEmit --strict`); raw output per tool: `{"ok":true,"diagnostics":"","exitCode":0}`
- rust_port_supported: true — `ts_to_rust` ok:true, unsupported: []. Porter output + mechanical unused-parens fix only (regex `if (...) {` → `if ... {`, same as Wave-1 precedent): no logic added, no logic removed.
- rust_compile: true — `rustc --crate-type=lib --deny warnings tool.rs`, raw output empty, exit 0 (re-verified at receipt time for this file).
- rust_analyzer_clean: true — `rust_analyzer_diagnostics` on emitted tool.rs: `{"errors":0,"warnings":0,"hints":0,"information":0,"diagnostics":[]}` (workspace-still-loading note attached by server; zero diagnostics reported).
- sent: true (all four booleans true)
- Removed claims: No tool/server claims in source.
- NOT checked: live host-runtime I/O (file/network `:8088`/browser/MCP round-trips are outside the ts-to-rust subset; ported layer is the pure decision/default schema). Full-length prose lives in Trash source; this N3 SKILL.md is the interface.
