// Enforces site/R2 (handbook/guidelines/site-checklist.md): /llms-full.txt, the
// docs as one file for a model, holds every docs and Learn page the site
// exports. A page missing from it is one a model can't write code from.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const out = join(import.meta.dirname, '..', 'out');
const pagesUnder = (dir, url) =>
  readdirSync(join(out, dir), { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? pagesUnder(join(dir, e.name), `${url}/${e.name}`) : e.name === 'index.html' ? [url] : [],
  );
const full = readFileSync(join(out, 'llms-full.txt'), 'utf8');
const listed = new Set([...full.matchAll(/^URL: (\S+)$/gm)].map((m) => m[1]));
const pages = ['docs', 'learn'].flatMap((d) => pagesUnder(d, `/${d}`));
// Pages drawn from code rather than MDX, not yet rendered into the bundle; this list only shrinks.
const notYet = ['/learn/start', '/learn/from/cpp', '/learn/from/java', '/learn/from/python', '/learn/from/rust'];
const missing = pages.filter((u) => !listed.has(u) && !notYet.includes(u));
const stale = notYet.filter((u) => listed.has(u));
if (stale.length) {
  console.error(`llms-complete: now in /llms-full.txt, delete from the not-yet list: ${stale.join(', ')}`);
  process.exit(1);
}
if (missing.length) {
  console.error(`llms-complete: ${missing.length} of ${pages.length} pages are not in /llms-full.txt (site/R2):\n${missing.join('\n')}`);
  process.exit(1);
}
console.log(`llms-complete: ${pages.length - notYet.length} of ${pages.length} docs and Learn pages are in /llms-full.txt; ${notYet.length} drawn from code are not yet`);
