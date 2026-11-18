# Kimi K2.5 "OK Computer" Frontend & High-Fidelity Integration Workflow

This document defines the exact **5-Horizon Execution Loop** used by Moonshot Kimi (`OK Computer` mode / `kimi-cli`) to produce studio-grade frontends, 60–120fps WebGL/Canvas animations, multi-pane IDEs, Obsidian knowledge graphs, and interactive DAG workflows without falling into single-pass "AI slop."

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│ PHASE 0: BRIEF READ, 3-DIAL CALIBRATION & OKLCH TOKEN LOCK                 │
│ Output 1-line Design Read → Set [VARIANCE, MOTION, DENSITY] → Lock Tokens  │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ PHASE 1: SUBAGENT SWARM DECOMPOSITION (Isolated Contexts)                  │
│ ├─► Explorer Subagent: Scans repo, package.json, existing UI primitives    │
│ └─► Architect Subagent: Designs Zustand v5 slices, DAG schema & file tree  │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ PHASE 2: PARALLEL VERTICAL-SLICE IMPLEMENTATION                            │
│ ├─► Slice A (Surface & Motion): React 19 + Tailwind v4 + Motion v12        │
│ ├─► Slice B (Spatial/Graph/3D): R3F v9 / Shaders / Obsidian Graph / XYFlow │
│ └─► Slice C (Workbench/IDE): Dockview + Monaco + CodeMirror 6 + Cmdk       │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ PHASE 3: COMPILER & HOOK GATE (Deterministic Verification)                 │
│ Run `tsgo --noEmit` (TS 7.1) + Anti-Slop Static Check + `vite build`       │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ PHASE 4: VISION-IN-THE-LOOP BROWSER QA (Chrome DevTools MCP / Playwright)  │
│ Inspect live DOM at 1440x900 & 390x844 → Check Console/Network →           │
│ Run Bounding-Box Collision Script → Screenshot Self-Critique → Patch       │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Brief Read, 3-Dial Calibration & Token Lock

Before writing a single line of `.tsx` or `.css`, execute the **Pre-Flight Design Block**:

1. **One-Line Design Read:**
   > *"Reading this as: `<surface type>` for `<target user>`, with a `<aesthetic archetype>` language, powered by `<stack modules>`."*
2. **Set the Three Parametric Dials:**
   - `DESIGN_VARIANCE` (`1–10`): Controls grid asymmetry, Bento spans, editorial split layouts, or full-viewport spatial canvases.
   - `MOTION_INTENSITY` (`1–10`): Controls spring physics, staggered mask reveals, magnetic buttons, and shader/particle choreography.
   - `VISUAL_DENSITY` (`1–10`): Controls padding scale (`py-32` airy editorial vs. `h-screen overflow-hidden` 1px hairline Dockview IDE cockpit).
3. **Lock Design Tokens in OKLCH (`src/styles/app.css`):**
   - Background layers (`--bg-canvas`, `--bg-surface-1`, `--bg-surface-2`, `--bg-glass`)
   - Border rings (`--border-subtle: oklch(1 0 0 / 0.08)`, `--border-highlight: oklch(1 0 0 / 0.16)`)
   - Typography (`--font-display`, `--font-sans`, `--font-mono`)
   - Motion curves (`--ease-out-expo: cubic-bezier(0.16, 1, 0.3, 1)`, `--ease-fluid: cubic-bezier(0.32, 0.72, 0, 1)`)

---

## Phase 1: Subagent Swarm Decomposition

Never stuff exploration, architecture planning, shader math, and 15 React components into one monolithic turn. Offload via `define_subagent` and `invoke_subagent`:

1. **`kimi-ui-explorer` (`research` / read-only):**
   - Reads `package.json`, `tsconfig.json`, `vite.config.ts`, CSS entrypoints, and existing `/components/ui` primitives.
   - Returns a compact 40-line summary of installed package versions and available components.
2. **`kimi-state-dag-architect` (`plan`):**
   - Designs the **Zustand v5 store slices**, **TypeScript 7.1 `as const` discriminated unions**, **Dockview panel registry**, and **DAG / Knowledge Graph node-edge schemas**.
   - Outputs the exact file tree and TypeScript interface contracts (`src/types/domain.ts` and `src/store/useAppStore.ts`) first so parallel coders share a single source of truth.

