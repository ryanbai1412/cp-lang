// Enforces site/R1, site/R3, site/R4 (handbook/guidelines/site-checklist.md).
// Renders the reference under content/docs/ref/ (gitignored): spec/core,
// spec/lib and spec/tools as-is, and the library from `cpl doc --format json`.
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import katex from 'katex';
import { commonSetup, layoutSignature, mathSpans, parseExample, plainSpans, splitTop, texSpans } from './doc-examples.mjs';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const out = join(web, 'content/docs/ref');
const blob = 'https://github.com/ryanbai1412/cp-lang/blob/main';
const cpl =
  process.env.CPL ??
  ['target/release/cpl', 'target/debug/cpl'].map((p) => join(repo, p)).find(existsSync) ??
  'cpl';

rmSync(out, { recursive: true, force: true });
const write = (rel, text) => {
  mkdirSync(dirname(join(out, rel)), { recursive: true });
  writeFileSync(join(out, rel), text);
};
const yaml = (s) => JSON.stringify(s.replace(/`/g, ''));
const page = (title, description, body) =>
  `---\ntitle: ${yaml(title)}\ndescription: ${yaml(description)}\n---\n\n${body.trim()}\n`;

// Every math span must be LaTeX KaTeX can render; the build fails otherwise.
const mathErrors = [];
function checkMath(where, text) {
  for (const tex of mathSpans(text)) {
    try {
      katex.renderToString(tex, { throwOnError: true });
    } catch (e) {
      mathErrors.push(`${where}: \\(${tex}\\): ${e.message.split('\n')[0]}`);
    }
  }
  return text;
}

const specDirs = [
  ['core', 'Language', 'The normative CP Lang 0.3 specification.'],
  ['lib', 'Library families', 'How the library is organised, family by family.'],
  ['tools', 'Tools', 'The normative description of each tool.'],
];
const slugOf = (file) => basename(file, '.md');
const known = new Map();
for (const [dir] of specDirs) {
  const path = join(repo, 'spec', dir);
  if (!existsSync(path)) continue;
  for (const f of readdirSync(path)) if (f.endsWith('.md')) known.set(`${dir}/${f}`, `/docs/ref/${dir}/${slugOf(f)}`);
}

function rewriteLinks(text, dir) {
  return text.replace(/\]\((?!https?:|#|mailto:)([^)\s]+)\)/g, (_, target) => {
    const [path, hash = ''] = target.split('#');
    const parts = join(dir, path).split('/');
    const norm = parts.join('/');
    const local = known.get(norm);
    if (local) return `](${local}${hash ? `#${hash}` : ''})`;
    return `](${blob}/spec/${norm}${hash ? `#${hash}` : ''})`;
  });
}

// Each Language chapter that cites a spec/core chapter with <Spec> explains
// why that part is designed the way it is; the reference page links there.
const learnDir = join(web, 'content/learn/language');
const why = new Map();
for (const f of readdirSync(learnDir).filter((f) => f.endsWith('.mdx') && f !== 'index.mdx').sort()) {
  const src = readFileSync(join(learnDir, f), 'utf8');
  if (!src.includes("## Why it's designed this way")) continue;
  const title = /^title: "(.+)"$/m.exec(src)?.[1] ?? basename(f, '.mdx');
  for (const [, core] of src.matchAll(/<Spec file="core\/([0-9a-z-]+)"/g)) {
    const list = why.get(core) ?? [];
    const link = `[${title}](/learn/language/${basename(f, '.mdx')}#why-its-designed-this-way)`;
    if (!list.includes(link)) why.set(core, [...list, link]);
  }
}

const chapters = (links) =>
  links.length === 1 ? `the Language chapter ${links[0]}` : `the Language chapters ${links.slice(0, -1).join(', ')} and ${links.at(-1)}`;

