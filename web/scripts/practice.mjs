// Generates the practice table in content/learn/practice/index.mdx and the
// sidebar order in meta.json from each problem's frontmatter (`level`,
// `chapter`, `teaches`), so the two can't disagree with the problems or drift
// out of chapter order. Problems with the same chapter keep meta.json's order.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = join(dirname(dirname(fileURLToPath(import.meta.url))), 'content/learn/practice');
const metaPath = join(dir, 'meta.json');
const meta = JSON.parse(readFileSync(metaPath, 'utf8'));
const field = (fm, key, file) => {
  const m = new RegExp(`^${key}: (.+)$`, 'm').exec(fm);
  if (!m) throw new Error(`practice: ${file} has no \`${key}:\` in its frontmatter`);
  return m[1].startsWith('"') ? JSON.parse(m[1]) : m[1];
};
const problems = readdirSync(dir)
  .filter((f) => f.endsWith('.mdx') && f !== 'index.mdx')
  .map((f) => {
    const fm = /^---\n([\s\S]*?)\n---\n/.exec(readFileSync(join(dir, f), 'utf8'))[1];
    const slug = f.slice(0, -4);
    const chapter = Number(field(fm, 'chapter', f));
    if (!Number.isInteger(chapter)) throw new Error(`practice: ${f}: chapter must be a number`);
    return { slug, title: field(fm, 'title', f), level: field(fm, 'level', f), teaches: field(fm, 'teaches', f), chapter };
  });
const before = (s) => (meta.pages.indexOf(s) + 1 || Infinity);
problems.sort((a, b) => a.chapter - b.chapter || before(a.slug) - before(b.slug) || a.slug.localeCompare(b.slug));

const table = [
  '| Problem | Level | Teaches | After chapter |',
  '|---|---|---|---|',
  ...problems.map((p) => `| [${p.title}](/learn/practice/${p.slug}) | ${p.level} | ${p.teaches} | ${p.chapter} |`),
].join('\n');
const indexPath = join(dir, 'index.mdx');
const index = readFileSync(indexPath, 'utf8');
const next = index.replace(/\| Problem \|[^\n]*\n(?:\|[^\n]*\n?)*/, `${table}\n`);
if (next !== index) writeFileSync(indexPath, next);
const pages = problems.map((p) => p.slug);
if (JSON.stringify(pages) !== JSON.stringify(meta.pages)) writeFileSync(metaPath, `${JSON.stringify({ ...meta, pages })}\n`);
console.log(`practice: ${problems.length} problems`);
