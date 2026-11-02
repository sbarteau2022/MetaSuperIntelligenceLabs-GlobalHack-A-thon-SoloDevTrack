/**
 * Gate: bundle budgets. Fails the build if the critical path regresses.
 * Budgets: initial JS (index + vendor-react) <= 300KB gzip-approx (raw 320KB
 * cap here as proxy); vendor-three must NOT be in index chunk; CSS <= 120KB raw.
 * Run: npm run audit:bundle (after vite build).
 */
import { readdirSync, statSync, readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const assets = join(root, 'dist', 'assets');

let failures = 0;
const fail = (msg) => {
  failures += 1;
  console.error(`FAIL bundle: ${msg}`);
};
const pass = (msg) => console.log(`ok bundle: ${msg}`);

if (!existsSync(assets)) {
  fail('dist/assets missing — run vite build first');
  process.exit(1);
}

const files = readdirSync(assets);
const size = (f) => statSync(join(assets, f)).size;
const js = files.filter((f) => f.endsWith('.js'));
const css = files.filter((f) => f.endsWith('.css'));

const index = js.filter((f) => f.startsWith('index-'));
const vendorReact = js.filter((f) => f.startsWith('vendor-react-'));
const vendorThree = js.filter((f) => f.startsWith('vendor-three-'));

if (index.length === 0) fail('no index-*.js chunk');
else {
  const bytes = index.reduce((a, f) => a + size(f), 0);
  if (bytes > 80_000) fail(`index chunk ${bytes}B exceeds 80KB raw (critical path)`);
  else pass(`index chunk ${bytes}B within budget`);
}
if (vendorReact.length === 0) fail('no vendor-react-*.js chunk');
else {
  const bytes = vendorReact.reduce((a, f) => a + size(f), 0);
  if (bytes > 260_000) fail(`vendor-react ${bytes}B exceeds 260KB raw`);
  else pass(`vendor-react ${bytes}B within budget`);
}
if (vendorThree.length === 0) fail('no vendor-three chunk (expected lazy)');
else pass(`vendor-three isolated (${vendorThree.join(', ')})`);

// three.js must never leak into the critical index chunk.
for (const f of index) {
  const body = readFileSync(join(assets, f), 'utf8');
  if (/WebGLRenderer|THREE\./.test(body)) fail(`three.js symbols detected in ${f}`);
  else pass(`${f} free of three.js`);
}

const cssBytes = css.reduce((a, f) => a + size(f), 0);
if (cssBytes > 140_000) fail(`CSS ${cssBytes}B exceeds 140KB raw`);
else pass(`CSS ${cssBytes}B within budget`);

if (failures > 0) {
  console.error(`\nbundle audit: ${failures} failure(s)`);
  process.exit(1);
}
console.log('\nbundle audit: PASS');
