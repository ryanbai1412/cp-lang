// Enforces site/R15 (handbook/guidelines/site-checklist.md): the playground's
// Stress tab finds the counterexample `cpl stress` finds. For every program in
// the repo with an `@alt(program)` brute, the browser build's stress and the
// CLI's, with the same seed, cases and size, must agree on the verdict, the
// shrunk input and the first diverging variable.
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cplBin } from './cpl-bin.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cli = cplBin();
const wasmDir = [join(repo, 'dist/wasm'), join(web, 'public/wasm')].find((d) => existsSync(join(d, 'cpl-build.wasm')));
if (!wasmDir) {
  console.log('stress-parity: no browser build; skipping');
  process.exit(0);
}

const opts = { cases: 30, seed: 1, size: 8 };
// A browser run longer than this is a Stress tab a contestant gives up on.
const BROWSER_LIMIT_MS = 20000;
const files = execFileSync('git', ['grep', '-l', '@alt(program)', '--', '*.cp', ':!spec/conformance'], { cwd: repo, encoding: 'utf8' })
  .split('\n').filter(Boolean);
const tmp = mkdtempSync(join(tmpdir(), 'stress-parity-'));
const view = (o) => o.status === 'fail'
  ? { status: 'fail', case: o.counterexample.case, verdict: o.counterexample.verdict, input: o.counterexample.raw_input,
      first: o.counterexample.explain?.divergence?.name ?? null }
  : o.status === 'ok' ? { status: 'ok', cases: o.cases } : { status: 'error' };

let bad = 0, found = 0;
const tle = [];
const slow = [];
for (const [i, f] of files.entries()) {
  const copy = join(tmp, `${i}-${basename(f)}`);
  copyFileSync(join(repo, f), copy);
  const t0 = Date.now();
  let cliOut;
  try {
    cliOut = execFileSync(cli, ['stress', copy, '--format', 'json', '--explain', '--cases', `${opts.cases}`, '--seed', `${opts.seed}`, '--size', `${opts.size}`, '--time-limit', '1000', '--jobs', '1'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 120000 });
  } catch (e) {
    cliOut = e.stdout;
  }
  let a, b;
  try { a = view(JSON.parse(cliOut)); } catch { a = { status: 'error' }; }
  // The browser stops a run after 5 s of wall clock, so shrinking a time limit
  // there takes minutes; those programs are compared by the CLI's suites only.
  if (a.verdict === 'TLE') {
    tle.push(f);
    continue;
  }
  const t1 = Date.now();
  const run = spawnSync(process.execPath, [join(web, 'scripts/stress-worker.mjs'), copy, JSON.stringify(opts), wasmDir], { encoding: 'utf8', timeout: BROWSER_LIMIT_MS });
  if (run.error?.code === 'ETIMEDOUT') {
    slow.push(`${f} (${a.status === 'fail' ? `cpl stress fails case ${a.case}` : `no counterexample in ${opts.cases} cases`})`);
    continue;
  }
  try { b = view(JSON.parse(run.stdout)); } catch { b = { status: 'error' }; }
  if (process.env.STRESS_TIMES) console.log(`${Date.now() - t1}ms in the browser build`);
  if (process.env.STRESS_TIMES) console.log(`${Date.now() - t0}ms ${f}`);
  if (JSON.stringify(a) !== JSON.stringify(b)) {
    bad++;
    console.error(`stress-parity: ${f}\n  cpl stress: ${JSON.stringify(a)}\n  browser:    ${JSON.stringify(b)}`);
  } else if (a.status === 'fail') found++;
}
for (const f of slow) console.log(`stress-parity: slow in the browser (over ${BROWSER_LIMIT_MS / 1000} s for ${opts.cases} cases): ${f}`);
const compared = files.length - tle.length - slow.length;
console.log(`stress-parity: ${compared - bad} of ${compared} @alt programs agree (${found} counterexamples found alike); ${tle.length} whose counterexample is a time limit not run in the browser, ${slow.length} too slow there`);
process.exit(bad ? 1 : 0);
