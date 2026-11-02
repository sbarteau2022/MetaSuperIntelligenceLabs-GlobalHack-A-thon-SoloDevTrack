# Cloudflare Pages — atoma-labs-web

## Prereqs
- Node 20+, npm, `npx wrangler` access to the Cloudflare account.
- `node_modules` resolves via symlink in this workspace; on CI just `npm install`.

## Build
```
npm run build
```
Output: `dist/` (Vite SPA, hash routes `#/platform`, SPA fallback via `public/_redirects`).

## Verify (enterprise gate — run before every ship)
```
npm run verify
```
Runs in order: `typecheck` → `lint` (0 errors) → `vite build` →
`stamp:version` (writes `dist/version.json`) → `audit:headers` (CSP/HSTS/COOP
contract) → `audit:bundle` (critical-path budgets, three.js isolation).
Any failure stops the lane. CI (`.github/workflows/verify.yml`) runs the same
sequence on push/PR.

### What's tuned (competition build)
- **Critical path is tiny**: initial HTML pulls only `index.*.js` (~41 KB raw /
  ~13 KB gzip) + `vendor-react` (~225 KB / ~70 KB gzip) + CSS. No `three.js`,
  no `@xyflow`, no animation libs block first paint.
- **Lazy everything heavy**: every page shell (`AtomaPages`), the cloud
  workspace (`AtomaCloudApp`), `HeroCanvas`, all R3F sculptures, `ForceGraph3D`,
  sim labs, and `SwarmDAG` load via `React.lazy` + `Suspense` with skeleton
  fallbacks. `three.js` (~913 KB / ~241 KB gzip) and `@xyflow` (~175 KB /
  ~56 KB gzip) only download when the user opens a tab that needs them.
- **Zero animation deps**: `motion` + `gsap`/`ScrollTrigger` removed from
  `package.json`. Hero entrance, page transitions, and `[data-reveal]`
  scroll reveals are pure CSS keyframes + `IntersectionObserver`
  (`src/lib/reveal.ts`) with `prefers-reduced-motion` support.
- **Fonts: latin-only, self-hosted**: `@fontsource` latin 400–700
  (Space Grotesk) + latin 400/500 (JetBrains Mono). No Google Fonts request;
  full-subset + weight-300 files dropped (~10 subset files gone, ~400 KB
  saved). `.woff2` served first by bundler order.
- **WebGL diet**: all `Canvas` capped at `dpr={[1, 1.5]}`,
  `antialias: false`, `powerPreference: 'high-performance'`.
  Brand emblem `<img>` is `loading="lazy" decoding="async"`.
- **Images**: `brand_QGh3P.jpg` resized 784→~640px wide (160 KB → ~75 KB),
  served immutable. Dead `src/assets/*` + `App.css` removed. Favicon inlined
  as data-URI (no `/vite.svg` 404).
- **Headers**: immutable 1y on `/assets/*` + brand JPG; `must-revalidate` on
  `index.html`; `X-Frame-Options: DENY`, `nosniff`, strict referrer,
  `Permissions-Policy` lockdown.
- **SEO**: description / OG / theme-color meta in `index.html`.

## Enterprise hardening (2026-11-01 pass)
- **Resilience**: `ErrorBoundary` per lane (shell, page, cloud-app, judge-bar) —
  a crashed lazy chunk shows a retry card, never a blank OS. Single-shot
  chunk-retry in `main.tsx` recovers stale deploys (old HTML → new hash 404).
- **Routing contract**: hash routes `#/platform … #/cloud-app` — shareable,
  back/forward-safe, unknown hashes render a 404 lane (`NotFound`) instead of
  silently landing on platform. Store stays source of truth.
- **Observability**: `src/lib/config` (env contract, `VITE_` only) +
  `src/lib/logger` (level-gated, secret-redacting) + `src/lib/web-vitals`
  (passive CLS/LCP/INP, budgets in code) + `document.dataset.appVersion` boot
  receipt + `dist/version.json` health proof.
- **A11y gates**: skip-link, `#main` focus on route change, `aria-current="page"`,
  dialog semantics + focus-trap + scroll-lock on the command palette,
  per-route SEO via `src/lib/seo.ts` (cloud/sign-in `noindex`).
- **Edge security**: CSP (`frame-ancestors 'none'`, tight `connect-src`),
  HSTS preload, COOP/COEP same-origin, `Origin-Agent-Cluster` in
  `public/_headers` (audited by `scripts/audit-headers.mjs`).
- **SEO/proof surface**: `robots.txt`, `sitemap.xml`, `manifest.webmanifest`,
  `.well-known/security.txt`, canonical + JSON-LD + twitter meta in `index.html`.
- **Repo governance**: `.nvmrc` (20), `.editorconfig`, `.env.example`,
  `.gitignore`, `engines` pin, `verify` script, CI workflow.

## Deploy (manual)
```
npx wrangler pages deploy dist --project-name atoma-labs-web
```

## Custom domain
1. Pages project → Custom domains → Set up (e.g. `atoma.example.com`).
2. Add the CNAME/validation records Cloudflare shows at your DNS provider (auto if zone is on Cloudflare).
3. Wait for cert Active; SPA fallback handled by `public/_redirects` (`/* /index.html 200`).

## Env notes
- No secrets required for this static build. If env vars are added later, set them in Pages → Settings → Environment variables (Preview + Production separately) and prefix client-exposed keys with `VITE_`.

## Rollback
- Pages → Deployments → select prior good deployment → Rollback / Retry deployment. No live deploy was performed by this change.
