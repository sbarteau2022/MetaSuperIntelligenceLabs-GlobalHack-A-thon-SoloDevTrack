import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import { ErrorBoundary } from './components/layout/ErrorBoundary';
import { SkipLink } from './components/layout/Accessibility';
import { config, assertConfig } from './lib/config';
import { logger } from './lib/logger';
import { initWebVitals } from './lib/web-vitals';

// Fail-closed config check: loud in dev, quiet receipt in prod.
for (const w of assertConfig()) logger.warn('config', w);

// Single-shot chunk-recovery: if a lazy chunk 404s after a deploy
// (stale index.html → hashed asset gone), reload once with a cache-bust.
let chunkRetried = false;
try {
  chunkRetried = sessionStorage.getItem('atoma.chunk-retry') === '1';
} catch { /* private mode */ }
window.addEventListener('error', (e) => {
  const msg = String((e as ErrorEvent).message ?? '');
  if (/ChunkLoadError|Loading chunk|Failed to fetch dynamically imported module/i.test(msg)) {
    if (!chunkRetried) {
      chunkRetried = true;
      try {
        sessionStorage.setItem('atoma.chunk-retry', '1');
      } catch { /* private mode */ }
      logger.warn('chunk-retry', 'stale chunk detected — reloading once');
      window.location.reload();
    } else {
      logger.error('chunk-fatal', msg.slice(0, 200));
    }
  }
}, true);

// Passive Core Web Vitals (CLS/LCP/INP) — zero-dep, ~1KB.
initWebVitals();

// Unhandled rejection receipt — never silent in enterprise.
window.addEventListener('unhandledrejection', (e) => {
  logger.error('unhandledrejection', String(e.reason)?.slice(0, 300) ?? 'unknown');
});

// Build receipt in console + DOM hook for smoke tests.
logger.info(`boot ${config.appName} v${config.appVersion} (${config.commitSha}) [${config.env}]`);
try {
  document.documentElement.dataset.appVersion = config.appVersion;
  document.documentElement.dataset.appEnv = config.env;
} catch { /* noop */ }

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SkipLink />
    <ErrorBoundary lane="shell">
      <App />
    </ErrorBoundary>
  </StrictMode>,
);