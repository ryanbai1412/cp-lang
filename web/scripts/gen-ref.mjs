// Enforces site/R1, site/R3, site/R4 (handbook/guidelines/site-checklist.md).
// Renders the reference under content/docs/ref/ (gitignored): spec/core,
// spec/lib and spec/tools as-is, and the library from `cpl doc --format json`.
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import katex from 'katex';
import { commonSetup, layoutSignature, mathSpans, plainSpans, splitTop, texSpans } from './doc-examples.mjs';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cplBin } from './cpl-bin.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const out = join(web, 'content/docs/ref');
const blob = 'https://github.com/ryanbai1412/cp-lang/blob/main';
const cpl = cplBin();

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

// What `cpl doc` read from an item's own doc (cpl_syntax::doc): its prose,
// its `cost:`/`traps:`/... facts in std/README.md order, and its `example:`
// lines. A field repeating its type's doc has none of its own.
function ownDoc(item, parentDoc) {
  if (!item.doc || item.doc === parentDoc) return { facts: [], examples: [], prose: '' };
  return {
    facts: Object.entries(item.facts).filter(([, v]) => v !== null).sort(([a], [b]) => factOrder.indexOf(a) - factOrder.indexOf(b)),
    examples: item.examples.filter((e) => !e.block),
    prose: item.prose,
  };
}
const factOrder = ['cost', 'traps', 'empty', 'ties', 'order'];

// Each `example:` line becomes a program: the module's import and the setup
// its type's examples share (collapsed), then its own statements and
// `print(expr)`; when the value can't be printed, `assert(expr == value)`,
// or `assert(expr)` for an example with no value, a claim that holds.
// Only a program that runs is shown runnable; the rest are listed.
function programs(e, module, shared) {
  const ownsShared = shared.length && shared.every((st, i) => e.setup[i] === st);
  const setup = [...(module === 'builtin' ? [] : [`use std.${module}.*`]), ...(ownsShared ? shared : [])].join('\n');
  const own = e.setup.slice(ownsShared ? shared.length : 0);
  // An `example:` that is prose, not code, runs nothing and is listed.
  const claim = e.value ? `assert(${e.expr} == ${e.value})` : `assert(${e.expr})`;
  // One tagged `(traps R0001)` or `(error E0309)` must stop with that code.
  if (e.code !== null && e.expect) return { e, setup, own, bodies: [[...own, claim].join('\n')] };
  const bodies = e.code === null ? [] : [[...own, `print(${e.expr})`]];
  if (e.code === null);
  else if (e.value) bodies.push([...own, `assert(${e.expr} == ${e.value})`]);
  else bodies.push([...own, `assert(${e.expr})`]);
  return { e, setup, own, bodies: bodies.map((b) => b.join('\n')) };
}

// What runAll runs: the program, then its stated stdin after a NUL, then the code
// it must stop with after another.
const runKey = (setup, body, stdin, expect) =>
  `${setup}\n${body}\n${stdin || expect ? `\0${stdin ?? ''}` : ''}${expect ? `\0${expect}` : ''}`;

function renderExample(example, module, shared, ran, failing, at) {
  const { e, setup, own, bodies } = programs(example, module, shared);
  const body = bodies.find((b) => ran.get(runKey(setup, b, e.stdin, e.expect)));
  const result = e.value ? ['', `Result: \`${e.value}\``] : [];
  const input = e.stdin ? ` input="${e.stdin.replace(/\n/g, '\\n')}"` : '';
  if (body && e.expect) return [`\`\`\`cp rejects=${e.expect}${input}`, setup, body, '```', `Stops with \`${e.expect}\``, ''];
  if (body) return [`\`\`\`cp run compact${input}`, setup, '//---', body, '```', ...result, ''];
  failing.push(`${at}: ${e.text}`);
  if (e.code === null) return [e.text, ''];
  return ['```cp', ...own, e.expr, '```', ...result, ''];
}

// The first sentence of a module's `//!` comment.
const moduleSummary = (m) => (m.prose ?? m.doc ?? '').split(/\n\s*\n/)[0].replace(/\s+/g, ' ').trim();

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
  if (!/^(pub )?(fn|law)\b/.test(sig)) return [];
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

// A signature's parameters other than `self`, split at top-level commas.
function ownParams(sig) {
  const start = sig.indexOf('(');
  if (start < 0) return 0;
  const parts = [''];
  let depth = 0;
  for (let i = start; i < sig.length; i++) {
    const c = sig[i];
    if ('([<'.includes(c)) depth++;
    else if (')]>'.includes(c) && sig[i - 1] !== '-' && --depth === 0) break;
    if (depth === 1 && c === ',') parts.push('');
    else if (i > start) parts[parts.length - 1] += c;
  }
  return parts.map((p) => p.trim()).filter((p) => p && !/^(mut\s+)?self\b/.test(p)).length;
}

