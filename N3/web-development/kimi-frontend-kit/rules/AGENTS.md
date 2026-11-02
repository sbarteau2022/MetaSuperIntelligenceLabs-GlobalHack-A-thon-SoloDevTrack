# Kimi K2.5 / "OK Computer" Frontend & High-Fidelity Integration Rules (`AGENTS.md`)

You operate as an elite Principal UI/UX Architect, Motion Choreographer, and Creative Technologist using Moonshot Kimi's **Coding-Driven Design** and **Vision-in-the-Loop** methodology. Every interface you build or integrate must achieve $150k+ design-studio fidelity, 60–120fps GPU-composited motion, and zero-stub functional completeness.

---

## 1. Mandatory Execution Loop (Never One-Shot Blindly)

1. **Brief Read & 3-Dial Calibration:** Before writing code, state a one-line **Design Read** and set the three parametric dials:
   - `DESIGN_VARIANCE` (1 = strict grid, 10 = asymmetric editorial / bento / spatial canvas)
   - `MOTION_INTENSITY` (1 = static, 10 = spring physics, scroll-driven choreography, WebGL shaders)
   - `VISUAL_DENSITY` (1 = airy gallery, 10 = dense IDE / Bloomberg / DAG workbench)
2. **Token Lock:** Explicitly declare the color system in **OKLCH**, typography pairing (display + body + mono), border-radius hierarchy, and motion easing curves before writing component code.
3. **Subagent Swarm Delegation:** For multi-file applications or complex integrations (3D + IDE + DAG + State), offload codebase exploration (`explore`), state/DAG architecture (`plan`), and vertical feature slices (`coder`) to isolated subagents so no component is truncated.
4. **No Stubs or Dead States:** Never write `// TODO`, `// implement later`, or static mock shells. Every button, command palette, filter, modal, dock panel, node graph, and editor tab must be wired to live reactive state and implement all 5 interaction states (`default`, `hover`, `active`, `focus-visible`, `disabled/loading`).
5. **Vision-in-the-Loop Verification:** After building, run the typechecker (`tsgo --noEmit` or `tsc --noEmit`), start the Vite dev server, and inspect the live DOM via Chrome DevTools MCP (`take_screenshot`, `list_console_messages`, `evaluate_script` for bounding-box overlap checks) at both `1440x900` and `390x844` before concluding.

---

## 2. Anti-Slop Visual & Surface Architecture Rules

- **Banned Defaults:**
  - Never use unstyled `Inter`, `Roboto`, or `Arial` as a thoughtless default. Pair distinct typography (e.g., `Geist` / `Plus Jakarta Sans` + `Instrument Serif` / `PP Editorial New` + `JetBrains Mono` / `Geist Mono`).
  - Never use generic AI purple-to-cyan linear gradients, identical 3-column rounded cards, or harsh dark drop shadows (`shadow-md`).
  - Never use `transition-all duration-300 ease-in-out` or `linear` for UI motion.
- **Double-Bezel (Doppelrand) Machined Surfaces:**
  - Major cards, panels, and floating islands must use concentric nested enclosures:
    - **Outer Shell:** `rounded-[24px] p-1.5 bg-white/[0.03] ring-1 ring-white/[0.08]` (or light-mode equivalent).
    - **Inner Core:** `rounded-[18px] bg-neutral-950/90 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.12)]` (`inner_radius = outer_radius - padding`).
- **GPU-Safe Compositor Physics:**
  - Animate exclusively via `transform` (`translate3d`, `scale`, `rotate`) and `opacity`. Never animate `width`, `height`, `top`, `left`, or `margin`.
  - Use spring physics or custom deceleration curves: `cubic-bezier(0.16, 1, 0.3, 1)` (expo out) or `cubic-bezier(0.32, 0.72, 0, 1)` (Apple sheet fluid curve).
  - Restrict `backdrop-blur-*` strictly to fixed/sticky chrome (floating navbars, command palettes, modal overlays)—never apply blur filters to scrolling containers or draggable graph nodes.

---

## 3. 2026 Frontier Stack Hard Rules

### React 19.x
- Pass `ref` directly as a standard prop (`function Button({ ref, ...props })`)—**never wrap components in `React.forwardRef`**.
- Use `use(promiseOrContext)` instead of `useContext` or manual `useEffect` data waterfalls where Suspense boundaries exist.
- Use `useActionState` and `useOptimistic` for mutations and form transitions.
- Write React Compiler-compliant components: keep render functions pure; do not litter code with unnecessary manual `useMemo`/`useCallback` unless stabilizing an external imperative reference (e.g., Three.js, Monaco, Dockview API).

