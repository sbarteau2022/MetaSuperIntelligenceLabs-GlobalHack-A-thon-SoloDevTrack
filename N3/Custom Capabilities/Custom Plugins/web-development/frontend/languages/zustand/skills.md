# zustand: small stores, no prop-drilling, no re-render storms
**Done means these pass:** `tsc --noEmit`, `eslint`, `prettier --check .`, `vitest run` (store unit tests).
## Rules
1. Use the curried creator: `create<T>()((set, get) => ({...}))` so state is fully typed; never `create<T>({...})` object-literal form.
2. Select narrowly in components: `useStore(s => s.count)`; for object/array snapshots wrap the selector with `useShallow` to avoid infinite re-render loops.
3. Never select the whole store (`useStore(s => s)`) in a component; expose granular selectors or per-slice hooks.
4. Keep actions inside the store (`set`/`get`); components dispatch `useStore(s => s.addTodo)`, never mutate state directly.
5. Use transient reads (`useStore.getState()`) in event handlers, loops, and intervals instead of subscribing a component that does not render the value.
6. Keep stores flat and domain-scoped (one store per bounded context); compose via importing another store's `getState()`, not by nesting providers.
7. Async actions set `loading`/`error` explicitly with try/finally; never leave a store stuck in loading on failure.
8. Persist only what you must (`persist` partialize + version + migrate); never persist tokens or derived data.
9. Derive in selectors, not in state: `total: s => s.items.reduce(...)` computed at read time; never store a value that can go stale.
10. No middleware bloat: no `redux-devtools`-style action logs in prod, no immer unless nested updates genuinely hurt readability; vanilla `set` first.
## Pitfalls
Returning a fresh object literal from a selector re-renders every dispatch unless `useShallow` wraps it; `create` without the curried call loses middleware typing; `persist` rehydration flashes default state before merge so gate first paint on `_hasHydrated`; `subscribeWithSelector` fires on every set unless given an equality fn; storing Promises or class instances breaks serialisation and devtools.
## Versions
Pinned: Zustand v5 (curried `create<T>()(...)` + `useShallow` from `zustand/shallow`; node v26.9.0 installed, tsc VERIFIED 7.0.2 npm-global on this machine 2026-10-30). This skill assumes at least Zustand v4.
Rule: first run `node --version` and read the project's pin (package.json zustand version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Zustand docs (v5 API: create, useShallow, persist, subscribeWithSelector).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/zustand/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/zustand.md` (reference; do not author).
