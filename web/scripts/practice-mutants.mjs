// handbook/guidelines/site-checklist.md site/R1: a practice problem's tests must
// reject wrong solutions. Each one-token mutant of ref.cp that still checks is
// run on tests.json; a mutant that passes every test survived, which means the
// tests miss that bug, unless practice-mutants-equivalent.txt lists it with the
// reason no test can. Writes .gate/mutants.json.
import { spawn } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { availableParallelism, tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cplBin } from './cpl-bin.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = cplBin();
const only = process.argv.slice(2);

const operators = [
  [/<=/g, '<'], [/(?<![<\-])<(?![=<\-A-Z(])/g, '<='], [/>=/g, '>'], [/(?<![>\-=])>(?![=>])/g, '>='],
  [/==/g, '!='], [/!=/g, '=='], [/\.\.=/g, '..'], [/\.\.(?![=.])/g, '..='],
  [/ \+ 1\b/g, ''], [/ - 1\b/g, ''], [/ \+ (?!=)/g, ' - '], [/ - (?!=)/g, ' + '],
  [/\bmin\b/g, 'max'], [/\bmax\b/g, 'min'], [/\band\b/g, 'or'], [/\bor\b/g, 'and'],
  [/(?<![\w.])([0-9])(?![\w.])/g, (d) => String(Number(d) + 1)],
];

function mutants(src) {
  const lines = src.split('\n');
  const out = [];
  lines.forEach((line, i) => {
    const code = line.replace(/#.*$|\/\/.*$/, '');
    if (/\bread\w*\s*[(<]/.test(code)) return;
    if (/^\s*(use|fn \w+\(.*\)\s*->|type|domain)\b/.test(code) && !/=/.test(code)) return;
    for (const [re, to] of operators) {
      for (const m of code.matchAll(new RegExp(re.source, 'g'))) {
        if (/"[^"]*$/.test(code.slice(0, m.index))) continue;
        const rep = typeof to === 'function' ? to(m[0]) : to;
        const next = line.slice(0, m.index) + rep + line.slice(m.index + m[0].length);
        out.push({ line: i + 1, from: line.trim(), to: next.trim(), src: [...lines.slice(0, i), next, ...lines.slice(i + 1)].join('\n') });
      }
    }
  });
  return out;
}

const sh = (args, input = '', ms = 10000) =>
  new Promise((resolve) => {
    const p = spawn(cpl, args, { stdio: ['pipe', 'pipe', 'pipe'] });
    let stdout = '';
    p.stdout.setEncoding('utf8').on('data', (d) => (stdout += d));
    p.stderr.resume();
    const timer = setTimeout(() => p.kill('SIGKILL'), ms);
    p.on('close', (status) => (clearTimeout(timer), resolve({ status, stdout })));
    p.stdin.on('error', () => {});
    p.stdin.end(input);
  });

const jobs = [];
for (const id of readdirSync(join(web, 'content/practice')).sort()) {
  if (only.length && !only.includes(id)) continue;
  const dir = join(web, 'content/practice', id);
  const tests = JSON.parse(readFileSync(join(dir, 'tests.json'), 'utf8'));
  for (const m of mutants(readFileSync(join(dir, 'ref.cp'), 'utf8'))) jobs.push({ id, tests, ...m });
}
const work = join(tmpdir(), `cpl-mutants-${process.pid}`);
const results = [];
let at = 0;
await Promise.all(
  Array.from({ length: availableParallelism() }, async (_, slot) => {
    const dir = join(work, String(slot));
    mkdirSync(dir, { recursive: true });
    const file = join(dir, 'main.cp');
    while (at < jobs.length) {
      const j = jobs[at++];
      writeFileSync(file, j.src);
      if ((await sh(['check', file])).status !== 0) continue;
      let killed = false;
      for (const t of j.tests) {
        const r = await sh(['run', file], t.input);
        if (r.status !== 0 || r.stdout.trimEnd() !== t.expected.trimEnd()) {
          killed = true;
          break;
        }
      }
      results.push({ id: j.id, line: j.line, from: j.from, to: j.to, killed });
    }
  }),
);
const by = {};
for (const r of results) (by[r.id] ??= { mutants: 0, survived: [] }).mutants++, r.killed || by[r.id].survived.push(`${r.line}: ${r.from}  =>  ${r.to}`);
const total = results.length;
const survived = results.filter((r) => !r.killed).length;
mkdirSync(join(web, '.gate'), { recursive: true });
writeFileSync(join(web, '.gate/mutants.json'), JSON.stringify(by, null, 2) + '\n');
const listFile = join(web, 'scripts/practice-mutants-equivalent.txt');
const listed = new Set(readFileSync(listFile, 'utf8').split('\n').filter((l) => l.trim() && !l.startsWith('#')).map((l) => l.replace(/\s+#.*$/, '')));
const key = (r) => `${r.id}: ${r.from}  =>  ${r.to}`;
const live = new Set(results.filter((r) => !r.killed).map(key));
const fresh = [...live].filter((k) => !listed.has(k));
const stale = [...listed].filter((k) => !live.has(k) && (!only.length || only.includes(k.split(':')[0])));
if (fresh.length) console.error(`mutants: ${fresh.length} mutants of a practice ref.cp pass every test; add a test to its tests.json that fails them:\n  ${fresh.join('\n  ')}`);
// A listed mutant that a fix now kills is dropped here rather than failing
// the build (ci/R30): the change that fixed it commits the shorter list.
if (stale.length) {
  const keep = readFileSync(listFile, 'utf8').split('\n').filter((l) => !stale.includes(l.replace(/\s+#.*$/, '')));
  writeFileSync(listFile, keep.join('\n'));
  console.error(`mutants: ${stale.length} mutants in web/scripts/practice-mutants-equivalent.txt are killed now; dropped them, commit it:\n  ${stale.join('\n  ')}`);
}
console.log(`mutants: ${total} mutants that check, ${total - survived} killed (${((100 * (total - survived)) / Math.max(total, 1)).toFixed(1)}%), ${survived} survive, over ${Object.keys(by).length} problems`);
if (fresh.length) process.exit(1);
