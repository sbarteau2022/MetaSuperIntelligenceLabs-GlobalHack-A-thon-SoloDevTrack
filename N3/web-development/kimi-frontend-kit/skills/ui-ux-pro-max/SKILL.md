---
name: ui-ux-pro-max
description: "UI/UX design intelligence for web, mobile, and desktop. This skill should be used when designing, building, reviewing, or fixing interfaces, including pages, components, design systems, accessibility, interaction, responsive layout, typography, color, charts, and stack-specific UI implementation. Searchable local data: 192 product palettes and reasoning profiles, 74 font pairings, 119 UX guidelines, 105 icons, 17 GSAP presets, 25 chart types, and 22 stacks."
---

# ui-ux-pro-max (interface)

## Purpose
Design intelligence over local CSV data: BM25 search across style/color/chart/
landing/product/ux/typography/font/icon/motion/react domains, 22 stack guideline
tracks, full design-system generation (Master + page-override persistence), and
1–10 variance/motion/density dials over the spacing scale.

## When to use
Designing, building, reviewing, or fixing interfaces of any surface. `--design-system`
for new pages/projects (persist with `--output-dir <project-root>`, `--force` to
overwrite, `--page` for overrides, overrides win over MASTER.md). `--stack` for
stack-specific guidelines. Retry broader keywords and say so before falling back
when a search returns 0 results.

## Out of scope
`--stack` combined with `--design-system` (stack is ignored in that mode — run a
separate `--stack` query). `--persist` without an explicit `--output-dir` at the
project root. Executable scripts (`search.py`, `core.py`, `design_system.py`,
`reasoning_contract.py`, `validate_data.py` + tests) stay as data/scripts; the
compiled gate validates the query contract only.

## Type signature
Compiled gate: `tool.rs` (`promax-query-gate`, schema: `tool.json`).
- `promax_domain_valid(domain: String) -> bool` — style|color|chart|landing|product|ux|typography|stack
- `promax_mode_valid(mode: String) -> bool` — "search" | "design-system"
- `promax_count_valid(max_results: number) -> bool` — 1..20
- `dial_valid(value: number) -> bool` — 1..10
- `promax_gate(cfg: ProMaxQuery) -> bool` — mode AND domain AND count valid
- `make_promax(mode, domain, stack, max_results) -> ProMaxQuery`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/ui-ux-pro-max/`
(SKILL.md `ea087c341bfb5c3030268ede86da802d52627`; + data/ + references/ + scripts/).
Logic moved to `tool.rs`; prose shrunk to this interface 2026-10-31 (Agent A2).
`${CLAUDE_PLUGIN_ROOT}` paths are host-runtime references, not callable MCP/tool
claims; nothing removed. Scripts + data merge into existing N3 dir (same names
present — no duplicates added). See `RECEIPT.md`.
