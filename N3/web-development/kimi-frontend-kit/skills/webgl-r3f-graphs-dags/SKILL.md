---
name: webgl-r3f-graphs-dags
description: Production architectures for React Three Fiber (R3F v9) + WebGL/WebGL2 GLSL shaders + InstancedMesh, Obsidian-style Force-Directed Knowledge Graphs (d3-force + Canvas2D/WebGL + Quadtree + Semantic Zoom LOD + Ego-Network dimming), and Interactive DAG Workflow Renderers (@xyflow/react v12 + elkjs auto-layout + animated SVG particle edges).
---

# webgl-r3f-graphs-dags (interface)

## Purpose
Production 60–120fps R3F v9 architectures, Obsidian-style force-directed knowledge
graphs (d3-force, quadtree neighbor map, semantic-zoom LOD, 1-hop ego dimming),
and `@xyflow/react` v12 DAG renderers with elkjs layout and particle edges.

## When to use
3D/WebGL canvases, custom shaders, knowledge graphs above trivial size, node-based
DAG workflow editors.

## Out of scope
React `setState` inside `useFrame` (destroys frame pacing — use refs/transients).
Rendering >200 force-directed nodes as raw DOM `<svg>` (use HiDPI canvas/WebGL).
CPU-per-frame neighbor scans (precompute the lookup map on link change). Non
`transform`/`opacity` animation in hot paths.

## Type signature
Compiled gate: `tool.rs` (`r3f-frame-gate`, schema: `tool.json`).
- `frame_state_valid(set_state_in_frame: boolean) -> bool` — must be false
- `gpu_prop_valid(animated_prop: String) -> bool` — "transform" | "opacity"
- `graph_mode_valid(node_count: number, node_mode: String) -> bool` — >200 requires "canvas-webgl"
- `r3f_gate(cfg: R3fFrame) -> bool` — conjunction of the above
- `make_r3f(set_state_in_frame, animated_prop, node_count, node_mode) -> R3fFrame`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/webgl-r3f-graphs-dags/SKILL.md`
(`2f8e12181cabfcc81da85f1e1876ede86da85a52666`). Shrunk to this interface
2026-10-31 (Agent A2). `ResizeObserver` is a browser API, not a tool claim; nothing
removed. See `RECEIPT.md`.
