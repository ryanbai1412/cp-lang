// Enforces site/R14 (handbook/guidelines/site-checklist.md): every route a
// visitor lands on passes axe-core's WCAG 2 A and AA rules. Any violation fails;
// there is no list of accepted ones.
import AxeBuilder from '@axe-core/playwright';
import { base, done, open } from './browser.mjs';

const routes = ['/', '/docs/guides/tour', '/docs/ref/std/algebra', '/docs/ref/std/segtree', '/learn', '/learn/start', '/learn/intro', '/learn/read/standard-library', '/learn/practice/prime-count', '/play', '/corpus', '/tools', '/performance', '/setup', '/lints', '/lints/search_loop', '/errors', '/errors/E0301', '/decisions'];

const { page } = await open();
const found = new Map();
for (const route of routes) {
  await page.goto(base + route, { waitUntil: 'networkidle' });
  const { violations } = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  for (const v of violations) found.set(`${route} ${v.id}`, `${v.impact}: ${v.help} (${v.nodes.length}x, e.g. ${v.nodes[0].target.join(' ')})`);
}
for (const [k, v] of found) console.error(`a11y: ${k}: ${v}`);
if (found.size) done(1);
console.log(`a11y: ${routes.length} routes, no violations`);
done(0);
