# monaco: models are data, views are disposable
**Done means these pass:** `tsc --noEmit`, `eslint`, `prettier --check .`.
## Rules
1. One `ITextModel` per file identity (`monaco.Uri.file(path)`); many views may share a model, models outlive views.
2. Dispose views on unmount (`editor.dispose()`); dispose models only when the file identity is gone, or models leak.
3. Run language workers through the configured `MonacoEnvironment.getWorker`; never block the UI thread with synchronous tokenize/parse.
4. Set `automaticLayout: true` only when the container resizes without notification; otherwise call `layout()` explicitly to avoid layout thrash.
5. Normalise EOL (`model.getEOL()` / `setEOL`) at load; mixed endings corrupt diff ranges and inline decorations.
6. Register completions/hover/diagnostics via language providers scoped to the file's language id; deregister on teardown.
7. Keep editor options versioned in one config object (font, minimap, sticky scroll); diff configs in review, not in defaults.
8. Never store secrets in model contents; models persist in memory and in worker messages.
## Pitfalls
Two models for the same URI desync edits; undisposed views accumulate web workers; `setValue` resets undo history (use `pushEditOperations` for programmatic edits that must stay undoable); AMD loader double-initialisation when bundler + CDN copies mix.
## Versions
Pinned: Monaco 0.5x-era API (models, workers, providers; exact project pin via package.json). This skill assumes at least Monaco 0.34.
Rule: first read the project's pin (package.json monaco-editor version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Monaco Editor API docs (model/view lifecycle, workers, providers).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/monaco/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/monaco.md` (reference; do not author).
