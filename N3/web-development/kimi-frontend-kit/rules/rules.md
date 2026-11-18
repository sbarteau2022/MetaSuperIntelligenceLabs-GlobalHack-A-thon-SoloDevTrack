# Quick-Reference Rules Mirror (`rules.md` / `.cursorrules` / `.windsurfrules` / `GEMINI.md`)

> Drop this file at your project root as `AGENTS.md`, `GEMINI.md`, or `.agents/rules/frontend.md`.

1. **Workflow First:** Execute the 5-stage Kimi loop: `Brief Read & 3-Dial Calibration -> Token Spec (OKLCH) -> Subagent Decomposition -> Zero-Stub Implementation -> Browser Vision QA`.
2. **No AI Slop:** No default `Inter`/`Roboto`, no purple-blue SaaS gradients, no flat 1px gray cards, no `linear`/`ease-in-out` motion. Use **Double-Bezel nested surfaces** (`outer ring + inner inset highlight`) and custom spring/cubic-bezier physics (`cubic-bezier(0.16, 1, 0.3, 1)`).
3. **React 19:** Ref-as-prop (no `forwardRef`), `use()`, `useActionState`, `useOptimistic`, React Compiler pure render discipline.
4. **Tailwind CSS v4:** `@import "tailwindcss";` + `@theme { ... }` in CSS using `oklch()` tokens. Do not create `tailwind.config.js`.
5. **TypeScript 7.1 (`tsgo`):** `verbatimModuleSyntax: true`, `erasableSyntaxOnly: true`, no `enum` or `namespace` (use `as const` + `satisfies`), explicit `import type`.
6. **Zustand v5:** Curried `create<T>()(...)`, mandatory `useShallow` for object/array selectors, and transient `useStore.getState()` inside 60fps `useFrame` / Canvas loops.
7. **Motion v12:** Import from `"motion/react"`. Animate only `transform` and `opacity`.
8. **R3F & WebGL:** Zero React `setState` in `useFrame`. Use `InstancedMesh` for `>150` items and custom `shaderMaterial` uniforms mutated by ref.
9. **Dockview + Monaco + CodeMirror 6:** Persist `DockviewApi` JSON layouts, wire Monaco Web Workers + `file:///` URIs with `automaticLayout: true`, and use CodeMirror `Compartment` for live reconfiguration.
10. **Obsidian Graphs & XYFlow DAGs:** Use Canvas2D/WebGL + `d3-force` with quadtree hit-testing, semantic zoom label LOD, and 1-hop ego-network dimming for Knowledge Graphs. Use `@xyflow/react` v12 + `elkjs` (`layered` algorithm) + animated SVG particle edges for interactive DAGs.
