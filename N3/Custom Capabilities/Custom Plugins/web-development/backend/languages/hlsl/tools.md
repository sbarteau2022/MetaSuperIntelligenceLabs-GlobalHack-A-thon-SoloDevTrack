# hlsl / tools

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/hlsl/`.
Canonical routing entry: `plugins/web-development/languages-plugin.json` id `hlsl` (build source: `plugins/web-development/plugin-original.json`; legacy 41-lang reference: `~/Desktop/skills/plugins/languages/plugin.json` — do not author either).

- File extensions: `.hlsl`, `.hlsli`, `.fx`
- Checkers (run before done; paste output): `dxc`, `fxc`
- Harness tools: `vm_exec`
- Trigger words: hlsl

Tool contract (per TOOLS-2026.md + staging TOOLS-AUDIT.md): route-first, bounded output, receipt per run, why-receipt on failure. No new tool duplicates the 7 core tools (route, search, read, write-with-receipt, execute, test-with-judge, graph) — a language supplies only its schemas, judges, and compilers underneath.
