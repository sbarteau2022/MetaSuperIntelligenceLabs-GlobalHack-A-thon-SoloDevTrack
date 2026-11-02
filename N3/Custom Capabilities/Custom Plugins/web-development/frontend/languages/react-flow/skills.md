# react-flow: nodes and edges are state, the viewport is a camera
**Done means these pass:** `tsc --noEmit`, `eslint`, `prettier --check .`.
## Rules
1. Memoize `nodeTypes`/`edgeTypes` (define outside render or `useMemo`); inline type maps remount every node on every render.
2. Keep `nodes`/`edges` in state with `applyNodeChanges`/`applyEdgeChanges`; never mutate the arrays in place.
3. Large graphs: hide detail behind zoom (`minZoom`/`maxZoom`), use `onlyRenderVisibleElements`, and cap animated edges (SVG particles are per-edge DOM).
4. Layout with elkjs (`layered` algorithm) off the render path; compute positions in a worker or effect, then set nodes once.
5. Custom nodes are small, pure components; no data fetching inside a node — lift data up, pass it down.
6. Persist viewport + selection separately from graph data; restore camera after data load, not before.
7. Keyboard-delete and aria-labels on custom nodes; canvas graphs are hostile to AT without explicit labelling.
8. Pin `@xyflow/react` major; v11/v12 APIs differ (pro options, edge internals) — read the installed version first.
## Pitfalls
Unmemoized nodeTypes cause full-graph remounts and lost selection; thousands of animated edges freeze the tab; elkjs on the main thread janks large DAGs; controlled `viewport` without `onMoveEnd` fights the user's pan.
## Versions
Pinned: @xyflow/react v12-era with elkjs (exact project pin via package.json). This skill assumes at least v11.
Rule: first run `npm ls @xyflow/react elkjs` and read the project's pin. Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: XYFlow docs (nodes/edges/viewport), elkjs layered algorithm docs.
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/react-flow/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/react-flow.md` (reference; do not author).
