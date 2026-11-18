# landing-page-checklist

A 5-item checklist for shipping a landing page. Derived from the web-design skill.

1. **Semantic, accessible structure** — Landmarks and headings in order, real `<button>`/`<a>` elements, labels on inputs, alt text; WCAG 2.2 AA (contrast 4.5:1 text / 3:1 UI, visible focus, full keyboard path).
2. **Mobile-first responsive layout** — CSS grid/flex, fluid type on a modular scale (1.2–1.25), 4/8px spacing grid, design tokens as CSS variables; verify at 360, 768 and 1280px.
3. **Performance budget** — LCP < 2.5s, INP < 200ms, CLS < 0.1; ship less JS, lazy-load below the fold, size images, preload the critical font.
4. **One owner for state, resilient forms** — Single source of truth; forms validate inline and preserve user input on error; respect `prefers-reduced-motion` and `prefers-color-scheme`.
5. **Security + verification** — Content Security Policy on, no inline script; test in a real browser with a screen reader and keyboard only before launch.
