# react-three-fiber: the frame loop is sacred, React owns the graph
**Done means these pass:** `tsc --noEmit`, `eslint` (+ R3F/three rules where configured), `prettier --check .`.
## Rules
1. Never call React `setState` inside `useFrame`; mutate refs (`meshRef.current.position`) and read store state transiently via `getState()`.
2. Keep the render loop at 60fps by instancing: `InstancedMesh` for >150 repeated items; never one `<mesh>` per item at scale.
3. Load assets with `Suspense` (`useLoader` / `useGLTF`); gate the Canvas behind a loading boundary, never a blank frame.
4. Dispose GPU resources on unmount (`useGLTF.clear`, geometry/material dispose); GPU memory does not GC itself.
5. Animate only transforms (position/rotation/scale) and material uniforms via refs; avoid per-frame allocations in `useFrame`.
6. Size canvases to their container with explicit bounds; a zero-dimension canvas is a bug, not an empty state — assert non-zero in QA.
7. Colocate shaders as `shaderMaterial` uniforms mutated by ref; never rebuild material objects per frame.
8. Demand-framing (`frameloop="demand"`) for static scenes; invalidate explicitly on interaction.
9. Test with fixed-delta harnesses where possible; screenshot both viewports in QA (1440x900 + 390x844).
## Pitfalls
setState-in-frame causes re-render storms and frame drops; undisposed geometries leak GPU memory across route changes; `useLoader` outside Suspense throws; pointer events on instanced meshes need instanceId handling; DPR left unbounded melts mobile GPUs (cap `dpr={[1, 2]}`).
## Versions
Pinned: R3F v8/v9-era API with three.js current (exact project pin via package.json). This skill assumes at least R3F v8.
Rule: first read the project's pins (package.json @react-three/fiber, three, @react-three/drei versions). Write only for those versions; if the project pins older ones, follow the project, and if it pins nothing, add pins. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: R3F docs (Canvas, useFrame, useLoader), three.js manual (instancing, disposal).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/react-three-fiber/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/react-three-fiber.md` (reference; do not author).
