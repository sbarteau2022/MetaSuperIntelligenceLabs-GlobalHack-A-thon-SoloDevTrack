# vite: instant dev, explicit build
**Done means these pass:** `tsc --noEmit`, `vite build`, `vite preview --port 4173` smoke-check, `eslint`, `prettier --check .`.
## Rules
1. Keep `vite.config.ts` in `defineConfig` with plugins in order: framework plugin first (react/swc), then domain plugins; order matters because transform hooks run sequentially.
2. Static assets in `public/` are copied verbatim; imported assets (`import img from './x.png'`) get hashed URLs — pick deliberately, never both for the same file.
3. Env vars must be `VITE_`-prefixed to reach the client; read server-only secrets from plain `process.env` outside `import.meta.env`; never commit `.env` files with secrets.
4. `optimizeDeps.include` only for CJS deps that fail pre-bundling; prefer ESM-native deps and remove forced includes once upstream ships ESM.
5. Set explicit `build.target`, `outDir`, and `assetsInlineLimit`; verify `dist/` size budget in CI and fail the build over budget.
6. Use `import.meta.glob` for bulk imports (routes, content) instead of hand-maintained index files; prefer eager only where SSR/first-paint needs it.
7. Proxy API calls in `server.proxy` for dev; never hardcode localhost URLs in source — read from env per mode.
8. Split vendor chunks deliberately (`build.rollupOptions.output.manualChunks`) only after measuring; default splitting first, custom chunks on evidence.
9. Preview before ship: `vite preview` serves the production bundle locally so missing-env and base-path bugs surface before deploy; set `base` correctly for subpath deploys.
10. Keep `tsconfig` `moduleResolution: bundler` with Vite; `tsc` types are advisory — esbuild strips types without checking, so `tsc --noEmit` stays in the gate.
## Pitfalls
`import.meta.env` is statically replaced so dynamic key access (`env[name]`) yields undefined; top-level await in entry chunks breaks older targets; HMR state resets when a file's exports change shape; `optimizeDeps` staleness after adding a dep needs `--force`; sourcemaps off by default in build hides prod stack traces.
## Versions
Pinned: Vite v6/v7-era config shape (VERIFIED vite 8.3.5 npm-latest 2026-10-30; node v26.9.0 installed on this machine). This skill assumes at least Vite v5.
Rule: first run `node --version` and `npx vite --version`, and read the project's pin (package.json devDependencies). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Vite docs (config, env, build, optimizeDeps, preview).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/vite/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/vite.md` (reference; do not author).
