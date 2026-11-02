# dockview: layouts persist, panels are disposable
**Done means these pass:** `tsc --noEmit`, `eslint`, `prettier --check .`.
## Rules
1. Persist the `DockviewApi` layout JSON on change (debounced); restore on load before mounting panel content.
2. Panels are created from a factory registry (id -> component); never hardcode panel instances in layout code.
3. Scope panel state to the panel (params + local state); global state lives outside the layout, referenced by id.
4. Dispose panel resources on `onDidRemovePanel`; leaked subscriptions survive layout restores.
5. Version the persisted layout schema; migrate or discard on mismatch, never crash on an old shape.
6. Keep group constraints explicit (min sizes, locked groups); test restore at 1440x900 AND 390x844.
7. Keyboard: panels must be reachable and closable without a pointer; visible focus on the active group.
## Pitfalls
Restoring content before layout completes orphans panels; factory id drift (rename without migration) loses user layouts; unthrottled persist-on-change thrashes storage; two Dockview roots sharing a storage key clobber each other.
## Versions
Pinned: Dockview v4-era API (exact project pin via package.json). This skill assumes at least Dockview v3.
Rule: first run `npm ls dockview` and read the project's pin. Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Dockview docs (api, persistence, groups).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/dockview/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/dockview.md` (reference; do not author).
