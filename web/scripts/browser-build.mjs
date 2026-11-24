// The browser build of the compiler in dist/wasm/ (scripts/build-wasm.sh),
// made current before anything reads it. A build of other sources than this
// tree's differs from cpl on every program (site/R1), and one older than its
// sources once read 60 KB under its real size against the cap. The key is of
// content, not mtimes, so a rebase, a checkout or CI's back-dated sources
// cannot pass a stale build off as current.
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const script = join(repo, 'scripts/build-wasm.sh');
export const dist = join(repo, 'dist/wasm');
export const files = ['cpl.js', 'cpl.worker.js', 'cpl.wasm', 'cpl-build.wasm'];

// Why the build in dist/wasm/ can't be used as it is, or null when it can.
export function staleReason({ missing, stamp, key }) {
  if (missing) return `dist/wasm/${missing} is missing`;
  if (stamp === null) return 'dist/wasm/ does not say what it was built from';
  if (stamp !== key) return 'dist/wasm/ was built from other sources than this tree';
  return null;
}

export function whyRebuild() {
  const stamp = join(dist, 'source.txt');
  return staleReason({
    missing: files.find((f) => !existsSync(join(dist, f))),
    stamp: existsSync(stamp) ? readFileSync(stamp, 'utf8').trim() : null,
    key: execFileSync(script, ['--source'], { encoding: 'utf8' }).trim(),
  });
}

// dist/wasm/, rebuilt first when stale. Without a wasm toolchain a local run
// gets null and skips what needs it; CI always has one, so there it throws.
export function browserBuild({ rebuild = false, log = console.log } = {}) {
  const why = rebuild ? 'asked to' : whyRebuild();
  if (!why) return dist;
  log(`wasm: rebuilding: ${why}`);
  try {
    execFileSync(script, { stdio: 'inherit' });
  } catch (error) {
    if (process.env.CI) throw error;
    return null;
  }
  return dist;
}
