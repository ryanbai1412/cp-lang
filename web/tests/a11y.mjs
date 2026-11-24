// Enforces site/R14 (handbook/guidelines/site-checklist.md): every route a
// visitor lands on passes axe-core's WCAG 2 A and AA rules. Violations already
// known are listed in tests/a11y-known.txt as `route rule`; that list only shrinks.
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';
import { base, done, open } from './browser.mjs';

const routes = ['/', '/docs/guides/tour', '/docs/ref/std/algebra', '/docs/ref/std/segtree', '/learn', '/learn/start', '/learn/intro', '/learn/read/standard-library', '/learn/practice/prime-count', '/play', '/corpus', '/tools', '/performance', '/setup', '/lints', '/lints/search_loop', '/errors', '/errors/E0301', '/decisions'];
const knownFile = new URL('./a11y-known.txt', import.meta.url);
const known = new Set(readFileSync(knownFile, 'utf8').split('\n').map((l) => l.trim()).filter((l) => l && !l.startsWith('#')));

const { page } = await open();
const found = new Map();
for (const route of routes) {
  await page.goto(base + route, { waitUntil: 'networkidle' });
  const { violations } = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  for (const v of violations) found.set(`${route} ${v.id}`, `${v.impact}: ${v.help} (${v.nodes.length}x, e.g. ${v.nodes[0].target.join(' ')})`);
}
const fresh = [...found].filter(([k]) => !known.has(k));
const fixed = [...known].filter((k) => !found.has(k));
for (const [k, v] of fresh) console.error(`a11y: ${k}: ${v}`);
if (fixed.length) console.error(`a11y: fixed now, delete from tests/a11y-known.txt:\n${fixed.join('\n')}`);
if (process.env.A11Y_LIST) console.log([...found.keys()].join('\n'));
if (fresh.length || fixed.length) done(1);
console.log(`a11y: ${routes.length} routes, ${found.size} known violations left`);
done(0);
