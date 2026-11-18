---
name: skill-audit
description: Audit existing skills, tools, MCP servers, and plugins for completeness, correctness, and gaps. Use when the user asks to review, improve, or inventory agent capabilities and tooling.
---

# Skill and Tooling Audit

You are auditing agent capabilities (skills, tools, MCP servers, plugins).

## Audit checklist
1. **Inventory**
   - List all skills (global + project-local) with name, scope, and description quality.
   - List all built-in tools with risk level and confinement.
   - List all MCP servers in manifests with enabled/disabled state.
   - List plugin entries and their kinds.
   - List hooks, commands, workflows, holes present.
2. **Quality checks (per skill)**
   - Frontmatter parses (YAML valid, required fields present, name matches directory).
   - Description is specific and actionable (not vague).
   - Body is focused and does not duplicate AGENTS.md noise.
   - Dependencies resolve (topological, no cycles) if declared.
   - Supporting files exist and are referenced.
3. **Tool checks**
   - Confinement: no traversal (`..`, absolute paths) in read tools.
   - Write/high-risk tools require approval or hole binding.
   - No secrets in args/results/telemetry (scan manifests and logs).
   - Uniform output contract (JSON Value or error string).
   - Every tool has telemetry (counts + latency, no args/results).
4. **MCP checks**
   - Sessions are lazy (connect on first use) and reusable.
   - No literal keys in manifests (only slot names).
   - Tool results are data, never entered into command paths.
   - Idle eviction works and no process leaks.
5. **Plugin checks**
   - Manifest is parseable (or parse error reported, never silently reset).
   - Enabled state and kind are valid.
   - Capability gating (least privilege) is enforced.
6. **Gap identification**
   - Missing tool categories (file write/edit, git, web, terminal, subagent, etc.).
   - Stale or broken skills (files missing, deps unresolvable).
   - Under-documented tools (no usage examples, unclear params).
   - Security gaps (unconfined paths, unguarded writes, secret leakage risk).
7. **Output**
   - Report counts (skills, tools, plugins, MCP servers, hooks, commands).
   - List gaps by priority (high/medium/low) with concrete next steps.
   - Label every finding as VERIFIED, REPORTED, INFERRED, or PROPOSED.
   - Never claim a tool exists if it is only scaffolded; mark scaffolds as PROPOSED.

## Anti-patterns
- Do not mark scaffolded code as implemented.
- Do not claim test coverage without running tests.
- Do not invent tool counts; count actual files/entries.
