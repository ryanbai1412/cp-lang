// Enforces site/R1 (handbook/guidelines/site-checklist.md).
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
import { execFileSync, spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { availableParallelism, tmpdir } from 'node:os';
import { Worker } from 'node:worker_threads';
import { cplBin } from './cpl-bin.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = cplBin();
if (!existsSync(cpl)) execFileSync('cargo', ['build', '--release', '-q', '-p', 'cpl-cli'], { cwd: repo, stdio: 'inherit' });
// A cpl older than the compiler sources (or still being rebuilt) can report
// false failures; a failing run says so, without paying for a rebuild.
function newest(dir) {
  let t = 0;
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    if (e.name === 'target' || e.name.startsWith('.')) continue;
    const p = join(dir, e.name);
    t = Math.max(t, e.isDirectory() ? newest(p) : e.name.endsWith('.rs') ? statSync(p).mtimeMs : 0);
  }
  return t;
}
process.on('exit', (code) => {
  if (code && !process.env.CPL && newest(join(repo, 'crates')) > statSync(cpl).mtimeMs)
    console.error(`gate: WARNING: ${relative(repo, cpl)} is older than crates/; these failures may be stale (cargo build --release -p cpl-cli)`);
});

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
const sh = (args, input = '') => spawnSync(cpl, args, { input, encoding: 'utf8', timeout: 20000 });
// `sh` without blocking, so snippets run on every core.
const shAsync = (args, input = '') =>
  new Promise((resolve) => {
    const p = spawn(cpl, args);
    let stdout = '';
    let stderr = '';
    p.stdout.setEncoding('utf8').on('data', (d) => (stdout += d));
    p.stderr.setEncoding('utf8').on('data', (d) => (stderr += d));
    const timer = setTimeout(() => p.kill(), 20000);
    p.on('error', (e) => {
      clearTimeout(timer);
      resolve({ status: null, stdout, stderr: stderr + String(e) });
    });
    p.on('close', (status) => {
      clearTimeout(timer);
      resolve({ status, stdout, stderr });
    });
    p.stdin.on('error', () => {});
    p.stdin.end(input);
  });

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
  // handbook/guidelines/site-checklist.md site/R19.
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
// Each worker gets its own folder, so parallel runs never share a main.cp.
const slots = snippets.map(() => null);
const cli = snippets.map(() => null);
const errorsOf = snippets.map(() => []);
async function gate(k, dir) {
  const s = snippets[k];
  const errors = errorsOf[k];
  const file = join(dir, 'main.cp');
  writeFileSync(file, s.source);
  const first = (r) => (r.stderr || r.stdout).split('\n').find((l) => l.startsWith('error')) ?? '';
  const check = await shAsync(['check', file]);
  // A fence written ```cp rejects=E0602 shows a mistake: it must fail `cpl check` with that code.
  // rejects=R0202 shows a trap: it checks, and `cpl run` on its input stops with that code.
  if (s.rejects) {
    const trap = s.rejects.startsWith('R');
    const out = trap && check.status === 0 ? await shAsync(['run', file], s.stdin) : check;
    const got = /^error\[([ER]\d{4})\]/m.exec(out.stderr || out.stdout)?.[1];
    const ok = out.status !== 0 && got === s.rejects;
    slots[k] = { status: ok ? 'rejects' : 'fails', declared: 'rejects', ...(ok ? {} : { note: `wanted ${s.rejects}, got ${got ?? 'no error'}` }) };
    if (!ok) {
      errors.push(`gate: ${s.id} should be rejected with ${s.rejects}: ${slots[k].note}`);
    }
    return;
  }
  const build = await shAsync(['build', file, '-o', join(dir, 'main.cpp')]);
  const run = build.status === 0 && (s.stdin || s.expected != null) ? await shAsync(['run', file], s.stdin) : build;
  if (build.status === 0) cli[k] = { cpp: readFileSync(join(dir, 'main.cpp'), 'utf8'), stdout: run === build ? null : run.stdout, status: run.status };
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
  const live = status === 'runs' && (run !== build ? run : await shAsync(['run', file], ''));
  const ran = Boolean(live && live.status === 0 && live.stdout.trim());
  slots[k] = { status, declared: s.declared, ...(status === 'runs' ? { ran } : { note }) };
  if (rank[status] < rank[s.declared]) {
    errors.push(`gate: ${s.id} is ${status}, declared ${s.declared}: ${note}`);
  }
}
let next = 0;
await Promise.all(
  Array.from({ length: Math.min(availableParallelism(), snippets.length) }, async (_, w) => {
    const dir = join(tmp, `w${w}`);
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, 'cp.toml'), `[libs]\nlib = "${join(repo, 'examples/local-lib')}"\n`);
    while (next < snippets.length) await gate(next++, dir);
  }),
);
// The browser build must mean what the CLI means (site/R1): every program
// that runs gets the same output and the same C++ from cpl-build.wasm.
const wasmDir = [join(repo, 'dist/wasm'), join(web, 'public/wasm')].find((d) => existsSync(join(d, 'cpl-build.wasm')));
if (!wasmDir) console.log('gate: no browser build (scripts/build-wasm.sh); skipping the browser differential');
else {
  const started = Date.now();
  const todo = snippets.map((s, k) => k).filter((k) => cli[k] && slots[k]?.status === 'runs' && !/^\s*use lib\b/m.test(snippets[k].source));
  const spawnWorker = () => new Worker(join(web, 'scripts/wasm-worker.mjs'), { workerData: { dir: wasmDir } });
  // A worker that gives no answer in 20 s is replaced, and the program counts
  // as too slow for the browser.
  const ask = (w, msg) =>
    new Promise((resolve) => {
      const done = (r) => {
        clearTimeout(timer);
        w.off('message', done);
        w.off('error', failed);
        resolve(r);
      };
      const failed = (e) => done({ error: String(e) });
      const timer = setTimeout(() => done({ error: 'timeout: no answer in 20 s' }), 20000);
      w.on('message', done);
      w.on('error', failed);
      w.postMessage(msg);
    });
  let at = 0;
  const slow = [];
  await Promise.all(
    Array.from({ length: Math.min(availableParallelism(), todo.length) }, async () => {
      let w = spawnWorker();
      while (at < todo.length) {
        const k = todo[at++];
        const s = snippets[k];
        const c = cli[k];
        const r = await ask(w, { source: s.source, stdin: s.stdin, run: c.stdout !== null });
        const differs = [];
        // Too slow for the browser's time limit is a performance gap, counted
        // and listed below; a different answer is a failure.
        const outOfTime = r.error?.startsWith('timeout') || /took longer than its time limit/.test(r.stderr ?? '');
        if (outOfTime) {
          slow.push(s.id);
          if (r.error) (await w.terminate(), (w = spawnWorker()));
        } else if (r.error) {
          differs.push(r.error);
          await w.terminate();
          w = spawnWorker();
        } else {
          if (r.cpp !== c.cpp) differs.push('its C++');
          if (c.stdout !== null && (r.stdout !== c.stdout || r.ok !== (c.status === 0)))
            differs.push(`its output (${JSON.stringify(r.stdout?.slice(0, 80))} vs ${JSON.stringify(c.stdout.slice(0, 80))})`);
        }
        if (differs.length) errorsOf[k].push(`gate: ${s.id} differs in the browser from cpl: ${differs.join(', ')}`);
      }
      await w.terminate();
    }),
  );
  console.log(`gate: browser differential: ${todo.length} programs in ${((Date.now() - started) / 1000).toFixed(1)}s; ${slow.length} too slow for the browser's time limit`);
  for (const id of slow) console.log(`gate:   too slow in the browser: ${id}`);
  mkdirSync(join(web, '.gate'), { recursive: true });
  writeFileSync(join(web, '.gate/browser-slow.json'), JSON.stringify(slow, null, 2) + '\n');
}
snippets.forEach((s, k) => {
  results[s.id] = slots[k];
  for (const e of errorsOf[k]) {
    failed++;
    console.error(e);
  }
});

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
