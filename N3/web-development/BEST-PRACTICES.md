# BEST-PRACTICES.md

Date: 2026-10-30. Status: PROPOSED.
Source basis: local `modern-web-guidance` catalog (CSS, design tokens, accessibility, motion, dark mode) + standard practice.

---

## Frontend Architecture

- **Predictable cascade over specificity hacks.** Declare layer order once, then let layers do the work. Within a layer, wrap optional filters in `:where()` so they don't inflate specificity.
- **Let CSS express state.** Style a parent from a child's state with `:has()` instead of toggling a class from JS. Do not nest `:has()` or place pseudo-elements inside it.
- **Use `:is()` / `:where()` for fallbacks, never duplicated rules.** Forgiving parsing means one rule covers both the native and polyfilled form.
- **Express intent with exclusions.** `button:hover:not(:disabled)` beats overriding `:disabled` afterward — the result is order-independent.
- **Scope by proximity when nesting matters.** `@scope (.card) to (.content)` handles nested cards correctly; `:not(.content *)` does not.
- **Native nesting for grouping; `@scope` for proximity.** Use nesting to group related rules, `@scope` when the closest ancestor should win.
- **No global reset.** Target specific elements and conditions. `* {}` cannot be overridden by web components or lower layers without `!important`.

## Design Tokens

- **Tier the tokens.** Literal → semantic → UI → component. Each tier builds on the last. Smaller projects need fewer tiers — do not overengineer.
- **Semantic tokens are the public contract.** `--color-accent` survives a rebrand; `--color-blue-500` does not. Components consume semantic tokens only.
- **Themes override semantic tokens**, not component tokens. A theme is a swap of the semantic layer; components stay untouched.
- **Use container style queries for density modes** so components react to their own container, not the viewport. Container style queries are Baseline since 2026-05-19 (Chrome 111, Safari 18, Firefox 151). NOT Baseline-wide yet → for core density switching, use a `data-density` attribute fallback with `:where()` and layer the style query on top as progressive enhancement.
- **Feature-detect in CSS with a sentinel custom property** (`--style-queries-supported: check`) rather than relying on the existence of `CSSContainerRule`.

## Theming

- Use `color-scheme` and `light-dark()` so browser-generated UI (scrollbars, form controls, selection) matches the theme.
- Use `color-scheme` on the element itself to force a subtree into a specific mode when a component (e.g. a code block or media player) must stay light.
- In forced-colors mode, rely on system colors rather than hardcoded palettes.

## Typography

- Use logical, relative units (`em`, `rem`) so text scales with user preferences.
- Respect the user's font-size and zoom; never clamp text in a way that prevents 200% zoom.
- Keep line length near 45–75 characters for body copy.

## Motion

- Animate `transform` and `opacity`; avoid animating layout properties (`width`, `height`, `top`) — they trigger layout work.
- Honor `prefers-reduced-motion: reduce` by shortening or removing non-essential motion.
- Keep durations short for micro-interactions and longer for larger transitions; use tokens for both.

## Responsive & Layout

- Size components to the space available, not to the viewport — use container queries for reusable components.
- Reserve space for media before it loads to avoid layout shift.
- Use logical properties so layouts survive RTL without a second stylesheet.

## Backend & API

- Validate at the edge, coerce deliberately, and reject with a stable error code clients can branch on.
- Keep controllers thin: parse → authorize → call service → serialize. Business logic lives in services.
- Make handlers side-effect-safe: retries must not double-charge or double-send.
- Return an envelope that includes a machine-readable `code`, a human `message`, and a `requestId` for tracing.
- Version contracts and write them down (OpenAPI). Generate types from the contract rather than hand-writing both.

## Data

- Forward-only, additive migrations; never edit an applied migration in place.
- Make migrations dual-compatible (expand/contract) so deploy and migrate can happen independently.
- Index for the access patterns you actually have; verify with `EXPLAIN`, not intuition.

## Testing

- Regression test first: reproduce, fix, then keep the test.
- Cover error, empty, and loading states — they are where users actually hit friction.
- Keep unit tests hermetic; push cross-service behavior to integration tests.

## Security

- Deny by default; grant explicitly.
- Rotate session IDs on any privilege change.
- Prefer short-lived, scoped tokens over long-lived opaque ones.
- Treat file uploads, redirects, and HTML rendering as untrusted input; sanitize at the sink.

## Performance

- Ship less JavaScript before optimizing it. Measure the bundle, then the interaction, then the paint.
- Set `fetchpriority="high"` on the LCP element and lazy-load below-the-fold media.
- Use `content-visibility` for long off-screen content.
- Preconnect only to origins you are certain you will use — every preconnect costs a connection.

---

## Anti-patterns (do not ship)

| Anti-pattern | Why it hurts | Instead |
|---|---|---|
| `!important` battles | Hides the real conflict; breaks cascade layers | Fix layer order or scope the rule |
| `* { margin: 0 }` reset | Unoverridable by components | Targeted element resets |
| BEM naming for specificity | Class-name soup | Layers + `:where()` |
| Inline `#06c` in a component | Unthemeable, unauditable | Semantic token |
| `aria-hidden` on focusable | Focus goes nowhere for AT | Remove the attribute |
| Retrying non-idempotent writes | Double charges/sends | Idempotency key |
| Hand-editing prod rows | Unreproducible, unrollbackable | Reviewed script + backup |
| Untested error path | Most bugs live here | Error + empty state tests |
