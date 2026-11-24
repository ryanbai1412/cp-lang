// `pnpm site:test [suite...]`: brings web/out up to date with the working
// tree, rebuilding only what changed, then runs the named browser suites
// (tests/run.mjs; its default set with none). The prebuild stages are cached
// by content hash (prebuild.mjs) and so is `next build`: its key is every
// file the static export is built from.
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { hashFiles, record, recorded, saveMemo, web } from './prebuild/cache.mjs';

const step = (what, cmd, args) => {
  const start = Date.now();
  const r = spawnSync(cmd, args, { stdio: 'inherit', cwd: web });
  console.log(`site:test: ${what} ${((Date.now() - start) / 1000).toFixed(1)} s`);
  if (r.status !== 0) process.exit(r.status ?? 1);
};

step('prebuild', process.execPath, ['scripts/prebuild.mjs']);

const site = ['web', '!web/node_modules', '!web/.next', '!web/out', '!web/tests', '!web/scripts', '!web/.gate', '!web/public/version.json'];
const key = hashFiles(site);
const out = ['web/out'];
const last = recorded('next-build');
if (process.env.PREBUILD_CACHE !== '0' && last?.key === key && last.outputs === hashFiles(out)) console.log('site:test: next build cached');
else {
  step('next build', join(web, 'node_modules/.bin/next'), ['build']);
  record('next-build', { key, outputs: hashFiles(out) });
  saveMemo();
}
step('browser suites', process.execPath, ['tests/run.mjs', ...process.argv.slice(2)]);
