# web design and front-end best practices
Semantic HTML first (landmarks, headings in order, real buttons and links). WCAG 2.2 AA: contrast 4.5:1 text / 3:1 UI, visible
focus, full keyboard path, labels on inputs, alt text, respect prefers-reduced-motion and prefers-color-scheme. Mobile-first
responsive layout with CSS grid/flex; fluid type with a modular scale (1.2-1.25); 4/8px spacing grid; design tokens as CSS
variables. Performance budget: LCP < 2.5s, INP < 200ms, CLS < 0.1; ship less JS, lazy-load below the fold, size images,
preload the critical font. State has one owner; forms validate inline and keep user input on error. Test in a real browser at
360, 768 and 1280 px, and with a screen reader and keyboard only. Content Security Policy on; no inline script.
