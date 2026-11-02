---
name: brutalist-frontend
description: Raw mechanical interfaces fusing Swiss typographic print with military terminal aesthetics. Rigid grids, extreme type scale contrast, utilitarian color, analog degradation effects. For data-heavy dashboards, portfolios, or editorial sites that need to feel like declassified blueprints.
---

# brutalist-frontend (interface)

## Purpose
Architect web interfaces that fuse mid-century Swiss typographic design, industrial
manufacturing manuals, and retro-futuristic aerospace/military terminal interfaces:
rigid modular grids, extreme typographic scale contrast, utilitarian palettes,
programmatic analog degradation. Projects must feel raw, mechanical, high-density.

## When to use
Data-heavy dashboards, portfolios, or editorial sites that need to feel like
declassified blueprints. Pick ONE substrate per project and commit: Swiss Industrial
Print (light newsprint) OR Tactical Telemetry CRT (dark mono terminal). Never mix.

## Out of scope
Conventional consumer UI patterns. Gradients, soft drop shadows, modern
translucency (strictly prohibited). `border-radius` (absolute rejection: 90-degree
corners only). Mixing light and dark substrates in one interface. Terminal green
for general text (single-indicator use only).

## Type signature
Compiled gate: `tool.rs` (`brutalist-gate`, schema: `tool.json`).
- `substrate_is_valid(substrate: String) -> bool` — "swiss-print" | "tactical-terminal"
- `corners_allowed(use_border_radius: bool) -> bool` — false only
- `gradient_allowed(use_gradient: bool) -> bool` — false only
- `terminal_green_allowed(target: String) -> bool` — "single-indicator" only
- `brutalist_gate(cfg: BrutalistConfig) -> bool` — conjunction of the above
- `make_brutalist(substrate, use_border_radius, use_gradient, text_color) -> BrutalistConfig`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/brutalist-frontend/SKILL.md`
(`a3c761afe46c60d52eb3f9945073b8ca4c8d0c2c55daed25b12e6ed98afdc`).
Full prose shrunk to this interface 2026-10-31 (Agent A2). No tool/server claims in
source (no MCP/server references found); nothing removed. See `RECEIPT.md`.
