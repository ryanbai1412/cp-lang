// Internal link check over the static build (web/out): every href="/..." on
// every page must resolve to a file, and a #fragment to an id on that page.
// /p/* and /api/* are the Worker's.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const out = join(dirname(dirname(fileURLToPath(import.meta.url))), 'out');
const broken = new Map();
const ids = new Map();
const idsOf = (file) => {
  if (!ids.has(file)) ids.set(file, new Set([...readFileSync(file, 'utf8').matchAll(/\sid="([^"]+)"/g)].map((m) => m[1])));
  return ids.get(file);
};
let links = 0;
let anchors = 0;
const walk = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walk(join(dir, e.name)) : e.name.endsWith('.html') ? [join(dir, e.name)] : [],
  );
for (const page of walk(out)) {
  for (const [, href, hash] of readFileSync(page, 'utf8').matchAll(/href="(\/[^"#?]*)(?:\?[^"#]*)?(?:#([^"]*))?"/g)) {
    links++;
    if (/^\/(p|api)\//.test(href)) continue;
    const p = join(out, decodeURIComponent(href).replace(/\/$/, ''));
    const file = [p, `${p}.html`, join(p, 'index.html')].find((f) => statSync(f, { throwIfNoEntry: false })?.isFile());
    let target = file ? null : href;
    if (file && hash && file.endsWith('.html')) {
      anchors++;
      if (!idsOf(file).has(decodeURIComponent(hash))) target = `${href}#${hash}`;
    }
    if (!target) continue;
    if (!broken.has(target)) broken.set(target, []);
    broken.get(target).push('/' + relative(out, page));
  }
}
for (const [href, pages] of broken) console.error(`links: ${href} <- ${pages.length} pages, e.g. ${pages[0]}`);
console.log(`links: ${links} internal links (${anchors} to an anchor), ${broken.size} broken targets`);
process.exit(broken.size ? 1 : 0);
