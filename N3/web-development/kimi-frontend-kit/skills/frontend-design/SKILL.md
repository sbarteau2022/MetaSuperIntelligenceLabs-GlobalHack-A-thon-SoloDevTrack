---
name: frontend-design
description: Guidance for distinctive, intentional visual design when building new UI or reshaping an existing one. Helps with aesthetic direction, typography, and making choices that don't read as templated defaults.
---

# frontend-design (interface)

## Purpose
Subject-grounded, intentional visual design: brief-first aesthetic direction,
typographic contrast, one memorable element per page, self-critique with
screenshots, conversational end-user copy, failure/empty states with direction.

## When to use
Building new UI or reshaping existing UI where the design must not read as a
templated default. Plan, review against the brief, build, critique.

## Out of scope
Default typographic treatments as choices (Inter-everywhere, generic hero +
3-cards + footer skeletons). Clever-over-legible naming. Apologetic error copy.
Decorative additions that do not serve the brief (remove one accessory). Announcing
a quality floor instead of building to it (responsive, focus, reduced-motion).

## Type signature
Compiled gate: `tool.rs` (`design-verdict-gate`, schema: `tool.json`).
- `brief_wins(brief_pins: boolean, default_used: boolean) -> bool` — brief pins win; defaults fail only on free axes
- `design_gate(cfg: DesignVerdict) -> bool` — grounded AND brief_wins AND tone_ok AND accessible
- `make_verdict(grounded, default_used, tone_ok, accessible) -> DesignVerdict`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/frontend-design/SKILL.md`
(`d91970639e9f5c37682ac7ab60094d35f1c7c1d31`). Shrunk to this interface 2026-10-31
(Agent A2). No tool/server claims in source; nothing removed. See `RECEIPT.md`.
