---
name: ide-dockview-editors
description: High-fidelity workbench and code editor integration skill for Dockview (dockview-react), Monaco Editor (@monaco-editor/react + Shiki + Vite Web Workers + multi-model URI filesystem), and CodeMirror 6 (@codemirror/view + Compartments + inline AI diff decorations).
---

# ide-dockview-editors (interface)

## Purpose
Desktop-class multi-pane workbenches: Dockview layout + persistence, Monaco with
Vite workers + multi-model `file:///` filesystem, CodeMirror 6 compartment
reconfiguration with zero state loss, inline AI diff decorations.

## When to use
Split-view IDEs, shader sandboxes, node-inspector panels, AI code diff editors
built on Dockview / Monaco / CodeMirror 6.

## Out of scope
Importing Dockview CSS is mandatory (`dockview-react/dist/styles/dockview.css`),
not optional. Destroying/recreating `EditorView` for readOnly/language/theme
toggles (use Compartments). Running TS/JSON/CSS language services on the UI thread
(use `MonacoEnvironment` workers).

## Type signature
Compiled gate: `tool.rs` (`workbench-gate`, schema: `tool.json`).
- `css_present(dockview_css: boolean) -> bool`
- `editor_valid(editor_reused: boolean) -> bool`
- `layout_valid(layout_mode: String) -> bool` — "responsive" | "fixed-workbench"
- `workbench_gate(cfg: WorkbenchConfig) -> bool` — conjunction of the above
- `make_workbench(dockview_css, editor_reused, layout_mode) -> WorkbenchConfig`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/ide-dockview-editors/SKILL.md`
(`cf845b265bde0056881391f351f04e8aabd92408e316f3fe7b27ce`). Shrunk to this
interface 2026-10-31 (Agent A2). No tool/server claims in source; nothing removed.
See `RECEIPT.md`.
