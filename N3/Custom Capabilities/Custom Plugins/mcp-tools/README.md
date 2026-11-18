# plugins / mcp-tools — top node (route-first, 7 core tools)

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/mcp-tools/` → subcat (`web-development/`) → `languages/` → five docs.

This node sits ON TOP of the plugin tree per Stewart's spec: MCP tools route first, then the subcat, then languages, then the five per-language files. No agent loads 37 servers x 10 tools every turn.

## The 7 core tools (parameterized by language, not duplicated per language)

1. `route(task)` — minimal tool/skill list + token estimate + reason (~700 tokens upfront, 1500-token cap).
2. `search(kind, keyword)` — summaries only, never full definitions.
3. `read(path, language)` — bounded, with hash.
4. `write(path, language)` — with receipt per write (prev_hash, new_hash, bytes, ok/fail).
5. `execute(name, args, language)` — allowlisted only, output truncated at 4000 chars, receipt per run.
6. `test(language)` — deterministic judges; pass only if every judge passes; why-receipt on fail.
7. `graph(query, language)` — HyperMap / RAG query, summaries first, details on demand.

Each language supplies only its schemas, judges, and compilers underneath these 7 (see `../web-development/*/languages/<lang>/tools.md`). Standard: `tool_router` (route/search/execute, truncation, receipts, token cap) — PASS, use as template. OD 37-server stack as-is — FAIL (no route-first, no cap, no receipts, placeholder keys); backbone only.

## Sources (copies live in staging; do not load staging into OD/Zed until vetted)

- Staging: `N3/staging-tools-2026-10-30/` — mcp-servers/ (tool_router, autonomous_runbook, language_toolchain, zed_engineering_toolchain, zed_research_sources, optiq_harness_bridge), configs/ (4), TOOLS-AUDIT.md.
- Staging catalogs/ dir is empty (no catalog files staged); server inventory lives in TOOLS-AUDIT.md + configs/.
- Brief: `N3/brief-tools-fetch.md`. Queue block 08-mcp-surface owns the MCP-surface audit.

## Routing

Language checkers + harness tools per language: `../web-development/languages-plugin.json`. Per-language tool rows: `../web-development/{frontend,backend,middleware}/languages/<lang>/tools.md`.
