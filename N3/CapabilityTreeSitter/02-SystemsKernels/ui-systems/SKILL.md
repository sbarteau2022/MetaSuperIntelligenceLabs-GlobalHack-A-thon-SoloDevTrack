---
name: ui-systems
description: Build and verify React, Next.js, Tailwind CSS v4, shadcn/ui, Tauri, GPUI, and WASM interfaces with accessibility and integration safeguards.
---

# UI systems

Read the local design system and application architecture before changing a component. Keep server/client boundaries explicit in Next.js and keep Tauri, GPUI, and WASM bridges typed and narrow.

For TypeScript, run typecheck, lint, tests, and build when scripts exist. For Tailwind/shadcn, use the existing project configuration; do not globally install or duplicate components. For Tauri, validate both the web layer and `src-tauri` Rust crate. For GPUI, validate the Rust crate and platform behavior separately.

Check: focus order, visible focus, keyboard escape, screen-reader labels, errors announced without color alone, narrow viewports, large text, loading/empty/error states, slow network, and reconnect/retry paths. Use the `e2e-browser` skill for an end-to-end journey.
