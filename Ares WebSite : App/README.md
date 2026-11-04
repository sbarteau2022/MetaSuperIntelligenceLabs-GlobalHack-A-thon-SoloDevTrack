# Ares Website — Atoma Labs marketing site (Framer Dispatch export + Atoma overlay)

The live site is the extracted Framer export (`browser-download-*framer.app-full/`),
served as-is with an Atoma overlay: marketing copy from
`atoma-marketing-copy-dispatch-layout-2026-11-04.pdf`, trusted-by logos in
`logos/`, and the Atoma 1–7 stack section. No layout rebuild — template
structure, motion, and styling are untouched. Overlay scripts (one-shot, in
`/tmp/atoma-*.pl`, not rerunnable): text swaps, removal of unverified
claims/cards/badges/dates, title/meta. Pristine backup: the export `.zip`.
Stack section is v2 blueprint plate (`/tmp/atoma-stack2.pl`, textual splice):
7 wireframe SVG marks on a spine with leader dots, reference subtitles, footer
strip, scroll-driven entry reveals with reduced-motion guard. Never
re-serialize `index.html` with HTML::TreeBuilder (drops nodes, mangles
entities) — textual surgery only.

Serve (static, no transform — the export ships binary `.js` data blobs that
Vite dev tries to parse): `./node_modules/.bin/vite preview --outDir
"./browser-download-20261104T104256.365566343Z-1-gentle-situation-101376.framer.app-full"
--host 127.0.0.1 --port 5173` → http://127.0.0.1:5173/
Do NOT serve the export folder with `vite` dev (oxc overlay on the blobs);
an earlier dev-only middleware workaround was removed for exactly that
reason — fewer moving parts.

## Earlier React build (not served; src/ kept intact)

Single-page marketing site for Atoma Labs / Ares–SpecOps. Copy source:
`atoma-marketing-copy-dispatch-layout-2026-11-04.pdf` (Dispatch layout).
Only verified copy is published — every `[FILL]` slot from the source doc is
omitted from the page until real figures, quotes, links, or assets exist.

Dev: `./node_modules/.bin/vite --host 127.0.0.1 --port 5173` →
http://127.0.0.1:5173/

## Template notes (React + TypeScript + Vite)

This template provides a minimal setup to get React working in Vite with HMR and some Oxlint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Oxc](https://oxc.rs)
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/)

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the Oxlint configuration

If you are developing a production application, we recommend enabling type-aware lint rules by installing `oxlint-tsgolint` and editing `.oxlintrc.json`:

```json
{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["react", "typescript", "oxc"],
  "options": {
    "typeAware": true
  },
  "rules": {
    "react/rules-of-hooks": "error",
    "react/only-export-components": ["warn", { "allowConstantExport": true }]
  }
}
```

See the [Oxlint rules documentation](https://oxc.rs/docs/guide/usage/linter/rules) for the full list of rules and categories.