---

## Phase 2: Parallel Vertical-Slice Implementation

Once `src/types/domain.ts` and `src/store/useAppStore.ts` are written to disk, dispatch isolated `coder` subagents (or execute sequentially by slice) with 100% token budget per slice:

- **Slice A — Shell, Double-Bezel Surfaces & Motion Choreography:**
  - Floating Island Navbar or IDE Activity Bar, Command Palette (`cmdk`), Double-Bezel Bento cards, Button-in-Button CTAs, and `motion/react` layout transitions (`layoutId`).
- **Slice B — Spatial Canvas, WebGL / R3F, Knowledge Graph, or DAG Engine:**
  - **R3F / WebGL:** Custom `shaderMaterial`, `InstancedMesh`, transient Zustand subscription inside `useFrame`.
  - **Obsidian Knowledge Graph:** HTML5 Canvas / WebGL `d3-force` simulation, Quadtree spatial lookup, semantic zoom label LOD, 1-hop ego-network hover highlighting, interactive node pinning.
  - **Interactive DAG Workflow:** `@xyflow/react` v12 custom typed nodes, `elkjs` layered auto-layout button, animated SVG data-flow pulse edges, live topological execution simulator.
- **Slice C — Multi-Pane Dockview Workbench & Code Editors:**
  - `DockviewReact` layout with serialized layout persistence, custom tab headers, split panes containing **Monaco Editor** (with Vite workers + multi-model URI filesystem) and/or **CodeMirror 6** (with `Compartment` live reconfig and inline diff decorations).

---

## Phase 3: Compiler & Hook Gate

Every edit passes through deterministic verification before visual inspection:

```bash
# 1. TypeScript 7.1 Native Preview (or standard tsc fallback)
npx tsgo --noEmit || npx tsc --noEmit

# 2. Anti-Slop & Frontier Stack Static Audit
./.agents/hooks/scripts/audit-frontend.sh

# 3. Production Bundle Verification
npx vite build
```

---

## Phase 4: Vision-in-the-Loop Browser QA (Chrome DevTools MCP)

Do not trust that code compiles—**inspect the rendered pixels and live DOM metrics**:

1. **Boot the Dev Server:** Run `npm run dev` as a background daemon task.
2. **Navigate & Check Runtime Health:**
   - Call `chrome-devtools-mcp`: `navigate_page` to `http://localhost:5173`.
   - Call `list_console_messages` and `list_network_requests` — verify **0 runtime errors**, **0 React hydration/key warnings**, and **0 failed asset/worker fetches (404s)**.
3. **Run Mathematical Layout & Overlap Audit via `evaluate_script`:**
   Execute this diagnostic script inside Chrome DevTools MCP to catch clipped panels, zero-height canvases, and horizontal overflow automatically:

   ```javascript
   () => {
     const issues = [];
     if (document.documentElement.scrollWidth > window.innerWidth) {
       issues.push(`Horizontal overflow: scrollWidth=${document.documentElement.scrollWidth} > innerWidth=${window.innerWidth}`);
     }
     document.querySelectorAll('canvas, .dv-dockview, .react-flow, .monaco-editor').forEach((el) => {
       const rect = el.getBoundingClientRect();
       if (rect.width === 0 || rect.height === 0) {
         issues.push(`Zero-dimension interactive container: ${el.className || el.tagName} (${rect.width}x${rect.height})`);
       }
     });
     return { viewport: { w: window.innerWidth, h: window.innerHeight }, issues };
   }
   ```

4. **Desktop & Mobile Viewport Screenshot Critique:**
   - Resize to `1440x900` (`resize_page`), call `take_screenshot`, and visually verify typography hierarchy, Double-Bezel borders, contrast, and canvas rendering.
   - Resize to `390x844` (`resize_page`), call `take_screenshot`, and verify responsive single-column collapse or mobile drawer behavior.
5. **Interactive State Verification:**
   - Trigger a node drag, DAG layout pass, Dockview tab split, or Command Palette (`Cmd+K`) via `click` / `press_key` and confirm smooth state transitions.
