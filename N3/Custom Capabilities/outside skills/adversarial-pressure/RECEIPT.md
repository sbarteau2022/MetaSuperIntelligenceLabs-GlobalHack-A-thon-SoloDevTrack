# RECEIPT — adversarial-pressure

Date: 2026-10-31. Agent: Wave 2 Agent A2 (Skills-B). Tool-write chain, boolean gate.

- Tool: pressure-gate
- Source: `TRASH skills-desktop-plugins-20261030/engineering/skills/adversarial-pressure.md` (sha256 `ae5a320c462f8b79431a140b0c006e75819df3a91` — VERIFIED)
- N3 dest: `/Users/stewartbarteau/Desktop/N3/skills/adversarial-pressure/` (`tool.ts`, `tool.rs`, `tool.json`, `SKILL.md`, `RECEIPT.md`)
- N3 tool.ts sha256: `c7292cf642ee5e40ed40d91d49094e66b3cef8b444a8f2955f7dab882ac5f5ed`
- N3 tool.rs sha256 (as compiled): `29d8ab1cbe4a27d21b392c3e6bc34add087f763da003c95add62c97211ace1cd`
- N3 tool.json: present (typed JSON schema for the gate inputs)
- ts_strict: true — `ts_check` exit 0, zero diagnostics (TS 7.1 via tsrs, `--noEmit --strict`); raw output per tool: `{"ok":true,"diagnostics":"","exitCode":0}`
- rust_port_supported: true — `ts_to_rust` ok:true, unsupported: []. Porter output + mechanical unused-parens fix only (regex `if (...) {` → `if ... {`, same as Wave-1 precedent): no logic added, no logic removed.
- rust_compile: true — `rustc --crate-type=lib --deny warnings tool.rs`, raw output empty, exit 0 (re-verified at receipt time for this file).
- rust_analyzer_clean: true — `rust_analyzer_diagnostics` on emitted tool.rs: `{"errors":0,"warnings":0,"hints":0,"information":0,"diagnostics":[]}` (workspace-still-loading note attached by server; zero diagnostics reported).
- sent: true (all four booleans true)
- Removed claims: run_python/vm_exec = host belt tools, not compiled claims.
- NOT checked: live host-runtime I/O (file/network `:8088`/browser/MCP round-trips are outside the ts-to-rust subset; ported layer is the pure decision/default schema). Full-length prose lives in Trash source; this N3 SKILL.md is the interface.
