// Enforces site/R13 (handbook/guidelines/site-checklist.md): a link to any page,
// shared on a social site or in a chat app, previews with that page's title,
// its description and the site's image.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const out = join(import.meta.dirname, '..', 'out');
const pages = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? pages(join(dir, e.name)) : e.name.endsWith('.html') ? [join(dir, e.name)] : [],
  );
const meta = (html, key) => new RegExp(`<meta (?:property|name)="${key}" content="([^"]*)"`).exec(html)?.[1];
const bad = [];
// The debugger's own HTML output, served as `cpl trace` writes it.
const traces = ['/trace.html', '/trace-demo.html'];
const all = pages(out).filter((f) => !traces.includes(f.slice(out.length)));
for (const file of all) {
  const html = readFileSync(file, 'utf8');
  const where = file.slice(out.length);
  const missing = ['og:title', 'og:description', 'og:image', 'twitter:card'].filter((k) => !meta(html, k));
  if (missing.length) bad.push(`${where}: no ${missing.join(', ')}`);
  const image = meta(html, 'og:image');
  if (image && !existsSync(join(out, new URL(image, 'https://cp-lang.org').pathname))) bad.push(`${where}: og:image ${image} is not in the export`);
}
if (bad.length) {
  console.error(`social: ${bad.length} of ${all.length} pages don't preview (site/R13):\n${bad.slice(0, 20).join('\n')}`);
  process.exit(1);
}
console.log(`social: all ${all.length} pages preview with a title, a description and an image`);
