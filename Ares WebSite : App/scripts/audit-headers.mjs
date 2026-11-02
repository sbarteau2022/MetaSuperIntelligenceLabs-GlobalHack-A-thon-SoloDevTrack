/**
 * Gate: _headers must carry the enterprise security contract.
 * Run: npm run audit:headers — exits non-zero on any violation.
 */
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const headersPath = join(root, 'public', '_headers');
const redirectsPath = join(root, 'public', '_redirects');

let failures = 0;
const fail = (msg) => {
  failures += 1;
  console.error(`FAIL headers: ${msg}`);
};
const pass = (msg) => console.log(`ok headers: ${msg}`);

if (!existsSync(headersPath)) {
  fail('public/_headers missing');
  process.exit(1);
}
const headers = readFileSync(headersPath, 'utf8');
for (const required of [
  'Strict-Transport-Security',
  'Content-Security-Policy',
  'X-Frame-Options: DENY',
  'X-Content-Type-Options: nosniff',
  'Referrer-Policy: strict-origin-when-cross-origin',
  'Permissions-Policy',
  'Cross-Origin-Opener-Policy: same-origin',
  'Cross-Origin-Resource-Policy: same-origin',
  "frame-ancestors 'none'",
  'max-age=31536000, immutable',
]) {
  if (headers.includes(required)) pass(required);
  else fail(`missing "${required}"`);
}

if (!existsSync(redirectsPath)) fail('public/_redirects missing');
else {
  const redirects = readFileSync(redirectsPath, 'utf8');
  if (redirects.includes('/* /index.html 200')) pass('SPA fallback /* /index.html 200');
  else fail('SPA fallback missing in public/_redirects');
}

for (const f of ['public/robots.txt', 'public/sitemap.xml', 'public/manifest.webmanifest', 'public/.well-known/security.txt']) {
  if (existsSync(join(root, f))) pass(f);
  else fail(`missing ${f}`);
}

if (failures > 0) {
  console.error(`\nheaders audit: ${failures} failure(s)`);
  process.exit(1);
}
console.log('\nheaders audit: PASS');
