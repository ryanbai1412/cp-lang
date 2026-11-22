// The snippet gate: every CP Lang program on the site is CP Lang 0.3 and gets
// its status from the compiler, never by hand. Every snippet must pass the 0.3
// checker (`CPL_SPEC=1 CPL_STD=spec/std cpl check`). `runs`: today's compiler
// also runs it (matching `.out` when given) and emits C++. `checks`: it does
// not run yet, so the page shows it without Run and without output.
// A practice problem's ref.cp must also pass every test in its tests.json.
// Pages take their programs from content/, never from literals in TSX, so
// everything a visitor can run is covered. Writes .gate/status.json and
// .gate/report.md (the run/check split per page, and why each check-only
// snippet does not run), and fails when a snippet is not 0.3 or is below its
// declared status. Fragments without input only need to build.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = process.env.CPL ?? join(repo, 'target/release/cpl');
if (!existsSync(cpl)) execFileSync('cargo', ['build', '--release', '-q', '-p', 'cpl-cli'], { cwd: repo, stdio: 'inherit' });

const rank = { fails: 0, checks: 1, runs: 2 };
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
    const tests = join(dirname(p), 'tests.json');
    if (p.endsWith('/ref.cp') && existsSync(tests))
      JSON.parse(readFileSync(tests, 'utf8')).forEach((t, i) =>
        snippets.push({
          id: `${relative(web, p)}@test${i + 1}`,
          source: readFileSync(p, 'utf8'),
          stdin: t.input,
          expected: t.expected,
          declared: 'runs',
        }),
      );
  } else if (p.endsWith('.mdx')) {
    const text = readFileSync(p, 'utf8');
    const fence = /^```cp((?:[ \t][^\n]*)?)\n([\s\S]*?)^```/gm;
    let m;
    let i = 0;
    while ((m = fence.exec(text))) {
      i++;
      const meta = m[1];
      const declared = /\bstatus=(runs|checks)\b/.exec(meta)?.[1] ?? 'runs';
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
  const first = (r) => (r.stderr || r.stdout).split('\n').find((l) => l.startsWith('error')) ?? '';
  const v03 = { CPL_SPEC: '1', CPL_STD: join(repo, 'spec/std') };
  const check = sh(['check', file], v03);
  const specBuild = sh(['build', file, '-o', join(tmp, 'spec.cpp')], v03);
  const specRun =
    specBuild.status === 0 && (s.stdin || s.expected != null) ? sh(['run', file], v03, s.stdin) : specBuild;
  const specOk =
    specBuild.status === 0 &&
    specRun.status === 0 &&
    (s.expected == null || specRun.stdout.trimEnd() === s.expected.trimEnd());
  const specNote = specOk
    ? ''
    : specBuild.status === 0 && specRun.status === 0
      ? `output ${JSON.stringify(specRun.stdout)} != expected`
      : first(specBuild.status === 0 ? specRun : specBuild) || (specRun.stderr || '').split('\n')[0];
  const build = sh(['build', file, '-o', join(tmp, 'main.cpp')]);
  const run = build.status === 0 && (s.stdin || s.expected != null) ? sh(['run', file], {}, s.stdin) : build;
  const matches = s.expected == null || run.stdout.trimEnd() === s.expected.trimEnd();
  let status = 'fails';
  let note = '';
  if (check.status !== 0) note = first(check);
  else if (build.status === 0 && run.status === 0 && matches) status = 'runs';
  else {
    status = 'checks';
    note = run.status === 0 && build.status === 0 ? `output ${JSON.stringify(run.stdout)} != expected` : first(build.status === 0 ? run : build);
  }
  results[s.id] = { status, declared: s.declared, ...(status === 'runs' ? {} : { note }), spec: specOk ? 'runs' : specNote };
  if (rank[status] < rank[s.declared]) {
    failed++;
    console.error(`gate: ${s.id} is ${status}, declared ${s.declared}: ${note}`);
  }
}

mkdirSync(join(web, '.gate'), { recursive: true });
writeFileSync(join(web, '.gate/status.json'), JSON.stringify(results, null, 2) + '\n');
const pages = {};
for (const [id, r] of Object.entries(results)) {
  if (id.includes('@test')) continue;
  const page = id.replace(/#\d+$/, '').replace(/\/[^/]+\.cp$/, '/');
  const p = (pages[page] ??= { runs: 0, checks: 0, fails: 0, spec: 0, notes: [], specNotes: [] });
  p[r.status]++;
  if (r.spec === 'runs') p.spec++;
  else if (r.status !== 'fails') p.specNotes.push(`  - \`${id}\`: ${r.spec}`);
  if (r.status !== 'runs') p.notes.push(`  - \`${id}\` ${r.status}: ${r.note}`);
}
const all = Object.values(pages);
writeFileSync(
  join(web, '.gate/report.md'),
  [
    '# Snippet status by page',
    '',
    'Every snippet passes the 0.3 checker unless counted under "not 0.3". "Runs in 0.3 mode" is `cpl run` with',
    '`CPL_SPEC=1 CPL_STD=spec/std`; "runs in the browser" is the default mode the WASM build uses, which decides the Run button.',
    '',
    '| page | snippets | runs in 0.3 mode | runs in the browser | not 0.3 |',
    '|---|---|---|---|---|',
  ]
    .concat(
      Object.entries(pages).map(
        ([k, p]) => `| ${k} | ${p.runs + p.checks + p.fails} | ${p.spec} | ${p.runs} | ${p.fails} |`,
      ),
    )
    .concat(['', '## Fails in 0.3 mode', ''], all.flatMap((p) => p.specNotes))
    .concat(['', '## Fails in the browser (default mode)', ''], all.flatMap((p) => p.notes))
    .join('\n') + '\n',
);
const counts = Object.values(results).reduce((c, r) => ((c[r.status] = (c[r.status] ?? 0) + 1), c), {});
const specRuns = Object.values(results).filter((r) => r.spec === 'runs').length;
console.log(`gate: ${snippets.length} snippets`, counts, `0.3-mode runs: ${specRuns}`, failed ? `${failed} below declared` : 'ok');
process.exit(failed ? 1 : 0);
