import { readFileSync, existsSync, writeFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
const version = process.env.VITE_APP_VERSION || pkg.version || '0.0.0';
const commit = (process.env.CF_PAGES_COMMIT_SHA || process.env.VITE_COMMIT_SHA || 'local').slice(0, 8);

const template = join(root, 'public', 'version.json');
if (!existsSync(template)) {
  console.error('version stamp: public/version.json template missing');
  process.exit(1);
}
// version.json ships static in public/; stamp the built copy.
const dist = join(root, 'dist', 'version.json');
const body = JSON.stringify(
  {
    name: pkg.name,
    version,
    commit,
    builtAt: new Date().toISOString(),
    status: 'ok',
  },
  null,
  0,
);
writeFileSync(dist, body);
console.log(`ok version: ${version} (${commit}) → dist/version.json`);
