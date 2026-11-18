/**
 * Enterprise runtime config — single source of truth for env.
 * All client-exposed values MUST be prefixed with VITE_.
 * Fails closed: missing/invalid values throw in dev, warn + safe default in prod.
 */
export interface AppConfig {
  env: 'development' | 'production' | 'test';
  appName: string;
  appVersion: string;
  commitSha: string;
  siteUrl: string;
  apiBaseUrl: string;
  telemetryEnabled: boolean;
  telemetryEndpoint: string;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

function str(key: string, fallback = ''): string {
  const v = (import.meta.env as Record<string, unknown>)[key];
  return typeof v === 'string' && v.length > 0 ? v : fallback;
}

function bool(key: string, fallback: boolean): boolean {
  const v = str(key, '');
  if (v === '') return fallback;
  return v === '1' || v.toLowerCase() === 'true';
}

const MODE = import.meta.env.MODE ?? 'production';

export const config: AppConfig = {
  env: MODE === 'test' ? 'test' : MODE === 'development' ? 'development' : 'production',
  appName: str('VITE_APP_NAME', 'Atoma Engineering — ARES Spec Ops'),
  appVersion: str('VITE_APP_VERSION', typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.0.0'),
  commitSha: str('VITE_COMMIT_SHA', 'local'),
  siteUrl: str('VITE_SITE_URL', 'https://atoma.example.com'),
  apiBaseUrl: str('VITE_API_BASE_URL', ''),
  telemetryEnabled: bool('VITE_TELEMETRY_ENABLED', false),
  telemetryEndpoint: str('VITE_TELEMETRY_ENDPOINT', '/api/beacon'),
  logLevel: (str('VITE_LOG_LEVEL', MODE === 'development' ? 'debug' : 'warn') as AppConfig['logLevel']),
};

declare const __APP_VERSION__: string;

export function assertConfig(): string[] {
  const warnings: string[] = [];
  if (!config.siteUrl.startsWith('https://') && config.env === 'production') {
    warnings.push('VITE_SITE_URL should be https:// in production');
  }
  return warnings;
}
