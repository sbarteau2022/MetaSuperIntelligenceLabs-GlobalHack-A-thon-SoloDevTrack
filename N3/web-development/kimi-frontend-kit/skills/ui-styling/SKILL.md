---
name: ui-styling
description: Create beautiful, accessible user interfaces with shadcn/ui components (built on Radix UI + Tailwind), Tailwind CSS utility-first styling, and canvas-based visual designs. Use when building user interfaces, implementing design systems, creating responsive layouts, adding accessible components (dialogs, dropdowns, forms, tables), customizing themes and colors, implementing dark mode, generating visual designs and posters, or establishing consistent styling patterns across applications.
---

# ui-styling (interface)

## Purpose
shadcn/ui component installation + Tailwind theme generation + canvas visual
designs: accessible Radix-based primitives, utility-first responsive styling,
dark mode, font/color theme customization, poster-grade canvas work.

## When to use
Building UIs, implementing design systems, responsive layouts, accessible
components (dialog/dropdown/form/table), theme + dark-mode customization, visual
designs and posters.

## Out of scope
Installing components into an uninitialized project (`components.json` required —
`npx shadcn@latest init` first). Overwriting existing components without
`--overwrite`. Shipping a `tailwind.config.*` over an existing file without
`--force` review. Executable scripts (`shadcn_add.py`, `tailwind_config_gen.py`)
stay as data/scripts; the compiled gate validates request inputs only.

## Type signature
Compiled gate: `tool.rs` (`styling-gate`, schema: `tool.json`).
- `component_name_valid(component: String) -> bool` — non-empty
- `tailwind_ext_valid(ext: String) -> bool` — "ts" | "js" | "cjs" | "mjs"
- `framework_valid(framework: String) -> bool` — "react" | "vue" | "svelte" | "nextjs"
- `shadcn_gate(cfg: ShadcnRequest) -> bool` — named component AND initialized config
- `make_shadcn(component, overwrite, config_present) -> ShadcnRequest`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/ui-styling/`
(SKILL.md `625e302c60ee8f2e22d56cbe638a419e2657f566dae426`; + LICENSE.txt,
canvas-fonts/, references/, scripts/). Logic moved to `tool.rs`; prose shrunk to
this interface 2026-10-31 (Agent A2). No MCP/server claims in source (only local
`npx`/`python scripts/...`); nothing removed. Scripts + fonts merge into existing
N3 dir (same names present — no duplicates added). See `RECEIPT.md`.
