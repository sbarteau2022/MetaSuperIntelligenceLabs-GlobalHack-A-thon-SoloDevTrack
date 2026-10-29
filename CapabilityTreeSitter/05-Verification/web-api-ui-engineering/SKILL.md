---
name: web-api-ui-engineering
description: Build and pressure-test web applications, APIs, and UI systems with the RAPID Qwen profile.
---

# Web, API, and UI Engineering

Use this skill for HTML, React, TypeScript, JavaScript, API, Tailwind CSS v4, shadcn/ui, and browser-facing work.

## Workflow

1. Read project instructions, package manifests, lockfiles, and existing patterns before editing.
2. Use the language-toolchain MCP to discover available tooling and read project conventions.
3. Keep API boundaries typed. Validate untrusted input at the boundary and never place credentials, tokens, or private URLs in source, logs, or prompts.
4. Prefer semantic HTML, keyboard-operable controls, visible focus, labels, and accessible error states.
5. For Tailwind CSS v4, preserve the project token system and responsive conventions. For shadcn/ui, compose project-owned components rather than editing generated primitives without a reason.
6. Run the smallest relevant checks first, then typecheck, lint, tests, and production build. State what actually ran and what did not.

## Pressure-Test Checklist

- API: success, malformed input, auth failure, authorization failure, rate/timeout behavior, and retry-safe mutations.
- UI: loading, empty, error, long text, narrow and wide layouts, keyboard navigation, focus restoration, and destructive-action confirmation.
- Integration: schema changes, network failures, duplicate requests, cancellation, and stale responses.
- Before claiming completion: inspect diagnostics, exercise the main path, and record any untested browser/device/cloud dependency.

## Tool Use

- Use Zed file, search, diagnostics, terminal, and language-toolchain tools for local work.
- Use fetch or web search only when external information is necessary. External calls remain confirmation-gated.
- Browser automation is required for visual or interaction claims; do not infer it from a successful build. Add a project-local Playwright or equivalent runner when the project needs it.
