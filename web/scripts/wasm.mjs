// Copies the browser build of the compiler (scripts/build-wasm.sh) into
// public/wasm/, building it first when dist/wasm/ is missing or older than
// any source it is built from (a stale build once read 60 KB under its real
// size against the cap), and its types
// into lib/cpl-types.ts so the site cannot drift from the compiler's API.
import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const dist = join(repo, 'dist/wasm');
const files = ['cpl.js', 'cpl.worker.js', 'cpl.wasm', 'cpl-build.wasm'];

// The newest file the build reads: crate sources and manifests, std, the lock.
function newest(path) {
  const st = statSync(path, { throwIfNoEntry: false });
  if (!st) return null;
  if (!st.isDirectory()) return { path, mtime: st.mtimeMs };
  let best = null;
  for (const e of readdirSync(path)) {
    if (e === 'target' || e === 'node_modules' || e === 'tests') continue;
    const n = newest(join(path, e));
    if (n && (!best || n.mtime > best.mtime)) best = n;
  }
  return best;
}
const built = statSync(join(dist, 'cpl.wasm'), { throwIfNoEntry: false })?.mtimeMs ?? 0;
// cpl-wasm and the cpl-* crates it depends on, read from their manifests.
const crates = new Set();
const visit = (name) => {
  if (crates.has(name)) return;
  crates.add(name);
  const manifest = readFileSync(join(repo, 'crates', name, 'Cargo.toml'), 'utf8').split(/^\[dev-dependencies\]/m)[0];
  for (const [, dep] of manifest.matchAll(/^(cpl-[\w-]+)\s*(?:\.workspace|=)/gm)) visit(dep);
};
visit('cpl-wasm');
const inputs = [...[...crates].map((c) => join(repo, 'crates', c)), join(repo, 'std'), join(repo, 'Cargo.lock')]
  .map(newest)
  .filter(Boolean);
const changed = inputs.find((n) => n.mtime > built);
const missing = !files.every((f) => existsSync(join(dist, f)));
const why = process.argv.includes('--rebuild') ? 'asked to' : missing ? 'dist/wasm/ is missing' : changed ? `${changed.path.slice(repo.length + 1)} is newer` : null;
if (process.argv.includes('--dry-run')) {
  console.log(`wasm: ${why ? `would rebuild (${why})` : 'up to date'}`);
  process.exit(0);
}
if (why) {
  console.log(`wasm: rebuilding: ${why}`);
  execFileSync(join(repo, 'scripts/build-wasm.sh'), { stdio: 'inherit' });
}
mkdirSync(join(web, 'public/wasm'), { recursive: true });
for (const f of files) copyFileSync(join(dist, f), join(web, 'public/wasm', f));
writeFileSync(
  join(web, 'lib/cpl-types.ts'),
  readFileSync(join(repo, 'crates/cpl-wasm/js/cpl.d.ts'), 'utf8').replace(/^export (class|function|const) /gm, 'export declare $1 '),
);
console.log(`wasm: copied ${files.join(', ')} to public/wasm/`);
