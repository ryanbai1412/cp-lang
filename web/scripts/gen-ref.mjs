// Renders the reference under content/docs/ref/ (gitignored): spec/core,
// spec/lib and spec/tools as-is, and the library from `cpl doc --format json`.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
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

const fenceLangs = new Set(['cp', 'cpp', 'java', 'rust', 'sh', 'json', 'text']);
let specPages = 0;
for (const [dir, title, description] of specDirs) {
  const path = join(repo, 'spec', dir);
  if (!existsSync(path)) continue;
  const files = readdirSync(path).filter((f) => f.endsWith('.md') && f !== 'README.md').sort();
  for (const f of files) {
    const src = readFileSync(join(path, f), 'utf8');
    const heading = /^# (.+)$/m.exec(src)?.[1] ?? slugOf(f);
    const body = rewriteLinks(src.replace(/^# .+\n/m, ''), dir).replace(/^```(\w+)/gm, (_, lang) => '```' + (lang === 'python' ? 'cp' : fenceLangs.has(lang) ? lang : 'text'));
    const note =
      dir === 'core'
        ? `This is the normative specification ([source](${blob}/spec/${dir}/${f})). For a gentler introduction, start with the [guides](/docs/guides).\n\n`
        : `Normative text from [spec/${dir}/${f}](${blob}/spec/${dir}/${f}).\n\n`;
    write(`${dir}/${slugOf(f)}.md`, page(heading, description, note + body));
    specPages++;
  }
  write(`${dir}/meta.json`, JSON.stringify({ title, pages: files.map(slugOf) }, null, 2));
}

const std = JSON.parse(
  execFileSync(cpl, ['doc', '--format', 'json', 'spec/std'], {
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

function renderItem(item, level, prefix, parentDoc, index, module) {
  const name = prefix ? `${prefix}.${item.name}` : item.name;
  const own = item.doc && item.doc !== parentDoc ? item.doc : '';
  const { facts, examples, prose } = splitDoc(own);
  const lines = [`${'#'.repeat(level)} \`${name}\``, '', '```cp', item.signature, '```', ''];
  if (prose) lines.push(prose, '');
  if (facts.length) {
    for (const [k, v] of facts) lines.push(`- **${k}**: ${v}`);
    lines.push('');
  }
  if (examples.length) {
    lines.push('Examples:', '');
    for (const e of examples) lines.push(`- ${e}`);
    lines.push('');
  }
  lines.push(`[Source](${blob}/spec/std/${module}.cp#L${item.line})`, '');
  if (item.kind === 'fn' || item.kind === 'struct' || item.kind === 'enum' || item.kind === 'trait') {
    index.push({ name, module, kind: item.kind, cost: facts.find(([k]) => k === 'cost')?.[1] ?? '' });
  }
  for (const m of item.members ?? []) {
    if (m.kind === 'field' && (!m.doc || m.doc === item.doc)) continue;
    lines.push(renderItem(m, Math.min(level + 1, 4), item.name, item.doc, index, module));
  }
  return lines.join('\n');
}

const index = [];
const modules = std.modules.filter((m) => m.items.some((i) => /^pub\b/.test(i.signature))).sort((a, b) => a.name.localeCompare(b.name));
for (const m of modules) {
  const fields = m.items.filter((i) => i.kind === 'field' || (i.kind !== 'impl' && !/^pub\b/.test(i.signature)));
  const body = [
    `Generated from [spec/std/${m.name}.cp](${blob}/spec/std/${m.name}.cp) by \`cpl doc --format json\`. The 0.3 library is specified but not yet implemented by the 0.2 compiler.`,
    '',
    m.doc,
    '',
    ...m.items.filter((i) => !fields.includes(i)).map((i) => renderItem(i, 2, '', '', index, m.name)),
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
      `Generated from \`spec/std\` by \`cpl doc --format json\`. Costs and traps follow the [doc standard](${blob}/spec/std/README.md).`,
      '',
      '## Modules',
      '',
      ...modules.map((m) => `- [${m.name}](/docs/ref/std/${m.name}) (${m.items.length} items)`),
      '',
      '## A to Z',
      '',
      '| Item | Module | Cost |',
      '| --- | --- | --- |',
      ...index.map((i) => `| \`${esc(i.name)}\` | [${i.module}](/docs/ref/std/${i.module}) | ${esc(i.cost)} |`),
    ].join('\n'),
  ),
);
write('std/meta.json', JSON.stringify({ title: 'Library', pages: ['index', ...modules.map((m) => m.name)] }, null, 2));
write(
  'meta.json',
  JSON.stringify({ title: 'Reference', pages: ['core', 'std', ...specDirs.slice(1).map(([d]) => d).filter((d) => existsSync(join(out, d)))] }, null, 2),
);
console.log(`ref: ${specPages} spec pages, ${modules.length} library modules, ${index.length} items`);
