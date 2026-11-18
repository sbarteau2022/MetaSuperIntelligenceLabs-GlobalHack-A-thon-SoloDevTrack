---
name: minimalist-frontend
description: Clean editorial-style interfaces. Warm monochrome palette, typographic contrast, flat bento grids, muted pastels. No gradients, no heavy shadows.
---

# minimalist-frontend (interface)

## Purpose
Premium utilitarian minimalism: warm monochrome + spot pastels, extreme typographic
contrast, flat bento grids, desaturated warm imagery. Quiet sophistication — motion
is invisible, never spectacle.

## When to use
Clean editorial interfaces where restraint is the feature. Bento grids, text-heavy
layouts, muted-pastel accents. `translateY(12px)` + opacity 600ms reveals via
`IntersectionObserver`; staggered `calc(var(--index) * 80ms)` cascades.

## Out of scope
Tailwind heavy drop shadows (`shadow-md/lg/xl`). Primary-colored hero sections.
Absolute-black body text (use `#111111`/`#2F3437`, `line-height: 1.6`). Oversaturated
stock photography. `border-radius` above 12px. Layout-triggering animation props.
`window.addEventListener('scroll')` (banned).

## Type signature
Compiled gate: `tool.rs` (`minimalist-gate`, schema: `tool.json`).
- `shadow_allowed(shadow: String) -> bool` — "none" | "diffuse-005"
- `bg_allowed(bg_style: String) -> bool` — "flat" | "subtle-texture" | "imagery"
- `text_allowed(text_color: String) -> bool` — anything but "absolute-black"
- `radius_allowed(radius: number) -> bool` — radius <= 12
- `minimalist_gate(cfg: MinimalistConfig) -> bool` — conjunction of the above
- `make_minimalist(shadow, bg_style, text_color, radius) -> MinimalistConfig`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/minimalist-frontend/SKILL.md`
(`206830251b0515c572c033785d4b668a35db358694274`). Full protocol shrunk to this
interface 2026-10-31 (Agent A2). No tool/server claims in source; nothing removed.
See `RECEIPT.md`.
