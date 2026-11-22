// Copies the browser build of the compiler (scripts/build-wasm.sh) into
// public/wasm/, building it first when dist/wasm/ is missing, and its types
// into lib/cpl-types.ts so the site cannot drift from the compiler's API.
import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const dist = join(repo, 'dist/wasm');
const files = ['cpl.js', 'cpl.worker.js', 'cpl.wasm'];

if (process.argv.includes('--rebuild') || !files.every((f) => existsSync(join(dist, f)))) {
  execFileSync(join(repo, 'scripts/build-wasm.sh'), { stdio: 'inherit' });
}
mkdirSync(join(web, 'public/wasm'), { recursive: true });
for (const f of files) copyFileSync(join(dist, f), join(web, 'public/wasm', f));
writeFileSync(
  join(web, 'lib/cpl-types.ts'),
  readFileSync(join(repo, 'crates/cpl-wasm/js/cpl.d.ts'), 'utf8').replace(/^export (class|function) /gm, 'export declare $1 '),
);
console.log(`wasm: copied ${files.join(', ')} to public/wasm/`);
