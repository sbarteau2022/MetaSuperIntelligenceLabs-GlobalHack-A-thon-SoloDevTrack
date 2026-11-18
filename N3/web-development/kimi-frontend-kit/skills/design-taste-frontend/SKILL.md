---
name: design-taste-frontend
description: Anti-slop frontend skill for landing pages, portfolios, and redesigns. The agent reads the brief, infers the right design direction, and ships interfaces that do not look templated. Real design systems when applicable, audit-first on redesigns, strict pre-flight check.
---

# design-taste-frontend (interface)

## Purpose
Anti-slop landing/portfolio/redesign delivery: brief inference to a one-line Design
Read, three-dial calibration (variance/motion/density), real design-system packages
per brief, audit-first on redesigns, mandatory pre-flight matrix before output.

## When to use
Landing pages, portfolios, redesigns where the output must not look templated.
Server Components default, Motion/scroll/pointer work isolated in `'use client'`
leaf islands. Verify every third-party import exists in `package.json` first.

## Out of scope
`window.addEventListener("scroll")` (banned — Motion `useScroll`/ScrollTrigger/
IntersectionObserver/CSS scroll-driven only). Decorative status dots, marketing
version footers, fake live-stock counters. Div-fake screenshots and hand-rolled
decorative SVGs (generate real assets first). Assuming a library exists. Brand
appendices are reference tables, not compiled behavior; canonical install commands
and doc links live in the trash source, not here.

## Type signature
Compiled gate: `tool.rs` (`antislop-gate`, schema: `tool.json`).
- `scroll_ok(scroll_listener: boolean) -> bool` — must be false
- `dots_ok(status_dots: String) -> bool` — "none" | "semantic-single"
- `antislop_gate(cfg: AntiSlop) -> bool` — no-scroll AND real images AND dots-ok AND no footer-version
- `make_antislop(scroll_listener, images_real, status_dots, footer_version) -> AntiSlop`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/design-taste-frontend/SKILL.md`
(`aa194351b246b8b4799099d4ed7b033d29e8c6d18b8c`). 87KB appendix tables shrunk to
this interface 2026-10-31 (Agent A2). No callable MCP/tool schema in source
appendices (install commands + doc links, not executables); nothing removed.
See `RECEIPT.md`.
