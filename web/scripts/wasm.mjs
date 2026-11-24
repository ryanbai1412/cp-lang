// Copies the browser build of the compiler (scripts/build-wasm.sh) into
// public/wasm/, building it first when dist/wasm/ is missing or was built
// from other sources (browser-build.mjs). The site imports its types straight from
// crates/cpl-wasm/js/cpl.d.ts, so there is no copy to drift.
import { copyFileSync, cpSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { browserBuild, dist, files, whyRebuild } from './browser-build.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
if (process.argv.includes('--dry-run')) {
  const why = whyRebuild();
  console.log(`wasm: ${why ? `would rebuild (${why})` : 'up to date'}`);
  process.exit(0);
}
if (!browserBuild({ rebuild: process.argv.includes('--rebuild') })) throw new Error('wasm: the browser build failed; rerun: scripts/build-wasm.sh');
mkdirSync(join(web, 'public/wasm'), { recursive: true });
for (const f of files) copyFileSync(join(dist, f), join(web, 'public/wasm', f));
// The std module texts the page fetches the first time a program uses one.
cpSync(join(dist, 'std'), join(web, 'public/wasm/std'), { recursive: true });
console.log(`wasm: copied ${files.join(', ')} to public/wasm/`);
