// The snippet gate: every CP Lang program on the site is CP Lang 0.3 and gets
// its status from the compiler, never by hand. `runs`: the compiler runs it
// (matching `.out` when given) and emits C++. `checks`: it passes the checker
// but does not run yet. `spec`: it is written to spec/ and the compiler does
// not accept it yet; only `*.spec.cp` files and `status=spec` fences may be
// that, and without an input file one is `checks` at best. The page shows `checks` and `spec` without Run and without output.
// A practice problem's ref.cp must also pass every test in its tests.json.
// Pages take their programs from content/, never from literals in TSX, so
// everything a visitor can run is covered. Writes .gate/status.json and
// .gate/report.md (the run/check split per page, and why each check-only
// snippet does not run), and fails when a snippet is below its declared
// status. Fragments without input only need to build; `ran` says whether a
// fence prints something when run (with empty input if it has none), which is
// what gives it a Run button.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = process.env.CPL ?? join(repo, 'target/release/cpl');
if (!existsSync(cpl)) execFileSync('cargo', ['build', '--release', '-q', '-p', 'cpl-cli'], { cwd: repo, stdio: 'inherit' });

const rank = { fails: 0, spec: 1, checks: 2, runs: 3 };
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
      declared: p.endsWith('.spec.cp') ? 'spec' : 'runs',
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
      const rejects = /\brejects=([ER]\d{4})\b/.exec(meta)?.[1];
      const declared = rejects ? 'rejects' : (/\bstatus=(runs|checks|spec)\b/.exec(meta)?.[1] ?? 'runs');
      const stdin = /\binput="([^"]*)"/.exec(meta)?.[1]?.replace(/\\n/g, '\n') ?? '';
      snippets.push({ id: `${relative(web, p)}#${i}`, source: m[2], stdin, expected: null, declared, rejects });
    }
  }
});

const tmp = join(tmpdir(), 'cp-gate');
mkdirSync(tmp, { recursive: true });
writeFileSync(join(tmp, 'cp.toml'), `[libs]\nlib = "${join(repo, 'examples/local-lib')}"\n`);
const sh = (args, input = '') => spawnSync(cpl, args, { input, encoding: 'utf8', timeout: 20000 });

