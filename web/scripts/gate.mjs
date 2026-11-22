// The snippet gate: every CP Lang program on the site gets a status from the
// compiler, never by hand. `runs`: `cpl run` succeeds (and matches `.out`
// when given) and `cpl build` emits C++. `checks`: the 0.3 spec checker
// accepts it. `spec`: neither, allowed only when the snippet says so.
// Writes .gate/status.json and fails when a snippet is below its declared
// status. Fragments without input only need to build; with input they must run.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = process.env.CPL ?? join(repo, 'target/release/cpl');
if (!existsSync(cpl)) execFileSync('cargo', ['build', '--release', '-q', '-p', 'cpl-cli'], { cwd: repo, stdio: 'inherit' });

const rank = { spec: 0, checks: 1, runs: 2 };
const snippets = [];

function walk(dir, f) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) walk(p, f);
    else f(p);
  }
}

walk(join(web, 'content'), (p) => {
  if (p.endsWith('.cp')) {
    const base = p.replace(/(\.spec)?\.cp$/, '');
    const read = (ext) => (existsSync(`${base}.${ext}`) ? readFileSync(`${base}.${ext}`, 'utf8') : null);
    snippets.push({
      id: relative(web, p),
      source: readFileSync(p, 'utf8'),
      stdin: read('in') ?? '',
      expected: p.endsWith('.spec.cp') ? null : read('out'),
      declared: p.endsWith('.spec.cp') ? 'checks' : 'runs',
    });
  } else if (p.endsWith('.mdx')) {
    const text = readFileSync(p, 'utf8');
    const fence = /^```cp((?:[ \t][^\n]*)?)\n([\s\S]*?)^```/gm;
    let m;
    let i = 0;
    while ((m = fence.exec(text))) {
      i++;
      const meta = m[1];
      const declared = /\bstatus=(runs|checks|spec)\b/.exec(meta)?.[1] ?? 'runs';
      const stdin = /\binput="([^"]*)"/.exec(meta)?.[1]?.replace(/\\n/g, '\n') ?? '';
      snippets.push({ id: `${relative(web, p)}#${i}`, source: m[2], stdin, expected: null, declared });
    }
  }
});

const tmp = join(tmpdir(), 'cp-gate');
mkdirSync(tmp, { recursive: true });
writeFileSync(join(tmp, 'cp.toml'), `[libs]\nlib = "${join(repo, 'examples/local-lib')}"\n`);
const sh = (args, env = {}, input = '') =>
  spawnSync(cpl, args, { input, encoding: 'utf8', timeout: 20000, env: { ...process.env, ...env } });

const results = {};
let failed = 0;
for (const s of snippets) {
  const file = join(tmp, 'main.cp');
  writeFileSync(file, s.source);
  let status = 'spec';
  let note = '';
  const build = sh(['build', file, '-o', join(tmp, 'main.cpp')]);
  const run = build.status === 0 && (s.stdin || s.expected != null) ? sh(['run', file], {}, s.stdin) : build;
  if (build.status === 0 && run.status === 0 && (s.expected == null || run.stdout === s.expected)) status = 'runs';
  else {
    if (run.status === 0 && s.expected != null && run.stdout !== s.expected) note = `output ${JSON.stringify(run.stdout)} != expected`;
    const check = sh(['check', file], { CPL_SPEC: '1', CPL_STD: join(repo, 'spec/std') });
    if (check.status === 0) status = 'checks';
    else note ||= (check.stderr || check.stdout).split('\n').slice(0, 3).join(' ');
  }
  results[s.id] = { status, declared: s.declared };
  if (rank[status] < rank[s.declared]) {
    failed++;
    console.error(`gate: ${s.id} is ${status}, declared ${s.declared}${note ? `: ${note}` : ''}`);
  }
}

mkdirSync(join(web, '.gate'), { recursive: true });
writeFileSync(join(web, '.gate/status.json'), JSON.stringify(results, null, 2) + '\n');
const counts = Object.values(results).reduce((c, r) => ((c[r.status] = (c[r.status] ?? 0) + 1), c), {});
console.log(`gate: ${snippets.length} snippets`, counts, failed ? `${failed} below declared` : 'ok');
process.exit(failed ? 1 : 0);
