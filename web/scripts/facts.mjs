// Enforces site/R2, site/R8 (handbook/guidelines/site-checklist.md).
// Contradiction gate: facts several pages state must agree with one source.
// - install one-liners and the version string match lib/facts.json, whose
//   version matches the workspace Cargo.toml;
// - every name the stdlib guide's module table lists is documented in the
//   std reference;
// - retired spellings stay retired.
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const root = dirname(web);
import { contradictions } from './facts-rules.mjs';
const facts = JSON.parse(readFileSync(join(web, 'lib/facts.json'), 'utf8'));
const problems = [];
const say = (file, line, msg) => problems.push(`${relative(root, file)}:${line}: ${msg}`);

const cargo = /^\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m.exec(readFileSync(join(root, 'Cargo.toml'), 'utf8'))?.[1];
if (cargo !== facts.version) say(join(web, 'lib/facts.json'), 1, `version ${facts.version} but Cargo.toml says ${cargo}`);
if (!facts.edgeVersion.startsWith(`cpl ${facts.version}-edge+`)) say(join(web, 'lib/facts.json'), 1, 'edgeVersion does not match version');

const walk = (dir, ext) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? (e.name === 'node_modules' ? [] : walk(join(dir, e.name), ext)) : ext.test(e.name) ? [join(dir, e.name)] : [],
  );
const files = [
  ...walk(join(web, 'content'), /\.mdx?$/),
  ...walk(join(web, 'app'), /\.tsx?$/),
  ...walk(join(web, 'components'), /\.tsx?$/),
  join(root, 'CHANGELOG.md'),
];

for (const file of files)
  for (const { line, msg } of contradictions(readFileSync(file, 'utf8'), facts)) say(file, line, msg);

// The stdlib guide's module table names only documented items.
const refDir = join(web, 'content/docs/ref/std');
const documented = new Set(
  readdirSync(refDir)
    .filter((f) => f.endsWith('.md'))
    .flatMap((f) => [...readFileSync(join(refDir, f), 'utf8').matchAll(/^#{2,3} `([A-Za-z_]\w*)/gm)].map((m) => m[1])),
);
const guide = join(web, 'content/docs/guides/stdlib.mdx');
readFileSync(guide, 'utf8').split('\n').forEach((text, i) => {
  const row = /^\| `std\.\w+` \| ([^|]*) \|/.exec(text);
  if (!row) return;
  for (const [, name] of row[1].matchAll(/`([A-Za-z_]\w*)/g))
    if (!documented.has(name)) say(guide, i + 1, `module table lists \`${name}\`, which no std reference page documents`);
});

for (const p of problems) console.error(`facts: ${p}`);
console.log(`facts: ${files.length} files, ${problems.length} contradictions`);
// Every spec/core chapter is taught by some Language chapter.
const langDir = join(web, 'content/learn/language');
const lang = readdirSync(langDir).filter((f) => f.endsWith('.mdx')).map((f) => readFileSync(join(langDir, f), 'utf8')).join('\n');
for (const f of readdirSync(join(root, 'spec/core')).filter((f) => f.endsWith('.md'))) {
  const name = f.replace(/\.md$/, '');
  if (!lang.includes(`<Spec file="core/${name}"`)) say(join(root, 'spec/core', f), 1, `no Language chapter cites core/${name}`);
}

process.exit(problems.length ? 1 : 0);
