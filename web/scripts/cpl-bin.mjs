// The `cpl` the site scripts run: $CPL, else the repo's release or debug build.
// A release build bundles std/ when it is built, so a binary older than std/ or
// the compiler's sources answers differently from main; rebuild it rather
// than publish (or test against) those answers.
import { execFileSync } from 'node:child_process';
import { existsSync, statSync, utimesSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));

export function cplBin() {
  if (process.env.CPL) return process.env.CPL;
  const bin = ['target/release/cpl', 'target/debug/cpl'].map((p) => join(repo, p)).find(existsSync);
  if (!bin) return 'cpl';
  const built = statSync(bin).mtimeMs;
  const sources = execFileSync('git', ['ls-files', 'std', 'crates'], { cwd: repo, encoding: 'utf8' }).split('\n');
  const newer = sources.find((f) => f && existsSync(join(repo, f)) && statSync(join(repo, f)).mtimeMs > built);
  if (newer) {
    // Stale against this tree: rebuild it here rather than answer for an
    // older compiler (cargo only redoes what changed).
    console.error(`${bin} is older than ${newer}; rebuilding it`);
    const release = bin.includes('/release/');
    execFileSync('cargo', ['build', '-q', ...(release ? ['--release'] : []), '-p', 'cpl-cli'], { cwd: repo, stdio: ['ignore', 2, 2] });
    // Cargo leaves a binary its edit didn't change (a test file) untouched.
    const now = new Date();
    utimesSync(bin, now, now);
  }
  return bin;
}
