# codemirror: extensions are data, state is immutable
**Done means these pass:** `tsc --noEmit`, `eslint`, `prettier --check .`, `vitest run` (state/transaction tests).
## Rules
1. Model documents as `EditorState` + dispatched `Transaction`s; never mutate the view's DOM directly.
2. Reconfigure live behaviour with `Compartment`s (`language`, `tabSize`, placeholders); do not rebuild the `EditorView`.
3. Compose behaviour from extensions (`basicSetup`, language packs, lint, autocomplete); write custom behaviour as `StateField`/`ViewPlugin`, not as framework components.
4. Keep the editor framework-agnostic: one `EditorView` instance per mount, created in an effect, destroyed on unmount; never recreate on keystroke.
5. Debounce expensive work (lint, parse, large decorations) out of the transaction path; decorations via `DecorationSet` mapped through changes.
6. Accessibility: keep the hidden textarea semantics intact, expose line numbers optionally, honour `prefers-reduced-motion` for animated panels.
7. Pin `@codemirror/*` versions together; mixed major versions of state/view/lang packages break at runtime, not at typecheck.
8. Test transactions, not rendering: assert doc + selection + effects after dispatch; use headless state tests before browser tests.
## Pitfalls
Creating a new `EditorView` per render leaks editors and duplicates keymaps; `updateListener` that dispatches unconditionally loops; stale compartments (reconfigure forgotten) silently keep old tab sizes; mixing v5 (`CodeMirror.fromTextArea`) APIs with v6 packages fails silently.
## Versions
Pinned: CodeMirror 6.x extension set (verified pattern; exact project pin via package.json). This skill assumes at least CodeMirror 6.0.
Rule: first run `npm ls @codemirror/state` and read the project's pin. Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: CodeMirror 6 guides (state, view, compartments, decorations).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/codemirror/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/codemirror.md` (reference; do not author).