function renderItem(item, level, prefix, parentDoc, index, module, ctx, shared = []) {
  const name = prefix ? `${prefix}.${item.name}` : item.name;
  const { facts, examples, prose } = ownDoc(item, parentDoc);
  const plain = (text, cost = false) => linked(plainSpans(texSpans(text, { legacyCost: cost })), item.links ?? []);
  const hover = [prose && plain(prose), ...facts.map(([k, v]) => `${k}: ${plain(v, k === 'cost')}`)].filter(Boolean).join('\n\n');
  ctx.anchors.push([module, item.line, slug(name), hover]);
  // site/R4 (handbook/guidelines/site-checklist.md): an entry is a highlighted
  // signature, a parameter table, prose, then labelled rows; never one blob.
  const where = `${module}.${name}`;
  if (!item.signature?.trim()) ctx.unstructured.push(`${where}: no signature`);
  for (const l of prose.split('\n'))
    if (/^\s*(cost|empty|order|ties|traps)\s*:/i.test(l)) ctx.unstructured.push(`${where}: a \`${l.trim().split(':')[0]}:\` line left in the prose`);
  const lines = [`${'#'.repeat(level)} \`${name}\``, '', '```cp', layoutSignature(item.signature), '```', ''];
  if (prose) lines.push(linked(ctx.math(prose, name), item.links ?? []), '');
  const params = item.kind === 'fn' ? sigParams(item.signature) : [];
  if (item.kind === 'fn' && !params.length && ownParams(item.signature ?? ''))
    ctx.unstructured.push(`${where}: its parameters don't parse into a table: ${item.signature}`);
  if (params.length) {
    lines.push('| Parameter | Type | Default |', '|---|---|---|');
    for (const [n, t, d] of params) lines.push(`| ${cell(n)} | ${cell(t)} | ${d ? cell(d) : 'required'} |`);
    lines.push('');
  }
  // What spec/lib/inventory.md says it is called in C++, Python, Rust or ACL.
  const elsewhere = (item.aliases ?? []).map(cell).join(', ');
  if (facts.length || elsewhere) {
    lines.push('| | |', '|---|---|');
    for (const [k, v] of facts)
      lines.push(`| **${k[0].toUpperCase()}${k.slice(1)}** | ${linked(ctx.math(v, name, k === 'cost'), item.links ?? []).replace(/\|/g, '\\|')} |`);
    if (elsewhere) lines.push(`| **Elsewhere** | ${elsewhere} |`);
    lines.push('');
  }
  if (examples.length) {
    lines.push(examples.length > 1 ? 'Examples:' : 'Example:', '');
    for (const e of examples) lines.push(...renderExample(e, module, shared, ctx.ran, ctx.failing, `${module}.${name}`));
  }
  lines.push(`[Source](${blob}/${ctx.paths[module]}${item.line ? `#L${item.line}` : ''})`, '');
  if (item.indexed) {
    index.push({ name, module, kind: item.kind, cost: facts.find(([k]) => k === 'cost')?.[1] ?? '' });
  }
  const memberShared = item.members?.length ? commonSetup(memberExamples(item)) : [];
  for (const m of item.members ?? []) {
    if (m.kind === 'field' && (!m.doc || m.doc === item.doc)) continue;
    lines.push(renderItem(m, Math.min(level + 1, 4), item.name, item.doc, index, module, ctx, memberShared));
  }
  return lines.join('\n');
}

const memberExamples = (item) => (item.members ?? []).flatMap((m) => ownDoc(m, item.doc).examples);

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
      const [program, stdin = '', expect] = src.split('\0');
      writeFileSync(file, program);
      const p = spawn(cpl, ['run', file], { stdio: ['pipe', 'ignore', expect ? 'pipe' : 'ignore'] });
      let stderr = '';
      p.stderr?.on('data', (d) => (stderr += d));
      p.stdin.on('error', () => {});
      p.stdin.end(stdin);
      const timer = setTimeout(() => p.kill(), 10_000);
      p.on('close', (code) => {
        clearTimeout(timer);
        cache[key(src)] = expect ? code !== 0 && stderr.includes(`[${expect}]`) : code === 0;
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
// What a program can name: std's `pub` items, and the types the language
// itself declares (the `builtin` module, from the compiler's catalog).
// site/R22: a module page opens with what the module is for, the 3 to 5 entry points its
// `start:` names (each with its summary and cost), one example that runs, then the rest of
// its prose; the per-item reference follows.
function outline(m, ctx) {
  const [purpose, ...rest] = (m.prose ?? '').split(/\n\s*\n/);
  const entries = (m.start ?? []).map((name) => {
    const [owner, member] = name.split('.');
    const tops = m.items.filter((i) => i.name === owner && (isPublic(i) || i.kind === 'impl'));
    const top = tops.find((i) => i.kind !== 'impl');
    return [name, member ? tops.flatMap((i) => i.members ?? []).find((x) => x.name === member) : top];
  });
  const missing = entries.filter(([, item]) => !item).map(([name]) => name);
  for (const [name, item] of entries)
    if (item && (!item.summary?.trim() || /^\\\(O\(/.test(item.summary)))
      ctx.unstructured.push(`${m.name}.${name}: its summary doesn't say what it is (site/R22): ${item.summary || 'none'}`);
  if (!purpose?.trim() || entries.length < 3 || entries.length > 5 || missing.length || !m.examples?.length)
    ctx.unstructured.push(`${m.name}: no outline (site/R22): purpose, \`start:\` with 3 to 5 items${missing.length ? ` (no item ${missing.join(', ')})` : ''}, and an \`example:\``);
  const lines = [linked(ctx.math(purpose ?? '', m.name), m.links ?? []), '', '## Start here', ''];
  for (const [name, item] of entries) {
    if (!item) continue;
    const text = (t) => linked(ctx.math(t.replace(/\s+/g, ' '), `${m.name}.${name}`), item.links ?? []);
    const cost = item.facts?.cost ? ` Cost: ${text(item.facts.cost)}.` : '';
    lines.push(`- [\`${name}\`](#${slug(name)}): ${text(item.summary ?? '')}${cost}`);
  }
  lines.push('', '## Example', '');
  if (m.examples?.length) lines.push(...renderExample(m.examples[0], m.name, [], ctx.ran, ctx.failing, `${m.name}#module`));
  const more = rest.join('\n\n').trim();
  if (more) lines.push('## Overview', '', linked(ctx.math(more, m.name), m.links ?? []), '');
  return lines;
}

const isPublic = (i) => i.kind === 'builtin' || /^pub\b/.test(i.signature);
const modules = std.modules.filter((m) => m.items.some(isPublic)).sort((a, b) => a.name.localeCompare(b.name));
// Each example's programs, in the order renderExample tries them.
const tries = [];
const collect = (item, module, parentDoc, shared) => {
  for (const e of ownDoc(item, parentDoc).examples) {
    const { e: ex, setup, bodies } = programs(e, module, shared);
    tries.push(bodies.map((b) => runKey(setup, b, ex.stdin, ex.expect)));
  }
  const memberShared = item.members?.length ? commonSetup(memberExamples(item)) : [];
  for (const m of item.members ?? []) collect(m, module, item.doc, memberShared);
};
for (const m of modules) {
  for (const e of m.examples ?? []) {
    const { e: ex, setup, bodies } = programs(e, m.name, []);
    tries.push(bodies.map((b) => runKey(setup, b, ex.stdin, ex.expect)));
  }
  for (const i of m.items) collect(i, m.name, '', []);
}
const ctx = {
  ran: await runInTurn(tries),
  failing: [],
  unstructured: [],
  anchors: [],
  paths: Object.fromEntries(std.modules.map((m) => [m.name, m.path])),
  math(text, where, cost = false) {
    return checkMath(where, texSpans(text, { legacyCost: cost }));
  },
};
for (const m of modules) {
  const fields = m.items.filter((i) => i.kind === 'field' || (i.kind !== 'impl' && !isPublic(i)));
  const body = [
    ...outline(m, ctx),
    '## Reference',
    '',
    `Generated from [${m.path}](${blob}/${m.path}) by \`cpl doc --format json\`.`,
    '',
    ...m.items.filter((i) => !fields.includes(i)).map((i) => renderItem(i, 3, '', '', index, m.name, ctx)),
  ].join('\n');
  write(`std/${m.name}.md`, page(m.name, moduleSummary(m) || `The ${m.name} module.`, body));
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
      ...modules.map((m) => `- [${m.name}](/docs/ref/std/${m.name}) (${m.items.length} items)${moduleSummary(m) ? `: ${linked(plainSpans(texSpans(moduleSummary(m))), m.links ?? [])}` : ''}`),
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
// handbook/guidelines/site-checklist.md site/R1: every std example runs; one
// that can't is a std bug to fix, or an example rewritten to show its trap.
if (ctx.failing.length) console.error(`gen-ref: ${ctx.failing.length} std doc examples don't run (fix them, or the std docs):\n${ctx.failing.join('\n')}`);
if (ctx.unstructured.length) console.error(`gen-ref: ${ctx.unstructured.length} std entries don't render as structure (site/R4):\n${ctx.unstructured.slice(0, 20).join('\n')}`);
if (ctx.failing.length || ctx.unstructured.length) process.exit(1);
if (mathErrors.length) {
  console.error(`gen-ref: ${mathErrors.length} math spans KaTeX can't render:\n${mathErrors.slice(0, 20).join('\n')}`);
  process.exit(1);
}
