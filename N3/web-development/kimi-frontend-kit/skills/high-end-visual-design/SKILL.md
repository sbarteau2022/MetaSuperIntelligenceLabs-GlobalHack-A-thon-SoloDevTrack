---
name: high-end-visual-design
description: Teaches the AI to design like a high-end agency. Defines the exact fonts, spacing, shadows, card structures, and animations that make a website feel expensive. Blocks all the common defaults that make AI designs look cheap or generic.
---

# high-end-visual-design (interface)

## Purpose
Awwwards-tier agency direction: variance-mandated layouts (never repeat the same
aesthetic twice), double-bezel machined surfaces, custom cubic-bezier motion,
GPU-safe animation, strict pre-output checklist.

## When to use
Premium marketing/experience sites that must feel expensive. Roll archetype dice
(vibe + layout) per brief; asymmetric desktop MUST collapse to `w-full px-4 py-8`
below 768px; `min-h-[100dvh]` never `h-screen`.

## Out of scope (ABSOLUTE ZERO)
Banned fonts: Inter, Roboto, Arial, Open Sans, Helvetica. Thick-stroked default
icons. 1px gray borders, harsh dark shadows. Sticky edge-to-edge navbars, symmetric
3-col Bootstrap grids. `linear`/`ease-in-out` motion, instant state changes.
`window.addEventListener('scroll')`. Animating top/left/width/height. Blur on
scrolling containers. Grain on anything but fixed pointer-events-none layers.

## Type signature
Compiled gate: `tool.rs` (`highend-gate`, schema: `tool.json`).
- `font_allowed(font: String) -> bool` — rejects inter/roboto/arial/helvetica
- `ease_allowed(motion_ease: String) -> bool` — rejects linear/ease-in-out
- `prop_allowed(anim_prop: String) -> bool` — "transform" | "opacity"
- `blur_allowed(blur_target: String) -> bool` — "fixed-only"
- `highend_gate(cfg: HighEndConfig) -> bool` — conjunction of the above
- `make_highend(font, motion_ease, anim_prop, blur_target) -> HighEndConfig`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/high-end-visual-design/SKILL.md`
(`e1e32f5e2d420872c6c7332b53d5ff764e6768c4822c6fbc`). Full protocol shrunk to
this interface 2026-10-31 (Agent A2). No tool/server claims in source; nothing
removed. See `RECEIPT.md`.