const fenceLangs = new Set(['cp', 'cpp', 'java', 'rust', 'sh', 'json', 'text']);
let specPages = 0;
for (const [dir, title, description] of specDirs) {
  const path = join(repo, 'spec', dir);
  if (!existsSync(path)) continue;
  const files = readdirSync(path).filter((f) => f.endsWith('.md') && f !== 'README.md').sort();
  for (const f of files) {
    const src = readFileSync(join(path, f), 'utf8');
    const heading = /^# (.+)$/m.exec(src)?.[1] ?? slugOf(f);
    const body = checkMath(`spec/${dir}/${f}`, texSpans(rewriteLinks(src.replace(/^# .+\n/m, ''), dir))).replace(/^```(\w+)/gm, (_, lang) => '```' + (lang === 'python' ? 'cp' : fenceLangs.has(lang) ? lang : 'text'));
    const note =
      dir === 'core'
        ? `This is the normative specification ([source](${blob}/spec/${dir}/${f})). For a gentler introduction, start with the [tour](/docs/guides/tour).${why.has(slugOf(f)) ? ` The alternatives considered and why this design won are in ${chapters(why.get(slugOf(f)))}.` : ''}\n\n`
        : `Normative text from [spec/${dir}/${f}](${blob}/spec/${dir}/${f}).\n\n`;
    write(`${dir}/${slugOf(f)}.md`, page(heading, description, note + body));
    specPages++;
  }
  write(`${dir}/meta.json`, JSON.stringify({ title, pages: files.map(slugOf) }, null, 2));
}

const std = JSON.parse(
  execFileSync(cpl, ['doc', '--format', 'json', 'std'], {
    cwd: repo,
    encoding: 'utf8',
    maxBuffer: 64 << 20,
  }),
);

const labelNames = ['cost', 'traps', 'empty', 'ties', 'order'];
function splitDoc(doc) {
  const facts = [];
  const examples = [];
  const prose = [];
  for (const line of doc.split('\n')) {
    const m = /^([a-z]+):\s+(.*)$/.exec(line);
    if (m && labelNames.includes(m[1])) facts.push([m[1], m[2]]);
    else if (m && m[1] === 'example') examples.push(m[2]);
    else if (m && facts.length && !line.trim()) prose.push('');
    else prose.push(line);
  }
  return { facts, examples, prose: prose.join('\n').trim() };
}

// Each `example:` line becomes a program: the module's import and the setup
// its type's examples share (collapsed), then its own statements and
// `print(expr)`; when the value can't be printed, `assert(expr == value)`,
// or `assert(expr)` for an example with no value, a claim that holds.
// Only a program that runs is shown runnable; the rest are listed.
function programs(text, module, shared) {
  const e = parseExample(text);
  const ownsShared = shared.length && shared.every((st, i) => e.setup[i] === st);
  const setup = [`use std.${module}.*`, ...(ownsShared ? shared : [])].join('\n');
  const own = e.setup.slice(ownsShared ? shared.length : 0);
  const bodies = [[...own, `print(${e.expr})`]];
  if (e.value) bodies.push([...own, `assert(${e.expr} == ${e.value})`]);
  else bodies.push([...own, `assert(${e.expr})`]);
  return { e, setup, own, bodies: bodies.map((b) => b.join('\n')) };
}

function renderExample(text, module, shared, ran, failing, at) {
  const { e, setup, own, bodies } = programs(text, module, shared);
  const body = bodies.find((b) => ran.get(`${setup}\n${b}\n`));
  const result = e.value ? ['', `Result: \`${e.value}\``] : [];
  if (body) return ['```cp run compact', setup, '//---', body, '```', ...result, ''];
  failing.push(`${at}: ${text}`);
  return ['```cp', ...own, e.expr, '```', ...result, ''];
}

// The heading id fumadocs gives `### \`Rooted.up\``.
const slug = (anchor) => anchor.toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu, '').replace(/ /g, '-');

// [`up`] and [`text`](Rooted.up) become links to the item `cpl doc` resolved
// them to; plain `up` stays code, and a link to a URL is left alone.
function linked(text, links) {
  const to = new Map(links.map((l) => [`${l.text}\0${l.path}`, `/docs/ref/std/${l.module}#${slug(l.anchor)}`]));
  return text.replace(/\[`([^`\]]+)`\](?:\(([\w.]+)\))?/g, (all, name, path) => {
    const url = to.get(`${name}\0${path ?? name}`);
    return url ? `[\`${name}\`](${url})` : all;
  });
}

// The parameters after `self` of a `fn` signature, as `[name, type, default]`.
function sigParams(sig) {
  if (!/^(pub )?fn\b/.test(sig)) return [];
  const open = sig.indexOf('(');
  let depth = 0;
  let close = -1;
  for (let i = open; open >= 0 && i < sig.length; i++) {
    if ('([{'.includes(sig[i])) depth++;
    else if (')]}'.includes(sig[i]) && --depth === 0) {
      close = i;
      break;
    }
  }
  if (close < 0) return [];
  return splitTop(sig.slice(open + 1, close), ',')
    .map((p) => p.trim())
    .filter((p) => p && !/^(mut )?self\b/.test(p))
    .map((p) => {
      const [head, ...dflt] = splitTop(p, ' = ');
      const colon = head.indexOf(':');
      return colon < 0 ? [head, '', dflt.join(' = ')] : [head.slice(0, colon).trim(), head.slice(colon + 1).trim(), dflt.join(' = ')];
    });
}

const cell = (code) => (code ? `\`${code.replace(/\|/g, '\\|')}\`` : '');

function renderItem(item, level, prefix, parentDoc, index, module, ctx, shared = []) {
  const name = prefix ? `${prefix}.${item.name}` : item.name;
  const own = item.doc && item.doc !== parentDoc ? item.doc : '';
  const { facts, examples, prose } = splitDoc(own);
  const plain = (text, cost = false) => linked(plainSpans(texSpans(text, { legacyCost: cost })), item.links ?? []);
  const hover = [prose && plain(prose), ...facts.map(([k, v]) => `${k}: ${plain(v, k === 'cost')}`)].filter(Boolean).join('\n\n');
  ctx.anchors.push([module, item.line, slug(name), hover]);
  const lines = [`${'#'.repeat(level)} \`${name}\``, '', '```cp', layoutSignature(item.signature), '```', ''];
  if (prose) lines.push(linked(ctx.math(prose, name), item.links ?? []), '');
  const params = item.kind === 'fn' ? sigParams(item.signature) : [];
  if (params.length) {
    lines.push('| Parameter | Type | Default |', '|---|---|---|');
    for (const [n, t, d] of params) lines.push(`| ${cell(n)} | ${cell(t)} | ${d ? cell(d) : 'required'} |`);
    lines.push('');
  }
  if (facts.length) {
    lines.push('| | |', '|---|---|');
    for (const [k, v] of facts)
      lines.push(`| **${k[0].toUpperCase()}${k.slice(1)}** | ${linked(ctx.math(v, name, k === 'cost'), item.links ?? []).replace(/\|/g, '\\|')} |`);
    lines.push('');
  }
  if (examples.length) {
    lines.push(examples.length > 1 ? 'Examples:' : 'Example:', '');
    for (const e of examples) lines.push(...renderExample(e, module, shared, ctx.ran, ctx.failing, `${module}.${name}`));
  }
  lines.push(`[Source](${blob}/std/${module}.cp#L${item.line})`, '');
  if (item.kind === 'fn' || item.kind === 'struct' || item.kind === 'enum' || item.kind === 'trait') {
    index.push({ name, module, kind: item.kind, cost: facts.find(([k]) => k === 'cost')?.[1] ?? '' });
  }
  const memberShared = item.members?.length ? commonSetup(memberExamples(item).map(parseExample)) : [];
  for (const m of item.members ?? []) {
    if (m.kind === 'field' && (!m.doc || m.doc === item.doc)) continue;
    lines.push(renderItem(m, Math.min(level + 1, 4), item.name, item.doc, index, module, ctx, memberShared));
  }
  return lines.join('\n');
}

const memberExamples = (item) => (item.members ?? []).flatMap((m) => (m.doc && m.doc !== item.doc ? splitDoc(m.doc).examples : []));

// Runs every example program once, 16 at a time, remembering results by hash.
async function runAll(sources) {
  const cacheFile = join(web, 'node_modules/.cache/doc-examples.json');
  const cache = existsSync(cacheFile) ? JSON.parse(readFileSync(cacheFile, 'utf8')) : {};
  // A rebuilt cpl answers anew: the key names the binary, not just its path.
  const built = existsSync(cpl) ? `${statSync(cpl).mtimeMs}:${statSync(cpl).size}` : '';
  const key = (src) => createHash('sha1').update(`${cpl}\0${built}\0${src}`).digest('hex');
  const todo = [...sources].filter((src) => !(key(src) in cache));
  const dir = join(web, 'node_modules/.cache/doc-examples');
  mkdirSync(dir, { recursive: true });
  let next = 0;
  const one = (src, i) =>
    new Promise((done) => {
      const file = join(dir, `${i % 16}.cp`);
      writeFileSync(file, src);
      const p = spawn(cpl, ['run', file], { stdio: 'ignore' });
      const timer = setTimeout(() => p.kill(), 10_000);
      p.on('close', (code) => {
        clearTimeout(timer);
        cache[key(src)] = code === 0;
        done();
      });
    });
  await Promise.all(Array.from({ length: 16 }, async (_, w) => {
    while (next < todo.length) {
      const i = next++;
      await one(todo[i], w);
    }
  }));
  mkdirSync(dirname(cacheFile), { recursive: true });
  writeFileSync(cacheFile, JSON.stringify(cache));
  return new Map([...sources].map((src) => [src, cache[key(src)]]));
}

// Runs each example's first program, and the next only where the one before
// failed: most print, so the assert fallbacks rarely run.
async function runInTurn(tries) {
  const ran = new Map();
  for (let turn = 0; ; turn++) {
    const next = new Set(tries.filter((t) => t.length > turn && !t.slice(0, turn).some((src) => ran.get(src))).map((t) => t[turn]));
    if (!next.size) return ran;
    for (const [src, ok] of await runAll(next)) ran.set(src, ok);
  }
}

const index = [];
const modules = std.modules.filter((m) => m.items.some((i) => /^pub\b/.test(i.signature))).sort((a, b) => a.name.localeCompare(b.name));
// Each example's programs, in the order renderExample tries them.
const tries = [];
const collect = (item, module, parentDoc, shared) => {
  const own = item.doc && item.doc !== parentDoc ? item.doc : '';
  for (const e of splitDoc(own).examples) {
    const { setup, bodies } = programs(e, module, shared);
    tries.push(bodies.map((b) => `${setup}\n${b}\n`));
  }
  const memberShared = item.members?.length ? commonSetup(memberExamples(item).map(parseExample)) : [];
  for (const m of item.members ?? []) collect(m, module, item.doc, memberShared);
};
for (const m of modules) for (const i of m.items) collect(i, m.name, '', []);
const ctx = {
  ran: await runInTurn(tries),
  failing: [],
  anchors: [],
  math(text, where, cost = false) {
    return checkMath(where, texSpans(text, { legacyCost: cost }));
  },
};
for (const m of modules) {
  const fields = m.items.filter((i) => i.kind === 'field' || (i.kind !== 'impl' && !/^pub\b/.test(i.signature)));
  const body = [
    `Generated from [std/${m.name}.cp](${blob}/std/${m.name}.cp) by \`cpl doc --format json\`.`,
    '',
    linked(m.doc, m.links ?? []),
    '',
    ...m.items.filter((i) => !fields.includes(i)).map((i) => renderItem(i, 2, '', '', index, m.name, ctx)),
  ].join('\n');
  const summary = m.items.find((i) => i.kind !== 'field')?.summary ?? '';
  write(`std/${m.name}.md`, page(m.name, summary || `The ${m.name} module.`, body));
}
index.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
const esc = (s) => s.replace(/\|/g, '\\|');
write(
  'std/index.md',
  page(
    'Library',
    `Every public item in the ${modules.length} modules of the 0.3 standard library.`,
    [
      `Generated from \`std\` by \`cpl doc --format json\`. Costs and traps follow the [doc standard](${blob}/std/README.md).`,
      '',
      '## Modules',
      '',
      ...modules.map((m) => `- [${m.name}](/docs/ref/std/${m.name}) (${m.items.length} items)`),
      '',
      '## A to Z',
      '',
      '| Item | Module | Cost |',
      '| --- | --- | --- |',
      ...index.map((i) => `| \`${esc(i.name)}\` | [${i.module}](/docs/ref/std/${i.module}) | ${esc(plainSpans(texSpans(i.cost, { legacyCost: true })))} |`),
    ].join('\n'),
  ),
);
write('std/meta.json', JSON.stringify({ title: 'Library', pages: ['index', ...modules.map((m) => m.name)] }, null, 2));
write(
  'meta.json',
  JSON.stringify({ title: 'Reference', pages: ['core', 'std', ...specDirs.slice(1).map(([d]) => d).filter((d) => existsSync(join(out, d)))] }, null, 2),
);
// Each std item's source line, page anchor and doc text, one file per module,
// for a site editor's hover and go to definition (the wasm build carries no
// std doc comments).
const items = {};
for (const [module, line, anchor, doc] of ctx.anchors) (items[module] ??= []).push([line, anchor, doc]);
rmSync(join(web, 'public/std-docs'), { recursive: true, force: true });
mkdirSync(join(web, 'public/std-docs'), { recursive: true });
for (const [module, list] of Object.entries(items)) {
  list.sort((a, b) => a[0] - b[0]);
  writeFileSync(join(web, `public/std-docs/${module}.json`), JSON.stringify(list));
}
const failingFile = join(web, 'node_modules/.cache/doc-examples-failing.txt');
writeFileSync(failingFile, ctx.failing.join('\n') + '\n');
console.log(`ref: ${specPages} spec pages, ${modules.length} library modules, ${index.length} items; examples: ${ctx.ran.size} programs, ${ctx.failing.length} shown static (listed in ${failingFile.slice(repo.length + 1)})`);
// handbook/guidelines/site-checklist.md site/R1: a std example that does not
// run is shown static only while it is listed here; the list only shrinks.
const knownFile = join(web, 'scripts/doc-examples-failing.txt');
const knownStatic = new Set(readFileSync(knownFile, 'utf8').split('\n').filter((l) => l.trim() && !l.startsWith('#')));
const failingNow = new Set(ctx.failing);
const newlyFailing = ctx.failing.filter((f) => !knownStatic.has(f));
const nowRunning = [...knownStatic].filter((f) => !failingNow.has(f));
if (newlyFailing.length) console.error(`gen-ref: ${newlyFailing.length} std doc examples don't run (fix them, or the std docs):\n${newlyFailing.join('\n')}`);
if (nowRunning.length) console.error(`gen-ref: ${nowRunning.length} std doc examples in ${knownFile.slice(repo.length + 1)} run now; delete their lines:\n${nowRunning.join('\n')}`);
if (newlyFailing.length || nowRunning.length) process.exit(1);
if (mathErrors.length) {
  console.error(`gen-ref: ${mathErrors.length} math spans KaTeX can't render:\n${mathErrors.slice(0, 20).join('\n')}`);
  process.exit(1);
}
