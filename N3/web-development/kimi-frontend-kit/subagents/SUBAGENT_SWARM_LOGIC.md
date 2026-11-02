# Subagent Swarm Logic & Orchestration Guide

Kimi K2.5’s **Agent Swarm** architecture prevents context window degradation and late-file code truncation by splitting frontend engineering into **6 specialized subagent archetypes**.

---

## Why Subagent Isolation Makes Frontend Code Better

1. **Zero Context Pollution:** Scanning `node_modules`, `package.json`, and 20 existing components consumes 15k–40k tokens. Offloading that to `kimi-ui-explorer` keeps the UI coder's context window 100% focused on aesthetics and physics.
2. **Full Token Budget Per Visual Subsystem:** When a single agent tries to write a WebGL shader, a Force-Directed Knowledge Graph, a Dockview IDE, and a Bento marketing page in one turn, it shortcuts the animations. Dedicated subagents give each subsystem 100% attention.
3. **A/B Aesthetic Swarming:** For high-stakes heroes or dashboards, spawn two parallel `kimi-surface-motion-coder` subagents with `Workspace: "branch"`—one building **Archetype A (Ethereal OLED Glass + R3F Shader)** and one building **Archetype B (Warm Editorial Swiss Grid + Kinetic Typography)**—then compare both in Chrome DevTools.

---

## The 6 Specialized Subagents (Antigravity `define_subagent` Specs)

### 1. `kimi-ui-explorer`
- **Role:** `Frontend Codebase & Token Explorer`
- **Write Tools:** `false` | **MCP Tools:** `false`
- **When to Invoke:** Step 1 on any existing codebase to inventory installed libraries (React 19, Tailwind v4, Zustand v5, R3F, XYFlow, Dockview, Monaco), CSS variables, and component primitives.

### 2. `kimi-state-dag-architect`
- **Role:** `State Machine, Type & Graph Architect`
- **Write Tools:** `true` | **MCP Tools:** `false`
- **When to Invoke:** Step 2 before parallel UI coding. Authors `src/types/domain.ts` (TS 7.1 `as const` unions, branded IDs) and `src/store/useAppStore.ts` (Zustand v5 slices with `useShallow` selectors and transient 60fps getters).

### 3. `kimi-surface-motion-coder`
- **Role:** `Principal UI Surface & Motion Choreographer`
- **Write Tools:** `true` | **MCP Tools:** `false`
- **When to Invoke:** Builds the Tailwind v4 `@theme` OKLCH token sheet, Double-Bezel (Doppelrand) Bento components, shadcn/ui customizations, Fluid Island navigation, and `motion/react` v12 spring choreography.

### 4. `kimi-webgl-graph-coder`
- **Role:** `WebGL, R3F, Knowledge Graph & DAG Engineer`
- **Write Tools:** `true` | **MCP Tools:** `false`
- **When to Invoke:** Builds React Three Fiber v9 scenes, custom GLSL/TSL shaders, `InstancedMesh` particle fields, Obsidian-style `d3-force` Canvas/WebGL knowledge graphs (with quadtree hit-testing + semantic zoom LOD + ego-network dimming), and `@xyflow/react` v12 + `elkjs` interactive DAG workflows.

### 5. `kimi-workbench-ide-coder`
- **Role:** `Dockview Workbench, Monaco & CodeMirror Specialist`
- **Write Tools:** `true` | **MCP Tools:** `false`
- **When to Invoke:** Builds multi-pane desktop-class workbenches using `dockview-react`, configures Vite Web Workers for Monaco Editor (`@monaco-editor/react` + Shiki + multi-file `file:///` URI intellisense), and builds CodeMirror 6 (`@codemirror/view` + `Compartment` + inline diff/ghost-text decorations).

### 6. `kimi-visual-qa-critic`
- **Role:** `Vision-in-the-Loop Browser QA Critic`
- **Write Tools:** `true` | **MCP Tools:** `true` (Requires `chrome-devtools-mcp`)
- **When to Invoke:** Final verification gate. Opens the live Vite preview in Chrome DevTools MCP, checks console/network errors, runs the bounding-box & zero-dimension canvas detector, captures desktop (`1440x900`) and mobile (`390x844`) screenshots, and fixes any visual defects.

---

## Copy-Paste Orchestration Pattern (How the Main Agent Calls Them)

```text
1. Call `define_subagent` using the definitions in `subagent_definitions.json`.
2. Invoke `kimi-state-dag-architect` first to write `src/types/` and `src/store/` contracts.
3. Once contracts exist on disk, invoke `kimi-surface-motion-coder`, `kimi-webgl-graph-coder`, and `kimi-workbench-ide-coder` IN PARALLEL with non-overlapping file ownership:
   - Subagent A owns: `src/components/layout/*`, `src/components/ui/*`, `src/styles/*`
   - Subagent B owns: `src/components/canvas/*`, `src/components/graph/*`, `src/components/dag/*`
   - Subagent C owns: `src/components/workbench/*`, `src/components/editors/*`
4. Assemble `src/App.tsx`, run `npx tsgo --noEmit` (or `npx tsc --noEmit`), and invoke `kimi-visual-qa-critic` to verify the live render in Chrome DevTools.
```