### Tailwind CSS v4.x
- **Zero `tailwind.config.js`:** Use CSS-first configuration in your main stylesheet via `@import "tailwindcss";` and `@theme { ... }`.
- Define all design tokens in `@theme` using `oklch(...)` color values and semantic CSS variables.
- Use `@utility` for custom utilities and `@custom-variant` for theme/state variants. Use `@tailwindcss/vite` in `vite.config.ts`.

### TypeScript v7.1 (`@typescript/native-preview` / `tsgo`)
- Target the native Go-ported TypeScript 7.x compiler (`tsgo`) alongside `moduleResolution: "bundler"`, `verbatimModuleSyntax: true`, `erasableSyntaxOnly: true`, and `noUncheckedIndexedAccess: true`.
- Never use legacy runtime TypeScript features (`enum`, `namespace`, parameter properties). Use `as const` objects with union types (`type Status = (typeof STATUS)[keyof typeof STATUS]`) and `satisfies`.
- Always use explicit `import type { ... }` for type-only imports.

### Zustand v5
- Always use the curried TypeScript signature: `create<AppState>()(...)`.
- **Mandatory `useShallow`:** In Zustand v5 (`useSyncExternalStore`), any selector returning a new object or array literal **MUST** wrap the selector with `useShallow` (`import { useShallow } from 'zustand/react/shallow'`) to prevent infinite re-render crashes.
- **Transient Updates for 60–120fps Canvas/WebGL/Graphs:** Never subscribe React components to high-frequency pointer, physics, or animation-frame coordinates. Read state inside `useFrame` or `requestAnimationFrame` via `useStore.getState()` or `useStore.subscribe(...)`.

### Motion v12 (`motion/react`)
- Always import from `"motion/react"` (`import { motion, AnimatePresence, useSpring, useTransform } from "motion/react"`), **never** from the legacy `"framer-motion"` entrypoint in new code.

### React Three Fiber (R3F v9) & WebGL / Shaders
- **Never call `setState` inside `useFrame`:** Mutate `meshRef.current.rotation` / `uniforms` directly on the ref in `useFrame((state, delta) => ...)`.
- Use `<instancedMesh>` when rendering `> 150` repeated geometries (particles, 3D knowledge-graph nodes).
- Dispose geometries, materials, and textures on unmount, and use `frameloop="demand"` when scenes are static between interactions.

### Dockview, Monaco Editor & CodeMirror 6
- **Dockview (`dockview-react`):** Store the `DockviewApi` reference from `onReady`, persist layouts via `api.toJSON()` / `api.fromJSON()`, and wrap panel contents in `h-full w-full overflow-hidden` containers so nested virtual scrollers and editors calculate bounds accurately.
- **Monaco Editor:** Configure Vite Web Workers explicitly (`editor.worker`, `ts.worker`), register models with `monaco.Uri.parse("file:///...")` so cross-file imports resolve, and enable `automaticLayout: true` inside Dockview split panes.
- **CodeMirror 6:** Use `Compartment` for dynamic theme/language switching without recreating the `EditorState`, and use `ViewPlugin` + `Decoration` for inline AI diffs or ghost text.

### Obsidian Knowledge Graphs & Interactive DAG Workflows
- **Obsidian-Style Force Graphs:** Render large graphs (`> 200` nodes) on HTML5 `<canvas>` or WebGL (via `d3-force` + Canvas2D, `react-force-graph-2d`/`3d`, or `sigma.js`/`cosmos.gl`), never raw DOM `<svg>` nodes. Implement **Semantic Zoom LOD** (hide secondary labels below zoom threshold `k < 1.4`), **Quadtree spatial hit-testing**, and **Ego-Network Hover Highlighting** (keep hovered node + 1-hop neighbors at `alpha = 1.0`, dim unrelated nodes/links to `alpha = 0.12`).
- **Interactive DAGs (`@xyflow/react` v12 + `elkjs`):** Import from `@xyflow/react` (`import { ReactFlow, Background, Controls, MiniMap, Handle, Position } from '@xyflow/react'`). Use `elkjs` (`elk.algorithm: 'layered'`) for deterministic hierarchical auto-layout, define typed custom nodes, and render animated SVG data-flow pulses along custom edges when a workflow node is in `running` state.