const results = {};
let failed = 0;
// A practice problem is a folder (starter.cp, ref.cp, tests.json with a sample),
// a Learn page that judges it and an entry in the practice meta.json.
const learn = join(web, 'content/learn/practice');
const listed = JSON.parse(readFileSync(join(learn, 'meta.json'), 'utf8')).pages;
for (const id of readdirSync(join(web, 'content/practice'))) {
  const file = (name) => join(web, 'content/practice', id, name);
  const page = join(learn, `${id}.mdx`);
  const lacks = ['starter.cp', 'ref.cp', 'tests.json'].filter((f) => !existsSync(file(f)));
  if (existsSync(file('tests.json')) && !JSON.parse(readFileSync(file('tests.json'), 'utf8')).some((t) => t.sample)) lacks.push('a test with "sample": true');
  if (!existsSync(page) || !readFileSync(page, 'utf8').includes(`<Judge id="${id}"`)) lacks.push(`content/learn/practice/${id}.mdx with <Judge id="${id}">`);
  if (!listed.includes(id)) lacks.push('an entry in content/learn/practice/meta.json');
  const hints = existsSync(page) ? /<Judge[^>]*>([\s\S]*?)<\/Judge>/.exec(readFileSync(page, 'utf8'))?.[1] ?? '' : '';
  if (!/\]\(\/[^)]+\)/.test(hints)) lacks.push('a hint that links to the docs, a chapter or a lint');
  if (existsSync(file('ref.cp'))) {
    const lint = spawnSync(cpl, ['lint', file('ref.cp')], { encoding: 'utf8' });
    const found = (lint.stdout + lint.stderr).match(/^warning\[L\d{4}\].*$/gm);
    if (found) lacks.push(`a ref.cp with no lint warnings (has ${found.join('; ')})`);
  }
  if (lacks.length) {
    failed++;
    console.error(`gate: practice problem ${id} lacks ${lacks.join(', ')}`);
  }
}
for (const s of snippets) {
  const file = join(tmp, 'main.cp');
  writeFileSync(file, s.source);
  const first = (r) => (r.stderr || r.stdout).split('\n').find((l) => l.startsWith('error')) ?? '';
  const check = sh(['check', file]);
  // A fence written ```cp rejects=E0602 shows a mistake: it must fail `cpl check` with that code.
  // rejects=R0202 shows a trap: it checks, and `cpl run` on its input stops with that code.
  if (s.rejects) {
    const trap = s.rejects.startsWith('R');
    const out = trap && check.status === 0 ? sh(['run', file], s.stdin) : check;
    const got = /^error\[([ER]\d{4})\]/m.exec(out.stderr || out.stdout)?.[1];
    const ok = out.status !== 0 && got === s.rejects;
    results[s.id] = { status: ok ? 'rejects' : 'fails', declared: 'rejects', ...(ok ? {} : { note: `wanted ${s.rejects}, got ${got ?? 'no error'}` }) };
    if (!ok) {
      failed++;
      console.error(`gate: ${s.id} should be rejected with ${s.rejects}: ${results[s.id].note}`);
    }
    continue;
  }
  const build = sh(['build', file, '-o', join(tmp, 'main.cpp')]);
  const run = build.status === 0 && (s.stdin || s.expected != null) ? sh(['run', file], s.stdin) : build;
  const matches = s.expected == null || run.stdout.trimEnd() === s.expected.trimEnd();
  let status = 'fails';
  let note = '';
  if (check.status !== 0) {
    note = first(check);
    if (s.declared === 'spec') status = 'spec';
  }
  else if (build.status === 0 && run.status === 0 && matches) status = s.declared === 'spec' && !s.stdin ? 'checks' : 'runs';
  else {
    status = 'checks';
    note = run.status === 0 && build.status === 0 ? `output ${JSON.stringify(run.stdout)} != expected` : first(build.status === 0 ? run : build);
  }
  // A fence the gate only built runs live on the page only if it also runs with no input.
  const live = status === 'runs' && (run !== build ? run : sh(['run', file], ''));
  const ran = Boolean(live && live.status === 0 && live.stdout.trim());
  results[s.id] = { status, declared: s.declared, ...(status === 'runs' ? { ran } : { note }) };
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
  const p = (pages[page] ??= { runs: 0, checks: 0, spec: 0, fails: 0, rejects: 0, notes: [] });
  p[r.status]++;
  if (r.status !== 'runs' && r.status !== 'rejects') p.notes.push(`  - \`${id}\` ${r.status}: ${r.note}`);
}
writeFileSync(
  join(web, '.gate/report.md'),
  ['# Snippet status by page', '', '| page | snippets | runs | check-only | spec only |', '|---|---|---|---|---|']
    .concat(Object.entries(pages).map(([k, p]) => `| ${k} | ${p.runs + p.checks + p.spec + p.fails + p.rejects} | ${p.runs + p.rejects} | ${p.checks} | ${p.spec + p.fails} |`))
    .concat(['', '## Does not run', ''], Object.values(pages).flatMap((p) => p.notes))
    .join('\n') + '\n',
);
const corpus = {};
for (const e of readdirSync(join(repo, 'corpus/problems'), { withFileTypes: true })) {
  const spec = join(repo, 'corpus/problems', e.name, 'spec.cp');
  if (e.isDirectory() && existsSync(spec)) corpus[e.name] = sh(['check', spec]).status === 0 ? 'checks' : 'spec';
}
writeFileSync(join(web, '.gate/corpus.json'), JSON.stringify(corpus, null, 2) + '\n');
const counts = Object.values(results).reduce((c, r) => ((c[r.status] = (c[r.status] ?? 0) + 1), c), {});
console.log(`gate: ${snippets.length} snippets`, counts, failed ? `${failed} below declared` : 'ok');
process.exit(failed ? 1 : 0);
