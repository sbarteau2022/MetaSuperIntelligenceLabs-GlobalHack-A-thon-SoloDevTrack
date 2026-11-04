import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

function appVersion(): string {
  try {
    const raw = readFileSync(new URL('./package.json', import.meta.url), 'utf8');
    const pkg = JSON.parse(raw) as { version?: string };
    return process.env.VITE_APP_VERSION || pkg.version || '0.0.0';
  } catch {
    return process.env.VITE_APP_VERSION || '0.0.0';
  }
}

const version = appVersion();
const commit = (process.env.CF_PAGES_COMMIT_SHA || process.env.VITE_COMMIT_SHA || 'local').slice(0, 8);
const buildTime = new Date().toISOString();

export default defineConfig({
  base: '/',
  publicDir: 'public',
  plugins: [react(), tailwindcss()],
  server: {
    fs: {
      allow: [
        process.cwd(),
        '/Users/stewartbarteau/Desktop/framer file/atoma-labs-web',
      ],
      strict: false,
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(version),
    __COMMIT_SHA__: JSON.stringify(commit),
    __BUILD_TIME__: JSON.stringify(buildTime),
  },
  build: {
    target: 'baseline-widely-available',
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    cssCodeSplit: true,
    cssMinify: true,
    minify: true,
    reportCompressedSize: true,
    chunkSizeWarningLimit: 500,
    assetsInlineLimit: 4096,
    modulePreload: { polyfill: false },
    rolldownOptions: {
      output: {
        // Long-lived vendor chunks land in /assets/* (immutable on CF).
        // three.js is ONLY reachable via lazy chunks (HeroCanvas, sculptures,
        // ForceGraph3D) so it never blocks first paint.
        codeSplitting: {
          groups: [
            {
              name: 'vendor-react',
              test: /node_modules[\\/](react|react-dom|scheduler|zustand)/,
            },
            {
              name: 'vendor-three',
              test: /node_modules[\\/](three|@react-three)/,
            },
            {
              name: 'vendor-xyflow',
              test: /node_modules[\\/]@xyflow/,
            },
          ],
        },
      },
    },
  },
})
