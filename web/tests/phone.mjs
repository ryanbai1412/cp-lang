// Phone pass (probe P4, mechanical half): loads each route at 390 px wide and
// lists what sticks out past the viewport, outside any element that scrolls
// sideways on purpose. Run with `pnpm --dir web phone`.
import { base, open } from './browser.mjs';

const routes = ['/', '/docs/guides/tour', '/docs/ref/std/algebra', '/learn', '/learn/start', '/play', '/corpus', '/tools', '/performance', '/setup'];
const { page, errors } = await open();
await page.setViewportSize({ width: 390, height: 844 });
let bad = 0;
for (const path of routes) {
  await page.goto(base + path, { waitUntil: 'networkidle' });
  const found = await page.evaluate(() => {
    const width = document.documentElement.clientWidth;
    const scrolls = (e) => {
      for (let p = e.parentElement; p && p !== document.body; p = p.parentElement) if (/auto|scroll|hidden|clip/.test(getComputedStyle(p).overflowX)) return true;
      return false;
    };
    const out = [];
    for (const e of document.body.querySelectorAll('*')) {
      const r = e.getBoundingClientRect();
      if (r.width && r.right > width + 1 && !scrolls(e) && !out.some((o) => o.node.contains(e))) out.push({ node: e, right: Math.round(r.right) });
    }
    const name = (e) => e.tagName.toLowerCase() + (e.id ? `#${e.id}` : '') + (typeof e.className === 'string' && e.className ? `.${e.className.split(/\s+/).slice(0, 2).join('.')}` : '');
    return { page: document.documentElement.scrollWidth - width, items: out.map((o) => `${name(o.node)} ${o.right}px`) };
  });
  bad += found.page > 0;
  console.log(`phone: ${path} ${found.page > 0 ? `scrolls ${found.page}px sideways` : 'fits'}${found.items.length ? `; past the edge: ${found.items.slice(0, 5).join(', ')}` : ''}`);
}
console.log(`phone: ${routes.length - bad}/${routes.length} routes fit 390 px`);
await page.close();
process.exit(errors.length ? 1 : 0);
