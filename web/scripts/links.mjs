// Internal link check over the static build (web/out): every href="/..." on
// every page must resolve to a file. /p/* and /api/* are the Worker's.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const out = join(dirname(dirname(fileURLToPath(import.meta.url))), 'out');
const broken = new Map();
let links = 0;
const walk = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walk(join(dir, e.name)) : e.name.endsWith('.html') ? [join(dir, e.name)] : [],
  );
for (const page of walk(out)) {
  for (const [, href] of readFileSync(page, 'utf8').matchAll(/href="(\/[^"#?]*)/g)) {
    links++;
    if (/^\/(p|api)\//.test(href)) continue;
    const p = join(out, decodeURIComponent(href).replace(/\/$/, ''));
    if ([p, `${p}.html`, join(p, 'index.html')].some((f) => statSync(f, { throwIfNoEntry: false })?.isFile())) continue;
    if (!broken.has(href)) broken.set(href, []);
    broken.get(href).push('/' + relative(out, page));
  }
}
for (const [href, pages] of broken) console.error(`links: ${href} <- ${pages.length} pages, e.g. ${pages[0]}`);
console.log(`links: ${links} internal links, ${broken.size} broken targets`);
process.exit(broken.size ? 1 : 0);
