/// <reference types="vite/client" />

declare module '*.css';

/** Injected at build time by vite.config.ts `define`. Falls back to package.json version. */
declare const __APP_VERSION__: string;
declare const __COMMIT_SHA__: string;
declare const __BUILD_TIME__: string;

interface ImportMetaEnv {
  readonly MODE: string;
  readonly VITE_APP_NAME?: string;
  readonly VITE_APP_VERSION?: string;
  readonly VITE_COMMIT_SHA?: string;
  readonly VITE_SITE_URL?: string;
  readonly VITE_API_BASE_URL?: string;
  readonly VITE_TELEMETRY_ENABLED?: string;
  readonly VITE_TELEMETRY_ENDPOINT?: string;
  readonly VITE_LOG_LEVEL?: string;
}
