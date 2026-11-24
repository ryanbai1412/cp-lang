// Enforces site/R15 (handbook/guidelines/site-checklist.md): the playground's
// Stress tab finds the counterexample `cpl stress` finds. For every program in
// the repo with an `@alt(program)` brute or an `@interactor`, the browser
// build's stress and the CLI's, with the same seed, cases, size and time limit,
// must agree on the verdict, the shrunk input, the first diverging variable,
// the solution's output and, for an interactive run, the transcript and the
// interactor's verdict.
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
  // CI always builds it; only a local run without one may skip.
  console.error('stress-parity: no browser build; run scripts/build-wasm.sh');
  process.exit(process.env.CI ? 1 : 0);
}

const opts = { cases: 30, seed: 1, size: 8, timeLimit: 1000 };
// A browser run longer than this is a Stress tab a contestant gives up on.
const BROWSER_LIMIT_MS = 20000;
const files = execFileSync('git', ['grep', '-lE', '@alt\\(program\\)|^\\s*@interactor', '--', '*.cp', ':!spec/conformance'], { cwd: repo, encoding: 'utf8' })
  .split('\n').filter(Boolean);
const tmp = mkdtempSync(join(tmpdir(), 'stress-parity-'));
const view = (o) => o.status === 'fail'
  ? { status: 'fail', case: o.counterexample.case, verdict: o.counterexample.verdict, input: o.counterexample.raw_input,
      explain: o.counterexample.explain ?? null, got: o.counterexample.got ?? null,
      interaction: o.counterexample.interaction ?? null }
  : o.status === 'ok' ? { status: 'ok', cases: o.case_count } : { status: 'error' };

let bad = 0, found = 0;
const slow = [];
for (const [i, f] of files.entries()) {
  const copy = join(tmp, `${i}-${basename(f)}`);
  copyFileSync(join(repo, f), copy);
  const t0 = Date.now();
  let cliOut;
  try {
    cliOut = execFileSync(cli, ['stress', copy, '--format', 'json', '--explain', '--cases', `${opts.cases}`, '--seed', `${opts.seed}`, '--size', `${opts.size}`, '--time-limit', `${opts.timeLimit}`, '--jobs', '1'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 120000 });
  } catch (e) {
    cliOut = e.stdout;
  }
  let a, b;
  try { a = view(JSON.parse(cliOut)); } catch { a = { status: 'error' }; }
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
for (const f of slow) console.error(`stress-parity: too slow in the browser (over ${BROWSER_LIMIT_MS / 1000} s for ${opts.cases} cases): ${f}`);
const compared = files.length - slow.length;
console.log(`stress-parity: ${compared - bad} of ${files.length} stress programs agree (${found} counterexamples found alike); ${slow.length} too slow`);
process.exit(bad || slow.length ? 1 : 0);
