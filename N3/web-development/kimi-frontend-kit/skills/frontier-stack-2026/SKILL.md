---
name: frontier-stack-2026
description: Definitive 2026 implementation patterns and anti-hallucination rules for React 19, Tailwind CSS v4 (OKLCH @theme), Vite 6/7, TypeScript v7.1 (tsgo native preview), Zustand v5 (useShallow + transient updates), Motion v12 (motion/react), and shadcn/ui.
---

# frontier-stack-2026 (interface)

## Purpose
Pinned 2026 implementation patterns: React 19 (ref-as-prop, `use()`/`useActionState`/
`useOptimistic`), Tailwind v4 CSS-first OKLCH `@theme`, tsgo typecheck, Zustand v5
`useShallow` + transient subscriptions, `motion/react` choreography, shadcn/ui.

## When to use
Writing or reviewing code on React 19 / Tailwind v4 / Vite / TS 7.1 tsgo /
Zustand v5 / Motion v12 / shadcn. Check command: `npx tsgo --noEmit`.

## Out of scope
`forwardRef` (ref is a normal prop in React 19). `framer-motion` import source
(use `motion/react`). Default coast-motion stack choices without the pinned
versions. `lenis` on fixed IDE workbenches (marketing/editorial only).

## Type signature
Compiled gate: `tool.rs` (`frontier-stack-gate`, schema: `tool.json`).
- `motion_src_valid(motion_src: String) -> bool` — "motion-react"
- `lenis_valid(surface: String) -> bool` — "marketing"
- `frontier_gate(cfg: FrontierStack) -> bool` — react19 AND tailwind_v4 AND tsgo AND motion_src_valid
- `make_frontier(react19, tailwind_v4, tsgo, motion_src) -> FrontierStack`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/frontier-stack-2026/SKILL.md`
(`048b23ff89fa434bf8758e3d28c464ae7da97f27fb194615fa7c0c2c0`). Shrunk to this
interface 2026-10-31 (Agent A2). `npx tsgo`/`npm i` are shell commands, not an MCP/tool
claim; nothing removed. See `RECEIPT.md`.
