import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const config = JSON.parse(readFileSync(resolve(root, 'src-tauri/tauri.conf.json'), 'utf8'));
const policy = config.app.security.csp;
const imagePolicy = typeof policy === 'string'
  ? policy.split(';').find(directive => directive.trim().startsWith('img-src ')) ?? ''
  : policy['img-src'] ?? '';
assert.ok(imagePolicy.includes("'self'"), 'Packaged sprites require same-origin images in the native CSP');

const directory = resolve(root, 'dist/assets');
const files = readdirSync(directory);
const bundle = files.filter(file => file.endsWith('.js')).map(file => readFileSync(resolve(directory, file), 'utf8')).join('\n');
assert.ok(bundle.length, 'Production JavaScript bundle is missing; run npm run build first');
if (!imagePolicy.includes('data:')) {
  assert.ok(!/data:image\/[a-z+.-]+;base64,/.test(bundle), 'Production bundle contains inline images blocked by the native CSP');
}

const animationSource = readFileSync(resolve(root, 'src/animation.ts'), 'utf8');
const sprites = [...animationSource.matchAll(/from\s+['"]\.\.\/sprites\/([^'"]+\.png)['"]/g)].map(match => match[1]);
assert.ok(sprites.length, 'No sprite-sheet imports were found');
for (const sprite of sprites) {
  const stem = basename(sprite, '.png');
  const matches = files.filter(file => file.startsWith(`${stem}-`) && file.endsWith('.png'));
  assert.equal(matches.length, 1, `Expected one packaged asset for ${sprite}`);
  assert.ok(bundle.includes(matches[0]), `Bundle does not reference ${sprite}`);
  const png = readFileSync(resolve(directory, matches[0]));
  assert.equal(png.subarray(1, 4).toString(), 'PNG', `${sprite} is not a PNG`);
  assert.ok(png.readUInt32BE(16) > 0 && png.readUInt32BE(20) > 0, `${sprite} has invalid dimensions`);
}
console.log(`Packaged assets verified: ${sprites.length} sprite sheets load under the native CSP.`);
