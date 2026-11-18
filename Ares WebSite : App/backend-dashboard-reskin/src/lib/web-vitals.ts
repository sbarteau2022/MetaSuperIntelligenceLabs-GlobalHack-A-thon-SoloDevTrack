import { config } from './config';
import { logger } from './logger';

export interface WebVital {
  name: 'CLS' | 'LCP' | 'INP' | 'FCP' | 'TTFB';
  value: number;
  rating: 'good' | 'needs-improvement' | 'poor';
}

/** Budget gates (Google CWV thresholds). CI + runtime share these numbers. */
export const VITALS_BUDGETS: Record<WebVital['name'], [number, number]> = {
  LCP: [2500, 4000],
  INP: [200, 500],
  CLS: [0.1, 0.25],
  FCP: [1800, 3000],
  TTFB: [800, 1800],
};

function rate(name: WebVital['name'], value: number): WebVital['rating'] {
  const [good, poor] = VITALS_BUDGETS[name];
  if (value <= good) return 'good';
  if (value <= poor) return 'needs-improvement';
  return 'poor';
}

function report(v: WebVital) {
  logger.event('web-vital', { name: v.name, value: Math.round(v.value * 100) / 100, rating: v.rating });
  // Keep a DOM hook for e2e / smoke assertions.
  try {
    document.documentElement.dataset[`vital${v.name}`] = String(Math.round(v.value));
  } catch {
    /* noop */
  }
}

/**
 * Passive Core Web Vitals observer. Zero deps, ~1KB.
 * Safe to call once from main.tsx — no-ops when PerformanceObserver missing.
 */
export function initWebVitals(): () => void {
  if (typeof window === 'undefined' || !('PerformanceObserver' in window)) return () => {};
  if (config.env !== 'production' && !config.telemetryEnabled) {
    // Still observe in dev so engineers see values in console via logger.event debug.
  }
  const cleanups: Array<() => void> = [];
  try {
    // LCP
    const lcp = new PerformanceObserver((list) => {
      const last = list.getEntries().at(-1) as PerformanceEntry & { startTime: number } | undefined;
      if (!last) return;
      report({ name: 'LCP', value: last.startTime, rating: rate('LCP', last.startTime) });
    });
    lcp.observe({ type: 'largest-contentful-paint', buffered: true });
    cleanups.push(() => lcp.disconnect());
  } catch { /* older browsers */ }
  try {
    // CLS (session-window sum, simplified cumulative)
    let cls = 0;
    const o = new PerformanceObserver((list) => {
      for (const e of list.getEntries() as Array<PerformanceEntry & { value?: number; hadRecentInput?: boolean }>) {
        if (e.hadRecentInput) continue;
        cls += e.value ?? 0;
      }
      report({ name: 'CLS', value: cls, rating: rate('CLS', cls) });
    });
    o.observe({ type: 'layout-shift', buffered: true });
    cleanups.push(() => o.disconnect());
  } catch { /* noop */ }
  try {
    // INP via event timing
    const inp = new PerformanceObserver((list) => {
      let worst = 0;
      for (const e of list.getEntries() as Array<PerformanceEntry & { duration: number }>) {
        worst = Math.max(worst, e.duration);
      }
      if (worst > 0) report({ name: 'INP', value: worst, rating: rate('INP', worst) });
    });
    inp.observe({ type: 'event', buffered: true, durationThreshold: 16 } as PerformanceObserverInit);
    cleanups.push(() => inp.disconnect());
  } catch { /* noop */ }
  return () => cleanups.forEach((fn) => fn());
}
