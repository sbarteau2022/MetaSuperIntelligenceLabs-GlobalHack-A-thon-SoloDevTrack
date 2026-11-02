---
name: design-system
description: Token architecture, component specifications, and slide generation. Three-layer tokens (primitive→semantic→component), CSS variables, spacing/typography scales, component specs, strategic slide creation. Use for design tokens, systematic design, brand-compliant presentations.
---

# design-system (interface)

## Purpose
Three-layer token architecture (primitive → semantic → component) with CSS-variable
output, token-gated component specs, and BM25-driven strategic slide creation
(strategy/layout/copy/chart domains + contextual decision CSVs).

## When to use
Design-token setup, systematic component spec authoring, brand-compliant slide
generation. Script paths resolve from the skill base directory; the working
directory stays at the project root (reads/writes `docs/brand-guidelines.md`,
`assets/design-tokens.json`, `src/`).

## Out of scope
Hardcoded values in shipped code (hex colors, 2+ digit px, rem outside token
definitions — scan and suggest tokens). Slide generation without token compliance
validation. `slide-token-validator.py` is a legacy wrapper: prefer
`html-token-validator.py --type slides`. Executable scripts
(`generate-tokens.cjs`, `validate-tokens.cjs`, `search-slides.py`,
`slide-token-validator.py`, `fetch-background.py`, `generate-slide.py`) stay as
data/scripts alongside this interface; the compiled gate below validates
workflow inputs only, it does not execute them.

## Type signature
Compiled gate: `tool.rs` (`design-system-gate`, schema: `tool.json`).
- `token_ext_valid(ext: String) -> bool` — "css" | "json"
- `hardcoded_hit(value: String) -> bool` — "hardcoded" scan marker
- `search_domain_valid(domain: String) -> bool` — "strategy" | "layout" | "copy" | "chart"
- `design_system_gate(cfg: TokenPaths, ext: String) -> bool` — has_config AND ext-valid AND out_dir_set
- `make_token_paths(has_config, out_dir_set, fix_only) -> TokenPaths`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/design-system/`
(`8aa5ff93320559a6e5289d37b6b06b3307737dbfb4a6f3fc44135c705613511` for SKILL.md;
full dir: SKILL.md + data/ + references/ + scripts/ + templates/). Logic moved to
`tool.rs`; prose shrunk to this interface 2026-10-31 (Agent A2). No MCP/server
claims in source (only local `node`/`python scripts/...` invocations); nothing
removed. Scripts merge into existing N3 dir (same names already present —
no duplicate added). See `RECEIPT.md`.
